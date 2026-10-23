"""
Diagnostic for prop (0x00B1B104) parse failures.

Goal: understand WHY the prop walker desynchronises on the ~313 resources that
cluster in groups 0x0001FFFF and 0x00010000.

Strategy: for a failing resource, re-walk item by item, and at each step print
the bytes the parser is about to read plus the interpretation it would give.
That shows exactly where the layout first diverges from the documented rules.

Outputs: a handful of full hex dumps of failing resources, plus a per-item trace
for the first record of each.
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress  # noqa: E402

REC = 28
PROP_TYPE = 0x00B1B104
ARRAY_SPECIFIERS = frozenset({0x009C, 0x001C})

FIXED = {
    0x09: 4, 0x0a: 4, 0x0d: 4,
    0x30: 8, 0x31: 12, 0x32: 12, 0x33: 16, 0x34: 16,
    0x38: 64, 0x39: 24,
}
SINGLE_PADDING = {0x30: 4, 0x31: 4, 0x32: 4, 0x33: 4, 0x38: 4, 0x39: 4}
KEY_SINGLE_SIZE = 16
KEY_ARRAY_STRIDE = 12


def parse_index(path: Path):
    with path.open("rb") as fh:
        head = fh.read(0x60)
        count = struct.unpack_from("<I", head, 0x24)[0]
        idx_off = struct.unpack_from("<I", head, 0x40)[0]
        fh.seek(idx_off + PREFIX)
        blob = fh.read(count * REC)
    out = []
    for i in range(count):
        g, t, ihi, ilo, off, sz, dec = struct.unpack_from("<IIIIIII", blob, i * REC)
        if t == PROP_TYPE:
            out.append((g, off, sz & 0x7FFFFFFF, dec))
    return out


PREFIX = 4


def hexdump(b: bytes, start=0, n=96):
    out = []
    for i in range(0, n, 16):
        chunk = b[start + i:start + i + 16]
        if not chunk:
            break
        hexs = " ".join(f"{x:02x}" for x in chunk)
        asc = "".join(chr(x) if 32 <= x < 127 else "." for x in chunk)
        out.append(f"{start + i:06x}  {hexs:<47}  {asc}")
    return "\n".join(out)


def trace(d: bytes, max_items=8):
    """Walk, printing each item's raw interpretation, until desync or end."""
    lines = []
    p = 0
    try:
        count, p = struct.unpack_from(">I", d, p)[0], p + 4
    except struct.error:
        return ["(cannot read header count)"]
    lines.append(f"item count (BE) = {count}")
    if count > 5000:
        lines.append(f"  !! implausible count {count} -> desync at header")
        return lines
    for k in range(count):
        if p + 8 > len(d):
            lines.append(f"  item {k}: need 8 bytes at {p}, only {len(d) - p} left")
            break
        ident, typ, spec = struct.unpack_from(">IHH", d, p)
        if typ in (0x12, 0x13):
            n = struct.unpack_from(">I", d, p + 8)[0]
            lines.append(f"  item {k} @ {p:04x}: id={ident:08x} type={typ:04x} spec={spec:04x} -> STRING len={n}")
        elif spec in ARRAY_SPECIFIERS:
            n, stride = struct.unpack_from(">II", d, p + 8)
            lines.append(f"  item {k} @ {p:04x}: id={ident:08x} type={typ:04x} spec={spec:04x} -> ARRAY n={n} stride={stride}")
        else:
            lines.append(f"  item {k} @ {p:04x}: id={ident:08x} type={typ:04x} spec={spec:04x} (scalar)")
        if k >= max_items - 1:
            lines.append(f"  ... ({count - max_items} more items)")
            break
    return lines


def main() -> int:
    pkgs = sys.argv[1:] or [r"D:\Games\SPORE Collection\Data\Spore_Game.package"]
    fail_groups = Counter()
    dumps = []  # (group, d, reason)
    desync = 0
    under = 0
    for arg in pkgs:
        path = Path(arg)
        for g, off, stored, dec in parse_index(path):
            with path.open("rb") as fh:
                fh.seek(off)
                raw = fh.read(stored)
            try:
                d = decompress(raw, out_size=None, header=True)[0] if dec > stored else raw
            except Exception:  # noqa: BLE001
                continue
            # quick walk to detect failure
            ok, reason, _at = quick(d)
            if not ok:
                fail_groups[g] += 1
                is_under = reason.startswith("ended at")
                if is_under and under < 4 and len(dumps) < 10:
                    dumps.append((g, d, reason))
                    under += 1
                elif not is_under and desync < 5 and len(dumps) < 10:
                    dumps.append((g, d, reason))
                    desync += 1
    print("FAILURE GROUPS:")
    for g, n in fail_groups.most_common():
        print(f"  0x{g:08X}  {n}")
    print()
    for g, d, reason in dumps:
        print(f"=== group 0x{g:08X} len={len(d)} reason={reason} ===")
        print(hexdump(d, 0, min(112, len(d))))
        # If the failure reports a divergence offset, dump around it.
        if "at " in reason:
            try:
                at = int(reason.split("at ")[1].split()[0])
                if at < len(d):
                    print(f"--- bytes around divergence @ {at} ---")
                    print(hexdump(d, max(0, at - 16), min(64, len(d) - max(0, at - 16))))
            except ValueError:
                pass
        # If under-consumed, dump the trailing bytes.
        if reason.startswith("ended at"):
            try:
                at = int(reason.split("ended at ")[1].split()[0])
                print(f"--- trailing bytes from {at} (count={len(d)-at}) ---")
                print(hexdump(d, at, min(96, len(d) - at)))
            except ValueError:
                pass
        print()
    return 0


def scalar_size(typ, spec):
    if typ == 0x01:
        return 1
    if typ in FIXED:
        return FIXED[typ] + (0 if spec in ARRAY_SPECIFIERS else SINGLE_PADDING.get(typ, 0))
    if typ == 0x20:
        return 16
    if typ == 0x00:
        return spec
    return None


def quick(d: bytes, dump_at=None):
    """Return (ok, reason) by walking with the same rules as prop_parse.py.

    `dump_at`, if given, is the failure offset; the caller uses it to pull the
    bytes around the divergence.
    """
    p = 0
    try:
        count, p = struct.unpack_from(">I", d, p)[0], p + 4
    except struct.error:
        return False, "no header", p
    if count > 5000:
        return False, f"implausible count {count}", p
    for _ in range(count):
        if p + 8 > len(d):
            return False, f"item header past end at {p}", p
        ident, typ, spec = struct.unpack_from(">IHH", d, p)
        p += 8
        if spec in ARRAY_SPECIFIERS:
            if p + 8 > len(d):
                return False, f"array header past end at {p}", p
            n, stride = struct.unpack_from(">II", d, p)
            p += 8
            if n > 100_000 or stride > 8192:
                return False, f"array {n}x{stride} implausible", p
            if p + n * stride > len(d):
                return False, f"array span overruns at {p}", p
            p += n * stride
        elif typ in (0x12, 0x13):
            if p + 4 > len(d):
                return False, "string len past end", p
            n = struct.unpack_from(">I", d, p)[0]
            p += 4
            if p + n * (1 if typ == 0x12 else 2) > len(d):
                return False, f"string overruns at {p}", p
            p += n * (1 if typ == 0x12 else 2)
        elif typ == 0x22:
            if p + 4 > len(d):
                return False, "texts count past end", p
            n = struct.unpack_from(">I", d, p)[0]
            p += 4
            for _ in range(n):
                if p + 12 > len(d):
                    return False, "texts entry past end", p
                sz, _f2, _f3 = struct.unpack_from(">III", d, p)
                p += 12
                if p + sz > len(d):
                    return False, f"texts buffer overruns at {p} (sz={sz})", p
                p += sz
        else:
            n = scalar_size(typ, spec)
            if n is None:
                return False, f"unhandled type 0x{typ:04x} at {p - 8} (hdr bytes {d[p-8:p].hex()})", p - 8
            if p + n > len(d):
                return False, f"scalar type {typ:04x} needs {n} overruns at {p}", p
            p += n
    if p == len(d):
        return True, "", p
    return False, f"ended at {p} != {len(d)}", p


if __name__ == "__main__":
    raise SystemExit(main())

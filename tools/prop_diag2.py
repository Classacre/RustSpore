"""
Test harness for the prop variant layout.

Hypothesis (measured from hex dumps):
  Some prop resources (groups 0x0001FFFF, 0x00010000) contain a 4-byte separator
  value `3c c4 12 00` (LE 0x0012C43C) at item boundaries, and/or trailing data after
  the nominal `count` items. The standard walker desyncs on the marker.

This script walks with a tolerant strategy and measures exact-consumption:
  - at each item boundary, if the next 4 bytes equal the marker, skip them
  - do NOT stop at the nominal `count`; keep reading items until the end
  - still validate that each item's type is known and sizes are sane
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress  # noqa: E402

REC = 28
PREFIX = 4
PROP_TYPE = 0x00B1B104

ARRAY_SPECIFIERS = frozenset({0x009C, 0x001C})
FIXED = {0x09: 4, 0x0a: 4, 0x0d: 4, 0x30: 8, 0x31: 12, 0x32: 12, 0x33: 16, 0x34: 16, 0x38: 64, 0x39: 24}
SINGLE_PADDING = {0x30: 4, 0x31: 4, 0x32: 4, 0x33: 4, 0x38: 4, 0x39: 4}


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


MARKER = b"\x3c\xc4\x12\x00"


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


def tolerant_walk(d: bytes):
    """Walk tolerantly. Returns (ok, items_read, end, reason)."""
    p = 0
    try:
        _count, p = struct.unpack_from(">I", d, p)[0], p + 4
    except struct.error:
        return False, 0, p, "no header"
    items = 0
    while p + 8 <= len(d):
        # skip separator marker if present
        if d[p:p + 4] == MARKER:
            p += 4
            continue
        ident, typ, spec = struct.unpack_from(">IHH", d, p)
        p += 8
        if spec in ARRAY_SPECIFIERS:
            if p + 8 > len(d):
                return False, items, p, f"array hdr past end @ {p}"
            n, stride = struct.unpack_from(">II", d, p)
            p += 8
            if n > 200_000 or stride > 16384:
                return False, items, p - 8, f"array {n}x{stride} implausible @ {p-8} (id={ident:08x})"
            if p + n * stride > len(d):
                return False, items, p, f"array span overruns @ {p} (n={n} stride={stride})"
            p += n * stride
        elif typ in (0x12, 0x13):
            if p + 4 > len(d):
                return False, items, p, "str len past end"
            n = struct.unpack_from(">I", d, p)[0]
            p += 4
            need = n * (1 if typ == 0x12 else 2)
            if p + need > len(d):
                return False, items, p, f"string overruns @ {p} (n={n})"
            p += need
        elif typ == 0x22:
            if p + 4 > len(d):
                return False, items, p, "texts count past end"
            n = struct.unpack_from(">I", d, p)[0]
            p += 4
            for _ in range(n):
                if p + 12 > len(d):
                    return False, items, p, "texts entry past end"
                sz, _f2, _f3 = struct.unpack_from(">III", d, p)
                p += 12
                if p + sz > len(d):
                    return False, items, p, f"texts buf overruns @ {p} (sz={sz})"
                p += sz
        else:
            n = scalar_size(typ, spec)
            if n is None:
                return False, items, p - 8, f"unhandled type 0x{typ:04x} @ {p-8} (id={ident:08x})"
            if p + n > len(d):
                return False, items, p, f"scalar {typ:04x} needs {n} overruns @ {p}"
            p += n
        items += 1
    # trailing marker: some resources end with a lone 3c c4 12 00 separator
    if d[p:] == MARKER:
        p += 4
    if p == len(d):
        return True, items, p, ""
    return False, items, p, f"ended at {p} != {len(d)}"


def main() -> int:
    pkgs = sys.argv[1:] or [r"D:\Games\SPORE Collection\Data\Spore_Game.package"]
    exact = under = over = failed = total = 0
    fail_reasons = Counter()
    fail_groups = Counter()
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
            total += 1
            ok, items, end, reason = tolerant_walk(d)
            if ok:
                exact += 1
            elif end < len(d):
                under += 1
                fail_reasons[reason] += 1
                fail_groups[g] += 1
            else:
                over += 1
                fail_reasons[reason] += 1
                fail_groups[g] += 1
    print(f"TOLERANT WALK over {total} prop resources:")
    print(f"  exact      : {exact}  ({100.0*exact/max(total,1):.2f}%)")
    print(f"  under      : {under}")
    print(f"  over       : {over}")
    if fail_reasons:
        print("  failure reasons:")
        for r, n in fail_reasons.most_common(12):
            print(f"     {n:>6}  {r}")
        print("  failure groups:")
        for g, n in fail_groups.most_common(8):
            print(f"     0x{g:08X}  {n}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

"""Find boundaries where the naive ANONYMOUS-FIRST rule fails, to learn the real
named/anonymous dispatch rule. Dumps hex context at the first divergence."""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress  # noqa: E402

REC = 28
PREFIX = 4
PROP_TYPE = 0x00B1B104
ARRAY_SPECIFIERS = frozenset({0x009C, 0x001C})
FIXED = {0x09: 4, 0x0a: 4, 0x0d: 4, 0x30: 8, 0x31: 12, 0x32: 12, 0x33: 16, 0x34: 16, 0x38: 64, 0x39: 24}
KNOWN_SPECS = frozenset({0x0000, 0x001C, 0x009C} | {t for t in FIXED})
KNOWN_TYPES = frozenset(set(FIXED) | {0x01, 0x20, 0x00, 0x12, 0x13, 0x22})
SEP_LOW_WORDS = (b"\x12\x00", b"\xa0\x00")
CONST_SEP = b"\x01\x00\x00\x00"
KEY_ARRAY_STRIDE = 12


def parse_index(path):
    with path.open("rb") as fh:
        head = fh.read(0x60)
        count = struct.unpack_from("<I", head, 0x24)[0]
        idx_off = struct.unpack_from("<I", head, 0x40)[0]
        fh.seek(idx_off + PREFIX)
        avail = max(0, fh.seek(0, 2) - (idx_off + PREFIX))
        fh.seek(idx_off + PREFIX)
        blob = fh.read(min(count * REC, avail))
    n = len(blob) // REC
    return [(g, off, sz & 0x7FFFFFFF, dec) for (g, t, ihi, ilo, off, sz, dec)
            in [struct.unpack_from("<IIIIIII", blob, i * REC) for i in range(n)]
            if t == PROP_TYPE]


def is_separator(d, p):
    if d[p:p + 4] == CONST_SEP:
        return True
    return d[p + 2:p + 4] in SEP_LOW_WORDS


def value_size(d, p, typ, spec):
    if spec in ARRAY_SPECIFIERS:
        if p + 8 > len(d):
            return None
        n, stride = struct.unpack_from(">II", d, p)
        if n > 200_000 or stride > 16384:
            return None
        if typ in (0x12, 0x13):
            pp = p + 8
            w = 1 if typ == 0x12 else 2
            for _ in range(n):
                if pp + 4 > len(d):
                    return None
                ln = struct.unpack_from(">I", d, pp)[0]
                pp += 4 + ln * w
                if pp > len(d):
                    return None
            return pp - p
        if p + 8 + n * stride > len(d):
            return None
        return 8 + n * stride
    if typ in (0x12, 0x13):
        if p + 4 > len(d):
            return None
        ln = struct.unpack_from(">I", d, p)[0]
        w = 1 if typ == 0x12 else 2
        if p + 4 + ln * w > len(d):
            return None
        return 4 + ln * w
    if typ == 0x22:
        if p + 4 > len(d):
            return None
        n = struct.unpack_from(">I", d, p)[0]
        pp = p + 4
        for _ in range(n):
            if pp + 12 > len(d):
                return None
            sz, _, _ = struct.unpack_from(">III", d, pp)
            pp += 12
            if pp + sz > len(d):
                return None
            pp += sz
        return pp - p
    if typ == 0x01:
        return 1
    if typ == 0x20:
        return KEY_ARRAY_STRIDE if spec in ARRAY_SPECIFIERS else 16
    if typ == 0x00:
        return spec if 0 <= spec <= 65535 else None
    if typ in FIXED:
        base = FIXED[typ]
        if spec in ARRAY_SPECIFIERS:
            return base
        if typ in (0x33, 0x34):
            return base
        return base + 4
    return None


def walk_anon_first(d):
    p = 4
    while p + 8 <= len(d):
        if is_separator(d, p):
            p += 4
            continue
        t0 = struct.unpack_from(">H", d, p)[0]
        if t0 in KNOWN_TYPES:
            typ, spec = struct.unpack_from(">HH", d, p)
            if spec in KNOWN_SPECS:
                vsz = value_size(d, p + 4, typ, spec)
                if vsz is None or p + 4 + vsz > len(d):
                    return False, p
                p += 4 + vsz
                continue
        ident, typ, spec = struct.unpack_from(">IHH", d, p)
        vsz = value_size(d, p + 8, typ, spec)
        if vsz is None or p + 8 + vsz > len(d):
            return False, p
        p += 8 + vsz
    if is_separator(d, p):
        p += 4
    return (p == len(d)), p


def main():
    pkgs = sys.argv[1:]
    limit = int(sys.argv[-1]) if sys.argv[-1].isdigit() else 6
    shown = 0
    for arg in pkgs:
        path = Path(arg)
        for g, off, stored, dec in parse_index(path):
            with path.open("rb") as fh:
                fh.seek(off); raw = fh.read(stored)
            try:
                d = decompress(raw, out_size=None, header=True)[0] if dec > stored else raw
            except Exception:
                continue
            ok, dp = walk_anon_first(d)
            if ok:
                continue
            print(f"\n##### {path.name} group=0x{g:08x} len={len(d)} desync@{dp} #####")
            lo = max(0, dp - 40)
            for i in range(lo, min(len(d), dp + 24), 16):
                chunk = d[i:i + 16]
                mark = "<<" if i <= dp < i + 16 else "  "
                print(f"   {mark} {i:04x}  " + " ".join(f"{b:02x}" for b in chunk) + "   " + "".join(chr(b) if 32 <= b < 127 else "." for b in chunk))
            shown += 1
            if shown >= limit:
                return 0
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

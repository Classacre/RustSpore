"""Full-walk tracer: dumps every step of the robust walk for one prop."""

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
SINGLE_PADDING = {0x30: 4, 0x31: 4, 0x32: 4, 0x33: 4, 0x38: 4, 0x39: 4}
KNOWN_SPECS = frozenset({0x0000, 0x001C, 0x009C} | set(FIXED))
KNOWN_TYPES = frozenset(set(FIXED) | {0x01, 0x20, 0x00, 0x12, 0x13, 0x22})
SEP_LOW_WORDS = (b"\x12\x00", b"\xa0\x00")
CONST_SEP = b"\x01\x00\x00\x00"


def parse_index(path):
    with path.open("rb") as fh:
        head = fh.read(0x60)
        count = struct.unpack_from("<I", head, 0x24)[0]
        idx_off = struct.unpack_from("<I", head, 0x40)[0]
        fh.seek(idx_off + PREFIX)
        blob = fh.read(count * REC)
    return [(g, off, sz & 0x7FFFFFFF, dec) for (g, t, ihi, ilo, off, sz, dec)
            in [struct.unpack_from("<IIIIIII", blob, i * REC) for i in range(count)]
            if t == PROP_TYPE]


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


def entry_size(d, p, typ, spec):
    if spec in ARRAY_SPECIFIERS:
        if p + 8 > len(d):
            return None
        n, stride = struct.unpack_from(">II", d, p)
        if n > 200_000 or stride > 16384 or p + 8 + n * stride > len(d):
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
    sz = scalar_size(typ, spec)
    if sz is None or p + sz > len(d):
        return None
    return sz


def is_separator(d, p):
    if d[p:p + 4] == CONST_SEP:
        return True
    return d[p + 2:p + 4] in SEP_LOW_WORDS


def main():
    pkg = Path(sys.argv[1])
    target_group = int(sys.argv[2], 16) if len(sys.argv) > 2 else None
    max_d = int(sys.argv[3]) if len(sys.argv) > 3 else None
    found = False
    for g, off, stored, dec in parse_index(pkg):
        if target_group is not None and g != target_group:
            continue
        with pkg.open("rb") as fh:
            fh.seek(off); raw = fh.read(stored)
        try:
            d = decompress(raw, out_size=None, header=True)[0] if dec > stored else raw
        except Exception:
            continue
        if max_d is not None and len(d) > max_d:
            continue
        # preview: detect desync quickly
        pp = 0
        cnt0, pp = struct.unpack_from(">I", d, pp)[0], pp + 4
        ok = True
        while pp + 8 <= len(d):
            if is_separator(d, pp):
                pp += 4; continue
            ident, typ, spec = struct.unpack_from(">IHH", d, pp)
            sz = entry_size(d, pp + 8, typ, spec)
            if sz is not None:
                pp += 8 + sz; continue
            typ2, spec2 = struct.unpack_from(">HH", d, pp)
            sz2 = entry_size(d, pp + 4, typ2, spec2)
            if typ2 in KNOWN_TYPES and sz2 is not None and spec2 in KNOWN_SPECS:
                pp += 4 + sz2; continue
            ok = False; break
        if is_separator(d, pp):
            pp += 4
        if not ok or pp != len(d):
            # this is a desyncing (or non-exact) prop; dump it
            pass
        else:
            continue
        p = 0
        cnt, p = struct.unpack_from(">I", d, p)[0], p + 4
        print(f"\n##### group=0x{g:08x} len(d)={len(d)} header_count={cnt} #####")
        step = 0
        while p + 8 <= len(d):
            if is_separator(d, p):
                print(f"  {step:3d} @{p:4d} SEP {d[p:p+4].hex()}")
                p += 4; step += 1; continue
            ident, typ, spec = struct.unpack_from(">IHH", d, p)
            sz = entry_size(d, p + 8, typ, spec)
            if sz is not None:
                tag = "NAMED"
                adv = 8 + sz
            else:
                typ2, spec2 = struct.unpack_from(">HH", d, p)
                sz2 = entry_size(d, p + 4, typ2, spec2)
                if typ2 in KNOWN_TYPES and sz2 is not None and spec2 in KNOWN_SPECS:
                    tag = "ANON"
                    ident = None
                    adv = 4 + sz2
                else:
                    print(f"  {step:3d} @{p:4d} DESYNC named id=0x{ident:08x} type=0x{typ:04x} spec=0x{spec:04x}; anon type=0x{typ2:04x} spec=0x{spec2:04x}")
                    lo = max(0, p - 64)
                    for i in range(lo, p + 16, 16):
                        chunk = d[i:i + 16]
                        mark = "<<" if i <= p < i + 16 else "  "
                        print(f"      {mark} {i:04x}  " + " ".join(f"{b:02x}" for b in chunk) + "   " + "".join(chr(b) if 32 <= b < 127 else "." for b in chunk))
                    break
            print(f"  {step:3d} @{p:4d} {tag} id={ident if ident is None else f'0x{ident:08x}'} type=0x{typ:04x} spec=0x{spec:04x} size={sz} adv={adv}")
            p += adv; step += 1
        if is_separator(d, p):
            print(f"  trailing SEP {d[p:p+4].hex()}")
            p += 4
        print(f"  END p={p} len={len(d)} exact={p==len(d)}")
        break
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

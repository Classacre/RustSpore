"""Find a prop in a package that desyncs at a given offset and trace it fully."""

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
SEPARATORS = (b"\x3c\xc4\x12\x00", b"\xe4\x0c\xa0\x00")
KNOWN_SPECS = frozenset({0x0000, 0x001C, 0x009C} | {t for t in FIXED})
TNAMES = {0x01: "bool", 0x09: "int32", 0x0a: "uint32", 0x0d: "float", 0x12: "str8",
          0x13: "str16", 0x20: "key", 0x22: "texts", 0x30: "vec2", 0x31: "vec3",
          0x32: "colRGB", 0x33: "vec4", 0x34: "colRGBA", 0x38: "xfrm", 0x39: "bbox", 0x00: "raw"}


def parse_index(path: Path):
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


def walk_desync(d):
    p = 0
    _c, p = struct.unpack_from(">I", d, p)[0], p + 4
    while p + 8 <= len(d):
        if d[p:p + 4] in SEPARATORS:
            p += 4
            continue
        ident, typ, spec = struct.unpack_from(">IHH", d, p)
        sz = entry_size(d, p + 8, typ, spec)
        if sz is None:
            typ2, spec2 = struct.unpack_from(">HH", d, p)
            sz2 = entry_size(d, p + 4, typ2, spec2)
            if typ2 in (0x01, 0x20, 0x00, 0x12, 0x13, 0x22) or typ2 in FIXED:
                if sz2 is not None and spec2 in KNOWN_SPECS:
                    p += 4 + sz2
                    continue
            return p
        p += 8 + sz
    return None


def trace(d):
    p = 0
    count, p = struct.unpack_from(">I", d, p)[0], p + 4
    print(f"count (BE)={count} len={len(d)}")
    n = 0
    while p + 8 <= len(d):
        if d[p:p + 4] in SEPARATORS:
            print(f"  [{n:3d}] @ {p:04x} SEPARATOR skipped")
            p += 4
            continue
        ident, typ, spec = struct.unpack_from(">IHH", d, p)
        sz = entry_size(d, p + 8, typ, spec)
        if sz is None:
            typ2, spec2 = struct.unpack_from(">HH", d, p)
            sz2 = entry_size(d, p + 4, typ2, spec2)
            if typ2 in (0x01, 0x20, 0x00, 0x12, 0x13, 0x22) or typ2 in FIXED:
                if sz2 is not None and spec2 in KNOWN_SPECS:
                    print(f"  [{n:3d}] @ {p:04x} ANON type={typ2:04x}({TNAMES.get(typ2,'?')}) spec={spec2:04x} -> {p+4+sz2:04x}")
                    p += 4 + sz2
                    n += 1
                    continue
            print(f"  [{n:3d}] @ {p:04x} id={ident:08x} DESYNC type={typ:04x} spec={spec:04x}")
            print("        bytes: " + " ".join(f"{x:02x}" for x in d[p:p + 40]))
            break
        print(f"  [{n:3d}] @ {p:04x} id={ident:08x} {TNAMES.get(typ,'?')}({typ:04x}) spec={spec:04x} -> {p+8+sz:04x}")
        p += 8 + sz
        n += 1
    if p < len(d) and d[p:] not in SEPARATORS:
        print(f"  TRAILING {len(d)-p}: " + " ".join(f"{x:02x}" for x in d[p:]))


def main():
    pkg = sys.argv[1]
    target = int(sys.argv[2]) if len(sys.argv) > 2 else 48
    path = Path(pkg)
    for g, off, stored, dec in parse_index(path):
        with path.open("rb") as fh:
            fh.seek(off); raw = fh.read(stored)
        try:
            d = decompress(raw, out_size=None, header=True)[0] if dec > stored else raw
        except Exception:
            continue
        dp = walk_desync(d)
        if dp == target:
            print(f"=== group 0x{g:08X} len={len(d)} off={off} ===")
            trace(d)
            print(f"--- hex around {target} ---")
            print("  " + " ".join(f"{x:02x}" for x in d[target - 16:target + 48]))
            return
    print(f"no prop desyncing exactly at {target}")


if __name__ == "__main__":
    main()

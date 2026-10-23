"""Precise item-by-item tracer for a single failing prop, found by group+length."""

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
MARKER = b"\x3c\xc4\x12\x00"

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


def trace(d: bytes):
    p = 0
    count, p = struct.unpack_from(">I", d, p)[0], p + 4
    print(f"count (BE) = {count}  len={len(d)}")
    n = 0
    while p + 8 <= len(d):
        if d[p:p + 4] == MARKER:
            print(f"  [{n:3d}] @ {p:04x}  >>> MARKER 3c c4 12 00 skipped")
            p += 4
            continue
        ident, typ, spec = struct.unpack_from(">IHH", d, p)
        p2 = p + 8
        if spec in ARRAY_SPECIFIERS:
            n2, stride = struct.unpack_from(">II", d, p2)
            span = n2 * stride
            print(f"  [{n:3d}] @ {p:04x} id={ident:08x} type={typ:04x}({TNAMES.get(typ,'?')}) "
                  f"spec={spec:04x} ARRAY n={n2} stride={stride} span={span} -> end {p2+8+span:04x}")
            p = p2 + 8 + span
        elif typ in (0x12, 0x13):
            ln = struct.unpack_from(">I", d, p2)[0]
            w = 1 if typ == 0x12 else 2
            print(f"  [{n:3d}] @ {p:04x} id={ident:08x} type={typ:04x}({TNAMES[typ]}) "
                  f"spec={spec:04x} str len={ln} -> end {p2+4+ln*w:04x}")
            p = p2 + 4 + ln * w
        elif typ == 0x22:
            n2 = struct.unpack_from(">I", d, p2)[0]
            pp = p2 + 4
            tot = 0
            for _ in range(n2):
                sz, _, _ = struct.unpack_from(">III", d, pp)
                pp += 12 + sz
                tot += sz
            print(f"  [{n:3d}] @ {p:04x} id={ident:08x} type={typ:04x}(texts) "
                  f"spec={spec:04x} entries={n2} -> end {pp:04x}")
            p = pp
        else:
            sz = scalar_size(typ, spec)
            if sz is None:
                print(f"  [{n:3d}] @ {p:04x} id={ident:08x} type={typ:04x}(UNHANDLED) "
                      f"spec={spec:04x}  >>> DESYNC. next 32 bytes:")
                print("        " + " ".join(f"{x:02x}" for x in d[p:p + 32]))
                break
            print(f"  [{n:3d}] @ {p:04x} id={ident:08x} type={typ:04x}({TNAMES.get(typ,'?')}) "
                  f"spec={spec:04x} size={sz} -> end {p2+sz:04x}")
            p = p2 + sz
        n += 1
    if p < len(d):
        rem = len(d) - p
        tail = d[p:]
        print(f"  TRAILING {rem} bytes: " + " ".join(f"{x:02x}" for x in tail))
        if rem >= 4:
            print(f"    as LE u32: {struct.unpack_from('<I', tail, 0)[0]:#010x}")


def main():
    pkg = sys.argv[1]
    target_group = int(sys.argv[2], 0) if len(sys.argv) > 2 else 0x0001FFFF
    target_len = int(sys.argv[3]) if len(sys.argv) > 3 else 175
    path = Path(pkg)
    for g, off, stored, dec in parse_index(path):
        if g != target_group:
            continue
        with path.open("rb") as fh:
            fh.seek(off); raw = fh.read(stored)
        try:
            d = decompress(raw, out_size=None, header=True)[0] if dec > stored else raw
        except Exception:
            continue
        if len(d) != target_len:
            continue
        print(f"=== group 0x{g:08X} len={len(d)} off={off} ===")
        trace(d)
        return
    print(f"no prop with group 0x{target_group:08X} and len {target_len} found")


if __name__ == "__main__":
    main()

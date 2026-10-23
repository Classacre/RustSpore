"""Trace the first failing prop (len < limit) in a package, full item-by-item."""

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
            print(f"  [{n:3d}] @ {p:04x}  >>> MARKER skipped")
            p += 4
            continue
        ident, typ, spec = struct.unpack_from(">IHH", d, p)
        p2 = p + 8
        if spec in ARRAY_SPECIFIERS:
            n2, stride = struct.unpack_from(">II", d, p2)
            print(f"  [{n:3d}] @ {p:04x} id={ident:08x} ARRAY type={typ:04x} n={n2} stride={stride}")
            p = p2 + 8 + n2 * stride
        elif typ in (0x12, 0x13):
            ln = struct.unpack_from(">I", d, p2)[0]
            w = 1 if typ == 0x12 else 2
            raw = d[p2 + 4:p2 + 4 + ln * w]
            try:
                s = raw.decode("utf-16-le" if typ == 0x13 else "latin-1")
            except Exception:
                s = repr(raw[:20])
            print(f"  [{n:3d}] @ {p:04x} id={ident:08x} STR type={typ:04x} len={ln} '{s[:24]}'")
            p = p2 + 4 + ln * w
        elif typ == 0x22:
            n2 = struct.unpack_from(">I", d, p2)[0]
            pp = p2 + 4
            for _ in range(n2):
                sz, _, _ = struct.unpack_from(">III", d, pp)
                pp += 12 + sz
            print(f"  [{n:3d}] @ {p:04x} id={ident:08x} texts n={n2} -> {pp:04x}")
            p = pp
        else:
            sz = scalar_size(typ, spec)
            if sz is None:
                print(f"  [{n:3d}] @ {p:04x} id={ident:08x} UNHANDLED type={typ:04x} spec={spec:04x}")
                print("        next 48 bytes: " + " ".join(f"{x:02x}" for x in d[p:p + 48]))
                break
            print(f"  [{n:3d}] @ {p:04x} id={ident:08x} {TNAMES.get(typ,'?')}({typ:04x}) -> {p2+sz:04x}")
            p = p2 + sz
        n += 1
    if p < len(d):
        rem = len(d) - p
        print(f"  TRAILING {rem}: " + " ".join(f"{x:02x}" for x in d[p:]))


def main():
    pkg = sys.argv[1]
    limit = int(sys.argv[2]) if len(sys.argv) > 2 else 400
    target_off = int(sys.argv[3]) if len(sys.argv) > 3 else None
    path = Path(pkg)
    found = 0
    for g, off, stored, dec in parse_index(path):
        with path.open("rb") as fh:
            fh.seek(off); raw = fh.read(stored)
        try:
            d = decompress(raw, out_size=None, header=True)[0] if dec > stored else raw
        except Exception:
            continue
        if len(d) > limit:
            continue
        ok = _std_walk(d)
        if ok:
            continue
        if target_off is not None and found == 0:
            # only show the one whose desync is at target_off (best effort)
            pass
        print(f"=== group 0x{g:08X} len={len(d)} off={off} ===")
        trace(d)
        found += 1
        if found >= 4:
            break


def _std_walk(d):
    p = 0
    try:
        _c, p = struct.unpack_from(">I", d, p)[0], p + 4
    except struct.error:
        return False
    while p + 8 <= len(d):
        if d[p:p + 4] == MARKER:
            p += 4
            continue
        ident, typ, spec = struct.unpack_from(">IHH", d, p)
        p2 = p + 8
        if spec in ARRAY_SPECIFIERS:
            n2, stride = struct.unpack_from(">II", d, p2); p2 += 8
            if n2 > 200_000 or stride > 16384 or p2 + n2 * stride > len(d):
                return False
            p = p2 + n2 * stride
        elif typ in (0x12, 0x13):
            ln = struct.unpack_from(">I", d, p2)[0]; p2 += 4
            p = p2 + ln * (1 if typ == 0x12 else 2)
        elif typ == 0x22:
            n2 = struct.unpack_from(">I", d, p2)[0]; p2 += 4
            for _ in range(n2):
                sz, _, _ = struct.unpack_from(">III", d, p2); p2 += 12
                if p2 + sz > len(d):
                    return False
                p2 += sz
            p = p2
        else:
            sz = scalar_size(typ, spec)
            if sz is None:
                return False
            if p2 + sz > len(d):
                return False
            p = p2 + sz
    if d[p:] == MARKER:
        p += 4
    return p == len(d)


if __name__ == "__main__":
    main()

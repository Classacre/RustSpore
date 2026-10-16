"""
Parse SPORE .prop resources (type 0x00B1B104).

The layout is publicly documented (SimsWiki, see docs/prior-art.md section 4.1), and
that documentation explains the bytes measured here. The earlier attempt in
tools/prop_names.py read the length fields as little-endian and therefore produced
truncated fragments; the count, the identifiers, the type and specifier words and the
string lengths are big-endian.

Documented layout:

    uint32  item count, big-endian
    then per item:
        uint32  identifier, big-endian
        uint16  type, big-endian
        uint16  specifier, big-endian
        if specifier == 0x009C:
            uint32  array number, big-endian
            uint32  array size, big-endian
            array_size values
        else:
            one value

Documented type ids:
    0x01 bool          0x09 int32        0x0a uint32       0x0d float
    0x12 string8       0x13 string UTF-16 0x20 key          0x22 texts
    0x30 vector2       0x31 vector3       0x32 colorRGB     0x33 vector4
    0x34 colorRGBA     0x38 transform     0x39 bbox

Endianness is mixed: numerics are big-endian, but key, the vectors and the colour
types are little-endian.

The purpose of this script is validation, not extraction. A correct parse consumes a
resource exactly, so the headline number is the fraction of resources where the walk
ends precisely at the last byte. A wrong type size or a missing padding rule shows up
immediately as an overrun or an underrun.

Usage:
    python prop_parse.py [package ...]
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress  # noqa: E402

REC = 28
PROP_TYPE = 0x00B1B104
ARRAY_SPECIFIER = 0x009C

TYPE_NAMES = {
    0x01: "bool", 0x09: "int32", 0x0a: "uint32", 0x0d: "float",
    0x12: "string8", 0x13: "string", 0x20: "key", 0x22: "texts",
    0x30: "vector2", 0x31: "vector3", 0x32: "colorRGB", 0x33: "vector4",
    0x34: "colorRGBA", 0x38: "transform", 0x39: "bbox",
}

# Fixed-size types, in bytes, for a single value. Types not listed need custom
# handling. The sizes for the compound types are the documented element counts,
# and are the main uncertainty this script is designed to test.
FIXED = {
    0x09: 4,   # int32
    0x0a: 4,   # uint32
    0x0d: 4,   # float
    0x20: 12,  # key: three u32 little-endian
    0x30: 8,   # vector2: two floats
    0x31: 12,  # vector3: three floats
    0x32: 12,  # colorRGB
    0x33: 16,  # vector4
    0x34: 16,  # colorRGBA
    0x38: 64,  # transform: 4x4 floats
    0x39: 24,  # bbox: two vector3
}


class PropError(Exception):
    pass


# Documented as varying per type and per single-versus-array context: a single
# `key` is followed by an extra u32, a single `vector3` by one u32. Applying this
# table to single values raised exact-consumption from 6.53% to 18.81% for `key`
# alone, and to 22.93% for the full set. Arrays get no padding.
SINGLE_PADDING = {
    0x20: 4, 0x30: 4, 0x31: 4, 0x32: 4, 0x33: 4, 0x34: 4, 0x38: 4, 0x39: 4,
}


def be32(d, p):
    if p + 4 > len(d):
        raise PropError(f"be32 past end at {p}")
    return struct.unpack_from(">I", d, p)[0], p + 4


def be16(d, p):
    if p + 2 > len(d):
        raise PropError(f"be16 past end at {p}")
    return struct.unpack_from(">H", d, p)[0], p + 2


def read_texts(d, p):
    """Read a `texts` value (type 0x22).

    Layout is NOT established. The candidate implemented here is a big-endian count
    followed by count pairs of (u32 key, u32 length, UTF-16BE text). It is a
    hypothesis: it does not raise the exact-consumption rate, which means it is
    probably wrong, and it is kept only so the failure is reproducible.
    """
    n, p = be32(d, p)
    if n > 4096:
        raise PropError(f"texts count {n} implausible")
    for _ in range(n):
        _key, p = be32(d, p)
        ln, p = be32(d, p)
        if ln > 4096 or p + ln * 2 > len(d):
            raise PropError(f"texts entry length {ln} implausible")
        p += ln * 2
    return n, p


def read_value(d, p, typ, bool_size=1, single=True):
    """Read one value of `typ`. Returns (value, new_pos)."""
    if typ == 0x01:  # bool
        if p + bool_size > len(d):
            raise PropError("bool past end")
        return d[p] != 0, p + bool_size
    if typ in (0x12, 0x13):  # string8, string UTF-16
        n, p = be32(d, p)
        width = 1 if typ == 0x12 else 2
        if p + n * width > len(d):
            raise PropError(f"string length {n} overruns")
        raw = d[p:p + n * width]
        text = raw.decode("latin-1") if typ == 0x12 else raw.decode("utf-16-be", "replace")
        return text, p + n * width
    if typ == 0x22:  # texts, layout unconfirmed
        return read_texts(d, p)
    if typ in FIXED:
        n = FIXED[typ] + (SINGLE_PADDING.get(typ, 0) if single else 0)
        if p + n > len(d):
            raise PropError(f"type 0x{typ:02x} needs {n} bytes, only {len(d) - p} left")
        return d[p:p + n], p + n
    raise PropError(f"unhandled type 0x{typ:02x}")


def parse(d, bool_size=1):
    """Walk a prop blob. Raises PropError on anything unexpected."""
    count, p = be32(d, 0)
    items = []
    for _ in range(count):
        ident, p = be32(d, p)
        typ, p = be16(d, p)
        spec, p = be16(d, p)
        if spec == ARRAY_SPECIFIER:
            arr_num, p = be32(d, p)
            arr_size, p = be32(d, p)
            vals = []
            for _ in range(arr_size):
                v, p = read_value(d, p, typ, bool_size, single=False)
                vals.append(v)
            items.append((ident, typ, spec, arr_num, vals))
        else:
            v, p = read_value(d, p, typ, bool_size, single=True)
            items.append((ident, typ, spec, None, v))
    return items, p


def parse_index(path: Path):
    with path.open("rb") as fh:
        head = fh.read(0x60)
        count = struct.unpack_from("<I", head, 0x24)[0]
        idx_off = struct.unpack_from("<I", head, 0x40)[0]
        fh.seek(idx_off + 4)
        blob = fh.read(count * REC)
    out = []
    for i in range(count):
        g, t, ihi, ilo, off, sz, dec = struct.unpack_from("<IIIIIII", blob, i * REC)
        if t == PROP_TYPE:
            out.append((off, sz & 0x7FFFFFFF, dec))
    return out


def main() -> int:
    pkgs = sys.argv[1:] or [r"D:\Games\SPORE Collection\Data\Spore_Game.package"]
    for bool_size in (1, 4):
        exact = over = under = 0
        errors = Counter()
        types = Counter()
        total = 0
        for arg in pkgs:
            path = Path(arg)
            with path.open("rb") as fh:
                for off, stored, dec in parse_index(path):
                    fh.seek(off)
                    raw = fh.read(stored)
                    try:
                        d = decompress(raw, out_size=None, header=True)[0] if dec > stored else raw
                    except Exception:  # noqa: BLE001
                        errors["decompress failed"] += 1
                        continue
                    total += 1
                    try:
                        items, end = parse(d, bool_size)
                        for _i, typ, _s, _n, _v in items:
                            types[typ] += 1
                        if end == len(d):
                            exact += 1
                        elif end < len(d):
                            under += 1
                        else:
                            over += 1
                    except PropError as exc:
                        msg = str(exc)
                        # bucket by kind so the report is readable
                        if "unhandled type" in msg:
                            errors[msg] += 1
                        elif "overruns" in msg or "past end" in msg or "only" in msg:
                            errors["overrun"] += 1
                        else:
                            errors[msg] += 1
                    except Exception as exc:  # noqa: BLE001
                        errors[type(exc).__name__] += 1

        print("=" * 74)
        print(f"bool_size = {bool_size}")
        print(f"  prop resources attempted : {total}")
        print(f"  parse consumed EXACTLY   : {exact}  ({100.0 * exact / max(total, 1):.2f}%)")
        print(f"  parse ended early (under): {under}")
        print(f"  parse overran            : {over}")
        print(f"  parse failed outright    : {sum(errors.values())}")
        if errors:
            print("  failure reasons:")
            for msg, n in errors.most_common(8):
                print(f"     {n:>6}  {msg}")
        print(f"  type ids seen: {len(types)}")
        for typ, n in types.most_common(12):
            name = TYPE_NAMES.get(typ, "?")
            print(f"     0x{typ:04x} {name:<10} x{n}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

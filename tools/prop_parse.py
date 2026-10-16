"""
Parse SPORE .prop resources (type 0x00B1B104).

The layout is publicly documented (SimsWiki, via docs/prior-art.md section 4.1). The
documentation is correct, but it describes intent rather than a byte-exact spec, so
several details had to be measured. The measured ones are marked below.

VALIDATION METRIC: a correct parse consumes a resource exactly, ending on the last
byte. Overruns and underruns are both failures. The headline number this script prints
is the fraction of resources that consume exactly, so any wrong size shows up at once.
Current state: 11,354 of 11,953 (94.99%) across three packages.

Measured layout:

    u32 item count, BIG-ENDIAN
    then per item:
        u32 identifier, big-endian
        u16 type, big-endian
        u16 specifier, big-endian
        if specifier is an ARRAY MARKER:
            u32 element count, big-endian
            u32 element stride in bytes, big-endian
            count * stride bytes of elements
        else:
            one value, sized by type

Measured details the public notes do not state exactly:

1. **There are TWO array markers, not one: `0x009C` and `0x001C`.** The public notes
   mention only `0x009C`. Recognising `0x001C` as well is the single largest finding in
   this file: it moved the exact-consumption rate from 52.82% to 94.99% over three
   packages. It was found by dumping the bytes at a failure, where a `bbox` record with
   specifier `0x001C` was followed by `count=1, stride=0x18`, and a bbox is exactly 24
   bytes.

2. The array header is `count` then `element stride in bytes`. Treating the second word
   as the count is wrong and was the second largest source of failures. Advancing by
   `count * stride` raised the rate from 22.93% to 68.17% on its own.

3. Endianness is mixed, and the notes only hint at it. The count, identifier, type,
   specifier, array header and STRING LENGTHS are big-endian, but the UTF-16 string
   CHARACTERS are little-endian. Reading characters as big-endian turns "Name" into CJK.

4. A single `key` is 16 bytes, not the 12 that three u32 would suggest. Measured: 12
   bytes gives 6.51% exact, 16 bytes gives 22.93% at that stage of the work. The array
   stride for a key is 12, so the extra word is a single-value concern.

5. Array elements carry no single-value padding; the stride covers them.

6. `texts` (type 0x22) needs no special handling: it is an ordinary array whose stride
   is 520 bytes. An earlier revision of this file gave it a bespoke layout derived from
   a single-record prop; that derivation happened to consume the same number of bytes,
   which is why it appeared to work. The array rule supersedes it.

7. For type `0x00` the specifier appears to be the value size in bytes. This affects a
   small number of resources.

Remaining failures at 94.99%: a handful of `key` overruns, `0x100e`, and a few records
reading `0x03` as a type word. Together these are about 600 of 11,953 resources.

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

# Both markers mean "an array follows". See note 1.
ARRAY_SPECIFIERS = frozenset({0x009C, 0x001C})

TYPE_NAMES = {
    0x01: "bool", 0x09: "int32", 0x0a: "uint32", 0x0d: "float",
    0x12: "string8", 0x13: "string", 0x20: "key", 0x22: "texts",
    0x30: "vector2", 0x31: "vector3", 0x32: "colorRGB", 0x33: "vector4",
    0x34: "colorRGBA", 0x38: "transform", 0x39: "bbox",
}

# Fixed single-value sizes in bytes. `key` is handled separately because its
# single-value size (16) differs from its array stride (12).
FIXED = {
    0x09: 4,   # int32
    0x0a: 4,   # uint32
    0x0d: 4,   # float
    0x30: 8,   # vector2: two floats
    0x31: 12,  # vector3: three floats
    0x32: 12,  # colorRGB
    0x33: 16,  # vector4
    0x34: 16,  # colorRGBA
    0x38: 64,  # transform: 4x4 floats
    0x39: 24,  # bbox: two vector3
}

# Extra bytes a SINGLE value of these types carries. Arrays do not get this; the
# array stride covers them. Measured: applying this moved the rate from 42.40% to
# 68.17% at that stage.
# NOTE: colorRGBA (0x34) is deliberately absent. It is 16 bytes with no padding.
SINGLE_PADDING = {
    0x30: 4, 0x31: 4, 0x32: 4, 0x33: 4, 0x38: 4, 0x39: 4,
}

BOOL_SIZE = 1
KEY_SINGLE_SIZE = 16
KEY_ARRAY_STRIDE = 12


class PropError(Exception):
    pass


def be32(d, p):
    if p + 4 > len(d):
        raise PropError(f"be32 past end at {p}")
    return struct.unpack_from(">I", d, p)[0], p + 4


def be16(d, p):
    if p + 2 > len(d):
        raise PropError(f"be16 past end at {p}")
    return struct.unpack_from(">H", d, p)[0], p + 2


def read_texts(d, p):
    """Read a `texts` value (type 0x22) that is NOT flagged as an array.

    Rare, since the array markers cover almost every texts record. Kept because the
    non-array form does occur. See note 6.
    """
    n, p = be32(d, p)
    if n > 4096:
        raise PropError(f"texts count {n} implausible")
    for _ in range(n):
        size, p = be32(d, p)
        _f2, p = be32(d, p)
        _f3, p = be32(d, p)
        if size > 65536 or p + size > len(d):
            raise PropError(f"texts buffer size {size} overruns")
        p += size
    return n, p


def read_value(d, p, typ, spec, in_array):
    """Read one value. `in_array` selects array rules over single-value rules."""
    if typ == 0x01:
        if p + BOOL_SIZE > len(d):
            raise PropError("bool past end")
        return d[p] != 0, p + BOOL_SIZE

    if typ in (0x12, 0x13):
        n, p = be32(d, p)
        width = 1 if typ == 0x12 else 2
        if p + n * width > len(d):
            raise PropError(f"string length {n} overruns")
        raw = d[p:p + n * width]
        text = raw.decode("latin-1") if typ == 0x12 else raw.decode("utf-16-le", "replace")
        return text, p + n * width

    if typ == 0x22:
        return read_texts(d, p)

    if typ == 0x20:  # key
        n = KEY_ARRAY_STRIDE if in_array else KEY_SINGLE_SIZE
        if p + n > len(d):
            raise PropError("key past end")
        return d[p:p + n], p + n

    if typ == 0x00:
        # The specifier appears to be the value size in bytes for this type.
        if spec > len(d):
            raise PropError(f"type 0x00 size {spec} overruns")
        return d[p:p + spec], p + spec

    if typ in FIXED:
        n = FIXED[typ] + (0 if in_array else SINGLE_PADDING.get(typ, 0))
        if p + n > len(d):
            raise PropError(f"type 0x{typ:02x} needs {n} bytes, only {len(d) - p} left")
        return d[p:p + n], p + n

    raise PropError(f"unhandled type 0x{typ:02x}")


def parse(d):
    """Walk a prop blob. Returns (items, end_pos). Raises PropError on anything odd."""
    count, p = be32(d, 0)
    items = []
    for _ in range(count):
        ident, p = be32(d, p)
        typ, p = be16(d, p)
        spec, p = be16(d, p)
        if spec in ARRAY_SPECIFIERS:
            n, p = be32(d, p)       # element count
            stride, p = be32(d, p)  # element stride in bytes
            if n > 100_000 or stride > 8192:
                raise PropError(f"array header {n}x{stride} implausible")
            if p + n * stride > len(d):
                raise PropError(f"array span {n}x{stride} overruns")
            items.append((ident, typ, spec, n, None))
            p += n * stride
        else:
            v, p = read_value(d, p, typ, spec, in_array=False)
            items.append((ident, typ, spec, 1, v))
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
    exact = under = over = failed = total = 0
    errors = Counter()
    types = Counter()
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
                    items, end = parse(d)
                    for _i, typ, _s, _n, _v in items:
                        types[typ] += 1
                    if end == len(d):
                        exact += 1
                    elif end < len(d):
                        under += 1
                    else:
                        over += 1
                except PropError as exc:
                    failed += 1
                    errors[str(exc)] += 1

    print(f"prop resources attempted : {total}")
    print(f"parse consumed EXACTLY   : {exact}  ({100.0 * exact / max(total, 1):.2f}%)")
    print(f"ended early (under)      : {under}")
    print(f"overran                  : {over}")
    print(f"failed outright          : {failed}")
    if errors:
        print("failure reasons:")
        for msg, n in errors.most_common(8):
            print(f"   {n:>6}  {msg}")
    print(f"type ids seen: {len(types)}")
    for typ, n in types.most_common(12):
        print(f"   0x{typ:04x} {TYPE_NAMES.get(typ, '?'):<10} x{n}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

"""Collect 4-byte sequences at/around desync points to find missing delimiters."""

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
SEPARATORS = (b"\x3c\xc4\x12\x00", b"\xe4\x0c\xa0\x00", b"\x01\x93\x12\x00", b"\x01\x00\x00\x00")
KNOWN_SPECS = frozenset({0x0000, 0x001C, 0x009C} | set(FIXED))
KNOWN_TYPES = frozenset(set(FIXED) | {0x01, 0x20, 0x00, 0x12, 0x13, 0x22})


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


def main():
    pkg = Path(sys.argv[1])
    limit = int(sys.argv[2]) if len(sys.argv) > 2 else 200
    at_desync = Counter()
    before8 = Counter()
    n = 0
    for g, off, stored, dec in parse_index(pkg):
        with pkg.open("rb") as fh:
            fh.seek(off); raw = fh.read(stored)
        try:
            d = decompress(raw, out_size=None, header=True)[0] if dec > stored else raw
        except Exception:
            continue
        p = 0
        try:
            _c = struct.unpack_from(">I", d, p)[0]; p += 4
        except struct.error:
            continue
        while p + 8 <= len(d):
            if d[p:p + 4] in SEPARATORS:
                p += 4; continue
            ident, typ, spec = struct.unpack_from(">IHH", d, p)
            sz = entry_size(d, p + 8, typ, spec)
            if sz is None:
                typ2, spec2 = struct.unpack_from(">HH", d, p)
                sz2 = entry_size(d, p + 4, typ2, spec2)
                if typ2 in KNOWN_TYPES and sz2 is not None and spec2 in KNOWN_SPECS:
                    p += 4 + sz2; continue
                at_desync[d[p:p + 4]] += 1
                before8[d[max(0, p - 8):p]] += 1
                n += 1
                if n >= limit:
                    break
                else:
                    break  # only first per prop
            p += 8 + sz
        if n >= limit:
            break
    print(f"collected {n} desync samples from {pkg.name}")
    print("4 bytes AT desync (d[p:p+4]):")
    for k, v in at_desync.most_common(20):
        print(f"   {k.hex()}  x{v}")
    print("8 bytes BEFORE desync:")
    for k, v in before8.most_common(20):
        print(f"   {k.hex()}  x{v}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

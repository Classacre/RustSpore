"""
Robust prop walker v3: handles the variant layout.
  - skip 4-byte separators 3c c4 12 00 and e4 0c a0 00
  - at an item boundary, also try an ANONYMOUS entry (no id): if reading
    (id,type,spec) yields an unhandled type but d[p:p+2] is a known type and
    d[p+2:p+4] a known spec, read it as (type, spec, value) without an id
  - continue past the nominal count; consume trailing separator
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
KNOWN_SPECS = frozenset({0x0000, 0x001C, 0x009C} | {t for t in FIXED})

# Delimiters are little-endian u32 values whose LOW 16 bits are 0x0012 or 0x00a0
# (e.g. 3c c4 12 00, 01 93 12 00, 34 ab 12 00, bc a1 12 00, e4 0c a0 00). The high
# word varies, so detect by the low word rather than a fixed byte list. Plus the
# constant 01 00 00 00 sentinel.
SEP_LOW_WORDS = (b"\x12\x00", b"\xa0\x00")
CONST_SEP = b"\x01\x00\x00\x00"


def is_separator(d, p):
    if d[p:p + 4] == CONST_SEP:
        return True
    return d[p + 2:p + 4] in SEP_LOW_WORDS


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
        return 12
    if typ == 0x00:
        return spec
    return None


def entry_size(d, p, typ, spec):
    """Return (size, is_anonymous) for a value starting at p given typ/spec."""
    if spec in ARRAY_SPECIFIERS:
        if p + 8 > len(d):
            return None, False
        n, stride = struct.unpack_from(">II", d, p)
        if n > 200_000 or stride > 16384:
            return None, False
        if typ in (0x12, 0x13):
            # String arrays are length-prefixed per element, not fixed-stride.
            pp = p + 8
            w = 1 if typ == 0x12 else 2
            for _ in range(n):
                if pp + 4 > len(d):
                    return None, False
                ln = struct.unpack_from(">I", d, pp)[0]
                pp += 4 + ln * w
                if pp > len(d):
                    return None, False
            return pp - p, False
        if p + 8 + n * stride > len(d):
            return None, False
        return 8 + n * stride, False
    if typ in (0x12, 0x13):
        if p + 4 > len(d):
            return None, False
        ln = struct.unpack_from(">I", d, p)[0]
        w = 1 if typ == 0x12 else 2
        if p + 4 + ln * w > len(d):
            return None, False
        return 4 + ln * w, False
    if typ == 0x22:
        if p + 4 > len(d):
            return None, False
        n = struct.unpack_from(">I", d, p)[0]
        pp = p + 4
        for _ in range(n):
            if pp + 12 > len(d):
                return None, False
            sz, _, _ = struct.unpack_from(">III", d, pp)
            pp += 12
            if pp + sz > len(d):
                return None, False
            pp += sz
        return pp - p, False
    sz = scalar_size(typ, spec)
    if sz is None:
        return None, False
    if p + sz > len(d):
        return None, False
    return sz, False


def walk(d: bytes):
    p = 0
    try:
        _c, p = struct.unpack_from(">I", d, p)[0], p + 4
    except struct.error:
        return False, 0, p, "no header"
    items = 0
    while p + 8 <= len(d):
        if is_separator(d, p):
            p += 4
            continue
        ident, typ, spec = struct.unpack_from(">IHH", d, p)
        sz, _ = entry_size(d, p + 8, typ, spec)
        if sz is None:
            # try anonymous entry: type at p, spec at p+2
            typ2, spec2 = struct.unpack_from(">HH", d, p)
            sz2, _ = entry_size(d, p + 4, typ2, spec2)
            if typ2 in FIXED or typ2 in (0x01, 0x20, 0x00, 0x12, 0x13, 0x22) and sz2 is not None \
                    and spec2 in KNOWN_SPECS:
                # anonymous: consume type(2)+spec(2)+value
                p += 4 + sz2
                items += 1
                continue
            return False, items, p, f"desync type 0x{typ:04x} @ {p}"
        p += 8 + sz
        items += 1
    if is_separator(d, p):
        p += 4
    return (p == len(d)), items, p, ("" if p == len(d) else f"ended {p}/{len(d)}")


def main():
    pkgs = sys.argv[1:] or [r"D:\Games\SPORE Collection\Data\Spore_Game.package"]
    exact = under = over = failed = total = 0
    fr = Counter()
    for arg in pkgs:
        path = Path(arg)
        for g, off, stored, dec in parse_index(path):
            with path.open("rb") as fh:
                fh.seek(off); raw = fh.read(stored)
            try:
                d = decompress(raw, out_size=None, header=True)[0] if dec > stored else raw
            except Exception:
                continue
            total += 1
            ok, items, end, reason = walk(d)
            if ok:
                exact += 1
            elif end < len(d):
                under += 1
                fr[reason] += 1
            else:
                over += 1
                fr[reason] += 1
    print(f"ROBUST WALK over {total} props: exact={exact} ({100.0*exact/max(total,1):.2f}%) under={under} over={over}")
    if fr:
        print("  residual reasons:")
        for r, n in fr.most_common(10):
            print(f"     {n:>5}  {r}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

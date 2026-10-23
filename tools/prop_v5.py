"""
prop variant parser v5: backtracking exact-consumption.

At every item boundary we enumerate every structurally valid interpretation and
keep only the branches that consume the whole blob exactly. This resolves the two
residual classes without hand-tuned heuristics:

  A) ambiguous single-value sizes: 0x33 vector4 (16 vs 20 with padding), key 0x20
     (12 vs 16 single; array stride stays 12).
  B) named vs anonymous entries. A named entry is (id u32, type u16, spec u16, v);
     an anonymous entry is (type u16, spec u16, v). The "type 0x0000 => size=spec
     bytes" quirk makes a misread named entry look valid, so we try both and let
     exact consumption disambiguate.

Validation metric: a blob is EXACT if bytes [0:len] can be consumed. The header
item count is advisory only; exact consumption is what we measure.

This historical size-only walk does not certify typed property values. Primitive
single sizes were corrected on 2026-10-22: int32/uint32/float are four bytes,
not four bytes plus padding. The old padded rule could manufacture anonymous
recovery paths while discarding real identifiers. Use prop_numeric_probe.py for
an independent named/count-bounded primitive numeric check.
"""

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

# Test knob: force vector4 (0x33) single size. "both" keeps the ambiguity;
# "16" forces 16 bytes (no padding); "20" forces 20 bytes (legacy padding).
V33 = __import__("os").environ.get("PROP_V33", "both")


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


def scalar_sizes(typ, spec, in_array):
    """Return a set of plausible value sizes for (typ, spec)."""
    out = set()
    if typ == 0x01:
        out.add(1)
    elif typ == 0x20:  # key
        if in_array:
            out.add(KEY_ARRAY_STRIDE)
        else:
            out.add(16)  # documented single-key size
    elif typ == 0x00:
        if 0 <= spec <= 65535:
            out.add(spec)
    elif typ in (0x09, 0x0a, 0x0d):
        out.add(4)
    elif typ in FIXED:
        base = FIXED[typ]
        if in_array:
            out.add(base)
        elif typ in (0x33, 0x34):
            out.add(base)  # 4-float types: no single-value padding
        else:
            out.add(base + 4)  # other vector/bbox singles carry 4 bytes of padding
    return out


def array_value_size(d, p, typ):
    """Size of an array value whose header starts at p (after the (spec) marker)."""
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


def value_sizes(d, p, typ, spec):
    """All plausible total value sizes (excluding the id/type/spec prefix)."""
    if spec in ARRAY_SPECIFIERS:
        s = array_value_size(d, p, typ)
        return {s} if s is not None else set()
    if typ in (0x12, 0x13):
        if p + 4 > len(d):
            return set()
        ln = struct.unpack_from(">I", d, p)[0]
        w = 1 if typ == 0x12 else 2
        if p + 4 + ln * w > len(d):
            return set()
        return {4 + ln * w}
    if typ == 0x22:
        if p + 4 > len(d):
            return set()
        n = struct.unpack_from(">I", d, p)[0]
        pp = p + 4
        for _ in range(n):
            if pp + 12 > len(d):
                return set()
            sz, _, _ = struct.unpack_from(">III", d, pp)
            pp += 12
            if pp + sz > len(d):
                return set()
            pp += sz
        return {pp - p}
    return scalar_sizes(typ, spec, in_array=False)


def exact(d, p, memo):
    if p == len(d):
        return True
    if p in memo:
        return memo[p]
    memo[p] = False
    result = False
    # separator
    if p + 4 <= len(d) and is_separator(d, p):
        if exact(d, p + 4, memo):
            result = True
    if not result and p + 8 <= len(d):
        ident, typ, spec = struct.unpack_from(">IHH", d, p)
        # NAMED: id(4) + type(2) + spec(2) + value
        for vsz in value_sizes(d, p + 8, typ, spec):
            nxt = p + 8 + vsz
            if nxt <= len(d) and exact(d, nxt, memo):
                result = True
                break
        # ANONYMOUS: type(2) + spec(2) + value  (no id)
        if not result and typ in KNOWN_TYPES and spec in KNOWN_SPECS:
            for vsz in value_sizes(d, p + 4, typ, spec):
                nxt = p + 4 + vsz
                if nxt <= len(d) and exact(d, nxt, memo):
                    result = True
                    break
    memo[p] = result
    return result


def main():
    pkgs = sys.argv[1:] or [r"D:\Games\SPORE Collection\Data\Spore_Game.package"]
    exact_n = total = 0
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
            if exact(d, 4, {}):
                exact_n += 1
    print(f"BACKTRACK WALK over {total} props: exact={exact_n} ({100.0*exact_n/max(total,1):.4f}%)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

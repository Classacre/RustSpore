"""
Extract the property vocabulary from SPORE `.prop` resources (type 0x00B1B104).

Prop files are SPORE's data format for game objects, and they turn out to be
self-describing: each property carries its name as a length-prefixed UTF-16LE
string. Measured layout of the leading region:

    u32  property count
    then per property:
        u32  property id / hash
        u32  a field that is constant within a file (0x0D001300 in the samples)
        u32  name length, in UTF-16 code units
        u16  name[length]     UTF-16LE, ASCII range in practice

The name encoding is the useful part, because it makes the game's data
vocabulary readable without decompilation. This script recovers it by scanning
for that exact shape rather than by assuming where properties end, since the
value encoding after each name is not yet established.

Usage:
    python prop_names.py [package ...]
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress  # noqa: E402

REC = 28
PROP_TYPE = 0x00B1B104
MIN_LEN, MAX_LEN = 3, 256

# Property names are identifiers. Dialogue and placeholder text also appears as
# length-prefixed UTF-16 in the same blobs, so a name must look like an
# identifier: letters, digits, underscore and hyphen only, and not all digits.
IDENT_CHARS = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-")


def looks_like_identifier(s: str) -> bool:
    if not s or all(c.isdigit() for c in s):
        return False
    return all(c in IDENT_CHARS for c in s)


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


def payload(fh, off, stored, dec):
    fh.seek(off)
    raw = fh.read(stored)
    if dec > stored:
        try:
            return decompress(raw, out_size=None, header=True)[0]
        except Exception:  # noqa: BLE001
            return None
    return raw


def names_in(data: bytes):
    """Yield (offset, name) for every length-prefixed UTF-16 name in the blob."""
    i = 0
    n = len(data)
    while i + 8 <= n:
        ln = struct.unpack_from("<I", data, i)[0]
        if MIN_LEN <= ln <= MAX_LEN and i + 4 + ln * 2 <= n:
            chunk = data[i + 4:i + 4 + ln * 2]
            ok = True
            chars = []
            for k in range(0, len(chunk), 2):
                lo, hi = chunk[k], chunk[k + 1]
                if hi != 0 or not (0x20 <= lo < 0x7F):
                    ok = False
                    break
                chars.append(chr(lo))
            if ok:
                word = "".join(chars)
                if looks_like_identifier(word):
                    yield i, word
                i += 4 + ln * 2
                continue
        i += 1


def main() -> int:
    pkgs = sys.argv[1:] or [r"D:\Games\SPORE Collection\Data\Spore_Game.package"]
    all_names = Counter()
    total_props = 0
    with_names = 0

    for arg in pkgs:
        path = Path(arg)
        props = parse_index(path)
        print(f"{path.name}: {len(props)} prop resources")
        total_props += len(props)
        with path.open("rb") as fh:
            for off, stored, dec in props:
                data = payload(fh, off, stored, dec)
                if not data:
                    continue
                found = list(names_in(data))
                if found:
                    with_names += 1
                for _, name in found:
                    all_names[name] += 1

    print()
    print(f"prop resources scanned      : {total_props}")
    print(f"prop resources with names   : {with_names}")
    print(f"distinct property names     : {len(all_names)}")
    print(f"total name occurrences      : {sum(all_names.values())}")
    print()
    print("top 60 property names by occurrence:")
    for name, count in all_names.most_common(60):
        print(f"  {count:>7}  {name}")

    out = Path(__file__).resolve().parent.parent / "re" / "data" / "prop-names.txt"
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8") as fh:
        fh.write("Property names extracted from SPORE .prop resources (type 0x00B1B104).\n")
        fh.write("Method: scan for length-prefixed UTF-16LE names, see tools/prop_names.py.\n")
        fh.write(f"prop resources scanned: {total_props}\n")
        fh.write(f"distinct names: {len(all_names)}\n\n")
        fh.write("count\tname\n")
        for name, count in all_names.most_common():
            fh.write(f"{count}\t{name}\n")
    print()
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

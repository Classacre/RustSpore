#!/usr/bin/env python3
"""Validate type 0x02D5C9AF: a two-section hash-pair table.

Layout (all big-endian):
    u32 version = 4
    section 1: u32 count_a, then count_a * 12-byte records
    section 2: u32 count_b, then count_b * 12-byte records
    total = 12 + 12*(count_a + count_b)

Each projected 12-byte record consists of three raw words. This historical
validator checks framing only; it does not validate or classify middle words.
The first section contains both 0x02E1A75D and 0x02E1A7FF in the wider corpus,
and the meaning of the second section's middle words remains unverified.
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x02D5C9AF


def walk(d):
    """Return (ok, sections) where sections is a list of (count, offset)."""
    n = len(d)
    if n < 8:
        return False, []
    if struct.unpack_from(">I", d, 0)[0] != 4:
        return False, []
    p = 4
    sections = []
    while p < n:
        if p + 4 > n:
            return False, sections
        cnt = struct.unpack_from(">I", d, p)[0]
        p += 4
        if cnt > 1_000_000:
            return False, sections
        start = p
        p += cnt * 12
        if p > n:
            return False, sections
        sections.append((cnt, start))
    return p == n, sections


def main():
    total = 0
    ok = 0
    cats = Counter()
    sec_counts = Counter()
    for pkg in sorted(BASE.glob("*.package")):
        try:
            entries, _ = parse_index(pkg)
        except Exception:
            continue
        with pkg.open("rb") as fh:
            for e in entries:
                if e["type"] != TARGET:
                    continue
                fh.seek(e["offset"])
                raw = fh.read(e["stored"])
                if e["dec"] > e["stored"]:
                    try:
                        raw = decompress(raw, out_size=None, header=True)[0]
                    except Exception:
                        continue
                total += 1
                good, sections = walk(raw)
                if good:
                    ok += 1
                    sec_counts[len(sections)] += 1
                else:
                    cats["fail"] += 1
    print(f"Total 0x02D5C9AF: {total}")
    print(f"OK: {ok} ({100*ok/total:.4f}%)")
    print(f"Sections per sample: {dict(sec_counts)}")
    print(f"Failures: {dict(cats)}")


if __name__ == "__main__":
    main()

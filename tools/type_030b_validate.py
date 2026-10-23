#!/usr/bin/env python3
"""Validate type 0x030BDEE3: a content-assembly manifest.

Layout (all big-endian):
    offset  0: u32 version = 10
    offset 84: u32 ascii_length
    offset 88: ascii tag (ascii_length bytes), e.g. "tag:spore.com,2006:AssembledContent"
    then:
        u32 count_a, then count_a * (u32 utf16_length + utf16_length*2 bytes)
        u32 count_b, then count_b * 4-byte hash
        one 4-byte hash
        one optional extra 4-byte hash (the 139-byte variant)
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x030BDEE3


def walk(d):
    n = len(d)
    if n < 88:
        return False
    if struct.unpack_from(">I", d, 0)[0] != 10:
        return False
    sl = struct.unpack_from(">I", d, 84)[0]
    if 88 + sl > n:
        return False
    p = 88 + sl

    if p + 4 > n:
        return False
    count_a = struct.unpack_from(">I", d, p)[0]
    p += 4
    if count_a > 1000:
        return False
    for _ in range(count_a):
        if p + 4 > n:
            return False
        ln = struct.unpack_from(">I", d, p)[0]
        p += 4
        if ln > 1000 or p + ln * 2 > n:
            return False
        p += ln * 2

    if p + 4 > n:
        return False
    count_b = struct.unpack_from(">I", d, p)[0]
    p += 4
    if count_b > 1000:
        return False
    p += count_b * 4

    if p + 4 > n:
        return False
    p += 4  # trailing hash
    if n - p == 4:
        p += 4  # 139-byte variant carries one extra hash
    return p == n


def main():
    total = 0
    ok = 0
    cats = Counter()
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
                if walk(raw):
                    ok += 1
                else:
                    cats["fail"] += 1
    print(f"Total 0x030BDEE3: {total}")
    print(f"OK: {ok} ({100*ok/total:.4f}%)")
    print(f"Failures: {dict(cats)}")


if __name__ == "__main__":
    main()

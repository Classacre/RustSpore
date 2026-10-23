#!/usr/bin/env python3
"""Validate type 0x0F43029A (.creaturedata).

Structure (from SporeModder FX / NeoSporeMaster CreatureDataFile, reimplemented):
    i32 header = 0xABB455B7 (LE)
    i32 version (9 or 10)
    3 counts (version 9) or 4 counts (version 10), i32 LE each
    RLE chunk buffer, decompressed length 0x44 (0x80-0x3C):
        repeat: u16 LE chunk_size, then chunk_size bytes; decode until 0x44 output
        RLE: byte 0xFE = escape; next byte count (0 = literal 0xFE, else repeat last)
             last byte is set to the CHUNK's first byte (SporeModder quirk)
    counts[0] * 0x8C bytes (data0)
    counts[1] * 4 bytes (capability ids)
    counts[1] * 1 byte  (capability numbers)
    counts[2] * 4 bytes (morph ids)
    counts[2] * 4 bytes (value1 f32)
    counts[2] * 4 bytes (value2 f32)
    4 bytes (field110)
    if version >= 10: counts[3] * 0x38 bytes (data3)
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x0F43029A
HEADER = 0xABB455B7


def read_compressed(d, p, length):
    """Return the new offset after reading the RLE chunk buffer, or None on error."""
    n = len(d)
    dst = 0
    last = 0
    current = None
    idx = 0
    while dst < length:
        if current is None or idx >= len(current):
            if p + 2 > n:
                return None
            size = struct.unpack_from("<H", d, p)[0]
            p += 2
            if size < 1 or size > 0x800 or p + size > n:
                return None
            current = d[p:p + size]
            p += size
            idx = 0
        if current[idx] != 254:
            last = current[0]  # SporeModder quirk: chunk's first byte
            dst += 1
        else:
            idx += 1
            if idx < len(current):
                count = current[idx]
                if count == 0:
                    dst += 1
                else:
                    dst = min(dst + count, length)
        idx += 1
    return p


def parse(d):
    n = len(d)
    if n < 8:
        return False
    if struct.unpack_from("<I", d, 0)[0] != HEADER:
        return False
    version = struct.unpack_from("<I", d, 4)[0]
    if version < 9 or version > 10:
        return False
    cc = 3 if version < 10 else 4
    p = 8
    counts = []
    for _ in range(cc):
        c = struct.unpack_from("<I", d, p)[0]
        p += 4
        if c > 1_000_000:
            return False
        counts.append(c)
    p = read_compressed(d, p, 0x44)
    if p is None:
        return False
    p += counts[0] * 0x8C
    p += counts[1] * 4
    p += counts[1] * 1
    p += counts[2] * 4
    p += counts[2] * 4
    p += counts[2] * 4
    p += 4
    if version >= 10:
        p += counts[3] * 0x38
    return p == n


def main():
    total = 0
    ok = 0
    cats = Counter()
    vers = Counter()
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
                if len(raw) >= 8:
                    vers[struct.unpack_from("<I", raw, 4)[0]] += 1
                if parse(raw):
                    ok += 1
                else:
                    cats["fail"] += 1
    print(f"Total 0x0F43029A: {total}")
    print(f"OK: {ok} ({100*ok/total:.4f}%)")
    print(f"Versions: {dict(vers)}")
    print(f"Failures: {dict(cats)}")


if __name__ == "__main__":
    main()

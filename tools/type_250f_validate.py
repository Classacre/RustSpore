#!/usr/bin/env python3
"""Validate the spui (0x250FE9A2) container across the full population.

Structure (from NeoSporeMaster SpuiFile.cs, reimplemented):
    u32 magic = 0xE3FE3FB8 (LE)
    u16 version (<= 3)
    u16 imageCount, u16 atlasCount, u16 hitMaskCount, u16 proxyCount
    imageCount * ResourceKey (u32 instance, type, group)
    atlasCount * ResourceKey
    hitMaskCount * hit mask:
        if version >= 3: u8 useRle
        if useRle: i32 width, i32 height, i32 wordCount, then wordCount*2 bytes
        else: ResourceKey
    proxyCount * i32
    element section: the remaining bytes (verbatim; needs external designer schema)
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x250FE9A2
MAGIC = 0xE3FE3FB8


def parse(d):
    n = len(d)
    if n < 14:
        return False
    if struct.unpack_from("<I", d, 0)[0] != MAGIC:
        return False
    version = struct.unpack_from("<H", d, 4)[0]
    if version > 3:
        return False
    ic = struct.unpack_from("<H", d, 6)[0]
    ac = struct.unpack_from("<H", d, 8)[0]
    hc = struct.unpack_from("<H", d, 10)[0]
    pc = struct.unpack_from("<H", d, 12)[0]
    if ic + ac + hc + pc > 6000:
        return False
    p = 14
    p += ic * 12
    p += ac * 12
    for _ in range(hc):
        if p >= n:
            return False
        if version >= 3:
            use_rle = d[p] != 0
            p += 1
        else:
            use_rle = False
        if use_rle:
            if p + 12 > n:
                return False
            wc = struct.unpack_from("<i", d, p + 8)[0]
            p += 12
            if wc < 0 or wc > 10_000_000:
                return False
            p += wc * 2
        else:
            p += 12
        if p > n:
            return False
    p += pc * 4
    if p > n:
        return False
    # Remaining bytes are the element section (kept verbatim).
    return True


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
                if len(raw) >= 6:
                    vers[struct.unpack_from("<H", raw, 4)[0]] += 1
                if parse(raw):
                    ok += 1
                else:
                    cats["fail"] += 1
    print(f"Total 0x250FE9A2 (spui): {total}")
    print(f"Container OK: {ok} ({100*ok/total:.4f}%)")
    print(f"Versions: {dict(vers)}")
    print(f"Failures: {dict(cats)}")


if __name__ == "__main__":
    main()

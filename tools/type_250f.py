#!/usr/bin/env python3
"""Deep investigation of type 0x250FE9A2 (601 resources).

Structure:
- Magic 0xB83FFEE3 (BE u32)
- BE u32 = 3 (version)
- BE u16 count
- Repeating 12-byte records: hash + u32 + u32
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


def main():
    samples = []
    for pkg in sorted(BASE.glob("*.package")):
        try:
            entries, _ = parse_index(pkg)
        except Exception:
            continue
        with pkg.open("rb") as fh:
            for e in entries:
                if e["type"] != TARGET:
                    continue
                if len(samples) >= 20:
                    break
                fh.seek(e["offset"])
                raw = fh.read(e["stored"])
                if e["dec"] > e["stored"]:
                    try:
                        raw = decompress(raw, out_size=None, header=True)[0]
                    except Exception:
                        continue
                samples.append(raw)

    print(f"Collected {len(samples)} samples\n")

    # Parse header
    for i, d in enumerate(samples[:5]):
        magic = struct.unpack_from(">I", d, 0)[0]
        ver = struct.unpack_from(">I", d, 4)[0]
        u8_be = struct.unpack_from(">H", d, 8)[0]
        u8_le = struct.unpack_from("<H", d, 8)[0]
        u10_be = struct.unpack_from(">H", d, 10)[0]
        u10_le = struct.unpack_from("<H", d, 10)[0]

        print(f"--- Sample {i+1} (size={len(d)}) ---")
        print(f"  Magic: 0x{magic:08X}, Version: {ver}")
        print(f"  BE u16@8={u8_be}, LE u16@8={u8_le}, BE u16@10={u10_be}, LE u16@10={u10_le}")

        # Show first 64 bytes
        for row in range(0, min(64, len(d)), 16):
            hex_str = " ".join(f"{b:02x}" for b in d[row:row+16])
            print(f"  +{row:04x}: {hex_str}")

        # Try to parse records starting at offset 12
        # Each record: BE u32 hash + BE u32 val1 + BE u32 val2 = 12 bytes
        off = 12
        records = []
        while off + 12 <= len(d) and len(records) < 10:
            h = struct.unpack_from(">I", d, off)[0]
            v1 = struct.unpack_from(">I", d, off + 4)[0]
            v2 = struct.unpack_from(">I", d, off + 8)[0]
            records.append((h, v1, v2))
            off += 12

        print(f"  First {len(records)} records (hash, val1, val2):")
        for h, v1, v2 in records:
            print(f"    0x{h:08X}  {v1:>10d}  {v2:>10d}")

        # Check total record count
        total_records = (len(d) - 12) // 12
        remainder = (len(d) - 12) % 12
        print(f"  Total records: {total_records}, remainder: {remainder}")
        print()

    # Statistical analysis
    print("=" * 70)
    print("Statistical analysis:")
    print("=" * 70)

    magic_dist = Counter()
    ver_dist = Counter()
    record_hash_dist = Counter()
    record_pattern = Counter()

    for d in samples:
        magic_dist[struct.unpack_from(">I", d, 0)[0]] += 1
        ver_dist[struct.unpack_from(">I", d, 4)[0]] += 1

        # Parse all records
        off = 12
        while off + 12 <= len(d):
            h = struct.unpack_from(">I", d, off)[0]
            v1 = struct.unpack_from(">I", d, off + 4)[0]
            v2 = struct.unpack_from(">I", d, off + 8)[0]
            record_hash_dist[h] += 1
            # Track (v1, v2) patterns
            record_pattern[(v1, v2)] += 1
            off += 12

    print(f"Magic: {dict(magic_dist.most_common(5))}")
    print(f"Version: {dict(ver_dist.most_common(5))}")
    print()
    print("Top 10 record hashes:")
    for h, c in record_hash_dist.most_common(10):
        print(f"  0x{h:08X}: {c}")
    print()
    print("Top 10 (val1, val2) patterns:")
    for (v1, v2), c in record_pattern.most_common(10):
        print(f"  ({v1}, {v2}): {c}")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Deep investigation of blockdata format (type 0x5CAA5E28).

Previously found:
- All samples begin with 6 LE floats (bounding box: min XYZ + max XYZ)
- 7th float and 4 flag bytes at +28
- Size distribution: 36 (121), 48 (241), 84 (256), 228 (279), 804 (81), etc.

This script investigates the full structure after the flags.
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x5CAA5E28


def collect_samples(limit=200):
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
                fh.seek(e["offset"])
                raw = fh.read(e["stored"])
                if e["dec"] > e["stored"]:
                    try:
                        raw = decompress(raw, out_size=None, header=True)[0]
                    except Exception:
                        continue
                samples.append(raw)
                if len(samples) >= limit:
                    return samples
    return samples


def hex_dump(d, start=0, length=128):
    for row in range(start, min(start + length, len(d)), 16):
        hex_str = " ".join(f"{b:02x}" for b in d[row:row+16])
        ascii_str = "".join(chr(b) if 32 <= b < 127 else "." for b in d[row:row+16])
        print(f"    +{row:04x}: {hex_str:<48s} {ascii_str}")


def main():
    samples = collect_samples(200)
    print(f"Collected {len(samples)} blockdata samples\n")

    # Group by size
    by_size = {}
    for d in samples:
        size = len(d)
        if size not in by_size:
            by_size[size] = []
        by_size[size].append(d)

    print("Size groups:")
    for size in sorted(by_size.keys()):
        print(f"  {size} bytes: {len(by_size[size])} samples")
    print()

    # Analyze each size group
    for size in sorted(by_size.keys()):
        group = by_size[size]
        print(f"{'='*70}")
        print(f"Size group: {size} bytes ({len(group)} samples)")
        print(f"{'='*70}")

        # Show first 2 samples in detail
        for i, d in enumerate(group[:2]):
            print(f"\n  Sample {i+1}:")
            hex_dump(d, 0, min(size, 128))

            # Parse header fields
            floats = [struct.unpack_from("<f", d, j)[0] for j in range(0, 28, 4)]
            print(f"    Bbox floats: {[f'{f:.4f}' for f in floats]}")
            if size >= 32:
                flags = d[28:32]
                print(f"    Flags: {flags.hex()}")
        print()

        # Statistical analysis of bytes after offset 32
        if size > 32:
            print(f"  Bytes after +32 (size-32 = {size-32} bytes):")
            # Check if all zeros
            all_zero = all(b == 0 for d in group for b in d[32:])
            print(f"    All zero: {all_zero}")

            if not all_zero:
                # Find non-zero regions
                for i, d in enumerate(group[:3]):
                    nonzero_offsets = [j for j in range(32, len(d)) if d[j] != 0]
                    if nonzero_offsets:
                        print(f"    Sample {i+1} non-zero range: +{min(nonzero_offsets)} to +{max(nonzero_offsets)}")
                        # Show non-zero bytes
                        for j in range(32, min(len(d), 96), 4):
                            val = struct.unpack_from("<I", d, j)[0]
                            if val != 0:
                                print(f"      +{j}: LE u32 = {val} (0x{val:08X})")
            print()


if __name__ == "__main__":
    main()

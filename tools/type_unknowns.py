#!/usr/bin/env python3
"""Investigate remaining large unknown types.

Types to investigate:
- 0x01A527DB (2,898) - first: 05 00 56 22 00 03 9a f5 00 00 fd 6c 00 03 9a f5
- 0x1A99B06B (1,733) - first: d3 7e 65 48 05 00 00 00 0f 00 00 00 00 00 00 00
- 0x030BDEE3 (1,641) - first: 00 00 00 0a 00 00 00 00 00 00 05 01 43 8f 63 47
- 0x02D5C9AF (1,631) - first: 00 00 00 04 00 00 00 0a 02 dd 90 af 02 e1 a7 5d
- 0x0F43029A (1,022) - first: b7 55 b4 ab 09 00 00 00 0b 00 00 00 08 00 00 00
- 0x250FE9A2 (601)  - first: b8 3f fe e3 03 00 00 00 18 00 01 00 6f 00 c9 6c
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")

TARGETS = [0x01A527DB, 0x1A99B06B, 0x030BDEE3, 0x02D5C9AF, 0x0F43029A, 0x250FE9A2]


def collect_samples(target, limit=10):
    """Collect decompressed samples for a type."""
    samples = []
    for pkg in sorted(BASE.glob("*.package")):
        try:
            entries, _ = parse_index(pkg)
        except Exception:
            continue
        with pkg.open("rb") as fh:
            for e in entries:
                if e["type"] != target:
                    continue
                if len(samples) >= limit:
                    return samples
                fh.seek(e["offset"])
                raw = fh.read(min(e["stored"], 4096))
                if e["dec"] > e["stored"]:
                    try:
                        result = decompress(raw, out_size=None, header=True)
                        if result and result[0]:
                            raw = result[0][:4096]
                    except Exception:
                        continue
                samples.append(raw)
    return samples


def hex_dump(d, length=96):
    """Print hex dump of first `length` bytes."""
    for row in range(0, min(length, len(d)), 16):
        hex_str = " ".join(f"{b:02x}" for b in d[row:row+16])
        ascii_str = "".join(chr(b) if 32 <= b < 127 else "." for b in d[row:row+16])
        print(f"    +{row:04x}: {hex_str:<48s} {ascii_str}")


def analyze_type(target):
    """Analyze a single type."""
    samples = collect_samples(target, 10)
    if not samples:
        print(f"  No samples found")
        return

    print(f"\n{'='*70}")
    print(f"Type 0x{target:08X} ({len(samples)} samples)")
    print(f"{'='*70}")

    # Statistical analysis of first 32 bytes
    field_dist = {}
    for off in range(0, 32, 4):
        be_vals = Counter()
        le_vals = Counter()
        for d in samples:
            if len(d) >= off + 4:
                be_vals[struct.unpack_from(">I", d, off)[0]] += 1
                le_vals[struct.unpack_from("<I", d, off)[0]] += 1
        field_dist[off] = (be_vals, le_vals)

    print("  Field analysis (BE / LE u32):")
    for off, (be_vals, le_vals) in field_dist.items():
        be_top = be_vals.most_common(3)
        le_top = le_vals.most_common(3)
        be_str = ", ".join(f"{v}({c})" for v, c in be_top)
        le_str = ", ".join(f"{v}({c})" for v, c in le_top)
        print(f"    +{off:2d}: BE=[{be_str}]  LE=[{le_str}]")

    # Show first 2 samples in detail
    for i, d in enumerate(samples[:2]):
        print(f"\n  Sample {i+1} (first 96 bytes):")
        hex_dump(d, 96)

        # Check for strings
        strings = []
        for off in range(len(d)):
            if 32 <= d[off] < 127:
                s = ""
                for j in range(off, min(off + 40, len(d))):
                    if 32 <= d[j] < 127:
                        s += chr(d[j])
                    else:
                        break
                if len(s) >= 4:
                    strings.append((off, s))
        if strings:
            print(f"    Strings: {[(o, s) for o, s in strings[:3]]}")


def main():
    for target in TARGETS:
        analyze_type(target)


if __name__ == "__main__":
    main()

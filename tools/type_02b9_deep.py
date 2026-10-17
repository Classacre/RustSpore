#!/usr/bin/env python3
"""Deep analysis of type 0x02B9F662 structure.

Pattern observed:
- BE u32: entry count
- Entries with hash IDs (0x5F6317D5, 0x25DF0108, etc.)
- Pairs of consecutive hashes at the end (ranges?)
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x02B9F662


def collect_samples(limit=50):
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
                if len(samples) < limit:
                    samples.append(raw)
    return samples


def analyze_structure(samples):
    """Analyze the common structural pattern."""
    # Track field distributions
    count_dist = Counter()
    hash_at_4 = Counter()
    u16_at_8 = Counter()
    u16_at_10 = Counter()

    for d in samples:
        if len(d) < 12:
            continue
        count = struct.unpack_from(">I", d, 0)[0]
        count_dist[count] += 1
        h = struct.unpack_from(">I", d, 4)[0]
        hash_at_4[h] += 1
        u16_at_8[struct.unpack_from(">H", d, 8)[0]] += 1
        u16_at_10[struct.unpack_from(">H", d, 10)[0]] += 1

    print("Entry count distribution (BE u32 @0):")
    for v, c in count_dist.most_common(10):
        print(f"  {v}: {c}")
    print()

    print("Hash at +4 (top 10):")
    for v, c in hash_at_4.most_common(10):
        print(f"  0x{v:08X}: {c}")
    print()

    print("BE u16 @8 (top 10):")
    for v, c in u16_at_8.most_common(10):
        print(f"  {v} (0x{v:04X}): {c}")
    print()

    print("BE u16 @10 (top 5):")
    for v, c in u16_at_10.most_common(5):
        print(f"  {v}: {c}")
    print()


def find_repeated_hashes(samples):
    """Find hashes that appear across multiple samples."""
    hash_positions = {}

    for si, d in enumerate(samples):
        for off in range(0, len(d) - 4, 4):
            h = struct.unpack_from(">I", d, off)[0]
            # Only track values that look like hashes (high bit patterns)
            if h > 0x01000000 and h < 0xFFFFFFFF:
                if h not in hash_positions:
                    hash_positions[h] = []
                hash_positions[h].append((si, off))

    # Find hashes appearing in many samples
    common = [(h, positions) for h, positions in hash_positions.items()
              if len(set(si for si, _ in positions)) >= 5]

    common.sort(key=lambda x: -len(set(si for si, _ in x[1])))

    print("Hashes appearing in 5+ samples:")
    for h, positions in common[:20]:
        sample_count = len(set(si for si, _ in positions))
        offsets = sorted(set(off for _, off in positions))
        print(f"  0x{h:08X}: {sample_count} samples, offsets={offsets[:5]}")


def parse_entries(samples):
    """Try to parse the entry structure."""
    for i, d in enumerate(samples[:5]):
        print(f"\n--- Sample {i+1} (size={len(d)}) ---")
        count = struct.unpack_from(">I", d, 0)[0]
        print(f"Entry count: {count}")

        off = 4
        for entry_idx in range(min(count, 10)):
            if off + 8 > len(d):
                print(f"  Entry {entry_idx}: TRUNCATED")
                break
            h = struct.unpack_from(">I", d, off)[0]
            u16a = struct.unpack_from(">H", d, off + 4)[0]
            u16b = struct.unpack_from(">H", d, off + 6)[0]
            extra = ""
            if off + 12 <= len(d):
                f = struct.unpack_from(">f", d, off + 8)[0]
                if -1000 < f < 1000 and f != 0:
                    extra = f", float={f:.4f}"
                else:
                    u32v = struct.unpack_from(">I", d, off + 8)[0]
                    extra = f", u32=0x{u32v:08X}"
            print(f"  Entry {entry_idx} @+{off}: hash=0x{h:08X}, u16a={u16a}, u16b={u16b}{extra}")
            off += 8
            if u16a in (13, 1) and off + 4 <= len(d):
                # Extended entry with float
                off += 4

        # Show remaining bytes
        if off < len(d):
            remaining = d[off:]
            print(f"  Remaining {len(remaining)} bytes:")
            for row in range(0, min(len(remaining), 64), 16):
                hex_str = " ".join(f"{b:02x}" for b in remaining[row:row+16])
                print(f"    +{off+row:04x}: {hex_str}")


def main():
    samples = collect_samples(50)
    print(f"Collected {len(samples)} samples\n")

    analyze_structure(samples)
    find_repeated_hashes(samples)
    parse_entries(samples)


if __name__ == "__main__":
    main()

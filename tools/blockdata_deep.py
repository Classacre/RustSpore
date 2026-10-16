"""
Deep investigation of blockdata format structure.

Earlier findings:
- All samples begin with 6 LE floats (bounding box: min XYZ + max XYZ)
- 36-byte samples: 6 floats + 3 u32 zeros
- Larger samples: 7th float + 4 flag bytes at +28 + variable data

This script investigates the structure after the flags for larger samples.
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

BLOCKDATA_TYPE = 0x5CAA5E28


def get_samples(path: Path, target_type: int, limit: int = 200):
    entries, _ = parse_index(path)
    samples = []
    with path.open("rb") as fh:
        for e in entries:
            if e["type"] != target_type:
                continue
            if len(samples) >= limit:
                break
            fh.seek(e["offset"])
            raw = fh.read(e["stored"])
            if e["dec"] > e["stored"]:
                try:
                    raw = decompress(raw, out_size=None, header=True)[0]
                except Exception:
                    continue
            if len(raw) < 36:
                continue
            samples.append(raw)
    return samples


def analyze_by_size(samples):
    """Analyze blockdata structure grouped by size."""
    print("=" * 70)
    print("BLOCKDATA STRUCTURE BY SIZE")
    print("=" * 70)

    by_size = {}
    for d in samples:
        size = len(d)
        if size not in by_size:
            by_size[size] = []
        by_size[size].append(d)

    for size in sorted(by_size.keys()):
        group = by_size[size]
        print(f"\n{'='*70}")
        print(f"Size {size}: {len(group)} samples")
        print(f"{'='*70}")

        # Show first 2 samples of each size
        for idx in range(min(2, len(group))):
            d = group[idx]
            print(f"\n  Sample {idx+1} (size={size}):")

            # Bounding box
            min_xyz = struct.unpack_from("<fff", d, 0)
            max_xyz = struct.unpack_from("<fff", d, 12)
            print(f"    BBox min: ({min_xyz[0]:.4f}, {min_xyz[1]:.4f}, {min_xyz[2]:.4f})")
            print(f"    BBox max: ({max_xyz[0]:.4f}, {max_xyz[1]:.4f}, {max_xyz[2]:.4f})")

            # 7th float
            if size >= 28:
                f24 = struct.unpack_from("<f", d, 24)[0]
                print(f"    Float@24: {f24:.6f}")

            # Flags at +28
            if size >= 32:
                flags = d[28:32]
                print(f"    Flags@28: {flags.hex(' ')}")

            # Full hex dump (up to 128 bytes)
            print(f"    Hex dump (first {min(128, size)} bytes):")
            for j in range(0, min(128, size), 16):
                chunk = d[j:j+16]
                hexstr = chunk.hex(' ')
                asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
                print(f"      {j:>5}  {hexstr:<47}  {asc}")

            # Analyze as u32 array
            print(f"    As LE u32 array:")
            for j in range(0, min(size, 128), 4):
                v = struct.unpack_from("<I", d, j)[0]
                f = struct.unpack_from("<f", d, j)[0]
                note = ""
                if 0 < v < size:
                    note = f" [offset within file]"
                elif 0 < v < 10000:
                    note = f" [small value]"
                print(f"      +{j:>3}: u32={v:<12} float={f:.4f}{note}")

            if idx >= 1:
                break


def analyze_structure_patterns(samples):
    """Look for structural patterns across all blockdata samples."""
    print(f"\n{'='*70}")
    print("STRUCTURAL PATTERNS")
    print(f"{'='*70}")

    # Check if there's a consistent sub-header after the bounding box
    # For 48-byte samples: 6 floats (24) + 1 float (4) + 4 flags (4) + 16 bytes
    # For 84-byte samples: 6 floats (24) + 1 float (4) + 4 flags (4) + 52 bytes

    # Analyze the bytes after the flags (offset 32)
    after_flags = Counter()
    for d in samples:
        if len(d) < 48:
            continue
        after = d[32:48]  # 16 bytes after flags
        after_flags[after.hex()] += 1

    print(f"\n  16 bytes after flags (top 10):")
    for v, c in after_flags.most_common(10):
        print(f"    {v}: {c}")

    # Check for repeating patterns
    print(f"\n  Repeating 4-byte patterns in 84-byte samples:")
    patterns = Counter()
    for d in samples:
        if len(d) != 84:
            continue
        for j in range(32, 80, 4):
            patterns[d[j:j+4].hex()] += 1
    for v, c in patterns.most_common(10):
        print(f"    {v}: {c}")

    # Check if the data after flags looks like floats
    print(f"\n  Float analysis after flags (84-byte samples):")
    float_count = 0
    total_count = 0
    for d in samples:
        if len(d) != 84:
            continue
        for j in range(32, 80, 4):
            f = struct.unpack_from("<f", d, j)[0]
            total_count += 1
            if abs(f) < 1000 and f != 0:
                float_count += 1
    print(f"    Plausible floats: {float_count}/{total_count}")

    # Check for u16 patterns
    print(f"\n  u16 analysis after flags (84-byte samples):")
    u16_count = 0
    total_count = 0
    for d in samples:
        if len(d) != 84:
            continue
        for j in range(32, 80, 2):
            v = struct.unpack_from("<H", d, j)[0]
            total_count += 1
            if v < 1000:
                u16_count += 1
    print(f"    Small u16 values: {u16_count}/{total_count}")

    # Check for string data
    print(f"\n  String analysis (all sizes):")
    for d in samples[:5]:
        # Look for ASCII strings
        strings = []
        current = ""
        for j in range(len(d)):
            if 32 <= d[j] < 127:
                current += chr(d[j])
            else:
                if len(current) >= 4:
                    strings.append(current)
                current = ""
        if len(current) >= 4:
            strings.append(current)
        if strings:
            print(f"    size={len(d)}: {strings[:5]}")


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_samples(pkg, BLOCKDATA_TYPE, limit=200)
        except (ValueError, struct.error):
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} blockdata resources")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)}")
    if all_samples:
        analyze_by_size(all_samples)
        analyze_structure_patterns(all_samples)


if __name__ == "__main__":
    raise SystemExit(main())

"""
Quick investigation of type 0xDFAD9F51 (223 occurrences in Spore_Game).

This is one of the remaining unclassified types.
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

TARGET_TYPE = 0xDFAD9F51


def get_samples(path: Path, target_type: int, limit: int = 100):
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
            if len(raw) < 16:
                continue
            samples.append(raw)
    return samples


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_samples(pkg, TARGET_TYPE, limit=100)
        except (ValueError, struct.error):
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} resources")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)}")

    if not all_samples:
        print("No samples found")
        return

    # First 4 bytes
    first4 = Counter()
    for d in all_samples:
        first4[d[:4].hex()] += 1
    print(f"\n  First 4 bytes: {dict(first4)}")

    # First 8 bytes
    first8 = Counter()
    for d in all_samples:
        first8[d[:8].hex()] += 1
    print(f"\n  First 8 bytes (top 5): {first8.most_common(5)}")

    # Show first 3 samples
    for i, d in enumerate(all_samples[:3]):
        print(f"\n  Sample {i+1}: size={len(d)}")
        print(f"    First 64 bytes: {d[:64].hex(' ')}")
        asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in d[:64])
        print(f"    ASCII: {asc}")

        # Check for float data
        for j in range(0, min(32, len(d) - 3), 4):
            f = struct.unpack_from("<f", d, j)[0]
            if abs(f) < 1000 and f != 0:
                print(f"    float @+{j}: {f:.4f}")


if __name__ == "__main__":
    main()

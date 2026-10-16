"""
Investigate the SPORE blockdata format (type 0x5CAA5E28).

This is the last major undecoded payload type. The name "blockdata" suggests
it might contain block/terrain data or some other structured binary data.

Usage:
    python tools/blockdata_parse.py "D:/Games/SPORE Collection/Data/Spore_Game.package"
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

BLOCKDATA_TYPE = 0x5CAA5E28


def get_samples(path: Path, target_type: int, limit: int = 500):
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


def statistical_summary(samples):
    """Statistical analysis of blockdata headers."""
    print("=" * 70)
    print("BLOCKDATA FORMAT STATISTICAL ANALYSIS")
    print("=" * 70)

    print(f"\n  Samples: {len(samples)}")

    # Size distribution
    sizes = Counter()
    for d in samples:
        sizes[len(d)] += 1
    print(f"\n  Size distribution (top 10): {sizes.most_common(10)}")

    # First 4 bytes
    first4 = Counter()
    first8 = Counter()
    first16 = Counter()
    for d in samples:
        first4[d[:4].hex()] += 1
        first8[d[:8].hex()] += 1
        first16[d[:16].hex()] += 1

    print(f"\n  First 4 bytes (top 10):")
    for v, c in first4.most_common(10):
        print(f"    {v}: {c}")

    print(f"\n  First 8 bytes (top 10):")
    for v, c in first8.most_common(10):
        print(f"    {v}: {c}")

    # Check for known magic strings
    magics = {
        b"DBPF": "DBPF container",
        b"ANIM": "ANIM animation",
        b"RW4": "RenderWare 4",
        b"\x89PNG": "PNG image",
        b"PROP": "Property tree",
        b"GMSH": "Mesh",
        b"BLKD": "Block data?",
        b"BLCK": "Block?",
        b"TERR": "Terrain?",
        b"Raster": "Raster texture",
    }

    for magic, name in magics.items():
        count = sum(1 for d in samples if d[:len(magic)] == magic)
        if count > 0:
            print(f"\n  Magic '{name}' ({magic.hex()}): {count}/{len(samples)}")

    # Field analysis for first few samples
    print(f"\n  Field analysis (first 5 samples):")
    for i, d in enumerate(samples[:5]):
        print(f"\n    Sample {i+1}: size={len(d)}")
        print(f"      First 48 bytes: {d[:48].hex(' ')}")
        asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in d[:48])
        print(f"      ASCII: {asc}")

        for off in [0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44]:
            if off + 4 <= len(d):
                u32_le = struct.unpack_from("<I", d, off)[0]
                u32_be = struct.unpack_from(">I", d, off)[0]
                u16_le = struct.unpack_from("<H", d, off)[0]
                u16_be = struct.unpack_from(">H", d, off)[0]
                f_le = struct.unpack_from("<f", d, off)[0]
                print(f"      +{off:>2}: u32 LE={u32_le:<12} BE={u32_be:<12} "
                      f"u16 LE={u16_le:<8} BE={u16_be:<8} fLE={f_le:.4f}")


def deep_inspect(samples, n=5):
    """Deep inspection of smallest samples."""
    sorted_s = sorted(samples, key=len)

    for idx in range(min(n, len(sorted_s))):
        d = sorted_s[idx]
        print(f"\n{'='*70}")
        print(f"Sample {idx+1}: size={len(d)}")
        print(f"{'='*70}")

        # Full hex dump of first 256 bytes
        print(f"  First 256 bytes:")
        for j in range(0, min(256, len(d)), 16):
            chunk = d[j:j+16]
            hexstr = chunk.hex(' ')
            asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
            print(f"    {j:>5}  {hexstr:<47}  {asc}")

        # Look for patterns
        print(f"\n  Repeating 4-byte patterns:")
        patterns = Counter()
        for j in range(0, len(d) - 4, 4):
            patterns[d[j:j+4]] += 1
        for pat, cnt in patterns.most_common(10):
            if cnt > 1:
                print(f"    {pat.hex()}: {cnt} occurrences")

        # Check if data looks like floats
        float_count = 0
        for j in range(0, min(len(d) - 4, 1024), 4):
            f = struct.unpack_from("<f", d, j)[0]
            if abs(f) < 1000 and f != 0:
                float_count += 1
        print(f"\n  Plausible LE floats in first 1024 bytes: {float_count}/256")

        # Check if data looks like u16 pairs
        u16_count = 0
        for j in range(0, min(len(d) - 2, 1024), 2):
            v = struct.unpack_from("<H", d, j)[0]
            if v < 65536:
                u16_count += 1
        print(f"  Valid u16 in first 1024 bytes: {u16_count}/512")


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_samples(pkg, BLOCKDATA_TYPE, limit=500)
        except (ValueError, struct.error):
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} blockdata resources")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)} blockdata resources")

    if all_samples:
        statistical_summary(all_samples)
        deep_inspect(all_samples, n=3)


if __name__ == "__main__":
    raise SystemExit(main())

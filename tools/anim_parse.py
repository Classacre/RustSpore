"""
Investigate the SPORE anim format (type 0xEE17C6AD).

The anim format is bound to Cal3D, a public animation library. Cal3D file formats
are documented. This tool checks whether SPORE anim resources match Cal3D formats.

Cal3D file types:
  - .csf: Cal3D Skeleton File
  - .cmf: Cal3D Mesh File
  - .crf: Cal3D Resource File
  - .caf: Cal3D Animation File
  - .xsf: Cal3D XML Skeleton File
  - .xaf: Cal3D XML Animation File
  - .xmf: Cal3D XML Mesh File
  - .xrf: Cal3D XML Morph File

Binary Cal3D files have magic headers:
  - CSF: "CAF" (but that's animation...) Actually:
  - The binary formats use specific magic strings

Usage:
    python tools/anim_parse.py "D:/Games/SPORE Collection/Data/Spore_Game.package"
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

ANIM_TYPE = 0xEE17C6AD


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
            if len(raw) < 16:
                continue
            samples.append(raw)
    return samples


def check_cal3d_magic(d: bytes):
    """Check if the data starts with a known Cal3D magic string."""
    # Cal3D binary format magic strings (from cal3d source code)
    magics = {
        b"CAF": "Cal3D Animation File (binary)",
        b"CSF": "Cal3D Skeleton File (binary)",
        b"CMF": "Cal3D Mesh File (binary)",
        b"CRF": "Cal3D Resource File (binary)",
        b"XAF": "Cal3D XML Animation File",
        b"XSF": "Cal3D XML Skeleton File",
        b"XMF": "Cal3D XML Mesh File",
        b"XRF": "Cal3D XML Resource File",
        b"CPF": "Cal3D Physics File",
        b"XPF": "Cal3D XML Physics File",
    }

    for magic, name in magics.items():
        if d[:len(magic)] == magic:
            return magic, name
    return None, None


def statistical_summary(samples):
    """Statistical analysis of anim resource headers."""
    print("=" * 70)
    print("ANIM FORMAT STATISTICAL ANALYSIS")
    print("=" * 70)

    print(f"\n  Samples: {len(samples)}")

    # Check first 4 bytes
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

    # Check for Cal3D magic
    cal3d_count = 0
    cal3d_types = Counter()
    for d in samples:
        magic, name = check_cal3d_magic(d)
        if magic:
            cal3d_count += 1
            cal3d_types[name] += 1

    print(f"\n  Cal3D magic found: {cal3d_count}/{len(samples)}")
    if cal3d_types:
        for name, c in cal3d_types.most_common():
            print(f"    {name}: {c}")

    # Try to interpret as various formats
    print(f"\n  Field analysis (first 5 samples):")
    for i, d in enumerate(samples[:5]):
        print(f"\n    Sample {i+1}: size={len(d)}")
        print(f"      First 32 bytes: {d[:32].hex(' ')}")

        # Try as ASCII
        asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in d[:32])
        print(f"      ASCII: {asc}")

        # Try as u32 LE
        for off in [0, 4, 8, 12, 16, 20, 24, 28]:
            if off + 4 <= len(d):
                u32_le = struct.unpack_from("<I", d, off)[0]
                u32_be = struct.unpack_from(">I", d, off)[0]
                print(f"      u32 @+{off}: LE={u32_le} (0x{u32_le:08X})  BE={u32_be} (0x{u32_be:08X})")

        # Try as u16 LE
        for off in [0, 4, 8, 12, 16, 20, 24, 28]:
            if off + 2 <= len(d):
                u16_le = struct.unpack_from("<H", d, off)[0]
                u16_be = struct.unpack_from(">H", d, off)[0]
                print(f"      u16 @+{off}: LE={u16_le}  BE={u16_be}")


def deep_inspect(samples, n=3):
    """Deep inspection of a few anim samples."""
    sorted_s = sorted(samples, key=len)

    for idx in range(min(n, len(sorted_s))):
        d = sorted_s[idx]
        print(f"\n{'='*70}")
        print(f"Sample {idx+1}: size={len(d)}")
        print(f"{'='*70}")

        # Full hex dump of first 128 bytes
        print(f"  First 128 bytes:")
        for j in range(0, min(128, len(d)), 16):
            chunk = d[j:j+16]
            hexstr = chunk.hex(' ')
            asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
            print(f"    {j:>5}  {hexstr:<47}  {asc}")

        # Check for float data
        print(f"\n  Float scan (LE):")
        for j in range(0, min(64, len(d) - 3), 4):
            f = struct.unpack_from("<f", d, j)[0]
            if abs(f) < 1000 and f != 0:
                print(f"    off {j:>3}: {f:.6f}")

        # Check for string data
        print(f"\n  String scan:")
        current = ""
        for j in range(min(256, len(d))):
            if 32 <= d[j] < 127:
                current += chr(d[j])
            else:
                if len(current) >= 4:
                    print(f"    off {j - len(current):>3}: \"{current}\"")
                current = ""


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    # Collect anim samples from all packages
    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_samples(pkg, ANIM_TYPE, limit=200)
        except (ValueError, struct.error):
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} anim resources")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)} anim resources")

    if all_samples:
        statistical_summary(all_samples)
        deep_inspect(all_samples, n=3)


if __name__ == "__main__":
    raise SystemExit(main())

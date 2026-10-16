"""
Analyze blockdata 84-byte structured data as quantized vertices.

The 48 bytes after the flags (offset 36-83) show repeating patterns with
4 groups of 12 bytes. Each group has changing first/last bytes and constant
middle bytes (66 bb). This might be quantized vertex positions.
"""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

BLOCKDATA_TYPE = 0x5CAA5E28


def get_samples(path: Path, target_type: int, limit: int = 50):
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


def analyze_84byte(samples):
    """Analyze 84-byte blockdata samples."""
    print("=" * 70)
    print("BLOCKDATA 84-BYTE STRUCTURED DATA ANALYSIS")
    print("=" * 70)

    count = 0
    for d in samples:
        if len(d) != 84:
            continue
        count += 1
        if count > 5:
            break

        print(f"\n  Sample (size=84):")

        # Bounding box
        min_xyz = struct.unpack_from("<fff", d, 0)
        max_xyz = struct.unpack_from("<fff", d, 12)
        print(f"    BBox: min=({min_xyz[0]:.4f},{min_xyz[1]:.4f},{min_xyz[2]:.4f}) "
              f"max=({max_xyz[0]:.4f},{max_xyz[1]:.4f},{max_xyz[2]:.4f})")

        # 7th float
        f24 = struct.unpack_from("<f", d, 24)[0]
        print(f"    Float@24: {f24:.6f}")

        # Flags
        flags = d[28:32]
        print(f"    Flags: {flags.hex(' ')}")

        # Structured data (offset 36-83, 48 bytes)
        data = d[36:84]
        print(f"    Structured data (48 bytes): {data.hex(' ')}")

        # Try as BE u16 array
        u16s = [struct.unpack_from(">H", data, j)[0] for j in range(0, 48, 2)]
        print(f"    As BE u16: {u16s}")

        # Try as LE u16 array
        u16s_le = [struct.unpack_from("<H", data, j)[0] for j in range(0, 48, 2)]
        print(f"    As LE u16: {u16s_le}")

        # Try as 4 groups of 12 bytes
        print(f"    As 4 groups of 12 bytes:")
        for g in range(4):
            group = data[g*12:(g+1)*12]
            hexstr = group.hex(' ')
            # Parse as 6 BE u16
            vals = [struct.unpack_from(">H", group, j)[0] for j in range(0, 12, 2)]
            print(f"      Group {g}: {hexstr}  BE u16: {vals}")

        # Check if the data could be quantized coordinates
        # Normalize BE u16 values to 0-1 range and compare with bbox
        print(f"    Normalized BE u16 (val/65535) compared to bbox:")
        for g in range(4):
            group = data[g*12:(g+1)*12]
            vals = [struct.unpack_from(">H", group, j)[0] / 65535.0 for j in range(0, 12, 2)]
            # Map to bbox
            mapped = []
            for i, v in enumerate(vals):
                if i % 2 == 0:  # x-like
                    mapped.append(min_xyz[0] + v * (max_xyz[0] - min_xyz[0]))
                else:  # y-like
                    mapped.append(min_xyz[1] + v * (max_xyz[1] - min_xyz[1]))
            print(f"      Group {g} mapped: {['%.4f' % m for m in mapped[:6]]}")


def analyze_patterns(samples):
    """Look for patterns across all 84-byte samples."""
    print(f"\n{'='*70}")
    print("PATTERN ANALYSIS ACROSS 84-BYTE SAMPLES")
    print(f"{'='*70}")

    # Check if bytes at specific positions are constant
    positions = {}
    for d in samples:
        if len(d) != 84:
            continue
        for j in range(36, 84):
            if j not in positions:
                positions[j] = set()
            positions[j].add(d[j])

    constant_bytes = {j: vals.pop() for j, vals in positions.items() if len(vals) == 1}
    variable_bytes = {j: len(vals) for j, vals in positions.items() if len(vals) > 1}

    print(f"\n  Constant bytes in structured data (offset 36-83):")
    for j in sorted(constant_bytes):
        print(f"    offset {j}: 0x{constant_bytes[j]:02X}")

    print(f"\n  Variable bytes (offset 36-83):")
    for j in sorted(variable_bytes):
        print(f"    offset {j}: {variable_bytes[j]} distinct values")


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_samples(pkg, BLOCKDATA_TYPE, limit=50)
        except (ValueError, struct.error):
            continue
        if samples:
            all_samples.extend(samples)

    print(f"  Total: {len(all_samples)} blockdata samples")
    if all_samples:
        analyze_84byte(all_samples)
        analyze_patterns(all_samples)


if __name__ == "__main__":
    raise SystemExit(main())

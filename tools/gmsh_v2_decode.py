"""
Robust gmsh v2+ index section decoder using header fields.

Instead of searching for valid u16 indices (which can find false positives),
read the count and stride directly from the header and calculate the expected
index data start position.

Formula:
  header_size = 16 + version * 4
  count = BE u16 at idx_offset + header_size
  stride = BE u16 at idx_offset + header_size + 2
  matrix_data_end = idx_offset + header_size + 4 + count * stride
  index_data_start = matrix_data_end + 8 (footer)
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

GMSH_TYPE = 0x01C135DA


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
            if len(raw) < 24:
                continue
            samples.append(raw)
    return samples


def decode_v2_index(d, idx_offset):
    """Decode v2+ index section using header fields."""
    version = struct.unpack_from(">I", d, idx_offset + 4)[0]
    header_size = 16 + version * 4

    # Read count and stride from header
    count_offset = idx_offset + header_size
    if count_offset + 4 > len(d):
        return None

    count = struct.unpack_from(">H", d, count_offset)[0]
    stride = struct.unpack_from(">H", d, count_offset + 2)[0]

    # Calculate index data start
    matrix_end = count_offset + 4 + count * stride
    index_start = matrix_end + 8  # 8-byte footer

    return {
        "version": version,
        "header_size": header_size,
        "count": count,
        "stride": stride,
        "matrix_end": matrix_end,
        "index_start": index_start,
    }


def validate(samples):
    """Validate v2+ index section decoding across all samples."""
    print("=" * 70)
    print("GMSH V2+ INDEX SECTION DECODE (header-based)")
    print("=" * 70)

    total_v2plus = 0
    valid = 0
    invalid = 0
    version_dist = Counter()
    stride_dist = Counter()
    details = []

    for d in samples:
        idx_offset = d.find(b"\x15\x00\x06\x08")
        if idx_offset < 0:
            continue

        version = struct.unpack_from(">I", d, idx_offset + 4)[0]
        if version == 1:
            continue

        total_v2plus += 1
        version_dist[version] += 1

        result = decode_v2_index(d, idx_offset)
        if result is None:
            invalid += 1
            continue

        vcount = struct.unpack_from(">H", d, 20)[0]
        index_start = result["index_start"]

        # Validate: index_start should be within the file
        if index_start >= len(d):
            invalid += 1
            if len(details) < 10:
                details.append({
                    "version": version, "size": len(d),
                    "count": result["count"], "stride": result["stride"],
                    "index_start": index_start, "status": "OVERFLOW",
                })
            continue

        # Validate: u16 indices at index_start should be < vcount
        remaining = len(d) - index_start
        num_indices = remaining // 2
        ok_count = 0
        for j in range(min(num_indices, 50)):
            v = struct.unpack_from("<H", d, index_start + j * 2)[0]
            if v >= vcount:
                break
            ok_count += 1

        if ok_count >= 10:
            valid += 1
            stride_dist[result["stride"]] += 1
            if len(details) < 10:
                details.append({
                    "version": version, "size": len(d),
                    "count": result["count"], "stride": result["stride"],
                    "index_start": index_start, "status": f"OK ({ok_count}+ valid)",
                    "vcount": vcount,
                })
        else:
            invalid += 1
            if len(details) < 20:
                details.append({
                    "version": version, "size": len(d),
                    "count": result["count"], "stride": result["stride"],
                    "index_start": index_start,
                    "status": f"INVALID ({ok_count} valid, need 10+)",
                    "vcount": vcount,
                })

    print(f"\n  Total v2+ samples: {total_v2plus}")
    print(f"  Valid: {valid} ({valid/total_v2plus*100:.1f}%)")
    print(f"  Invalid: {invalid} ({invalid/total_v2plus*100:.1f}%)")
    print(f"  Version distribution: {dict(version_dist)}")
    print(f"  Stride distribution (valid samples): {dict(stride_dist)}")

    print(f"\n  Details (first 20):")
    for det in details:
        print(f"    v{det['version']} size={det['size']} "
              f"count={det['count']} stride={det['stride']} "
              f"idx_start={det['index_start']} {det['status']}")

    return valid, total_v2plus


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_samples(pkg, GMSH_TYPE, limit=500)
        except (ValueError, struct.error):
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} gmsh")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)}")
    validate(all_samples)


if __name__ == "__main__":
    raise SystemExit(main())

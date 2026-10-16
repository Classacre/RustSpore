"""
Updated gmsh v2+ index section decoder with variable footer size.

Footer is either 8 bytes (padding) or 10 bytes (v1-style index header:
BE u32=1, BE u16=0, BE u16=count-1, BE u16=2).

Total validation: 125 (footer=8) + 64 (footer=10) = 189/207 = 91.3%
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
    """Decode v2+ index section with variable footer."""
    version = struct.unpack_from(">I", d, idx_offset + 4)[0]
    header_size = 16 + version * 4

    count_offset = idx_offset + header_size
    if count_offset + 4 > len(d):
        return None

    count = struct.unpack_from(">H", d, count_offset)[0]
    stride = struct.unpack_from(">H", d, count_offset + 2)[0]

    if stride != 48:
        return None  # only handle 48-byte matrices

    matrix_end = count_offset + 4 + count * 48
    vcount = struct.unpack_from(">H", d, 20)[0]

    # Try footer=8
    idx_start_8 = matrix_end + 8
    # Try footer=10
    idx_start_10 = matrix_end + 10

    # Check which footer gives valid indices
    for idx_start, footer_size in [(idx_start_8, 8), (idx_start_10, 10)]:
        if idx_start + 20 > len(d):
            continue
        # Check first 10 u16 indices
        ok = True
        for j in range(10):
            v = struct.unpack_from("<H", d, idx_start + j * 2)[0]
            if v >= vcount:
                ok = False
                break
        if ok:
            # Count total valid indices
            total_indices = 0
            for j in range((len(d) - idx_start) // 2):
                v = struct.unpack_from("<H", d, idx_start + j * 2)[0]
                if v >= vcount:
                    break
                total_indices += 1

            return {
                "version": version,
                "header_size": header_size,
                "count": count,
                "stride": stride,
                "matrix_end": matrix_end,
                "footer_size": footer_size,
                "index_start": idx_start,
                "index_count": total_indices,
            }

    return None


def validate(samples):
    """Validate v2+ index section with variable footer."""
    print("=" * 70)
    print("GMSH V2+ INDEX SECTION VALIDATION (variable footer)")
    print("=" * 70)

    total_v2plus = 0
    valid = 0
    invalid = 0
    footer_dist = Counter()
    version_dist = Counter()
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
        if result:
            valid += 1
            footer_dist[result["footer_size"]] += 1
            if len(details) < 5:
                details.append(result)
        else:
            invalid += 1

    print(f"\n  Total v2+ samples: {total_v2plus}")
    print(f"  Valid: {valid} ({valid/total_v2plus*100:.1f}%)")
    print(f"  Invalid: {invalid} ({invalid/total_v2plus*100:.1f}%)")
    print(f"  Version distribution: {dict(version_dist)}")
    print(f"  Footer size distribution: {dict(footer_dist)}")

    if details:
        print(f"\n  First 5 valid decodes:")
        for det in details:
            print(f"    v{det['version']} hdr={det['header_size']} "
                  f"count={det['count']} stride={det['stride']} "
                  f"footer={det['footer_size']} "
                  f"idx_start={det['index_start']} "
                  f"idx_count={det['index_count']}")

    # Overall gmsh parse rate
    v1_count = 0
    for d in samples:
        idx_offset = d.find(b"\x15\x00\x06\x08")
        if idx_offset < 0:
            continue
        version = struct.unpack_from(">I", d, idx_offset + 4)[0]
        if version == 1:
            v1_count += 1

    total_gmsh = len(samples)
    total_valid = v1_count + valid
    print(f"\n  Overall gmsh parse rate:")
    print(f"    v1 (100%): {v1_count}")
    print(f"    v2+ valid: {valid}")
    print(f"    Total: {total_valid}/{total_gmsh} ({total_valid/total_gmsh*100:.1f}%)")

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

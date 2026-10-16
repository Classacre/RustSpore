"""
Investigate gmsh v2+ invalid samples: find actual footer size.

For samples where the predicted index data start (header + 4 + count*48 + 8)
doesn't have valid indices, search for where valid u16 indices actually start
and compute the offset difference. This reveals the actual footer size.
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


def find_valid_indices(d, start, vcount, min_count=20):
    """Find the first position >= start where valid u16 indices begin."""
    for off in range(start, min(len(d) - min_count * 2, start + 200), 2):
        count = 0
        for j in range(off, len(d) - 1, 2):
            v = struct.unpack_from("<H", d, j)[0]
            if v >= vcount:
                break
            count += 1
        if count >= min_count:
            return off, count
    return None, 0


def investigate(samples):
    """For invalid v2+ samples, find actual index data start."""
    print("=" * 70)
    print("GMSH V2+ FOOTER SIZE INVESTIGATION")
    print("=" * 70)

    footer_diffs = Counter()
    details = []

    for d in samples:
        idx_offset = d.find(b"\x15\x00\x06\x08")
        if idx_offset < 0:
            continue

        version = struct.unpack_from(">I", d, idx_offset + 4)[0]
        if version == 1:
            continue

        header_size = 16 + version * 4
        count_offset = idx_offset + header_size
        if count_offset + 4 > len(d):
            continue

        count = struct.unpack_from(">H", d, count_offset)[0]
        stride = struct.unpack_from(">H", d, count_offset + 2)[0]

        if stride != 48:
            continue  # only investigate stride=48 samples

        vcount = struct.unpack_from(">H", d, 20)[0]
        matrix_end = count_offset + 4 + count * 48

        # Predicted index start with 8-byte footer
        predicted_start = matrix_end + 8

        # Check if predicted start has valid indices
        actual_start, actual_count = find_valid_indices(d, predicted_start, vcount)

        if actual_start is not None:
            # Found valid indices at actual_start
            diff = actual_start - matrix_end
            footer_diffs[diff] += 1
            if len(details) < 15:
                details.append({
                    "version": version, "size": len(d), "vcount": vcount,
                    "count": count, "matrix_end": matrix_end,
                    "predicted": predicted_start,
                    "actual": actual_start, "footer": diff,
                    "idx_count": actual_count,
                })
        else:
            # No valid indices found after matrix_end
            # Try searching from matrix_end itself
            actual_start2, actual_count2 = find_valid_indices(d, matrix_end, vcount)
            if actual_start2 is not None:
                diff = actual_start2 - matrix_end
                footer_diffs[diff] += 1
                if len(details) < 15:
                    details.append({
                        "version": version, "size": len(d), "vcount": vcount,
                        "count": count, "matrix_end": matrix_end,
                        "predicted": predicted_start,
                        "actual": actual_start2, "footer": diff,
                        "idx_count": actual_count2,
                        "note": "found after matrix_end (no 8-byte footer)",
                    })
            else:
                footer_diffs["NOT_FOUND"] += 1
                if len(details) < 15:
                    details.append({
                        "version": version, "size": len(d), "vcount": vcount,
                        "count": count, "matrix_end": matrix_end,
                        "predicted": predicted_start,
                        "actual": None, "footer": "NOT_FOUND",
                    })

    print(f"\n  Footer size distribution (matrix_end to index_data_start):")
    for diff, cnt in footer_diffs.most_common(20):
        print(f"    {diff}: {cnt}")

    print(f"\n  Details (first 15):")
    for det in details:
        note = det.get("note", "")
        print(f"    v{det['version']} size={det['size']} vc={det['vcount']} "
              f"count={det['count']} matrix_end={det['matrix_end']} "
              f"footer={det['footer']} idx_count={det.get('idx_count', '?')} {note}")

    # For samples where footer != 8, dump the footer bytes
    print(f"\n  Footer bytes for non-8 footers:")
    shown = 0
    for d in samples:
        idx_offset = d.find(b"\x15\x00\x06\x08")
        if idx_offset < 0:
            continue
        version = struct.unpack_from(">I", d, idx_offset + 4)[0]
        if version == 1:
            continue
        header_size = 16 + version * 4
        count_offset = idx_offset + header_size
        if count_offset + 4 > len(d):
            continue
        count = struct.unpack_from(">H", d, count_offset)[0]
        stride = struct.unpack_from(">H", d, count_offset + 2)[0]
        if stride != 48:
            continue
        vcount = struct.unpack_from(">H", d, 20)[0]
        matrix_end = count_offset + 4 + count * 48
        actual_start, _ = find_valid_indices(d, matrix_end, vcount)
        if actual_start is not None:
            diff = actual_start - matrix_end
            if diff != 8 and shown < 5:
                print(f"\n    v{version} size={len(d)} footer={diff}:")
                footer_bytes = d[matrix_end:actual_start]
                print(f"      Footer hex: {footer_bytes.hex(' ')}")
                # Also dump first 32 bytes of index data
                idx_bytes = d[actual_start:actual_start + 32]
                print(f"      Index data: {idx_bytes.hex(' ')}")
                shown += 1


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
    investigate(all_samples)


if __name__ == "__main__":
    raise SystemExit(main())

"""
Investigate the gap between the index section magic and the actual index data
for version 2+ gmsh samples.

For version 2, the index data starts at offset 34214 (228 bytes after the magic
at 33986). The gap contains sub-section headers and bounding box floats.

This script analyzes the gap structure across multiple version 2+ samples.
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


def find_index_section(d):
    """Find the index section and return its offset."""
    pos = d.find(b"\x15\x00\x06\x08")
    return pos if pos >= 0 else None


def find_index_data_start(d, idx_offset, vcount):
    """Find where valid u16 index data starts after the index section."""
    total = len(d)
    version = struct.unpack_from(">I", d, idx_offset + 4)[0]

    # Search for a run of valid u16 indices
    best_start = None
    best_count = 0

    for off in range(idx_offset + 24, min(total - 4, idx_offset + 400)):
        count = 0
        for j in range(off, total - 1, 2):
            v = struct.unpack_from("<H", d, j)[0]
            if v >= vcount:
                break
            count += 1
        if count > best_count:
            best_count = count
            best_start = off

    return best_start, best_count, version


def analyze_gap(samples, n=10):
    """Analyze the gap between index magic and index data for v2+ samples."""
    print("=" * 70)
    print("GMSH INDEX SECTION GAP ANALYSIS (v2+)")
    print("=" * 70)

    gap_sizes = Counter()
    versions = Counter()
    details = []

    for d in samples:
        idx_offset = find_index_section(d)
        if idx_offset is None:
            continue

        version = struct.unpack_from(">I", d, idx_offset + 4)[0]
        if version == 1:
            continue  # skip v1, already decoded

        vcount = struct.unpack_from(">H", d, 20)[0]
        data_start, data_count, ver = find_index_data_start(d, idx_offset, vcount)

        if data_start is None:
            continue

        gap = data_start - idx_offset
        gap_sizes[gap] += 1
        versions[version] += 1

        if len(details) < n:
            details.append({
                "size": len(d),
                "vcount": vcount,
                "version": version,
                "idx_offset": idx_offset,
                "gap": gap,
                "data_start": data_start,
                "data_count": data_count,
                "first_bytes": d[idx_offset:idx_offset+min(64, gap)].hex(),
            })

    print(f"\n  Version distribution: {dict(versions)}")
    print(f"\n  Gap size distribution (top 10):")
    for v, c in gap_sizes.most_common(10):
        print(f"    {v}: {c}")

    if details:
        print(f"\n  First {len(details)} v2+ samples:")
        for det in details:
            print(f"\n    size={det['size']}, vcount={det['vcount']}, "
                  f"version={det['version']}")
            print(f"      idx_offset={det['idx_offset']}, gap={det['gap']}, "
                  f"data_start={det['data_start']}, data_count={det['data_count']}")
            print(f"      Gap bytes: {det['first_bytes'][:128]}")

    # Analyze gap structure for one sample in detail
    if details:
        det = details[0]
        d = None
        for s in samples:
            if len(s) == det["size"]:
                d = s
                break

        if d:
            idx_offset = det["idx_offset"]
            gap = det["gap"]
            print(f"\n  Detailed gap analysis for size={det['size']}:")
            print(f"  Gap is {gap} bytes from offset {idx_offset} to {idx_offset + gap}")

            # Dump gap bytes
            for j in range(idx_offset, idx_offset + min(gap, 128), 16):
                chunk = d[j:j+16]
                hexstr = chunk.hex(' ')
                asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
                print(f"    {j:>6}  {hexstr:<47}  {asc}")

            # Try to parse gap as sub-sections
            print(f"\n  Trying to parse gap as sub-sections:")
            offset = idx_offset + 24  # skip main index header
            sub_count = 0
            while offset < idx_offset + gap - 8 and sub_count < 20:
                magic = d[offset:offset+4]
                vc = struct.unpack_from(">I", d, offset+4)[0]
                stride = struct.unpack_from(">H", d, offset+8)[0]
                data_size = vc * stride
                end = offset + 10 + data_size

                print(f"    Sub {sub_count}: magic={magic.hex()}, vcount={vc}, "
                      f"stride={stride}, size={10 + data_size}")

                if end > idx_offset + gap + 100:  # allow some overflow
                    print(f"      OVERFLOW: end={end} > gap_end={idx_offset + gap}")
                    break

                offset = end
                sub_count += 1

            # Also try: gap might contain bounding box data
            print(f"\n  Gap as floats:")
            for j in range(idx_offset + 24, idx_offset + min(gap, 88), 4):
                f = struct.unpack_from("<f", d, j)[0]
                if abs(f) < 10000:
                    print(f"    +{j - idx_offset}: {f:.4f}")


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
    analyze_gap(all_samples, n=10)


if __name__ == "__main__":
    raise SystemExit(main())

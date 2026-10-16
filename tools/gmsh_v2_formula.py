"""
Systematic gmsh v2+ index section decoder.

Hypothesis: gap = 16 + version*4 + 4 (count/stride) + N*48 (matrices) + 8 (footer)

Where:
  - 16 = common header (magic + version + 1024 + 256)
  - version*4 = per-version attribute fields
  - 4 = count (BE u16) + stride (BE u16 = 48)
  - N*48 = N 3x4 transformation matrices
  - 8 = footer

This script tests this formula across all v2+ samples and reports fit rate.
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


def find_index_data_start(d, idx_offset, vcount):
    """Find where valid u16 index data starts after the index section."""
    total = len(d)
    best_start = None
    best_count = 0

    for off in range(idx_offset + 24, min(total - 4, idx_offset + 600)):
        count = 0
        for j in range(off, total - 1, 2):
            v = struct.unpack_from("<H", d, j)[0]
            if v >= vcount:
                break
            count += 1
        if count > best_count:
            best_count = count
            best_start = off

    return best_start, best_count


def test_formula(samples):
    """Test: gap = 16 + version*4 + 4 + N*48 + 8."""
    print("=" * 70)
    print("GMSH V2+ INDEX SECTION FORMULA TEST")
    print("=" * 70)

    fits = 0
    no_fit = 0
    version_dist = Counter()
    details = []

    for d in samples:
        idx_offset = d.find(b"\x15\x00\x06\x08")
        if idx_offset < 0:
            continue

        version = struct.unpack_from(">I", d, idx_offset + 4)[0]
        if version == 1:
            continue

        vcount = struct.unpack_from(">H", d, 20)[0]
        data_start, data_count = find_index_data_start(d, idx_offset, vcount)
        if data_start is None:
            continue

        gap = data_start - idx_offset
        version_dist[version] += 1

        # Test formula: gap = 16 + version*4 + 4 + N*48 + 8
        # = 28 + version*4 + N*48
        remainder = gap - 28 - version * 4
        if remainder >= 0 and remainder % 48 == 0:
            n = remainder // 48
            fits += 1
            if len(details) < 15:
                details.append({
                    "version": version, "gap": gap, "N": n,
                    "size": len(d), "vcount": vcount,
                    "data_count": data_count,
                    "header_size": 16 + version * 4,
                })
        else:
            no_fit += 1
            if len(details) < 30:
                details.append({
                    "version": version, "gap": gap, "N": None,
                    "size": len(d), "vcount": vcount,
                    "data_count": data_count,
                    "header_size": 16 + version * 4,
                    "remainder": remainder,
                })

    total = fits + no_fit
    print(f"\n  Total v2+ samples: {total}")
    print(f"  Formula fits: {fits} ({fits/total*100:.1f}%)")
    print(f"  No fit: {no_fit} ({no_fit/total*100:.1f}%)")
    print(f"  Version distribution: {dict(version_dist)}")

    print(f"\n  Details (first 30):")
    for det in details:
        if det["N"] is not None:
            print(f"    v{det['version']} gap={det['gap']} N={det['N']} "
                  f"hdr={det['header_size']} size={det['size']} "
                  f"vc={det['vcount']} idx={det['data_count']}")
        else:
            print(f"    v{det['version']} gap={det['gap']} NO-FIT "
                  f"remainder={det.get('remainder', '?')} "
                  f"hdr={det['header_size']} size={det['size']} "
                  f"vc={det['vcount']} idx={det['data_count']}")

    # For no-fit samples, try alternative formulas
    print(f"\n  Alternative formulas for no-fit samples:")
    alt_fits = Counter()
    for d in samples:
        idx_offset = d.find(b"\x15\x00\x06\x08")
        if idx_offset < 0:
            continue
        version = struct.unpack_from(">I", d, idx_offset + 4)[0]
        if version == 1:
            continue
        vcount = struct.unpack_from(">H", d, 20)[0]
        data_start, _ = find_index_data_start(d, idx_offset, vcount)
        if data_start is None:
            continue
        gap = data_start - idx_offset

        # Try: gap = 24 + N*48 + 8 (v2 style, ignore version*4)
        r = gap - 24 - 4 - 8
        if r >= 0 and r % 48 == 0:
            alt_fits["24+4+N*48+8"] += 1
            continue

        # Try: gap = 24 + N*48 (no footer)
        r = gap - 24 - 4
        if r >= 0 and r % 48 == 0:
            alt_fits["24+4+N*48"] += 1
            continue

        # Try: gap = header + N*48 (no count/stride, no footer)
        header = 16 + version * 4
        r = gap - header
        if r >= 0 and r % 48 == 0:
            alt_fits[f"hdr({header})+N*48"] += 1
            continue

        # Try: gap = header + 4 + N*48 (no footer)
        r = gap - header - 4
        if r >= 0 and r % 48 == 0:
            alt_fits[f"hdr({header})+4+N*48"] += 1
            continue

        # Try: different matrix sizes
        for mat_size in [32, 36, 40, 44, 52, 56, 60, 64]:
            header = 16 + version * 4
            r = gap - header - 4 - 8
            if r >= 0 and r % mat_size == 0:
                alt_fits[f"hdr+4+N*{mat_size}+8"] += 1
                break
        else:
            alt_fits["no_fit"] += 1

    print(f"\n  Alternative formula results:")
    for pattern, count in alt_fits.most_common():
        print(f"    {pattern}: {count}")


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
    test_formula(all_samples)


if __name__ == "__main__":
    raise SystemExit(main())

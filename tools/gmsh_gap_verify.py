"""
Verify the v2+ index section gap structure.

Hypothesis: gap = 24 (main header) + 4 (sub-header: count + stride=48) +
count * 48 (3x4 matrix data) + 8 (footer)

But the sub-header position varies by version. Let me check if the structure
is: 24 + 4 + N*48 + 8 = gap_size for various N.
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
    """Find where valid u16 index data starts."""
    total = len(d)
    best_start = None
    best_count = 0

    for off in range(idx_offset + 24, min(total - 4, idx_offset + 500)):
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


def verify_structure(samples):
    """Verify: gap = 24 + 4 + N*48 + 8."""
    print("=" * 70)
    print("V2+ GAP STRUCTURE VERIFICATION")
    print("=" * 70)

    fits = Counter()
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

        # Try: gap = 24 + 4 + N*48 + 8
        remainder = gap - 24 - 4 - 8
        if remainder >= 0 and remainder % 48 == 0:
            n = remainder // 48
            fits["24+4+N*48+8"] += 1
            if len(details) < 10:
                details.append({
                    "version": version, "gap": gap, "N": n,
                    "size": len(d), "vcount": vcount,
                    "data_count": data_count,
                })
        else:
            # Try: gap = 24 + N*48 + 8
            remainder2 = gap - 24 - 8
            if remainder2 >= 0 and remainder2 % 48 == 0:
                n = remainder2 // 48
                fits["24+N*48+8"] += 1
            else:
                # Try: gap = 24 + N*48
                remainder3 = gap - 24
                if remainder3 >= 0 and remainder3 % 48 == 0:
                    n = remainder3 // 48
                    fits["24+N*48"] += 1
                else:
                    # Try: gap = 24 + 4 + N*48
                    remainder4 = gap - 24 - 4
                    if remainder4 >= 0 and remainder4 % 48 == 0:
                        n = remainder4 // 48
                        fits["24+4+N*48"] += 1
                    else:
                        fits["no_fit"] += 1

    print(f"\n  Structure fit results:")
    for pattern, count in fits.most_common():
        print(f"    {pattern}: {count}")

    if details:
        print(f"\n  First 10 fitting samples:")
        for det in details:
            print(f"    version={det['version']}, gap={det['gap']}, N={det['N']}, "
                  f"size={det['size']}, vcount={det['vcount']}, "
                  f"data_count={det['data_count']}")

    # Also check: does the version correlate with N?
    version_n = Counter()
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
        remainder = gap - 24 - 4 - 8
        if remainder >= 0 and remainder % 48 == 0:
            n = remainder // 48
            version_n[(version, n)] += 1

    print(f"\n  Version vs N (matrix count) distribution:")
    for (ver, n), cnt in sorted(version_n.items()):
        print(f"    version={ver}, N={n}: {cnt}")


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
    verify_structure(all_samples)


if __name__ == "__main__":
    raise SystemExit(main())

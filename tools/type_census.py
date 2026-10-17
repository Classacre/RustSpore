#!/usr/bin/env python3
"""Census all resource types across all packages.

Identifies which types have been decoded and which remain unknown,
with sample counts and size distributions.
"""

import struct
import sys
from pathlib import Path
from collections import Counter, defaultdict

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")

# Known types with their decode status
KNOWN = {
    0x2F7D0004: "PNG (decoded)",
    0x00B1B104: "prop (97.38%)",
    0x2F4E681B: "RW4 (header decoded)",
    0x2F4E681C: "RASTER (decoded)",
    0x01C135DA: "gmsh (96.6%)",
    0xEE17C6AD: "anim (100%)",
    0x5CAA5E28: "blockdata (partial)",
    0xDFAD9F51: "string_table (100%)",
}


def main():
    type_counts = Counter()
    type_sizes = defaultdict(list)
    type_packages = defaultdict(set)
    type_samples = defaultdict(list)
    total_resources = 0

    for pkg in sorted(BASE.glob("*.package")):
        try:
            entries, _ = parse_index(pkg)
        except Exception:
            continue

        with pkg.open("rb") as fh:
            for e in entries:
                t = e["type"]
                type_counts[t] += 1
                total_resources += 1
                type_packages[t].add(pkg.name)

                # Collect a few samples per type for size analysis
                if len(type_samples[t]) < 5:
                    fh.seek(e["offset"])
                    raw = fh.read(e["stored"])
                    if e["dec"] > e["stored"]:
                        try:
                            raw = decompress(raw, out_size=None, header=True)[0]
                        except Exception:
                            continue
                    type_sizes[t].append(len(raw))
                    type_samples[t].append(raw)

    print(f"Total resources: {total_resources}")
    print(f"Distinct types: {len(type_counts)}")
    print()

    # Sort by count descending
    print("=" * 100)
    print(f"{'Type':<12} {'Count':>7}  {'Status':<30}  {'Packages'}")
    print("=" * 100)

    for t, count in type_counts.most_common():
        status = KNOWN.get(t, "*** UNKNOWN ***")
        pkgs = ", ".join(sorted(type_packages[t]))
        print(f"0x{t:08X}  {count:>7}  {status:<30}  {pkgs}")

    print()
    print("=" * 100)
    print("Unknown types (top 20 by count):")
    print("=" * 100)
    for t, count in type_counts.most_common():
        if t not in KNOWN:
            sizes = type_sizes.get(t, [])
            size_str = ", ".join(str(s) for s in sizes[:5])
            # Show first 16 bytes of first sample
            sample = type_samples.get(t, [None])[0]
            hex_str = ""
            if sample:
                hex_str = " ".join(f"{b:02x}" for b in sample[:16])
            print(f"0x{t:08X}  count={count:>6}  sizes=[{size_str}]  first16=[{hex_str}]")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Validate gmsh v2+ stride=2 index sections.

Key insight: stride=2 samples use the same header formula (16 + version*4)
but the data is direct u16 LE indices, not 48-byte matrices.
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
GMSH_TYPE = 0x01C135DA


def collect_gmsh(limit=2000):
    samples = []
    for pkg in sorted(BASE.glob("*.package")):
        try:
            entries, _ = parse_index(pkg)
        except Exception:
            continue
        with pkg.open("rb") as fh:
            for e in entries:
                if e["type"] != GMSH_TYPE:
                    continue
                fh.seek(e["offset"])
                raw = fh.read(e["stored"])
                if e["dec"] > e["stored"]:
                    try:
                        raw = decompress(raw, out_size=None, header=True)[0]
                    except Exception:
                        continue
                samples.append(raw)
                if len(samples) >= limit:
                    return samples
    return samples


def parse_gmsh(d):
    """Parse gmsh header, attributes, and index section."""
    if len(d) < 24:
        return None

    vcount = struct.unpack_from(">H", d, 20)[0]
    pos_stride = struct.unpack_from(">H", d, 22)[0]

    # Position data
    off = 24 + vcount * pos_stride

    # Skip attribute sections
    attrs = []
    while off + 10 <= len(d):
        magic = d[off:off+4]
        if magic == b"\x15\x00\x06\x08":
            break
        attr_vcount = struct.unpack_from(">I", d, off + 4)[0]
        attr_stride = struct.unpack_from(">H", d, off + 8)[0]
        if attr_vcount > 10000 or attr_stride > 100:
            break
        attrs.append({"magic": magic.hex(), "vcount": attr_vcount, "stride": attr_stride})
        off += 10 + attr_vcount * attr_stride

    if off + 4 > len(d) or d[off:off+4] != b"\x15\x00\x06\x08":
        return None

    # Index section
    idx_off = off
    version = struct.unpack_from(">I", d, idx_off + 4)[0]

    if version == 1:
        # v1: 24-byte header, count at +20, stride at +22
        count = struct.unpack_from(">H", d, idx_off + 20)[0]
        stride = struct.unpack_from(">H", d, idx_off + 22)[0]
        data_off = idx_off + 24
    else:
        # v2+: header = 16 + version*4, then sub-header (count + stride)
        hdr_size = 16 + version * 4
        sub_off = idx_off + hdr_size
        if sub_off + 4 > len(d):
            return None
        count = struct.unpack_from(">H", d, sub_off)[0]
        stride = struct.unpack_from(">H", d, sub_off + 2)[0]
        data_off = sub_off + 4

    # Read index data
    data_size = count * stride
    if data_off + data_size > len(d):
        return None

    indices = []
    if stride == 2:
        for i in range(count):
            idx = struct.unpack_from("<H", d, data_off + i * 2)[0]
            indices.append(idx)

    # Check footer
    footer_start = data_off + data_size
    footer_size = len(d) - footer_start
    footer_ff = False
    if footer_size >= 8:
        footer_ff = d[-8:] == b"\xff" * 8

    return {
        "vcount": vcount,
        "version": version,
        "count": count,
        "stride": stride,
        "indices": indices,
        "data_off": data_off,
        "data_size": data_size,
        "footer_size": footer_size,
        "footer_ff": footer_ff,
        "exact_fit": footer_size == 0,
        "total_size": len(d),
        "attrs": attrs,
    }


def main():
    samples = collect_gmsh(2000)
    print(f"Collected {len(samples)} gmsh samples\n")

    # Parse all samples
    results = []
    for i, d in enumerate(samples):
        parsed = parse_gmsh(d)
        if parsed:
            parsed["sample_idx"] = i
            results.append(parsed)

    print(f"Successfully parsed: {len(results)}/{len(samples)} ({100*len(results)/len(samples):.1f}%)\n")

    # Categorize by version and stride
    by_ver_stride = Counter()
    for r in results:
        by_ver_stride[(r["version"], r["stride"])] += 1

    print("Version/stride distribution:")
    for (v, s), c in sorted(by_ver_stride.items()):
        print(f"  version={v}, stride={s}: {c}")
    print()

    # Validate stride=2 samples
    stride2 = [r for r in results if r["stride"] == 2]
    print(f"Stride=2 samples: {len(stride2)}")

    if stride2:
        # Check if all indices are valid (< vcount)
        all_valid = 0
        some_invalid = 0
        for r in stride2:
            if r["indices"] and all(idx < r["vcount"] for idx in r["indices"]):
                all_valid += 1
            else:
                some_invalid += 1

        print(f"  All indices valid (< vcount): {all_valid}")
        print(f"  Some indices invalid: {some_invalid}")

        # Footer analysis
        footer_dist = Counter()
        for r in stride2:
            footer_dist[r["footer_size"]] += 1
        print(f"  Footer sizes: {dict(footer_dist.most_common(10))}")

        # Exact fit
        exact = sum(1 for r in stride2 if r["exact_fit"])
        print(f"  Exact fit (footer=0): {exact}/{len(stride2)} ({100*exact/len(stride2):.1f}%)")

        # Show a few examples
        print("\n  Example stride=2 samples:")
        for r in stride2[:3]:
            idx_preview = r["indices"][:10]
            print(f"    vcount={r['vcount']}, count={r['count']}, "
                  f"footer={r['footer_size']}, first10={idx_preview}")

    # Overall exact fit
    exact_all = sum(1 for r in results if r["exact_fit"])
    print(f"\nOverall exact fit: {exact_all}/{len(results)} ({100*exact_all/len(results):.1f}%)")

    # v1 exact fit
    v1 = [r for r in results if r["version"] == 1]
    v1_exact = sum(1 for r in v1 if r["exact_fit"])
    print(f"V1 exact fit: {v1_exact}/{len(v1)} ({100*v1_exact/len(v1):.1f}%)")

    # v2+ exact fit
    v2plus = [r for r in results if r["version"] != 1]
    v2_exact = sum(1 for r in v2plus if r["exact_fit"])
    print(f"V2+ exact fit: {v2_exact}/{len(v2plus)} ({100*v2_exact/len(v2plus):.1f}%)")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Complete gmsh parser with unified index section handling.

Structure:
- stride=2: direct u16 LE indices after sub-header
- stride=48: 3x4 matrices, then v1-style header + u16 LE indices in footer
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


def parse_full(d):
    """Parse complete gmsh format including index data."""
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
        # Validate: section must fit within file, stride must be reasonable
        if attr_stride == 0 or attr_stride > 256:
            break
        if off + 10 + attr_vcount * attr_stride > len(d):
            break
        attrs.append({"vcount": attr_vcount, "stride": attr_stride})
        off += 10 + attr_vcount * attr_stride

    if off + 4 > len(d) or d[off:off+4] != b"\x15\x00\x06\x08":
        return None

    # Index section
    idx_off = off
    version = struct.unpack_from(">I", d, idx_off + 4)[0]

    # Read count and stride from header
    if version == 1:
        count = struct.unpack_from(">H", d, idx_off + 20)[0]
        stride = struct.unpack_from(">H", d, idx_off + 22)[0]
        hdr_size = 24
    else:
        hdr_size = 16 + version * 4
        sub_off = idx_off + hdr_size
        if sub_off + 4 > len(d):
            return None
        count = struct.unpack_from(">H", d, sub_off)[0]
        stride = struct.unpack_from(">H", d, sub_off + 2)[0]
        hdr_size += 4  # include sub-header

    data_off = idx_off + hdr_size
    data_size = count * stride

    if data_off + data_size > len(d):
        return None

    indices = []
    matrix_count = 0

    if stride == 2:
        # Direct u16 LE indices
        for i in range(count):
            idx = struct.unpack_from("<H", d, data_off + i * 2)[0]
            indices.append(idx)
        end_off = data_off + data_size
    elif stride == 48:
        # 3x4 transformation matrices
        matrix_count = count
        # After matrices, look for v1-style index header
        footer_off = data_off + data_size
        # v1-style header: BE u32 version=1, BE u32 count, BE u16 stride=2, BE u16 flags=0
        if footer_off + 12 <= len(d):
            foot_ver = struct.unpack_from(">I", d, footer_off)[0]
            foot_count = struct.unpack_from(">I", d, footer_off + 4)[0]
            foot_stride = struct.unpack_from(">H", d, footer_off + 8)[0]
            foot_flags = struct.unpack_from(">H", d, footer_off + 10)[0]

            if foot_ver == 1 and foot_stride == 2:
                idx_data_off = footer_off + 12
                for i in range(foot_count):
                    if idx_data_off + i * 2 + 2 > len(d):
                        break
                    idx = struct.unpack_from("<H", d, idx_data_off + i * 2)[0]
                    indices.append(idx)
                end_off = idx_data_off + foot_count * 2
            else:
                end_off = footer_off
        else:
            end_off = footer_off
    else:
        end_off = data_off + data_size

    # Check footer (0xff bytes at end)
    footer_size = len(d) - end_off
    footer_ff = footer_size >= 8 and d[-8:] == b"\xff" * 8

    # Validate indices
    all_valid = bool(indices) and all(idx < vcount for idx in indices)

    return {
        "vcount": vcount,
        "version": version,
        "count": count,
        "stride": stride,
        "matrix_count": matrix_count,
        "indices": indices,
        "index_count": len(indices),
        "all_valid": all_valid,
        "end_off": end_off,
        "footer_size": footer_size,
        "footer_ff": footer_ff,
        "total_size": len(d),
        "attrs": attrs,
        "attr_count": len(attrs),
    }


def main():
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
                if len(samples) >= 2000:
                    break
        if len(samples) >= 2000:
            break

    print(f"Collected {len(samples)} gmsh samples\n")

    # Parse all
    results = []
    for i, d in enumerate(samples):
        parsed = parse_full(d)
        if parsed:
            parsed["idx"] = i
            results.append(parsed)

    print(f"Parsed: {len(results)}/{len(samples)} ({100*len(results)/len(samples):.1f}%)\n")

    # Categorize
    by_type = Counter()
    for r in results:
        if r["stride"] == 2:
            by_type["direct_indices"] += 1
        elif r["stride"] == 48 and r["indices"]:
            by_type["matrices+indices"] += 1
        elif r["stride"] == 48:
            by_type["matrices_only"] += 1
        else:
            by_type[f"stride_{r['stride']}"] += 1

    print("Parse categories:")
    for k, v in by_type.most_common():
        print(f"  {k}: {v}")
    print()

    # Validation
    valid_indices = sum(1 for r in results if r["all_valid"])
    has_indices = sum(1 for r in results if r["indices"])
    print(f"Samples with indices: {has_indices}")
    print(f"All indices valid: {valid_indices}/{has_indices} ({100*valid_indices/max(has_indices,1):.1f}%)")
    print()

    # Footer analysis
    ff_count = sum(1 for r in results if r["footer_ff"])
    print(f"Footer ends with 0xff*8: {ff_count}/{len(results)} ({100*ff_count/len(results):.1f}%)")
    print()

    # Version distribution
    ver_dist = Counter(r["version"] for r in results)
    print("Version distribution:")
    for v, c in sorted(ver_dist.items()):
        print(f"  v{v}: {c}")
    print()

    # Stride distribution
    stride_dist = Counter(r["stride"] for r in results)
    print("Stride distribution:")
    for s, c in sorted(stride_dist.items()):
        print(f"  stride={s}: {c}")
    print()

    # Show examples
    print("Examples:")
    for r in results[:5]:
        print(f"  #{r['idx']}: vcount={r['vcount']}, ver={r['version']}, "
              f"stride={r['stride']}, matrices={r['matrix_count']}, "
              f"indices={r['index_count']}, valid={r['all_valid']}, "
              f"footer={r['footer_size']}")

    # Summary
    total_with_indices = sum(1 for r in results if r["indices"] and r["all_valid"])
    print(f"\n{'='*70}")
    print(f"SUMMARY: {total_with_indices}/{len(results)} samples fully decoded "
          f"({100*total_with_indices/len(results):.1f}%)")
    print(f"{'='*70}")


if __name__ == "__main__":
    main()

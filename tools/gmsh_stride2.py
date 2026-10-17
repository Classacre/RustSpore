#!/usr/bin/env python3
"""Investigate the remaining gmsh samples that don't fit v1 or v2+ patterns.

From the summary: 19 samples have stride=2 in the index section header,
which doesn't match the v2+ pattern (stride=48 matrices).
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
    """Collect all gmsh samples."""
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


def parse_header(d):
    """Parse the 24-byte gmsh header."""
    if len(d) < 24:
        return None
    return {
        "u0_be": struct.unpack_from(">I", d, 0)[0],
        "u4_be": struct.unpack_from(">I", d, 4)[0],
        "u8_le": struct.unpack_from("<I", d, 8)[0],
        "u12_le": struct.unpack_from("<I", d, 12)[0],
        "u16_le": struct.unpack_from("<I", d, 16)[0],
        "vcount_be": struct.unpack_from(">H", d, 20)[0],
        "stride_be": struct.unpack_from(">H", d, 22)[0],
    }


def find_index_section(d, vcount, stride):
    """Find the index section magic 15 00 06 08 after vertex + attribute data."""
    # Vertex data: vcount * 12 bytes starting at offset 24
    off = 24 + vcount * 12

    # Skip attribute sections
    # Each section: 4-byte magic + BE u32 vcount + BE u16 stride + vcount*stride bytes
    while off + 10 <= len(d):
        magic = d[off:off+4]
        if magic == b"\x15\x00\x06\x08":
            return off
        # Try to parse as attribute section
        attr_vcount = struct.unpack_from(">I", d, off + 4)[0]
        attr_stride = struct.unpack_from(">H", d, off + 8)[0]
        if attr_vcount > 10000 or attr_stride > 100:
            break
        off += 10 + attr_vcount * attr_stride

    return off if off + 4 <= len(d) and d[off:off+4] == b"\x15\x00\x06\x08" else None


def main():
    samples = collect_gmsh(2000)
    print(f"Collected {len(samples)} gmsh samples\n")

    # Find samples with stride=2 in index section
    stride2_samples = []
    for i, d in enumerate(samples):
        hdr = parse_header(d)
        if hdr is None:
            continue

        idx_off = find_index_section(d, hdr["vcount_be"], hdr["stride_be"])
        if idx_off is None:
            continue

        # Read index section version
        version = struct.unpack_from(">I", d, idx_off + 4)[0]
        if version != 1:
            # Read count and stride from header
            hdr_size = 16 + version * 4
            sub_off = idx_off + hdr_size
            if sub_off + 4 <= len(d):
                sub_count = struct.unpack_from(">H", d, sub_off)[0]
                sub_stride = struct.unpack_from(">H", d, sub_off + 2)[0]
                if sub_stride == 2:
                    stride2_samples.append((i, d, hdr, idx_off, version, sub_count, sub_stride))

    print(f"Found {len(stride2_samples)} samples with stride=2 in index section\n")

    for i, d, hdr, idx_off, version, sub_count, sub_stride in stride2_samples[:5]:
        print(f"--- Sample {i} (size={len(d)}, vcount={hdr['vcount_be']}, "
              f"version={version}, sub_count={sub_count}, sub_stride={sub_stride}) ---")

        # Dump bytes around the index section
        start = idx_off
        for row in range(start, min(start + 128, len(d)), 16):
            hex_str = " ".join(f"{b:02x}" for b in d[row:row+16])
            print(f"  +{row:04x}: {hex_str}")

        # Check what's after the stride=2 sub-header
        sub_off = idx_off + 16 + version * 4
        data_start = sub_off + 4
        remaining = len(d) - data_start
        print(f"  Sub-header at +{sub_off}, data starts at +{data_start}")
        print(f"  Remaining bytes: {remaining}")
        print(f"  If stride=2: {remaining // 2} u16 values")
        print()

    # Statistical analysis of stride=2 samples
    if stride2_samples:
        print("=" * 70)
        print("Stride=2 sample analysis:")
        print("=" * 70)
        ver_dist = Counter()
        count_dist = Counter()
        size_ratio = Counter()

        for i, d, hdr, idx_off, version, sub_count, sub_stride in stride2_samples:
            ver_dist[version] += 1
            count_dist[sub_count] += 1
            # Check if sub_count * 2 fits in remaining data
            sub_off = idx_off + 16 + version * 4
            data_start = sub_off + 4
            remaining = len(d) - data_start
            ratio = remaining / max(sub_count * 2, 1)
            size_ratio[round(ratio, 1)] += 1

        print(f"Version distribution: {dict(ver_dist.most_common(10))}")
        print(f"Count distribution: {dict(count_dist.most_common(10))}")
        print(f"Size ratio (remaining / count*2): {dict(size_ratio.most_common(10))}")


if __name__ == "__main__":
    main()

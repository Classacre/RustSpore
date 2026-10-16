"""
Full gmsh format parser and validator.

Based on the analysis of hex dumps, the gmsh format appears to be:

HEADER (24 bytes):
  offset  0: BE u32 = 1        (version)
  offset  4: BE u32 = 1        (flags)
  offset  8: LE u32 = 1024     (constant)
  offset 12: BE u16 = N        (section count: position + attributes + index)
  offset 14: BE u16 = 256      (constant)
  offset 16: LE u32 = 3        (floats per position vertex)
  offset 20: BE u16 = vcount   (vertex count)
  offset 22: BE u16 = 12       (position stride in bytes)

SECTIONS:
  Position: vcount * 12 bytes of float3 data (no section header)

  Attribute sections (variable count):
    4 bytes: magic (02 00 07 01, 02 00 03 01, 03 00 07 02, 08 00 02 02, etc.)
    4 bytes: BE u32 = vcount
    2 bytes: BE u16 = stride
    vcount * stride bytes of data

  Index section:
    4 bytes: magic (15 00 06 08)
    4 bytes: BE u32 = 1
    4 bytes: LE u32 = 1024
    4 bytes: zeros
    2 bytes: BE u16 = 1
    2 bytes: BE u16 = 0
    2 bytes: BE u16 = index_count
    2 bytes: BE u16 = 2 (index stride)
    index_count * 2 bytes of u16 indices

FOOTER:
    2 bytes: BE u16 = 8
    8 bytes: 0xff

This script validates this structure across all gmsh samples.
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

GMSH_TYPE = 0x01C135DA


def get_gmsh_samples(path: Path, limit: int = 2000):
    entries, _ = parse_index(path)
    samples = []
    with path.open("rb") as fh:
        for e in entries:
            if e["type"] != GMSH_TYPE:
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


def parse_gmsh(d: bytes) -> dict:
    """Parse a gmsh resource according to the hypothesized format."""
    result = {
        "size": len(d),
        "sections": [],
        "errors": [],
    }

    # Header
    result["version"] = struct.unpack_from(">I", d, 0)[0]
    result["flags"] = struct.unpack_from(">I", d, 4)[0]
    result["const_1024"] = struct.unpack_from("<I", d, 8)[0]
    result["section_count"] = struct.unpack_from(">H", d, 12)[0]
    result["const_256"] = struct.unpack_from(">H", d, 14)[0]
    result["floats_per_pos"] = struct.unpack_from("<I", d, 16)[0]
    result["vcount"] = struct.unpack_from(">H", d, 20)[0]
    result["pos_stride"] = struct.unpack_from(">H", d, 22)[0]

    vcount = result["vcount"]
    pos_stride = result["pos_stride"]

    # Position data
    pos_end = 24 + vcount * pos_stride
    if pos_end > len(d):
        result["errors"].append(f"position data overruns: need {pos_end}, have {len(d)}")
        return result

    result["sections"].append({
        "type": "position",
        "offset": 24,
        "size": vcount * pos_stride,
        "stride": pos_stride,
        "count": vcount,
    })

    # Parse attribute sections and index section
    offset = pos_end
    section_count = result["section_count"]
    parsed_sections = 1  # position already counted

    while offset < len(d) and parsed_sections < section_count:
        remaining = len(d) - offset

        # Check for index section magic
        if remaining >= 4 and d[offset:offset+4] == b"\x15\x00\x06\x08":
            # Index section
            if remaining < 24:
                result["errors"].append(f"index section too short at {offset}")
                break

            idx_magic = d[offset:offset+4]
            idx_u32_1 = struct.unpack_from(">I", d, offset+4)[0]
            idx_le_1024 = struct.unpack_from("<I", d, offset+8)[0]
            idx_zeros = struct.unpack_from(">I", d, offset+12)[0]
            idx_u16_1 = struct.unpack_from(">H", d, offset+16)[0]
            idx_u16_0 = struct.unpack_from(">H", d, offset+18)[0]
            idx_count = struct.unpack_from(">H", d, offset+20)[0]
            idx_stride = struct.unpack_from(">H", d, offset+22)[0]

            idx_data_start = offset + 24
            idx_data_size = idx_count * idx_stride
            idx_data_end = idx_data_start + idx_data_size

            result["sections"].append({
                "type": "index",
                "offset": offset,
                "magic": idx_magic.hex(),
                "u32_1": idx_u32_1,
                "le_1024": idx_le_1024,
                "u16_1": idx_u16_1,
                "u16_0": idx_u16_0,
                "count": idx_count,
                "stride": idx_stride,
                "data_offset": idx_data_start,
                "data_size": idx_data_size,
            })

            if idx_data_end > len(d):
                result["errors"].append(
                    f"index data overruns: need {idx_data_end}, have {len(d)}")
                break

            offset = idx_data_end
            parsed_sections += 1
            continue

        # Attribute section
        if remaining < 10:
            result["errors"].append(f"attribute section too short at {offset}")
            break

        magic = d[offset:offset+4]
        attr_vcount = struct.unpack_from(">I", d, offset+4)[0]
        attr_stride = struct.unpack_from(">H", d, offset+8)[0]

        attr_data_start = offset + 10
        attr_data_size = attr_vcount * attr_stride
        attr_data_end = attr_data_start + attr_data_size

        result["sections"].append({
            "type": "attribute",
            "offset": offset,
            "magic": magic.hex(),
            "vcount": attr_vcount,
            "stride": attr_stride,
            "data_offset": attr_data_start,
            "data_size": attr_data_size,
        })

        if attr_data_end > len(d):
            result["errors"].append(
                f"attribute data overruns at {offset}: need {attr_data_end}, have {len(d)}")
            break

        offset = attr_data_end
        parsed_sections += 1

    # Check for footer
    if offset < len(d):
        footer_bytes = len(d) - offset
        result["footer_size"] = footer_bytes
        result["footer_hex"] = d[offset:offset+min(32, footer_bytes)].hex()
    else:
        result["footer_size"] = 0

    return result


def validate(samples):
    """Validate the gmsh format across all samples."""
    print("=" * 70)
    print("GMSH FORMAT VALIDATION")
    print("=" * 70)

    total = len(samples)
    perfect = 0
    errors = Counter()
    section_counts = Counter()
    magic_counts = Counter()
    stride_counts = Counter()
    footer_sizes = Counter()
    details = []

    for d in samples:
        r = parse_gmsh(d)

        if not r["errors"]:
            # Check if we consumed all sections
            expected = r["section_count"]
            actual = len(r["sections"])
            if actual == expected:
                perfect += 1
            else:
                errors[f"section count mismatch: expected {expected}, got {actual}"] += 1
        else:
            for e in r["errors"]:
                errors[e.split(":")[0]] += 1

        section_counts[len(r["sections"])] += 1

        for s in r["sections"]:
            if s["type"] == "attribute":
                magic_counts[s["magic"]] += 1
                stride_counts[s["stride"]] += 1

        if "footer_size" in r:
            footer_sizes[r["footer_size"]] += 1

        if len(details) < 5 and not r["errors"]:
            details.append(r)

    print(f"\n  Total samples: {total}")
    print(f"  Perfect parse (no errors, correct section count): {perfect} ({perfect/total*100:.1f}%)")
    print(f"\n  Errors:")
    for e, c in errors.most_common(10):
        print(f"    {e}: {c}")

    print(f"\n  Section count distribution: {dict(section_counts)}")
    print(f"\n  Attribute magic values:")
    for m, c in magic_counts.most_common(10):
        print(f"    {m}: {c}")
    print(f"\n  Attribute stride values:")
    for s, c in stride_counts.most_common(10):
        print(f"    {s}: {c}")
    print(f"\n  Footer size distribution (top 10):")
    for s, c in footer_sizes.most_common(10):
        print(f"    {s}: {c}")

    if details:
        print(f"\n  First 5 successful parses:")
        for i, r in enumerate(details):
            print(f"\n    Sample {i+1}: size={r['size']}, vcount={r['vcount']}, "
                  f"sections={r['section_count']}")
            for s in r["sections"]:
                if s["type"] == "position":
                    print(f"      position: offset={s['offset']}, size={s['size']}")
                elif s["type"] == "attribute":
                    print(f"      attribute: magic={s['magic']}, stride={s['stride']}, "
                          f"count={s['vcount']}, data_size={s['data_size']}")
                elif s["type"] == "index":
                    print(f"      index: count={s['count']}, stride={s['stride']}, "
                          f"data_size={s['data_size']}")
            if "footer_size" in r:
                print(f"      footer: {r['footer_size']} bytes")

    return perfect, total


def main():
    base = Path("D:/Games/SPORE Collection/Data")
    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_gmsh_samples(pkg, limit=2000)
        except (ValueError, struct.error):
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} gmsh resources")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)} samples")
    validate(all_samples)


if __name__ == "__main__":
    raise SystemExit(main())

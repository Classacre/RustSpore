"""
Updated gmsh format parser with index section version handling.

Key finding: the index section (magic 15 00 06 08) has TWO formats:
  Version 1 (BE u16 @+6 = 1): 24-byte header, count at +20 (BE u16), stride at +22 (BE u16 = 2)
  Version 2 (BE u16 @+6 = 2): different header layout, possibly with sub-mesh data

This script validates the full format and reports statistics on both versions.
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


def parse_gmsh_v2(d: bytes) -> dict:
    """Parse gmsh with index section version handling."""
    result = {
        "size": len(d),
        "sections": [],
        "errors": [],
        "index_version": None,
    }

    # Header
    result["vcount"] = struct.unpack_from(">H", d, 20)[0]
    result["section_count"] = struct.unpack_from(">H", d, 12)[0]
    vcount = result["vcount"]
    pos_stride = struct.unpack_from(">H", d, 22)[0]

    # Position
    pos_end = 24 + vcount * pos_stride
    if pos_end > len(d):
        result["errors"].append("position overruns")
        return result

    result["sections"].append({"type": "position", "offset": 24, "size": vcount * pos_stride})

    # Attribute sections
    offset = pos_end
    section_count = result["section_count"]
    parsed = 1

    while offset < len(d) and parsed < section_count:
        remaining = len(d) - offset

        if remaining >= 4 and d[offset:offset+4] == b"\x15\x00\x06\x08":
            # Index section
            if remaining < 24:
                result["errors"].append(f"index header too short at {offset}")
                break

            # Read version (BE u16 @+6)
            idx_version = struct.unpack_from(">H", d, offset+6)[0]
            result["index_version"] = idx_version

            if idx_version == 1:
                # Version 1: count at +20, stride at +22
                count = struct.unpack_from(">H", d, offset+20)[0]
                stride = struct.unpack_from(">H", d, offset+22)[0]
                data_start = offset + 24
                data_size = count * stride
            elif idx_version == 2:
                # Version 2: try count at +20 (BE u16), assume stride=2
                count = struct.unpack_from(">H", d, offset+20)[0]
                stride = 2  # assume u16 indices
                data_start = offset + 24
                data_size = count * stride

                # If count*2 overflows, try LE u16
                if data_start + data_size > len(d):
                    count_le = struct.unpack_from("<H", d, offset+20)[0]
                    if data_start + count_le * 2 <= len(d):
                        count = count_le
                        data_size = count * 2
            else:
                result["errors"].append(f"unknown index version {idx_version} at {offset}")
                break

            data_end = data_start + data_size

            result["sections"].append({
                "type": "index",
                "offset": offset,
                "version": idx_version,
                "count": count,
                "stride": stride,
                "data_offset": data_start,
                "data_size": data_size,
            })

            if data_end > len(d):
                result["errors"].append(
                    f"index v{idx_version} overruns: count={count}, need {data_end}, have {len(d)}")
                break

            offset = data_end
            parsed += 1
            continue

        # Attribute section
        if remaining < 10:
            result["errors"].append(f"attr too short at {offset}")
            break

        magic = d[offset:offset+4]
        attr_vcount = struct.unpack_from(">I", d, offset+4)[0]
        attr_stride = struct.unpack_from(">H", d, offset+8)[0]
        data_size = attr_vcount * attr_stride
        data_end = offset + 10 + data_size

        result["sections"].append({
            "type": "attribute",
            "offset": offset,
            "magic": magic.hex(),
            "vcount": attr_vcount,
            "stride": attr_stride,
            "data_size": data_size,
        })

        if data_end > len(d):
            result["errors"].append(f"attr overruns at {offset}")
            break

        offset = data_end
        parsed += 1

    # Footer
    if offset < len(d):
        result["footer_size"] = len(d) - offset
    else:
        result["footer_size"] = 0

    # Check section count match
    if not result["errors"] and len(result["sections"]) != section_count:
        result["errors"].append(
            f"section count: expected {section_count}, got {len(result['sections'])}")

    return result


def validate(samples):
    print("=" * 70)
    print("GMSH VALIDATION v2")
    print("=" * 70)

    total = len(samples)
    perfect = 0
    version_counts = Counter()
    errors = Counter()

    for d in samples:
        r = parse_gmsh_v2(d)
        if not r["errors"]:
            perfect += 1
        else:
            for e in r["errors"]:
                errors[e.split(":")[0]] += 1
        if r["index_version"] is not None:
            version_counts[r["index_version"]] += 1

    print(f"\n  Total: {total}")
    print(f"  Perfect: {perfect} ({perfect/total*100:.1f}%)")
    print(f"  Index versions: {dict(version_counts)}")
    print(f"  Errors: {dict(errors)}")

    # Detailed breakdown by version
    for ver in sorted(version_counts):
        ok = 0
        fail = 0
        for d in samples:
            r = parse_gmsh_v2(d)
            if r["index_version"] == ver:
                if not r["errors"]:
                    ok += 1
                else:
                    fail += 1
        print(f"    v{ver}: {ok} ok, {fail} fail ({ok/(ok+fail)*100:.1f}%)")

    return perfect, total


def main():
    base = Path("D:/Games/SPORE Collection/Data")
    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_gmsh_samples(pkg, limit=500)
        except (ValueError, struct.error):
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} gmsh")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)}")
    validate(all_samples)


if __name__ == "__main__":
    raise SystemExit(main())

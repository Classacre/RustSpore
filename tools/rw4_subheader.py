"""
Detailed RW4 payload analysis: sub-header and type code table structure.

The RW4 payload starts at offset 96 (after the 96-byte header). The first 56
bytes (96-151) appear to be a sub-header. Type codes start around offset 152.

This script:
  1. Dumps the sub-header for multiple samples
  2. Checks correlation between header offset fields (48, 68, 76) and type codes
  3. Tries to parse the type code table entries
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

RW4_TYPE = 0x2F4E681B

OBJECT_TYPES = {
    0x00010004: "Type_10004",
    0x00010005: "Type_10005",
    0x00010030: "BaseResource",
    0x00020003: "Raster",
    0x00020004: "VertexDescription",
    0x00020005: "VertexBuffer",
    0x00020007: "IndexBuffer",
    0x00020009: "Mesh",
    0x00070001: "KeyframeAnim",
    0x00070002: "Skeleton",
    0x000200AF: "BlendShapeBuffer",
}


def get_samples(path: Path, target_type: int, limit: int = 30):
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
            if len(raw) < 256:
                continue
            samples.append(raw)
    return samples


def analyze_subheader(samples, n=5):
    """Analyze the RW4 sub-header (offset 96-151)."""
    print("=" * 70)
    print("RW4 SUB-HEADER ANALYSIS (offset 96-151)")
    print("=" * 70)

    for idx in range(min(n, len(samples))):
        d = samples[idx]
        total = len(d)
        subtype = struct.unpack_from("<I", d, 28)[0]

        # Header offset fields
        off48 = struct.unpack_from("<I", d, 48)[0]
        off68 = struct.unpack_from("<I", d, 68)[0]
        off76 = struct.unpack_from("<I", d, 76)[0]

        print(f"\n  Sample {idx+1}: size={total}, subtype=0x{subtype:08X}")
        print(f"    Header: off48={off48}, off68={off68}, off76={off76}")
        print(f"    Footer check: len-off76={total - off76}, off68={off68}, "
              f"match={total - off76 == off68}")

        # Sub-header as u32 array
        print(f"    Sub-header (offset 96-151) as LE u32:")
        for j in range(96, 152, 4):
            v = struct.unpack_from("<I", d, j)[0]
            note = ""
            if v in OBJECT_TYPES:
                note = f" *** {OBJECT_TYPES[v]} ***"
            elif 96 <= v < total:
                note = f" [file offset {v}]"
            print(f"      +{j - 96:>3} (file {j}): 0x{v:08X} ({v}){note}")

        # Find first type code after offset 152
        first_tc = None
        for j in range(152, min(total - 4, 300)):
            v = struct.unpack_from("<I", d, j)[0]
            if v in OBJECT_TYPES:
                first_tc = (j, v)
                break
        if first_tc:
            print(f"    First type code: 0x{first_tc[1]:08X} "
                  f"({OBJECT_TYPES[first_tc[1]]}) at file offset {first_tc[0]}")


def analyze_type_code_table(samples, n=5):
    """Try to parse the type code table."""
    print(f"\n{'='*70}")
    print("TYPE CODE TABLE ANALYSIS")
    print(f"{'='*70}")

    for idx in range(min(n, len(samples))):
        d = samples[idx]
        total = len(d)
        subtype = struct.unpack_from("<I", d, 28)[0]

        print(f"\n  Sample {idx+1}: size={total}, subtype=0x{subtype:08X}")

        # Find all type codes
        codes = []
        for j in range(96, total - 4, 4):
            v = struct.unpack_from("<I", d, j)[0]
            if v in OBJECT_TYPES:
                codes.append((j, v))

        print(f"    Type codes found: {len(codes)}")
        for offset, code in codes[:15]:
            name = OBJECT_TYPES[code]
            # Dump 16 bytes after the type code
            after = d[offset+4:offset+20].hex(' ')
            print(f"      file {offset}: 0x{code:08X} ({name})")
            print(f"        after: {after}")


def check_header_correlation(samples):
    """Check if header offset fields correlate with type code positions."""
    print(f"\n{'='*70}")
    print("HEADER FIELD CORRELATION")
    print(f"{'='*70}")

    for idx in range(min(10, len(samples))):
        d = samples[idx]
        total = len(d)

        off48 = struct.unpack_from("<I", d, 48)[0]
        off68 = struct.unpack_from("<I", d, 68)[0]
        off76 = struct.unpack_from("<I", d, 76)[0]

        # Find first type code
        first_tc_offset = None
        for j in range(96, min(total - 4, 500)):
            v = struct.unpack_from("<I", d, j)[0]
            if v in OBJECT_TYPES:
                first_tc_offset = j
                break

        # Check if off48 or off76 point to type codes
        at_off48 = "N/A"
        at_off76 = "N/A"
        if 96 <= off48 < total - 4:
            v = struct.unpack_from("<I", d, off48)[0]
            at_off48 = OBJECT_TYPES.get(v, f"0x{v:08X}")
        if 96 <= off76 < total - 4:
            v = struct.unpack_from("<I", d, off76)[0]
            at_off76 = OBJECT_TYPES.get(v, f"0x{v:08X}")

        print(f"  size={total}: off48={off48} (->{at_off48}), "
              f"off76={off76} (->{at_off76}), "
              f"first_tc at {first_tc_offset}")


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_samples(pkg, RW4_TYPE, limit=10)
        except (ValueError, struct.error):
            continue
        if samples:
            all_samples.extend(samples)

    print(f"  Total: {len(all_samples)} RW4 samples")
    if all_samples:
        analyze_subheader(all_samples, n=3)
        analyze_type_code_table(all_samples, n=3)
        check_header_correlation(all_samples)


if __name__ == "__main__":
    raise SystemExit(main())

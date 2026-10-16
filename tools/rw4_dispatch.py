"""
Investigate RW4 object dispatch.

The RW4 header is decoded (96 bytes). The payload after the header should
contain object type codes that select the decoder. Object types from the
binary analysis:
  0x20004 = VertexDescription
  0x20005 = VertexBuffer
  0x20007 = IndexBuffer
  0x20009 = Mesh
  0x10030 = BaseResource
  0x20003 = Raster
  0x70002 = Skeleton
  0x70001 = KeyframeAnim
  0x200af = BlendShapeBuffer

This script dumps RW4 payloads and searches for these type codes.
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

RW4_TYPE = 0x2F4E681B

# Known RW4 object type codes
OBJECT_TYPES = {
    0x20004: "VertexDescription",
    0x20005: "VertexBuffer",
    0x20007: "IndexBuffer",
    0x20009: "Mesh",
    0x10030: "BaseResource",
    0x20003: "Raster",
    0x70002: "Skeleton",
    0x70001: "KeyframeAnim",
    0x200af: "BlendShapeBuffer",
}


def get_samples(path: Path, target_type: int, limit: int = 20):
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
            if len(raw) < 128:
                continue
            samples.append(raw)
    return samples


def dump_hex(d, start, length=64, label=""):
    end = min(start + length, len(d))
    print(f"    {label} (offset {start}-{end}):")
    for j in range(start, end, 16):
        chunk = d[j:j+16]
        hexstr = chunk.hex(' ')
        asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
        print(f"      {j:>6}  {hexstr:<47}  {asc}")


def find_type_codes(d):
    """Find all known RW4 object type codes in the payload."""
    found = []
    for type_code, name in OBJECT_TYPES.items():
        # Search for LE u32
        pattern = struct.pack("<I", type_code)
        pos = 0
        while True:
            idx = d.find(pattern, pos)
            if idx < 0:
                break
            found.append((idx, type_code, name, "LE"))
            pos = idx + 1
        # Search for BE u32
        pattern_be = struct.pack(">I", type_code)
        pos = 0
        while True:
            idx = d.find(pattern_be, pos)
            if idx < 0:
                break
            found.append((idx, type_code, name, "BE"))
            pos = idx + 1
    return sorted(found)


def investigate(samples, n=5):
    """Investigate RW4 payload structure."""
    for idx in range(min(n, len(samples))):
        d = samples[idx]
        total = len(d)

        print(f"\n{'='*70}")
        print(f"RW4 Sample {idx+1}: size={total}")
        print(f"{'='*70}")

        # Header fields
        subtype = struct.unpack_from("<I", d, 28)[0]
        print(f"  Subtype: {subtype}")

        # Dump payload after 96-byte header
        print(f"\n  Payload after 96-byte header (first 128 bytes):")
        dump_hex(d, 96, 128, "payload")

        # Search for type codes
        type_codes = find_type_codes(d)
        if type_codes:
            print(f"\n  Type codes found:")
            for offset, code, name, endian in type_codes[:10]:
                print(f"    offset {offset}: 0x{code:08X} ({name}, {endian})")
        else:
            print(f"\n  No known type codes found")

        # Dump first 32 bytes of payload as u32
        print(f"\n  Payload as LE u32 (first 32 values):")
        for j in range(96, min(96 + 128, total), 4):
            v = struct.unpack_from("<I", d, j)[0]
            note = ""
            if v in OBJECT_TYPES:
                note = f" *** {OBJECT_TYPES[v]} ***"
            print(f"    +{j - 96:>3} (file {j}): 0x{v:08X} ({v}){note}")


def statistical(samples):
    """Statistical analysis of RW4 payloads."""
    print(f"\n{'='*70}")
    print("RW4 PAYLOAD STATISTICS")
    print(f"{'='*70}")

    # Check first u32 after header
    first_u32 = Counter()
    for d in samples:
        if len(d) >= 100:
            v = struct.unpack_from("<I", d, 96)[0]
            first_u32[v] += 1
    print(f"\n  First u32 after header (top 10):")
    for v, c in first_u32.most_common(10):
        name = OBJECT_TYPES.get(v, "")
        print(f"    0x{v:08X} ({v}): {c} {name}")

    # Check for type codes anywhere in payload
    all_codes = Counter()
    for d in samples:
        codes = find_type_codes(d)
        for _, code, name, _ in codes:
            all_codes[(code, name)] += 1
    print(f"\n  Type code occurrences across all samples:")
    for (code, name), cnt in all_codes.most_common(15):
        print(f"    0x{code:08X} ({name}): {cnt}")

    # Check subtype distribution
    subtypes = Counter()
    for d in samples:
        if len(d) >= 32:
            subtype = struct.unpack_from("<I", d, 28)[0]
            subtypes[subtype] += 1
    print(f"\n  Subtype distribution: {dict(subtypes)}")


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_samples(pkg, RW4_TYPE, limit=20)
        except (ValueError, struct.error):
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} RW4 resources")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)}")
    if all_samples:
        statistical(all_samples)
        investigate(all_samples, n=3)


if __name__ == "__main__":
    raise SystemExit(main())

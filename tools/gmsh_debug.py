"""Debug: dump bytes at expected index section offset and search for magic."""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

GMSH_TYPE = 0x01C135DA


def get_sample(path, idx=0):
    entries, _ = parse_index(path)
    count = 0
    with path.open("rb") as fh:
        for e in entries:
            if e["type"] != GMSH_TYPE:
                continue
            if count == idx:
                fh.seek(e["offset"])
                raw = fh.read(e["stored"])
                if e["dec"] > e["stored"]:
                    raw = decompress(raw, out_size=None, header=True)[0]
                return raw
            count += 1
    return None


def dump_hex(d, start, length=64, label=""):
    """Dump hex bytes."""
    end = min(start + length, len(d))
    print(f"  {label} (offset {start}-{end}):")
    for j in range(start, end, 16):
        chunk = d[j:j+16]
        hexstr = chunk.hex(' ')
        asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
        print(f"    {j:>6}  {hexstr:<47}  {asc}")


def find_all(d, pattern):
    """Find all occurrences of pattern."""
    results = []
    pos = 0
    while True:
        idx = d.find(pattern, pos)
        if idx < 0:
            break
        results.append(idx)
        pos = idx + 1
    return results


def main():
    base = Path("D:/Games/SPORE Collection/Data")
    pkg = base / "Spore_Graphics.package"

    # Get the first few gmsh samples
    for sample_idx in range(3):
        d = get_sample(pkg, sample_idx)
        if d is None:
            continue

        vcount = struct.unpack_from(">H", d, 20)[0]
        section_count = struct.unpack_from(">H", d, 12)[0]
        total = len(d)

        print(f"\n{'='*70}")
        print(f"Sample {sample_idx}: size={total}, vcount={vcount}, section_count={section_count}")
        print(f"{'='*70}")

        # Parse header
        print(f"  Header:")
        print(f"    version={struct.unpack_from('>I', d, 0)[0]}")
        print(f"    flags={struct.unpack_from('>I', d, 4)[0]}")
        print(f"    const_1024={struct.unpack_from('<I', d, 8)[0]}")
        print(f"    section_count={section_count}")
        print(f"    const_256={struct.unpack_from('>H', d, 14)[0]}")
        print(f"    floats_per_pos={struct.unpack_from('<I', d, 16)[0]}")
        print(f"    vcount={vcount}")
        print(f"    pos_stride={struct.unpack_from('>H', d, 22)[0]}")

        # Parse sections
        offset = 24 + vcount * 12
        print(f"\n  Position ends at offset {offset}")

        # Parse attribute sections
        for sec in range(section_count - 2):  # -1 for position, -1 for index
            if offset + 10 > total:
                print(f"  OVERFLOW at attribute section {sec}")
                break

            magic = d[offset:offset+4]
            attr_vcount = struct.unpack_from(">I", d, offset+4)[0]
            attr_stride = struct.unpack_from(">H", d, offset+8)[0]
            data_size = attr_vcount * attr_stride
            data_end = offset + 10 + data_size

            print(f"\n  Attribute {sec}: magic={magic.hex()}, vcount={attr_vcount}, "
                  f"stride={attr_stride}, data_size={data_size}")
            print(f"    Header at offset {offset}:")
            dump_hex(d, offset, 16, "attr header")

            if data_end > total:
                print(f"    OVERFLOW: data_end={data_end} > total={total}")
                break

            offset = data_end

        # Now at expected index section
        print(f"\n  Expected index section at offset {offset}")
        print(f"  Remaining bytes: {total - offset}")

        # Check for 15 00 06 08 at this offset
        if offset + 4 <= total:
            magic = d[offset:offset+4]
            print(f"  Magic at offset: {magic.hex()}")
            if magic == b"\x15\x00\x06\x08":
                print(f"  *** FOUND index magic! ***")
                dump_hex(d, offset, 48, "index section")
            else:
                print(f"  *** NOT index magic! ***")
                dump_hex(d, offset, 64, "unexpected data")

        # Search for all 15 00 06 08 in the file
        positions = find_all(d, b"\x15\x00\x06\x08")
        print(f"\n  All '15 00 06 08' positions: {positions[:20]}")
        if len(positions) > 20:
            print(f"    ... and {len(positions) - 20} more")

        # For each position, dump 32 bytes
        for pos in positions[:5]:
            print(f"\n  At position {pos}:")
            dump_hex(d, pos, 32, "index candidate")

        # Also search for 00 00 00 01 (might be a section marker)
        positions2 = find_all(d, b"\x00\x00\x00\x01")
        print(f"\n  All '00 00 00 01' positions: {positions2[:20]}")
        if len(positions2) > 20:
            print(f"    ... and {len(positions2) - 20} more")

        # Dump the last 64 bytes (potential footer)
        print(f"\n  Last 64 bytes:")
        dump_hex(d, max(0, total - 64), 64, "footer")

        if sample_idx >= 2:
            break


if __name__ == "__main__":
    main()

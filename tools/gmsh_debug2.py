"""Debug specific failing sample: find size=39722 and dump index section bytes."""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

GMSH_TYPE = 0x01C135DA


def find_and_debug(base: Path, target_size: int):
    """Find a gmsh sample with the target size and debug it."""
    for pkg in sorted(base.glob("*.package")):
        try:
            entries, _ = parse_index(pkg)
        except (ValueError, struct.error):
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
                if len(raw) == target_size:
                    return raw, pkg.name
    return None, None


def dump_hex(d, start, length=64, label=""):
    end = min(start + length, len(d))
    print(f"  {label} (offset {start}-{end}):")
    for j in range(start, end, 16):
        chunk = d[j:j+16]
        hexstr = chunk.hex(' ')
        asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
        print(f"    {j:>6}  {hexstr:<47}  {asc}")


def main():
    base = Path("D:/Games/SPORE Collection/Data")
    target_size = 39722

    d, pkg_name = find_and_debug(base, target_size)
    if d is None:
        print(f"Sample with size={target_size} not found!")
        return

    vcount = struct.unpack_from(">H", d, 20)[0]
    section_count = struct.unpack_from(">H", d, 12)[0]
    total = len(d)

    print(f"Found in {pkg_name}: size={total}, vcount={vcount}, section_count={section_count}")

    # Parse sections step by step
    offset = 24 + vcount * 12
    print(f"\nPosition ends at offset {offset}")

    for sec in range(section_count - 2):
        if offset + 10 > total:
            print(f"  OVERFLOW at attribute {sec}")
            break

        magic = d[offset:offset+4]
        attr_vcount = struct.unpack_from(">I", d, offset+4)[0]
        attr_stride = struct.unpack_from(">H", d, offset+8)[0]
        data_size = attr_vcount * attr_stride
        data_end = offset + 10 + data_size

        print(f"\n  Attr {sec}: magic={magic.hex()}, vcount={attr_vcount}, "
              f"stride={attr_stride}, header_10bytes")
        dump_hex(d, offset, 16, "attr header")

        # Also try 12-byte header interpretation
        if offset + 12 <= total:
            magic12 = d[offset:offset+4]
            vc12 = struct.unpack_from(">I", d, offset+4)[0]
            st12 = struct.unpack_from(">H", d, offset+8)[0]
            extra = struct.unpack_from(">H", d, offset+10)[0]
            print(f"    12-byte header: magic={magic12.hex()}, vcount={vc12}, "
                  f"stride={st12}, extra={extra}")

        if data_end > total:
            print(f"    OVERFLOW: data_end={data_end} > total={total}")
            break

        offset = data_end

    # Now at expected index section
    print(f"\n  Expected index at offset {offset}, remaining={total - offset}")
    dump_hex(d, offset, 48, "index section")

    # Parse index section fields
    if offset + 24 <= total:
        magic = d[offset:offset+4]
        u32_1 = struct.unpack_from(">I", d, offset+4)[0]
        le_1024 = struct.unpack_from("<I", d, offset+8)[0]
        zeros = struct.unpack_from(">I", d, offset+12)[0]
        u16_1 = struct.unpack_from(">H", d, offset+16)[0]
        u16_0 = struct.unpack_from(">H", d, offset+18)[0]
        count_be = struct.unpack_from(">H", d, offset+20)[0]
        stride_be = struct.unpack_from(">H", d, offset+22)[0]

        print(f"\n  Index section parse:")
        print(f"    magic={magic.hex()}")
        print(f"    BE u32 @+4 = {u32_1}")
        print(f"    LE u32 @+8 = {le_1024}")
        print(f"    BE u32 @+12 = {zeros}")
        print(f"    BE u16 @+16 = {u16_1}")
        print(f"    BE u16 @+18 = {u16_0}")
        print(f"    BE u16 @+20 = {count_be} (index count)")
        print(f"    BE u16 @+22 = {stride_be} (index stride)")

        # Try different offsets for count/stride
        print(f"\n  Trying different count/stride offsets:")
        for off in range(16, 28, 2):
            if offset + off + 2 <= total:
                v_be = struct.unpack_from(">H", d, offset+off)[0]
                v_le = struct.unpack_from("<H", d, offset+off)[0]
                print(f"    BE u16 @+{off} = {v_be} (0x{v_be:04X})  "
                      f"LE u16 @+{off} = {v_le} (0x{v_le:04X})")

        # Check if count * stride fits
        if count_be > 0 and stride_be > 0:
            data_end = offset + 24 + count_be * stride_be
            print(f"\n  If count={count_be}, stride={stride_be}: "
                  f"data_end={data_end}, remaining={total - data_end}")
        elif count_be > 0 and stride_be == 0:
            print(f"\n  count={count_be}, stride=0 -> no data?")
            # Try stride=2
            data_end = offset + 24 + count_be * 2
            print(f"  If stride=2: data_end={data_end}, remaining={total - data_end}")

    # Search for all 15 00 06 08
    positions = []
    pos = 0
    while True:
        idx = d.find(b"\x15\x00\x06\x08", pos)
        if idx < 0:
            break
        positions.append(idx)
        pos = idx + 1
    print(f"\n  All '15 00 06 08' positions: {positions}")
    for p in positions:
        print(f"\n  At {p}:")
        dump_hex(d, p, 32, "candidate")

    # Dump last 64 bytes
    print(f"\n  Last 64 bytes:")
    dump_hex(d, max(0, total - 64), 64, "footer")


if __name__ == "__main__":
    main()

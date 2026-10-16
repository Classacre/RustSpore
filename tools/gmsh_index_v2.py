"""Deep dive into index section version 2 format.

For version 2 (BE u32 @+4 = 2), the index section has a different structure.
This script tries to find the actual index data by searching for valid u16 values.
"""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

GMSH_TYPE = 0x01C135DA


def get_sample(path, target_size=None, idx=0):
    entries, _ = parse_index(path)
    count = 0
    with path.open("rb") as fh:
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
            if target_size and len(raw) != target_size:
                continue
            if count == idx:
                return raw
            count += 1
    return None


def find_index_data(d, vcount, idx_offset):
    """Search for valid u16 index data after the index section header."""
    total = len(d)
    version = struct.unpack_from(">I", d, idx_offset + 4)[0]

    print(f"\n  Index section at {idx_offset}, version={version}")
    print(f"  vcount={vcount}, total={total}, remaining={total - idx_offset}")

    # Dump first 48 bytes of index section
    print(f"  First 48 bytes:")
    for j in range(idx_offset, min(idx_offset + 48, total), 16):
        chunk = d[j:j+16]
        hexstr = chunk.hex(' ')
        print(f"    {j:>6}  {hexstr}")

    # For version 1: count at +20, stride at +22
    if version == 1:
        count = struct.unpack_from(">H", d, idx_offset + 20)[0]
        stride = struct.unpack_from(">H", d, idx_offset + 22)[0]
        data_start = idx_offset + 24
        data_size = count * stride
        print(f"\n  V1: count={count}, stride={stride}, data={data_size} bytes")
        print(f"  Data at {data_start}-{data_start + data_size}")

        # Verify indices
        if stride == 2:
            ok = True
            for j in range(data_size // 2):
                v = struct.unpack_from("<H", d, data_start + j * 2)[0]
                if v >= vcount:
                    print(f"    BAD index at {j}: {v} >= {vcount}")
                    ok = False
                    break
            if ok:
                print(f"    All {data_size // 2} indices valid!")

    # For version 2+: try to find index data
    else:
        print(f"\n  V{version}: trying to find index data...")

        # Try: count at +20 (BE u16), data at +24
        count_be = struct.unpack_from(">H", d, idx_offset + 20)[0]
        count_le = struct.unpack_from("<H", d, idx_offset + 20)[0]

        print(f"  BE u16 @+20 = {count_be}, LE u16 @+20 = {count_le}")

        # Try BE count with stride 2
        for stride in [1, 2, 4]:
            data_size = count_be * stride
            data_start = idx_offset + 24
            if data_start + data_size <= total:
                # Check if indices are valid
                if stride == 2:
                    ok = True
                    for j in range(min(data_size // 2, 100)):
                        v = struct.unpack_from("<H", d, data_start + j * 2)[0]
                        if v >= vcount:
                            ok = False
                            break
                    if ok:
                        print(f"  BE count={count_be}, stride={stride}: "
                              f"first 100 indices valid!")
                        # Show first 20
                        vals = [struct.unpack_from("<H", d, data_start + j * 2)[0]
                                for j in range(min(20, data_size // 2))]
                        print(f"    First 20: {vals}")
                        remaining = total - (data_start + data_size)
                        print(f"    Remaining after data: {remaining}")

        # Try LE count with stride 2
        for stride in [1, 2, 4]:
            data_size = count_le * stride
            data_start = idx_offset + 24
            if data_start + data_size <= total:
                if stride == 2:
                    ok = True
                    for j in range(min(data_size // 2, 100)):
                        v = struct.unpack_from("<H", d, data_start + j * 2)[0]
                        if v >= vcount:
                            ok = False
                            break
                    if ok:
                        print(f"  LE count={count_le}, stride={stride}: "
                              f"first 100 indices valid!")
                        vals = [struct.unpack_from("<H", d, data_start + j * 2)[0]
                                for j in range(min(20, data_size // 2))]
                        print(f"    First 20: {vals}")
                        remaining = total - (data_start + data_size)
                        print(f"    Remaining after data: {remaining}")

        # Try: multiple sub-mesh counts
        # For version N, there might be N (count, offset) pairs
        print(f"\n  Trying multi-buffer interpretation (N={version}):")
        for header_size in [24, 28, 32, 24 + version * 4, 24 + version * 8]:
            if idx_offset + header_size > total:
                continue
            data_start = idx_offset + header_size
            remaining = total - data_start

            # Try to find a run of valid u16 indices
            best_start = None
            best_count = 0
            for off in range(0, min(remaining - 4, 200), 2):
                cnt = 0
                for j in range(off, remaining - 1, 2):
                    v = struct.unpack_from("<H", d, data_start + j)[0]
                    if v >= vcount:
                        break
                    cnt += 1
                if cnt > best_count:
                    best_count = cnt
                    best_start = off

            if best_start is not None and best_count >= 10:
                print(f"    header_size={header_size}: index data at +{best_start}, "
                      f"{best_count} valid indices")
                vals = [struct.unpack_from("<H", d, data_start + best_start + j * 2)[0]
                        for j in range(min(20, best_count))]
                print(f"      First 20: {vals}")

        # Also try: the "count" might be at a different offset for v2+
        # Look at all BE u16 values in the first 48 bytes
        print(f"\n  All BE u16 values in index header:")
        for off in range(0, 48, 2):
            if idx_offset + off + 2 <= total:
                v_be = struct.unpack_from(">H", d, idx_offset + off)[0]
                v_le = struct.unpack_from("<H", d, idx_offset + off)[0]
                if v_be < 10000 or v_le < 10000:
                    print(f"    +{off:>2}: BE={v_be} (0x{v_be:04X})  LE={v_le} (0x{v_le:04X})")


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    # Get the PatchData sample (version 2)
    d = get_sample(base / "PatchData.package", target_size=39722)
    if d:
        vcount = struct.unpack_from(">H", d, 20)[0]
        # Find the index section
        pos = d.find(b"\x15\x00\x06\x08")
        if pos >= 0:
            find_index_data(d, vcount, pos)

    # Also get a version 1 sample for comparison
    print("\n" + "=" * 70)
    print("VERSION 1 SAMPLE (for comparison)")
    print("=" * 70)
    d2 = get_sample(base / "Spore_Graphics.package", idx=0)
    if d2:
        vcount = struct.unpack_from(">H", d2, 20)[0]
        pos = d2.find(b"\x15\x00\x06\x08")
        if pos >= 0:
            find_index_data(d2, vcount, pos)


if __name__ == "__main__":
    main()

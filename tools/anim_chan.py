"""
Decode the ANIM CHAN section.

Structure discovered:
  - Offset table at file offset 512: u32 values + "CHAN" magic
  - After "CHAN": bone names and keyframe data
  - Trailer at end: count + hash + animation name (ASCII)

This script parses the CHAN section and tries to decode keyframes.
"""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

ANIM_TYPE = 0xEE17C6AD


def get_samples(path: Path, target_type: int, limit: int = 5):
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
            if len(raw) < 600:
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


def parse_chan_section(d):
    """Parse the CHAN section and offset table."""
    total = len(d)

    # Find CHAN magic
    chan_pos = d.find(b"CHAN")
    if chan_pos < 0:
        print("  CHAN magic not found!")
        return

    print(f"  CHAN magic at offset {chan_pos}")

    # Parse offset table (starts at 512, ends at chan_pos)
    table_start = 512
    table_end = chan_pos
    table_size = table_end - table_start
    num_offsets = table_size // 4

    print(f"  Offset table: {num_offsets} u32 values (offset {table_start}-{table_end})")
    offsets = []
    for i in range(num_offsets):
        v = struct.unpack_from("<I", d, table_start + i * 4)[0]
        offsets.append(v)
    print(f"    Offsets: {offsets}")

    # Dump data after CHAN
    print(f"\n  Data after CHAN (first 128 bytes):")
    dump_hex(d, chan_pos, 128, "after CHAN")

    # Try to parse bone names after CHAN
    # Format might be: "CHAN" + u32 count + per channel: name + data
    offset = chan_pos + 4
    if offset + 4 <= total:
        count = struct.unpack_from("<I", d, offset)[0]
        print(f"\n  u32 after CHAN: {count}")
        offset += 4

    # Try to read bone names (null-terminated ASCII, possibly fixed-size)
    print(f"\n  Trying to read bone names:")
    for i in range(min(20, num_offsets)):
        if offset + 32 > total:
            break
        # Try fixed-size name field (32 bytes?)
        name_bytes = d[offset:offset+32]
        # Find null terminator
        null_pos = name_bytes.find(b"\x00")
        if null_pos >= 0:
            name = name_bytes[:null_pos].decode("ascii", errors="replace")
        else:
            name = name_bytes.decode("ascii", errors="replace")
        print(f"    Channel {i}: name=\"{name}\" (raw: {name_bytes[:16].hex(' ')})")
        offset += 32  # try 32-byte name field

    # Try to parse keyframes at each offset
    print(f"\n  Trying to parse keyframes at offsets:")
    for i, off in enumerate(offsets):
        if off >= total - 32:
            print(f"    Offset {i}: {off} -- too close to end")
            continue

        # Try Cal3D keyframe: time(f) + trans(3f) + rot(4f) = 32 bytes
        time = struct.unpack_from("<f", d, off)[0]
        tx, ty, tz = struct.unpack_from("<fff", d, off + 4)
        rx, ry, rz, rw = struct.unpack_from("<ffff", d, off + 16)

        print(f"    Offset {i} (file {off}):")
        print(f"      time={time:.4f} trans=({tx:.4f},{ty:.4f},{tz:.4f}) "
              f"rot=({rx:.4f},{ry:.4f},{rz:.4f},{rw:.4f})")

        # Also dump raw bytes
        if i < 3:
            dump_hex(d, off, 32, f"offset {i}")

    # Parse trailer
    print(f"\n  Trailer analysis:")
    # The trailer starts with 60 00 00 00 (0x60 = 96)
    trailer_pos = d.find(b"\x60\x00\x00\x00\x02\x00\x00\x00", total - 500)
    if trailer_pos < 0:
        # Try from the end
        trailer_pos = total - 200
    print(f"  Trailer at ~{trailer_pos}")
    dump_hex(d, trailer_pos, 128, "trailer")

    # Find animation name at end (ASCII, null-terminated)
    # Search backwards for printable ASCII
    name_end = total
    while name_end > 0 and d[name_end - 1] == 0:
        name_end -= 1
    name_start = name_end
    while name_start > 0 and 32 <= d[name_start - 1] < 127:
        name_start -= 1
    if name_start < name_end:
        name = d[name_start:name_end].decode("ascii", errors="replace")
        print(f"\n  Animation name: \"{name}\" (offset {name_start}-{name_end})")


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_samples(pkg, ANIM_TYPE, limit=3)
        except (ValueError, struct.error):
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} anim resources")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)}")
    for i, d in enumerate(all_samples[:3]):
        print(f"\n{'='*70}")
        print(f"Sample {i+1}: size={len(d)}")
        print(f"{'='*70}")
        parse_chan_section(d)


if __name__ == "__main__":
    raise SystemExit(main())

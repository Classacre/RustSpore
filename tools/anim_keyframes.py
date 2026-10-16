"""
Decode anim keyframe data within each bone's CHAN section.

Structure: each bone has a CHAN section starting with "CHAN" + u32(0) + bone name.
Within each section: INFO, POS, ROT sub-sections with keyframe data.

This script dumps the data after INFO/POS/ROT magics to find keyframes.
"""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

ANIM_TYPE = 0xEE17C6AD


def get_samples(path: Path, target_type: int, limit: int = 3):
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


def find_magic(d, magic, start=0):
    """Find all occurrences of a 4-byte magic."""
    results = []
    pos = start
    while True:
        idx = d.find(magic, pos)
        if idx < 0:
            break
        results.append(idx)
        pos = idx + 1
    return results


def parse_bone_section(d, bone_offset, next_offset, bone_idx):
    """Parse a single bone's CHAN section."""
    section = d[bone_offset:next_offset]
    section_len = len(section)

    # Extract bone name
    name_start = bone_offset + 8  # skip CHAN + u32
    name_end = d.find(b"\x00", name_start)
    if name_end < 0 or name_end > bone_offset + 64:
        bone_name = "?"
    else:
        bone_name = d[name_start:name_end].decode("ascii", errors="replace")

    print(f"\n  Bone {bone_idx}: \"{bone_name}\" at {bone_offset}, "
          f"section_len={section_len}")

    # Find INFO, POS, ROT within this section
    for magic_name, magic_bytes in [("INFO", b"INFO"), ("POS", b"POS\x00"),
                                     ("ROT", b"ROT\x00")]:
        positions = find_magic(d, magic_bytes, bone_offset)
        # Filter to within this bone's section
        positions = [p for p in positions if bone_offset <= p < next_offset]

        if not positions:
            continue

        pos = positions[0]
        # Read header: magic(4) + u32 + u32 + u32
        if pos + 16 <= len(d):
            v1 = struct.unpack_from("<I", d, pos + 4)[0]
            v2 = struct.unpack_from("<I", d, pos + 8)[0]
            v3 = struct.unpack_from("<I", d, pos + 12)[0]
            print(f"    {magic_name} at {pos} (section+{pos - bone_offset}): "
                  f"v1={v1}, v2={v2}, v3={v3}")

            # Dump data after the 16-byte header
            data_start = pos + 16
            data_len = min(64, next_offset - data_start)
            if data_len > 0:
                dump_hex(d, data_start, data_len, f"{magic_name} data")

            # Try to parse as keyframes
            # POS: time(f) + trans(3f) = 16 bytes per keyframe?
            # ROT: time(f) + rot(4f) = 20 bytes per keyframe?
            if magic_name == "POS":
                kf_size = 16
                num_kf = (next_offset - data_start) // kf_size
                print(f"      Trying {kf_size}-byte keyframes ({num_kf} possible):")
                for kf in range(min(3, num_kf)):
                    off = data_start + kf * kf_size
                    t = struct.unpack_from("<f", d, off)[0]
                    x, y, z = struct.unpack_from("<fff", d, off + 4)
                    print(f"        KF[{kf}]: t={t:.4f} pos=({x:.4f},{y:.4f},{z:.4f})")

            elif magic_name == "ROT":
                kf_size = 20
                num_kf = (next_offset - data_start) // kf_size
                print(f"      Trying {kf_size}-byte keyframes ({num_kf} possible):")
                for kf in range(min(3, num_kf)):
                    off = data_start + kf * kf_size
                    t = struct.unpack_from("<f", d, off)[0]
                    x, y, z, w = struct.unpack_from("<ffff", d, off + 4)
                    print(f"        KF[{kf}]: t={t:.4f} rot=({x:.4f},{y:.4f},{z:.4f},{w:.4f})")


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_samples(pkg, ANIM_TYPE, limit=2)
        except (ValueError, struct.error):
            continue
        if samples:
            all_samples.extend(samples)

    for i, d in enumerate(all_samples[:2]):
        print(f"\n{'='*70}")
        print(f"Sample {i+1}: size={len(d)}")
        print(f"{'='*70}")

        # Parse offset table
        chan_pos = d.find(b"CHAN")
        if chan_pos < 0:
            continue

        table_start = 512
        num_offsets = (chan_pos - table_start) // 4
        offsets = []
        for j in range(num_offsets):
            v = struct.unpack_from("<I", d, table_start + j * 4)[0]
            offsets.append(v)

        print(f"  Offsets: {offsets}")

        # Parse each bone section
        for bone_idx in range(len(offsets)):
            bone_offset = offsets[bone_idx]
            if bone_idx + 1 < len(offsets):
                next_offset = offsets[bone_idx + 1]
            else:
                # Last bone: ends at trailer
                trailer_pos = d.find(b"\x60\x00\x00\x00\x02\x00\x00\x00",
                                     len(d) - 500)
                next_offset = trailer_pos if trailer_pos > 0 else len(d)

            parse_bone_section(d, bone_offset, next_offset, bone_idx)


if __name__ == "__main__":
    raise SystemExit(main())

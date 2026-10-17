#!/usr/bin/env python3
"""Investigate anim POS keyframe data structure.

ROT keyframes are 20 bytes (time + quaternion). POS keyframes are likely
16 bytes (time + float3 position). This script finds and validates them.
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0xEE17C6AD


def collect_samples(limit=50):
    samples = []
    for pkg in sorted(BASE.glob("*.package")):
        try:
            entries, _ = parse_index(pkg)
        except Exception:
            continue
        with pkg.open("rb") as fh:
            for e in entries:
                if e["type"] != TARGET:
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


def main():
    samples = collect_samples(50)
    print(f"Collected {len(samples)} anim samples\n")

    for i, d in enumerate(samples[:5]):
        print(f"--- Sample {i+1} (size={len(d)}) ---")

        # Read offset table at 512
        num_offsets = struct.unpack_from("<I", d, 508)[0] if len(d) > 512 else 0
        print(f"  Offset table at 512: {num_offsets} entries")

        # Read first bone offset
        if num_offsets > 0 and len(d) > 516:
            bone_off = struct.unpack_from("<I", d, 512)[0]
            print(f"  First bone at offset: {bone_off}")

            # Parse bone section
            if bone_off + 320 <= len(d):
                # CHAN magic
                chan = d[bone_off:bone_off+4]
                print(f"  CHAN magic: {chan}")

                # Bone name
                name_end = bone_off + 4
                while name_end < len(d) and d[name_end] != 0:
                    name_end += 1
                bone_name = d[bone_off+4:name_end].decode("ascii", errors="replace")
                print(f"  Bone name: \"{bone_name}\"")

                # Headers relative to bone section start
                # After CHAN + name + padding, the bone section data starts
                # INFO at +232, POS at +264, ROT at +296 (relative to bone_offset)
                # But bone_offset is the offset of the u32 in the table, not the CHAN magic
                # Let me recalculate

                # The offset table entry points to the CHAN magic
                # bone_off is the offset of CHAN
                # After CHAN(4) + name + padding, headers are at fixed offsets

                # Actually, from the previous analysis:
                # INFO at bone_offset + 232
                # POS at bone_offset + 264
                # ROT at bone_offset + 296

                info_off = bone_off + 232
                pos_off = bone_off + 264
                rot_off = bone_off + 296

                if info_off + 16 <= len(d) and d[info_off:info_off+4] == b"INFO":
                    info_kf = struct.unpack_from("<I", d, info_off + 8)[0]
                    print(f"  INFO: kf_count={info_kf}")

                if pos_off + 16 <= len(d) and d[pos_off:pos_off+4] == b"POS\x00":
                    pos_v1 = struct.unpack_from("<I", d, pos_off + 4)[0]
                    pos_kf = struct.unpack_from("<I", d, pos_off + 8)[0]
                    pos_v3 = struct.unpack_from("<I", d, pos_off + 12)[0]
                    print(f"  POS: v1={pos_v1}, kf_count={pos_kf}, v3={pos_v3}")

                    # POS keyframe data should start at pos_off + 16
                    pos_data_off = pos_off + 16
                    pos_data_end = rot_off  # ROT header starts here
                    pos_data_size = pos_data_end - pos_data_off
                    print(f"    POS data: +{pos_data_off} to +{pos_data_end} ({pos_data_size} bytes)")
                    print(f"    If kf_count={pos_kf}: {pos_data_size / max(pos_kf, 1):.1f} bytes per keyframe")

                    # Show POS data hex
                    if pos_data_size > 0:
                        for row in range(pos_data_off, min(pos_data_off + 64, pos_data_end), 16):
                            hex_str = " ".join(f"{b:02x}" for b in d[row:row+16])
                            print(f"    +{row:04x}: {hex_str}")

                if rot_off + 16 <= len(d) and d[rot_off:rot_off+4] == b"ROT\x00":
                    rot_v1 = struct.unpack_from("<I", d, rot_off + 4)[0]
                    rot_kf = struct.unpack_from("<I", d, rot_off + 8)[0]
                    rot_v3 = struct.unpack_from("<I", d, rot_off + 12)[0]
                    print(f"  ROT: v1={rot_v1}, kf_count={rot_kf}, v3={rot_v3}")

                    # ROT keyframe data starts at rot_off + 16
                    rot_data_off = rot_off + 16
                    # Show first few keyframes
                    if rot_data_off + 40 <= len(d):
                        for kf in range(min(2, rot_kf)):
                            base = rot_data_off + kf * 20
                            time = struct.unpack_from("<f", d, base)[0]
                            qx = struct.unpack_from("<f", d, base + 4)[0]
                            qy = struct.unpack_from("<f", d, base + 8)[0]
                            qz = struct.unpack_from("<f", d, base + 12)[0]
                            qw = struct.unpack_from("<f", d, base + 16)[0]
                            print(f"    ROT kf{kf}: time={time:.4f}, quat=({qx:.4f}, {qy:.4f}, {qz:.4f}, {qw:.4f})")

                    # Where does ROT data end?
                    # Next bone section or end of file
                    if num_offsets > 1 and len(d) > 516:
                        next_bone = struct.unpack_from("<I", d, 516)[0]
                        rot_end = next_bone
                    else:
                        rot_end = len(d)
                    rot_size = rot_end - rot_data_off
                    print(f"    ROT data: +{rot_data_off} to +{rot_end} ({rot_size} bytes)")
                    print(f"    If kf_count={rot_kf}: {rot_size / max(rot_kf, 1):.1f} bytes per keyframe")
        print()


if __name__ == "__main__":
    main()

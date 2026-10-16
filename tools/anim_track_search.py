"""
Investigate anim track data by examining file offsets found in the metadata.

Earlier finding: after the 40-byte anim data header, there are u32 values
like 1, 21744, 10, 512, 43, 21572. The values 21744 and 21572 are close to
the file size (21860), suggesting they might be file offsets to track data.

This script:
  1. Parses the anim header and anim data header
  2. Extracts all u32 values in the first 100 bytes of anim data
  3. For values that could be file offsets, dumps the bytes at those offsets
  4. Searches for Cal3D-style track patterns (bone_id + kf_count + keyframes)
"""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

ANIM_TYPE = 0xEE17C6AD


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
            if len(raw) < 272 + 100:
                continue
            samples.append(raw)
    return samples


def dump_hex(d, start, length=64, label=""):
    end = min(start + length, len(d))
    print(f"    {label} (file offset {start}-{end}):")
    for j in range(start, end, 16):
        chunk = d[j:j+16]
        hexstr = chunk.hex(' ')
        asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
        print(f"      {j:>6}  {hexstr:<47}  {asc}")


def investigate(samples, n=5):
    """Investigate anim track data locations."""
    for idx in range(min(n, len(samples))):
        d = samples[idx]
        total = len(d)
        vcount_total = total

        # Parse anim data header
        data = d[272:]
        hash_val = struct.unpack_from("<I", data, 0)[0]
        frame_dur = struct.unpack_from("<f", data, 12)[0]
        duration = struct.unpack_from("<f", data, 16)[0]
        field_24 = struct.unpack_from("<I", data, 24)[0]
        field_28 = struct.unpack_from("<I", data, 28)[0]
        field_32 = struct.unpack_from("<I", data, 32)[0]

        print(f"\n{'='*70}")
        print(f"Sample {idx+1}: file_size={total}, hash=0x{hash_val:08X}")
        print(f"  frame_dur={frame_dur:.6f}, duration={duration:.1f}, "
              f"f24={field_24}, f28={field_28}, f32={field_32}")
        print(f"{'='*70}")

        # Extract all u32 values in first 100 bytes of anim data
        print(f"\n  All LE u32 in anim data (first 100 bytes):")
        for off in range(0, 100, 4):
            v = struct.unpack_from("<I", data, off)[0]
            f = struct.unpack_from("<f", data, off)[0]
            file_off = 272 + off
            note = ""
            if 0 < v < total:
                note = f" [file offset {v}, {total - v} bytes from end]"
            print(f"    anim+{off:>3} (file {file_off}): u32={v:<12} float={f:.4f}{note}")

        # For each u32 value that could be a file offset, dump bytes there
        print(f"\n  Dumping bytes at potential file offsets:")
        for off in range(0, 100, 4):
            v = struct.unpack_from("<I", data, off)[0]
            if 272 < v < total - 16:
                dump_hex(d, v, 48, f"value at anim+{off}")

        # Search for Cal3D-style track patterns
        # A track: bone_id(u32, small), kf_count(u32, small), then keyframes
        # Keyframe: time(f) + trans(3f) + rot(4f) = 32 bytes
        print(f"\n  Searching for Cal3D-style track patterns:")
        found = 0
        for off in range(40, min(len(data) - 40, 2000), 4):
            bone_id = struct.unpack_from("<I", data, off)[0]
            kf_count = struct.unpack_from("<I", data, off + 4)[0]

            if 0 < bone_id < 200 and 0 < kf_count < 500:
                # Check if keyframes look valid
                kf_off = off + 8
                if kf_off + 32 <= len(data):
                    time = struct.unpack_from("<f", data, kf_off)[0]
                    tx, ty, tz = struct.unpack_from("<fff", data, kf_off + 4)
                    rx, ry, rz, rw = struct.unpack_from("<ffff", data, kf_off + 16)

                    # Check if values are reasonable
                    if (abs(time) < duration * 2 and
                        abs(tx) < 100 and abs(ty) < 100 and abs(tz) < 100 and
                        abs(rx) <= 1.01 and abs(ry) <= 1.01 and
                        abs(rz) <= 1.01 and abs(rw) <= 1.01):
                        print(f"    Possible track at anim+{off}: bone={bone_id}, "
                              f"kf_count={kf_count}")
                        print(f"      KF[0]: t={time:.4f} "
                              f"trans=({tx:.3f},{ty:.3f},{tz:.3f}) "
                              f"rot=({rx:.3f},{ry:.3f},{rz:.3f},{rw:.3f})")
                        found += 1
                        if found >= 5:
                            break

        if found == 0:
            print(f"    No Cal3D-style tracks found in first 2000 bytes")

        # Also check: maybe the track data is at the end of the file
        # Look at the last 500 bytes
        print(f"\n  Last 200 bytes of file:")
        dump_hex(d, max(0, total - 200), 200, "file end")


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_samples(pkg, ANIM_TYPE, limit=10)
        except (ValueError, struct.error):
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} anim resources")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)}")
    if all_samples:
        investigate(all_samples, n=5)


if __name__ == "__main__":
    raise SystemExit(main())

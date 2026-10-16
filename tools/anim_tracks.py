"""
Deep analysis of ANIM animation track data (corrected offsets).

Animation data starts at offset 272. Structure:
  +0:  LE u32 (hash/ID, varies)
  +4:  LE u32 = 0
  +8:  LE u32 = 0
  +12: LE float = 0.0333 (1/30, frame duration)
  +16: LE float (duration in frames, e.g. 77.0)
  +20: LE u32 = 0
  +24: LE u32 (frame count - 1?)
  +28: LE u32 (offset or size?)
  +32: track data begins
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

ANIM_TYPE = 0xEE17C6AD


def get_samples(path: Path, target_type: int, limit: int = 500):
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
            if len(raw) < 272 + 64:
                continue
            samples.append(raw)
    return samples


def parse_anim_data(d: bytes):
    """Parse animation data at offset 272."""
    data = d[272:]
    result = {}
    result["hash"] = struct.unpack_from("<I", data, 0)[0]
    result["pad4"] = struct.unpack_from("<I", data, 4)[0]
    result["pad8"] = struct.unpack_from("<I", data, 8)[0]
    result["frame_dur"] = struct.unpack_from("<f", data, 12)[0]
    result["duration"] = struct.unpack_from("<f", data, 16)[0]
    result["pad20"] = struct.unpack_from("<I", data, 20)[0]
    result["field_24"] = struct.unpack_from("<I", data, 24)[0]
    result["field_28"] = struct.unpack_from("<I", data, 28)[0]
    result["field_32"] = struct.unpack_from("<I", data, 32)[0]
    result["field_36"] = struct.unpack_from("<I", data, 36)[0]
    return result, data


def analyze_tracks(samples, n=5):
    """Analyze track data in detail."""
    print("=" * 70)
    print("ANIM TRACK DATA ANALYSIS (corrected offsets)")
    print("=" * 70)

    for idx in range(min(n, len(samples))):
        d = samples[idx]
        fields, data = parse_anim_data(d)

        print(f"\n{'='*70}")
        print(f"Sample {idx+1}: total={len(d)}, anim_data={len(data)}")
        print(f"{'='*70}")

        fps = 1.0 / fields["frame_dur"] if fields["frame_dur"] > 0 else 0
        print(f"  hash:      0x{fields['hash']:08X}")
        print(f"  pad4:      {fields['pad4']}")
        print(f"  pad8:      {fields['pad8']}")
        print(f"  frame_dur: {fields['frame_dur']:.6f} ({fps:.1f} FPS)")
        print(f"  duration:  {fields['duration']:.1f} frames ({fields['duration'] * fields['frame_dur']:.2f} sec)")
        print(f"  pad20:     {fields['pad20']}")
        print(f"  field_24:  {fields['field_24']}")
        print(f"  field_28:  {fields['field_28']} (0x{fields['field_28']:08X})")
        print(f"  field_32:  {fields['field_32']} (0x{fields['field_32']:08X})")
        print(f"  field_36:  {fields['field_36']} (0x{fields['field_36']:08X})")

        # Dump first 128 bytes
        print(f"\n  First 128 bytes:")
        for j in range(0, min(128, len(data)), 16):
            chunk = data[j:j+16]
            hexstr = chunk.hex(' ')
            asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
            print(f"    {j:>5}  {hexstr:<47}  {asc}")

        # Try to parse tracks from offset 32
        # Cal3D style: bone_id(u32), kf_count(u32), then keyframes
        # Keyframe: time(f) + trans(3f) + rot(4f) = 32 bytes
        offset = 32
        print(f"\n  Track parsing (Cal3D style, 32-byte keyframes):")
        for track in range(20):
            if offset + 8 > len(data):
                print(f"    Reached end at track {track}")
                break

            bone_id = struct.unpack_from("<I", data, offset)[0]
            kf_count = struct.unpack_from("<I", data, offset + 4)[0]

            if kf_count > 10000 or bone_id > 10000:
                print(f"    Track {track}: bone_id={bone_id}, kf_count={kf_count} -- INVALID")
                # Try to resync by showing raw bytes
                print(f"      Raw: {data[offset:offset+16].hex(' ')}")
                break

            track_size = 8 + kf_count * 32
            print(f"    Track {track}: bone_id={bone_id}, kf_count={kf_count}, "
                  f"size={track_size}")

            if kf_count > 0 and offset + 8 + 32 <= len(data):
                kf_off = offset + 8
                time = struct.unpack_from("<f", data, kf_off)[0]
                tx, ty, tz = struct.unpack_from("<fff", data, kf_off + 4)
                rx, ry, rz, rw = struct.unpack_from("<ffff", data, kf_off + 16)
                print(f"      KF[0]: t={time:.4f} trans=({tx:.3f},{ty:.3f},{tz:.3f}) "
                      f"rot=({rx:.3f},{ry:.3f},{rz:.3f},{rw:.3f})")

                if kf_count > 1:
                    last = kf_off + (kf_count - 1) * 32
                    if last + 32 <= len(data):
                        time = struct.unpack_from("<f", data, last)[0]
                        tx, ty, tz = struct.unpack_from("<fff", data, last + 4)
                        rx, ry, rz, rw = struct.unpack_from("<ffff", data, last + 16)
                        print(f"      KF[-1]: t={time:.4f} trans=({tx:.3f},{ty:.3f},{tz:.3f}) "
                              f"rot=({rx:.3f},{ry:.3f},{rz:.3f},{rw:.3f})")

            offset += track_size

        # Also try: maybe tracks start at a different offset
        # field_28 or field_32 might be the track start offset
        for start_off in [fields["field_28"], fields["field_32"], fields["field_24"]]:
            if 0 < start_off < len(data) - 8:
                print(f"\n  Trying track parse from offset {start_off}:")
                bone_id = struct.unpack_from("<I", data, start_off)[0]
                kf_count = struct.unpack_from("<I", data, start_off + 4)[0]
                print(f"    bone_id={bone_id}, kf_count={kf_count}")
                if 0 < kf_count < 1000 and 0 < bone_id < 1000:
                    print(f"    Looks valid!")


def statistical_analysis(samples):
    """Statistical analysis of corrected anim data fields."""
    print(f"\n{'='*70}")
    print("ANIM DATA STATISTICS (corrected)")
    print(f"{'='*70}")

    frame_durs = Counter()
    durations = []
    field_24s = []
    field_28s = []
    field_32s = []
    field_36s = []
    pad4s = Counter()
    pad8s = Counter()
    pad20s = Counter()

    for d in samples:
        fields, _ = parse_anim_data(d)
        frame_durs[round(fields["frame_dur"], 4)] += 1
        durations.append(fields["duration"])
        field_24s.append(fields["field_24"])
        field_28s.append(fields["field_28"])
        field_32s.append(fields["field_32"])
        field_36s.append(fields["field_36"])
        pad4s[fields["pad4"]] += 1
        pad8s[fields["pad8"]] += 1
        pad20s[fields["pad20"]] += 1

    print(f"\n  Frame durations: {dict(frame_durs)}")
    print(f"  Duration range: {min(durations):.1f} to {max(durations):.1f}")
    print(f"  Field 24 range: {min(field_24s)} to {max(field_24s)}")
    print(f"  Field 28 range: {min(field_28s)} to {max(field_28s)}")
    print(f"  Field 32 range: {min(field_32s)} to {max(field_32s)}")
    print(f"  Field 36 range: {min(field_36s)} to {max(field_36s)}")
    print(f"  Pad4 values: {dict(pad4s)}")
    print(f"  Pad8 values: {dict(pad8s)}")
    print(f"  Pad20 values: {dict(pad20s)}")

    # Correlation between duration and field_24
    print(f"\n  Duration vs field_24 (first 10):")
    for i in range(min(10, len(samples))):
        fields, _ = parse_anim_data(samples[i])
        print(f"    dur={fields['duration']:.1f}, f24={fields['field_24']}, "
              f"f28={fields['field_28']}, f32={fields['field_32']}, f36={fields['field_36']}")


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_samples(pkg, ANIM_TYPE, limit=200)
        except (ValueError, struct.error):
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} anim resources")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)} anim resources")

    if all_samples:
        statistical_analysis(all_samples)
        analyze_tracks(all_samples, n=3)


if __name__ == "__main__":
    raise SystemExit(main())

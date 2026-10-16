"""
Full anim format parser and validator.

Validates the CHAN section structure across all anim samples:
  - Offset table at 512
  - Per-bone CHAN sections with INFO/POS/ROT headers
  - Keyframe counts and data sizes
  - Trailer with animation name

Usage:
    python tools/anim_validate.py
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
            if len(raw) < 600:
                continue
            samples.append(raw)
    return samples


def parse_anim(d: bytes) -> dict:
    """Parse a complete anim resource."""
    result = {
        "size": len(d),
        "errors": [],
        "bones": [],
    }

    # Header
    if d[:4] != b"ANIM":
        result["errors"].append("bad magic")
        return result

    file_size = struct.unpack_from("<I", d, 4)[0]
    if file_size != len(d):
        result["errors"].append(f"size mismatch: {file_size} vs {len(d)}")

    const_25 = struct.unpack_from("<I", d, 8)[0]

    # String field
    str_end = d.find(b"\x00", 12)
    if str_end < 0:
        result["errors"].append("no string terminator")
        return result
    result["path"] = d[12:str_end].decode("ascii", errors="replace")

    # Anim data header
    data = d[272:]
    result["hash"] = struct.unpack_from("<I", data, 0)[0]
    result["frame_dur"] = struct.unpack_from("<f", data, 12)[0]
    result["duration"] = struct.unpack_from("<f", data, 16)[0]
    result["field_24"] = struct.unpack_from("<I", data, 24)[0]
    result["field_28"] = struct.unpack_from("<I", data, 28)[0]
    result["field_32"] = struct.unpack_from("<I", data, 32)[0]

    # Offset table at 512
    chan_pos = d.find(b"CHAN", 512)
    if chan_pos < 0:
        result["errors"].append("no CHAN magic")
        return result

    num_offsets = (chan_pos - 512) // 4
    offsets = []
    for i in range(num_offsets):
        v = struct.unpack_from("<I", d, 512 + i * 4)[0]
        offsets.append(v)
    result["num_bones"] = num_offsets
    result["offsets"] = offsets

    # Parse each bone
    for bone_idx in range(num_offsets):
        bone_offset = offsets[bone_idx]
        if bone_idx + 1 < num_offsets:
            next_offset = offsets[bone_idx + 1]
        else:
            # Last bone ends at trailer
            trailer_pos = d.find(b"\x60\x00\x00\x00\x02\x00\x00\x00",
                                 len(d) - 500)
            next_offset = trailer_pos if trailer_pos > 0 else len(d)

        bone = {"offset": bone_offset, "section_len": next_offset - bone_offset}

        # Bone name
        name_start = bone_offset + 8
        name_end = d.find(b"\x00", name_start)
        if name_end < 0 or name_end > bone_offset + 64:
            bone["name"] = "?"
            bone["errors"] = ["bad name"]
        else:
            bone["name"] = d[name_start:name_end].decode("ascii", errors="replace")

        # INFO header at section+232
        info_pos = bone_offset + 232
        if info_pos + 16 <= len(d) and d[info_pos:info_pos+4] == b"INFO":
            bone["info_v1"] = struct.unpack_from("<I", d, info_pos + 4)[0]
            bone["info_kf"] = struct.unpack_from("<I", d, info_pos + 8)[0]
            bone["info_v3"] = struct.unpack_from("<I", d, info_pos + 12)[0]
        else:
            bone["errors"] = bone.get("errors", []) + ["no INFO"]

        # POS header at section+264
        pos_pos = bone_offset + 264
        if pos_pos + 16 <= len(d) and d[pos_pos:pos_pos+4] == b"POS\x00":
            bone["pos_v1"] = struct.unpack_from("<I", d, pos_pos + 4)[0]
            bone["pos_kf"] = struct.unpack_from("<I", d, pos_pos + 8)[0]
            bone["pos_v3"] = struct.unpack_from("<I", d, pos_pos + 12)[0]
            bone["has_pos"] = True
        else:
            bone["has_pos"] = False

        # ROT header at section+296
        rot_pos = bone_offset + 296
        if rot_pos + 16 <= len(d) and d[rot_pos:rot_pos+4] == b"ROT\x00":
            bone["rot_v1"] = struct.unpack_from("<I", d, rot_pos + 4)[0]
            bone["rot_kf"] = struct.unpack_from("<I", d, rot_pos + 8)[0]
            bone["rot_v3"] = struct.unpack_from("<I", d, rot_pos + 12)[0]
            bone["has_rot"] = True
        else:
            bone["has_rot"] = False

        result["bones"].append(bone)

    # Trailer
    name_end = len(d)
    while name_end > 0 and d[name_end - 1] == 0:
        name_end -= 1
    name_start = name_end
    while name_start > 0 and 32 <= d[name_start - 1] < 127:
        name_start -= 1
    if name_start < name_end:
        result["anim_name"] = d[name_start:name_end].decode("ascii", errors="replace")

    return result


def validate(samples):
    """Validate anim format across all samples."""
    print("=" * 70)
    print("ANIM FORMAT VALIDATION")
    print("=" * 70)

    total = len(samples)
    perfect = 0
    errors = Counter()
    bone_counts = Counter()
    info_offsets = Counter()
    pos_offsets = Counter()
    rot_offsets = Counter()
    kf_counts = Counter()
    pos_kf = Counter()
    rot_kf = Counter()
    bone_names = Counter()
    details = []

    for d in samples:
        r = parse_anim(d)

        if not r["errors"]:
            # Check all bones have INFO at +232
            all_ok = True
            for bone in r["bones"]:
                if "errors" in bone and bone["errors"]:
                    all_ok = False
                    break
                # Verify INFO at +232
                info_pos = bone["offset"] + 232
                if d[info_pos:info_pos+4] != b"INFO":
                    all_ok = False
                    break

            if all_ok:
                perfect += 1
            else:
                errors["bone errors"] += 1
        else:
            for e in r["errors"]:
                errors[e.split(":")[0]] += 1

        bone_counts[r.get("num_bones", 0)] += 1

        for bone in r["bones"]:
            bone_names[bone.get("name", "?")] += 1
            if "info_kf" in bone:
                kf_counts[bone["info_kf"]] += 1
            if bone.get("has_pos"):
                pos_kf[bone.get("pos_kf", 0)] += 1
            if bone.get("has_rot"):
                rot_kf[bone.get("rot_kf", 0)] += 1

        if len(details) < 3 and not r["errors"]:
            details.append(r)

    print(f"\n  Total: {total}")
    print(f"  Perfect parse: {perfect} ({perfect/total*100:.1f}%)")
    print(f"  Errors: {dict(errors)}")

    print(f"\n  Bone count distribution (top 10):")
    for v, c in bone_counts.most_common(10):
        print(f"    {v} bones: {c}")

    print(f"\n  Bone names (top 15):")
    for name, c in bone_names.most_common(15):
        print(f"    \"{name}\": {c}")

    print(f"\n  INFO keyframe counts (top 10):")
    for v, c in kf_counts.most_common(10):
        print(f"    {v}: {c}")

    print(f"\n  POS keyframe counts (top 5):")
    for v, c in pos_kf.most_common(5):
        print(f"    {v}: {c}")

    print(f"\n  ROT keyframe counts (top 5):")
    for v, c in rot_kf.most_common(5):
        print(f"    {v}: {c}")

    if details:
        print(f"\n  First 3 successful parses:")
        for i, r in enumerate(details):
            print(f"\n    Sample {i+1}: size={r['size']}, bones={r['num_bones']}, "
                  f"duration={r['duration']:.1f}, name=\"{r.get('anim_name', '?')}\"")
            for bone in r["bones"][:5]:
                pos = "POS" if bone.get("has_pos") else "---"
                rot = "ROT" if bone.get("has_rot") else "---"
                kf = bone.get("info_kf", "?")
                print(f"      {bone['name']:<20} kf={kf}  {pos}  {rot}")

    return perfect, total


def main():
    base = Path("D:/Games/SPORE Collection/Data")

    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_samples(pkg, ANIM_TYPE, limit=500)
        except (ValueError, struct.error):
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} anim resources")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)}")
    if all_samples:
        validate(all_samples)


if __name__ == "__main__":
    raise SystemExit(main())

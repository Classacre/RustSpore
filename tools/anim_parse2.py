"""
Analyze the ANIM format header and post-string data.

Structure discovered:
  offset  0: "ANIM" (4 bytes magic)
  offset  4: LE u32 = file size
  offset  8: LE u32 = 25 (constant?)
  offset 12: null-terminated ASCII string (Windows file path)
  after string: animation data (format unknown)

This script:
  1. Verifies the constant at offset 8 across all samples
  2. Finds where the string ends
  3. Analyzes the data after the string
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
            if len(raw) < 16:
                continue
            samples.append(raw)
    return samples


def parse_anim_header(d: bytes):
    """Parse the ANIM header and return string end offset."""
    if d[:4] != b"ANIM":
        return None

    file_size = struct.unpack_from("<I", d, 4)[0]
    const_25 = struct.unpack_from("<I", d, 8)[0]

    # Find null terminator of string starting at offset 12
    str_end = d.find(b"\x00", 12)
    if str_end < 0:
        return None

    path_str = d[12:str_end].decode("ascii", errors="replace")

    return {
        "file_size": file_size,
        "const_25": const_25,
        "path": path_str,
        "str_end": str_end,
        "data_start": str_end + 1,
        "total": len(d),
    }


def validate_header(samples):
    """Validate the ANIM header across all samples."""
    print("=" * 70)
    print("ANIM HEADER VALIDATION")
    print("=" * 70)

    total = len(samples)
    magic_ok = 0
    size_ok = 0
    const_vals = Counter()
    errors = []

    for d in samples:
        if d[:4] == b"ANIM":
            magic_ok += 1
        else:
            errors.append(f"bad magic: {d[:4].hex()}")
            continue

        file_size = struct.unpack_from("<I", d, 4)[0]
        if file_size == len(d):
            size_ok += 1

        const_25 = struct.unpack_from("<I", d, 8)[0]
        const_vals[const_25] += 1

    print(f"\n  Total: {total}")
    print(f"  Magic 'ANIM': {magic_ok}/{total} ({magic_ok/total*100:.1f}%)")
    print(f"  File size matches: {size_ok}/{total} ({size_ok/total*100:.1f}%)")
    print(f"  Constant @+8 values: {dict(const_vals)}")

    if errors:
        print(f"  Errors (first 5): {errors[:5]}")


def analyze_post_string(samples, n=10):
    """Analyze data after the string."""
    print(f"\n{'='*70}")
    print("POST-STRING DATA ANALYSIS")
    print(f"{'='*70}")

    details = []
    for d in samples[:n]:
        h = parse_anim_header(d)
        if h is None:
            continue

        data_start = h["data_start"]
        data = d[data_start:]
        data_len = len(data)

        # Skip leading null bytes
        non_null_start = 0
        while non_null_start < len(data) and data[non_null_start] == 0:
            non_null_start += 1

        actual_data = data[non_null_start:]
        actual_start = data_start + non_null_start

        print(f"\n  Sample: size={h['total']}, path_len={len(h['path'])}, "
              f"str_end={h['str_end']}, data_start={data_start}")
        print(f"    Path: {h['path'][:80]}...")
        print(f"    Null padding: {non_null_start} bytes")
        print(f"    Actual data starts at: {actual_start}")
        print(f"    Actual data length: {len(actual_data)}")

        if len(actual_data) >= 32:
            # Dump first 32 bytes of actual data
            print(f"    First 32 bytes of actual data:")
            for j in range(0, min(32, len(actual_data)), 16):
                chunk = actual_data[j:j+16]
                hexstr = chunk.hex(' ')
                asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
                print(f"      +{j:>3}  {hexstr:<47}  {asc}")

            # Try as u32 LE
            u32s = [struct.unpack_from("<I", actual_data, j)[0]
                    for j in range(0, min(16, len(actual_data) - 3), 4)]
            print(f"    As LE u32: {u32s}")

            # Try as floats
            floats = []
            for j in range(0, min(32, len(actual_data) - 3), 4):
                f = struct.unpack_from("<f", actual_data, j)[0]
                if abs(f) < 10000:
                    floats.append(f)
            if floats:
                print(f"    As LE floats: {['%.4f' % f for f in floats[:8]]}")

        details.append({
            "path": h["path"],
            "str_end": h["str_end"],
            "null_padding": non_null_start,
            "data_len": len(actual_data),
        })

    # Statistics on null padding
    paddings = Counter(d["null_padding"] for d in details)
    print(f"\n  Null padding distribution: {dict(paddings)}")


def statistical_post_string(samples):
    """Statistical analysis of post-string data across all samples."""
    print(f"\n{'='*70}")
    print("POST-STRING STATISTICS (all samples)")
    print(f"{'='*70}")

    null_paddings = Counter()
    first4_after_padding = Counter()
    first8_after_padding = Counter()
    path_lengths = Counter()

    for d in samples:
        h = parse_anim_header(d)
        if h is None:
            continue

        path_lengths[len(h["path"])] += 1

        data = d[h["data_start"]:]
        non_null_start = 0
        while non_null_start < len(data) and data[non_null_start] == 0:
            non_null_start += 1

        null_paddings[non_null_start] += 1

        actual = data[non_null_start:]
        if len(actual) >= 4:
            first4_after_padding[actual[:4].hex()] += 1
        if len(actual) >= 8:
            first8_after_padding[actual[:8].hex()] += 1

    print(f"\n  Path length distribution (top 10):")
    for v, c in path_lengths.most_common(10):
        print(f"    {v}: {c}")

    print(f"\n  Null padding distribution (top 10):")
    for v, c in null_paddings.most_common(10):
        print(f"    {v}: {c}")

    print(f"\n  First 4 bytes after padding (top 10):")
    for v, c in first4_after_padding.most_common(10):
        print(f"    {v}: {c}")

    print(f"\n  First 8 bytes after padding (top 10):")
    for v, c in first8_after_padding.most_common(10):
        print(f"    {v}: {c}")


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

    print(f"\n  Total: {len(all_samples)} anim resources")

    if all_samples:
        validate_header(all_samples)
        statistical_post_string(all_samples)
        analyze_post_string(all_samples, n=5)


if __name__ == "__main__":
    raise SystemExit(main())

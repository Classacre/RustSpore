"""
Deep investigation of the gmsh post-vertex-data structure.

After the vertex positions (vcount * 12 bytes), the data starts with:
    02 00 07 01 00 00 00 XX   (XX = vcount as BE u32)

This script investigates what follows that 8-byte sub-header:
  - Is there a second vertex attribute buffer?
  - Where does index data start?
  - What is the structure between attributes and indices?
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

GMSH_TYPE = 0x01C135DA


def get_gmsh_samples(path: Path, limit: int = 500):
    entries, _ = parse_index(path)
    samples = []
    with path.open("rb") as fh:
        for e in entries:
            if e["type"] != GMSH_TYPE:
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
            if len(raw) < 24:
                continue
            samples.append(raw)
    return samples


def find_magic(d: bytes, pattern: bytes, start: int = 0):
    """Find all occurrences of pattern in d."""
    results = []
    pos = start
    while True:
        idx = d.find(pattern, pos)
        if idx < 0:
            break
        results.append(idx)
        pos = idx + 1
    return results


def analyze_structure(samples, n=20):
    """For the smallest n samples, dump full structure."""
    sorted_s = sorted(samples, key=len)

    for idx in range(min(n, len(sorted_s))):
        d = sorted_s[idx]
        total = len(d)
        vc = struct.unpack_from(">H", d, 20)[0]
        vdata_end = 24 + vc * 12

        print(f"\n{'='*70}")
        print(f"Sample {idx+1}: size={total}, vcount={vc}, vdata_end={vdata_end}")
        print(f"{'='*70}")

        # Check for 02 00 07 01 pattern
        magic = b"\x02\x00\x07\x01"
        positions = find_magic(d, magic)
        print(f"  '02 00 07 01' found at offsets: {positions}")

        # Full hex dump
        print(f"  Full hex dump:")
        for j in range(0, total, 16):
            chunk = d[j:j+16]
            hexstr = chunk.hex(' ')
            asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
            print(f"    {j:>5}  {hexstr:<47}  {asc}")

        # Analyze post-vertex region
        if vdata_end < total:
            post = d[vdata_end:]
            post_len = len(post)
            print(f"\n  Post-vertex region ({post_len} bytes):")

            # Check first 8 bytes
            if post_len >= 8:
                magic_ok = post[:4] == magic
                vc_post = struct.unpack_from(">I", post, 4)[0]
                print(f"    Magic: {post[:4].hex(' ')} (match: {magic_ok})")
                print(f"    BE u32 @+4: {vc_post} (vcount={vc}, match: {vc_post == vc})")

            # After the 8-byte sub-header
            if post_len > 8:
                after = post[8:]
                after_len = len(after)
                print(f"    After 8-byte header: {after_len} bytes")

                # Try to find where valid u16 indices start
                # Valid index = value < vc
                for offset in range(0, min(after_len - 2, 200), 2):
                    all_valid = True
                    count = 0
                    for j in range(offset, after_len - 1, 2):
                        v = struct.unpack_from("<H", after, j)[0]
                        if v >= vc:
                            all_valid = False
                            break
                        count += 1
                        if count >= 10:  # Check at least 10 consecutive
                            break
                    if all_valid and count >= 10:
                        print(f"    Possible LE u16 index data at post+8+{offset} "
                              f"(abs offset {vdata_end + 8 + offset})")
                        vals = [struct.unpack_from("<H", after, offset + k*2)[0]
                                for k in range(min(20, (after_len - offset) // 2))]
                        print(f"      First 20: {vals}")
                        break

                # Also try BE u16
                for offset in range(0, min(after_len - 2, 200), 2):
                    all_valid = True
                    count = 0
                    for j in range(offset, after_len - 1, 2):
                        v = struct.unpack_from(">H", after, j)[0]
                        if v >= vc:
                            all_valid = False
                            break
                        count += 1
                        if count >= 10:
                            break
                    if all_valid and count >= 10:
                        print(f"    Possible BE u16 index data at post+8+{offset} "
                              f"(abs offset {vdata_end + 8 + offset})")
                        vals = [struct.unpack_from(">H", after, offset + k*2)[0]
                                for k in range(min(20, (after_len - offset) // 2))]
                        print(f"      First 20: {vals}")
                        break

                # Dump first 64 bytes after the 8-byte header as various types
                lim = min(64, after_len)
                print(f"    First {lim} bytes after header:")
                for j in range(0, lim, 16):
                    chunk = after[j:j+16]
                    hexstr = chunk.hex(' ')
                    asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
                    print(f"      +{j:>3}  {hexstr:<47}  {asc}")

                # Try as floats
                if after_len >= 16:
                    floats = []
                    for j in range(0, min(32, after_len - 3), 4):
                        f = struct.unpack_from("<f", after, j)[0]
                        floats.append(f)
                    print(f"    As LE floats: {['%.4f' % f for f in floats]}")

                # Try as u16 LE
                u16s = [struct.unpack_from("<H", after, j)[0]
                        for j in range(0, min(32, after_len - 1), 2)]
                print(f"    As LE u16: {u16s}")

                # Try as u16 BE
                u16s_be = [struct.unpack_from(">H", after, j)[0]
                           for j in range(0, min(32, after_len - 1), 2)]
                print(f"    As BE u16: {u16s_be}")

                # Try as u32 LE
                u32s = [struct.unpack_from("<I", after, j)[0]
                        for j in range(0, min(32, after_len - 3), 4)]
                print(f"    As LE u32: {u32s}")

        if idx >= 4:
            break  # Only show first 5 in detail


def statistical_magic_check(samples):
    """Check if 02 00 07 01 appears at the expected position in ALL samples."""
    print(f"\n{'='*70}")
    print("STATISTICAL: 02 00 07 01 position check")
    print(f"{'='*70}")

    magic = b"\x02\x00\x07\x01"
    match_count = 0
    mismatch = []

    for d in samples:
        vc = struct.unpack_from(">H", d, 20)[0]
        vdata_end = 24 + vc * 12
        if vdata_end + 4 <= len(d):
            if d[vdata_end:vdata_end+4] == magic:
                match_count += 1
            else:
                mismatch.append((len(d), vc, vdata_end,
                                d[vdata_end:vdata_end+4].hex(' ')))

    total = len(samples)
    print(f"  Magic at vdata_end: {match_count}/{total} ({match_count/total*100:.1f}%)")
    if mismatch:
        print(f"  Mismatches (first 10):")
        for m in mismatch[:10]:
            print(f"    size={m[0]}, vc={m[1]}, vdata_end={m[2]}, bytes={m[3]}")


def statistical_vcount_check(samples):
    """Check if BE u32 at vdata_end+4 equals vcount in ALL samples."""
    print(f"\n{'='*70}")
    print("STATISTICAL: BE u32 at vdata_end+4 == vcount?")
    print(f"{'='*70}")

    match = 0
    mismatch = []
    for d in samples:
        vc = struct.unpack_from(">H", d, 20)[0]
        vdata_end = 24 + vc * 12
        if vdata_end + 8 <= len(d):
            vc_post = struct.unpack_from(">I", d, vdata_end + 4)[0]
            if vc_post == vc:
                match += 1
            else:
                mismatch.append((len(d), vc, vc_post))

    total = len(samples)
    print(f"  BE u32 @vdata_end+4 == vcount: {match}/{total} ({match/total*100:.1f}%)")
    if mismatch:
        print(f"  Mismatches (first 10): {mismatch[:10]}")


def analyze_post_header(samples):
    """Analyze the structure after the 8-byte sub-header (magic + vcount)."""
    print(f"\n{'='*70}")
    print("ANALYSIS: structure after 02 00 07 01 + vcount")
    print(f"{'='*70}")

    # For each sample, compute the bytes remaining after the 8-byte header
    # and look for patterns
    remaining_sizes = Counter()
    u16_at_8 = Counter()  # BE u16 at post+8
    u16_at_10 = Counter()  # BE u16 at post+10
    u32_at_8 = Counter()  # BE u32 at post+8

    for d in samples:
        vc = struct.unpack_from(">H", d, 20)[0]
        vdata_end = 24 + vc * 12
        post = d[vdata_end:]
        if len(post) < 12:
            continue

        remaining = len(post) - 8
        remaining_sizes[remaining] += 1

        u16_8 = struct.unpack_from(">H", post, 8)[0]
        u16_at_8[u16_8] += 1

        if len(post) >= 12:
            u16_10 = struct.unpack_from(">H", post, 10)[0]
            u16_at_10[u16_10] += 1

        if len(post) >= 12:
            u32_8 = struct.unpack_from(">I", post, 8)[0]
            u32_at_8[u32_8] += 1

    print(f"\n  BE u16 @post+8 (top 10): {u16_at_8.most_common(10)}")
    print(f"  BE u16 @post+10 (top 10): {u16_at_10.most_common(10)}")
    print(f"  BE u32 @post+8 (top 10): {u32_at_8.most_common(10)}")

    # Check if remaining / vcount gives a clean number
    clean_fits = 0
    for d in samples:
        vc = struct.unpack_from(">H", d, 20)[0]
        vdata_end = 24 + vc * 12
        post = d[vdata_end:]
        if len(post) < 12 or vc == 0:
            continue
        remaining = len(post) - 8
        # Try various per-vertex sizes
        for per_v in [4, 8, 12, 16, 20, 24, 32, 36, 48]:
            if remaining % vc == 0 and remaining // vc == per_v:
                clean_fits += 1
                break

    print(f"\n  Samples where (remaining-8) / vcount is a clean per-vertex size: "
          f"{clean_fits}/{len(samples)}")

    # More detailed: for each sample, compute remaining-8 and divide by vcount
    per_vertex = Counter()
    for d in samples[:200]:
        vc = struct.unpack_from(">H", d, 20)[0]
        vdata_end = 24 + vc * 12
        post = d[vdata_end:]
        if len(post) < 12 or vc == 0:
            continue
        remaining = len(post) - 8
        ratio = remaining / vc
        per_vertex[round(ratio, 2)] += 1

    print(f"  (remaining-8)/vcount distribution (top 20):")
    for val, cnt in per_vertex.most_common(20):
        print(f"    {val}: {cnt}")


def find_index_start(samples):
    """For each sample, try to find where valid u16 indices begin after the 8-byte header."""
    print(f"\n{'='*70}")
    print("FINDING: index data start position")
    print(f"{'='*70}")

    index_starts = Counter()
    details = []

    for d in samples[:500]:
        vc = struct.unpack_from(">H", d, 20)[0]
        vdata_end = 24 + vc * 12
        post = d[vdata_end:]
        if len(post) < 16 or vc < 4:
            continue

        after = post[8:]  # skip magic + vcount
        after_len = len(after)

        # Scan for a run of valid LE u16 indices (< vc)
        best_start = None
        best_count = 0
        for offset in range(0, min(after_len - 4, 300), 2):
            count = 0
            for j in range(offset, after_len - 1, 2):
                v = struct.unpack_from("<H", after, j)[0]
                if v >= vc:
                    break
                count += 1
            if count > best_count:
                best_count = count
                best_start = offset

        if best_start is not None and best_count >= 10:
            # The index data starts at post + 8 + best_start
            # But there might be a header before it
            abs_start = vdata_end + 8 + best_start
            index_starts[best_start] += 1
            if len(details) < 10:
                details.append({
                    "size": len(d),
                    "vc": vc,
                    "vdata_end": vdata_end,
                    "post_len": len(post),
                    "index_offset_in_post": best_start,
                    "index_count": best_count,
                    "abs_index_start": abs_start,
                })

    print(f"  Index offset within post-8-byte-header (top 20):")
    for val, cnt in index_starts.most_common(20):
        print(f"    offset {val}: {cnt} samples")

    if details:
        print(f"\n  First 10 samples with index data:")
        for det in details:
            print(f"    size={det['size']}, vc={det['vc']}, "
                  f"idx_offset_in_post={det['index_offset_in_post']}, "
                  f"idx_count={det['index_count']}")


def main():
    base = Path("D:/Games/SPORE Collection/Data")
    all_samples = []
    for pkg in sorted(base.glob("*.package")):
        try:
            samples = get_gmsh_samples(pkg, limit=500)
        except (ValueError, struct.error):
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} gmsh resources")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)} samples")

    statistical_magic_check(all_samples)
    statistical_vcount_check(all_samples)
    analyze_post_header(all_samples)
    find_index_start(all_samples)
    analyze_structure(all_samples, n=5)


if __name__ == "__main__":
    raise SystemExit(main())

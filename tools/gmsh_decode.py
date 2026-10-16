"""
Focused gmsh decoder test.

Hypothesis from the 480-byte sample:
  - Header is 24 bytes.
  - u16 @20 BIG-ENDIAN = vertex count
  - u16 @22 BIG-ENDIAN = vertex stride in bytes (always 12 = 3 floats)
  - Vertex data: offset 24, length = vcount * stride
  - After vertex data: index buffer (format unknown)

This script:
  1. Validates the header fields across all gmsh resources.
  2. Tests whether vcount * stride + 24 divides cleanly into vertex + index regions.
  3. Attempts to decode the index buffer with several hypotheses.
  4. Cross-validates vertex coordinates (should be small floats near origin).

Usage:
    python tools/gmsh_decode.py "D:/Games/SPORE Collection/Data/Spore_Graphics.package"
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress  # noqa: E402
import prop_parse as P  # noqa: E402
from dbpf_extract import parse_index  # noqa: E402

GMSH_TYPE = 0x01C135DA


def iterate_index(path: Path):
    """Yield (offset, stored_size, decompressed_size, type, group, instance) for all resources."""
    entries, _size = parse_index(path)
    for e in entries:
        yield (e["offset"], e["stored"], e["dec"],
               e["type"], e["group"], e["instance"])


def get_payload(fh, off, stored, dec):
    fh.seek(off)
    raw = fh.read(stored)
    if dec > stored:
        try:
            return decompress(raw, out_size=None, header=True)[0]
        except Exception:
            return None
    return raw


def collect_gmsh(path: Path, limit: int = 2000):
    """Collect all gmsh payloads up to limit."""
    samples = []
    with path.open("rb") as fh:
        for off, stored, dec, typ, group, inst in iterate_index(path):
            if typ != GMSH_TYPE:
                continue
            if len(samples) >= limit:
                break
            d = get_payload(fh, off, stored, dec)
            if d is None or len(d) < 24:
                continue
            samples.append((d, group, inst))
    return samples


def test_header_hypothesis(samples):
    """Test the hypothesis:
      - Bytes 0-15: some header (version, flags, etc.)
      - Bytes 16-19: u32 LE = 3 (attribute count?)
      - Bytes 20-21: u16 BE = vertex count
      - Bytes 22-23: u16 BE = vertex stride (always 12)
      - Bytes 24+: vertex data then index data
    """
    print("=" * 70)
    print("TEST 1: Header field validation")
    print("=" * 70)

    u0_vals = Counter()  # offset 0 as LE u32
    u4_vals = Counter()  # offset 4 as LE u32
    u8_vals = Counter()  # offset 8 as LE u32
    u12_vals = Counter()  # offset 12 as LE u32
    u16_vals = Counter()  # offset 16 as LE u32

    # The hypothesis fields
    vcount_be_vals = Counter()  # u16 BE @20
    stride_be_vals = Counter()  # u16 BE @22
    vcount_le_vals = Counter()  # u16 LE @20
    stride_le_vals = Counter()  # u16 LE @22

    # Also try u32 LE @20 and u32 BE @20
    u20_le_vals = Counter()
    u20_be_vals = Counter()

    for d, g, i in samples:
        u0_vals[struct.unpack_from("<I", d, 0)[0]] += 1
        u4_vals[struct.unpack_from("<I", d, 4)[0]] += 1
        u8_vals[struct.unpack_from("<I", d, 8)[0]] += 1
        u12_vals[struct.unpack_from("<I", d, 12)[0]] += 1
        u16_vals[struct.unpack_from("<I", d, 16)[0]] += 1

        vcount_be_vals[struct.unpack_from(">H", d, 20)[0]] += 1
        stride_be_vals[struct.unpack_from(">H", d, 22)[0]] += 1
        vcount_le_vals[struct.unpack_from("<H", d, 20)[0]] += 1
        stride_le_vals[struct.unpack_from("<H", d, 22)[0]] += 1

        u20_le_vals[struct.unpack_from("<I", d, 20)[0]] += 1
        u20_be_vals[struct.unpack_from(">I", d, 20)[0]] += 1

    print(f"\n  Samples: {len(samples)}")

    for name, c in [("u0_LE", u0_vals), ("u4_LE", u4_vals), ("u8_LE", u8_vals),
                    ("u12_LE", u12_vals), ("u16_LE", u16_vals)]:
        top = c.most_common(5)
        print(f"  {name}: {len(c)} distinct, top: {top}")

    print(f"\n  vcount_BE@20: {len(vcount_be_vals)} distinct")
    top_vc = vcount_be_vals.most_common(15)
    print(f"    top 15: {top_vc}")

    print(f"\n  stride_BE@22: {len(stride_be_vals)} distinct")
    top_st = stride_be_vals.most_common(10)
    print(f"    top 10: {top_st}")

    print(f"\n  vcount_LE@20: {len(vcount_le_vals)} distinct")
    top_vc_le = vcount_le_vals.most_common(10)
    print(f"    top 10: {top_vc_le}")

    print(f"\n  stride_LE@22: {len(stride_le_vals)} distinct")
    top_st_le = stride_le_vals.most_common(10)
    print(f"    top 10: {top_st_le}")

    print(f"\n  u20_BE: {len(u20_be_vals)} distinct, top 5: {u20_be_vals.most_common(5)}")
    print(f"  u20_LE: {len(u20_le_vals)} distinct, top 5: {u20_le_vals.most_common(5)}")

    return vcount_be_vals, stride_be_vals


def test_vertex_region(samples, stride_override=None):
    """Test if vcount * stride + 24 fits within the file and leaves room for indices.

    Also check that the vertex data looks like small floats (vertex coordinates).
    """
    print("\n" + "=" * 70)
    print("TEST 2: Vertex region fit")
    print("=" * 70)

    exact_fit = 0
    fits_with_room = 0
    too_short = 0
    too_long = 0
    bad_stride = 0

    # Also track: does the vertex data actually look like floats?
    float_ok = 0
    float_bad = 0

    # Track remaining bytes after vertex data
    remaining = Counter()

    fit_details = []

    for idx, (d, g, inst) in enumerate(samples):
        vc_be = struct.unpack_from(">H", d, 20)[0]
        st_be = struct.unpack_from(">H", d, 22)[0]

        # Use the override stride if given (e.g., 12)
        stride = stride_override if stride_override else st_be

        if stride == 0 or stride > 256:
            bad_stride += 1
            continue

        vdata_size = vc_be * stride
        vdata_end = 24 + vdata_size
        total = len(d)

        if vdata_end > total:
            too_short += 1
            continue
        if vdata_end == total:
            exact_fit += 1
            remaining[0] += 1
            continue

        # There's data after vertices
        rem = total - vdata_end
        fits_with_room += 1
        remaining[rem] += 1

        # Check float sanity for the first few vertices
        ok = True
        for vi in range(min(vc_be, 20)):
            base = 24 + vi * stride
            for fi in range(stride // 4):
                f = struct.unpack_from("<f", d, base + fi * 4)[0]
                if f != f or abs(f) > 10000:  # NaN or absurd
                    ok = False
                    break
            if not ok:
                break
        if ok:
            float_ok += 1
        else:
            float_bad += 1

        if idx < 10:
            fit_details.append({
                "size": total,
                "vcount": vc_be,
                "stride": stride,
                "vdata_end": vdata_end,
                "remaining": rem,
                "float_ok": ok,
            })

    print(f"\n  Results (stride={'override=' + str(stride_override) if stride_override else 'from BE@22'}):")
    print(f"    exact fit (no index data): {exact_fit}")
    print(f"    fits with room for indices: {fits_with_room}")
    print(f"    too short: {too_short}")
    print(f"    too long (stride 0 or >256): {bad_stride}")
    print(f"    vertex floats look valid: {float_ok}")
    print(f"    vertex floats look invalid: {float_bad}")

    print(f"\n  Remaining bytes distribution (top 20):")
    for val, cnt in remaining.most_common(20):
        print(f"    {val:>8} bytes: {cnt} files")

    if fit_details:
        print(f"\n  First 10 samples:")
        for s in fit_details:
            print(f"    size={s['size']}, vcount={s['vcount']}, stride={s['stride']}, "
                  f"vdata_end={s['vdata_end']}, remaining={s['remaining']}, float_ok={s['float_ok']}")

    return remaining


def test_index_buffer(samples, stride=12):
    """For samples where the vertex region fits, try to decode the index buffer.

    Hypotheses for the index buffer (starting right after vertex data):
      A) Simple: u32 count, then count * u16 indices
      B) Simple: u32 count, then count * u32 indices
      C) GMDL-style: u32 prim_type, u32 count, u32 bits, u32 buf_size, then data
      D) LE u16 count, then count * u16 indices
      E) No header, just raw u16 indices to end of file
      F) No header, just raw u32 indices to end of file
    """
    print("\n" + "=" * 70)
    print(f"TEST 3: Index buffer decode (vertex stride={stride})")
    print("=" * 70)

    results = {hypo: {"ok": 0, "fail": 0} for hypo in "ABCDEF"}
    detail_samples = []

    for idx, (d, g, inst) in enumerate(samples):
        vc = struct.unpack_from(">H", d, 20)[0]
        vdata_end = 24 + vc * stride
        total = len(d)

        if vdata_end >= total:
            continue

        idx_start = vdata_end
        idx_bytes = total - idx_start

        if idx < 5:
            detail_samples.append({
                "size": total,
                "vcount": vc,
                "idx_start": idx_start,
                "idx_bytes": idx_bytes,
                "first16": d[idx_start:idx_start + min(16, idx_bytes)].hex(" "),
            })

        # Hypothesis A: u32 LE count, then u16 indices
        if idx_bytes >= 4:
            count_a = struct.unpack_from("<I", d, idx_start)[0]
            expected_a = 4 + count_a * 2
            if expected_a == idx_bytes and count_a < vc * 4:
                results["A"]["ok"] += 1
            else:
                results["A"]["fail"] += 1
        else:
            results["A"]["fail"] += 1

        # Hypothesis B: u32 LE count, then u32 indices
        if idx_bytes >= 4:
            count_b = struct.unpack_from("<I", d, idx_start)[0]
            expected_b = 4 + count_b * 4
            if expected_b == idx_bytes and count_b < vc * 4:
                results["B"]["ok"] += 1
            else:
                results["B"]["fail"] += 1
        else:
            results["B"]["fail"] += 1

        # Hypothesis C: GMDL TriangleBuffer (4 u32 header, then data)
        if idx_bytes >= 16:
            prim_c = struct.unpack_from("<I", d, idx_start)[0]
            icount_c = struct.unpack_from("<I", d, idx_start + 4)[0]
            ibits_c = struct.unpack_from("<I", d, idx_start + 8)[0]
            bsize_c = struct.unpack_from("<I", d, idx_start + 12)[0]
            # ibits should be 16 or 32, bsize should match remaining
            if ibits_c in (16, 32) and bsize_c == idx_bytes - 16:
                if ibits_c == 16:
                    expected_c = icount_c * 2
                else:
                    expected_c = icount_c * 4
                if expected_c == bsize_c and icount_c < vc * 4:
                    results["C"]["ok"] += 1
                else:
                    results["C"]["fail"] += 1
            else:
                results["C"]["fail"] += 1
        else:
            results["C"]["fail"] += 1

        # Hypothesis D: u16 LE count, then u16 indices
        if idx_bytes >= 2:
            count_d = struct.unpack_from("<H", d, idx_start)[0]
            expected_d = 2 + count_d * 2
            if expected_d == idx_bytes and count_d < vc * 4:
                results["D"]["ok"] += 1
            else:
                results["D"]["fail"] += 1
        else:
            results["D"]["fail"] += 1

        # Hypothesis E: raw u16 to end
        if idx_bytes % 2 == 0:
            num_idx = idx_bytes // 2
            # All indices must be < vcount
            ok = True
            for j in range(num_idx):
                v = struct.unpack_from("<H", d, idx_start + j * 2)[0]
                if v >= vc:
                    ok = False
                    break
            if ok and num_idx > 0:
                results["E"]["ok"] += 1
            else:
                results["E"]["fail"] += 1
        else:
            results["E"]["fail"] += 1

        # Hypothesis F: raw u32 to end
        if idx_bytes % 4 == 0:
            num_idx = idx_bytes // 4
            ok = True
            for j in range(min(num_idx, 200)):  # check first 200
                v = struct.unpack_from("<I", d, idx_start + j * 4)[0]
                if v >= vc:
                    ok = False
                    break
            if ok and num_idx > 0:
                results["F"]["ok"] += 1
            else:
                results["F"]["fail"] += 1
        else:
            results["F"]["fail"] += 1

    print(f"\n  Results:")
    for hypo, counts in results.items():
        total = counts["ok"] + counts["fail"]
        pct = counts["ok"] / total * 100 if total else 0
        print(f"    {hypo}: ok={counts['ok']}, fail={counts['fail']}, pct={pct:.1f}%")

    if detail_samples:
        print(f"\n  First 5 index region samples:")
        for s in detail_samples:
            print(f"    size={s['size']}, vcount={s['vcount']}, idx_start={s['idx_start']}, "
                  f"idx_bytes={s['idx_bytes']}")
            print(f"      first16: {s['first16']}")

    return results


def deep_inspect(samples, n=5):
    """For a few small samples, dump everything we know."""
    print("\n" + "=" * 70)
    print("DEEP INSPECTION of smallest samples")
    print("=" * 70)

    # Sort by size
    sorted_samples = sorted(samples, key=lambda x: len(x[0]))

    for idx in range(min(n, len(sorted_samples))):
        d, g, inst = sorted_samples[idx]
        total = len(d)
        print(f"\n--- Sample {idx + 1} (size={total}, group=0x{g:08X}) ---")

        # Full header hex
        print(f"  Header (24 bytes):")
        for j in range(0, min(24, total), 4):
            u32_le = struct.unpack_from("<I", d, j)[0]
            u32_be = struct.unpack_from(">I", d, j)[0]
            raw = d[j:j + 4].hex(" ")
            print(f"    off {j:>2}: {raw}  LE=0x{u32_le:08X} ({u32_le})  BE=0x{u32_be:08X} ({u32_be})")

        # u16 at offset 20
        vc_be = struct.unpack_from(">H", d, 20)[0]
        st_be = struct.unpack_from(">H", d, 22)[0]
        vc_le = struct.unpack_from("<H", d, 20)[0]
        st_le = struct.unpack_from("<H", d, 22)[0]
        print(f"  u16@20 BE={vc_be} LE={vc_le}  u16@22 BE={st_be} LE={st_le}")

        # Vertex data interpretation (stride=12, 3 floats per vertex)
        stride = 12
        vdata_end = 24 + vc_be * stride
        print(f"  If vcount={vc_be}, stride={stride}: vdata_end={vdata_end}, remaining={total - vdata_end}")

        if vdata_end <= total:
            print(f"  Vertex data (first 5 vertices, 3 floats each):")
            for vi in range(min(vc_be, 5)):
                base = 24 + vi * stride
                x, y, z = struct.unpack_from("<fff", d, base)
                print(f"    v[{vi}]: ({x:.6f}, {y:.6f}, {z:.6f})")

        # Index region
        if vdata_end < total:
            idx_bytes = total - vdata_end
            print(f"  Index region: {idx_bytes} bytes starting at offset {vdata_end}")
            print(f"    First 32 bytes hex: {d[vdata_end:vdata_end + min(32, idx_bytes)].hex(' ')}")

            # Try various reads on the first few values
            if idx_bytes >= 4:
                u32_le_0 = struct.unpack_from("<I", d, vdata_end)[0]
                u32_be_0 = struct.unpack_from(">I", d, vdata_end)[0]
                u16_le_0 = struct.unpack_from("<H", d, vdata_end)[0]
                u16_be_0 = struct.unpack_from(">H", d, vdata_end)[0]
                print(f"    First u32 LE={u32_le_0} BE={u32_be_0}  u16 LE={u16_le_0} BE={u16_be_0}")

            if idx_bytes >= 8:
                u32_le_1 = struct.unpack_from("<I", d, vdata_end + 4)[0]
                u32_be_1 = struct.unpack_from(">I", d, vdata_end + 4)[0]
                print(f"    Second u32 LE={u32_le_1} BE={u32_be_1}")

            if idx_bytes >= 16:
                u32_le_2 = struct.unpack_from("<I", d, vdata_end + 8)[0]
                u32_le_3 = struct.unpack_from("<I", d, vdata_end + 12)[0]
                print(f"    Third u32 LE={u32_le_2}  Fourth u32 LE={u32_le_3}")

            # Try raw u16 indices
            if idx_bytes % 2 == 0:
                u16s = []
                for j in range(idx_bytes // 2):
                    u16s.append(struct.unpack_from("<H", d, vdata_end + j * 2)[0])
                print(f"    As LE u16: {u16s[:30]}")
                max_idx = max(u16s)
                print(f"    Max u16={max_idx}, vcount={vc_be}, all < vcount: {max_idx < vc_be}")

            # Try raw u32 indices
            if idx_bytes % 4 == 0:
                u32s = []
                for j in range(min(idx_bytes // 4, 30)):
                    u32s.append(struct.unpack_from("<I", d, vdata_end + j * 4)[0])
                print(f"    As LE u32: {u32s}")

        # Also try with larger strides to see if multi-attribute vertices work
        print(f"\n  Trying larger strides:")
        for st in [24, 32, 36, 48]:
            vd_end = 24 + vc_be * st
            if vd_end <= total:
                rem = total - vd_end
                print(f"    stride={st}: vdata_end={vd_end}, remaining={rem}")
                # Check float sanity
                ok = True
                for vi in range(min(vc_be, 5)):
                    base = 24 + vi * st
                    for fi in range(st // 4):
                        if base + fi * 4 + 4 <= total:
                            f = struct.unpack_from("<f", d, base + fi * 4)[0]
                            if f != f or abs(f) > 10000:
                                ok = False
                                break
                print(f"      floats valid: {ok}")


def test_all_packages():
    """Run across all packages that contain gmsh resources."""
    base = Path("D:/Games/SPORE Collection/Data")
    packages = list(base.glob("*.package"))

    all_samples = []
    for pkg in sorted(packages):
        try:
            samples = collect_gmsh(pkg, limit=500)
        except (ValueError, struct.error) as e:
            print(f"  {pkg.name}: skipped ({e})")
            continue
        if samples:
            print(f"  {pkg.name}: {len(samples)} gmsh resources")
            all_samples.extend(samples)

    print(f"\n  Total: {len(all_samples)} gmsh resources across {len(packages)} packages")
    return all_samples


def main() -> int:
    base = Path("D:/Games/SPORE Collection/Data")

    # Collect from all packages
    print("Collecting gmsh resources from all packages...")
    all_samples = test_all_packages()

    if not all_samples:
        print("No gmsh resources found!")
        return 1

    # Run tests
    test_header_hypothesis(all_samples)
    remaining = test_vertex_region(all_samples, stride_override=12)
    test_index_buffer(all_samples, stride=12)
    deep_inspect(all_samples, n=10)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

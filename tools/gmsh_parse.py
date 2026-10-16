"""
Parse SPORE gmsh mesh resources (type 0x01C135DA).

Status: header partially understood. Initial investigation found:
  - All resources start with 0x01000000 (u32 = 1)
  - Float vertex data begins at or near offset 24
  - Offset 16 = 3 in all samples (possible attribute count)

This tool performs deeper statistical analysis to determine:
  - The exact header layout and field meanings
  - Vertex buffer location, stride, and attribute layout
  - Index buffer location and format
  - Sub-mesh structure

Usage:
    python tools/gmsh_parse.py "D:/Games/SPORE Collection/Data/Spore_Graphics.package"
    python tools/gmsh_parse.py <package> dump
    python tools/gmsh_parse.py <package> stats
"""

import struct
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress  # noqa: E402
import prop_parse as P  # noqa: E402

GMSH_TYPE = 0x01C135DA


def iterate_index(path: Path):
    """Yield (offset, stored_size, decompressed_size, type, group, instance) for all resources."""
    with path.open("rb") as fh:
        head = fh.read(0x60)
        count = struct.unpack_from("<I", head, 0x24)[0]
        idx_off = struct.unpack_from("<I", head, 0x40)[0]
        fh.seek(idx_off + 4)
        blob = fh.read(count * P.REC)
        for i in range(count):
            g, t, ihi, ilo, off, sz, dec = struct.unpack_from("<IIIIIII", blob, i * P.REC)
            yield off, sz & 0x7FFFFFFF, dec, t, g, (ihi << 32) | ilo


def get_payload(fh, off, stored, dec):
    """Read and optionally decompress a resource payload."""
    fh.seek(off)
    raw = fh.read(stored)
    if dec > stored:
        try:
            return decompress(raw, out_size=None, header=True)[0]
        except Exception:
            return None
    return raw


def parse_gmsh_header(d: bytes) -> dict:
    """Parse the gmsh header fields.

    Based on prior investigation:
      offset  0: u32, always 0x01000000 (version or magic)
      offset  4: u32
      offset  8: u32
      offset 12: u32
      offset 16: u32, always 3 (attribute count?)
      offset 20: u32
      offset 24+: float vertex data?

    This is a working hypothesis, to be confirmed or revised by the
    statistical analysis in this tool.
    """
    if len(d) < 32:
        raise ValueError(f"too short: {len(d)} bytes")

    h = {
        "size": len(d),
        "u0": struct.unpack_from("<I", d, 0)[0],
        "u4": struct.unpack_from("<I", d, 4)[0],
        "u8": struct.unpack_from("<I", d, 8)[0],
        "u12": struct.unpack_from("<I", d, 12)[0],
        "u16": struct.unpack_from("<I", d, 16)[0],
        "u20": struct.unpack_from("<I", d, 20)[0],
    }
    return h


def dump_first_n(path: Path, n: int = 5, hex_bytes: int = 128):
    """Dump the first n gmsh resources with hex and field analysis."""
    with path.open("rb") as fh:
        shown = 0
        for off, stored, dec, typ, group, inst in iterate_index(path):
            if typ != GMSH_TYPE:
                continue
            if shown >= n:
                break
            d = get_payload(fh, off, stored, dec)
            if d is None or len(d) < 32:
                continue
            shown += 1

            h = parse_gmsh_header(d)
            print(f"=== gmsh #{shown}: size={len(d)}, group=0x{group:08X}, inst=0x{inst:016X}")
            for k, v in h.items():
                if k == "size":
                    continue
                print(f"  {k:>4} = 0x{v:08X} ({v})")

            # Hex dump
            lim = min(hex_bytes, len(d))
            print(f"  First {lim} bytes:")
            for j in range(0, lim, 16):
                chunk = d[j:j + 16]
                hexstr = chunk.hex(' ')
                asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
                print(f"    {j:>5}  {hexstr:<47}  {asc}")

            # Try interpreting offset 24 onward as floats
            print(f"  Float scan from offset 24:")
            for j in range(24, min(24 + 48, len(d) - 3), 4):
                f = struct.unpack_from("<f", d, j)[0]
                print(f"    off {j:>4}: {f:>12.6f}")
            print()


def statistical_summary(path: Path, sample_limit: int = 500):
    """Statistical analysis of gmsh header fields across many samples."""
    field_vals = {f"u{i}": Counter() for i in [0, 4, 8, 12, 16, 20]}
    sizes = Counter()
    total = 0
    bad = 0
    float_at_24 = []

    with path.open("rb") as fh:
        for off, stored, dec, typ, group, inst in iterate_index(path):
            if typ != GMSH_TYPE:
                continue
            if total >= sample_limit:
                break
            d = get_payload(fh, off, stored, dec)
            if d is None or len(d) < 32:
                bad += 1
                continue
            total += 1

            h = parse_gmsh_header(d)
            for key in field_vals:
                field_vals[key][h[key]] += 1
            sizes[len(d)] += 1

            # Sample first float at offset 24
            if len(d) >= 28:
                f24 = struct.unpack_from("<f", d, 24)[0]
                float_at_24.append(f24)

    print(f"gmsh resources: {total} sampled (limit {sample_limit}), {bad} skipped")
    print()

    for key in ["u0", "u4", "u8", "u12", "u16", "u20"]:
        vals = field_vals[key]
        print(f"  {key} distinct values: {len(vals)}")
        for v, c in vals.most_common(10):
            print(f"    0x{v:08X} ({v}): {c}")
        print()

    print(f"  Size distribution (top 10): {sizes.most_common(10)}")
    print()

    if float_at_24:
        pos = sum(1 for f in float_at_24 if abs(f) < 1.0 and f != 0.0)
        zero = sum(1 for f in float_at_24 if f == 0.0)
        big = sum(1 for f in float_at_24 if abs(f) >= 1.0)
        print(f"  Float@24: {pos} small (|f|<1), {zero} zero, {big} big (|f|>=1)")
        print(f"  Float@24 sample: {[f'{f:.4f}' for f in float_at_24[:10]]}")


def stride_analysis(path: Path, sample_limit: int = 20):
    """Analyze vertex stride by looking for repeating float patterns.

    If vertices start at some offset and each has N attributes (floats),
    the stride is N*4 bytes. We look for the stride by finding where
    the first float value pattern repeats.
    """
    with path.open("rb") as fh:
        shown = 0
        for off, stored, dec, typ, group, inst in iterate_index(path):
            if typ != GMSH_TYPE:
                continue
            if shown >= sample_limit:
                break
            d = get_payload(fh, off, stored, dec)
            if d is None or len(d) < 64:
                continue
            shown += 1

            # Take the first 4 floats from offset 24
            f0 = struct.unpack_from("<f", d, 24)[0]
            f1 = struct.unpack_from("<f", d, 28)[0]
            f2 = struct.unpack_from("<f", d, 32)[0]
            f3 = struct.unpack_from("<f", d, 36)[0]

            # Search for the pattern f0,f1 in the next 256 bytes
            pattern = struct.pack("<ff", f0, f1)
            hits = []
            start = 24 + 8
            search_end = min(start + 256, len(d) - 8)
            pos = start
            while pos < search_end:
                idx = d.find(pattern, pos, search_end)
                if idx < 0:
                    break
                hits.append(idx)
                pos = idx + 4

            stride = None
            if len(hits) >= 2:
                stride = hits[1] - hits[0]

            if shown <= 5:
                print(f"  gmsh #{shown}: size={len(d)}, f0={f0:.6f}, f1={f1:.6f}, f2={f2:.6f}, f3={f3:.6f}")
                if hits:
                    print(f"    pattern hits at offsets: {hits[:8]}")
                    print(f"    inferred stride: {stride} bytes ({stride // 4 if stride else '?'} floats)")
                else:
                    print(f"    no repeating pattern found in first 256 bytes")

    # Now try a more systematic approach: check common strides
    print()
    print("  Systematic stride test (offset 24, testing strides 12-64 bytes):")
    with path.open("rb") as fh:
        sample = None
        for off, stored, dec, typ, group, inst in iterate_index(path):
            if typ != GMSH_TYPE:
                continue
            d = get_payload(fh, off, stored, dec)
            if d is None or len(d) < 256:
                continue
            sample = d
            break

    if sample:
        for stride in range(12, 65, 4):
            # Check if floats at offset 24 and offset 24+stride look similar
            # (both should be small vertex coordinates)
            ok = True
            count = 0
            off_test = 24
            while off_test + stride + 4 <= len(sample) and count < 20:
                f1 = struct.unpack_from("<f", sample, off_test)[0]
                f2 = struct.unpack_from("<f", sample, off_test + stride)[0]
                # Both should be "reasonable" vertex coords (not huge, not NaN)
                if abs(f1) > 1000 or abs(f2) > 1000:
                    ok = False
                    break
                count += 1
                off_test += stride
            n_floats = stride // 4
            print(f"    stride={stride:>3} ({n_floats} floats): {'OK' if ok else 'FAIL'}, checked {count} vertices")


def scan_for_structure(path: Path, sample_limit: int = 3):
    """Deep scan of a few gmsh resources to understand the full structure.

    Looks for:
    - Where vertex data starts and ends
    - Where index data might start (look for u16/u32 patterns)
    - Sub-mesh boundaries
    """
    with path.open("rb") as fh:
        shown = 0
        for off, stored, dec, typ, group, inst in iterate_index(path):
            if typ != GMSH_TYPE:
                continue
            if shown >= sample_limit:
                break
            d = get_payload(fh, off, stored, dec)
            if d is None or len(d) < 64:
                continue
            shown += 1

            h = parse_gmsh_header(d)
            print(f"=== gmsh #{shown}: size={len(d)}")
            print(f"  u0=0x{h['u0']:08X} u4=0x{h['u4']:08X} u8=0x{h['u8']:08X} "
                  f"u12=0x{h['u12']:08X} u16=0x{h['u16']:08X} u20=0x{h['u20']:08X}")

            # Hypothesis: u4 might be mesh count, u8 might be vertex count
            # If u8 = vertex_count and each vertex is 3 floats (12 bytes),
            # vertex_data_size = u8 * 12. Check if that lines up with something.
            for vcount_field, vcount_name in [("u4", "u4"), ("u8", "u8")]:
                vcount = h[vcount_field]
                if vcount == 0 or vcount > 100000:
                    continue
                for stride in [12, 16, 20, 24, 28, 32, 36, 40, 48]:
                    vdata_end = 24 + vcount * stride
                    if vdata_end <= len(d) and vdata_end > 24:
                        remaining = len(d) - vdata_end
                        print(f"  if {vcount_name}={vcount} is vertex_count, stride={stride}: "
                              f"vdata_end={vdata_end}, remaining={remaining} bytes")

            # Also check if u12 is a byte offset
            u12 = h["u12"]
            if 24 < u12 < len(d):
                print(f"  u12={u12} as offset: bytes at that location:")
                chunk = d[u12:u12 + min(32, len(d) - u12)]
                print(f"    {chunk.hex(' ')}")
                # Try as floats
                fvals = []
                for j in range(0, min(32, len(d) - u12), 4):
                    fvals.append(struct.unpack_from("<f", d, u12 + j)[0])
                print(f"    as floats: {['%.4f' % f for f in fvals]}")
                # Try as u32
                uvals = []
                for j in range(0, min(32, len(d) - u12), 4):
                    uvals.append(struct.unpack_from("<I", d, u12 + j)[0])
                print(f"    as u32: {['0x%08X' % u for u in uvals]}")

            # Check what's at offset u4 if it's a reasonable offset
            u4 = h["u4"]
            if 24 < u4 < len(d):
                print(f"  u4={u4} as offset: bytes at that location:")
                chunk = d[u4:u4 + min(32, len(d) - u4)]
                print(f"    {chunk.hex(' ')}")

            print()

            # Full hex dump of first 256 bytes
            print(f"  First 256 bytes hex:")
            lim = min(256, len(d))
            for j in range(0, lim, 16):
                chunk = d[j:j + 16]
                hexstr = chunk.hex(' ')
                asc = ''.join(chr(x) if 32 <= x < 127 else '.' for x in chunk)
                print(f"    {j:>5}  {hexstr:<47}  {asc}")
            print()


def main() -> int:
    if len(sys.argv) < 2:
        print("Usage: gmsh_parse.py <package> [dump|stats|stride|structure]")
        return 1
    path = Path(sys.argv[1])
    if not path.exists():
        print(f"not found: {path}")
        return 1

    mode = sys.argv[2] if len(sys.argv) >= 3 else "stats"

    if mode == "dump":
        n = int(sys.argv[3]) if len(sys.argv) >= 4 else 5
        dump_first_n(path, n=n)
    elif mode == "stats":
        statistical_summary(path)
    elif mode == "stride":
        stride_analysis(path)
    elif mode == "structure":
        scan_for_structure(path)
    else:
        print(f"unknown mode: {mode}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

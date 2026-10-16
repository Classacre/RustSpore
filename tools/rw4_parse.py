"""
Parse SPORE RW4 resources (type 0x2F4E681B).

Status: header structure partially understood. The payload after the header is
expected to contain RenderWare 4 objects, but the object dispatch is not yet
implemented.

Usage:
    python tools/rw4_parse.py "D:/Games/SPORE Collection/Data/Spore_Graphics.package"
"""
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress  # noqa: E402
import prop_parse as P  # noqa: E402

RW4_TYPE = 0x2F4E681B

# Magic signature at offset 0
RW4_MAGIC = bytes.fromhex("89 52 57 34 77 33 32 00 0d 0a 1a 0a".replace(" ", ""))


def parse_rw4_header(d: bytes) -> dict:
    """Parse what is known of the RW4 header.

    Returns a dict with the raw fields. Raises ValueError on bad magic.
    """
    if len(d) < 32:
        raise ValueError(f"too short: {len(d)} bytes")
    if d[:12] != RW4_MAGIC:
        raise ValueError(f"bad magic: {d[:12].hex()}")

    # Header fields (all little-endian based on statistical analysis)
    platform = d[16:20].decode("ascii", "replace").rstrip("\0")
    version = d[20:24].decode("ascii", "replace").rstrip("\0")

    h = {
        "size": len(d),
        "platform": platform,
        "version": version,
        "u12": struct.unpack_from("<I", d, 12)[0],
        "u24": struct.unpack_from("<I", d, 24)[0],
        "subtype": struct.unpack_from("<I", d, 28)[0],
        "u32": struct.unpack_from("<I", d, 32)[0],
        "u36": struct.unpack_from("<I", d, 36)[0],
        "u40": struct.unpack_from("<I", d, 40)[0],
        "u44": struct.unpack_from("<I", d, 44)[0],
        "off48": struct.unpack_from("<I", d, 48)[0],
        "u52": struct.unpack_from("<I", d, 52)[0],   # always 0x98 in samples
        "u56": struct.unpack_from("<I", d, 56)[0],
        "u60": struct.unpack_from("<I", d, 60)[0],
        "u64": struct.unpack_from("<I", d, 64)[0],
        "off68": struct.unpack_from("<I", d, 68)[0],
        "u72": struct.unpack_from("<I", d, 72)[0],
        "off76": struct.unpack_from("<I", d, 76)[0],
        "u80": struct.unpack_from("<I", d, 80)[0],   # always 0x04
        "u84": struct.unpack_from("<I", d, 84)[0],
        "u88": struct.unpack_from("<I", d, 88)[0],   # always 0x01
        "u92": struct.unpack_from("<I", d, 92)[0],
        "u96": struct.unpack_from("<I", d, 96)[0],   # always 0x01
        "off100": struct.unpack_from("<I", d, 100)[0],
        "u104": struct.unpack_from("<I", d, 104)[0], # always 0x04
        "u108": struct.unpack_from("<I", d, 108)[0],
        "u112": struct.unpack_from("<I", d, 112)[0], # always 0x01
        "u116": struct.unpack_from("<I", d, 116)[0],
        "u120": struct.unpack_from("<I", d, 120)[0], # always 0x01
        "u124": struct.unpack_from("<I", d, 124)[0],
    }

    # Measured invariant: footer size = file_size - off76
    # Also: off68 often equals footer_size (but not always)
    h["footer_size"] = len(d) - h["off76"]
    return h


def iterate_index(path: Path):
    """Yield (offset, stored_size, decompressed_size, type) for all resources."""
    with path.open("rb") as fh:
        head = fh.read(0x60)
        count = struct.unpack_from("<I", head, 0x24)[0]
        idx_off = struct.unpack_from("<I", head, 0x40)[0]
        fh.seek(idx_off + 4)
        blob = fh.read(count * P.REC)
        for i in range(count):
            g, t, ihi, ilo, off, sz, dec = struct.unpack_from("<IIIIIII", blob, i * P.REC)
            yield off, sz & 0x7FFFFFFF, dec, t


def summarise(path: Path, sample_limit: int = 500):
    from collections import Counter

    subtype_counts = Counter()
    sizes = Counter()
    off48_vals = Counter()
    footer_sizes = Counter()
    bad = 0
    total = 0

    with path.open("rb") as fh:
        for off, stored, dec, typ in iterate_index(path):
            if typ != RW4_TYPE:
                continue
            if total >= sample_limit:
                break
            fh.seek(off)
            raw = fh.read(stored)
            try:
                d = decompress(raw, out_size=None, header=True)[0] if dec > stored else raw
            except Exception:
                continue
            if len(d) < 32:
                continue
            total += 1
            try:
                h = parse_rw4_header(d)
            except ValueError as e:
                bad += 1
                continue
            subtype_counts[h["subtype"]] += 1
            sizes[len(d)] += 1
            off48_vals[h["off48"]] += 1
            footer_sizes[h["footer_size"]] += 1

    print(f"RW4 resources: {total} sampled (limit {sample_limit}), {bad} bad magic")
    print(f"Subtype distribution:")
    for st, n in subtype_counts.most_common():
        print(f"  0x{st:08X}: {n}")
    print(f"Size distribution (top 8): {sizes.most_common(8)}")
    print(f"Footer size distribution (top 8): {footer_sizes.most_common(8)}")
    print(f"off48 distribution (top 8): {off48_vals.most_common(8)}")


def dump_one(path: Path, subtype: int = None):
    """Dump the header of the first matching resource."""
    with path.open("rb") as fh:
        for off, stored, dec, typ in iterate_index(path):
            if typ != RW4_TYPE:
                continue
            fh.seek(off)
            raw = fh.read(stored)
            try:
                d = decompress(raw, out_size=None, header=True)[0] if dec > stored else raw
            except Exception:
                continue
            if len(d) < 128:
                continue
            h = parse_rw4_header(d)
            if subtype is not None and h["subtype"] != subtype:
                continue
            print(f"=== RW4 resource, size={len(d)}, subtype=0x{h['subtype']:08X}")
            for k, v in h.items():
                if k == "size":
                    continue
                if isinstance(v, int):
                    print(f"  {k:>8} = 0x{v:08X} ({v})")
                else:
                    print(f"  {k:>8} = {v!r}")
            print()
            print(f"  First 128 bytes:")
            for j in range(0, 128, 16):
                c = d[j:j + 16]
                print(
                    f"    {j:>4}  {c.hex(' '):<47} "
                    f"{''.join(chr(x) if 32 <= x < 127 else '.' for x in c)}"
                )
            return
    print("No matching resource found.")


def main() -> int:
    if len(sys.argv) < 2:
        print("Usage: rw4_parse.py <package> [dump <subtype>]")
        return 1
    path = Path(sys.argv[1])
    if not path.exists():
        print(f"not found: {path}")
        return 1

    if len(sys.argv) >= 3 and sys.argv[2] == "dump":
        subtype = int(sys.argv[3], 0) if len(sys.argv) >= 4 else None
        dump_one(path, subtype)
    else:
        summarise(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

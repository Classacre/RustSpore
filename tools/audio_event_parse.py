#!/usr/bin/env python3
"""Parse and validate type 0x02B9F662 (audio event configuration table).

Structure (all big-endian):
- BE u32: entry count
- N entries of 12 bytes: FNV-1 hash(4) + u16 type(2) + u16 flags(2) + value(4)
- "samples" section: hash=0x701ED91E + u16 + u16 + count + stride + pairs
- "codec" section: hash=0xBA03D0CD + u16 + u16 + count

Known FNV-1 hashes:
  0x5F6317D5 = "parent"
  0x25DF0108 = "gain"
  0x12D2A4D4 = "is3d"
  0x701ED91E = "samples"
  0xBA03D0CD = "codec"
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x02B9F662

KNOWN_HASHES = {
    0x5F6317D5: "parent",
    0x25DF0108: "gain",
    0x12D2A4D4: "is3d",
    0x701ED91E: "samples",
    0xBA03D0CD: "codec",
}


def fnv1_32(s: str) -> int:
    """FNV-1 32-bit hash."""
    h = 0x811C9DC5
    for b in s.encode("utf-8"):
        h = (h * 0x01000193) & 0xFFFFFFFF
        h ^= b
    return h


def hash_name(h: int) -> str:
    """Return the known string for a hash, or hex if unknown."""
    return KNOWN_HASHES.get(h, f"0x{h:08X}")


def parse(d: bytes):
    """Parse a 0x02B9F662 payload. Returns a dict or None on failure."""
    if len(d) < 4:
        return None

    count = struct.unpack_from(">I", d, 0)[0]
    if count > 100:
        return None

    off = 4
    entries = []
    for _ in range(count):
        if off + 12 > len(d):
            return None
        h = struct.unpack_from(">I", d, off)[0]
        u16a = struct.unpack_from(">H", d, off + 4)[0]
        u16b = struct.unpack_from(">H", d, off + 6)[0]
        val = struct.unpack_from(">I", d, off + 8)[0]
        entries.append({
            "hash": h,
            "name": hash_name(h),
            "type": u16a,
            "flags": u16b,
            "value": val,
            "float": struct.unpack_from(">f", d, off + 8)[0],
        })
        off += 12

    # Look for "samples" section
    samples_section = None
    if off + 8 <= len(d):
        h = struct.unpack_from(">I", d, off)[0]
        if h == 0x701ED91E:  # "samples"
            u16a = struct.unpack_from(">H", d, off + 4)[0]
            u16b = struct.unpack_from(">H", d, off + 6)[0]
            off += 8
            if off + 8 <= len(d):
                pair_count = struct.unpack_from(">I", d, off)[0]
                stride = struct.unpack_from(">I", d, off + 4)[0]
                off += 8
                pairs = []
                for _ in range(min(pair_count, 50)):
                    if off + 12 > len(d):
                        break
                    ph = struct.unpack_from(">I", d, off)[0]
                    pairs.append(ph)
                    off += 12
                samples_section = {
                    "type": u16a,
                    "flags": u16b,
                    "pair_count": pair_count,
                    "stride": stride,
                    "pairs": pairs,
                }

    # Look for "codec" section
    codec_section = None
    if off + 8 <= len(d):
        h = struct.unpack_from(">I", d, off)[0]
        if h == 0xBA03D0CD:  # "codec"
            u16a = struct.unpack_from(">H", d, off + 4)[0]
            u16b = struct.unpack_from(">H", d, off + 6)[0]
            off += 8
            if off + 4 <= len(d):
                codec_count = struct.unpack_from(">I", d, off)[0]
                off += 4
                codec_section = {
                    "type": u16a,
                    "flags": u16b,
                    "count": codec_count,
                }

    return {
        "entry_count": count,
        "entries": entries,
        "samples": samples_section,
        "codec": codec_section,
        "end_offset": off,
        "total_size": len(d),
        "exact_fit": off == len(d),
    }


def validate(samples):
    """Validate the parser across all samples."""
    results = Counter()
    hash_freq = Counter()
    type_freq = Counter()
    exact_fits = 0
    total = 0

    for d in samples:
        parsed = parse(d)
        total += 1
        if parsed is None:
            results["parse_failed"] += 1
            continue

        if parsed["exact_fit"]:
            exact_fits += 1
            results["exact_fit"] += 1
        else:
            results["inexact"] += 1

        for e in parsed["entries"]:
            hash_freq[e["hash"]] += 1
            type_freq[e["type"]] += 1

        if parsed["samples"]:
            results["has_samples_section"] += 1
        if parsed["codec"]:
            results["has_codec_section"] += 1

    print(f"Total samples: {total}")
    print(f"Exact fits: {exact_fits} ({100*exact_fits/total:.1f}%)")
    print()

    print("Entry hash frequency:")
    for h, c in hash_freq.most_common(20):
        print(f"  {hash_name(h):>20s}: {c}")
    print()

    print("Entry type frequency:")
    for t, c in type_freq.most_common(10):
        print(f"  {t}: {c}")
    print()

    for k, v in results.most_common():
        print(f"  {k}: {v}")


def main():
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

    print(f"Collected {len(samples)} samples\n")
    validate(samples)

    # Show detailed parse of first few samples
    print("\n" + "=" * 70)
    print("Detailed parse of first 5 samples:")
    print("=" * 70)
    for i, d in enumerate(samples[:5]):
        parsed = parse(d)
        if parsed is None:
            print(f"\nSample {i+1}: PARSE FAILED")
            continue
        print(f"\nSample {i+1} (size={parsed['total_size']}, exact_fit={parsed['exact_fit']}):")
        for e in parsed["entries"]:
            val_str = f"{e['value']} (0x{e['value']:08X})"
            if e["type"] in (13, 1):
                val_str += f" float={e['float']:.4f}"
            print(f"  {e['name']:>20s} type={e['type']:>3d} flags={e['flags']} value={val_str}")
        if parsed["samples"]:
            s = parsed["samples"]
            print(f"  [samples] type={s['type']} flags={s['flags']} count={s['pair_count']} stride={s['stride']}")
            for p in s["pairs"][:5]:
                print(f"    pair hash: 0x{p:08X}")
        if parsed["codec"]:
            c = parsed["codec"]
            print(f"  [codec] type={c['type']} flags={c['flags']} count={c['count']}")


if __name__ == "__main__":
    main()

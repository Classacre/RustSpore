#!/usr/bin/env python3
"""Investigate variable-size entry structure in type 0x02B9F662.

Hypothesis: entries with type=0 are 8 bytes (no value), others are 12 bytes.
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


def hash_name(h):
    return KNOWN_HASHES.get(h, f"0x{h:08X}")


def parse_variable(d):
    """Parse with variable-size entries based on type."""
    if len(d) < 4:
        return None

    count = struct.unpack_from(">I", d, 0)[0]
    if count > 100:
        return None

    off = 4
    entries = []
    for _ in range(count):
        if off + 8 > len(d):
            return None
        h = struct.unpack_from(">I", d, off)[0]
        typ = struct.unpack_from(">H", d, off + 4)[0]
        flags = struct.unpack_from(">H", d, off + 6)[0]

        entry = {"hash": h, "name": hash_name(h), "type": typ, "flags": flags}
        off += 8

        # Types that have a value field
        if typ != 0:
            if off + 4 > len(d):
                return None
            entry["value"] = struct.unpack_from(">I", d, off)[0]
            entry["float"] = struct.unpack_from(">f", d, off)[0]
            off += 4

        entries.append(entry)

    # Look for "samples" section
    samples_section = None
    if off + 8 <= len(d):
        h = struct.unpack_from(">I", d, off)[0]
        if h == 0x701ED91E:
            typ = struct.unpack_from(">H", d, off + 4)[0]
            flags = struct.unpack_from(">H", d, off + 6)[0]
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
                    "type": typ, "flags": flags,
                    "pair_count": pair_count, "stride": stride,
                    "pairs": pairs,
                }

    # Look for "codec" section
    codec_section = None
    if off + 8 <= len(d):
        h = struct.unpack_from(">I", d, off)[0]
        if h == 0xBA03D0CD:
            typ = struct.unpack_from(">H", d, off + 4)[0]
            flags = struct.unpack_from(">H", d, off + 6)[0]
            off += 8
            if off + 4 <= len(d):
                codec_count = struct.unpack_from(">I", d, off)[0]
                off += 4
                codec_section = {
                    "type": typ, "flags": flags, "count": codec_count,
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
                if len(samples) < 1000:
                    samples.append(raw)

    print(f"Testing {len(samples)} samples with variable-size entries\n")

    exact = 0
    hash_freq = Counter()
    type_freq = Counter()
    type_sizes = {}

    for d in samples:
        parsed = parse_variable(d)
        if parsed is None:
            continue
        if parsed["exact_fit"]:
            exact += 1
        for e in parsed["entries"]:
            hash_freq[e["hash"]] += 1
            type_freq[e["type"]] += 1
            t = e["type"]
            if t not in type_sizes:
                type_sizes[t] = set()
            # Track entry size
            size = 8 if t == 0 else 12
            type_sizes[t].add(size)

    print(f"Exact fits: {exact}/{len(samples)} ({100*exact/len(samples):.1f}%)\n")

    print("Entry hash frequency (top 15):")
    for h, c in hash_freq.most_common(15):
        print(f"  {hash_name(h):>20s}: {c}")
    print()

    print("Entry type frequency:")
    for t, c in type_freq.most_common(15):
        sizes = type_sizes.get(t, set())
        print(f"  type={t:>6d}: {c:>6d}  sizes={sizes}")
    print()

    # Show detailed parse of a few exact-fit samples
    print("Exact-fit sample details:")
    shown = 0
    for d in samples:
        parsed = parse_variable(d)
        if parsed and parsed["exact_fit"] and shown < 5:
            print(f"\n  size={parsed['total_size']}, entries={parsed['entry_count']}")
            for e in parsed["entries"]:
                val = e.get("value", "-")
                print(f"    {e['name']:>20s} type={e['type']:>3d} flags={e['flags']:>3d} value={val}")
            if parsed["samples"]:
                s = parsed["samples"]
                print(f"    [samples] count={s['pair_count']} stride={s['stride']} pairs={len(s['pairs'])}")
            if parsed["codec"]:
                c = parsed["codec"]
                print(f"    [codec] count={c['count']}")
            shown += 1


if __name__ == "__main__":
    main()

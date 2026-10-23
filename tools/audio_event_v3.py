#!/usr/bin/env python3
"""Discovery analyzer for type 0x02B9F662 (audio event).

Strategy: parse the leading `entry_count` * 12-byte simple property entries, then
fingerprint the trailing remainder: the hash that begins it, and the remainder size
distribution. This reveals the additional section types beyond `samples`/`codec`.
"""

import struct
import sys
from pathlib import Path
from collections import Counter, defaultdict

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x02B9F662

KNOWN = {
    0x5F6317D5: "parent",
    0x25DF0108: "gain",
    0x12D2A4D4: "is3d",
    0x701ED91E: "samples",
    0xBA03D0CD: "codec",
}


def collect():
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
    return samples


def main():
    samples = collect()
    print(f"Total samples: {len(samples)}")

    exact = 0
    remainder_size_hist = Counter()
    # leading-hash of remainder -> list of remainder sizes
    lead_hash_sizes = defaultdict(list)
    # leading-hash of remainder -> count of samples
    lead_hash_count = Counter()
    # (lead_hash, remainder_size) -> 1 (to find exact combos)
    combos = Counter()

    for d in samples:
        if len(d) < 4:
            continue
        count = struct.unpack_from(">I", d, 0)[0]
        if count > 100:
            remainder_size_hist["OVERCOUNT"] += 1
            continue
        off = 4 + count * 12
        if off > len(d):
            remainder_size_hist["TRUNC"] += 1
            continue
        rem = d[off:]
        if off == len(d):
            exact += 1
            continue
        # leading section hash
        lh = struct.unpack_from(">I", d, off)[0]
        lead_hash_sizes[lh].append(len(rem))
        lead_hash_count[lh] += 1
        combos[(hex(lh), len(rem))] += 1
        remainder_size_hist[len(rem)] += 1

    print(f"Exact (no remainder): {exact} ({100*exact/len(samples):.2f}%)")
    print()
    print("Remainder size histogram (top 20):")
    for sz, c in remainder_size_hist.most_common(20):
        print(f"  {sz}: {c}")
    print()
    print("Leading remainder hash (top 25):")
    for h, c in lead_hash_count.most_common(25):
        sizes = sorted(set(lead_hash_sizes[h]))
        print(f"  0x{h:08X} {KNOWN.get(h,'?'):>10s}: {c}  sizes={sizes[:8]}")


if __name__ == "__main__":
    main()

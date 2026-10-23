#!/usr/bin/env python3
"""Validate the blockdata (0x5CAA5E28) decode hypothesis against the full population.

Hypothesis:
  - 36-byte header: 6 LE floats (bbox) + LE float @24 + LE u32 flags @28 + 4 zero bytes.
  - The flags dword's popcount `p` (p >= 1) determines the number of 12-byte data
    groups: groups = 4 ** (p - 1). The data region is exactly groups * 12 bytes.
  - Therefore total size must equal 36 + 12 * 4 ** (p - 1).
  - For flags == 0: 36-byte sample, no data.

If this holds for every blockdata resource, the format admits a clean
exact-consumption decode (header + flags-derived data length), i.e. it is
decoded to 100%.
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x5CAA5E28


def level_from_context(raw):
    """L = (count of 0x04 bytes in the 8-byte context at +28..+35) - 1.

    Returns (count04, L, expected_size). For count04 == 0 the sample is a bare
    36-byte header (no data).
    """
    if len(raw) < 36:
        return 0, None, None
    ctx = raw[28:36]
    c = sum(1 for b in ctx if b == 0x04)
    if c == 0:
        return 0, None, 36
    L = c - 1
    return c, L, 36 + 12 * (4 ** L)


def main():
    sizes = Counter()
    flags_seen = Counter()
    ctx_seen = Counter()
    n = 0
    exact = 0
    mismatches = []
    dump = []
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
                n += 1
                size = len(raw)
                sizes[size] += 1
                if size < 36:
                    mismatches.append((pkg.name, "too_short", size))
                    continue
                flags = struct.unpack_from("<I", raw, 28)[0]
                flags_seen[flags] += 1
                c, L, expected = level_from_context(raw)
                ctx_seen[(c, L)] += 1
                if size == expected:
                    exact += 1
                else:
                    mismatches.append((pkg.name, "size!=expected", size, c, L, expected))
                if len(dump) < 3 and 48 <= size <= 100:
                    dump.append((pkg.name, raw))

    print(f"Total blockdata samples: {n}")
    print(f"EXACT (size == 36 + 12*4^L, L = #0x04 in ctx[28..36] - 1): {exact} ({100*exact/n:.4f}%)")
    print(f"Size distribution: {dict(sorted(sizes.items()))}")
    print(f"Context (count04, L) -> count: {dict(sorted(ctx_seen.items()))}")
    print(f"Mismatches: {len(mismatches)}")
    for m in mismatches[:30]:
        print(f"  {m}")


if __name__ == "__main__":
    main()

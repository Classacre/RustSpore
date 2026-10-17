#!/usr/bin/env python3
"""Investigate gmdl format (type 0x00E6BCE5, 4,209 resources).

From SporeModder Blender Addon source (GPL-3.0, documented not copied):
- GMDL has version, referenced files, mesh count, bounding box,
  triangle buffers, vertex formats, vertex buffers, mesh-to-buffer mappings
- Version is LE u32, expected value 9

First bytes: 08 00 00 00 00 00 00 05 64 02 8f e0 41 60 61 00
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x00E6BCE5


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
                if len(samples) < 5:
                    samples.append(raw)

    print(f"Collected {len(samples)} samples\n")

    for i, d in enumerate(samples[:5]):
        print(f"--- Sample {i+1} (size={len(d)}) ---")
        # Hex dump first 128 bytes
        for row in range(0, min(128, len(d)), 16):
            hex_str = " ".join(f"{b:02x}" for b in d[row:row+16])
            ascii_str = "".join(chr(b) if 32 <= b < 127 else "." for b in d[row:row+16])
            print(f"  +{row:04x}: {hex_str:<48s} {ascii_str}")

        # Parse fields
        if len(d) >= 16:
            ver_le = struct.unpack_from("<I", d, 0)[0]
            ver_be = struct.unpack_from(">I", d, 0)[0]
            u4_le = struct.unpack_from("<I", d, 4)[0]
            u4_be = struct.unpack_from(">I", d, 4)[0]
            u8_le = struct.unpack_from("<I", d, 8)[0]
            u8_be = struct.unpack_from(">I", d, 8)[0]
            f12_le = struct.unpack_from("<f", d, 12)[0]
            f12_be = struct.unpack_from(">f", d, 12)[0]
            print(f"  LE u32@0={ver_le}, BE u32@0={ver_be}")
            print(f"  LE u32@4={u4_le}, BE u32@4={u4_be}")
            print(f"  LE u32@8={u8_le}, BE u32@8={u8_be}")
            print(f"  LE float@12={f12_le:.6f}, BE float@12={f12_be:.6f}")

            # Check for ASCII strings in first 200 bytes
            strings = []
            for off in range(0, min(200, len(d))):
                if 32 <= d[off] < 127:
                    s = ""
                    for j in range(off, min(off + 50, len(d))):
                        if 32 <= d[j] < 127:
                            s += chr(d[j])
                        else:
                            break
                    if len(s) >= 4:
                        strings.append((off, s))
            if strings:
                print(f"  Strings found:")
                for off, s in strings[:5]:
                    print(f"    +{off}: \"{s}\"")
        print()

    # Statistical analysis
    print("=" * 70)
    print("Statistical analysis:")
    print("=" * 70)
    ver_dist = Counter()
    u4_dist = Counter()
    u8_dist = Counter()

    for d in samples:
        if len(d) < 16:
            continue
        ver_le = struct.unpack_from("<I", d, 0)[0]
        ver_be = struct.unpack_from(">I", d, 0)[0]
        u4_le = struct.unpack_from("<I", d, 4)[0]
        u8_le = struct.unpack_from("<I", d, 8)[0]
        ver_dist[ver_le] += 1
        u4_dist[u4_le] += 1
        u8_dist[u8_le] += 1

    print("LE u32@0 (version?):", dict(ver_dist.most_common(10)))
    print("LE u32@4:", dict(u4_dist.most_common(10)))
    print("LE u32@8:", dict(u8_dist.most_common(10)))


if __name__ == "__main__":
    main()

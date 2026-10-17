#!/usr/bin/env python3
"""Investigate type 0x02B9F662 (16,841 resources).

First bytes: 00 00 00 04 25 df 01 08 00 0d 00 00 3e b3 33 33
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
                if len(samples) < 20:
                    samples.append(raw)

    print(f"Collected {len(samples)} samples")
    print()

    for i, d in enumerate(samples):
        print(f"--- Sample {i+1} (size={len(d)}) ---")
        # Hex dump first 64 bytes
        for row in range(0, min(64, len(d)), 16):
            hex_str = " ".join(f"{b:02x}" for b in d[row:row+16])
            ascii_str = "".join(chr(b) if 32 <= b < 127 else "." for b in d[row:row+16])
            print(f"  +{row:04x}: {hex_str:<48s} {ascii_str}")

        # Parse fields
        if len(d) >= 16:
            u0 = struct.unpack_from(">I", d, 0)[0]
            u4 = struct.unpack_from(">I", d, 4)[0]
            u8 = struct.unpack_from(">H", d, 8)[0]
            u10 = struct.unpack_from(">H", d, 10)[0]
            f12_be = struct.unpack_from(">f", d, 12)[0]
            f12_le = struct.unpack_from("<f", d, 12)[0]
            print(f"  BE u32@0={u0}, BE u32@4=0x{u4:08X}, BE u16@8={u8}, BE u16@10={u10}")
            print(f"  BE float@12={f12_be:.6f}, LE float@12={f12_le:.6f}")

            # Check if u4 correlates with type 0x25DF0112
            if u4 == 0x25DF0108 or (u4 >> 16) == 0x25DF:
                print(f"  *** u4 matches GAIT type prefix (0x25DF) ***")
        print()


if __name__ == "__main__":
    main()

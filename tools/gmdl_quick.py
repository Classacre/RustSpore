#!/usr/bin/env python3
"""Quick investigation of gmdl format (type 0x00E6BCE5).

Only reads first 256 bytes of each sample to avoid slow decompression.
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
    # Only look in Spore_Content.package where gmdl lives
    pkg = BASE / "Spore_Content.package"
    entries, _ = parse_index(pkg)

    samples = []
    count = 0
    with pkg.open("rb") as fh:
        for e in entries:
            if e["type"] != TARGET:
                continue
            count += 1
            if len(samples) >= 10:
                break
            fh.seek(e["offset"])
            # Only read first 512 bytes of stored data
            raw = fh.read(min(e["stored"], 512))
            if e["dec"] > e["stored"]:
                try:
                    # Decompress but only keep first 512 bytes
                    result = decompress(raw, out_size=None, header=True)
                    if result and result[0]:
                        raw = result[0][:512]
                except Exception:
                    continue
            samples.append(raw)

    print(f"Total gmdl in Spore_Content: {count}")
    print(f"Collected {len(samples)} samples (first 512 bytes each)\n")

    for i, d in enumerate(samples):
        print(f"--- Sample {i+1} (first {len(d)} bytes) ---")
        for row in range(0, min(128, len(d)), 16):
            hex_str = " ".join(f"{b:02x}" for b in d[row:row+16])
            ascii_str = "".join(chr(b) if 32 <= b < 127 else "." for b in d[row:row+16])
            print(f"  +{row:04x}: {hex_str:<48s} {ascii_str}")

        if len(d) >= 16:
            ver_le = struct.unpack_from("<I", d, 0)[0]
            u4_le = struct.unpack_from("<I", d, 4)[0]
            u8_le = struct.unpack_from("<I", d, 8)[0]
            u12_le = struct.unpack_from("<I", d, 12)[0]
            print(f"  LE: ver={ver_le}, u4={u4_le}, u8={u8_le}, u12=0x{u12_le:08X}")

            # Look for ASCII strings
            strings = []
            for off in range(0, min(256, len(d))):
                if 32 <= d[off] < 127:
                    s = ""
                    for j in range(off, min(off + 60, len(d))):
                        if 32 <= d[j] < 127:
                            s += chr(d[j])
                        else:
                            break
                    if len(s) >= 4:
                        strings.append((off, s))
                        off += len(s)
            if strings:
                print(f"  Strings:")
                for off, s in strings[:5]:
                    print(f"    +{off}: \"{s}\"")
        print()


if __name__ == "__main__":
    main()

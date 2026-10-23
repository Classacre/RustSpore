#!/usr/bin/env python3
"""Decode the gmdl vertex section and dump the trailer that follows the vertex data.

Measured so far (clean samples, count=1):
    header 64 bytes
    index buffer  indices_count * 2  (u16 LE)
    vertex section header 56 bytes:
        +0  u32 = 3        (descriptor field)
        +4  u32 = 0
        +8  u32 = 2
        +12 u32 = 0
        +16..+47  descriptor bytes
        +48 u32 vertex_count
        +52 u32 vertex_data_size   (= vertex_count * stride, stride = 24)
    vertex data  vertex_data_size bytes (LE floats: position XYZ + normal XYZ)

This tool dumps [vertex_data_end : EOF] to reverse-engineer the trailer.
"""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x00E6BCE5


def main():
    pkg = BASE / "Spore_Content.package"
    entries, _ = parse_index(pkg)
    collected = []
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
            # clean samples only
            if len(raw) < 64:
                continue
            version = struct.unpack_from("<I", raw, 0)[0]
            count = struct.unpack_from("<I", raw, 8)[0]
            idx_count = struct.unpack_from("<I", raw, 48)[0]
            if version != 8 or count != 1 or idx_count == 0:
                continue
            idx_end = 64 + idx_count * 2
            if idx_end + 56 > len(raw):
                continue
            vcount = struct.unpack_from("<I", raw, idx_end + 48)[0]
            vsize = struct.unpack_from("<I", raw, idx_end + 52)[0]
            if vcount == 0 or vsize != vcount * 24:
                continue
            collected.append((raw, idx_end, vcount, vsize))
            if len(collected) >= 4:
                break

    for raw, idx_end, vcount, vsize in collected:
        vdata_end = idx_end + 56 + vsize
        print(f"=== total={len(raw)} idx_end={idx_end} vcount={vcount} vsize={vsize} "
              f"vdata_end={vdata_end} trailer={len(raw)-vdata_end}")
        # Dump the 48-byte vertex descriptor.
        print("  vertex descriptor (48 bytes):")
        for row in range(idx_end, idx_end + 48, 16):
            chunk = raw[row:row + 16]
            print(f"    +{row-idx_end:02x}: " + " ".join(f"{b:02x}" for b in chunk))
        # Dump the trailer.
        print("  trailer:")
        for row in range(vdata_end, min(vdata_end + 160, len(raw)), 16):
            chunk = raw[row:row + 16]
            hs = " ".join(f"{b:02x}" for b in chunk)
            asc = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
            print(f"    +{row-vdata_end:04x}: {hs:<47} {asc}")
        # Interpret the trailer head as u32s.
        print("  trailer head as u32/float:")
        for j in range(0, min(64, len(raw) - vdata_end), 4):
            v = struct.unpack_from("<I", raw, vdata_end + j)[0]
            f = struct.unpack_from("<f", raw, vdata_end + j)[0]
            print(f"    +{j:02x}: u32={v:<12} float={f:.4f}")
        print()


if __name__ == "__main__":
    main()

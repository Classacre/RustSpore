#!/usr/bin/env python3
"""Deep investigation of gmdl (0x00E6BCE5) after the index buffer.

Known so far (re/payload-formats.md):
    offset  0: LE u32 version = 8
    offset  4: LE u32 flags = 0
    offset  8: LE u32 count (1 or 2)
    offset 12: 7 LE floats (bbox min/max/radius)  -> 12..40
    offset 40: LE u32 triangle_buffer_count = 1
    offset 44: LE u32 primitive_type = 4
    offset 48: LE u32 indices_count
    offset 52: LE u32 indices_bits = 16
    offset 56: LE u32 buffer_size (= indices_count*2)
    offset 60: LE u32 = 0
    offset 64: index data (u16 LE), indices_count * 2 bytes

This tool dumps the region immediately after the index buffer and the file tail,
to reverse-engineer the vertex format + vertex buffer layout.
"""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x00E6BCE5


def hexdump(d, start, length, label=""):
    end = min(start + length, len(d))
    print(f"{label} bytes [{start:08x}..{end:08x}] (len {len(d)}):")
    for row in range(start, end, 16):
        chunk = d[row:row + 16]
        hs = " ".join(f"{b:02x}" for b in chunk)
        ax = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
        print(f"  +{row:08x}: {hs:<48s} {ax}")


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
            collected.append(raw)
            if len(collected) >= 6:
                break

    print(f"Collected {len(collected)} gmdl samples\n")
    for i, d in enumerate(collected):
        assert len(d) >= 64
        version = struct.unpack_from("<I", d, 0)[0]
        flags = struct.unpack_from("<I", d, 4)[0]
        count = struct.unpack_from("<I", d, 8)[0]
        tb_count = struct.unpack_from("<I", d, 40)[0]
        prim = struct.unpack_from("<I", d, 44)[0]
        idx_count = struct.unpack_from("<I", d, 48)[0]
        idx_bits = struct.unpack_from("<I", d, 52)[0]
        buf_size = struct.unpack_from("<I", d, 56)[0]
        idx_end = 64 + idx_count * 2
        print(f"=== Sample {i+1}: total={len(d)} ver={version} flags={flags} count={count}")
        print(f"    tb_count={tb_count} prim={prim} idx_count={idx_count} idx_bits={idx_bits} buf_size={buf_size} idx_end={idx_end}")
        if idx_end + 64 > len(d):
            print(f"    WARNING: index buffer claims to run past EOF (idx_end={idx_end} > len={len(d)})")
        # Dump region right after index buffer.
        hexdump(d, idx_end, 192, label="AFTER-INDEX")
        # Dump last 80 bytes.
        hexdump(d, max(0, len(d) - 80), 80, label="TAIL")
        print()


if __name__ == "__main__":
    main()

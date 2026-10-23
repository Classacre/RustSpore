#!/usr/bin/env python3
"""Dump one representative blockdata sample per distinct size class."""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x5CAA5E28


def popcount32(x):
    return bin(x).count("1")


def main():
    seen = {}
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
                size = len(raw)
                if size in seen:
                    continue
                seen[size] = (pkg.name, raw)
                if len(seen) >= 8:
                    break
        if len(seen) >= 8:
            break

    for size in sorted(seen):
        name, raw = seen[size]
        flags = struct.unpack_from("<I", raw, 28)[0] if size >= 32 else 0
        k = (size - 36) // 12 if (size - 36) % 12 == 0 else None
        print(f"=== size={size} pkg={name} flags=0x{flags:08X} popcount={popcount32(flags)} k={k}")
        for row in range(0, min(40, size), 16):
            chunk = raw[row:row + 16]
            hs = " ".join(f"{b:02x}" for b in chunk)
            asc = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
            print(f"    +{row:04x}: {hs:<47} {asc}")
        if k is not None and k > 0:
            off = 36
            fv = struct.unpack_from("<fff", raw, off)
            print(f"    first group @36: {fv}")
        print()

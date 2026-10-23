#!/usr/bin/env python3
"""Print the same raster digest as `rustspore --raster-digest`.

Format: `<instance:016X> <width> <height> <format:08X> <rgba_fnv1a:016X>`.

Diffing the two outputs is a byte-exact check of the Rust DXT decoder against the
Python one, which is the implementation that produced the extracted PNGs.

Usage:
    python tools/raster_digest.py "<package>" > py.txt
    rustspore --raster-digest "<package>" 2>/dev/null > rs.txt
    diff py.txt rs.txt && echo identical
"""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress
from raster_extract import (
    FMT_DXT1,
    FMT_DXT5,
    FMT_RAW,
    decode_dxt,
    decode_raw_rgba,
    parse_raster_header,
)

RASTER_TYPE = 0x2F4E681C


def fnv1a64(data: bytes) -> int:
    h = 0xCBF29CE484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    pkg = Path(sys.argv[1])

    entries, _ = parse_index(pkg)
    ok = 0
    failed = 0
    with pkg.open("rb") as fh:
        for e in entries:
            if e["type"] != RASTER_TYPE:
                continue
            fh.seek(e["offset"])
            raw = fh.read(e["stored"])
            if e["dec"] > e["stored"]:
                try:
                    raw = decompress(raw, out_size=None, header=True)[0]
                except Exception as ex:
                    print(f"read 0x{e['instance']:016X}: {ex}", file=sys.stderr)
                    failed += 1
                    continue
            try:
                hdr = parse_raster_header(raw)
                w, h, fmt = hdr["width"], hdr["height"], hdr["format"]
                tex = raw[28:]
                if fmt == FMT_DXT5:
                    rgba = decode_dxt(tex, w, h, FMT_DXT5)
                elif fmt == FMT_DXT1:
                    rgba = decode_dxt(tex, w, h, FMT_DXT1)
                elif fmt == FMT_RAW:
                    rgba = decode_raw_rgba(tex, w, h)
                else:
                    raise ValueError(f"unknown format 0x{fmt:08X}")
            except Exception as ex:
                print(f"decode 0x{e['instance']:016X}: {ex}", file=sys.stderr)
                failed += 1
                continue
            print(f"{e['instance']:016X} {w:>5} {h:>5} {fmt:08X} {fnv1a64(rgba):016X}")
            ok += 1

    print(f"decoded {ok}, failed {failed}", file=sys.stderr)
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())

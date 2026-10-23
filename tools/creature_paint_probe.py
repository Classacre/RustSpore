#!/usr/bin/env python3
"""Probe the paint parameters inside `.creaturedata` (type 0x0F43029A).

The decompressed header buffer is 0x44 bytes. Nine floats at offset 32 are the three
paint colours. This tool asks what the other 32 bytes are, by dumping them across every
creature in Spore_Content and looking at how many distinct values each field takes.

A field that takes a small number of values is a candidate paint parameter (pattern
index, coat style). A field that is always the same is a constant. A field that varies
continuously is a float with no obvious enum meaning.

Usage:
    python creature_paint_probe.py [--package PATH] [--header] [--data3] [--json OUT]
"""

import argparse
import json
import struct
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index, payload  # noqa: E402

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x0F43029A
HEADER = 0xABB455B7
UNCOMPRESSED_LEN = 0x80 - 0x3C  # 0x44


def decode_buffer(d, p, length):
    """Decode the RLE chunk buffer. Returns (bytes, new_offset)."""
    n = len(d)
    dst = bytearray()
    last = 0
    while len(dst) < length:
        if p + 2 > n:
            return None, p
        size = struct.unpack_from("<H", d, p)[0]
        p += 2
        if size < 1 or size > 0x800 or p + size > n:
            return None, p
        chunk = d[p:p + size]
        p += size
        i = 0
        while i < len(chunk) and len(dst) < length:
            b = chunk[i]
            i += 1
            if b != 0xFE:
                dst.append(b)
                last = b
                continue
            if i >= len(chunk):
                break
            count = chunk[i]
            i += 1
            if count == 0:
                dst.append(0xFE)
                last = 0xFE
            else:
                for _ in range(count):
                    if len(dst) >= length:
                        break
                    dst.append(last)
        last = chunk[0]
    return bytes(dst), p


def parse(d):
    if len(d) < 8 or struct.unpack_from("<I", d, 0)[0] != HEADER:
        return None
    version = struct.unpack_from("<i", d, 4)[0]
    if version not in (9, 10):
        return None
    cc = 3 if version < 10 else 4
    p = 8
    counts = list(struct.unpack_from("<" + "i" * cc, d, p))
    p += 4 * cc
    head, p = decode_buffer(d, p, UNCOMPRESSED_LEN)
    if head is None:
        return None
    return {"version": version, "counts": counts, "head": head, "offset_after_head": p}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--package", default=str(BASE / "Spore_Content.package"))
    ap.add_argument("--json", default=None)
    ap.add_argument("--limit", type=int, default=0)
    args = ap.parse_args()

    path = Path(args.package)
    entries, _size = parse_index(path)
    rows = [e for e in entries if e["type"] == TARGET]
    if args.limit:
        rows = rows[: args.limit]
    print(f"{len(rows)} creaturedata resources in {path.name}")

    words = defaultdict(Counter)   # offset -> value counts
    floats = defaultdict(Counter)
    versions = Counter()
    palettes = []
    records = []

    for e in rows:
        raw = payload(path, e)
        parsed = parse(raw)
        if parsed is None:
            continue
        versions[parsed["version"]] += 1
        h = parsed["head"]
        for off in range(0, 32, 4):
            words[off][struct.unpack_from("<I", h, off)[0]] += 1
        for off in range(0, 32, 4):
            v = struct.unpack_from("<f", h, off)[0]
            floats[off][round(v, 4)] += 1
        pal = struct.unpack_from("<9f", h, 32)
        palettes.append(pal)
        records.append({
            "instance": f"{e['instance']:016X}",
            "version": parsed["version"],
            "counts": parsed["counts"],
            "head": h.hex(),
            "palette": [round(v, 5) for v in pal],
        })

    print(f"versions: {dict(versions)}")
    print()
    print("fields 0..32 as u32 (distinct values, top 6):")
    for off in range(0, 32, 4):
        c = words[off]
        top = ", ".join(f"{v:#010X}x{n}" for v, n in c.most_common(6))
        print(f"  +{off:02d} u32 distinct={len(c):5d}  {top}")
    print()
    print("fields 0..32 as f32 (distinct values, top 6):")
    for off in range(0, 32, 4):
        c = floats[off]
        top = ", ".join(f"{v:g}x{n}" for v, n in c.most_common(6))
        print(f"  +{off:02d} f32 distinct={len(c):5d}  {top}")
    print()
    print(f"palette range: min={min(min(p) for p in palettes):.4f} "
          f"max={max(max(p) for p in palettes):.4f}")
    distinct_pal = len({tuple(round(v, 3) for v in p) for p in palettes})
    print(f"distinct palettes (3dp): {distinct_pal} of {len(palettes)}")

    if args.json:
        Path(args.json).write_text(json.dumps(records, indent=1))
        print(f"wrote {args.json}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

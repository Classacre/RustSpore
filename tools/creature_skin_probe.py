#!/usr/bin/env python3
"""Extract and describe the skin atlas a creature ships with.

A creature's skin is not named by any prop: it is a RASTER whose instance shares the
creature instance's low 32 bits. This tool finds those rasters, decodes the largest, and
reports what it actually contains, so the question "can a generator reproduce this" is
answered by the pixel data rather than by assumption.

Usage:
    python creature_skin_probe.py 40666200D1B4BB56 --png out.png
    python creature_skin_probe.py --all --stats
"""

import argparse
import struct
import sys
import zlib
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index, payload  # noqa: E402
from raster_extract import (  # noqa: E402
    FMT_DXT1,
    FMT_DXT5,
    FMT_RAW,
    decode_dxt,
    decode_raw_rgba,
    parse_raster_header,
)

BASE = Path("D:/Games/SPORE Collection/Data")
RASTER = 0x2F4E681C
CREATURE = 0x0F43029A


def write_png(path: Path, width: int, height: int, rgba: bytes) -> None:
    """Minimal RGBA PNG writer (filter 0 on every row)."""
    def chunk(tag: bytes, data: bytes) -> bytes:
        return (struct.pack(">I", len(data)) + tag + data
                + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF))

    raw = bytearray()
    stride = width * 4
    for y in range(height):
        raw.append(0)
        raw.extend(rgba[y * stride:(y + 1) * stride])
    out = bytearray(b"\x89PNG\r\n\x1a\n")
    out += chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
    out += chunk(b"IDAT", zlib.compress(bytes(raw), 6))
    out += chunk(b"IEND", b"")
    path.write_bytes(bytes(out))


def decode_raster(d: bytes):
    hdr = parse_raster_header(d)
    w, h, fmt = hdr["width"], hdr["height"], hdr["format"]
    tex = d[28:]
    if fmt == FMT_DXT5:
        return w, h, decode_dxt(tex, w, h, FMT_DXT5)
    if fmt == FMT_DXT1:
        return w, h, decode_dxt(tex, w, h, FMT_DXT1)
    if fmt == FMT_RAW:
        return w, h, decode_raw_rgba(tex, w, h)
    raise ValueError(f"format 0x{fmt:08X}")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("instance", nargs="?", help="creature instance, hex")
    ap.add_argument("--package", default=str(BASE / "Spore_Content.package"))
    ap.add_argument("--png", default=None)
    ap.add_argument("--all", action="store_true", help="survey every creature with a skin")
    ap.add_argument("--limit", type=int, default=0)
    args = ap.parse_args()

    path = Path(args.package)
    entries, _ = parse_index(path)
    by_low = {}
    for e in entries:
        if e["type"] == RASTER:
            by_low.setdefault(e["instance"] & 0xFFFFFFFF, []).append(e)
    creatures = [e for e in entries if e["type"] == CREATURE]

    if args.all:
        stats = Counter()
        rows = []
        for c in creatures:
            low = c["instance"] & 0xFFFFFFFF
            cands = by_low.get(low, [])
            if not cands:
                continue
            # The declared decompressed size orders the candidates without decoding
            # all six of them, which is what makes a full survey affordable.
            best = None
            for r in sorted(cands, key=lambda e: -e["dec"]):
                try:
                    w, h, rgba = decode_raster(payload(path, r))
                except Exception:
                    continue
                if best is None or w * h > best[0] * best[1]:
                    best = (w, h, rgba, r)
                break
            if best is None:
                continue
            w, h, rgba, r = best
            opaque = sum(1 for i in range(3, len(rgba), 4) if rgba[i] > 0)
            colours = Counter()
            for i in range(0, len(rgba), 4):
                if rgba[i + 3] > 0:
                    colours[(rgba[i] >> 3, rgba[i + 1] >> 3, rgba[i + 2] >> 3)] += 1
            stats[(w, h)] += 1
            rows.append((c["instance"], len(cands), w, h, opaque / (w * h), len(colours)))
            if args.limit and len(rows) >= args.limit:
                break
        print(f"creatures with a skin raster: {len(rows)}")
        print(f"skin sizes: {dict(stats)}")
        ratios = sorted(r[4] for r in rows)
        counts = sorted(r[5] for r in rows)
        print(f"opaque fraction: min={ratios[0]:.3f} median={ratios[len(ratios)//2]:.3f} max={ratios[-1]:.3f}")
        print(f"distinct 5-bit colours: min={counts[0]} median={counts[len(counts)//2]} max={counts[-1]}")
        for r in rows[:12]:
            print(f"  {r[0]:016X} rasters={r[1]} {r[2]}x{r[3]} opaque={r[4]:.3f} colours={r[5]}")
        return 0

    if not args.instance:
        print("give a creature instance or --all")
        return 2
    inst = int(args.instance, 16)
    low = inst & 0xFFFFFFFF
    cands = by_low.get(low, [])
    print(f"creature {inst:016X}: {len(cands)} raster(s) sharing low half {low:08X}")
    best = None
    for r in cands:
        try:
            w, h, rgba = decode_raster(payload(path, r))
        except Exception as exc:
            print(f"  {r['instance']:016X} undecodable: {exc}")
            continue
        print(f"  {r['instance']:016X} {w}x{h}")
        if best is None or w * h > best[0] * best[1]:
            best = (w, h, rgba, r)
    if best is None:
        return 1
    w, h, rgba, r = best
    print(f"largest: {r['instance']:016X} {w}x{h}")
    if args.png:
        write_png(Path(args.png), w, h, rgba)
        print(f"wrote {args.png}")

    colours = Counter()
    opaque = 0
    for i in range(0, len(rgba), 4):
        if rgba[i + 3] > 0:
            opaque += 1
            colours[(rgba[i] >> 3, rgba[i + 1] >> 3, rgba[i + 2] >> 3)] += 1
    print(f"opaque {opaque}/{w*h} = {opaque/(w*h):.3f}, distinct 5-bit colours {len(colours)}")
    total = max(opaque, 1)
    for c, n in colours.most_common(12):
        print(f"  ({c[0]*8:3d},{c[1]*8:3d},{c[2]*8:3d}) {n:7d} {n/total:6.1%}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

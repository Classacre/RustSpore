#!/usr/bin/env python3
"""Census the gmsh attribute sections: magic, stride, count, and value ranges.

gmsh resources carry vertex positions, then zero or more attribute sections, each
`<4-byte magic><u32 BE count><u16 BE stride><count*stride bytes>`.

This tool records which magics appear, with what strides, and what the values look
like, so each section can be mapped to a semantic (normals, UVs, colours, ...)
instead of being carried as an opaque blob.

Usage:
    python tools/gmsh_attributes.py [package ...]
"""

import struct
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

GMSH_TYPE = 0x01C135DA
INDEX_MAGICS = (b"\x15\x00\x06\x08", b"\x14\x00\x06\x08")


def walk(d):
    """Yield (magic, count, stride, offset_of_data) for each attribute section."""
    if len(d) < 24:
        return
    vcount = struct.unpack_from(">H", d, 20)[0]
    pos_stride = struct.unpack_from(">H", d, 22)[0]
    if vcount == 0 or pos_stride == 0:
        return
    off = 24 + vcount * pos_stride
    while off + 10 <= len(d):
        magic = d[off:off + 4]
        if magic in INDEX_MAGICS:
            return
        count = struct.unpack_from(">I", d, off + 4)[0]
        stride = struct.unpack_from(">H", d, off + 8)[0]
        if stride == 0 or stride > 256:
            return
        end = off + 10 + count * stride
        if end > len(d):
            return
        yield magic, count, stride, off + 10
        off = end


def stats(d, count, stride, at, limit=400):
    """Component min/max and a unit-length check for float layouts."""
    n = min(count, limit)
    comps = stride // 4 if stride % 4 == 0 else 0
    mins = [float("inf")] * comps
    maxs = [float("-inf")] * comps
    unit = 0
    for i in range(n):
        base = at + i * stride
        vals = []
        for c in range(comps):
            v = struct.unpack_from("<f", d, base + 4 * c)[0]
            if v != v or v in (float("inf"), float("-inf")):
                return None
            vals.append(v)
            mins[c] = min(mins[c], v)
            maxs[c] = max(maxs[c], v)
        if comps in (3, 4):
            length = sum(v * v for v in vals[:3]) ** 0.5
            if abs(length - 1.0) < 0.02:
                unit += 1
    return {"mins": mins, "maxs": maxs, "unit_ratio": unit / n if n else 0.0, "sampled": n}


def main():
    pkgs = sys.argv[1:]
    if not pkgs:
        pkgs = [str(p) for p in sorted(Path("D:/Games/SPORE Collection").rglob("*.package"))]

    combos = Counter()
    samples = defaultdict(list)
    total = 0
    for pkg in pkgs:
        try:
            entries, _ = parse_index(Path(pkg))
        except Exception:
            continue
        with open(pkg, "rb") as fh:
            for e in entries:
                if e["type"] != GMSH_TYPE:
                    continue
                fh.seek(e["offset"])
                raw = fh.read(e["stored"])
                if e["dec"] > e["stored"]:
                    try:
                        raw = decompress(raw, out_size=None, header=True)[0]
                    except Exception:
                        continue
                total += 1
                for magic, count, stride, at in walk(raw):
                    combos[(magic.hex(), stride)] += 1
                    key = (magic.hex(), stride)
                    if len(samples[key]) < 3:
                        s = stats(raw, count, stride, at)
                        if s:
                            samples[key].append(s)

    print(f"gmsh resources: {total}")
    print(f"distinct (magic, stride): {len(combos)}\n")
    print(f"{'magic':10} {'stride':>6} {'sections':>8}  value ranges")
    for (magic, stride), n in combos.most_common(20):
        s = samples[(magic, stride)]
        desc = ""
        if s:
            a = s[0]
            if a["mins"]:
                desc = " ".join(
                    f"[{a['mins'][i]:.3g},{a['maxs'][i]:.3g}]" for i in range(len(a["mins"]))
                )
                desc += f" unit={a['unit_ratio']:.2f}"
        print(f"{magic:10} {stride:>6} {n:>8}  {desc}")


if __name__ == "__main__":
    main()

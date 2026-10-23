#!/usr/bin/env python3
"""Identify which gmsh attribute section is the normal, the tangent and the UV set.

Naming a section by its magic would be a guess. This tool tests each float3
section against the geometry: a true normal should point roughly along the
triangle normals, so the mean dot of the per-vertex vector with the incident face
normals is strongly positive. A tangent should be roughly perpendicular to it.

Usage:
    python tools/gmsh_semantics.py [package ...]
"""

import math
import struct
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress
from gmsh_attributes import walk
from gmsh_complete import parse_full

GMSH_TYPE = 0x01C135DA


def sub(a, b):
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def dot(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def norm(a):
    n = math.sqrt(dot(a, a))
    return (a[0] / n, a[1] / n, a[2] / n) if n > 1e-12 else (0.0, 0.0, 0.0)


def face_normals(positions, indices):
    """Accumulate area-weighted face normals per vertex."""
    acc = defaultdict(lambda: [0.0, 0.0, 0.0])
    for t in range(0, len(indices) - 2, 3):
        i0, i1, i2 = indices[t], indices[t + 1], indices[t + 2]
        if max(i0, i1, i2) >= len(positions):
            continue
        n = cross(sub(positions[i1], positions[i0]), sub(positions[i2], positions[i0]))
        for i in (i0, i1, i2):
            acc[i][0] += n[0]
            acc[i][1] += n[1]
            acc[i][2] += n[2]
    return {k: norm(tuple(v)) for k, v in acc.items()}


def main():
    pkgs = sys.argv[1:] or [str(p) for p in sorted(Path("D:/Games/SPORE Collection").rglob("*.package"))]

    # magic -> list of (mean |dot| with geometric normal, unit ratio)
    results = defaultdict(list)
    examined = 0

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
                parsed = parse_full(raw)
                if not parsed or not parsed.get("all_valid"):
                    continue
                indices = parsed["indices"]
                if len(indices) < 3:
                    continue
                vcount = struct.unpack_from(">H", raw, 20)[0]
                pstride = struct.unpack_from(">H", raw, 22)[0]
                if pstride != 12 or vcount == 0:
                    continue
                positions = [
                    struct.unpack_from("<3f", raw, 24 + i * 12) for i in range(vcount)
                ]
                geo = face_normals(positions, indices)
                if len(geo) < 3:
                    continue

                for magic, count, stride, at in walk(raw):
                    if stride != 12:
                        continue
                    dots = []
                    unit = 0
                    for i in range(min(count, vcount)):
                        v = struct.unpack_from("<3f", raw, at + i * 12)
                        ln = math.sqrt(dot(v, v))
                        if abs(ln - 1.0) < 0.02:
                            unit += 1
                        g = geo.get(i)
                        if g:
                            dots.append(abs(dot(norm(v), g)))
                    if dots:
                        results[magic.hex()].append((sum(dots) / len(dots), unit / max(1, min(count, vcount))))
                examined += 1
                if examined >= 60:
                    break
        if examined >= 60:
            break

    print(f"meshes examined: {examined}\n")
    print(f"{'magic':10} {'samples':>8} {'mean|dot|':>10} {'unit':>6}")
    for magic, vals in sorted(results.items(), key=lambda kv: -len(kv[1])):
        md = sum(v[0] for v in vals) / len(vals)
        un = sum(v[1] for v in vals) / len(vals)
        print(f"{magic:10} {len(vals):>8} {md:>10.3f} {un:>6.2f}")


if __name__ == "__main__":
    main()

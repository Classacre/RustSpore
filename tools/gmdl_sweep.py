#!/usr/bin/env python3
"""Sweep fixed sizes for gmdl shader-data 0x238 and report exact counts."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress
import gmdl_full as G

BASE = Path("D:/Games/SPORE Collection/Data")


def main():
    samples = []
    for pkg in sorted(BASE.glob("*.package")):
        try:
            entries, _ = parse_index(pkg)
        except Exception:
            continue
        with pkg.open("rb") as fh:
            for e in entries:
                if e["type"] != G.TARGET:
                    continue
                fh.seek(e["offset"])
                raw = fh.read(e["stored"])
                if e["dec"] > e["stored"]:
                    try:
                        raw = decompress(raw, out_size=None, header=True)[0]
                    except Exception:
                        continue
                samples.append(raw)
    print(f"Collected {len(samples)}")

    # Baseline with the n-bytes branch (override None).
    G.S238_OVERRIDE = None
    base = 0
    for raw in samples:
        try:
            kind, ce, end, emsg = G.parse(raw)
        except G.Err:
            continue
        if kind in ("trailing", "core") and end == len(raw):
            base += 1
    print(f"baseline (n-bytes): {base}")

    best = (base, None)
    for size in range(0, 260, 4):
        G.S238_OVERRIDE = size
        exact = 0
        for raw in samples:
            try:
                kind, ce, end, emsg = G.parse(raw)
            except G.Err:
                continue
            if kind in ("trailing", "core") and end == len(raw):
                exact += 1
        if exact > best[0]:
            best = (exact, size)
        if exact > base:
            print(f"size {size}: exact {exact}")
    print(f"best: exact={best[0]} at 0x238 size={best[1]}")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Find and diagnose the one gmsh sample that parse_full rejects."""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress
from gmsh_complete import parse_full, GMSH_TYPE

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
                if e["type"] != GMSH_TYPE:
                    continue
                fh.seek(e["offset"])
                raw = fh.read(e["stored"])
                if e["dec"] > e["stored"]:
                    try:
                        raw = decompress(raw, out_size=None, header=True)[0]
                    except Exception:
                        continue
                samples.append((pkg.name, raw))
                if len(samples) >= 2000:
                    break
        if len(samples) >= 2000:
            break

    fails = []
    for i, (name, d) in enumerate(samples):
        if parse_full(d) is None:
            fails.append((i, name, d))

    print(f"Total {len(samples)}, failures {len(fails)}")
    for i, name, d in fails:
        print(f"\n=== FAIL #{i} pkg={name} size={len(d)}")
        # Header
        print(f"  vcount_BE@20={struct.unpack_from('>H', d, 20)[0]}")
        print(f"  pos_stride_BE@22={struct.unpack_from('>H', d, 22)[0]}")
        # Find the index magic.
        idx = d.find(b"\x15\x00\x06\x08")
        print(f"  index magic at {idx}")
        if idx >= 0:
            ver = struct.unpack_from(">I", d, idx + 4)[0]
            print(f"  index version={ver}")
        # Dump first 96 bytes.
        for row in range(0, min(96, len(d)), 16):
            chunk = d[row:row + 16]
            hs = " ".join(f"{b:02x}" for b in chunk)
            asc = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
            print(f"    +{row:04x}: {hs:<47} {asc}")
        # Re-trace the attribute walk to see where it stops.
        if len(d) >= 24:
            vcount = struct.unpack_from(">H", d, 20)[0]
            pos_stride = struct.unpack_from(">H", d, 22)[0]
            off = 24 + vcount * pos_stride
            print(f"  pos data end (attr start) = {off} (len {len(d)})")
            while off + 10 <= len(d):
                magic = d[off:off + 4]
                if magic == b"\x15\x00\x06\x08":
                    print(f"    index magic reached at off={off}")
                    break
                av = struct.unpack_from(">I", d, off + 4)[0]
                ast = struct.unpack_from(">H", d, off + 8)[0]
                print(f"    attr @{off}: magic={magic.hex(' ')} vcount={av} stride={ast}")
                if ast == 0 or ast > 256 or off + 10 + av * ast > len(d):
                    print(f"      -> invalid attr, break")
                    break
                off += 10 + av * ast


if __name__ == "__main__":
    main()

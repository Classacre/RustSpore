#!/usr/bin/env python3
"""Validate a closed-form gmdl (0x00E6BCE5) decoder across the full population.

Corrected header (the field at +4 is a reference count, not flags):
    +0  u32 version = 8
    +4  u32 ref_count
    +8  refs: ref_count * 12 bytes (4-byte type + 8-byte key; type 0x2F4E681B)
    +8+ref*12  u32 mesh_count
    +12+ref*12 7 floats bbox
    +40+ref*12 u32 tb_count
    +44+ref*12 u32 prim
    +48+ref*12 u32 idx_count
    +52+ref*12 u32 idx_bits
    +56+ref*12 u32 buf_size
    +60+ref*12 u32 pad
    header_size = 64 + ref_count*12
    index buffer: buf_size bytes (u16 LE)
    vertex section 56 bytes: 48-byte descriptor + u32 vcount + u32 vsize
    vertex data vsize bytes (= vcount*24)
    trailer: 76 + N*24 bytes (N at trailer+0x34)

Expected total = header_size + buf_size + 56 + vsize + 76 + N*24
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x00E6BCE5


def main():
    total = 0
    exact = 0
    cats = Counter()
    mesh_dist = Counter()
    ref_dist = Counter()
    n_dist = Counter()
    mismatches = []
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
                total += 1
                size = len(raw)
                if size < 64:
                    cats["too_short"] += 1
                    continue
                version = struct.unpack_from("<I", raw, 0)[0]
                if version != 8:
                    cats[f"version_{version}"] += 1
                    continue
                ref_count = struct.unpack_from(">I", raw, 4)[0]
                ref_dist[ref_count] += 1
                if ref_count > 10000:
                    cats["bad_ref_count"] += 1
                    continue
                hdr = 64 + ref_count * 12
                if hdr + 24 > size:
                    cats["hdr_oob"] += 1
                    continue
                mesh_count = struct.unpack_from("<I", raw, 8 + ref_count * 12)[0]
                idx_count = struct.unpack_from("<I", raw, 48 + ref_count * 12)[0]
                buf_size = struct.unpack_from("<I", raw, 56 + ref_count * 12)[0]
                mesh_dist[mesh_count] += 1
                if mesh_count != 1:
                    cats[f"mesh_{mesh_count}"] += 1
                    continue
                if buf_size == 0 or buf_size != idx_count * 2:
                    cats["bad_buf"] += 1
                    continue
                idx_end = hdr + buf_size
                if idx_end + 56 > size:
                    cats["idx_oob"] += 1
                    continue
                vcount = struct.unpack_from("<I", raw, idx_end + 48)[0]
                vsize = struct.unpack_from("<I", raw, idx_end + 52)[0]
                if vcount == 0 or vsize != vcount * 24:
                    cats["bad_vertex_section"] += 1
                    continue
                vdata_end = idx_end + 56 + vsize
                if vdata_end + 0x38 > size:
                    cats["trailer_oob"] += 1
                    continue
                N = struct.unpack_from("<I", raw, vdata_end + 0x34)[0]
                n_dist[N] += 1
                if N > 1000:
                    cats["bad_refN"] += 1
                    continue
                expected = vdata_end + 76 + N * 24
                if size == expected:
                    exact += 1
                    cats["exact"] += 1
                else:
                    cats["size_mismatch"] += 1
                    mismatches.append((size, expected, ref_count, idx_count, vcount, N))

    print(f"Total gmdl: {total}")
    print(f"EXACT: {exact} ({100*exact/total:.4f}%)")
    print(f"Categories: {dict(cats)}")
    print(f"mesh_count distribution: {dict(mesh_dist.most_common(12))}")
    print(f"ref_count distribution: {dict(ref_dist.most_common(12))}")
    print(f"trailer N distribution: {dict(n_dist.most_common(8))}")
    print(f"Mismatches: {len(mismatches)}")
    for m in mismatches[:10]:
        print(f"  size/expected/ref/idx/vcount/N: {m}")


if __name__ == "__main__":
    main()

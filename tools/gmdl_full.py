#!/usr/bin/env python3
"""Full gmdl (0x00E6BCE5) parser, following the public SporeModder structure.

Structure (little-endian unless noted):
    version          i32
    ref_count        i32 (BE)
    refs             ref_count * 12 bytes (ResourceKey: BE u32 instance, type, group)
    mesh_count       i32
    bbox             6 f32, then radius f32
    tri_buffers_count i32
      per buffer: primitive_type, indices_count, indices_bits, buffer_size (4 i32)
                  indices: indices_count * (2 or 4) bytes
    vertex_formats_count i32
      per format: element_count i32, then element_count * VertexElement (<hhbbbbi, 12 bytes)
    vertex_buffers_count i32
      per buffer: format_index, vertex_count, buffer_size (3 i32)
                  vertices: vertex_count * stride bytes (stride = sum of element sizes)
    mesh_count times: t_index i32, v_index i32
    mesh_count times: material_id i32
    [trailing section if present]
        u32 (0), material_count, materials, skeleton data

This tool reports the exact-consumption rate over the full population.
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

# D3DDECLTYPE -> byte size
DECL_SIZE = {
    0: 4, 1: 8, 2: 12, 3: 16, 4: 4, 5: 4, 6: 4, 7: 8, 8: 4,
    9: 4, 10: 8, 11: 4, 12: 8, 13: 4, 14: 4, 15: 4, 16: 8,
}
SHADER_DATA_SIZES = {0x210: 20, 0x218: 0, 0x238: 1}
S238_OVERRIDE = None  # when set, shader-data 0x238 uses this fixed byte size


class Err(Exception):
    pass


class Reader:
    def __init__(self, d):
        self.d = d
        self.p = 0

    def i32(self, be=False):
        if self.p + 4 > len(self.d):
            raise Err("eof i32")
        v = struct.unpack_from(">i" if be else "<i", self.d, self.p)[0]
        self.p += 4
        return v

    def f32(self):
        if self.p + 4 > len(self.d):
            raise Err("eof f32")
        v = struct.unpack_from("<f", self.d, self.p)[0]
        self.p += 4
        return v

    def take(self, n):
        if n < 0 or self.p + n > len(self.d):
            raise Err("eof take")
        self.p += n


def read_material(r):
    # Version 8 material: a shader-data count directly (no leading BE "unk" field,
    # which is a version 9 addition).
    shdata_count = r.i32()
    if shdata_count < 0 or shdata_count > 10000:
        raise Err("bad shdata_count")
    for _ in range(shdata_count):
        shdata_index = r.i32()
        if shdata_index == 0x20D:
            texture_count = r.i32()
            if texture_count < 0 or texture_count > 10000:
                raise Err("bad texture_count")
            for _ in range(texture_count):
                r.take(16)
                r.take(8)
        elif shdata_index in SHADER_DATA_SIZES:
            r.take(SHADER_DATA_SIZES[shdata_index])
        elif shdata_index == 0x238 and S238_OVERRIDE is not None:
            r.take(S238_OVERRIDE)
        else:
            raise Err("unknown shader data 0x%x" % shdata_index)


def parse(d):
    r = Reader(d)
    version = r.i32()
    ref_count = r.i32(be=True)
    if ref_count < 0 or ref_count > 10000:
        raise Err("bad ref_count")
    r.take(ref_count * 12)
    mesh_count = r.i32()
    if mesh_count < 0 or mesh_count > 10000:
        raise Err("bad mesh_count")
    r.take(6 * 4)  # bbox
    r.f32()        # radius

    tri_count = r.i32()
    if tri_count < 0 or tri_count > 10000:
        raise Err("bad tri_count")
    for _ in range(tri_count):
        prim = r.i32()
        idx_count = r.i32()
        bits = r.i32()
        r.i32()  # buffer size
        if idx_count < 0 or idx_count > 50_000_000:
            raise Err("bad idx_count")
        w = 2 if bits == 16 else 4
        r.take(idx_count * w)

    vf_count = r.i32()
    if vf_count < 0 or vf_count > 10000:
        raise Err("bad vf_count")
    vformats = []
    for _ in range(vf_count):
        elem_count = r.i32()
        if elem_count < 0 or elem_count > 1000:
            raise Err("bad elem_count")
        elems = []
        for _ in range(elem_count):
            if r.p + 12 > len(d):
                raise Err("eof element")
            stream, offset, etype, method, usage, uidx, rw_decl = struct.unpack_from("<hhbbbbi", d, r.p)
            r.p += 12
            elems.append(etype)
        vformats.append(elems)

    vb_count = r.i32()
    if vb_count < 0 or vb_count > 10000:
        raise Err("bad vb_count")
    for _ in range(vb_count):
        fmt_index = r.i32()
        vcount = r.i32()
        r.i32()  # buffer size
        if fmt_index < 0 or fmt_index >= len(vformats):
            raise Err("bad fmt_index")
        if vcount < 0 or vcount > 50_000_000:
            raise Err("bad vcount")
        stride = sum(DECL_SIZE.get(t, 4) for t in vformats[fmt_index])
        r.take(vcount * stride)

    for _ in range(mesh_count):
        r.i32()  # t_index
        r.i32()  # v_index
    for _ in range(mesh_count):
        r.i32()  # material_id

    core_end = r.p

    # Optional trailing (skinned) section.
    if r.p == len(d):
        return ("core", core_end, len(d), None)
    # Try to parse the trailing section.
    try:
        zero = r.i32()
        if zero != 0:
            raise Err("trailing nonzero")
        material_count = r.i32()
        if material_count < 0 or material_count > 10000:
            raise Err("bad material_count")
        for _ in range(material_count):
            read_material(r)
        count1 = r.i32()
        if count1 < 0 or count1 > 10000:
            raise Err("bad count1")
        r.take(count1 * 8)
        part_count = r.i32()
        if part_count < 0 or part_count > 100000:
            raise Err("bad part_count")
        for _ in range(part_count):
            r.take(0x38)
            r.take(0x38)
            r.i32()  # field_70
            r.take(12)  # key <III
            var = r.i32()
            if var != 0:
                bone_count = r.i32()
                if bone_count < 0 or bone_count > 100000:
                    raise Err("bad bone_count")
                r.take(0xA0 * bone_count)
        r.take(12)  # fixed 12-byte footer (00 00 00 00 | ff ff ff ff | 00 00 00 00)
        return ("trailing", core_end, r.p, None)
    except Err as ex:
        return ("fail_trailing", core_end, r.p, str(ex))


def main():
    total = 0
    exact = 0
    cats = Counter()
    errs = Counter()
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
                try:
                    kind, core_end, end, emsg = parse(raw)
                except Err as ex:
                    cats["error"] += 1
                    errs[str(ex)] += 1
                    continue
                if kind == "fail_trailing":
                    cats["fail_trailing"] += 1
                    errs[emsg] += 1
                    if len(mismatches) < 8:
                        mismatches.append((len(raw), core_end, end, emsg))
                else:
                    cats[kind] += 1
                    if end == len(raw):
                        exact += 1
                    else:
                        cats["not_exact_" + kind] += 1
                        if len(mismatches) < 8:
                            mismatches.append((len(raw), core_end, end))

    print(f"Total gmdl: {total}")
    print(f"EXACT: {exact} ({100*exact/total:.4f}%)")
    print(f"Categories: {dict(cats)}")
    print(f"Errors: {dict(errs.most_common(10))}")
    print(f"Mismatch samples (size, core_end, end): {mismatches}")


if __name__ == "__main__":
    main()

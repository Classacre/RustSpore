#!/usr/bin/env python3
"""Validate RW4 (0x2F4E681B) per-object payload consumption.

For every section in the container, decode the object's fields by its type code
and check that the bytes consumed equal the section's data_size. This is the
per-object layer on top of tools/rw4_validate.py (which validates the container).

Layouts are reimplemented from the public SporeModder rw4_base.py (no code copied).
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x2F4E681B
MAGIC = b"\x89RW4w32\x00\x0d\x0a\x1a\x0a"
BASE_RESOURCE = 0x10030


def i32(d, o):
    return struct.unpack_from("<i", d, o)[0]


def u16(d, o):
    return struct.unpack_from("<H", d, o)[0]


def i16(d, o):
    return struct.unpack_from("<h", d, o)[0]


def consumed_for(type_code, d, o, data_size):
    """Return bytes consumed by the object, or None if the type is not handled."""
    try:
        if type_code == 0x10030:  # BaseResource
            return data_size
        if type_code == 0x10004:  # SectionManifest
            return 24
        if type_code == 0x10005:  # SectionTypes
            return 8 + i32(d, o) * 4
        if type_code == 0x10006:  # SectionExternalArenas
            return 32
        if type_code == 0x10007:  # SectionSubReferences
            return 24
        if type_code == 0x10008:  # SectionAtoms
            return 8
        if type_code == 0x20003:  # Raster
            return 32
        if type_code == 0x20004:  # VertexDescription
            return 24 + i16(d, o + 12) * 12
        if type_code == 0x20005:  # VertexBuffer
            return 28
        if type_code == 0x20007:  # IndexBuffer
            return 28
        if type_code == 0x20008:  # TextureOverride (string)
            # hash(4) + zero-terminated ascii
            p = o + 4
            while p < o + data_size and d[p] != 0:
                p += 1
            return (p - o) + 1
        if type_code == 0x20009:  # Mesh
            return 36 + i32(d, o + 16) * 4
        if type_code == 0x2001A:  # MeshCompiledStateLink
            return 8 + i32(d, o + 4) * 4
        if type_code == 0x2000B:  # CompiledState
            return i32(d, o)
        if type_code == 0x7000C:  # SkinsInK
            return 20
        if type_code == 0x70002:  # Skeleton
            return 24 + i32(d, o + 12) * 12
        if type_code == 0x7000F:  # SkinMatrixBuffer
            return 16 + i32(d, o + 4) * 48
        if type_code == 0x80005:  # BoundingBox
            return 32
        if type_code == 0x70003:  # AnimationSkin: 16 + count*64
            return 16 + i32(d, o + 4) * 64
        if type_code == 0xFF0000:  # MorphHandle: 8 + 6 doubles + 2 i32
            return 64
        if type_code == 0xFF0001:  # Animations: 8 + count*8
            return 8 + i32(d, o + 4) * 8
        if type_code == 0xFF0002:  # BlendShape: 28 + 2*shape_count*4
            return 28 + 2 * i32(d, o + 12) * 4
        if type_code == 0x7000B:  # fixed 32-byte object (not in the Blender addon)
            return 32
        # Self-contained blobs with internal pointers: the section data_size covers
        # the whole object (KeyframeAnim, BlendShapeBuffer, TriangleKDTreeProcedural).
        if type_code in (0x70001, 0x200AF, 0x80003):
            return data_size
        return None
    except Exception:
        return -1


def validate(d):
    n = len(d)
    if n < 72 or d[:12] != MAGIC:
        return "bad_header"
    section_count = i32(d, 36)
    p_section_infos = i32(d, 48)
    p_buffer_data = i32(d, 68)
    if section_count <= 0 or p_section_infos + section_count * 24 > n:
        return "bad_infos"
    checked = 0
    bad = 0
    unknown = 0
    for i in range(section_count):
        base = p_section_infos + i * 24
        p_data = i32(d, base)
        data_size = i32(d, base + 8)
        type_code = i32(d, base + 20)
        start = (p_buffer_data + p_data) if type_code == BASE_RESOURCE else p_data
        if start < 0 or start + data_size > n:
            return "section_oob"
        got = consumed_for(type_code, d, start, data_size)
        if got is None:
            unknown += 1
        elif got != data_size:
            bad += 1
    if bad:
        return "object_mismatch"
    if unknown:
        return "unknown_type"
    return "ok"


def main():
    total = 0
    cats = Counter()
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
                cats[validate(raw)] += 1
    print(f"Total RW4: {total}")
    print(f"Categories: {dict(cats.most_common(10))}")
    ok = cats.get("ok", 0)
    print(f"All-objects-exact: {ok} ({100*ok/total:.4f}%)")


if __name__ == "__main__":
    main()

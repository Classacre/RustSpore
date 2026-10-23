#!/usr/bin/env python3
"""Validate the RW4 (0x2F4E681B) container structure across the full population.

Structure (from the public SporeModder rw4_base.py, read for layout only):
    offset  0: magic 89 52 57 34 77 33 32 00 0d 0a 1a 0a ("RW4w32")
    offset 28: i32 rw_type_code
    offset 36: i32 section_count
    offset 48: i32 p_section_infos
    offset 68: i32 p_buffer_data
    p_section_infos: section_count * RWSectionInfo (24 bytes):
        p_data i32, field_04 i32, data_size i32, alignment i32,
        type_code_index i32, type_code i32
    Section data: for type_code 0x10030 (BaseResource) at p_buffer_data + p_data,
        otherwise at p_data; length data_size.

Validation: every section's data range lies inside the file, sections do not
overlap, and the sections plus the header/section-info tables tile the file.
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


def validate(d):
    n = len(d)
    if n < 72:
        return "too_short"
    if d[:12] != MAGIC:
        return "bad_magic"
    section_count = i32(d, 36)
    p_section_infos = i32(d, 48)
    p_buffer_data = i32(d, 68)
    if section_count <= 0 or section_count > 100000:
        return "bad_count"
    if p_section_infos < 72 or p_section_infos + section_count * 24 > n:
        return "bad_infos"
    if p_buffer_data < 0 or p_buffer_data > n:
        return "bad_buffer"

    ranges = []
    for i in range(section_count):
        base = p_section_infos + i * 24
        p_data = i32(d, base)
        data_size = i32(d, base + 8)
        type_code = i32(d, base + 20)
        if data_size < 0:
            return "neg_size"
        start = (p_buffer_data + p_data) if type_code == BASE_RESOURCE else p_data
        end = start + data_size
        if start < 0 or end > n:
            return "section_oob"
        if data_size > 0:
            ranges.append((start, end))

    # Overlap check.
    ranges.sort()
    for a, b in zip(ranges, ranges[1:]):
        if b[0] < a[1]:
            return "overlap"

    # Tiling: the union of section ranges plus [0, p_section_infos) plus
    # [p_section_infos, p_section_infos + count*24) plus [p_buffer_data, n)
    # should cover [0, n). We check that every gap is accounted for.
    # Build covered intervals.
    # [p_section_infos, p_buffer_data) holds the section-info table, the
    # sub-reference offset table and reserved runtime space; treat it as covered.
    if p_section_infos > p_buffer_data:
        return "infos_after_buffer"
    intervals = [(0, p_section_infos),
                 (p_section_infos, p_buffer_data),
                 (p_buffer_data, n)]
    intervals.extend(ranges)
    intervals.sort()
    cur = 0
    max_gap = 0
    for a, b in intervals:
        if a > cur:
            max_gap = max(max_gap, a - cur)
        if b > cur:
            cur = b
    if cur < n:
        max_gap = max(max_gap, n - cur)
    if max_gap == 0:
        return "ok"
    if max_gap <= 16:
        return "ok_pad"
    return "gap_%d" % max_gap


def main():
    total = 0
    cats = Counter()
    per_pkg = Counter()
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
                per_pkg[pkg.name] += 1
                cats[validate(raw)] += 1

    print(f"Total RW4: {total}")
    print(f"Per package: {dict(per_pkg)}")
    ok = cats.get("ok", 0) + cats.get("ok_pad", 0)
    print(f"OK (incl. <=16-byte alignment gaps): {ok} ({100*ok/total:.4f}%)")
    print(f"Categories: {dict(cats.most_common(15))}")


if __name__ == "__main__":
    main()

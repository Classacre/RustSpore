#!/usr/bin/env python3
"""Validate type 0x1A99B06B (.bem, creature body editor model).

Structure (from SporeModder BEMMain.java, reimplemented):
    i32 magic = 0x48657ED3 (LE)
    i32 version (3 < version <= 5)
    i32 unk1 (<= 1024)
    i32 unk2 (<= 128)
    RLE section 1, decompressed size 128 (v<5) or 68 (v>=5)
    RLE section 2, decompressed size unk1 * 472
RLE: repeat u16 LE chunk_size (1..2048), then chunk_size bytes; 0xFE escapes to
either a literal 0xFE (count 0) or a run of the previous byte.
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x1A99B06B
MAGIC = 0x48657ED3


def rle_decompress(d, p, size):
    n = len(d)
    dst = 0
    buf = None
    i = 0
    buf_len = 0
    set_byte = 0
    while dst < size:
        if i >= buf_len:
            buf = None
        if buf is None:
            i = 0
            if p + 2 > n:
                return None
            buf_len = struct.unpack_from("<H", d, p)[0]
            p += 2
            if buf_len < 1 or buf_len > 2048 or p + buf_len > n:
                return None
            buf = d[p:p + buf_len]
            p += buf_len
        if buf[i] == 0xFE:
            i += 1
            if i >= buf_len:
                return None
            if buf[i] == 0:
                dst += 1
                set_byte = 0xFE
            else:
                set_size = buf[i]
                if set_size > size - dst:
                    return None
                dst += set_size
        else:
            dst += 1
            set_byte = buf[i]
        i += 1
    return p


def parse(d):
    n = len(d)
    if n < 16:
        return False
    if struct.unpack_from("<I", d, 0)[0] != MAGIC:
        return False
    version = struct.unpack_from("<I", d, 4)[0]
    if version <= 3 or version > 5:
        return False
    unk1 = struct.unpack_from("<I", d, 8)[0]
    unk2 = struct.unpack_from("<I", d, 12)[0]
    if unk1 > 1024 or unk2 > 128:
        return False
    # Measured: decompressed size 128 for every v5 sample (1,733/1,733). The
    # 68 value from the reference applies to no file in this install.
    section_size = 128 if version >= 5 else 68
    p = rle_decompress(d, 16, section_size)
    if p is None:
        return False
    p = rle_decompress(d, p, unk1 * 472)
    if p is None:
        return False
    return p == n


def main():
    total = 0
    ok = 0
    cats = Counter()
    vers = Counter()
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
                if len(raw) >= 8:
                    vers[struct.unpack_from("<I", raw, 4)[0]] += 1
                if parse(raw):
                    ok += 1
                else:
                    cats["fail"] += 1
    print(f"Total 0x1A99B06B (bem): {total}")
    print(f"OK: {ok} ({100*ok/total:.4f}%)")
    print(f"Versions: {dict(vers)}")
    print(f"Failures: {dict(cats)}")


if __name__ == "__main__":
    main()

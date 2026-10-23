#!/usr/bin/env python3
"""Validator for type 0x03F51892 (audio event variant).

Hypothesis (re/payload-formats.md): this type uses the SAME entry rules as
0x02B9F662 (FNV-1 hashed keys, variable-length entries, prop-style array
markers, length-prefixed string arrays, separators). This script applies the
exact-consumption walk from audio_event_v4.py to every 0x03F51892 resource and
reports the exact-parse rate.

If the rate is 100% the hypothesis is confirmed by measurement.
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x03F51892
ARRAY_MARKERS = {0x009C, 0x001C}
STRING_TYPES = {0x0012, 0x0013}


def is_sep(d, p):
    if p + 4 > len(d):
        return False
    if d[p:p + 4] == b"\x01\x00\x00\x00":
        return True
    return d[p + 2] == 0x12 and d[p + 3] == 0x00 or (d[p + 2] == 0xA0 and d[p + 3] == 0x00)


def _read_str_array(d, p, cnt, w):
    pp = p
    for _ in range(cnt):
        if pp + 4 > len(d):
            return None
        ln = struct.unpack_from(">I", d, pp)[0]
        if ln > 100_000:
            return None
        pp += 4 + ln * w
        if pp > len(d):
            return None
    return pp


def walk(d):
    n = len(d)
    p = 4
    while p < n:
        if is_sep(d, p):
            p += 4
            continue
        if p + 8 > n:
            return None, p
        h = struct.unpack_from(">I", d, p)[0]
        a = struct.unpack_from(">H", d, p + 4)[0]
        b = struct.unpack_from(">H", d, p + 6)[0]
        if b in ARRAY_MARKERS:
            if p + 16 > n:
                return None, p
            cnt = struct.unpack_from(">I", d, p + 8)[0]
            st = struct.unpack_from(">I", d, p + 12)[0]
            if a in STRING_TYPES:
                w = 1 if a == 0x0012 else 2
                end = _read_str_array(d, p + 16, cnt, w)
                if end is None:
                    return None, p
                p = end
            else:
                need = 16 + cnt * st
                if cnt > 1_000_000 or st > 1_000_000 or p + need > n:
                    return None, p
                p += need
        elif a in STRING_TYPES:
            if p + 12 > n:
                return None, p
            ln = struct.unpack_from(">I", d, p + 8)[0]
            w = 1 if a == 0x0012 else 2
            if ln > 100_000 or p + 12 + ln * w > n:
                return None, p
            p += 12 + ln * w
        elif a == 1:
            if p + 9 > n:
                return None, p
            p += 9
        else:
            if p + 12 > n:
                return None, p
            p += 12
    return p, n


def main():
    samples = []
    for pkg in sorted(BASE.glob("*.package")):
        try:
            entries, _ = parse_index(pkg)
        except Exception:
            continue
        with pkg.open("rb") as fh:
            for e in entries:
                if e["type"] != TARGET:
                    continue
                fh.seek(e["offset"]); raw = fh.read(e["stored"])
                if e["dec"] > e["stored"]:
                    try:
                        raw = decompress(raw, out_size=None, header=True)[0]
                    except Exception:
                        continue
                samples.append(raw)
    print(f"Total samples (0x{TARGET:08X}): {len(samples)}")
    if not samples:
        return
    exact = 0
    tail_hist = Counter()
    for d in samples:
        res = walk(d)
        if res[0] is None:
            tail_hist[("ERR", res[1], len(d))] += 1
            continue
        consumed, n = res
        if consumed == n:
            exact += 1
        else:
            tail_hist[(n - consumed)] += 1
    print(f"EXACT: {exact} ({100*exact/len(samples):.4f}%)")
    print("Residual tail sizes (top 15):")
    for k, c in tail_hist.most_common(15):
        print(f"  {k}: {c}")


if __name__ == "__main__":
    main()

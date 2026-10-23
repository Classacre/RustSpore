#!/usr/bin/env python3
"""Parser v4 for type 0x02B9F662 (audio event).

Entry model (all big-endian):
  hash (4) + u16 type (2) + u16 flags (2) + payload
  payload:
    - if flags in {0x009C, 0x001C}: array = u32 count (4) + u32 stride (4) + count*stride
    - elif type == 1 (bool): 1-byte value
    - else: 4-byte value
  Also: 4-byte separators (prop-style: low word 12 00 / a0 00, or constant 01 00 00 00).

The header is a u32 count that is advisory (like prop); we walk to end of buffer.
"""

import struct
import sys
from pathlib import Path
from collections import Counter

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x02B9F662
KNOWN = {0x5F6317D5: "parent", 0x25DF0108: "gain", 0x12D2A4D4: "is3d",
         0x701ED91E: "samples", 0xBA03D0CD: "codec"}
ARRAY_MARKERS = {0x009C, 0x001C}
STRING_TYPES = {0x0012, 0x0013}


def is_sep(d, p):
    if p + 4 > len(d):
        return False
    if d[p:p + 4] == b"\x01\x00\x00\x00":
        return True
    return d[p + 2] == 0x12 and d[p + 3] == 0x00 or (d[p + 2] == 0xA0 and d[p + 3] == 0x00)


def _read_str_array(d, p, cnt, w):
    """Read `cnt` length-prefixed strings starting at p. Return end offset or None."""
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
    """Return (consumed, n, n_entries, n_seps, n_arrays, n_strings) or (None,...) on error."""
    n = len(d)
    p = 4  # skip count header
    n_entries = n_seps = n_arrays = n_strings = 0
    while p < n:
        if is_sep(d, p):
            n_seps += 1
            p += 4
            continue
        if p + 8 > n:
            return None, p, n, n_entries, n_seps, n_arrays, n_strings
        h = struct.unpack_from(">I", d, p)[0]
        a = struct.unpack_from(">H", d, p + 4)[0]
        b = struct.unpack_from(">H", d, p + 6)[0]
        if b in ARRAY_MARKERS:
            if p + 16 > n:
                return None, p, n, n_entries, n_seps, n_arrays, n_strings
            cnt = struct.unpack_from(">I", d, p + 8)[0]
            st = struct.unpack_from(">I", d, p + 12)[0]
            if a in STRING_TYPES:
                w = 1 if a == 0x0012 else 2
                end = _read_str_array(d, p + 16, cnt, w)
                if end is None:
                    return None, p, n, n_entries, n_seps, n_arrays, n_strings
                n_arrays += 1
                p = end
                continue
            # fixed-stride array
            need = 16 + cnt * st
            if cnt > 1_000_000 or st > 1_000_000 or p + need > n:
                return None, p, n, n_entries, n_seps, n_arrays, n_strings
            n_arrays += 1
            p += need
        elif a in STRING_TYPES:
            if p + 12 > n:
                return None, p, n, n_entries, n_seps, n_arrays, n_strings
            ln = struct.unpack_from(">I", d, p + 8)[0]
            w = 1 if a == 0x0012 else 2
            if ln > 100_000 or p + 12 + ln * w > n:
                return None, p, n, n_entries, n_seps, n_arrays, n_strings
            n_strings += 1
            p += 12 + ln * w
        elif a == 1:  # bool -> 1-byte value
            if p + 9 > n:
                return None, p, n, n_entries, n_seps, n_arrays, n_strings
            p += 9
            n_entries += 1
        else:
            if p + 12 > n:
                return None, p, n, n_entries, n_seps, n_arrays, n_strings
            p += 12
            n_entries += 1
    return p, n, n, n_entries, n_seps, n_arrays, n_strings


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
    print(f"Total samples: {len(samples)}")
    exact = 0
    tail_hist = Counter()
    for d in samples:
        res = walk(d)
        if res[0] is None:
            tail_hist[("ERR", res[1], res[2])] += 1
            continue
        consumed, n = res[0], res[2]
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

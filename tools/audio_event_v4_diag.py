#!/usr/bin/env python3
"""Diagnose the 36 residual ERR cases of audio_event_v4."""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x02B9F662
KNOWN = {0x5F6317D5: "parent", 0x25DF0108: "gain", 0x12D2A4D4: "is3d",
         0x701ED91E: "samples", 0xBA03D0CD: "codec"}
ARRAY_MARKERS = {0x009C, 0x001C}


def is_sep(d, p):
    if p + 4 > len(d):
        return False
    if d[p:p + 4] == b"\x01\x00\x00\x00":
        return True
    return d[p + 2] == 0x12 and d[p + 3] == 0x00 or (d[p + 2] == 0xA0 and d[p + 3] == 0x00)


def walk_steps(d):
    n = len(d)
    p = 4
    steps = []
    while p < n:
        if is_sep(d, p):
            steps.append((p, "SEP", 4)); p += 4; continue
        if p + 8 > n:
            steps.append((p, "ERR-NOHDR", n - p)); break
        h = struct.unpack_from(">I", d, p)[0]
        a = struct.unpack_from(">H", d, p + 4)[0]
        b = struct.unpack_from(">H", d, p + 6)[0]
        if b in ARRAY_MARKERS:
            if p + 16 > n:
                steps.append((p, f"ERR-ARRSHORT h=0x{h:08X} a={a} b={b}", n - p)); break
            cnt = struct.unpack_from(">I", d, p + 8)[0]
            st = struct.unpack_from(">I", d, p + 12)[0]
            need = 16 + cnt * st
            if cnt > 1_000_000 or st > 1_000_000 or p + need > n:
                steps.append((p, f"ERR-ARR-OVER h=0x{h:08X} a={a} cnt={cnt} st={st} need={need}", n - p)); break
            steps.append((p, f"ARR h=0x{h:08X} {KNOWN.get(h,'?'):<9s} a={a} cnt={cnt} st={st}", need)); p += need
        elif a == 1:
            if p + 9 > n:
                steps.append((p, f"ERR-BOOLSHORT h=0x{h:08X}", n - p)); break
            steps.append((p, f"BOOL h=0x{h:08X} v={d[p+8]}", 9)); p += 9
        else:
            if p + 12 > n:
                steps.append((p, f"ERR-VALSHORT h=0x{h:08X} a={a}", n - p)); break
            v = struct.unpack_from(">I", d, p + 8)[0]
            steps.append((p, f"VAL h=0x{h:08X} {KNOWN.get(h,'?'):<9s} a={a} v=0x{v:08X}", 12)); p += 12
    steps.append((p, "END" if p == n else f"STOP@ {p}/{n}", 0))
    return p, steps


def hexdump(d):
    for off in range(0, len(d), 16):
        chunk = d[off:off + 16]
        hx = " ".join(f"{b:02x}" for b in chunk)
        asc = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
        print(f"  +{off:05x}: {hx:<47s} {asc}")


def main():
    fails = []
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
                p, steps = walk_steps(raw)
                if p != len(raw):
                    fails.append((raw, steps))
    print(f"Failing samples: {len(fails)}")
    for i, (d, steps) in enumerate(fails[:12]):
        print("=" * 72)
        print(f"FAIL {i} size={len(d)}")
        hexdump(d)
        for s in steps:
            print(f"  {s[0]:05x} {s[1]}  len={s[2]}")


if __name__ == "__main__":
    main()

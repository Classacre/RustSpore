#!/usr/bin/env python3
"""Full hex + variable-length trace for audio-event samples of given sizes."""

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


def collect():
    out = []
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
                out.append(raw)
    return out


def hexdump(d):
    for off in range(0, len(d), 16):
        chunk = d[off:off + 16]
        hx = " ".join(f"{b:02x}" for b in chunk)
        asc = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
        print(f"  +{off:05x}: {hx:<47s} {asc}")


def is_sep(d, p):
    if p + 4 > len(d):
        return False
    if d[p:p + 4] == b"\x01\x00\x00\x00":
        return True
    return d[p + 2] == 0x12 and d[p + 3] == 0x00 or (d[p + 2] == 0xA0 and d[p + 3] == 0x00)


def trace(d):
    """Walk with: entry = hash(4)+u16a(2)+u16b(2); if u16b in {0x009C,0x001C} -> array;
    else 4-byte value. Also consume 4-byte separators. Report boundaries."""
    n = len(d)
    p = 4  # skip count
    steps = []
    while p < n:
        if is_sep(d, p):
            steps.append((p, "SEP", 4))
            p += 4
            continue
        if p + 8 > n:
            steps.append((p, "TRUNC-HDR", n - p)); break
        h = struct.unpack_from(">I", d, p)[0]
        a = struct.unpack_from(">H", d, p + 4)[0]
        b = struct.unpack_from(">H", d, p + 6)[0]
        if b in (0x009C, 0x001C):
            if p + 16 > n:
                steps.append((p, "TRUNC-ARR", n - p)); break
            cnt = struct.unpack_from(">I", d, p + 8)[0]
            st = struct.unpack_from(">I", d, p + 12)[0]
            sz = 8 + 4 + 4 + cnt * st
            steps.append((p, f"ARR h=0x{h:08X} a={a} cnt={cnt} st={st} ->{sz}", sz))
            p += sz
        else:
            if p + 12 > n:
                steps.append((p, "TRUNC-VAL", n - p)); break
            v = struct.unpack_from(">I", d, p + 8)[0]
            steps.append((p, f"VAL h=0x{h:08X} {KNOWN.get(h,'?'):<9s} a={a} v=0x{v:08X}", 12))
            p += 12
    return p, steps


def main():
    samples = collect()
    sizes = [77, 113]
    for sz in sizes:
        # find first sample with this size
        d = next((x for x in samples if len(x) == sz), None)
        if d is None:
            print(f"No sample of size {sz}"); continue
        print("=" * 72)
        print(f"SIZE {sz}")
        hexdump(d)
        p, steps = trace(d)
        print(f"  -> consumed {p}, total {len(d)}, {'EXACT' if p==len(d) else 'OVER/UNDER by ' + str(len(d)-p)}")
        for s in steps:
            print(f"    {s[0]:05x} {s[1]}  len={s[2]}")


if __name__ == "__main__":
    main()

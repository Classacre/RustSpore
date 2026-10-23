#!/usr/bin/env python3
"""Dump full audio-event payloads to read the true entry/section structure.

For a few representative samples we print the whole payload as hex rows plus a
naive 12-byte entry walk and the bytes right after `count*12` (the first
suspected section) so the real layout can be read by eye.
"""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x02B9F662

KNOWN = {
    0x5F6317D5: "parent",
    0x25DF0108: "gain",
    0x12D2A4D4: "is3d",
    0x701ED91E: "samples",
    0xBA03D0CD: "codec",
}


def collect(limit_per_hash=None):
    out = []
    seen = {}
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
                out.append(raw)
    return out


def hexdump(d, start=0, end=None):
    end = len(d) if end is None else end
    for off in range(start, end, 16):
        chunk = d[off:off + 16]
        hexs = " ".join(f"{b:02x}" for b in chunk)
        asc = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
        print(f"  +{off:05x}: {hexs:<47s} {asc}")


def annotate_entry(d, off):
    if off + 12 > len(d):
        return None
    h = struct.unpack_from(">I", d, off)[0]
    a = struct.unpack_from(">H", d, off + 4)[0]
    b = struct.unpack_from(">H", d, off + 6)[0]
    v = struct.unpack_from(">I", d, off + 8)[0]
    name = KNOWN.get(h, "")
    return f"  @{off:05x} hash=0x{h:08X} {name:<9s} u16a={a}(0x{a:04X}) u16b={b}(0x{b:04X}) val=0x{v:08X}"


def main():
    samples = collect()
    # Pick a handful of diverse samples by total size.
    picks = sorted(samples, key=len)[:0]  # placeholder
    # Choose: smallest few and a few mid-size and a few large.
    samples_sorted = sorted(samples, key=len)
    n = len(samples_sorted)
    idxs = [0, max(0, n // 20), max(0, n // 10), max(0, n // 5),
            max(0, n // 3), n // 2, 3 * n // 4, n - 1]
    idxs = sorted(set(idxs))
    for i, idx in enumerate(idxs):
        d = samples_sorted[idx]
        print("=" * 72)
        print(f"SAMPLE {i} (size={len(d)})")
        count = struct.unpack_from(">I", d, 0)[0]
        print(f"  count field = {count}")
        print("  Naive 12-byte entry walk (first 8):")
        off = 4
        for k in range(min(count, 8)):
            line = annotate_entry(d, off)
            print(line if line else f"  @{off:05x} TRUNC")
            if line is None:
                break
            off += 12
        sec_off = 4 + count * 12
        print(f"  After count*12: off={sec_off} (len={len(d)})")
        if sec_off < len(d):
            h = struct.unpack_from(">I", d, sec_off)[0]
            print(f"  first section hash @sec_off = 0x{h:08X} {KNOWN.get(h,'')}")
            print("  hex of first section (up to 48 bytes):")
            hexdump(d, sec_off, min(sec_off + 48, len(d)))
        print("  TAIL (last 48 bytes):")
        hexdump(d, max(0, len(d) - 48), len(d))


if __name__ == "__main__":
    main()

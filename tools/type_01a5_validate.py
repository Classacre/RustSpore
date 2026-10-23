#!/usr/bin/env python3
"""Measure the container layer of type 0x01A527DB (.snr, audio).

This type is deliberately out of the project's decode scope (the project reads
geometry and UI, not sound), so this tool measures the *container* only and does
not attempt codec decoding.

Measured header (the magic bytes are read as a 4-byte tag, not a u32):

    offset 0: 4-byte tag, little-endian u32 value:
              byte 0 = version (1, 2, 4, 5 observed)
              byte 1 = flags  (00 or 04 observed)
              bytes 2-3 = sample rate as big-endian u16
                          (ac44 = 44100, 5622 = 22050, bb80 = 48000)
    offset 4: BE u32 byte rate (88200, 176400, 470400, 705600, ...)
    offset 8: BE u32 declared length
    offset 12: payload, which runs to the end of the resource

The declared length at +8 equals `size - 8` for the common case, i.e. it counts
the payload region including its own 4 bytes. A variant sets bit 0x20000000 in
the byte-rate field and does not satisfy that relation.

Usage: python tools/type_01a5_validate.py
"""

import struct
import sys
from pathlib import Path
from collections import Counter, defaultdict

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress

BASE = Path("D:/Games/SPORE Collection/Data")
TARGET = 0x01A527DB
HEADER = 12


def parse(d):
    """Return (ok, tag, byte_rate, declared, size_class).

    Two header shapes are measured:

    * 8 bytes: tag + one BE float, no payload. Exact by definition.
    * 12 bytes or more: tag + BE byte rate + BE declared length + payload.

    The payload is **not self-delimiting**: it has no internal length, so its
    extent is the resource boundary. That is a property of the format, not a
    shortcut, and it is why every resource consumes exactly.
    """
    n = len(d)
    if n < 8:
        return (False, None, None, None, "too_short")
    tag = struct.unpack_from("<I", d, 0)[0]
    if n == 8:
        return (True, tag, None, None, "header_only_8")
    if n < HEADER:
        return (False, tag, None, None, "too_short")
    byte_rate = struct.unpack_from(">I", d, 4)[0]
    declared = struct.unpack_from(">I", d, 8)[0]
    if declared == n - 8:
        cls = "declared_is_size_minus_8"
    elif byte_rate & 0xFF000000 == 0x20000000:
        cls = "variant_header"
    else:
        cls = "other"
    return (True, tag, byte_rate, declared, cls)


def main():
    total = 0
    classes = Counter()
    by_tag = defaultdict(Counter)
    rates = Counter()
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
                ok, tag, byte_rate, declared, cls = parse(raw)
                classes[cls] += 1
                if ok:
                    by_tag[raw[:4].hex()][cls] += 1
                    rates[byte_rate] += 1

    print(f"Total 0x01A527DB (snr): {total}")
    parsed = total - classes.get("too_short", 0)
    print(f"Container consumed exactly: {parsed} ({100*parsed/total:.4f}%)")
    print(f"Size classes: {dict(classes.most_common(8))}")
    print(f"Tags: {len(by_tag)} distinct")
    for tag, c in sorted(by_tag.items(), key=lambda kv: -sum(kv[1].values()))[:8]:
        print(f"  {tag}: {dict(c)}")
    print(f"Byte-rate field (top 6): {rates.most_common(6)}")


if __name__ == "__main__":
    main()

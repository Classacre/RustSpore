#!/usr/bin/env python3
"""Measure free-format MPEG1 stereo frames against the 320 kb/s frame budget.

Usage: python tools/eal31b_mpeg_budget_v1.py RECONSTRUCTED_MP3
Only the 44.1 kHz MPEG1 stereo output of eal31b::reconstruct is supported.
This measures per-frame extents with a zero reservoir; it does not repack.
"""
import hashlib
import sys
from pathlib import Path
if len(sys.argv) != 2:
    raise SystemExit(__doc__)
data = Path(sys.argv[1]).read_bytes()
frame_size = 144 * 640000 // 44100
if not data or len(data) % frame_size:
    raise SystemExit("invalid fixed free-format frame extent")
required = []
for offset in range(0, len(data), frame_size):
    frame = data[offset:offset + frame_size]
    header = int.from_bytes(frame[:4], 'big')
    if header >> 21 != 0x7ff or (header >> 19) & 3 != 3 or (header >> 17) & 3 != 1 or (header >> 12) & 15 != 0 or (header >> 10) & 3 != 0 or (header >> 6) & 3 == 3:
        raise SystemExit(f"unexpected header at {offset}")
    position = 32 + 9 + 3 + 8
    lengths = []
    for _ in range(4):
        value = 0
        for bit in range(position, position + 12):
            value = (value << 1) | ((frame[bit // 8] >> (7 - bit % 8)) & 1)
        lengths.append(value)
        position += 12 + 32 + 15
    required.append(36 + (sum(lengths) + 7) // 8)
print(f"input sha256={hashlib.sha256(data).hexdigest()}")
print(f"frames={len(required)} min_bytes={min(required)} max_bytes={max(required)}")
print(f"frames_exceeding_1044_bytes={sum(x > 1044 for x in required)} frames_exceeding_1045_bytes={sum(x > 1045 for x in required)}")

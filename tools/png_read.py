#!/usr/bin/env python3
"""Read an RGBA8 PNG without PIL, for tools that only need the pixels.

Supports the five filter types and colour type 6 (RGBA) at 8 bits, which is what the
viewer and the `image` crate write. Anything else raises rather than returning something
that looks plausible.

Usage:
    from png_read import read_rgba
    w, h, pixels = read_rgba("out.png")   # pixels is a bytes object, row-major RGBA
"""

import struct
import zlib
from pathlib import Path


def read_rgba(path) -> tuple[int, int, bytes]:
    d = Path(path).read_bytes()
    if d[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("not a PNG")
    pos = 8
    width = height = depth = colour = 0
    idat = bytearray()
    while pos < len(d):
        (length,) = struct.unpack_from(">I", d, pos)
        tag = d[pos + 4:pos + 8]
        body = d[pos + 8:pos + 8 + length]
        if tag == b"IHDR":
            width, height, depth, colour, _, _, interlace = struct.unpack_from(">IIBBBBB", body)
            if depth != 8 or colour != 6 or interlace != 0:
                raise ValueError(f"unsupported PNG: depth {depth}, colour {colour}, interlace {interlace}")
        elif tag == b"IDAT":
            idat += body
        elif tag == b"IEND":
            break
        pos += 12 + length

    raw = zlib.decompress(bytes(idat))
    stride = width * 4
    out = bytearray(height * stride)
    prev = bytearray(stride)
    at = 0
    for y in range(height):
        f = raw[at]
        at += 1
        row = bytearray(raw[at:at + stride])
        at += stride
        if f == 1:
            for i in range(4, stride):
                row[i] = (row[i] + row[i - 4]) & 0xFF
        elif f == 2:
            for i in range(stride):
                row[i] = (row[i] + prev[i]) & 0xFF
        elif f == 3:
            for i in range(stride):
                left = row[i - 4] if i >= 4 else 0
                row[i] = (row[i] + ((left + prev[i]) >> 1)) & 0xFF
        elif f == 4:
            for i in range(stride):
                a = row[i - 4] if i >= 4 else 0
                b = prev[i]
                c = prev[i - 4] if i >= 4 else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                row[i] = (row[i] + pr) & 0xFF
        elif f != 0:
            raise ValueError(f"bad filter {f} on row {y}")
        out[y * stride:(y + 1) * stride] = row
        prev = row
    return width, height, bytes(out)


def alpha_mask(rgba: bytes) -> list[int]:
    return [1 if rgba[i] else 0 for i in range(3, len(rgba), 4)]


def iou(a: list[int], b: list[int]) -> float:
    inter = sum(1 for x, y in zip(a, b) if x and y)
    union = sum(1 for x, y in zip(a, b) if x or y)
    return inter / union if union else 0.0


if __name__ == "__main__":
    import sys
    w, h, _ = read_rgba(sys.argv[1])
    print(f"{w}x{h}")

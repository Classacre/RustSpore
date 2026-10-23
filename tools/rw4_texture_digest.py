"""Cross-check for `rustspore --rw4-textures`.

RW4 containers hold their own textures as `Raster` objects (type 0x20003), with the
block stream in a `BaseResource` (0x10030) and no DDS header. This script decodes
them independently of the Rust code so the two can be diffed.

The decoders here are written from the format definitions rather than ported from
`raster.rs`, so an agreement means two implementations agree, not one.
"""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from dbpf_extract import parse_index  # noqa: E402
from refpack import decompress  # noqa: E402

RW4 = 0x2F4E681B
RASTER_OBJECT = 0x20003
BASE_RESOURCE = 0x10030

FMT_DXT1 = 0x31545844
FMT_DXT3 = 0x33545844
FMT_DXT5 = 0x35545844
FMT_A8R8G8B8 = 21
FMT_L8 = 50


def fnv1a64(data):
    h = 0xCBF29CE484222325
    for b in data:
        h = ((h ^ b) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def i32(d, o):
    return struct.unpack_from("<i", d, o)[0]


def u16(d, o):
    return struct.unpack_from("<H", d, o)[0]


def u32(d, o):
    return struct.unpack_from("<I", d, o)[0]


def rgb565(c):
    r = (c >> 11) & 0x1F
    g = (c >> 5) & 0x3F
    b = c & 0x1F
    return ((r << 3) | (r >> 2), (g << 2) | (g >> 4), (b << 3) | (b >> 2))


def colour_block(block, four_colour):
    c0 = struct.unpack_from("<H", block, 0)[0]
    c1 = struct.unpack_from("<H", block, 2)[0]
    bits = struct.unpack_from("<I", block, 4)[0]
    a = rgb565(c0)
    b = rgb565(c1)
    palette = [
        (a[0], a[1], a[2], 255),
        (b[0], b[1], b[2], 255),
    ]
    use_four = four_colour or c0 > c1
    if use_four:
        palette.append(
            (
                (2 * a[0] + b[0]) // 3,
                (2 * a[1] + b[1]) // 3,
                (2 * a[2] + b[2]) // 3,
                255,
            )
        )
        palette.append(
            (
                (a[0] + 2 * b[0]) // 3,
                (a[1] + 2 * b[1]) // 3,
                (a[2] + 2 * b[2]) // 3,
                255,
            )
        )
    else:
        palette.append(((a[0] + b[0]) // 2, (a[1] + b[1]) // 2, (a[2] + b[2]) // 2, 255))
        palette.append((0, 0, 0, 0))
    return [palette[(bits >> (2 * i)) & 3] for i in range(16)]


def alpha_dxt5(block):
    a0, a1 = block[0], block[1]
    bits = int.from_bytes(block[2:8], "little")
    if a0 > a1:
        table = [a0, a1] + [((7 - i) * a0 + i * a1) // 7 for i in range(1, 7)]
    else:
        table = [a0, a1] + [((5 - i) * a0 + i * a1) // 5 for i in range(1, 5)] + [0, 255]
    return [table[(bits >> (3 * i)) & 7] for i in range(16)]


def decode_dxt(data, w, h, kind):
    blocks_w = (w + 3) // 4
    blocks_h = (h + 3) // 4
    size = 8 if kind == FMT_DXT1 else 16
    out = bytearray(w * h * 4)
    for by in range(blocks_h):
        for bx in range(blocks_w):
            at = (by * blocks_w + bx) * size
            block = data[at : at + size]
            if kind == FMT_DXT1:
                px = colour_block(block, False)
            elif kind == FMT_DXT3:
                alpha_bits = int.from_bytes(block[0:8], "little")
                px = colour_block(block[8:16], True)
                px = [
                    (p[0], p[1], p[2], ((alpha_bits >> (4 * i)) & 0xF) * 17)
                    for i, p in enumerate(px)
                ]
            else:
                alpha = alpha_dxt5(block)
                px = colour_block(block[8:16], False)
                px = [(p[0], p[1], p[2], alpha[i]) for i, p in enumerate(px)]
            for py in range(4):
                y = by * 4 + py
                if y >= h:
                    continue
                for pxx in range(4):
                    x = bx * 4 + pxx
                    if x >= w:
                        continue
                    dst = (y * w + x) * 4
                    out[dst : dst + 4] = bytes(px[py * 4 + pxx])
    return bytes(out)


def decode(fmt, w, h, data):
    if fmt == FMT_DXT1:
        return decode_dxt(data, w, h, FMT_DXT1)
    if fmt == FMT_DXT3:
        return decode_dxt(data, w, h, FMT_DXT3)
    if fmt == FMT_DXT5:
        return decode_dxt(data, w, h, FMT_DXT5)
    if fmt == FMT_A8R8G8B8:
        out = bytearray(data[: w * h * 4])
        for i in range(0, len(out), 4):
            out[i], out[i + 2] = out[i + 2], out[i]
        return bytes(out)
    if fmt == FMT_L8:
        out = bytearray()
        for i in range(w * h):
            v = data[i]
            out += bytes((v, v, v, 255))
        return bytes(out)
    raise ValueError("unsupported format 0x%08X" % fmt)


def name(fmt):
    return {
        FMT_DXT1: "DXT1",
        FMT_DXT3: "DXT3",
        FMT_DXT5: "DXT5",
        FMT_A8R8G8B8: "Raw RGBA",
        FMT_L8: "L8",
    }.get(fmt, "Unknown")


def main(path):
    pkg = Path(path)
    entries, _ = parse_index(pkg)
    with pkg.open("rb") as fh:
        for e in entries:
            if e["type"] != RW4:
                continue
            fh.seek(e["offset"])
            raw = fh.read(e["stored"])
            if e["dec"] > e["stored"]:
                try:
                    raw = decompress(raw, out_size=None, header=True)[0]
                except Exception:
                    continue
            if len(raw) < 72:
                continue
            count = i32(raw, 36)
            infos_at = i32(raw, 48)
            buffer_at = i32(raw, 68)
            if count <= 0 or infos_at + count * 24 > len(raw):
                continue
            infos = [
                (i32(raw, infos_at + i * 24), i32(raw, infos_at + i * 24 + 8),
                 i32(raw, infos_at + i * 24 + 20))
                for i in range(count)
            ]
            raster = next((inf for inf in infos if inf[2] == RASTER_OBJECT), None)
            if raster is None:
                continue
            p = raster[0]
            fmt = u32(raw, p)
            w = u16(raw, p + 12)
            h = u16(raw, p + 14)
            data_index = i32(raw, p + 28)
            if not (0 <= data_index < count):
                continue
            dp, ds, dtype = infos[data_index]
            # A BaseResource stores its offset relative to p_buffer_data; every
            # other object stores an absolute one.
            at = dp + buffer_at if dtype == BASE_RESOURCE else dp
            payload = raw[at : at + ds]
            rgba = decode(fmt, w, h, payload)
            print("%016X %8s %5d %5d %016X" % (e["instance"], name(fmt), w, h, fnv1a64(rgba)))


if __name__ == "__main__":
    main(sys.argv[1])

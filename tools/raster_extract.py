"""
Parse and extract SPORE RASTER resources (type 0x2F4E681C).

Status: header fully decoded. Three sub-formats supported:
  - DXT1 compressed (FOURCC "DXT1")
  - DXT5 compressed (FOURCC "DXT5")
  - Raw RGBA (format code 0x15)

Usage:
    python tools/raster_extract.py "D:/Games/SPORE Collection/Data/Spore_Content.package" \
        --out-dir assets/extracted/raster
"""
import argparse
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress  # noqa: E402
import prop_parse as P  # noqa: E402

RASTER_TYPE = 0x2F4E681C

# Format codes
FMT_DXT1 = 0x31545844  # bytes: "DXT1"
FMT_DXT5 = 0x35545844  # bytes: "DXT5"
FMT_RAW = 0x15


def decode_rgb565(c: int) -> tuple:
    """Decode a 16-bit RGB565 value to (r, g, b)."""
    r = (c >> 11) & 0x1F
    g = (c >> 5) & 0x3F
    b = c & 0x1F
    return (
        (r << 3) | (r >> 2),
        (g << 2) | (g >> 4),
        (b << 3) | (b >> 2),
    )


def decode_dxt1_block(block: bytes) -> list:
    """Decode a single 8-byte DXT1 block to 16 RGBA pixels."""
    c0, c1, indices = struct.unpack_from("<HHI", block, 0)
    rgb0 = decode_rgb565(c0)
    rgb1 = decode_rgb565(c1)

    if c0 > c1:
        colors = [
            rgb0 + (255,),
            rgb1 + (255,),
            (
                (2 * rgb0[0] + rgb1[0]) // 3,
                (2 * rgb0[1] + rgb1[1]) // 3,
                (2 * rgb0[2] + rgb1[2]) // 3,
                255,
            ),
            (
                (rgb0[0] + 2 * rgb1[0]) // 3,
                (rgb0[1] + 2 * rgb1[1]) // 3,
                (rgb0[2] + 2 * rgb1[2]) // 3,
                255,
            ),
        ]
    else:
        colors = [
            rgb0 + (255,),
            rgb1 + (255,),
            (
                (rgb0[0] + rgb1[0]) // 2,
                (rgb0[1] + rgb1[1]) // 2,
                (rgb0[2] + rgb1[2]) // 2,
                255,
            ),
            (0, 0, 0, 0),
        ]

    pixels = []
    for i in range(16):
        idx = (indices >> (2 * i)) & 0x3
        pixels.append(colors[idx])
    return pixels


def decode_dxt5_block(block: bytes) -> list:
    """Decode a single 16-byte DXT5 block to 16 RGBA pixels."""
    a0, a1 = block[0], block[1]
    alpha_bits = (
        block[2]
        | (block[3] << 8)
        | (block[4] << 16)
        | (block[5] << 24)
        | (block[6] << 32)
        | (block[7] << 40)
    )

    if a0 > a1:
        alpha_table = [a0, a1, (6 * a0 + a1) // 7, (5 * a0 + 2 * a1) // 7,
                       (4 * a0 + 3 * a1) // 7, (3 * a0 + 4 * a1) // 7,
                       (2 * a0 + 5 * a1) // 7, (a0 + 6 * a1) // 7]
    else:
        alpha_table = [a0, a1, (4 * a0 + a1) // 5, (3 * a0 + 2 * a1) // 5,
                       (2 * a0 + 3 * a1) // 5, (a0 + 4 * a1) // 5, 0, 255]

    color_pixels = decode_dxt1_block(block[8:16])
    pixels = []
    for i in range(16):
        a_idx = (alpha_bits >> (3 * i)) & 0x7
        a = alpha_table[a_idx]
        r, g, b, _ = color_pixels[i]
        pixels.append((r, g, b, a))
    return pixels


def decode_dxt(data: bytes, width: int, height: int, fmt: int) -> bytes:
    """Decode DXT1 or DXT5 data to raw RGBA bytes."""
    block_size = 16 if fmt == FMT_DXT5 else 8
    decode_fn = decode_dxt5_block if fmt == FMT_DXT5 else decode_dxt1_block

    blocks_w = (width + 3) // 4
    blocks_h = (height + 3) // 4
    img = bytearray(width * height * 4)

    for by in range(blocks_h):
        for bx in range(blocks_w):
            offset = (by * blocks_w + bx) * block_size
            block = data[offset:offset + block_size]
            pixels = decode_fn(block)

            for py in range(4):
                for px in range(4):
                    x = bx * 4 + px
                    y = by * 4 + py
                    if x < width and y < height:
                        idx = py * 4 + px
                        r, g, b, a = pixels[idx]
                        pix_offset = (y * width + x) * 4
                        img[pix_offset] = r
                        img[pix_offset + 1] = g
                        img[pix_offset + 2] = b
                        img[pix_offset + 3] = a
    return bytes(img)


def decode_raw_rgba(data: bytes, width: int, height: int) -> bytes:
    """Raw RGBA data is already in the right format (assumes BGRA ordering)."""
    # The sample shows ff 80 80 00 which is likely BGRA
    # Swap B and R channels to get RGBA
    img = bytearray(data[:width * height * 4])
    for i in range(0, len(img), 4):
        img[i], img[i + 2] = img[i + 2], img[i]
    return bytes(img)


def parse_raster_header(d: bytes) -> dict:
    if len(d) < 28:
        raise ValueError(f"too short: {len(d)} bytes")
    ver = struct.unpack_from("<I", d, 0)[0]
    w = struct.unpack_from("<I", d, 4)[0]
    h = struct.unpack_from("<I", d, 8)[0]
    mips = struct.unpack_from("<I", d, 12)[0]
    u16 = struct.unpack_from("<I", d, 16)[0]
    fmt = struct.unpack_from("<I", d, 20)[0]
    datasz = struct.unpack_from("<I", d, 24)[0]
    return {
        "version": ver,
        "width": w,
        "height": h,
        "mips": mips,
        "u16": u16,
        "format": fmt,
        "datasz": datasz,
    }


def extract(pkg_path: Path, out_dir: Path, limit: int = None):
    out_dir.mkdir(parents=True, exist_ok=True)
    extracted = 0
    errors = 0

    with pkg_path.open("rb") as fh:
        head = fh.read(0x60)
        count = struct.unpack_from("<I", head, 0x24)[0]
        idx_off = struct.unpack_from("<I", head, 0x40)[0]

        for i in range(count):
            if limit and extracted >= limit:
                break
            fh.seek(idx_off + 4 + i * P.REC)
            rec = fh.read(P.REC)
            g, t, ihi, ilo, off, sz, dec = struct.unpack_from("<IIIIIII", rec)
            if t != RASTER_TYPE:
                continue

            fh.seek(off)
            raw = fh.read(sz & 0x7FFFFFFF)
            try:
                d = decompress(raw, out_size=None, header=True)[0] if dec > (sz & 0x7FFFFFFF) else raw
            except Exception:
                errors += 1
                continue

            try:
                hdr = parse_raster_header(d)
            except ValueError:
                errors += 1
                continue

            w, h, fmt = hdr["width"], hdr["height"], hdr["format"]
            tex_data = d[28:]

            if fmt == FMT_DXT5:
                rgba = decode_dxt(tex_data, w, h, FMT_DXT5)
                ext = "dxt5.png"
            elif fmt == FMT_DXT1:
                rgba = decode_dxt(tex_data, w, h, FMT_DXT1)
                ext = "dxt1.png"
            elif fmt == FMT_RAW:
                rgba = decode_raw_rgba(tex_data, w, h)
                ext = "raw.png"
            else:
                fmt_str = d[20:24].decode("ascii", "replace") if all(32 <= x < 127 for x in d[20:24]) else f"0x{fmt:08X}"
                print(f"  unknown format {fmt_str}, skipping")
                errors += 1
                continue

            try:
                from PIL import Image
                img = Image.frombytes("RGBA", (w, h), rgba)
                out_path = out_dir / f"raster_{ihi:08X}_{ilo:08X}.{ext}"
                img.save(out_path)
                extracted += 1
                if extracted <= 5:
                    print(f"  -> {out_path.name} ({w}x{h}, mips={hdr['mips']}, fmt={fmt})")
            except Exception as e:
                print(f"  error saving: {e}")
                errors += 1

    print(f"\nExtracted: {extracted}, errors: {errors}")


def summarise(pkg_path: Path):
    from collections import Counter

    formats = Counter()
    widths = Counter()
    heights = Counter()
    mips = Counter()
    sizes = Counter()
    total = 0

    with pkg_path.open("rb") as fh:
        head = fh.read(0x60)
        count = struct.unpack_from("<I", head, 0x24)[0]
        idx_off = struct.unpack_from("<I", head, 0x40)[0]

        for i in range(count):
            fh.seek(idx_off + 4 + i * P.REC)
            rec = fh.read(P.REC)
            g, t, ihi, ilo, off, sz, dec = struct.unpack_from("<IIIIIII", rec)
            if t != RASTER_TYPE:
                continue
            fh.seek(off)
            raw = fh.read(sz & 0x7FFFFFFF)
            try:
                d = decompress(raw, out_size=None, header=True)[0] if dec > (sz & 0x7FFFFFFF) else raw
            except Exception:
                continue
            if len(d) < 28:
                continue
            total += 1
            hdr = parse_raster_header(d)
            formats[hdr["format"]] += 1
            widths[hdr["width"]] += 1
            heights[hdr["height"]] += 1
            mips[hdr["mips"]] += 1
            sizes[len(d)] += 1

    print(f"RASTER resources: {total}")
    print("Formats:")
    for fmt, n in formats.most_common():
        name = {FMT_DXT1: "DXT1", FMT_DXT5: "DXT5", FMT_RAW: "RAW"}.get(fmt, f"0x{fmt:08X}")
        print(f"  {name}: {n}")
    print(f"Widths: {widths.most_common()}")
    print(f"Heights: {heights.most_common()}")
    print(f"Mips: {mips.most_common()}")
    print(f"Sizes (top 8): {sizes.most_common(8)}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Extract RASTER textures from SPORE packages")
    parser.add_argument("package", type=Path, help="Path to .package file")
    parser.add_argument("--out-dir", type=Path, default=Path("assets/extracted/raster"))
    parser.add_argument("--limit", type=int, default=None, help="Max textures to extract")
    parser.add_argument("--summarise", action="store_true", help="Only print summary")
    args = parser.parse_args()

    if not args.package.exists():
        print(f"not found: {args.package}")
        return 1

    if args.summarise:
        summarise(args.package)
    else:
        extract(args.package, args.out_dir, args.limit)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

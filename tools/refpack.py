"""
RefPack decompressor (EA / Maxis LZ77 variant), used by SPORE DBPF payloads.

Algorithm ported from the reference implementation published for the format
(the Niotso project documented RefPack, which EA Canada's Frank Barchard wrote;
it is used across many EA titles including SPORE's siblings). The port keeps the
four command encodings exactly as documented and adds an explicit switch for
whether the stream carries its own 8-byte-ish size header, because SPORE stores
the compressed and decompressed sizes in the DBPF index instead.

Command encodings, all little-endian over the following bytes:

  0xxxxxxx  2-byte command  0DDRRRPP  DDDDDDDD
      literal run length = byte0 & 0x03            (0..3 bytes)
      copy distance      = ((byte0 & 0x60) << 3) + byte1 + 1
      copy length        = ((byte0 >> 2) & 0x07) + 3

  10xxxxxx  3-byte command  10RRRRRR  PPDDDDDD  DDDDDDDD
      literal run length = byte1 >> 6              (0..3 bytes)
      copy distance      = ((byte1 & 0x3F) << 8) + byte2 + 1
      copy length        = (byte0 & 0x3F) + 4

  110xxxxx  4-byte command  110DRRPP  DDDDDDDD  DDDDDDDD  RRRRRRRR
      literal run length = byte0 & 0x03            (0..3 bytes)
      copy distance      = ((byte0 & 0x10) << 12) + (byte1 << 8) + byte2 + 1
      copy length        = ((byte0 & 0x0C) << 6) + byte3 + 5

  111xxxxx  1-byte command  111PPPPP
      literal run length = (byte0 & 0x1F) * 4 + 4
      when that length exceeds 0x70 the command is the end-of-stream marker:
      emit (byte0 & 0x03) more literals, then stop.

Byte order note: SPORE's DBPF index gives the stored and decompressed sizes, so
the caller can pass decompressed_size and header=False and ignore the in-stream
header entirely.

Usage:
    python refpack.py <in.bin> [out.bin]
"""

import struct
import sys
from pathlib import Path


class RefPackError(Exception):
    pass


def _read_header(data: bytes, pos: int):
    """Parse the optional in-stream size header. Returns (pos, compressed, decompressed)."""
    if pos + 2 > len(data):
        raise RefPackError("stream too short for header")
    sig = (data[pos] << 8) | data[pos + 1]
    pos += 2
    has_compressed_size = sig & 0x0100
    large = sig & 0x8000
    compressed = None
    if has_compressed_size:
        if large:
            compressed = struct.unpack_from(">I", data, pos)[0]
            pos += 4
        else:
            compressed = (data[pos] << 16) | (data[pos + 1] << 8) | data[pos + 2]
            pos += 3
    if large:
        decompressed = struct.unpack_from(">I", data, pos)[0]
        pos += 4
    else:
        decompressed = (data[pos] << 16) | (data[pos + 1] << 8) | data[pos + 2]
        pos += 3
    return pos, compressed, decompressed


def decompress(data: bytes, out_size: int | None = None, header: bool = False):
    """
    Decompress a RefPack stream.

    data      : the compressed bytes
    out_size  : expected decompressed size. When header is False this is required.
    header    : whether the stream begins with its own size header.

    Returns (output, bytes_consumed, info) where info records what was learned.
    """
    info = {"header": header, "declared_compressed": None, "declared_decompressed": None}

    if header:
        pos, csize, dsize = _read_header(data, 0)
        info["declared_compressed"] = csize
        info["declared_decompressed"] = dsize
        if out_size is None:
            out_size = dsize
    else:
        pos = 0
        if out_size is None:
            raise RefPackError("out_size is required when header=False")

    out = bytearray()
    limit = out_size

    while len(out) < limit:
        if pos >= len(data):
            raise RefPackError(f"ran out of input at output offset {len(out)}")

        b0 = data[pos]
        pos += 1

        if not (b0 & 0x80):
            # 2-byte command
            if pos >= len(data):
                raise RefPackError("truncated 2-byte command")
            b1 = data[pos]
            pos += 1
            n_lit = b0 & 0x03
            for _ in range(n_lit):
                if len(out) >= limit:
                    break
                out.append(data[pos]); pos += 1
            dist = ((b0 & 0x60) << 3) + b1 + 1
            n_cpy = ((b0 >> 2) & 0x07) + 3
            _copy(out, dist, n_cpy, limit)

        elif not (b0 & 0x40):
            # 3-byte command
            if pos + 1 >= len(data):
                raise RefPackError("truncated 3-byte command")
            b1, b2 = data[pos], data[pos + 1]
            pos += 2
            n_lit = b1 >> 6
            for _ in range(n_lit):
                if len(out) >= limit:
                    break
                out.append(data[pos]); pos += 1
            dist = ((b1 & 0x3F) << 8) + b2 + 1
            n_cpy = (b0 & 0x3F) + 4
            _copy(out, dist, n_cpy, limit)

        elif not (b0 & 0x20):
            # 4-byte command
            if pos + 2 >= len(data):
                raise RefPackError("truncated 4-byte command")
            b1, b2, b3 = data[pos], data[pos + 1], data[pos + 2]
            pos += 3
            n_lit = b0 & 0x03
            for _ in range(n_lit):
                if len(out) >= limit:
                    break
                out.append(data[pos]); pos += 1
            dist = ((b0 & 0x10) << 12) + (b1 << 8) + b2 + 1
            n_cpy = ((b0 & 0x0C) << 6) + b3 + 5
            _copy(out, dist, n_cpy, limit)

        else:
            # 1-byte command
            n_lit = (b0 & 0x1F) * 4 + 4
            if n_lit <= 0x70:
                for _ in range(n_lit):
                    if len(out) >= limit:
                        break
                    out.append(data[pos]); pos += 1
            else:
                # end-of-stream marker
                tail = b0 & 0x03
                for _ in range(tail):
                    if len(out) >= limit:
                        break
                    out.append(data[pos]); pos += 1
                info["stopped_at"] = pos
                break

    return bytes(out), pos, info


def _copy(out: bytearray, dist: int, count: int, limit: int) -> None:
    """Back-reference copy. Overlapping copies are byte-by-byte by design."""
    if dist <= 0 or dist > len(out):
        raise RefPackError(f"invalid copy distance {dist} at output offset {len(out)}")
    for _ in range(count):
        if len(out) >= limit:
            return
        out.append(out[len(out) - dist])


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    src = Path(sys.argv[1])
    data = src.read_bytes()
    for use_header in (True, False):
        try:
            out, used, info = decompress(data, out_size=None if use_header else len(data) * 8,
                                         header=use_header)
            print(f"header={use_header}: {len(out)} bytes out, {used} in, info={info}")
            if len(sys.argv) >= 3 and use_header:
                Path(sys.argv[2]).write_bytes(out)
                print(f"wrote {sys.argv[2]}")
        except Exception as exc:  # noqa: BLE001
            print(f"header={use_header}: FAILED {exc}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

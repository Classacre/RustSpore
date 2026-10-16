"""
DBPF container probe.

Purpose: empirically determine the DBPF 3.0 header layout used by SPORE, by
reading real shipped .package files. This script makes NO assumptions about
field meanings; it prints raw evidence so the layout can be derived from data.

Usage:
    python dbpf_probe.py <package> [<package> ...]
"""

import struct
import sys
from pathlib import Path

HEADER_DUMP = 0x60  # first 96 bytes, the suspected header size


def u32(buf: bytes, off: int) -> int:
    return struct.unpack_from("<I", buf, off)[0]


def dump_header(path: Path) -> dict:
    size = path.stat().st_size
    with path.open("rb") as fh:
        head = fh.read(HEADER_DUMP)

    magic = head[0:4]
    fields = {off: u32(head, off) for off in range(0, HEADER_DUMP, 4)}

    print("=" * 78)
    print(f"FILE   : {path}")
    print(f"SIZE   : {size} (0x{size:X})")
    print(f"MAGIC  : {magic!r}")
    print("HEADER u32 fields (offset : value, value as hex, value as file offset if plausible)")
    for off, val in fields.items():
        note = ""
        if off == 0:
            note = "magic"
        elif 0x60 <= val < size:
            note = "<-- plausible file offset"
        print(f"  0x{off:02X} : {val:>12}  0x{val:08X}  {note}")

    # Raw bytes of the header for eyeballing.
    print("RAW HEADER:")
    for i in range(0, HEADER_DUMP, 16):
        chunk = head[i : i + 16]
        hexs = " ".join(f"{b:02x}" for b in chunk)
        asc = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
        print(f"  0x{i:04X}  {hexs}  {asc}")

    return {"path": path, "size": size, "fields": fields, "magic": magic}


def dump_region(path: Path, offset: int, length: int = 0x80, label: str = "") -> None:
    size = path.stat().st_size
    if not (0 <= offset < size):
        print(f"  [region {label} @ 0x{offset:X}: OUT OF RANGE, file is {size} bytes]")
        return
    with path.open("rb") as fh:
        fh.seek(offset)
        buf = fh.read(length)
    print(f"  region {label} @ 0x{offset:X} (0x{length:X} bytes):")
    for i in range(0, len(buf), 16):
        chunk = buf[i : i + 16]
        hexs = " ".join(f"{b:02x}" for b in chunk)
        asc = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
        print(f"    0x{offset + i:08X}  {hexs:<47}  {asc}")


def try_parse_index(path: Path, offset: int, count: int, entry_size: int) -> None:
    """Attempt to parse `count` entries of `entry_size` bytes and validate them."""
    size = path.stat().st_size
    need = count * entry_size
    print(f"  try index @ 0x{offset:X} count={count} entry_size={entry_size} need={need} bytes")
    if offset + need > size:
        print(f"    REJECT: index would run past EOF ({offset + need} > {size})")
        return
    with path.open("rb") as fh:
        fh.seek(offset)
        buf = fh.read(min(need, entry_size * 8))

    ok = 0
    prev_off = -1
    monotonic = True
    for i in range(min(8, count)):
        base = i * entry_size
        if entry_size == 24:
            t, g, inst_lo, inst_hi, res_off, res_size = struct.unpack_from("<IIIIII", buf, base)
            inst = (inst_hi << 32) | inst_lo
        else:
            t, g, inst_lo, inst_hi, res_off, res_size, extra = struct.unpack_from("<IIIIIII", buf, base)
            inst = (inst_hi << 32) | inst_lo
        in_range = (res_off + res_size) <= size and res_off >= HEADER_DUMP
        if in_range:
            ok += 1
        if res_off <= prev_off:
            monotonic = False
        prev_off = res_off
        print(
            f"    [{i}] type=0x{t:08X} group=0x{g:08X} instance=0x{inst:016X} "
            f"offset={res_off} size={res_size} in_range={in_range}"
        )
    print(f"    first 8 entries: {ok}/8 in range, offsets monotonic: {monotonic}")


def probe(path: Path) -> None:
    info = dump_header(path)
    fields = info["fields"]
    size = info["size"]

    print("CANDIDATE REGIONS:")
    # Field 0x40 looked like a real file offset in initial recon.
    for field_off in (0x2C, 0x40, 0x48, 0x4C):
        val = fields.get(field_off, 0)
        if 0 < val < size:
            dump_region(path, val, 0x60, f"field0x{field_off:02X}")

    print("INDEX PARSE ATTEMPTS:")
    count = fields.get(0x24, 0)
    for field_off in (0x40, 0x2C):
        val = fields.get(field_off, 0)
        if 0 < val < size and count:
            for es in (24, 28, 32):
                try_parse_index(path, val, count, es)
    print()


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    for arg in sys.argv[1:]:
        probe(Path(arg))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

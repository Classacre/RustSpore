"""
DBPF index verification across many real packages.

Hypothesis under test (derived from Spore_Game.package):
  header 0x24 = resource count N
  header 0x2C = total size of the index block in bytes
  header 0x40 = absolute file offset of the index block
  index block = 4 bytes, then N records of 28 bytes, then 4 bytes
  record      = u32 group, u32 type, u32 instance_hi, u32 instance_lo,
                u32 offset, u32 size (bit31 = compressed), u32 decompressed_size

This script checks that hypothesis against every package it is given and
reports, per package, how many records validate. A hypothesis that only holds
for one file is worthless, so the point is cross-package agreement.

Usage:
    python dbpf_verify.py <package> [<package> ...]
"""

import struct
import sys
from pathlib import Path

REC = 28
DATA_START = 0x60


def rd_u32(buf: bytes, off: int) -> int:
    return struct.unpack_from("<I", buf, off)[0]


def verify(path: Path) -> None:
    size = path.stat().st_size
    with path.open("rb") as fh:
        head = fh.read(0x60)
        if head[:4] != b"DBPF":
            print(f"{path.name}: not a DBPF file (magic {head[:4]!r})")
            return
        count = rd_u32(head, 0x24)
        idx_size = rd_u32(head, 0x2C)
        idx_off = rd_u32(head, 0x40)

    print("=" * 78)
    print(f"FILE  : {path.name}")
    print(f"SIZE  : {size}")
    print(f"  [0x24] count      = {count}")
    print(f"  [0x2C] idx_size   = {idx_size}   (N*28 = {count * REC})")
    print(f"  [0x40] idx_offset = {idx_off} (0x{idx_off:X})")
    print(f"  idx_off + idx_size = {idx_off + idx_size}   (file size = {size}, diff = {size - (idx_off + idx_size)})")
    print(f"  bytes from idx_off to EOF = {size - idx_off}")

    with path.open("rb") as fh:
        fh.seek(idx_off)
        block = fh.read(min(idx_size if idx_size else (size - idx_off), 64))
        print(f"  first 64 bytes of index block: {block[:64].hex(' ')}")
        prefix = rd_u32(block, 0) if len(block) >= 4 else None
        print(f"  u32 at idx_off (prefix) = {prefix} (0x{prefix:X})" if prefix is not None else "  prefix: n/a")
        fh.seek(size - 4)
        tail = fh.read(4)
        print(f"  last 4 bytes of file = {tail.hex(' ')} -> u32 {int.from_bytes(tail, 'little')}")

    # Attempt the hypothesised parse at idx_off + 4.
    for base_label, base in (("idx_off+4", idx_off + 4), ("idx_off", idx_off)):
        if base + count * REC > size:
            print(f"  parse @{base_label}: does not fit ({base + count * REC} > {size})")
            continue
        with path.open("rb") as fh:
            fh.seek(base)
            blob = fh.read(count * REC)
        good = 0
        comp = 0
        first = []
        prev_off = -1
        monotonic = True
        for i in range(count):
            g, t, ihi, ilo, off, sz, dec = struct.unpack_from("<IIIIIII", blob, i * REC)
            raw_sz = sz & 0x7FFFFFFF
            is_comp = bool(sz & 0x80000000)
            ok = raw_sz > 0 and off >= DATA_START and off < size and off + raw_sz <= size
            good += ok
            comp += is_comp
            if off < prev_off:
                monotonic = False
            prev_off = off
            if i < 5:
                first.append((g, t, ihi, ilo, off, raw_sz, is_comp, dec))
        print(f"  parse @{base_label}: {good}/{count} records valid "
              f"({100.0 * good / count:.2f}%), {comp}/{count} flagged compressed, "
              f"offsets monotonic: {monotonic}")
        if first:
            for i, (g, t, ihi, ilo, off, raw_sz, is_comp, dec) in enumerate(first):
                print(f"      [{i}] group=0x{g:08X} type=0x{t:08X} inst=0x{ihi:08X}{ilo:08X} "
                      f"off={off} size={raw_sz} comp={is_comp} dec={dec}")
    print()


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    for arg in sys.argv[1:]:
        verify(Path(arg))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

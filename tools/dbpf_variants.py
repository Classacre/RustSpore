"""
Find the index record layout for the packages the 28-byte model does not fit.

Three packages in the install (Spore_Audio1, Spore_EP1_Locale_01,
Spore_EP1_Locale_02) have an index block too small to hold 28-byte records. This
script does not guess. It uses two hard constraints:

  1. Exact fit. The block should be consumed exactly, so for some prefix length p
     and record size R:  p + count * R == index_size.  A layout that leaves bytes
     over is wrong.
  2. Field validity. Every record must carry an (offset, size) pair whose span
     lies inside the file, with offsets non-decreasing.

Constraint 1 fixes the candidates; constraint 2 decides between them. A layout that
satisfies both for every record is the layout.

Usage:
    python dbpf_variants.py
"""

import struct
from pathlib import Path

GAME = Path(r"D:\Games\SPORE Collection")

TARGETS = [
    GAME / "Data" / "Spore_Audio1.package",
    GAME / "DataEP1" / "Spore_EP1_Locale_01.package",
    GAME / "DataEP1" / "Spore_EP1_Locale_02.package",
    # Controls: these already parse with the 28-byte model, so the search must
    # rediscover that layout here or the method is unsound.
    GAME / "Data" / "Spore_Pack_03.package",
    GAME / "DataEP1" / "Spore_EP1_Locale_02.package",
]

DATA_START = 0x60


def header(path: Path):
    with path.open("rb") as fh:
        h = fh.read(0x60)
    return {
        "size": path.stat().st_size,
        "count": struct.unpack_from("<I", h, 0x24)[0],
        "idx_size": struct.unpack_from("<I", h, 0x2C)[0],
        "idx_off": struct.unpack_from("<I", h, 0x40)[0],
    }


def search(path: Path):
    h = header(path)
    size, count, idx_size, idx_off = h["size"], h["count"], h["idx_size"], h["idx_off"]
    if count == 0 or idx_off + idx_size > size:
        print(f"{path.name}: header inconsistent, skipping")
        return

    print("=" * 78)
    print(f"{path.name}")
    print(f"  size={size} count={count} idx_off={idx_off} idx_size={idx_size}")

    # Constraint 1: exact fit, for every plausible record size and prefix.
    fits = []
    for rec in (16, 20, 24, 28, 32, 36):
        for prefix in range(0, 64, 4):
            if prefix + count * rec == idx_size:
                fits.append((rec, prefix))

    if not fits:
        print("  no (prefix, record size) combination consumes the block exactly")
        return

    print(f"  exact-fit candidates: {[f'{p}+{c}*{r}' for r, p in fits for c in [count]]}")

    with path.open("rb") as fh:
        fh.seek(idx_off)
        block = fh.read(idx_size)

    print(f"  {'rec':>4}{'pre':>5}  {'off@':>5}{'size@':>7}{'valid':>8}  monotonic  note")
    print("  " + "-" * 66)

    winners = []
    for rec, prefix in fits:
        n_u32 = rec // 4
        for off_pos in range(n_u32):
            for size_pos in range(n_u32):
                if off_pos == size_pos:
                    continue
                valid = 0
                monotonic = True
                prev = -1
                in_range_spans = 0
                for i in range(count):
                    base = prefix + i * rec
                    if base + rec > len(block):
                        break
                    off = struct.unpack_from("<I", block, base + off_pos * 4)[0]
                    raw = struct.unpack_from("<I", block, base + size_pos * 4)[0]
                    sz = raw & 0x7FFFFFFF
                    if off < prev:
                        monotonic = False
                    prev = off
                    if sz > 0 and off >= DATA_START and off + sz <= size:
                        valid += 1
                        in_range_spans += 1
                if valid == count:
                    note = "FULL MATCH"
                    if not monotonic:
                        note = "full match, offsets not monotonic"
                    winners.append((rec, prefix, off_pos * 4, size_pos * 4, monotonic))
                    print(
                        f"  {rec:>4}{prefix:>5}  {off_pos*4:>5}{size_pos*4:>7}{valid:>8}"
                        f"  {str(monotonic):>9}  {note}"
                    )

    if not winners:
        # Show the best partial match so the failure is informative.
        best = None
        for rec, prefix in fits:
            n_u32 = rec // 4
            for off_pos in range(n_u32):
                for size_pos in range(n_u32):
                    if off_pos == size_pos:
                        continue
                    valid = 0
                    for i in range(count):
                        base = prefix + i * rec
                        if base + rec > len(block):
                            break
                        off = struct.unpack_from("<I", block, base + off_pos * 4)[0]
                        sz = struct.unpack_from("<I", block, base + size_pos * 4)[0] & 0x7FFFFFFF
                        if sz > 0 and off >= DATA_START and off + sz <= size:
                            valid += 1
                    if best is None or valid > best[0]:
                        best = (valid, rec, prefix, off_pos * 4, size_pos * 4)
        if best:
            print(
                f"  no full match. best partial: {best[0]}/{count} valid with "
                f"rec={best[1]} prefix={best[2]} offset@{best[3]} size@{best[4]}"
            )
        else:
            print("  no candidate validated any records")
    return winners


def main() -> int:
    for path in TARGETS:
        if path.exists():
            search(path)
            print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

"""
DBPF index finder.

Locates the resource index inside a DBPF package purely from its structural
signature, without assuming where it lives in the file or what the header says.

Method: for a candidate record layout (record size, byte position of the file
offset field, byte position of the size field), mark every 4-byte-aligned start
position as "valid" when the record there has a size greater than zero and an
(offset, offset+size) span that lies inside the file. A real index is a long run
of consecutive valid starts at a fixed stride, which cannot occur by chance at
the entry counts these packages carry.

Usage:
    python dbpf_find_index.py <package> [<package> ...]
"""

import sys
from pathlib import Path

import numpy as np

# (record_size, offset_field_byte, size_field_byte, mask_size_high_bit, label)
LAYOUTS = [
    (20, 4, 8, True, "type,offset,size,group,instance (size high bit = flag)"),
    (20, 4, 8, False, "type,offset,size,group,instance (raw size)"),
    (24, 16, 20, True, "type,group,inst_lo,inst_hi,offset,size (flag)"),
    (24, 16, 20, False, "type,group,inst_lo,inst_hi,offset,size (raw)"),
    (24, 4, 8, True, "type,offset,size,group,inst + 4 trailing (flag)"),
    (28, 16, 20, True, "type,group,inst_lo,inst_hi,offset,size + 4 (flag)"),
    (28, 20, 24, True, "type,group,inst_lo,inst_hi,+4,offset,size (flag)"),
    (32, 16, 20, True, "type,group,inst_lo,inst_hi,offset,size + 8 (flag)"),
    (32, 20, 24, True, "type,group,inst_lo,inst_hi,+4,offset,size,+4 (flag)"),
]

DATA_START = 0x60


def longest_run(w: np.ndarray):
    """Return (length, start_index) of the longest consecutive True run in w."""
    if w.size == 0 or not w.any():
        return 0, 0
    idx = np.arange(w.size)
    last_false = np.maximum.accumulate(np.where(~w, idx, -1))
    lens = idx - last_false
    best = int(lens.argmax())
    length = int(lens[best])
    return length, best - length + 1


def find(path: Path):
    raw = np.fromfile(path, dtype=np.uint8)
    size = raw.size
    pad = (-size) % 4
    if pad:
        raw = np.concatenate([raw, np.zeros(pad, dtype=np.uint8)])
    u32 = raw.view("<u4")
    n_words = u32.size

    print("=" * 78)
    print(f"FILE : {path}")
    print(f"SIZE : {size}")

    with path.open("rb") as fh:
        fh.seek(0x24)
        claimed = int.from_bytes(fh.read(4), "little")
        fh.seek(0x40)
        f40 = int.from_bytes(fh.read(4), "little")
    print(f"header 0x24 (suspected entry count) = {claimed}")
    print(f"header 0x40 (suspected index offset) = {f40} (0x{f40:X})")

    best_overall = None

    for rec_size, off_pos, size_pos, mask_high, label in LAYOUTS:
        step = rec_size // 4
        n_k = (size - rec_size) // 4 + 1
        if n_k <= 0:
            continue
        k = np.arange(n_k, dtype=np.int64)
        base = 4 * k
        oi = (base + off_pos) // 4
        si = (base + size_pos) // 4
        if oi.max() >= n_words or si.max() >= n_words:
            keep = (oi < n_words) & (si < n_words)
            k, base, oi, si = k[keep], base[keep], oi[keep], si[keep]
        if k.size == 0:
            continue

        off = u32[oi].astype(np.int64)
        sz_raw = u32[si].astype(np.int64)
        sz = sz_raw & (0x7FFFFFFF if mask_high else 0xFFFFFFFF)

        valid = (
            (sz > 0)
            & (off >= DATA_START)
            & (off < size)
            & (off + sz <= size)
        )

        n_res = rec_size // 4
        for r in range(n_res):
            w = valid[r::step]
            if w.size == 0:
                continue
            length, start_j = longest_run(w)
            if length < 2:
                continue
            k_start = r + start_j * step
            s_start = 4 * k_start
            s_end = s_start + length * rec_size
            cand = (length, s_start, s_end, rec_size, label, mask_high, off_pos, size_pos)
            if best_overall is None or length > best_overall[0]:
                best_overall = cand

    if best_overall is None:
        print("no run of length >= 2 found for any layout")
        return None

    length, s_start, s_end, rec_size, label, mask_high, off_pos, size_pos = best_overall
    print()
    print(f"BEST RUN : {length} consecutive records")
    print(f"  layout      : {label}")
    print(f"  record size : {rec_size}")
    print(f"  offset field: record byte {off_pos}")
    print(f"  size field  : record byte {size_pos} (high bit used as flag: {mask_high})")
    print(f"  index start : {s_start} (0x{s_start:X})")
    print(f"  index end   : {s_end} (0x{s_end:X})")
    print(f"  bytes after index end : {size - s_end}")
    print(f"  claimed count from header 0x24 : {claimed}  (run/count = {length / claimed if claimed else 0:.4f})")
    print("  first 6 records:")
    with path.open("rb") as fh:
        for i in range(min(6, length)):
            fh.seek(s_start + i * rec_size)
            rec = fh.read(rec_size)
            words = [int.from_bytes(rec[j:j + 4], "little") for j in range(0, rec_size - 3, 4)]
            off = words[off_pos // 4]
            sz = words[size_pos // 4]
            masked = sz & (0x7FFFFFFF if mask_high else 0xFFFFFFFF)
            print(f"    [{i}] words={[hex(x) for x in words]} -> offset={off} size={masked} "
                  f"compressed={bool(sz & 0x80000000) if mask_high else '?'} end={off + masked}")
    return best_overall


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    for arg in sys.argv[1:]:
        find(Path(arg))
        print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

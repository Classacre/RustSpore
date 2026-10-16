"""
DBPF index structural search.

The header fields alone did not resolve the index layout: the candidate region
at (file size - 96) in the small locale package does not parse as 24-byte
records. Rather than keep guessing field meanings, this script searches for the
index by its structural signature.

An index of N records is a run of N consecutive fixed-size records where every
record carries a (file offset, size) pair that lands inside the file, with
offsets non-decreasing. For N in the thousands that pattern cannot occur by
chance, so finding it both locates the index and confirms the record size and
field order.

Usage:
    python dbpf_index_search.py <package> [<package> ...]
"""

import sys
from pathlib import Path

import numpy as np

# (record_size, {field_name: byte_offset_within_record})
LAYOUTS = [
    (16, {"type": 0, "group": 4, "offset": 8, "size": 12}),
    (20, {"type": 0, "group": 4, "offset": 8, "size": 12}),
    (20, {"type": 0, "group": 4, "inst_lo": 8, "inst_hi": 12, "offset": 16}),
    (24, {"type": 0, "group": 4, "inst_lo": 8, "inst_hi": 12, "offset": 16, "size": 20}),
    (24, {"type": 0, "group": 4, "offset": 8, "size": 12}),
    (28, {"type": 0, "group": 4, "inst_lo": 8, "inst_hi": 12, "offset": 16, "size": 20}),
    (28, {"type": 0, "group": 4, "inst_lo": 8, "inst_hi": 12, "offset": 20, "size": 24}),
    (32, {"type": 0, "group": 4, "inst_lo": 8, "inst_hi": 12, "offset": 16, "size": 20}),
    (32, {"type": 0, "group": 4, "inst_lo": 8, "inst_hi": 12, "offset": 20, "size": 24}),
]


def load_u32(path: Path):
    raw = np.fromfile(path, dtype=np.uint8)
    size = raw.size
    pad = (-size) % 4
    if pad:
        raw = np.concatenate([raw, np.zeros(pad, dtype=np.uint8)])
    u32 = raw.view("<u4")
    return raw, u32, size


def search(path: Path, count: int, min_start: int = 0x60, verbose: bool = True):
    raw, u32, size = load_u32(path)
    n_words = u32.size
    results = []

    for rec_size, fields in LAYOUTS:
        if "offset" not in fields or "size" not in fields:
            continue
        if count * rec_size > size:
            continue

        last_start = size - count * rec_size
        if last_start < min_start:
            continue

        # Candidate start offsets, u32 aligned.
        starts = np.arange(min_start, last_start + 1, 4, dtype=np.int64)
        if starts.size == 0:
            continue

        # Iteratively prune candidates by validating one record at a time.
        survivors = starts.copy()
        prev_off = np.full(survivors.shape, -1, dtype=np.int64)
        failed_at = None

        for i in range(count):
            base = survivors + i * rec_size
            o_idx = (base + fields["offset"]) // 4
            s_idx = (base + fields["size"]) // 4
            t_idx = (base + fields["type"]) // 4

            if o_idx.max() >= n_words or s_idx.max() >= n_words:
                keep = (o_idx < n_words) & (s_idx < n_words)
                survivors, o_idx, s_idx, t_idx = survivors[keep], o_idx[keep], s_idx[keep], t_idx[keep]
                prev_off = prev_off[keep]
                if survivors.size == 0:
                    failed_at = i
                    break

            res_off = u32[o_idx].astype(np.int64)
            res_size = u32[s_idx].astype(np.int64)
            rtype = u32[t_idx].astype(np.int64)

            ok = (
                (rtype != 0)
                & (res_size > 0)
                & (res_off >= min_start)
                & (res_off < size)
                & (res_off + res_size <= size)
                & (res_off >= prev_off)
            )
            if i == 0:
                ok &= True  # no monotonic constraint on first record
            survivors = survivors[ok]
            prev_off = res_off[ok]
            if survivors.size == 0:
                failed_at = i
                break

        if survivors.size:
            results.append((rec_size, dict(fields), survivors.copy(), count))

    if verbose:
        print("=" * 78)
        print(f"FILE  : {path}")
        print(f"SIZE  : {size}")
        print(f"COUNT : {count} (header field 0x24)")
        if not results:
            print("  no layout produced a fully valid index run")
        for rec_size, fields, starts, n in results:
            print(f"  HIT  rec_size={rec_size} fields={fields} -> {starts.size} candidate start(s)")
            for s in starts[:6]:
                print(f"        start=0x{s:X} ({s}), index spans to 0x{s + n * rec_size:X}, "
                      f"bytes after index = {size - (s + n * rec_size)}")
    return results


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    for arg in sys.argv[1:]:
        p = Path(arg)
        with p.open("rb") as fh:
            fh.seek(0x24)
            count = int.from_bytes(fh.read(4), "little")
        search(p, count)
        print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

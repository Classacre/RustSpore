"""
DBPF index field semantics.

Two questions are answered here with data rather than assumption.

1. Is the 28-byte record layout universal across SPORE packages?
   Checked by parsing every package with the hypothesised layout and reporting
   the fraction of records that validate.

2. Which field is which?
   A resource key has a small-cardinality part (type, group) and a
   high-cardinality part (instance). Measuring how many distinct values each of
   the seven u32 fields takes, and how often each value recurs, identifies the
   roles without guessing. Sort order across the record list then confirms it.

Usage:
    python dbpf_fields.py
"""

import struct
from collections import Counter
from pathlib import Path

GAME = Path(r"D:\Games\SPORE Collection")
REC = 28
DATA_START = 0x60

FIELDS = ["f0", "f1", "f2", "f3", "f4_offset", "f5_size", "f6_decomp"]


def read_header(path: Path):
    with path.open("rb") as fh:
        head = fh.read(0x60)
    return {
        "magic": head[:4],
        "major": struct.unpack_from("<I", head, 0x04)[0],
        "count": struct.unpack_from("<I", head, 0x24)[0],
        "idx_size": struct.unpack_from("<I", head, 0x2C)[0],
        "idx_off": struct.unpack_from("<I", head, 0x40)[0],
    }


def load_records(path: Path):
    h = read_header(path)
    size = path.stat().st_size
    with path.open("rb") as fh:
        fh.seek(h["idx_off"])
        prefix = struct.unpack("<I", fh.read(4))[0]
        blob = fh.read(h["count"] * REC)
        fh.seek(size - 4)
        trailer = struct.unpack("<I", fh.read(4))[0]
    recs = [struct.unpack_from("<IIIIIII", blob, i * REC) for i in range(h["count"])]
    return h, prefix, trailer, recs, size


def all_packages():
    out = []
    for sub in ("Data", "DataEP1", "bp1content"):
        d = GAME / sub
        if d.is_dir():
            out.extend(sorted(d.glob("*.package")))
    return out


def main() -> int:
    packages = all_packages()
    print(f"packages found: {len(packages)}")
    print()
    print(f"{'package':<34}{'count':>7}{'idx_ok':>8}{'valid%':>9}{'prefix':>8}{'trailer':>10}")
    print("-" * 78)

    results = []
    for p in packages:
        try:
            h, prefix, trailer, recs, size = load_records(p)
        except Exception as exc:  # noqa: BLE001
            print(f"{p.name:<34}  ERROR: {exc}")
            continue
        n = h["count"]
        good = 0
        comp = 0
        dec_gt = 0
        first_off = None
        for (f0, f1, f2, f3, off, sz, dec) in recs:
            raw = sz & 0x7FFFFFFF
            if raw > 0 and off >= DATA_START and off + raw <= size:
                good += 1
            if sz & 0x80000000:
                comp += 1
            if dec > raw:
                dec_gt += 1
        if recs:
            first_off = recs[0][4]
        idx_ok = "yes" if h["idx_off"] + h["idx_size"] == size else "NO"
        pct = 100.0 * good / n if n else 0.0
        print(f"{p.name:<34}{n:>7}{idx_ok:>8}{pct:>8.2f}%{prefix:>8}{trailer:>10}")
        results.append((p, h, prefix, trailer, recs, size, good, comp, dec_gt, first_off))

    print()
    print("=" * 78)
    print("FIELD SEMANTICS (largest package by record count)")
    print("=" * 78)
    best = max(results, key=lambda r: r[1]["count"])
    p, h, prefix, trailer, recs, size, good, comp, dec_gt, first_off = best
    n = h["count"]
    print(f"package: {p.name}  records: {n}  size: {size}")
    print(f"prefix u32 = {prefix}   trailer u32 = {trailer} (0x{trailer:X})")
    print(f"compressed-flagged records: {comp}/{n}")
    print(f"records where decompressed_size > stored_size: {dec_gt}/{n}")
    print(f"first record offset = {first_off} (0x{first_off:X})  [data start is 0x60]")
    print()
    print(f"{'field':<12}{'distinct':>10}{'top 6 values (count)'}")
    print("-" * 78)
    for i, name in enumerate(FIELDS):
        vals = [r[i] for r in recs]
        c = Counter(vals)
        top = ", ".join(f"0x{v:08X}({k})" for v, k in c.most_common(6))
        print(f"{name:<12}{len(c):>10}   {top}")

    print()
    print("SORT ORDER CHECKS (is the record list sorted by these key orders?)")
    print("-" * 78)
    orders = {
        "f1,f0,f2,f3": (1, 0, 2, 3),
        "f0,f1,f2,f3": (0, 1, 2, 3),
        "f1,f0,f3,f2": (1, 0, 3, 2),
        "f0,f1,f3,f2": (0, 1, 3, 2),
    }
    for label, idxs in orders.items():
        keys = [tuple(r[i] for i in idxs) for r in recs]
        print(f"  {label:<14} sorted: {keys == sorted(keys)}")

    print()
    print("GROUPED CHECK: for the most common value of f0, what are the f1 values?")
    common0 = Counter(r[0] for r in recs).most_common(3)
    for val, cnt in common0:
        f1s = Counter(r[1] for r in recs if r[0] == val).most_common(5)
        print(f"  f0=0x{val:08X} ({cnt} records) -> f1: "
              + ", ".join(f"0x{v:08X}({k})" for v, k in f1s))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

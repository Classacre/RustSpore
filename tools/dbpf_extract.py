"""
Extract resources from SPORE DBPF packages.

This is the end-to-end pipeline: read the index, slice the payload, decompress with
RefPack if needed, and write the result. It is also the strongest available test of
the format work, because a decoded PNG carries its own checksums: if the bytes are
wrong anywhere in the chain, the CRCs fail.

Usage:
    python dbpf_extract.py <package> --list
    python dbpf_extract.py <package> --type 0x2F7D0004 --out <dir> [--limit N]
    python dbpf_extract.py <package> --validate-png
"""

import argparse
import struct
import sys
import zlib
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress, RefPackError  # noqa: E402

REC = 28
PREFIX = 4


def parse_index(path: Path):
    size = path.stat().st_size
    with path.open("rb") as fh:
        head = fh.read(0x60)
        if head[:4] != b"DBPF":
            raise ValueError("not a DBPF file")
        count = struct.unpack_from("<I", head, 0x24)[0]
        idx_size = struct.unpack_from("<I", head, 0x2C)[0]
        idx_off = struct.unpack_from("<I", head, 0x40)[0]
        needed = PREFIX + 4 + count * REC
        if needed > idx_size:
            raise ValueError(
                f"index layout not supported: {count} entries need {needed} bytes "
                f"but the block is {idx_size} (see docs/dbpf-format.md section 6)"
            )
        fh.seek(idx_off + PREFIX)
        blob = fh.read(count * REC)

    entries = []
    for i in range(count):
        g, t, ihi, ilo, off, sz, dec = struct.unpack_from("<IIIIIII", blob, i * REC)
        entries.append({
            "group": g, "type": t, "instance": (ihi << 32) | ilo,
            "offset": off, "stored": sz & 0x7FFFFFFF, "dec": dec,
        })
    return entries, size


def payload(path: Path, entry) -> bytes:
    """Read a payload, decompressing when the stored and declared sizes differ."""
    with path.open("rb") as fh:
        fh.seek(entry["offset"])
        blob = fh.read(entry["stored"])
    if entry["dec"] > entry["stored"]:
        out, _used, _info = decompress(blob, out_size=None, header=True)
        return out
    return blob


def png_report(b: bytes) -> tuple[bool, str]:
    """Walk PNG chunks and verify every CRC."""
    if not b.startswith(b"\x89PNG\r\n\x1a\n"):
        return False, "no PNG signature"
    pos, chunks = 8, 0
    while pos + 8 <= len(b):
        ln = struct.unpack_from(">I", b, pos)[0]
        typ = b[pos + 4:pos + 8]
        if pos + 12 + ln > len(b):
            return False, f"truncated at chunk {chunks} ({typ!r})"
        data = b[pos + 8:pos + 8 + ln]
        crc = struct.unpack_from(">I", b, pos + 8 + ln)[0]
        if zlib.crc32(typ + data) & 0xFFFFFFFF != crc:
            return False, f"CRC mismatch at chunk {chunks} ({typ!r})"
        chunks += 1
        pos += 12 + ln
        if typ == b"IEND":
            ihdr = b.find(b"IHDR")
            w = h = None
            if ihdr > 0:
                w, h = struct.unpack_from(">II", b, ihdr + 4)
            return True, f"{chunks} chunks, {w}x{h}, trailing {len(b) - pos} bytes"
    return False, f"no IEND after {chunks} chunks"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("package")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--type", type=lambda s: int(s, 0))
    ap.add_argument("--out", type=Path)
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--validate-png", action="store_true")
    args = ap.parse_args()

    path = Path(args.package)
    entries, size = parse_index(path)
    print(f"{path.name}: {size} bytes, {len(entries)} resources")

    if args.list or not args.type:
        hist = Counter(e["type"] for e in entries)
        coded = sum(1 for e in entries if e["dec"] > e["stored"])
        print(f"  coded {coded}, raw {len(entries) - coded}")
        print("  resource types:")
        for t, c in hist.most_common(25):
            print(f"    0x{t:08X}  {c:>7}")

    if args.validate_png:
        png_type = 0x2F7D0004
        cands = [e for e in entries if e["type"] == png_type]
        if not cands:
            print(f"  no resources of type 0x{png_type:08X} in this package")
            return 0
        print(f"  validating {len(cands)} resources of type 0x{png_type:08X} (PNG)")
        ok = 0
        failed = []
        total_bytes = 0
        for e in cands:
            try:
                b = payload(path, e)
            except RefPackError as exc:
                failed.append((e, f"decompress failed: {exc}"))
                continue
            total_bytes += len(b)
            good, detail = png_report(b)
            if good:
                ok += 1
            elif len(failed) < 8:
                failed.append((e, detail))
        print(f"  valid PNGs: {ok}/{len(cands)}  ({100.0 * ok / len(cands):.1f}%)")
        print(f"  decoded bytes: {total_bytes}")
        for e, detail in failed:
            print(f"    FAIL type=0x{e['type']:08X} inst=0x{e['instance']:016X}: {detail}")
        return 0

    if args.type:
        cands = [e for e in entries if e["type"] == args.type]
        if args.limit:
            cands = cands[:args.limit]
        out_dir = args.out or (Path(__file__).resolve().parent.parent / "assets" / "extracted")
        out_dir.mkdir(parents=True, exist_ok=True)
        written = 0
        for e in cands:
            try:
                b = payload(path, e)
            except RefPackError:
                continue
            name = f"{e['type']:08X}_{e['group']:08X}_{e['instance']:016X}.bin"
            (out_dir / name).write_bytes(b)
            written += 1
        print(f"  wrote {written} resources to {out_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

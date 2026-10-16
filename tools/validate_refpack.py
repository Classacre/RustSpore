"""
Validate the RefPack decompressor against real SPORE package payloads.

The DBPF index records both the stored size and the decompressed size for every
resource, so a resource is a self-checking test case: decompressing the stored
bytes must yield exactly the recorded number of bytes, and the result should be
a recognisable file. A decompressor that produces the right length and a valid
PNG or DDS header is confirmed by the game's own data rather than by the spec.

Usage:
    python validate_refpack.py <package> [limit]
"""

import struct
import sys
import zlib
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress, RefPackError  # noqa: E402

REC = 28
DATA_START = 0x60


def parse_index(path: Path):
    size = path.stat().st_size
    with path.open("rb") as fh:
        head = fh.read(0x60)
        if head[:4] != b"DBPF":
            raise ValueError("not DBPF")
        count = struct.unpack_from("<I", head, 0x24)[0]
        idx_size = struct.unpack_from("<I", head, 0x2C)[0]
        idx_off = struct.unpack_from("<I", head, 0x40)[0]
        fh.seek(idx_off + 4)
        blob = fh.read(count * REC)
    if len(blob) < count * REC:
        raise ValueError(f"index truncated: {len(blob)} of {count * REC}")
    entries = []
    for i in range(count):
        g, t, ihi, ilo, off, sz, dec = struct.unpack_from("<IIIIIII", blob, i * REC)
        entries.append({
            "group": g, "type": t, "instance": (ihi << 32) | ilo,
            "offset": off, "stored": sz & 0x7FFFFFFF,
            "flag31": bool(sz & 0x80000000), "decompressed": dec,
        })
    return entries, size, count, idx_size, idx_off


MAGICS = [
    (b"\x89PNG\r\n\x1a\n", "PNG"),
    (b"DDS ", "DDS"),
    (b"RIFF", "RIFF"),
    (b"OggS", "OggS"),
    (b"<?xml", "XML"),
    (b"<", "XML-ish"),
    (b"DBPF", "DBPF"),
    (b"\xff\xfe", "UTF-16LE text"),
    (b"ID3", "MP3"),
    (b"PK\x03\x04", "ZIP"),
    (b"\x1bLua", "Lua"),
    (b"SNR", "SNR"),
]


def identify(b: bytes) -> str:
    for magic, name in MAGICS:
        if b.startswith(magic):
            return name
    # printable ascii?
    if b[:8] and all(32 <= c < 127 or c in (9, 10, 13) for c in b[:16]):
        return "ASCII text"
    return "?"


def png_ok(b: bytes) -> str:
    """Walk PNG chunks and verify CRCs. This is a strong correctness oracle."""
    if not b.startswith(b"\x89PNG\r\n\x1a\n"):
        return "not PNG"
    pos = 8
    n = 0
    while pos + 8 <= len(b):
        ln = struct.unpack_from(">I", b, pos)[0]
        typ = b[pos + 4:pos + 8]
        if pos + 12 + ln > len(b):
            return f"truncated at chunk {n} ({typ!r})"
        data = b[pos + 8:pos + 8 + ln]
        crc = struct.unpack_from(">I", b, pos + 8 + ln)[0]
        if zlib.crc32(typ + data) & 0xFFFFFFFF != crc:
            return f"CRC FAIL at chunk {n} ({typ!r})"
        n += 1
        pos += 12 + ln
        if typ == b"IEND":
            return f"VALID PNG, {n} chunks, IEND at {pos}, trailing {len(b) - pos} bytes"
    return f"no IEND after {n} chunks"


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    path = Path(sys.argv[1])
    limit = int(sys.argv[2]) if len(sys.argv) > 2 else 25

    entries, fsize, count, idx_size, idx_off = parse_index(path)
    print(f"file       : {path.name}")
    print(f"size       : {fsize}")
    print(f"entries    : {count}   index {idx_off} + {idx_size} = {idx_off + idx_size}")
    print(f"flag31 set : {sum(1 for e in entries if e['flag31'])}/{count}")
    print(f"dec > stored: {sum(1 for e in entries if e['decompressed'] > e['stored'])}/{count}")

    # Type histogram, so the sample below is interpretable.
    print("top resource types:", ", ".join(f"0x{t:08X}({c})" for t, c in
          Counter(e["type"] for e in entries).most_common(6)))
    print()

    # Hypothesis: stored == decompressed means the payload is stored raw, and
    # only the strictly larger decompressed size implies RefPack. Count both.
    raw_ct = sum(1 for e in entries if e["stored"] == e["decompressed"])
    print(f"stored == decompressed (raw?): {raw_ct}/{count}")
    print(f"stored <  decompressed (coded?): {count - raw_ct}/{count}")
    print()

    # Test only the genuinely coded resources, smallest first so the run is quick.
    common_type = Counter(e["type"] for e in entries).most_common(1)[0][0]
    cands = sorted((e for e in entries
                    if e["type"] == common_type and e["decompressed"] > e["stored"]),
                   key=lambda e: e["stored"])[:limit]
    print(f"decompressing {len(cands)} smallest coded resources of type 0x{common_type:08X}")
    print(f"{'#':>3}{'stored':>9}{'expect':>9}{'got':>9}{'hdr':>5}{'id':>10}  detail")
    print("-" * 92)

    ok_len = 0
    tried = 0
    for i, e in enumerate(cands):
        with path.open("rb") as fh:
            fh.seek(e["offset"])
            blob = fh.read(e["stored"])
        got = None
        for hdr in (False, True):
            try:
                out, used, info = decompress(blob, out_size=e["decompressed"], header=hdr)
                got = (out, hdr)
                break
            except (RefPackError, IndexError, struct.error):
                continue
        tried += 1
        if got is None:
            print(f"{i:>3}{e['stored']:>9}{e['decompressed']:>9}{'--':>9}{'--':>5}{'FAILED':>10}")
            continue
        out, hdr = got
        length_ok = len(out) == e["decompressed"]
        ok_len += length_ok
        ident = identify(out)
        detail = png_ok(out) if ident == "PNG" else out[:12].hex(" ")
        print(f"{i:>3}{e['stored']:>9}{e['decompressed']:>9}{len(out):>9}"
              f"{str(hdr):>5}{ident:>10}  {detail}")

    print()
    print(f"length matched declared decompressed size: {ok_len}/{tried}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

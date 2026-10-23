"""Dump concrete failing prop examples: under-by-4 and desync-with-000d0000."""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from refpack import decompress  # noqa: E402

REC = 28
PREFIX = 4
PROP_TYPE = 0x00B1B104
ARRAY_SPECIFIERS = frozenset({0x009C, 0x001C})
FIXED = {0x09: 4, 0x0a: 4, 0x0d: 4, 0x30: 8, 0x31: 12, 0x32: 12, 0x33: 16, 0x34: 16, 0x38: 64, 0x39: 24}
SINGLE_PADDING = {0x30: 4, 0x31: 4, 0x32: 4, 0x33: 4, 0x38: 4, 0x39: 4}
MARKER = b"\x3c\xc4\x12\x00"


def parse_index(path: Path):
    with path.open("rb") as fh:
        head = fh.read(0x60)
        count = struct.unpack_from("<I", head, 0x24)[0]
        idx_off = struct.unpack_from("<I", head, 0x40)[0]
        fh.seek(idx_off + PREFIX)
        blob = fh.read(count * REC)
    return [(g, off, sz & 0x7FFFFFFF, dec) for (g, t, ihi, ilo, off, sz, dec)
            in [struct.unpack_from("<IIIIIII", blob, i * REC) for i in range(count)]
            if t == PROP_TYPE]


def scalar_size(typ, spec):
    if typ == 0x01:
        return 1
    if typ in FIXED:
        return FIXED[typ] + (0 if spec in ARRAY_SPECIFIERS else SINGLE_PADDING.get(typ, 0))
    if typ == 0x20:
        return 16
    if typ == 0x00:
        return spec
    return None


def walk(d: bytes):
    p = 0
    try:
        _c, p = struct.unpack_from(">I", d, p)[0], p + 4
    except struct.error:
        return False, 0, p, "no header"
    items = 0
    while p + 8 <= len(d):
        if d[p:p + 4] == MARKER:
            p += 4
            continue
        ident, typ, spec = struct.unpack_from(">IHH", d, p)
        p += 8
        if spec in ARRAY_SPECIFIERS:
            n, stride = struct.unpack_from(">II", d, p); p += 8
            if n > 200_000 or stride > 16384 or p + n * stride > len(d):
                return False, items, p - 8, f"array bad @ {p-8}"
            p += n * stride
        elif typ in (0x12, 0x13):
            n = struct.unpack_from(">I", d, p)[0]; p += 4
            p += n * (1 if typ == 0x12 else 2)
        elif typ == 0x22:
            n = struct.unpack_from(">I", d, p)[0]; p += 4
            for _ in range(n):
                sz, _, _ = struct.unpack_from(">III", d, p); p += 12
                if p + sz > len(d):
                    return False, items, p, f"texts buf @ {p}"
                p += sz
        else:
            n = scalar_size(typ, spec)
            if n is None:
                return False, items, p - 8, f"type 0x{typ:04x} @ {p-8}"
            if p + n > len(d):
                return False, items, p, f"scalar over @ {p}"
            p += n
        items += 1
    return (p == len(d)), items, p, ("" if p == len(d) else f"ended {p}/{len(d)}")


def hexdump(b, start, n):
    out = []
    for i in range(0, n, 16):
        chunk = b[start + i:start + i + 16]
        if not chunk:
            break
        hx = " ".join(f"{x:02x}" for x in chunk)
        asc = "".join(chr(x) if 32 <= x < 127 else "." for x in chunk)
        out.append(f"{start+i:06x}  {hx:<47}  {asc}")
    return "\n".join(out)


def main():
    pkg = sys.argv[1] if len(sys.argv) > 1 else r"D:\Games\SPORE Collection\Data\Spore_Game.package"
    path = Path(pkg)
    shown_under = 0
    shown_desync = 0
    for g, off, stored, dec in parse_index(path):
        with path.open("rb") as fh:
            fh.seek(off); raw = fh.read(stored)
        try:
            d = decompress(raw, out_size=None, header=True)[0] if dec > stored else raw
        except Exception:
            continue
        ok, items, end, reason = walk(d)
        if ok:
            continue
        is_under4 = reason.startswith("ended") and len(d) - end == 4
        if is_under4 and shown_under < 2:
            print(f"=== UNDER-BY-4 group 0x{g:08X} len={len(d)} reason={reason} ===")
            print(hexdump(d, max(0, end - 24), min(48, len(d) - max(0, end - 24))))
            print()
            shown_under += 1
        elif (not is_under4) and shown_desync < 2:
            at = end - 8 if end >= 8 else 0
            print(f"=== DESYNC group 0x{g:08X} len={len(d)} reason={reason} ===")
            print(hexdump(d, max(0, at - 32), min(96, len(d) - max(0, at - 32))))
            print()
            shown_desync += 1
        if shown_under >= 2 and shown_desync >= 2:
            break


if __name__ == "__main__":
    main()

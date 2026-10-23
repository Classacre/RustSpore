#!/usr/bin/env python3
"""Print the same gmsh digest as `rustspore --gmsh-digest`.

Format: `<instance:016X> <vertices> <indices> <normals> <uvs> <fnv1a:016X>`.

Diffing the two outputs checks that the Rust parser produces the same typed
geometry as the Python one, including the packed-normal decoding.

Usage:
    python tools/gmsh_digest.py "<package>" > py.txt
    rustspore --gmsh-digest "<package>" 2>/dev/null > rs.txt
    diff py.txt rs.txt && echo identical
"""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from dbpf_extract import parse_index
from refpack import decompress
from gmsh_attributes import walk
from gmsh_complete import parse_full

GMSH_TYPE = 0x01C135DA
NORMAL_F32 = "02000301"
NORMAL_PACKED = "02000701"
UV_F32 = "08000202"


def fnv1a64(data: bytes) -> int:
    h = 0xCBF29CE484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    pkg = Path(sys.argv[1])
    entries, _ = parse_index(pkg)

    ok = 0
    failed = 0
    with pkg.open("rb") as fh:
        for e in entries:
            if e["type"] != GMSH_TYPE:
                continue
            fh.seek(e["offset"])
            raw = fh.read(e["stored"])
            if e["dec"] > e["stored"]:
                try:
                    raw = decompress(raw, out_size=None, header=True)[0]
                except Exception as ex:
                    print(f"read 0x{e['instance']:016X}: {ex}", file=sys.stderr)
                    failed += 1
                    continue

            if len(raw) < 24:
                print(f"parse 0x{e['instance']:016X}: too short", file=sys.stderr)
                failed += 1
                continue
            vcount = struct.unpack_from(">H", raw, 20)[0]
            pstride = struct.unpack_from(">H", raw, 22)[0]
            if pstride < 12 or 24 + vcount * pstride > len(raw):
                print(f"parse 0x{e['instance']:016X}: bad positions", file=sys.stderr)
                failed += 1
                continue

            positions = [struct.unpack_from("<3f", raw, 24 + i * pstride) for i in range(vcount)]
            normals = []
            uvs = []
            for magic, count, stride, at in walk(raw):
                h = magic.hex()
                if h == NORMAL_F32 and stride >= 12:
                    normals = [struct.unpack_from("<3f", raw, at + i * stride) for i in range(count)]
                elif h == NORMAL_PACKED and stride >= 4:
                    normals = []
                    for i in range(count):
                        b = raw[at + i * stride:at + i * stride + 4]
                        normals.append((b[0] / 127.5 - 1.0, b[1] / 127.5 - 1.0, b[2] / 127.5 - 1.0))
                elif h == UV_F32 and stride >= 8:
                    uvs = [struct.unpack_from("<2f", raw, at + i * stride) for i in range(count)]

            parsed = parse_full(raw)
            indices = list(parsed["indices"]) if parsed and parsed.get("all_valid") else []

            posb = bytearray()
            for p in positions:
                posb += struct.pack("<3f", *p)
            idxb = bytearray()
            for i in indices:
                idxb += struct.pack("<H", i)
            nrmb = bytearray()
            for n in normals:
                nrmb += struct.pack("<3f", *n)
            uvb = bytearray()
            for uv in uvs:
                uvb += struct.pack("<2f", *uv)

            print(
                f"{e['instance']:016X} {len(positions):>5} {len(indices):>6} "
                f"{len(normals):>6} {len(uvs):>5} {fnv1a64(bytes(posb)):016X} "
                f"{fnv1a64(bytes(idxb)):016X} {fnv1a64(bytes(nrmb)):016X} {fnv1a64(bytes(uvb)):016X}"
            )
            ok += 1

    print(f"parsed {ok}, failed {failed}", file=sys.stderr)
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())

"""Which prop keys point at which resources.

The prop format is parsed byte-exactly, but a tree of anonymous numbers is not a
schema. This script answers a narrower, checkable question: of all the `Key` values
in a prop resource, what do they point at?

A key is 12 bytes: group, type, instance. If a prop holds keys whose type is RW4
(0x2F4E681B) or gmsh (0x01C135DA), then props are the layer that names models, and
the idents carrying those keys are the ones worth naming.

Names are FNV-1a 32 of the key name, so a guessed name can be confirmed by matching
the hash. Guesses are listed at the bottom of the output rather than assumed.
"""

import struct
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from dbpf_extract import parse_index  # noqa: E402
from prop_parse import (  # noqa: E402
    ARRAY_SPECIFIERS,
    BOOL_SIZE,
    FIXED,
    KEY_ARRAY_STRIDE,
    KEY_SINGLE_SIZE,
    SINGLE_PADDING,
)
from refpack import decompress  # noqa: E402

PROP = 0x00B1B104

TARGET_NAMES = {
    0x2F4E681B: "RW4",
    0x01C135DA: "gmsh",
    0x00E6BCE5: "gmdl",
    0x2F4E681C: "RASTER",
    0x2F7D0004: "PNG",
    0xEE17C6AD: "anim",
    0x00B1B104: "prop",
    0x02B9F662: "audio_event",
    0x5CAA5E28: "blockdata",
    0xDFAD9F51: "string_table",
}


def be32(d, p):
    return struct.unpack_from(">I", d, p)[0], p + 4


def be16(d, p):
    return struct.unpack_from(">H", d, p)[0], p + 2


class Bad(Exception):
    pass


def read_value(d, p, typ, spec, in_array):
    if typ == 0x01:
        return None, p + BOOL_SIZE
    if typ in (0x12, 0x13):
        n, p = be32(d, p)
        width = 1 if typ == 0x12 else 2
        if p + n * width > len(d):
            raise Bad("string overruns")
        return None, p + n * width
    if typ == 0x22:
        n, p = be32(d, p)
        if p + n > len(d):
            raise Bad("texts overruns")
        return None, p + n
    if typ == 0x20:
        n = KEY_ARRAY_STRIDE if in_array else KEY_SINGLE_SIZE
        if p + n > len(d):
            raise Bad("key past end")
        # The ident is big-endian but the key members are little-endian. Measured:
        # reading them big-endian turns 0x00E6BCE5 (gmdl) into 0xE5BCE600.
        g, t, i = struct.unpack_from("<3I", d, p)
        return (g, t, i), p + n
    if typ == 0x00:
        if spec > len(d) - p:
            raise Bad("raw overruns")
        return None, p + spec
    if typ in FIXED:
        n = FIXED[typ] + (0 if in_array else SINGLE_PADDING.get(typ, 0))
        if p + n > len(d):
            raise Bad("fixed overruns")
        return None, p + n
    raise Bad("unhandled type 0x%02x" % typ)


def walk(d):
    """Yield (ident, keys) for one prop blob. Raises Bad if it does not fit."""
    count, p = be32(d, 0)
    if count > 200_000:
        raise Bad("implausible item count")
    for _ in range(count):
        ident, p = be32(d, p)
        typ, p = be16(d, p)
        spec, p = be16(d, p)
        if spec in ARRAY_SPECIFIERS:
            n, p = be32(d, p)
            stride, p = be32(d, p)
            if n > 100_000 or stride > 8192:
                raise Bad("implausible array")
            if p + n * stride > len(d):
                raise Bad("array overruns")
            keys = []
            if typ == 0x20:
                for k in range(n):
                    keys.append(struct.unpack_from("<3I", d, p + k * stride))
            p += n * stride
            yield ident, keys
        else:
            v, p = read_value(d, p, typ, spec, False)
            yield ident, ([v] if v else [])
    if p != len(d):
        raise Bad("consumed %d of %d" % (p, len(d)))


def fnv1a32(text):
    h = 0x811C9DC5
    for b in text.encode("latin-1"):
        h = ((h ^ b) * 0x01000193) & 0xFFFFFFFF
    return h


GUESSES = [
    "model", "models", "modelKey", "material", "materials", "materialKey",
    "mesh", "meshes", "texture", "textureKey", "parts", "part", "components",
    "rigblock", "rigblocks", "name", "description", "tags", "type", "id",
    "modelInstance", "materialIndex", "materialSrc", "materialDst",
    "geometry", "geometryKey", "animation", "animations", "scale", "transform",
]


def main(path):
    pkg = Path(path)
    entries, _ = parse_index(pkg)
    ok = 0
    bad = 0
    key_targets = Counter()
    ident_targets = defaultdict(Counter)
    total_keys = 0
    with pkg.open("rb") as fh:
        for e in entries:
            if e["type"] != PROP:
                continue
            fh.seek(e["offset"])
            raw = fh.read(e["stored"])
            if e["dec"] > e["stored"]:
                try:
                    raw = decompress(raw, out_size=None, header=True)[0]
                except Exception:
                    bad += 1
                    continue
            try:
                items = list(walk(raw))
            except Bad:
                bad += 1
                continue
            ok += 1
            for ident, keys in items:
                for k in keys:
                    total_keys += 1
                    key_targets[k[1]] += 1
                    ident_targets[ident][k[1]] += 1

    print("prop resources parsed: %d (failed %d)" % (ok, bad))
    print("keys seen: %d" % total_keys)
    print()
    print("key targets by type:")
    for t, n in key_targets.most_common(14):
        print("  0x%08X %-14s %7d" % (t, TARGET_NAMES.get(t, "?"), n))
    print()
    print("idents whose keys point at a model or texture:")
    rows = []
    for ident, targets in ident_targets.items():
        hits = sum(
            n for t, n in targets.items()
            if t in (0x2F4E681B, 0x01C135DA, 0x00E6BCE5, 0x2F4E681C, 0x2F7D0004)
        )
        if hits:
            rows.append((hits, ident, targets))
    rows.sort(reverse=True)
    for hits, ident, targets in rows[:15]:
        detail = ", ".join(
            "%s=%d" % (TARGET_NAMES.get(t, "0x%08X" % t), n)
            for t, n in targets.most_common(4)
        )
        print("  0x%08X %7d keys  %s" % (ident, hits, detail))
    print()
    known = {ident for _, ident, _ in rows}
    print("name guesses that match an ident carrying a model key:")
    any_hit = False
    for name in GUESSES:
        for variant in (name, name.lower(), name.upper()):
            h = fnv1a32(variant)
            if h in known:
                print("  %-16s -> 0x%08X" % (variant, h))
                any_hit = True
    if not any_hit:
        print("  none of the %d guesses matched" % len(GUESSES))


if __name__ == "__main__":
    main(sys.argv[1])

"""Inspect one explicit package's bounded SpaceGame candidate PROP graph.

This reports static key-word correspondence, not active package precedence,
provider identity, gameplay rules, or a resolved galaxy. Requires stable file.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path

if not __debug__:
    raise RuntimeError('normal Python required by native index decoder')
from native_archive_index_probe import decode
from refpack import decompress, _read_header

ROOT = 0x2DB6DAD3
TYPE = 0x00B1B104
GROUP = 0xCC489C6F
MAX_ARCHIVE = 8 * 1024**3


def read_shape(data):
    """Recognize only the observed two-property shape; leave keys as words."""
    if len(data) < 548 or struct.unpack_from('>I', data)[0] != 2:
        raise ValueError('expected complete two-property frame')
    if struct.unpack_from('>IHHII', data, 4) != (0x7435A2D0, 0x22, 0x1C, 1, 0x208):
        raise ValueError('unexpected first property')
    at = 20 + 0x208
    ident, typ, spec = struct.unpack_from('>IHH', data, at)
    if ident == 0x7435A2D3 and (typ, spec) == (0x20, 0x9c):
        if len(data) < at + 16:
            raise ValueError('missing array header')
        count, stride = struct.unpack_from('>II', data, at + 8)
        if count > 64 or stride != 12 or at + 16 + count * stride != len(data):
            raise ValueError('invalid bounded array extent')
        return {'kind': 'array', 'property_id': '7435A2D3',
                'words_le': [list(struct.unpack_from('<III', data, at + 16 + i * 12)) for i in range(count)]}
    if ident == 0x7435A2D2 and (typ, spec) == (0x20, 0) and len(data) == at + 24:
        return {'kind': 'single', 'property_id': '7435A2D2',
                'words_le': [list(struct.unpack_from('<IIII', data, at + 8))]}
    raise ValueError('unexpected second property')


def decompress_bounded(raw, decoded):
    """Reject mismatched stream header before allocating decoded output."""
    _, _, declared = _read_header(raw, 0)
    if declared != decoded:
        raise ValueError("RefPack header size mismatch")
    data, used, _ = decompress(raw, out_size=decoded, header=True)
    return data, used


def hash_file(path):
    h = hashlib.sha256(); size = 0
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''):
            size += len(chunk)
            if size > MAX_ARCHIVE:
                raise ValueError('archive exceeds size limit')
            h.update(chunk)
    return h.hexdigest(), size


def inspect(path, expected_sha):
    before, length = hash_file(path)
    if before.lower() != expected_sha.lower():
        raise ValueError('archive pin mismatch')
    with path.open('rb') as f:
        hdr = f.read(96)
        if len(hdr) != 96 or hdr[:4] != b'DBPF': raise ValueError('invalid DBPF')
        count, size, at = (struct.unpack_from('<I', hdr, x)[0] for x in (36, 44, 64))
        if count > 1_000_000 or size > 128 * 1024**2 or at + size > length: raise ValueError('index bounds')
        f.seek(at); block = f.read(size)
        if len(block) != size: raise ValueError('short index')
        flags, rows = decode(block, count)
        def load(instance):
            found = [r for r in rows if r[:3] == (instance, TYPE, GROUP)]
            if len(found) != 1: raise ValueError('candidate must be unique in supplied package')
            _, _, _, offset, stored, decoded, compression, _, _ = found[0]
            if not (0 < stored <= 1024**2 and 0 < decoded <= 1024**2 and offset + stored <= length):
                raise ValueError('resource bounds')
            f.seek(offset); raw = f.read(stored)
            if len(raw) != stored: raise ValueError('short resource')
            if compression == 0 and stored == decoded:
                data, used = raw, stored
            elif compression == 0xffff and stored < decoded:
                data, used = decompress_bounded(raw, decoded)
            else: raise ValueError('unsupported compression')
            if len(data) != decoded or not 0 <= used <= stored: raise ValueError('decoded extent')
            return {'instance': f'{instance:08X}', 'payload_sha256': hashlib.sha256(data).hexdigest(),
                    'stored': stored, 'decoded': decoded, 'consumed': used,
                    'unconsumed_hex': raw[used:].hex(), 'shape': read_shape(data)}
        root = load(ROOT)
        if root['shape']['kind'] != 'array' or len(root['shape']['words_le']) != 5:
            raise ValueError('unexpected candidate root')
        words = root['shape']['words_le']
        if any(w[1:] != [0, 0] for w in words) or len({w[0] for w in words}) != len(words):
            raise ValueError('root word assumptions not met')
        children = [load(w[0]) for w in words]
    after, after_len = hash_file(path)
    if (after, after_len) != (before, length): raise ValueError('archive changed')
    return {'status': 'matched_static', 'package_sha256': before, 'index_flags': flags,
            'scope': 'explicit stable package, one-level key-word association; no live provider or gameplay',
            'framing_limit': 'unconsumed bytes retained; original decoder acceptance unverified',
            'root': root, 'children': children}


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('package', type=Path); ap.add_argument('sha256')
    ap.add_argument('--admit-stable-file', action='store_true', required=True)
    args = ap.parse_args()
    print(json.dumps(inspect(args.package, args.sha256), indent=2))

if __name__ == '__main__': main()

"""Inspect the installed SpaceGame candidate's five raw 12-byte references.

The SpaceGame instance and Planets instance match lowercase FNV-1 hashes of
strings in SporeApp.exe. The containing resource and field semantics, package
precedence, and live use are unproved. This tool does not modify game state.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path

from dbpf_extract import payload
from native_archive_index_probe import decode

KEY = (0x2DB6DAD3, 0x00B1B104, 0xCC489C6F)


def inspect_bytes(data):
    # Admit only the complete observed two-property shape, without interpreting
    # the opaque 0x22 record or assigning a meaning to the reference triples.
    if len(data) < 24 or struct.unpack_from('>I', data)[0] != 2:
        raise ValueError('expected two-property candidate')
    ident, typ, spec, count, stride = struct.unpack_from('>IHHII', data, 4)
    if (ident, typ, spec, count, stride) != (0x7435A2D0, 0x22, 0x1C, 1, 0x208):
        raise ValueError('unexpected first property shape')
    at = 20 + count * stride
    if at + 16 > len(data):
        raise ValueError('truncated first property')
    ident, typ, spec, count, stride = struct.unpack_from('>IHHII', data, at)
    if (ident, typ, spec, count, stride) != (0x7435A2D3, 0x20, 0x9C, 5, 12):
        raise ValueError('unexpected second property shape')
    start = at + 16
    if start + count * stride != len(data):
        raise ValueError('truncated or trailing reference bytes')
    refs = [list(map(lambda word: f'{word:08X}', struct.unpack_from('<III', data, start + i * stride)))
            for i in range(count)]
    return {'payload_sha256': hashlib.sha256(data).hexdigest(),
            'key_instance_type_scope': [f'{v:08X}' for v in KEY],
            'opaque_first_record_sha256': hashlib.sha256(data[20:at]).hexdigest(),
            'second_property_id': '7435A2D3', 'reference_words_le': refs,
            'status': 'static candidate; names, word meanings, active precedence and runtime use unverified'}


def inspect_package(path):
    with path.open('rb') as source:
        header = source.read(96)
        if len(header) != 96 or header[:4] != b'DBPF':
            raise ValueError('invalid DBPF header')
        count, size, offset = (struct.unpack_from('<I', header, pos)[0] for pos in (36, 44, 64))
        if offset + size > path.stat().st_size:
            raise ValueError('index outside package')
        source.seek(offset)
        block = source.read(size)
    _, rows = decode(block, count)
    found = [r for r in rows if r[:3] == KEY]
    if len(found) != 1:
        raise ValueError('expected exactly one candidate')
    _, _, _, offset, stored, decoded, compression, _, _ = found[0]
    if offset + stored > path.stat().st_size:
        raise ValueError('resource outside package')
    if not ((compression == 0 and stored == decoded) or (compression == 0xFFFF and stored < decoded)):
        raise ValueError('unsupported compression or sizes')
    data = payload(path, dict(offset=offset, stored=stored, dec=decoded))
    if len(data) != decoded:
        raise ValueError('decoded size mismatch')
    result = inspect_bytes(data)
    result.update(package=str(path.resolve()), index_sha256=hashlib.sha256(block).hexdigest())
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('package', type=Path)
    args = parser.parse_args()
    print(json.dumps(inspect_package(args.package), indent=2))


if __name__ == '__main__':
    main()

"""Inspect one candidate Creature-related source property, without assigning gameplay semantics.

Version 2 uses native flag-driven index fields; legacy 0001FFFF was metadata,
not resource scope. The local executable uses the spellings Combat and StandardCreatureHealth. Their
lowercase FNV-1 hashes match the instance and FLOAT field in resource
A4C2F6BD:00B1B104:00000000 in both base and patch packages. The configuration
service lookup path and active package precedence remain unverified; hash
matching alone is not proof of identity or use.
This reader does not choose package precedence or apply the value to gameplay.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path

from prop_numeric_probe import fields
from native_archive_index_probe import decode
from dbpf_extract import payload

TYPE = 0x00B1B104
SCOPE = 0
INSTANCE = 0xA4C2F6BD
FIELD = 0x79B4244B


def inspect_bytes(data):
    matches = [(typ, spec, words) for ident, typ, spec, words in fields(data) if ident == FIELD]
    if len(matches) != 1:
        raise ValueError('expected exactly one candidate numeric field')
    typ, spec, words = matches[0]
    if typ != 0x0D or spec in (0x001C, 0x009C) or len(words) != 1:
        raise ValueError('candidate must be a single FLOAT property')
    return {'field_id': f'{FIELD:08X}', 'float_bits': f'{words[0]:08X}',
            'payload_sha256': hashlib.sha256(data).hexdigest(),
            'name_status': 'candidate hash match; identity and consumer unverified'}


def inspect_package(package):
    with package.open('rb') as source:
        header = source.read(96)
        if len(header) != 96 or header[:4] != b'DBPF':
            raise ValueError('invalid DBPF header')
        count, size, offset = (struct.unpack_from('<I', header, pos)[0] for pos in (36, 44, 64))
        source.seek(offset)
        block = source.read(size)
    if len(block) != size:
        raise ValueError('truncated DBPF index')
    _, rows = decode(block, count)
    matches = [row for row in rows if row[:3] == (INSTANCE, TYPE, SCOPE)]
    if len(matches) != 1:
        raise ValueError('expected exactly one source resource in package')
    _, _, _, offset, stored, decoded, compression, _, _ = matches[0]
    if compression != 0 or stored != decoded:
        raise ValueError('version 2 only admits uncompressed matching-size payloads')
    entry = dict(offset=offset, stored=stored, dec=decoded)
    data = payload(package, entry)
    if len(data) != decoded:
        raise ValueError('payload size mismatch')
    return inspect_bytes(data)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('package', type=Path)
    args = parser.parse_args()
    print(json.dumps(inspect_package(args.package), indent=2))


if __name__ == '__main__':
    main()

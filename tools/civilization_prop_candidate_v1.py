"""Read two candidate Civilization-related numeric fields from an installed PROP.

Resource key and field identifiers match lowercase FNV-1 hashes of executable
CommunityEditor strings. Native functions D0AD90 and E91780 reference those
names, but the lookup-to-package path and active precedence remain unresolved.
Values are returned as raw bits; this module does not modify the prototype stage.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path

from dbpf_extract import payload
from native_archive_index_probe import decode
from prop_numeric_probe import fields

KEY = (0x6B3F727B, 0x00B1B104, 0)
FIELDS = {0x134CA1E3: ('CivicObjectCost', 0x09),
          0x231A946B: ('CityRotationRingScaleFactor', 0x0D)}
# These FLOAT IDs occur in two native consumers of the same resource key.
# Their meanings and whether those consumers run in Civilization remain unknown.
CONSUMER_FIELDS = {0x039D48E7, 0x78AEAF4B, 0xF61CE4B0}


def inspect_bytes(data):
    found = {}
    consumer = {}
    for ident, typ, spec, words in fields(data):
        if ident in CONSUMER_FIELDS:
            if ident in consumer or typ != 0x0D or spec in (0x001C, 0x009C) or len(words) != 1:
                raise ValueError('duplicate or unexpected native-consumer property')
            consumer[ident] = f'{words[0]:08X}'
            continue
        if ident not in FIELDS:
            continue
        name, expected_type = FIELDS[ident]
        if name in found or typ != expected_type or spec in (0x001C, 0x009C) or len(words) != 1:
            raise ValueError('duplicate or unexpected candidate property')
        found[name] = {'field_id': f'{ident:08X}', 'type': f'{typ:04X}',
                       'value_bits': f'{words[0]:08X}'}
    if len(found) != len(FIELDS) or len(consumer) != len(CONSUMER_FIELDS):
        raise ValueError('missing candidate or native-consumer property')
    return {'resource_key_instance_type_scope': [f'{n:08X}' for n in KEY],
            'payload_sha256': hashlib.sha256(data).hexdigest(), 'fields': found,
            'native_consumer_float_bits': {f'{ident:08X}': value for ident, value in consumer.items()},
            'status': 'candidate CommunityEditor name hashes; lookup-to-package path and active precedence unverified'}


def inspect_package(package):
    with package.open('rb') as source:
        header = source.read(96)
        if len(header) != 96 or header[:4] != b'DBPF':
            raise ValueError('invalid DBPF header')
        count, size, offset = (struct.unpack_from('<I', header, p)[0] for p in (36, 44, 64))
        if offset + size > package.stat().st_size:
            raise ValueError('index outside package')
        source.seek(offset)
        block = source.read(size)
    _, rows = decode(block, count)
    matches = [row for row in rows if row[:3] == KEY]
    if len(matches) != 1:
        raise ValueError('expected one candidate resource')
    _, _, _, offset, stored, decoded, compression, _, _ = matches[0]
    if offset + stored > package.stat().st_size:
        raise ValueError('resource outside package')
    if not ((compression == 0 and stored == decoded) or
            (compression == 0xFFFF and stored < decoded)):
        raise ValueError('unsupported compression or sizes')
    data = payload(package, dict(offset=offset, stored=stored, dec=decoded))
    if len(data) != decoded:
        raise ValueError('decoded size mismatch')
    return inspect_bytes(data)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('package', type=Path)
    args = parser.parse_args()
    print(json.dumps(inspect_package(args.package), indent=2))


if __name__ == '__main__':
    main()

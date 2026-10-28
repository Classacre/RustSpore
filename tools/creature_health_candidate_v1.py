"""Inspect one candidate Creature-related source property, without assigning gameplay semantics.

The local executable uses the spellings Combat and StandardCreatureHealth. Their
lowercase FNV-1 hashes match the instance and FLOAT field in resource
00B1B104:0001FFFF:A4C2F6BD (historical index labels) in both base and patch packages. The configuration
service lookup path and active package precedence remain unverified; hash
matching alone is not proof of identity or use.
This reader does not choose package precedence or apply the value to gameplay.
"""
import argparse
import hashlib
import json
from pathlib import Path

from prop_numeric_probe import fields, index
from dbpf_extract import payload

TYPE = 0x00B1B104
GROUP = 0x0001FFFF
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
    matches = [e for e in index(package) if (e['type'], e['group'], e['instance']) == (TYPE, GROUP, INSTANCE)]
    if len(matches) != 1:
        raise ValueError('expected exactly one source resource in package')
    return inspect_bytes(payload(package, matches[0]))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('package', type=Path)
    args = parser.parse_args()
    print(json.dumps(inspect_package(args.package), indent=2))


if __name__ == '__main__':
    main()

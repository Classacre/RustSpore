"""Read the source Cell Editor Setup model-template key array.

Static native consumers 00574680 and 00574710 select an indexed 12-byte key
and cycle the stored index, respectively. This is not a part catalog. We do not
implement 00A68ED0 or claim how its returned index is generated.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import xml.etree.ElementTree as ET

from cell_capability_cache_probe import named_property_records
from dbpf_extract import payload
from gmdl_skin_probe import indexes

SETUP = 0x406001003615A30B
PROPERTY = 0x503D2F60


def array_keys(raw):
    fields = [r for r in named_property_records(raw) if r[0] == PROPERTY]
    if len(fields) != 1:
        raise ValueError('expected exactly one template property')
    _, typ, spec, value = fields[0]
    if typ != 0x20 or spec not in (0x1c, 0x9c) or len(value) < 8:
        raise ValueError('unexpected template array framing')
    count, stride = struct.unpack_from('>II', value)
    if count > 1024 or stride != 12 or len(value) != 8 + count * stride:
        raise ValueError('unexpected template array size')
    return [struct.unpack_from('<III', value, 8 + i * stride) for i in range(count)]


def next_index(index, count):
    # Native signed remainder agrees on this bounded reachable state subset.
    if count <= 0 or not 0 <= index < count:
        raise ValueError('index must identify an existing element')
    return (index + 1) % count


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('package', type=Path)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    a.out.resolve().relative_to((Path(__file__).resolve().parent.parent / 'target').resolve())
    if a.out.exists():
        raise FileExistsError(a.out)
    entries = indexes(a.package)
    setups = [e for e in entries if e['type'] == 0x00B1B104 and e['instance'] == SETUP]
    if len(setups) != 1:
        raise ValueError('expected one setup record in supplied package')
    setup = payload(a.package, setups[0])
    keys = array_keys(setup)
    records = []
    for ordinal, (low, typ, scope) in enumerate(keys):
        found = [e for e in entries if e['type'] == typ and e['instance'] == (scope << 32 | low)]
        if len(found) != 1:
            raise ValueError(f'missing or ambiguous template {ordinal}')
        raw = payload(a.package, found[0])
        if len(raw) != found[0]['dec']:
            raise ValueError('decoded size mismatch')
        root = ET.fromstring(raw)
        if root.tag != 'sporemodel':
            raise ValueError('unexpected source XML root')
        blocks = root.find('blocks')
        if blocks is None:
            raise ValueError('missing source blocks')
        records.append({'ordinal': ordinal, 'low': f'{low:08X}', 'type': f'{typ:08X}',
            'scope': f'{scope:08X}', 'decoded_sha256': hashlib.sha256(raw).hexdigest(),
            'decoded_bytes': len(raw), 'formatversion': root.findtext('formatversion'),
            'modeltype': root.findtext('properties/modeltype'),
            'block_ids': [child.findtext('blockid') for child in blocks],
            'next_ordinal': next_index(ordinal, len(keys))})
    result = {'scope': 'One explicit package copy of Cell Editor Setup property 503D2F60. Native consumers select a key or cycle it. This is not a part catalog or eligibility list. Package priority, caller state, RNG, and full XML semantics remain unresolved.',
        'package': str(a.package.resolve()), 'package_sha256': hashlib.file_digest(a.package.open('rb'), 'sha256').hexdigest(),
        'setup_sha256': hashlib.sha256(setup).hexdigest(), 'count': len(keys), 'templates': records}
    a.out.parent.mkdir(parents=True, exist_ok=True)
    with a.out.open('x', encoding='utf-8') as f:
        json.dump(result, f, indent=2)
        f.write('\n')
    print(json.dumps({'count': len(keys), 'setup_sha256': result['setup_sha256'], 'output': str(a.out)}, indent=2))

if __name__ == '__main__':
    main()

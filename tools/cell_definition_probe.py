"""Independent Cell fixtures using retained native descriptor offsets/types.

The schema input is ignored native evidence, not a Rust-derived layout. Source
resources stay local. This reads every package index, preserving each matching
resource rather than silently coalescing overlay copies.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import struct

from dbpf_extract import payload
from gmdl_skin_probe import indexes as package_index


def schema_fields(rows):
    classes = {name: [r for r in rows if r['struct'] == name and r['offset'] != 0xffffffff]
               for name in ['cell', 'cAIData', 'cEatData', 'cLocalizedString']}
    nested = {0x16b1a08: 'cAIData', 0x16b1ec4: 'cEatData', 0x16b214c: 'cLocalizedString'}
    leaves = []

    def visit(name, base):
        for row in classes[name]:
            kind = row.get('native_type')
            if kind is None:
                # Retain compatibility with the first ignored SDK-store ledger.
                encoded = row['raw'][4]
                match = re.fullmatch(r'call0094E9D0\((\d+)\)', encoded)
                if not match:
                    raise ValueError(f'unestablished native field type: {encoded}')
                kind = int(match[1])
            at = base + row['offset']
            if kind in nested:
                visit(nested[kind], at)
                continue
            size = {1: 4, 2: 4, 3: 4, 5: 4, 6: 1, 10: 0xa0, 11: 8}.get(kind)
            if size is None:
                raise ValueError(f'unsupported native primitive {kind}')
            leaves.append((at, kind, size, name, row['name']))

    visit('cell', 0)
    leaves.sort()
    end = 0
    for at, _, size, _, _ in leaves:
        assert end <= at < at+size <= 0x31c
        end = at+size
    assert len(leaves) == 176, len(leaves)
    return leaves


def digest(data, fields):
    value = 0xcbf29ce484222325
    for at, kind, size, _, _ in fields:
        for byte in struct.pack('<III', at, kind, size) + data[at:at+size]:
            value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return value


def rules_digest(data, rows):
    """Native E573E0 and bounded E7D0A0 branches, read directly from bytes."""
    offsets = {(r['struct'], r['name']): r['offset'] for r in rows}
    read_u = lambda at: struct.unpack_from('<I', data, at)[0]
    read_i = lambda at: struct.unpack_from('<i', data, at)[0]
    wrap = lambda value: ((value + 0x80000000) & 0xffffffff) - 0x80000000
    eat = offsets['cell', 'eat']
    hp = read_i(eat + offsets['cEatData', 'hpValue'])
    food = read_i(eat + offsets['cEatData', 'foodValue'])
    bomb = data[eat + offsets['cEatData', 'bomb']] != 0
    nova = data[eat + offsets['cEatData', 'poisonNova']] != 0
    result = bytearray()
    for difficulty in [-1, 0, 1, 2, 3]:
        ai = offsets['cell', 'ai']
        if difficulty == 0:
            candidate = offsets['cell', 'ai_easy']
            if read_u(candidate) != 0xffffffff:
                ai = candidate
        elif difficulty == 2:
            candidate = offsets['cell', 'ai_hard']
            if read_u(candidate) != 0xffffffff:
                ai = candidate
        result += struct.pack('<iIBB', difficulty, ai, read_u(ai) != 0, True)
        for present in [False, True]:
            for health in [-1, 0, 1, 4, 5, 6, 7]:
                after = health
                if after < 6:
                    if not present:
                        after = wrap(after + 2)
                    elif hp > 0:
                        addition = wrap(hp*2) if difficulty == 0 else hp
                        after = wrap(after + addition)
                after = min(after, 6)
                reward = food if present else 10
                result += struct.pack('<iBiBiBB', health, present, after, reward != 0,
                    reward, present and bomb, present and nova)
    value = 0xcbf29ce484222325
    for byte in result:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return value


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('install', type=Path)
    ap.add_argument('--metadata', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    metadata = json.loads(args.metadata.read_text())
    fields = schema_fields(metadata)
    args.out.mkdir(parents=True, exist_ok=False)
    records = []
    indexes = matching_packages = 0
    booleans = Counter()
    floats = nonfinite = 0
    with (args.out/'corpus.bin').open('xb') as output, (args.out/'rules.tsv').open('x') as rules:
        output.write(b'RSCELLD1' + bytes(4))
        for package in sorted(args.install.rglob('*.package')):
            entries = package_index(package)
            indexes += 1
            selected = [e for e in entries if e['type'] == 0xdfad9f51]
            matching_packages += bool(selected)
            for entry in selected:
                data = payload(package, entry)
                if len(data) != 796:
                    raise ValueError(f'wrong Cell resource extent in {package}')
                units = struct.unpack_from('<80H', data, 4)
                label = struct.pack('<80H', *units).decode('utf-16le').split('\0')[0]
                expected = digest(data, fields)
                output.write(struct.pack('<QIQ', entry['instance'], len(data), expected) + data)
                rules.write(f'{len(records)}\t{entry["instance"]:016X}\t{rules_digest(data, metadata):016X}\n')
                for at, kind, size, _, _ in fields:
                    if kind == 6:
                        booleans[data[at]] += 1
                    if kind == 2:
                        bits, = struct.unpack_from('<I', data, at)
                        floats += 1
                        nonfinite += (bits & 0x7f800000) == 0x7f800000
                records.append(dict(package=str(package.relative_to(args.install)),
                    key=f"{entry['group']:08X}:{entry['type']:08X}:{entry['instance']:016X}",
                    label=label, sha256=hashlib.sha256(data).hexdigest(),
                    digest=f'{expected:016X}'))
        output.seek(8)
        output.write(struct.pack('<I', len(records)))
    report = dict(indexes=indexes, matching_packages=matching_packages,
        resources=len(records), unique_ids=len({r['key'] for r in records}),
        fields_per_definition=len(fields), typed_field_values=len(records)*len(fields),
        source_float_fields=floats, nonfinite_float_fields=nonfinite,
        boolean_byte_counts=dict(sorted(booleans.items())),
        metadata_sha256=hashlib.sha256(args.metadata.read_bytes()).hexdigest(),
        fixture_sha256=hashlib.sha256((args.out/'corpus.bin').read_bytes()).hexdigest(),
        rule_resources=len(records), ai_selection_cases=len(records)*5,
        ai_gate_cases=len(records)*10, eating_numeric_cases=len(records)*70,
        rules_sha256=hashlib.sha256((args.out/'rules.tsv').read_bytes()).hexdigest(),
        complete_gameplay_algorithms_recovered=False, records=records)
    (args.out/'inventory.json').write_text(json.dumps(report, indent=2))
    print(json.dumps({k: v for k, v in report.items() if k != 'records'}, indent=2))


if __name__ == '__main__':
    main()

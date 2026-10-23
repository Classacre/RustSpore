"""Prepare all source capability rows and bounded synthetic contexts; no native execution."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import random
import struct

from anim_body_hierarchy_probe import package_entries, MASKS
from anim_body_capabilities_native_probe import INPUT_BYTES
from anim_quaternion_probe import CONTROLS
from creature_header_probe import header, inspect
from dbpf_extract import payload
from part_capabilities_probe import inspect as inspect_dictionary

ROOT = Path(__file__).resolve().parent.parent


def prepare(ids, numbers, aggregates):
    ids, numbers = list(ids), list(numbers)
    end = ids.index(0)
    for i in range(32):
        if not ids[i]:
            break
        members = aggregates.get(ids[i], [])
        if end+len(members) > 32:
            raise ValueError('aggregate capacity')
        for member in members:
            ids[end], numbers[end] = member, numbers[i]
            end += 1
    for i, identifier in enumerate(ids):
        if identifier:
            for j in range(i+1, 32):
                if ids[j] == identifier:
                    ids[j] = 0
                    numbers[i] = max(numbers[i], numbers[j])
    holes = [i for i, identifier in enumerate(ids) if not identifier]
    occupied = [i for i, identifier in enumerate(ids) if identifier]
    # Independent two-index compaction specification, retaining number bytes.
    while holes and occupied and holes[0] < occupied[-1]:
        a, b = holes.pop(0), occupied.pop()
        ids[a], numbers[a], ids[b] = ids[b], numbers[b], 0
    present = nonzero = 0
    for identifier, number in zip(ids, numbers):
        if not identifier:
            break
        flag = MASKS.get(struct.pack('<I', identifier), 0)
        present |= flag
        if number:
            nonzero |= flag
    return struct.pack('<34I32B', present, nonzero, *ids, *numbers)


def request(ids, numbers, aggregates, control):
    assert len(ids) == len(numbers) == 32 and len(aggregates) <= 8
    data = struct.pack('<4I32I32B', control, len(aggregates), 0, 0, *ids, *numbers)
    for identifier, members in sorted(aggregates.items()):
        assert len(members) <= 32
        data += struct.pack('<34I', identifier, len(members), *members, *([0]*(32-len(members))))
    data += bytes((8-len(aggregates))*136)
    assert len(data) == INPUT_BYTES
    return data


def feed(value, data):
    for byte in data:
        value = ((value ^ byte)*0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def synthetic():
    spin, limb, foot = [int.from_bytes(value, 'little') for value in (b'spin', b'limb', b'foot')]
    def case(family, initial, aggregates, stale=None):
        ids, numbers = [0]*32, [0]*32 if stale is None else list(stale)
        for i, (identifier, number) in enumerate(initial):
            ids[i], numbers[i] = identifier, number
        return family, ids, numbers, aggregates
    yield case('empty', [], {})
    yield case('duplicate-max-tail-order', [(10, 1), (20, 2), (10, 7)], {10: [spin, limb], 20: [foot]})
    yield case('zero-member-stops-before-compaction', [(10, 1), (0, 9), (20, 2), (30, 3)], {10: [0], 20: [spin]})
    yield case('complete-expanded32', [(10, 5)], {10: list(range(100, 131))})
    yield case('initial31', [(i+100, i) for i in range(31)], {})
    yield case('recursive-chain', [(10, 9)], {10: [20], 20: [30], 30: [40], 40: [spin]})
    yield case('unsigned-tree-keys', [(0xFFFFFFFF, 7), (1, 2), (0x80000000, 9)],
               {0: [foot], 1: [spin], 0x7FFFFFFF: [limb], 0x80000000: [foot], 0xFFFFFFFF: [limb]})
    yield case('presence-with-zero-number', [(spin, 0), (foot, 7)], {})
    rng = random.Random(0x9FC2F0)
    flags = list(int.from_bytes(value, 'little') for value in MASKS)
    keys = list(range(1000, 1008))
    for n in range(1024):
        aggregates = {key: [rng.choice(flags+[0, 77, 0xFFFFFFFF]) for _ in range(rng.randrange(3))]
                      for key in keys[:rng.randrange(9)]}
        initial = [(rng.choice(flags+keys+[0, 77, 0xFFFFFFFF]), rng.randrange(256))
                   for _ in range(rng.randrange(11))]
        yield case(f'synthetic-{n}', initial, aggregates, [rng.randrange(256) for _ in range(32)])


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('install', type=Path)
    p.add_argument('--out', type=Path, required=True)
    args = p.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve())
    args.out.mkdir(parents=True, exist_ok=False)
    packages = [(path, package_entries(path)) for path in sorted(args.install.rglob('*.package'))]
    dictionaries = []
    for path, entries in packages:
        for entry in entries:
            if entry['type'] == 0x7C19AA7A:
                data = payload(path, entry)
                value = inspect_dictionary(data)
                value.update(package=path.relative_to(args.install).as_posix(),
                    native_key=f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:08X}")
                dictionaries.append(value)
    assert len(dictionaries) == 3
    aggregate_tables = [{int(item['capability'], 16): [int(v, 16) for v in item['members']]
                        for item in d['aggregates']} for d in dictionaries]
    assert all(table == aggregate_tables[0] for table in aggregate_tables)
    table = aggregate_tables[0]
    assert len(table) == 8 and max(map(len, table.values())) == 1
    normalized_table = b''.join(struct.pack('<2I', key, len(members))+struct.pack('<'+'I'*len(members), *members)
                                for key, members in sorted(table.items()))
    input_hash, expected_hash, source_hash = hashlib.sha256(), hashlib.sha256(), hashlib.sha256()
    index, resources, lines, counts = [], [], [], Counter()
    with (args.out/'inputs.bin').open('xb') as stream, (args.out/'python-output.bin').open('xb') as expected, (args.out/'source-rows.bin').open('xb') as source_rows:
        def emit(ids, numbers, aggregates, cw, metadata):
            record = request(ids, numbers, aggregates, cw)
            output = prepare(ids, numbers, aggregates)
            stream.write(record); expected.write(output)
            input_hash.update(record); expected_hash.update(output)
            metadata.update(row=len(index), cw=f'{cw:04X}')
            index.append(metadata)
            return record, output
        for path, entries in packages:
            relative = path.relative_to(args.install).as_posix()
            for entry in entries:
                if entry['type'] != 0x0F43029A:
                    continue
                data = payload(path, entry)
                inspect(data, Counter())
                version = struct.unpack_from('<I', data, 4)[0]
                count, cap_count = struct.unpack_from('<2I', data, 8)
                start, _, _, _ = header(data, 20 if version == 9 else 24)
                cap_at = start+count*140
                cap_ids = list(struct.unpack_from('<'+'I'*cap_count, data, cap_at))
                cap_numbers = data[cap_at+cap_count*4:cap_at+cap_count*5]
                key = f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:08X}"
                digest = feed(0xCBF29CE484222325, b'native-body-capabilities-v1\0'+struct.pack('<I', count))
                first = len(index)
                for part in range(count):
                    raw = data[start+part*140:start+(part+1)*140]
                    cap_first = struct.unpack_from('<H', raw, 12)[0]
                    size = raw[16]
                    assert size <= 31
                    ids, numbers = [0]*32, [0]*32
                    ids[:size], numbers[:size] = cap_ids[cap_first:cap_first+size], cap_numbers[cap_first:cap_first+size]
                    record, result = emit(ids, numbers, table, 0x027F,
                        dict(family='source', package=relative, key=key, part=part))
                    source_rows.write(raw); source_hash.update(raw)
                    digest = feed(digest, struct.pack('<I', part)+raw+record[16:176]+result)
                    counts['source_rows'] += 1
                resources.append(dict(package=relative, key=key, first_row=first, source_rows=count,
                    source_sha256=hashlib.sha256(data).hexdigest(), expected_fnv1a64=f'{digest:016X}'))
                lines.append(f'{relative} {key} {digest:016X}')
        for family, ids, numbers, aggregates in synthetic():
            for cw in CONTROLS:
                emit(ids, numbers, aggregates, cw, dict(family=family))
                counts['synthetic_rows'] += 1
    assert counts['source_rows'] == 46418 and len(resources) == 1365 and len(packages) == 82
    encoded_index = (json.dumps(index, indent=2)+'\n').encode()
    (args.out/'input-index.json').write_bytes(encoded_index)
    (args.out/'source-resources.json').write_text(json.dumps(resources, indent=2)+'\n')
    (args.out/'dictionaries.json').write_text(json.dumps(dictionaries, indent=2)+'\n')
    normalized = '\n'.join(sorted(lines))+'\n'
    (args.out/'python-source-digests.txt').write_text(normalized, newline='\n')
    manifest = dict(schema='native-body-capability-input-cohort-v1', host_executed=False,
        package_indexes=len(packages), source_resources=len(resources), rows=len(index), counts=dict(counts),
        input_bytes=INPUT_BYTES, expected_bytes_per_row=168, input_sha256=input_hash.hexdigest(),
        input_index_sha256=hashlib.sha256(encoded_index).hexdigest(), python_output_sha256=expected_hash.hexdigest(),
        source_rows_sha256=source_hash.hexdigest(), source_digests_sha256=hashlib.sha256(normalized.encode()).hexdigest(),
        source_dictionary_resources=[dict(package=d['package'], key=d['native_key'], source_sha256=d['source_sha256'], version=d['version']) for d in dictionaries],
        aggregate_tables_identical=True, aggregate_count=8, maximum_source_members=1,
        canonical_aggregate_table_sha256=hashlib.sha256(normalized_table).hexdigest(),
        scope='All46418fresh source rows under an explicit aggregate context. All3installed PCTP aggregate tables are byte-equivalent after canonical decoding, so dictionary overlay choice does not alter this operation. Synthetic contexts cover slot ordering, stale numbers, holes/zero members, recursive expansion, full-expanded32, unsigned tree keys, masks and all12CWs. No native execution or finalized hierarchy claim.')
    (args.out/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps(manifest, indent=2))


if __name__ == '__main__':
    main()

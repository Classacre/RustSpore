"""Freeze bounded contribution requests and exact authored endpoint provenance.

Preparation only: no compilation, native host, retail process or GUI.
"""
import argparse
from collections import Counter
import csv
import hashlib
import itertools
import json
from pathlib import Path
import random
import struct

import cell_runtime_contributions_native_probe as host
from cell_runtime_curves_fixture import CONTEXTS, sha_file

ROOT = Path(__file__).resolve().parent.parent
RAW = [0, 0x80000000, 1, 0x80000001, 0x007FFFFF, 0x00800000,
       0x3F000000, 0xBF000000, 0x3F800000, 0xBF800000, 0x3F800001,
       0x7F7FFFFF, 0xFF7FFFFF, 0x7F800000, 0xFF800000,
       0x7FC12345, 0xFFC54321, 0x7F800001, 0xFF800123]
COLUMNS = ['case', 'family', 'cw', 'package', 'low', 'type', 'scope',
           'source_sha256', 'channel', 'component', 'row', 'source_byte_offset']


def descriptor(flags=0x93, tag=0x98C942B5, slot=4):
    return struct.pack('<8I', flags, tag, 0, 40, slot, 0, 0, 0)


def aggregate(count=0, duplicate=False, raw=0):
    result = bytearray(b'\xA5'*164)
    struct.pack_into('<I', result, 0, count)
    for i in range(count):
        struct.pack_into('<5I', result, 4+20*i,
            0x98C942B5 if duplicate or i == 0 else 0x1000+i,
            0xDEAD0000+i, raw, 0xCAFE0000+i, raw)
    return result


def request(descriptors, slots=None, blend=0x3F800000, mask=None, pos=0, rot=0,
            cw=0x027F, initial=None):
    assert len(descriptors) <= 32 and all(len(x) == 32 for x in descriptors)
    if slots is None:
        slots = [(0x3F000000, 0x3F800000)]*6
    assert len(slots) == 6
    if mask is None:
        mask = (1 << min(len(descriptors), 24))-1
    raw = struct.pack('<IHH6I', blend, cw, 0, mask, pos, rot, len(descriptors), 0, 0)
    raw += bytes(aggregate() if initial is None else initial)
    raw += b''.join(struct.pack('<2I', *slot) for slot in slots)
    raw += b''.join(descriptors)+bytes((32-len(descriptors))*32)
    assert len(raw) == host.INPUT_BYTES
    return raw


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host-dir', type=Path, required=True)
    parser.add_argument('--source-rows', type=Path, default=ROOT/'target/source-cell/runtime-data-source-curves-v1')
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve())
    manifest = json.loads((args.host_dir/'manifest.json').read_text())
    assert manifest['host_source_sha256'] == sha_file(args.host_dir/'host-x86.asm')
    assert manifest['preparation_tool_sha256'] == sha_file(Path(host.__file__))
    source = json.loads((args.source_rows/'report.json').read_text())
    assert source['failures'] == [] and source['statistics']['rows'] == 44127
    for name, expected in source['files'].items():
        assert sha_file(args.source_rows/name) == expected
    args.out.mkdir(parents=True, exist_ok=False)
    families, contexts = Counter(), Counter()
    cases = source_cases = 0
    with (args.out/'input.bin').open('xb') as binary, (args.out/'cases.tsv').open('x', newline='') as table:
        writer = csv.writer(table, delimiter='\t', lineterminator='\n')
        writer.writerow(COLUMNS)

        def emit(raw, family, item=None, row='-', offset='-'):
            nonlocal cases, source_cases
            cw = struct.unpack_from('<H', raw, 4)[0]
            identity = ['-']*7 if item is None else [item[k] for k in
                ['package', 'low', 'type', 'scope', 'source_sha256', 'channel', 'component']]
            writer.writerow([cases, family, f'{cw:04X}', *identity, row, offset])
            binary.write(raw)
            cases += 1
            families[family] += 1
            contexts[f'{cw:04X}'] += 1
            source_cases += item is not None

        for cw, blend in itertools.product(CONTEXTS, RAW):
            emit(request([descriptor()], blend=blend, cw=cw), 'outer_blend_raw')
        for cw, value, weight in itertools.product(CONTEXTS, RAW, RAW):
            emit(request([descriptor()], [(value, weight)]*6, cw=cw), 'raw_products')
        for cw, count, duplicate, flags in itertools.product(CONTEXTS, range(9), [False, True], [3, 0x83]):
            ds = [descriptor(flags, 0x98C942B5), descriptor(flags ^ 0x80, 0xABCD1234),
                  descriptor(flags, 0x98C942B5)]
            emit(request(ds, cw=cw, initial=aggregate(count, duplicate, RAW[count])), 'ordered_duplicate_capacity')
        for cw, n, pos, rot in itertools.product(CONTEXTS, [0, 1, 8, 24, 32], [0, 0x555555, 0xFFFFFFFF], [0, 0xAAAAAA, 0xFFFFFFFF]):
            ds = [descriptor([0x93, 0x83, 1, 2][i % 4], 0x1000+i, 3+i % 6) for i in range(n)]
            emit(request(ds, cw=cw, pos=pos, rot=rot), 'literal_masks_and_descriptor_capacity')
        # Invalid slot/angular fields are consumed only after mask and kind tests.
        for cw, flags, slot, mask in itertools.product(CONTEXTS, [1, 2, 0x43], [0, 9, 0xFFFFFFFF], [0, 1]):
            if flags == 0x43 and mask:
                continue
            emit(request([descriptor(flags, slot=slot)], cw=cw, mask=mask), 'unreached_unsupported_fields')
        rng = random.Random(0x99D190)
        for i in range(3072):
            n, count = rng.randrange(33), rng.randrange(9)
            ds = [descriptor(rng.choice([3, 0x83, 0x93, 0xB3, 1, 2]),
                             rng.choice([0x98C942B5, 0x98C942B6, 0x1000, 0x1001]), rng.randrange(3, 9)) for _ in range(n)]
            slots = [(rng.getrandbits(32), rng.getrandbits(32)) for _ in range(6)]
            emit(request(ds, slots, rng.choice(RAW), rng.getrandbits(24), rng.getrandbits(32),
                         rng.getrandbits(32), CONTEXTS[i % 12], aggregate(count, i % 2, rng.choice(RAW))), 'seeded_raw_ordered')
        # Each original value/weight row exactly once, at its source component bit.
        # Other descriptors/slot values are explicit synthetic masked placeholders.
        with (args.source_rows/'components.tsv').open(newline='') as table_in, (args.source_rows/'source-INFO20-custom40-rows.bin').open('rb') as rows_in:
            components = list(csv.DictReader(table_in, delimiter='\t'))
            assert len(components) == 2086
            for item in components:
                index, slot = int(item['component']), int(item['prepared_slot'], 16)
                flags, tag = int(item['prepared_flags'], 16), int(item['tag'], 16)
                assert index < 24 and flags & 0x40 == 0 and 3 <= slot <= 8
                ds = [descriptor(0, 0, 0)]*index+[descriptor(flags, tag, slot)]
                base = int(item['rows_byte_offset'])
                for row in range(int(item['rows'])):
                    at = base+60*row+20
                    rows_in.seek(at)
                    value, weight = struct.unpack('<2I', rows_in.read(8))
                    slots = [(0, 0)]*6
                    slots[slot-3] = (value, weight)
                    emit(request(ds, slots, mask=1 << index, cw=CONTEXTS[row % 12]),
                         'source_raw_row_explicit_isolated_descriptor', item, row, at)
    assert source_cases == 44127
    report = dict(schema='bounded-cell-runtime-contributions-inputs-v1', cases=cases,
        source_cases=source_cases, source_components=2086, prepared=True, native_executed=False,
        families=dict(families), contexts=dict(contexts), input_bytes=cases*host.INPUT_BYTES,
        protocol=dict(input_bytes=host.INPUT_BYTES, output_bytes=host.OUTPUT_BYTES),
        hashes={name:sha_file(args.out/name) for name in ['input.bin','cases.tsv']},
        tool_sha256=sha_file(Path(__file__)), host_manifest_sha256=sha_file(args.host_dir/'manifest.json'),
        source_rows_report_sha256=sha_file(args.source_rows/'report.json'), source_rows_files=source['files'],
        limitations='Authored raw endpoint pairs, not live sampled curves; one original selected scalar descriptor at its literal index with explicit synthetic masked placeholders. No live clip, clock, selector or pose binding claim. All raw nonfinite outcomes must compare; unsupported adapter policy tests are separate positive errors in Rust verifier.')
    (args.out/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

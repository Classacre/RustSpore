"""Plan or stream full-channel original custom-curve requests; never execute.

Default mode writes a small exact census/size plan. --prepare additionally
writes flat requests and per-case metadata. Memory stays bounded per channel.
"""
import argparse
from collections import Counter
import csv
import itertools
import json
from pathlib import Path
import struct

import cell_runtime_curves_fixture as source
import cell_runtime_curves_native_probe_v3 as host

ROOT = Path(__file__).resolve().parent.parent
COLUMNS = ['case', 'family', 'package', 'low', 'type', 'scope', 'source_sha256',
           'channel', 'component', 'frame_bits', 'cw', 'flags', 'slot', 'upper',
           'previous', 'left', 'right', 'following', 'expected_policy', 'source_component_ordinal']


def row(frame, index, modes=(0, 0, 0, 0)):
    info = struct.pack('<5I', frame, 0xA5A50000+index, 0x00FF0000+index,
                       0xAABB0000+index, 0xCCDD0000+index)
    raw = bytearray(range(40))
    values = [-2.5, .25, 1.5, -.75]
    weights = [-.5, 1.125, .75, 2.5]
    slopes = [.375, -1.25, .75, -0.]
    struct.pack_into('<2f', raw, 0, values[index % 4], weights[index % 4])
    struct.pack_into('<2f', raw, 8, slopes[index % 4], slopes[(index+1) % 4])
    struct.pack_into('<2f', raw, 24, slopes[(index+2) % 4], slopes[(index+3) % 4])
    raw[16], raw[17], raw[32], raw[33] = modes
    return info+raw


def request(frame, rows, duration, wrap, cw, flags, slot):
    assert len(rows) % 60 == 0 and len(rows)//60 <= 160
    assert cw in source.CONTEXTS
    choice = host.POLICIES.index((flags, slot))
    header = struct.pack('<8I', source.bits(frame), len(rows)//60, source.bits(duration),
                         8 if wrap else 0, cw, choice, 0, 0)
    raw = header+rows+bytes(160*60-len(rows))
    assert len(raw) == host.INPUT_BYTES
    return raw


def generate(args):
    args.out.mkdir(parents=True, exist_ok=False)
    manifest = json.loads((args.host_dir/'manifest.json').read_text())
    assert manifest['schema'] == 'bounded-original-cell-runtime-curves-host-v3'
    assert manifest['protocol']['input_bytes'] == host.INPUT_BYTES == 9632
    assert manifest['protocol']['output_bytes'] == host.OUTPUT_BYTES == 476
    assert source.sha_file(Path(host.__file__)) == manifest['preparation_tool_sha256']
    census = json.loads((args.census_dir/'report.json').read_text())
    assert source.sha_file(args.census_dir/'components.jsonl') == census['components_sha256']
    assert source.sha_file(args.source_rows/'report.json') == census['source_rows_report_sha256']
    for name, expected in census['source_rows_files'].items():
        assert source.sha_file(args.source_rows/name) == expected
    ordinal, source_cases = 0, 0
    families, contexts, flags_counts, slots, counts, policies = (Counter() for _ in range(6))
    binary = (args.out/'input.bin').open('xb') if args.prepare else None
    table = (args.out/'cases.tsv').open('x', newline='\n') if args.prepare else None
    try:
        if table is not None:
            table.write('\t'.join(COLUMNS)+'\n')

        def emit(frame, rows, duration, wrap, cw, flags, slot, family, policy='-', item=None):
            nonlocal ordinal, source_cases
            count = len(rows)//60
            times = [source.word(rows, 60*i) for i in range(count)]
            upper = source.upper_index(times, frame, cw)
            neighbors = source.neighbors(count, upper, wrap)
            if binary is not None:
                binary.write(request(frame, rows, duration, wrap, cw, flags, slot))
                identity = ['-', '-', '-', '-', '-', '0', '1'] if item is None else [
                    item[k] for k in ['package', 'low', 'type', 'scope', 'source_sha256', 'channel', 'component']]
                metadata = [str(ordinal), family, *identity, f'{source.bits(frame):08X}', f'{cw:04X}',
                    f'{flags:08X}', str(slot), str(upper), *map(str, neighbors), policy,
                    '-' if item is None else str(item['component_ordinal'])]
                assert len(metadata) == len(COLUMNS)
                table.write('\t'.join(metadata)+'\n')
            families[family] += 1
            contexts[f'{cw:04X}'] += 1
            flags_counts[f'{flags:08X}'] += 1
            slots[str(slot)] += 1
            counts[str(count)] += 1
            policies[policy] += 1
            source_cases += item is not None
            ordinal += 1

        # A positive endpoint smoke comes first for retained protocol reference.
        emit(0., row(0, 1), 1., False, 0x027F, 0x93, 4, 'smoke_endpoint40')
        # Independently vary previous/next tangent modes of both outputs.
        for cw, flags, wrap, modes in itertools.product(source.CONTEXTS, [0x93, 0xB3], [False, True],
                                                       itertools.product(range(4), repeat=4)):
            frames = [0, 3, 11, 17]
            rows = b''.join(row(frame, index, modes) for index, frame in enumerate(frames))
            emit(6.25, rows, 19., wrap, cw, flags, 3, 'synthetic_all_two_scalar_modepairs')
        time_patterns = [([0, 1, 97, 131], 50.25, 132.),
            ([0x1000000, 0x1000001, 0x1000005, 0x1000011], source.f32(0x1000004), source.f32(0x1000012)),
            ([0x7FFFFE00, 0x80000001, 0x80000403, 0x80000811], source.f32(0x80000200), source.f32(0x80001000)),
            ([0xFFFFFF00, 0xFFFFFF40, 0xFFFFFFC0, 0xFFFFFFFF], source.f32(0xFFFFFF80), source.f32(0x100000000)),
            ([0x7FFFFE01, 0x7FFFFE02, 0x7FFFFE40, 0x7FFFFE80], source.f32(0x7FFFFE40), source.f32(0x80000100))]
        for cw, flags, wrap, (frames, frame, duration), a, b in itertools.product(
                source.CONTEXTS, [0x93, 0xB3], [False, True], time_patterns, range(4), range(4)):
            rows = b''.join(row(key, index, (a, b, 3-a, 3-b)) for index, key in enumerate(frames))
            emit(frame, rows, duration, wrap, cw, flags, 8, 'synthetic_unsigned_and_asymmetric_times')
        # Every immutable descriptor policy and capacity, with enabled/disabled
        # interpolation and complete40-byte copies. No cropped row windows.
        for cw, (flags, slot), wrap, count in itertools.product(
                source.CONTEXTS, host.POLICIES, [False, True], [0, 1, 2, 4, 16, 133, 160]):
            frames = [2*i for i in range(count)]
            rows = b''.join(row(key, index, (1, 1, 1, 1)) for index, key in enumerate(frames))
            samples = [0.] if not count else [-1., 0., .75, float(frames[-1]), float(frames[-1]+1)]
            for frame in dict.fromkeys(samples):
                emit(frame, rows, float(max(count*2, 1)), wrap, cw, flags, slot,
                     'synthetic_full_capacity_enable_and_copy')
        for cw, flags, wrap, frame in itertools.product(source.CONTEXTS, source.FLAGS, [False, True], [-0., 1., 2., 3.]):
            rows = b''.join(row(key, index, (1, 1, 1, 1)) for index, key in enumerate([0, 0, 2, 2]))
            emit(frame, rows, 4., wrap, cw, flags, 5, 'synthetic_duplicate_time_selection')
        # Explicit finite-API policy inputs. Native outcomes, including clamp
        # masking and raw endpoint signaling-NaN copies, remain retained.
        for cw, flags, wrap, kind in itertools.product(source.CONTEXTS, [0x93, 0xB3], [False, True], range(8)):
            frames = [0, 3, 11, 17]
            rows = bytearray(b''.join(row(key, index, (1, 1, 1, 1)) for index, key in enumerate(frames)))
            frame, policy = 6.25, 'NonFiniteInput'
            if kind == 0:
                struct.pack_into('<I', rows, 60+20, 0x7FC12345)
            elif kind == 1:
                struct.pack_into('<I', rows, 60+24, 0x7F800001)
            elif kind == 2:
                struct.pack_into('<I', rows, 120+20, 0x7F800000)
            elif kind == 3:
                struct.pack_into('<I', rows, 120+24, 0xFF800000)
            elif kind == 4:
                rows[60+20+17] = 3
                struct.pack_into('<I', rows, 60+20+12, 0x7FC54321)
            elif kind == 5:
                frame = -1.
                struct.pack_into('<I', rows, 20, 0x7F800001)
            elif kind == 6:
                frame, policy = -1., '-'
                struct.pack_into('<I', rows, 20+8, 0x7F800001)
            else:
                policy = '-'
                struct.pack_into('<I', rows, 20, 0x7FC12345)
            emit(frame, bytes(rows), 19., wrap, cw, flags, 6,
                 'synthetic_explicit_nonfinite_policy' if policy != '-' else 'synthetic_unused_nonfinite_fields', policy)
        # Actual source components preserve full channel row counts/times,
        # duration and the exact native-prepared flags/slot. Native9A14C0 will
        # select upper from these full rows; no upper field enters the request.
        with (args.census_dir/'components.jsonl').open() as components, \
                (args.source_rows/'source-INFO20-custom40-rows.bin').open('rb') as source_rows:
            for line in components:
                item = json.loads(line)
                source_rows.seek(int(item['rows_byte_offset']))
                rows = source_rows.read(int(item['rows'])*60)
                assert len(rows) == int(item['rows'])*60
                duration = source.number(int(item['source_duration_bits'], 16))
                wrap = bool(int(item['source_header_flags'], 16) & 8)
                flags, slot = int(item['prepared_flags'], 16), int(item['prepared_slot'], 16)
                for sample in item['samples']:
                    frame = source.number(int(sample['frame_bits'], 16))
                    emit(frame, rows, duration, wrap, 0x027F, flags, slot, 'source_'+sample['kind'], item=item)
    finally:
        if binary is not None:
            binary.close()
            table.close()
    assert source_cases == census['statistics']['source_cases'] == 50367
    report = dict(schema='full-channel-runtime-curve-inputs-v1', native_executed=False, prepared=args.prepare,
        cases=ordinal, source_cases=source_cases, synthetic_cases=ordinal-source_cases,
        contexts=dict(contexts), flags=dict(flags_counts), slots=dict(slots), counts=dict(counts),
        families=dict(families), expected_policy_counts=dict(policies),
        input_bytes=ordinal*host.INPUT_BYTES, expected_output_bytes=ordinal*host.OUTPUT_BYTES,
        protocol={'input_bytes': host.INPUT_BYTES, 'output_bytes': host.OUTPUT_BYTES},
        host_manifest_sha256=source.sha_file(args.host_dir/'manifest.json'),
        host_source_sha256=manifest['host_source_sha256'], source_sha256=source.SOURCE_SHA256,
        census_report_sha256=source.sha_file(args.census_dir/'report.json'),
        source_components_sha256=census['components_sha256'],
        tool_sha256=source.sha_file(Path(__file__)), census_tool_sha256=source.sha_file(Path(source.__file__)),
        source_rows_files=census['source_rows_files'], source_rows_report_sha256=census['source_rows_report_sha256'],
        full_source_row_counts=True, native_source_upper_selector=True,
        source_case_context='CW027F/MXCSR1F80; exact source frames/rows/duration/flags/slot. Active clip/clock/blend/pose remain explicit.',
        metadata_columns=COLUMNS,
        named_source_span_counts={name: census['statistics'].get(name, 0) for name in [
            'duplicate_adjacent_spans', 'nonmonotonic_adjacent_spans', 'distinct_positive_spans',
            'no_representable_strict_midpoint', 'no_representable_frame_in_half_open_span',
            'unrepresentable_exact_endpoint_times']})
    if args.prepare:
        report['hashes'] = {name: source.sha_file(args.out/name) for name in ['input.bin', 'cases.tsv']}
    (args.out/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host-dir', type=Path, required=True)
    parser.add_argument('--census-dir', type=Path, required=True)
    parser.add_argument('--source-rows', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--prepare', action='store_true')
    args = parser.parse_args()
    for path in [args.host_dir, args.census_dir, args.source_rows, args.out]:
        path.resolve().relative_to((ROOT/'target').resolve())
    generate(args)


if __name__ == '__main__':
    main()

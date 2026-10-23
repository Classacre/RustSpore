"""Prepare streaming data2 consumer cases; --run executes the reviewed host."""
import argparse
from collections import Counter
import csv
import itertools
import json
from pathlib import Path
import struct

import cell_runtime_modifiers_native_probe as probe
from cell_runtime_data_native_run import (
    CONTEXTS, FINITE, NONFINITE, aggregate, digest, finite, mul_bits,
    pinned, reviewed_file_run, word,
)
from cell_schema_probe import SourceImage, SOURCE_SHA256

HOST = '62885b4c906f0890a8c554f7bb289356d45535c85a9f8f9ac820979fbef75db0'
ASM = '38a8d272d40c81bc91ff9e31228d5a3d2e45036c34564b639f6c42d7ac04f918'
DATA2 = 0x98C942B5
EXCLUDED = {0x12, 0x13, 0x14, 0x30}


def write_json(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')


def reviewed_host(args):
    m = json.loads((args.host_dir/'manifest.json').read_text())
    root = json.loads(args.root_review.read_text())
    peer = json.loads(args.peer_review.read_text())
    assert m['schema'] == 'bounded-original-cell-data2-consumer-host-v1'
    assert m['source_sha256'] == SOURCE_SHA256
    assert root['source_assembly_byte_exact'] and root['host_exe_sha256'] == HOST
    assert peer['verdict'] == 'PASS' and peer['linked_source_and_owned_audit_exact']
    assert peer['host_exe_sha256'] == HOST and peer['host_source_sha256'] == ASM
    assert digest(args.root_review) == peer['root_review_sha256']
    assert digest(args.host_dir/'host-x86.asm') == m['host_source_sha256'] == ASM
    assert digest(args.host_dir/'host-x86.exe') == HOST
    assert digest(Path(probe.__file__)) == m['preparation_tool_sha256']
    image = SourceImage(Path(m['source_path']))
    assert probe.audit_source(image) == m['code_allowlist']
    assert probe.host_source(image)[0] == (args.host_dir/'host-x86.asm').read_text()
    actual = probe.audit_linked(args.host_dir/'host-x86.exe', args.host_dir/'host-x86.map', image)
    assert actual == json.loads((args.host_dir/'linked-audit.json').read_text())
    return m


def request(raw, state=0, fc=1, handle=1, present=1, cw=0x027F):
    assert len(raw) == 164
    return struct.pack('<9I', state & 0xFFFFFFFF, fc, handle, present, cw, 0, 0, 0, 0)+raw


def row(a, b, count=1):
    raw = bytearray(aggregate(count))
    if count > 0:
        struct.pack_into('<I', raw, 4, DATA2)
        struct.pack_into('<I', raw, 12, a)
        struct.pack_into('<I', raw, 20, b)
    return bytes(raw)


def prepare(args):
    m = reviewed_host(args)
    report = json.loads((args.source_rows/'report.json').read_text())
    for name, expected in report['files'].items():
        assert digest(args.source_rows/name) == expected
    args.out.mkdir(parents=True, exist_ok=False)
    families, controls, count, source_rows = Counter(), Counter(), 0, 0
    with (args.out/'input.bin').open('xb') as binary, (args.out/'cases.tsv').open('x', newline='\n') as table:
        table.write('case\tfamily\tcw\tcomponent\tsource_row\n')
        def emit(raw, family, component='-', source_row='-'):
            nonlocal count
            assert len(raw) == 200
            binary.write(raw)
            cw = word(raw, 16)
            table.write(f'{count}\t{family}\t{cw:04X}\t{component}\t{source_row}\n')
            count += 1
            families[family] += 1
            controls[f'{cw:04X}'] += 1
        emit(request(aggregate(0), fc=0, handle=0, present=0), 'smoke_missing_link')
        shapes = [aggregate(0), row(0, 0x3F800000), row(0x7F800001, 0x7FC12345), aggregate(-1)]
        states = [0x80000000, 0xFFFFFFFF, 0, 0x11, 0x12, 0x13, 0x14, 0x15, 0x2F, 0x30, 0x31, 0x7FFFFFFF]
        for cw in CONTEXTS:
            for state, fc, handle, present, raw in itertools.product(
                    states, [0, 1, 2, 0xFFFFFFFF], [0, 1, 0xDEADBEEF], [0, 1], shapes):
                emit(request(raw, state, fc, handle, present, cw), 'literal_state_link_object_matrix')
            for a, b in itertools.product(FINITE+NONFINITE, repeat=2):
                family = 'finite_numeric_edges' if finite(a) and finite(b) else 'reached_nonfinite_policy'
                emit(request(row(a, b), cw=cw), family)
            for rows in range(9):
                raw = bytearray(aggregate(rows))
                for position in [-1, 0, rows-1]:
                    if rows and position >= 0:
                        struct.pack_into('<I', raw, 4+20*position, DATA2)
                        struct.pack_into('<I', raw, 12+20*position, 0x80000000 if position == 0 else 0x40000000)
                        struct.pack_into('<I', raw, 20+20*position, 0x3F800000)
                    emit(request(bytes(raw), cw=cw), 'ordered_count_and_duplicate_first_match')
            for rows in [-(1 << 31), -100]:
                emit(request(aggregate(rows), cw=cw), 'nonpositive_count_no_reads')
        with (args.source_rows/'components.tsv').open(newline='') as table_in, \
                (args.source_rows/'source-INFO20-custom40-rows.bin').open('rb') as source:
            for component, item in enumerate(csv.DictReader(table_in, delimiter='\t')):
                if int(item['tag'], 16) != DATA2:
                    continue
                source.seek(int(item['rows_byte_offset']))
                for index in range(int(item['rows'])):
                    original = source.read(60)
                    assert len(original) == 60
                    a, b = struct.unpack_from('<2I', original, 20)
                    weighted = mul_bits(a, mul_bits(b, 0x3F800000))
                    emit(request(row(weighted, b)), 'source_data2_row_explicit_blend1', component, index)
                    source_rows += 1
    assert source_rows == 16584
    write_json(args.out/'prepare-report.json', dict(schema='data2-consumer-input-cohort-v1',
        cases=count, source_rows=source_rows, source_context='CW027F/MXCSR1F80,blend1,normal state,explicit resolved aggregate; not full curve/actor binding',
        families=dict(families), contexts=dict(controls), input_bytes=count*200, expected_output_bytes=count*28,
        host_exe_sha256=HOST, host_source_sha256=ASM, runner_sha256=digest(Path(__file__)),
        hashes={name: digest(args.out/name) for name in ['input.bin', 'cases.tsv']},
        source_rows_files=report['files'], source_rows_report_sha256=digest(args.source_rows/'report.json'),
        source_executable_sha256=SOURCE_SHA256, native_executed=False,
        finite_api_policy='Reached nonfinite reader inputs/results are checked errors, with every native result retained. Early exits do not inspect unused numeric payloads.'))
    print(json.dumps(dict(prepared=True, cases=count, source_rows=source_rows, families=dict(families)), indent=2))


def inspect(inp, out):
    count, remainder = divmod(inp.stat().st_size, 200)
    assert remainder == 0 and out.stat().st_size == count*28
    controls, policies, calls = Counter(), Counter(), Counter()
    with inp.open('rb') as requests, out.open('rb') as replies:
        for _ in range(count):
            req, reply = requests.read(200), replies.read(28)
            cw, sw, mxcsr = struct.unpack_from('<HHI', reply, 20)
            assert cw == word(req, 16) and sw & 0x3800 == 0 and mxcsr & ~0x3F == 0x1F80
            reached = word(req, 0) not in EXCLUDED and word(req, 4) != 0 and word(req, 8) != 0
            assert word(reply, 16) == int(reached) and reply[10:12] == bytes(2)
            calls[str(int(reached))] += 1
            controls[f'{cw:04X}'] += 1
            signed_count = struct.unpack_from('<i', req, 36)[0]
            match = None
            if reached and word(req, 12):
                match = next((i for i in range(max(0, signed_count)) if word(req, 40+20*i) == DATA2), None)
            if match is None:
                assert reply[:10] == bytes.fromhex('0000000000000080ff3f') and word(reply, 12) == 0x3F800000
            elif not finite(word(req, 48+20*match)) or not finite(word(req, 56+20*match)):
                policies['reached_nonfinite_input'] += 1
            elif not finite(word(reply, 12)):
                policies['nonfinite_final_FST32'] += 1
    return dict(cases=count, contexts=dict(controls), manager_call_counts=dict(calls), policy_counts=dict(policies))


def execute(args):
    m = reviewed_host(args)
    report = json.loads((args.out/'prepare-report.json').read_text())
    assert report['runner_sha256'] == digest(Path(__file__))
    for name, expected in report['hashes'].items():
        assert digest(args.out/name) == expected
    folder = args.out/'native-v1'
    folder.mkdir(exist_ok=False)
    tools = Path(__file__).resolve().parent
    source = tools.parent/'crates/spore-core/src'
    paths = [Path(m['source_path']), args.root_review, args.peer_review,
        *[p for p in args.host_dir.iterdir() if p.is_file()],
        args.out/'prepare-report.json', *[args.out/name for name in report['hashes']],
        args.source_rows/'report.json', *[args.source_rows/name for name in report['source_rows_files']],
        *[tools/name for name in ['cell_runtime_modifiers_native_run.py', 'cell_runtime_modifiers_native_probe.py',
            'cell_runtime_data_native_run.py', 'cell_runtime_data_native_probe.py', 'cell_schema_probe.py',
            'cell_native_math_probe.py', 'native_crt_acos_probe.py']],
        *[source/name for name in ['cell_runtime_modifiers.rs', 'cell_runtime_data.rs', 'native_x87.rs', 'native_sse.rs']]]
    with pinned(paths, folder):
        host = (args.host_dir/'host-x86.exe').resolve()
        protocol = folder/'protocol'
        protocol.mkdir()
        with (args.out/'input.bin').open('rb') as stream:
            smoke = stream.read(200)
        ref = protocol/'reference-input.bin'
        ref.write_bytes(smoke)
        status, refout = reviewed_file_run(host, ref, protocol, 'reference')
        assert status['exit'] == 0 and status['stderr_bytes'] == 0
        inspect(ref, refout)
        one = refout.read_bytes()
        checks = [('empty', b'', 0, b'')]
        for size in [1, 35, 199]:
            checks += [(f'partial{size}', smoke[:size], 7, b''),
                       (f'complete_then_partial{size}', smoke+smoke[:size], 7, one)]
        for at, value, label in [(12, 2, 'object2'), (16, 0x17F, 'reservedPC'),
                (16, 0x27E, 'unmaskedCW'), (16, 0x1027F, 'highCW'),
                (36, 9, 'count9'), (36, 0x7FFFFFFF, 'count_max')]+[
                (at, 1, f'reserved{at}') for at in range(20, 36, 4)]:
            raw = bytearray(smoke)
            struct.pack_into('<I', raw, at, value)
            checks.append((label, bytes(raw), 7, b''))
        results = []
        for label, raw, expected_exit, expected in checks:
            inp = protocol/f'{label}-input.bin'
            inp.write_bytes(raw)
            status, output = reviewed_file_run(host, inp, protocol, label)
            assert status['exit'] == expected_exit and status['stderr_bytes'] == 0
            assert output.read_bytes() == expected
            results.append(dict(case=label, exact=True, exit=status['exit']))
        write_json(protocol/'report.json', dict(checks=len(results), cases=results))
        status, output = reviewed_file_run(host, args.out/'input.bin', folder, 'corpus', timeout=120)
        assert status['exit'] == 0 and status['stderr_bytes'] == 0
        result = inspect(args.out/'input.bin', output)
        result.update(schema='native-data2-consumer-execution-v1', native_executed=True,
            native_output_path=str(output.resolve()), native_output_sha256=digest(output),
            protocol_checks=len(results), host_exe_sha256=HOST, host_source_sha256=ASM,
            input_sha256=report['hashes']['input.bin'], source_sha256=SOURCE_SHA256,
            scope='Three complete source functions and a checked manager provider; explicit aggregate/state inputs, no actor construction or live animation/steering integration.')
        write_json(folder/'report.json', result)
        print(json.dumps(result, indent=2))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ['host-dir', 'root-review', 'peer-review', 'source-rows', 'out']:
        p.add_argument('--'+name, type=Path, required=True)
    p.add_argument('--run', action='store_true')
    args = p.parse_args()
    target = Path(__file__).resolve().parent.parent/'target'
    for path in [args.host_dir, args.root_review, args.peer_review, args.source_rows, args.out]:
        path.resolve().relative_to(target.resolve())
    execute(args) if args.run else prepare(args)


if __name__ == '__main__':
    main()

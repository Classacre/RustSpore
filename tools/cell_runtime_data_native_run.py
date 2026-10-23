"""Stream preparation; execute only a separately reviewed pinned kernel host.

The default writes requests and provenance only. --run is reserved for the
parent after source/linked/peer review. No full request/output stream is loaded
into a Python bytes object; every process attempt and failure remains saved.
"""
import argparse
from collections import Counter
from contextlib import contextmanager
import csv
import hashlib
import itertools
import json
from pathlib import Path
import random
import struct
import subprocess
import time

import cell_runtime_data_native_probe as probe
from cell_schema_probe import SourceImage, SOURCE_SHA256

HOST = '0fe85f1c282f5f977613c35c8c41e9242c478cc9949d83e1d1bfd3ff8d020294'
ASM = '40f7acb855b0a9a0de70c9d77dcd8188767089127836d7cf0b60b4663a293c82'
CONTEXTS = [0x7F | pc | rc for pc, rc in itertools.product([0, 0x200, 0x300], [0, 0x400, 0x800, 0xC00])]
FINITE = [0, 0x80000000, 1, 0x80000001, 0x007FFFFF, 0x807FFFFF, 0x00800000,
          0x80800000, 0x3F000000, 0xBF000000, 0x3F800000, 0xBF800000,
          0x3F800001, 0x3F7FFFFF, 0x7F7FFFFF, 0xFF7FFFFF]
NONFINITE = [0x7F800000, 0xFF800000, 0x7FC00000, 0xFFC00001,
             0x7FFFFFFF, 0xFFFFFFFF, 0x7F800001, 0xFF800001]


def digest(path):
    result = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for chunk in iter(lambda: stream.read(1 << 20), b''):
            result.update(chunk)
    return result.hexdigest()


def word(data, at):
    return struct.unpack_from('<I', data, at)[0]


def finite(bits):
    return bits & 0x7F800000 != 0x7F800000


def aggregate(count, seed=0):
    raw = bytearray(b'\xA5' * 164)
    struct.pack_into('<I', raw, 0, count & 0xFFFFFFFF)
    for index in range(max(count, 0)):
        assert index < 8
        struct.pack_into('<5I', raw, 4+20*index, 0x10203040+index,
            (seed*0x9E3779B1+index) & 0xFFFFFFFF, FINITE[(seed+index) % 16],
            FINITE[(seed*3+index+1) % 16], FINITE[(seed*5+index+2) % 16])
    return bytes(raw)


def request(mode, raw, wanted=0xDEADBEEF, cw=0x027F, kind=1, first=0x3F000000,
            second=0xBF000000, weight=0x3F800000):
    assert len(raw) == 164
    header = struct.pack('<IIHH6I', mode, wanted, cw, 0, kind, first, second, weight, 0, 0)
    result = header + raw
    assert len(result) == 200
    return result


def mul_bits(a, b):
    # A finite binary32 product is exact in binary64 (<=48 significand bits,
    # exponent >=-298), then this pack supplies nearest-even binary32 commit.
    assert finite(a) and finite(b)
    av, bv = (struct.unpack('<f', struct.pack('<I', x))[0] for x in (a, b))
    value = av * bv
    try:
        return struct.unpack('<I', struct.pack('<f', value))[0]
    except OverflowError:
        return 0x7F800000 | ((a ^ b) & 0x80000000)


def reviewed_host(args):
    manifest = json.loads((args.host_dir/'manifest.json').read_text())
    linked = json.loads((args.host_dir/'linked-audit.json').read_text())
    root_review = json.loads(args.root_review.read_text())
    peer = json.loads(args.peer_review.read_text())
    assert manifest['schema'] == 'bounded-original-cell-runtime-data-host-v1'
    assert manifest['source_sha256'] == SOURCE_SHA256
    assert manifest['host_source_sha256'] == ASM == root_review['source_assembly_sha256'] == peer['host_source_sha256']
    assert linked['host_exe_sha256'] == HOST == root_review['host_exe_sha256']
    assert root_review['source_assembly_byte_exact'] and peer['no_blocking_static_issue_found']
    assert peer['source_spans_exact'] and peer['write_masks_exact'] and peer['guarded_pointer_chain_bounds']
    original_manifest = Path(root_review['original_prepared_dir'])/'manifest.json'
    assert digest(original_manifest) == peer['manifest_sha256']
    assert digest(Path(probe.__file__)) == manifest['preparation_tool_sha256'] == peer['preparation_tool_sha256']
    assert digest(args.host_dir/'host-x86.exe') == HOST
    assert digest(args.host_dir/'host-x86.asm') == ASM
    image = SourceImage(args.source_exe)
    assert probe.audit_source(image) == manifest['code_allowlist']
    assert probe.host_source(image)[0] == (args.host_dir/'host-x86.asm').read_text()
    actual = probe.audit_linked(args.host_dir/'host-x86.exe', args.host_dir/'host-x86.map', image)
    assert all(linked.get(key) == value for key, value in actual.items())
    return manifest, linked, original_manifest


def prepare(args):
    _, linked, _ = reviewed_host(args)
    source_report = json.loads((args.source_rows/'report.json').read_text())
    assert source_report['statistics']['components'] == 2086
    assert source_report['statistics']['rows'] == 44127
    for name, expected in source_report['files'].items():
        assert digest(args.source_rows/name) == expected
    args.out.mkdir(parents=True, exist_ok=False)
    counts, modes, contexts = Counter(), Counter(), Counter()
    ordinal = 0
    input_path, table_path = args.out/'input.bin', args.out/'cases.tsv'
    with input_path.open('xb') as binary, table_path.open('x', newline='\n') as table:
        table.write('case\tfamily\tmode\tcw\tsource_component\tsource_row\n')
        def emit(raw, family, component='-', row='-'):
            nonlocal ordinal
            mode, cw = word(raw, 0), struct.unpack_from('<H', raw, 8)[0]
            binary.write(raw)
            table.write(f'{ordinal}\t{family}\t{mode}\t{cw:04X}\t{component}\t{row}\n')
            counts[family] += 1
            modes[str(mode)] += 1
            contexts[f'{cw:04X}'] += 1
            ordinal += 1
        # The first request is a simple independent protocol-smoke reference.
        emit(request(0, aggregate(0)), 'smoke_empty_reader')
        for cw in CONTEXTS:
            for count in range(9):
                raw = aggregate(count, count)
                for target in range(-1, count):
                    wanted = 0xDEADBEEF if target < 0 else word(raw, 4+20*target)
                    for mode in range(3):
                        emit(request(mode, raw, wanted, cw, kind=0xAABBCCDD), 'synthetic_order_capacity')
                if count >= 2:
                    duplicate = bytearray(raw)
                    struct.pack_into('<I', duplicate, 4+20*(count-1), word(raw, 4))
                    for mode in range(3):
                        emit(request(mode, bytes(duplicate), word(raw, 4), cw), 'synthetic_first_duplicate_wins')
            for count in [-0x80000000, -100, -1]:
                emit(request(0, aggregate(count), cw=cw), 'synthetic_nonpositive_count_no_reads')
            for a, b in itertools.product(FINITE, repeat=2):
                raw = bytearray(aggregate(1))
                struct.pack_into('<II', raw, 12, a, 0x7F800001)  # unread second field remains raw.
                struct.pack_into('<I', raw, 20, b)
                emit(request(0, bytes(raw), word(raw, 4), cw), 'reader_finite_operand_edges')
            for a, b in itertools.product(FINITE+NONFINITE, repeat=2):
                raw = bytearray(aggregate(1))
                struct.pack_into('<I', raw, 12, a)
                struct.pack_into('<I', raw, 20, b)
                if not finite(a) or not finite(b):
                    emit(request(0, bytes(raw), word(raw, 4), cw), 'reader_reached_nonfinite_policy')
                for mode in [1, 2]:
                    emit(request(mode, aggregate(0), cw=cw, first=a, second=a ^ 0x80000000,
                        weight=b), 'writer_raw_append_all_patterns')
                    emit(request(mode, bytes(raw), word(raw, 4), cw, kind=0xFEDCBA98,
                        first=b, second=a ^ 0x400001, weight=b ^ 0x80000000), 'writer_raw_update_all_patterns')
            rng = random.Random(0xE833B0)
            for index in range(128):
                raw = bytearray(aggregate(1, index))
                a, b = rng.getrandbits(32), rng.getrandbits(32)
                while not finite(a):
                    a = rng.getrandbits(32)
                while not finite(b):
                    b = rng.getrandbits(32)
                struct.pack_into('<I', raw, 12, a)
                struct.pack_into('<I', raw, 20, b)
                for mode in range(3):
                    emit(request(mode, bytes(raw), word(raw, 4), cw, first=a, second=b,
                        weight=FINITE[index % 16]), 'synthetic_random_finite_snapshot')
        # Every authored row is injected explicitly, with blend1. This is not
        # full-clip key selection/interpolation or actual gameplay blending.
        with (args.source_rows/'components.tsv').open(newline='') as component_table, \
                (args.source_rows/'source-INFO20-custom40-rows.bin').open('rb') as rows:
            source_count = 0
            for component, item in enumerate(csv.DictReader(component_table, delimiter='\t')):
                rows.seek(int(item['rows_byte_offset']))
                for index in range(int(item['rows'])):
                    raw_row = rows.read(60)
                    assert len(raw_row) == 60
                    value, weight = struct.unpack_from('<2I', raw_row, 20)
                    weighted = mul_bits(value, mul_bits(weight, 0x3F800000))
                    kind = (int(item['prepared_flags'], 16) >> 7) & 1
                    wanted = int(item['tag'], 16)
                    empty = aggregate(0)
                    emit(request(1, empty, wanted, 0x027F, kind=kind, first=weighted,
                        weight=weight), 'source_authored_row_explicit_blend1_append', component, index)
                    state = bytearray(empty)
                    struct.pack_into('<I', state, 0, 1)
                    struct.pack_into('<3I', state, 4, wanted, kind, weighted)
                    struct.pack_into('<I', state, 20, weight)
                    emit(request(0, bytes(state), wanted, 0x027F),
                        'source_authored_row_explicit_blend1_lookup', component, index)
                    source_count += 1
            assert source_count == 44127
    report = dict(schema='native-runtime-data-input-cohort-v1', host_executed=False,
        cases=ordinal, source_rows=44127, source_cases=88254, source_context='CW027F/MXCSR1F80; blend1 explicit; source row values, not a full-clip sample',
        synthetic_cases=ordinal-88254, contexts=dict(contexts), modes=dict(modes), families=dict(counts),
        source_executable_sha256=SOURCE_SHA256, host_exe_sha256=linked['host_exe_sha256'], host_source_sha256=ASM,
        source_rows_report_sha256=digest(args.source_rows/'report.json'),
        source_rows_files=source_report['files'], runner_sha256=digest(Path(__file__)),
        raw_policy='All native outputs retained. Raw writerNaN/infinity inputs/results are modeled bits; reader reached nonfinite inputs or nonfinite finalFST32 are positively checked typed API errors, with separate exact counts.',
        hashes={'input.bin': digest(input_path), 'cases.tsv': digest(table_path)},
        bytes=input_path.stat().st_size, expected_native_output_bytes=ordinal*180,
        memory_scope='Streaming flat requests/source rows and per-case TSV; no all-cases Python object list or full stdin/output copy.')
    (args.out/'prepare-report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


def saved_file_run(host, input_path, folder, label, timeout=120):
    output_path, error_path = folder/f'{label}-output.bin', folder/f'{label}-stderr.bin'
    process, failure, cleanup, timed_out = None, None, None, False
    started = time.perf_counter()
    try:
        with input_path.open('rb') as inp, output_path.open('xb') as out, error_path.open('xb') as err:
            try:
                process = subprocess.Popen([str(host)], stdin=inp, stdout=out, stderr=err,
                    creationflags=subprocess.CREATE_NO_WINDOW)
                process.wait(timeout=timeout)
            except BaseException as exc:
                failure = exc
                timed_out = isinstance(exc, subprocess.TimeoutExpired)
                if process is not None:
                    try:
                        if process.poll() is None:
                            process.kill()
                        process.wait(timeout=5)
                    except BaseException as cleanup_exception:
                        cleanup = repr(cleanup_exception)
    except BaseException as exc:
        failure = failure or exc
    finally:
        status = dict(exit=None if process is None else process.returncode, timeout=timed_out,
            exception=None if failure is None else repr(failure), cleanup_exception=cleanup,
            input_path=str(input_path.resolve()),
            input_bytes=input_path.stat().st_size if input_path.exists() else None,
            output_bytes=output_path.stat().st_size if output_path.exists() else None,
            stderr_bytes=error_path.stat().st_size if error_path.exists() else None,
            elapsed_seconds=time.perf_counter()-started, hidden=True, file_backed_streams=True)
        (folder/f'{label}-status.json').write_text(json.dumps(status, indent=2)+'\n')
    if failure is not None:
        raise failure
    return status, output_path


def reviewed_file_run(host, input_path, folder, label, timeout=120):
    attempts = []
    try:
        for attempt in range(1, 9):
            name = f'{label}-attempt{attempt:02}'
            try:
                status, output = saved_file_run(host, input_path, folder, name, timeout)
            except BaseException as exc:
                attempts.append(dict(attempt=attempt, label=name, exception=repr(exc), retried=False))
                raise
            retry = status['exit'] == 10 and status['output_bytes'] == 0 and status['stderr_bytes'] == 0
            attempts.append(dict(attempt=attempt, label=name, exit=status['exit'], allocation_only=retry,
                                 retried=retry and attempt < 8))
            if not retry:
                return status, output
        return status, output
    finally:
        (folder/f'{label}-attempts.json').write_text(json.dumps(dict(maximum_attempts=8,
            retry_policy='Only empty exit10 from fixed allocation before first ReadRecord; every attempt retained.',
            attempts=attempts), indent=2)+'\n')


@contextmanager
def pinned(paths, folder):
    pins = {str(path.resolve()): digest(path) for path in paths}
    (folder/'execution-pins-v1.json').write_text(json.dumps(pins, indent=2)+'\n')
    failure = None
    try:
        yield
    except BaseException as exc:
        failure = repr(exc)
        raise
    finally:
        changed, errors = [], []
        for name, expected in pins.items():
            try:
                if digest(name) != expected:
                    changed.append(name)
            except BaseException as exc:
                changed.append(name)
                errors.append(dict(path=name, error=repr(exc)))
        (folder/'immutability-v1.json').write_text(json.dumps(dict(unchanged=not changed,
            pinned_files=len(pins), changed_files=changed, read_errors=errors,
            operation_exception=failure), indent=2)+'\n')
        if changed:
            raise AssertionError('Pinned artifacts changed: '+repr(changed))


def inspect(input_path, output_path):
    count, remainder = divmod(input_path.stat().st_size, 200)
    assert not remainder and output_path.stat().st_size == count*180
    contexts, sticky, modes, policy, output_words = Counter(), Counter(), Counter(), Counter(), Counter()
    with input_path.open('rb') as inp, output_path.open('rb') as out:
        for ordinal in range(count):
            req, reply = inp.read(200), out.read(180)
            mode, wanted = struct.unpack_from('<2I', req)
            requested_cw = struct.unpack_from('<H', req, 8)[0]
            cw, sw, mxcsr = struct.unpack_from('<HHI', reply, 172)
            assert cw == requested_cw and sw & 0x3800 == 0 and mxcsr & ~0x3F == 0x1F80
            contexts[f'{cw:04X}'] += 1
            sticky[f'{mxcsr&0x3F:02X}'] += 1
            modes[str(mode)] += 1
            rows = struct.unpack_from('<i', req, 36)[0]
            matched = next((i for i in range(max(rows, 0)) if word(req, 40+20*i) == wanted), None)
            accepted = matched is not None if mode == 0 else matched is not None or rows < 8
            assert word(reply, 0) == int(accepted)
            if mode == 0:
                assert reply[8:172] == req[36:200]
                if matched is None:
                    assert word(reply, 4) == 0
                else:
                    at = 40+20*matched
                    if not finite(word(req, at+8)) or not finite(word(req, at+16)):
                        policy['reached_nonfinite_input'] += 1
                    elif not finite(word(reply, 4)):
                        policy['nonfinite_final_FST32'] += 1
            else:
                assert word(reply, 4) == 0xA5A5A5A5
                assert word(reply, 8) == rows+int(matched is None and accepted)
                for index in range(word(reply, 8)):
                    for field, at in [('first', 8), ('second', 12), ('weight', 16)]:
                        if not finite(word(reply, 12+20*index+at)):
                            output_words[f'nonfinite_active_{field}_words'] += 1
    return dict(cases=count, context_counts=dict(contexts), mode_counts=dict(modes),
                mxcsr_sticky_counts=dict(sticky), reader_policy_counts=dict(policy), raw_output_counts=dict(output_words))


def execute(args):
    _, _, original_manifest = reviewed_host(args)
    report = json.loads((args.out/'prepare-report.json').read_text())
    assert report['runner_sha256'] == digest(Path(__file__))
    assert report['host_exe_sha256'] == HOST and report['host_source_sha256'] == ASM
    for name, expected in report['hashes'].items():
        assert digest(args.out/name) == expected
    native = args.out/'native-v1'
    native.mkdir(exist_ok=False)
    tools = Path(__file__).resolve().parent
    paths = [args.source_exe, args.root_review, args.peer_review, original_manifest,
        args.out/'prepare-report.json', *[args.out/name for name in report['hashes']],
        *[p for p in args.host_dir.iterdir() if p.is_file()],
        *[tools/name for name in ['cell_runtime_data_native_run.py', 'cell_runtime_data_native_probe.py',
                                  'cell_schema_probe.py', 'cell_native_math_probe.py', 'native_crt_acos_probe.py']],
        args.source_rows/'report.json', *[args.source_rows/name for name in report['source_rows_files']]]
    with pinned(paths, native):
        host = (args.host_dir/'host-x86.exe').resolve()
        protocol = native/'protocol'
        protocol.mkdir()
        with (args.out/'input.bin').open('rb') as inp:
            smoke = inp.read(200)
        reference = protocol/'reference-input.bin'
        reference.write_bytes(smoke)
        status, reference_output = reviewed_file_run(host, reference, protocol, 'reference')
        assert status['exit'] == 0 and status['stderr_bytes'] == 0
        inspect(reference, reference_output)
        expected_reply = reference_output.read_bytes()
        negatives = [('empty', b'', 0, b'')]
        negatives += [(f'partial{size}', smoke[:size], 7, b'') for size in [1, 35, 199]]
        negatives.append(('complete_then_partial', smoke+b'x', 7, expected_reply))
        for offset, value, label in [(0, 3, 'bad_mode'), (28, 1, 'reserved28'),
                                     (32, 1, 'reserved32'), (36, 9, 'reader_count9'),
                                     (36, 0x7FFFFFFF, 'reader_count_max')]:
            raw = bytearray(smoke)
            struct.pack_into('<I', raw, offset, value)
            negatives.append((label, bytes(raw), 7, b''))
        for cw, label in [(0x017F, 'reserved_PC'), (0x027E, 'unmasked_exception'), (0xF27F, 'high_CW')]:
            raw = bytearray(smoke)
            struct.pack_into('<H', raw, 8, cw)
            negatives.append((label, bytes(raw), 7, b''))
        raw = bytearray(smoke)
        struct.pack_into('<H', raw, 10, 1)
        negatives.append(('reserved10', bytes(raw), 7, b''))
        for mode, rows in itertools.product([1, 2], [9, 0xFFFFFFFF]):
            raw = bytearray(smoke)
            struct.pack_into('<I', raw, 0, mode)
            struct.pack_into('<I', raw, 36, rows)
            negatives.append((f'writer{mode}_count{rows:08X}', bytes(raw), 7, b''))
        protocol_results = []
        for label, raw, expected_exit, expected_output in negatives:
            inp = protocol/f'{label}-input.bin'
            inp.write_bytes(raw)
            status, output = reviewed_file_run(host, inp, protocol, label)
            assert status['exit'] == expected_exit and status['stderr_bytes'] == 0
            assert output.read_bytes() == expected_output
            protocol_results.append(dict(label=label, exit=status['exit'], output_bytes=status['output_bytes']))
        (protocol/'report.json').write_text(json.dumps(dict(checks=len(negatives), cases=protocol_results), indent=2)+'\n')
        status, output = reviewed_file_run(host, args.out/'input.bin', native, 'corpus', timeout=180)
        assert status['exit'] == 0 and status['stderr_bytes'] == 0
        result = inspect(args.out/'input.bin', output)
        result.update(schema='native-runtime-data-execution-v1', native_output_path=str(output.resolve()),
            native_output_sha256=digest(output), input_sha256=digest(args.out/'input.bin'),
            host_exe_sha256=HOST, host_source_sha256=ASM, source_sha256=SOURCE_SHA256,
            protocol_checks=len(negatives), native_executed=True,
            scope='Only three reviewed source functions in owned hidden host; no game/initializer/GUI/network or customcurve execution.')
        (native/'report.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps(result, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-exe', type=Path, default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    parser.add_argument('--host-dir', type=Path, required=True)
    parser.add_argument('--root-review', type=Path, required=True)
    parser.add_argument('--peer-review', type=Path, required=True)
    parser.add_argument('--source-rows', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--run', action='store_true')
    args = parser.parse_args()
    target = (Path(__file__).resolve().parent.parent/'target').resolve()
    for path in [args.host_dir, args.root_review, args.peer_review, args.source_rows, args.out]:
        path.resolve().relative_to(target)
    if args.run:
        execute(args)
    else:
        prepare(args)


if __name__ == '__main__':
    main()

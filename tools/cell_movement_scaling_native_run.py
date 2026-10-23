"""Prepare original scaling goldens; execute only a separately reviewed host.

All original-table actor/progress combinations and all12 explicit x87 contexts
are retained. Synthetic scale-table overrides and nonfinite table words have
separate named lanes. Raw quotient bits never masquerade as a finite input.
"""
import argparse
from collections import Counter
import itertools
import json
from pathlib import Path
import random
import struct
import subprocess
import time

from cell_capability_scalar_run import digest, run_small
from cell_movement_scaling_native_probe import INPUT_BYTES, OUTPUT_BYTES, TABLE
from cell_schema_probe import SourceImage, SOURCE_SHA256

FINITE = [0, 0x80000000, 1, 0x80000001, 0x007fffff, 0x807fffff,
          0x00800000, 0x80800000, 0x3f000000, 0xbf000000, 0x3f800000,
          0xbf800000, 0x3f800001, 0x3f7fffff, 0x7f7fffff, 0xff7fffff]
NONFINITE = [0x7f800000, 0xff800000, 0x7fc00000, 0xffc00001,
             0x7fffffff, 0xffffffff, 0x7f800001, 0xff800001]
CONTEXTS = [0x7f | precision | rounding for precision, rounding in itertools.product(
    [0, 0x200, 0x300], [0, 0x400, 0x800, 0xc00])]


def request(table, actor, progress, control):
    assert len(table) == 80
    data = struct.pack('<iiH6x', actor, progress, control)+table+bytes(16)
    assert len(data) == INPUT_BYTES
    return data


def changed(table, numerator, denominator):
    out = bytearray(table)
    struct.pack_into('<2I', out, 0, numerator, denominator)
    return bytes(out)


def prepare(args, manifest):
    args.out.mkdir(parents=True, exist_ok=False)
    original = SourceImage(args.source_exe).read(TABLE, 80)
    assert digest(args.source_exe) == SOURCE_SHA256
    assert [f'{value:08X}' for value in struct.unpack('<20I', original)] == manifest['explicit_scale_table']['original_bits']
    (args.out/'original-scale-table.bin').write_bytes(original)
    counts, contexts, finite_count = Counter(), Counter(), 0
    rng = random.Random(0xe56330)
    random_pairs = []
    while len(random_pairs) < 512:
        a, b = rng.getrandbits(32), rng.getrandbits(32)
        if a & 0x7f800000 != 0x7f800000 and b & 0x7f800000 != 0x7f800000:
            random_pairs.append((a, b))
    with (args.out/'input.bin').open('xb') as stream, (args.out/'cases.tsv').open('x', newline='\n') as table:
        table.write('case\tcohort\tcw\n')
        def emit(raw, cohort):
            nonlocal finite_count
            cw = struct.unpack_from('<H', raw, 8)[0]
            stream.write(raw)
            table.write(f'{finite_count}\t{cohort}\t{cw:04X}\n')
            counts[cohort] += 1
            contexts[f'{cw:04X}'] += 1
            finite_count += 1
        for control in CONTEXTS:
            for actor, progress in itertools.product(range(-1, 20), range(1001)):
                emit(request(original, actor, progress, control), 'original_table_all_actor_progress')
            for progress in [-0x80000000, -1, 1001, 10000, 0x7fffffff]:
                emit(request(original, -1, progress, control), 'original_table_early_unused_progress')
            for numerator, denominator in itertools.product(FINITE, repeat=2):
                emit(request(changed(original, numerator, denominator), 0, 0, control), 'synthetic_finite_operand_edges')
            for numerator, denominator in random_pairs:
                emit(request(changed(original, numerator, denominator), 0, 0, control), 'synthetic_random_finite_operands')
            for boundary in [0x3f000000, 0x3f800000, 0x40000000, 0x40800000]:
                for delta in range(-4, 5):
                    emit(request(changed(original, boundary+delta, 0x3f800000), 0, 0, control), 'synthetic_threshold_neighbors')
    assert counts['original_table_all_actor_progress'] == 252252
    assert finite_count == 261960, finite_count
    nonfinite_count, nonfinite_counts = 0, Counter()
    with (args.out/'nonfinite-input.bin').open('xb') as stream, (args.out/'nonfinite-cases.tsv').open('x', newline='\n') as table:
        table.write('case\tcohort\tcw\n')
        def emit_nonfinite(raw, cohort):
            nonlocal nonfinite_count
            cw = struct.unpack_from('<H', raw, 8)[0]
            stream.write(raw)
            table.write(f'{nonfinite_count}\t{cohort}\t{cw:04X}\n')
            nonfinite_counts[cohort] += 1
            nonfinite_count += 1
        for control in CONTEXTS:
            for numerator, denominator in itertools.product(FINITE+NONFINITE, repeat=2):
                if numerator not in NONFINITE and denominator not in NONFINITE:
                    continue
                emit_nonfinite(request(changed(original, numerator, denominator), 0, 0, control), 'nonfinite_reached_table_word')
            all_nan = struct.pack('<20I', *([0x7f800001]*20))
            emit_nonfinite(request(all_nan, -1, -0x80000000, control), 'nonfinite_all_unused_early_exit')
            unused_nan = bytearray(all_nan)
            struct.pack_into('<2I', unused_nan, 0, 0x3f400000, 0x3f800000)
            emit_nonfinite(request(bytes(unused_nan), 0, 0, control), 'nonfinite_unused_other_table_words')
    assert nonfinite_count == 3864, nonfinite_count
    report = dict(schema='original-cell-movement-scaling-cohort-v1', host_executed=False,
        cases=finite_count, original_table_cases=252312, synthetic_finite_cases=9648,
        original_actor_indices='-1,0..19', original_progress_range='every integer0..1000',
        context_counts=dict(contexts), cohorts=dict(counts), explicit_contexts=12,
        nonfinite_cases=nonfinite_count, nonfinite_cohorts=dict(nonfinite_counts),
        finite_agreement_exclusions=0, expected_retained_outputs='finite for every admitted table word pattern; nonfinite quotient branches are literal native outcomes, not excluded rows',
        host_exe_sha256=manifest['host_exe_sha256'], host_source_sha256=manifest['host_source_sha256'],
        source_sha256=SOURCE_SHA256, MXCSR_controls='1F80',
        hashes={name: digest(args.out/name) for name in ['input.bin', 'cases.tsv', 'original-scale-table.bin', 'nonfinite-input.bin', 'nonfinite-cases.tsv']},
        scope='Original fixed scale rows/constants plus explicitly supplied table words. Actor/progress values are explicit fixture inputs, not recovered live actor construction. Unsupported raw native ranges are rejected before execution; index-1 ignores progress/table literally.')
    (args.out/'prepare-report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


def inspect(input_path, output_path):
    count, remainder = divmod(input_path.stat().st_size, INPUT_BYTES)
    assert remainder == 0 and output_path.stat().st_size == count*OUTPUT_BYTES
    nonfinite_ratio = Counter()
    with input_path.open('rb') as inp, output_path.open('rb') as out:
        for ordinal in range(count):
            req, reply = inp.read(INPUT_BYTES), out.read(OUTPUT_BYTES)
            actor, progress, cw = struct.unpack_from('<iiH', req)
            actual_cw, sw, mxcsr = struct.unpack_from('<HHI', reply, 24)
            assert actual_cw == cw and sw & 0x3800 == 0 and mxcsr & ~0x3f == 0x1f80
            assert reply[10:12] == bytes(2)
            exponent = struct.unpack_from('<H', reply, 8)[0]
            assert exponent & 0x7fff != 0x7fff
            committed, selected, ratio = struct.unpack_from('<3I', reply, 12)
            assert committed & 0x7f800000 != 0x7f800000
            if actor == -1:
                assert selected == 0xffffffff and ratio == 0 and committed == 0x3f800000
            else:
                assert 1 <= selected <= 10
                if progress == 1000:
                    assert selected == 10
                else:
                    assert selected == (progress//100)+1
                if ratio & 0x7f800000 == 0x7f800000:
                    nonfinite_ratio['nan' if ratio & 0x7fffff else 'negative_infinity' if ratio >> 31 else 'positive_infinity'] += 1
    return dict(cases=count, nonfinite_committed_ratios=dict(nonfinite_ratio), nonfinite_retained_outputs=0)


def execute(args, manifest):
    prepared = json.loads((args.out/'prepare-report.json').read_text())
    assert prepared['host_exe_sha256'] == manifest['host_exe_sha256']
    assert all(digest(args.out/name) == sha for name, sha in prepared['hashes'].items())
    paths = [args.source_exe, *[args.host_dir/name for name in ['host-x86.asm', 'host-x86.exe', 'manifest.json']],
             args.out/'prepare-report.json', *[args.out/name for name in prepared['hashes']]]
    pins = {str(path.resolve()): digest(path) for path in paths}
    (args.out/'execution-pins-v1.json').write_text(json.dumps(pins, indent=2)+'\n')
    host = (args.host_dir/'host-x86.exe').resolve()
    checks = args.out/'protocol'
    checks.mkdir(exist_ok=False)
    try:
        with (args.out/'input.bin').open('rb') as inp:
            smoke = inp.read(INPUT_BYTES)
        first = run_small(host, smoke, checks, 'smoke')
        assert first.returncode == 0 and len(first.stdout) == OUTPUT_BYTES and not first.stderr
        malformed = [('empty', b'', 0, b''), ('partial1', smoke[:1], 7, b''),
                     ('partial111', smoke[:-1], 7, b''), ('complete_then_partial', smoke+b'x', 7, first.stdout)]
        for name, offset, value, fmt in [('actor_minus2', 0, -2, 'i'), ('actor20', 0, 20, 'i'),
                ('actor_min', 0, -0x80000000, 'i'), ('actor_max', 0, 0x7fffffff, 'i'),
                ('reserved_PC', 8, 0x17f, 'H'), ('unmasked_CW', 8, 0x27e, 'H'),
                ('reserved_CW', 8, 0x227f, 'H'), ('reserved_header16', 10, 1, 'H'),
                ('reserved_header32', 12, 1, 'I'), ('reserved_tail', 96, 1, 'I')]:
            changed_request = bytearray(smoke)
            struct.pack_into('<'+fmt, changed_request, offset, value)
            malformed.append((name, bytes(changed_request), 7, b''))
        for progress in [-1, 1001, -0x80000000, 0x7fffffff]:
            changed_request = bytearray(smoke)
            struct.pack_into('<2i', changed_request, 0, 0, progress)
            malformed.append((f'progress{progress}', bytes(changed_request), 7, b''))
        protocol = []
        for name, data, status, expected in malformed:
            completed = run_small(host, data, checks, name)
            assert completed.returncode == status and completed.stdout == expected and not completed.stderr, name
            protocol.append(dict(case=name, exit=status, reply_bytes=len(expected), exact=True))
        (args.out/'protocol-checks.json').write_text(json.dumps(protocol, indent=2)+'\n')
        runs = {}
        for prefix in ['', 'nonfinite-']:
            input_path, output_path, error_path = [args.out/f'{prefix}{suffix}.bin' for suffix in ['input', 'output', 'stderr']]
            assert not output_path.exists()
            started, completed, failure = time.perf_counter(), None, None
            with input_path.open('rb') as inp, output_path.open('xb') as out, error_path.open('xb') as err:
                try:
                    completed = subprocess.run([str(host)], stdin=inp, stdout=out, stderr=err,
                        creationflags=subprocess.CREATE_NO_WINDOW, timeout=60)
                except Exception as exc:
                    failure = exc
            status = dict(exit=None if completed is None else completed.returncode,
                input_bytes=input_path.stat().st_size, output_bytes=output_path.stat().st_size,
                stderr_bytes=error_path.stat().st_size, elapsed_seconds=time.perf_counter()-started,
                error=None if failure is None else repr(failure))
            (args.out/f'{prefix}status.json').write_text(json.dumps(status, indent=2)+'\n')
            if failure:
                raise failure
            assert completed.returncode == 0 and not status['stderr_bytes'], status
            status.update(inspect(input_path, output_path))
            status['sha256'] = digest(output_path)
            runs[prefix or 'finite_inputs'] = status
        assert runs['finite_inputs']['cases'] == prepared['cases']
        assert runs['nonfinite-']['cases'] == prepared['nonfinite_cases']
        report = dict(prepared, host_executed=True, protocol_checks=len(protocol), runs=runs,
                      hidden_binary_pipes=True, original_game_process=False, GUI=False)
        (args.out/'report.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report, indent=2))
    finally:
        changed_pins = [name for name, sha in pins.items() if not Path(name).exists() or digest(Path(name)) != sha]
        (args.out/'immutability-v1.json').write_text(json.dumps(dict(
            pinned_files=len(pins), unchanged=not changed_pins, changed_files=changed_pins), indent=2)+'\n')
        assert not changed_pins, changed_pins


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    for name in ['host-dir', 'source-exe', 'out']:
        ap.add_argument('--'+name, type=Path, required=True)
    ap.add_argument('--reviewed-host-sha256', required=True)
    ap.add_argument('--execute-existing', action='store_true')
    args = ap.parse_args()
    target = (Path(__file__).resolve().parent.parent/'target').resolve()
    for name in ['host_dir', 'out']:
        getattr(args, name).resolve().relative_to(target)
    manifest = json.loads((args.host_dir/'manifest.json').read_text())
    assert digest(args.host_dir/'host-x86.exe') == manifest['host_exe_sha256'] == args.reviewed_host_sha256
    assert digest(args.host_dir/'host-x86.asm') == manifest['host_source_sha256']
    assert digest(args.source_exe) == manifest['source_sha256'] == SOURCE_SHA256
    assert manifest['protocol']['input_bytes'] == INPUT_BYTES and manifest['protocol']['output_bytes'] == OUTPUT_BYTES
    if args.execute_existing:
        execute(args, manifest)
    else:
        prepare(args, manifest)


if __name__ == '__main__':
    main()

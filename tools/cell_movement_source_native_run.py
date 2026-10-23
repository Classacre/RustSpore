"""Prepare original Cell getter fixtures; execute only a separately reviewed host.

Source definitions and actual source-derived scalar caches are separate lanes;
the latter deliberately use an explicit zero-style definition. No claim is
made that the game bound an arbitrary definition to one of those caches.
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
import subprocess
import time

from cell_capability_scalar_run import digest, run_small
from cell_movement_source_native_probe import INPUT_BYTES, OUTPUT_BYTES
from cell_schema_probe import SOURCE_SHA256

DEFINITION_SHA = 'f0a940e9c6a2a6baa994f5df3b2b21b19bd316920a43cc9e77649547ed168673'
SCALAR_SHA = 'b4fb59ab2c03527ef54ef4a9fb4f968ed775c9eb8871985a15c28d5f98a6e740'
DIFFICULTIES = [-0x80000000, -1, 0, 1, 2, 3, 0x7fffffff]
MASKS = [0, 1, 2, 3, 4, 7, 0x80000008, 0xffffffff]
SPEEDS = [0, 0x80000000, 1, 0x80000001, 0x007fffff, 0x807fffff,
          0x00800000, 0x80800000, 0x3f000000, 0xbf000000, 0x3f800000,
          0xbf800000, 0x3f800001, 0x3f7fffff, 0x7f7fffff, 0xff7fffff]
NONFINITE = [0x7f800000, 0xff800000, 0x7fc00000, 0xffc00001,
             0x7fffffff, 0xffffffff, 0x7f800001, 0xff800001]


def request(definition, difficulty, fc, mask, speed, control=0x27f,
            type_word=0x00b1b104, scope=0x407e0000):
    assert len(definition) == 796
    data = struct.pack('<6IH6x', fc, type_word, scope, difficulty & 0xffffffff,
                       mask, speed, control)+definition+bytes(4)
    assert len(data) == INPUT_BYTES
    return data


def source_definitions(args):
    inventory = json.loads((args.definition_dir/'inventory.json').read_text())
    data = (args.definition_dir/'corpus.bin').read_bytes()
    assert data[:8] == b'RSCELLD1' and digest(args.definition_dir/'corpus.bin') == DEFINITION_SHA
    count = struct.unpack_from('<I', data, 8)[0]
    assert count == inventory['resources'] == len(inventory['records']) == 489
    records, at = [], 12
    with (args.out/'definitions.bin').open('xb') as out, (args.out/'definitions.tsv').open('x', newline='\n') as table:
        table.write('ordinal\tpackage\tlegacy_inventory_key\tsha256\n')
        for ordinal, record in enumerate(inventory['records']):
            _compatibility_id, size, _digest = struct.unpack_from('<QIQ', data, at)
            at += 20
            raw = data[at:at+size]
            at += size
            assert size == 796 and hashlib.sha256(raw).hexdigest() == record['sha256']
            records.append(raw)
            out.write(raw)
            table.write(f'{ordinal}\t{record["package"]}\t{record["key"]}\t{record["sha256"]}\n')
    assert at == len(data)
    return records


def synthetic_definition(styles=(0, 0, 0), inherit=(False, False)):
    raw = bytearray(796)
    for offset, style in zip([0xe0, 0x194, 0x248], styles):
        struct.pack_into('<I', raw, offset+0x10, style)
    for offset, flag in zip([0x194, 0x248], inherit):
        struct.pack_into('<I', raw, offset, 0xffffffff if flag else 0)
    return bytes(raw)


def prepare(args, manifest):
    args.out.mkdir(parents=True, exist_ok=False)
    definitions = source_definitions(args)
    assert digest(args.scalar_dir/'output.bin') == SCALAR_SHA
    source_cache_rows = list(csv.DictReader((args.scalar_dir/'source-cases.tsv').open(newline=''), delimiter='\t'))
    assert len(source_cache_rows) == 5460 and [int(row['case']) for row in source_cache_rows] == list(range(5460))
    contexts, labels, count = Counter(), Counter(), 0
    rng = random.Random(0xe67d20)
    with (args.out/'input.bin').open('xb') as stream, (args.out/'cases.tsv').open('x', newline='\n') as metadata:
        metadata.write('case\tkind\tdefinition\tscalar_cache_case\tcw\n')
        def emit(data, kind, definition=-1, scalar_cache=-1):
            nonlocal count
            control = struct.unpack_from('<H', data, 24)[0]
            stream.write(data)
            metadata.write(f'{count}\t{kind}\t{definition}\t{scalar_cache}\t{control:04X}\n')
            contexts[f'{control:04X}'] += 1
            labels[kind] += 1
            count += 1
        for ordinal, raw in enumerate(definitions):
            for difficulty in [-1, 0, 1, 2, 3]:
                for fc in [0, 1]:
                    for index, mask in enumerate(MASKS):
                        speed = SPEEDS[(ordinal+index) % len(SPEEDS)]
                        emit(request(raw, difficulty, fc, mask, speed),
                             'source_definition_with_explicit_cache_words', ordinal)
        source_definition_cases = count
        assert source_definition_cases == 39120
        zero_definition = synthetic_definition()
        with (args.scalar_dir/'output.bin').open('rb') as native:
            for ordinal, _row in enumerate(source_cache_rows):
                reply = native.read(240)
                mask, speed = struct.unpack_from('<2I', reply, 0x1254-0x1204)
                assert speed & 0x7f800000 != 0x7f800000
                emit(request(zero_definition, [-1, 0, 1, 2, 3][ordinal % 5],
                             1, mask, speed), 'source_scalar_cache_with_explicit_zero_style_definition', scalar_cache=ordinal)
        source_cache_cases = count-source_definition_cases
        assert source_cache_cases == 5460
        for precision, rounding in itertools.product([0, 0x200, 0x300], [0, 0x400, 0x800, 0xc00]):
            control = 0x7f | precision | rounding
            for styles, inherit, difficulty, fc in itertools.product(
                    itertools.product([0, 1, 0x80000008], repeat=3),
                    itertools.product([False, True], repeat=2), DIFFICULTIES, [0, 1]):
                raw = synthetic_definition(styles, inherit)
                emit(request(raw, difficulty, fc, MASKS[count % len(MASKS)],
                             SPEEDS[count % len(SPEEDS)], control), 'synthetic_style_selection_inheritance_product')
            for mask, difficulty, fc in itertools.product([*range(16), 0x80000000, 0xffffffff], [-1, 0, 1, 2, 3], [0, 1]):
                emit(request(zero_definition, difficulty, fc, mask,
                             SPEEDS[count % len(SPEEDS)], control), 'synthetic_cache_mask_bits')
            random_speeds = []
            while len(random_speeds) < 128:
                bits = rng.getrandbits(32)
                if bits & 0x7f800000 != 0x7f800000:
                    random_speeds.append(bits)
            for speed, fc in itertools.product(SPEEDS+random_speeds, [0, 1]):
                emit(request(zero_definition, DIFFICULTIES[count % 7], fc,
                             MASKS[count % len(MASKS)], speed, control,
                             type_word=rng.getrandbits(32), scope=rng.getrandbits(32)),
                     'synthetic_finite_speed_copies_and_keys')
    assert count == 68340, count
    policy_rows = []
    with (args.out/'nonfinite-input.bin').open('xb') as stream, (args.out/'nonfinite-policy.tsv').open('x', newline='\n') as table:
        table.write('case\tfc\tspeed_bits\tcw\texpected_checked_policy\n')
        for precision, rounding, speed, fc in itertools.product([0, 0x200, 0x300], [0, 0x400, 0x800, 0xc00], NONFINITE, [0, 1]):
            control = 0x7f | precision | rounding
            policy = 'NonFiniteCachedSpeed' if fc else 'OkDefault'
            stream.write(request(zero_definition, 2, fc, 0x80000008, speed, control))
            table.write(f'{len(policy_rows)}\t{fc}\t{speed:08X}\t{control:04X}\t{policy}\n')
            policy_rows.append(policy)
    assert len(policy_rows) == 192
    hashes = {name: digest(args.out/name) for name in ['input.bin', 'cases.tsv', 'definitions.bin',
        'definitions.tsv', 'nonfinite-input.bin', 'nonfinite-policy.tsv']}
    report = dict(schema='original-cell-movement-source-cohort-v1', host_executed=False,
        cases=count, source_definition_cases=source_definition_cases, source_definition_copies=489,
        source_scalar_cache_cases=source_cache_cases, source_creations=1365,
        synthetic_cases=count-source_definition_cases-source_cache_cases,
        source_context='027F', policy_counts=dict(contexts), labels=dict(labels),
        nonfinite_policy_cases=len(policy_rows), nonfinite_policy_counts=dict(Counter(policy_rows)),
        finite_agreement_exclusions=0, MXCSR='1F80',
        host_source_sha256=manifest['host_source_sha256'], host_exe_sha256=manifest['host_exe_sha256'],
        definition_corpus_sha256=DEFINITION_SHA, source_scalar_output_sha256=SCALAR_SHA, hashes=hashes,
        scope='Source definition and actual source scalar-cache lanes are separate explicit inputs; no unproven definition/creation pairing or active actor/package/retail FP-state binding. Inventory key is retained verbatim under legacy_inventory_key, never interpreted as canonical resource identity.')
    (args.out/'prepare-report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


def inspect(input_path, output_path):
    count, remainder = divmod(input_path.stat().st_size, INPUT_BYTES)
    assert remainder == 0 and output_path.stat().st_size == count*OUTPUT_BYTES
    contexts, nonfinite, quieted = Counter(), 0, 0
    with input_path.open('rb') as inp, output_path.open('rb') as out:
        for ordinal in range(count):
            req, reply = inp.read(INPUT_BYTES), out.read(OUTPUT_BYTES)
            cw = struct.unpack_from('<H', req, 24)[0]
            contexts[f'{cw:04X}'] += 1
            actual_cw, sw, mxcsr = struct.unpack_from('<HHI', reply, 28)
            assert actual_cw == cw and sw & 0x3800 == 0 and mxcsr == 0x1f80
            assert reply[36:] == bytes(4)
            style, offset, calls, *speeds = struct.unpack_from('<7I', reply)
            assert offset in [0xe0, 0x194, 0x248]
            fc, _, _, difficulty, cache_mask, speed = struct.unpack_from('<6I', req)
            candidate = 0x248 if difficulty == 0 else 0x194 if difficulty == 2 else 0xe0
            raw = req[32:828]
            expected_offset = 0xe0 if candidate != 0xe0 and struct.unpack_from('<I', raw, candidate)[0] == 0xffffffff else candidate
            assert offset == expected_offset
            source_style = struct.unpack_from('<I', raw, offset+16)[0]
            assert style == (1 if not fc else source_style or cache_mask)
            assert calls == (0 if not fc else 1+int(source_style == 0))
            assert speeds == [speeds[0]]*4
            if not fc:
                assert speeds == [0x3f800000]*4
            elif speed & 0x7f800000 != 0x7f800000:
                assert speeds == [speed]*4
            else:
                nonfinite += 1
                quieted += int(speed != speeds[0])
    return dict(cases=count, contexts=dict(contexts), reached_nonfinite_speed_cases=nonfinite,
                nonfinite_speed_payload_changes=quieted)


def execute(args, manifest):
    prepared = json.loads((args.out/'prepare-report.json').read_text())
    assert prepared['host_exe_sha256'] == manifest['host_exe_sha256']
    assert all(digest(args.out/name) == sha for name, sha in prepared['hashes'].items())
    pinned_paths = [args.source_exe, *[args.host_dir/name for name in ['host-x86.asm', 'host-x86.exe', 'manifest.json']],
        args.out/'prepare-report.json', *[args.out/name for name in prepared['hashes']],
        args.definition_dir/'corpus.bin', args.definition_dir/'inventory.json',
        args.scalar_dir/'output.bin', args.scalar_dir/'source-cases.tsv']
    pins = {str(path.resolve()): digest(path) for path in pinned_paths}
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
                     ('partial831', smoke[:-1], 7, b''), ('complete_then_partial', smoke+b'x', 7, first.stdout)]
        for name, offset, value, fmt in [('reserved_PC', 24, 0x17f, 'H'), ('unmasked_CW', 24, 0x27e, 'H'),
                ('reserved_CW', 24, 0x227f, 'H'), ('reserved_header16', 26, 1, 'H'),
                ('reserved_header32', 28, 1, 'I'), ('reserved_tail', 828, 1, 'I')]:
            changed = bytearray(smoke)
            struct.pack_into('<'+fmt, changed, offset, value)
            malformed.append((name, bytes(changed), 7, b''))
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
            runs[prefix or 'finite'] = status
        assert runs['finite']['cases'] == prepared['cases'] and runs['finite']['reached_nonfinite_speed_cases'] == 0
        assert runs['nonfinite-']['cases'] == prepared['nonfinite_policy_cases']
        report = dict(prepared, host_executed=True, protocol_checks=len(protocol), runs=runs,
                      hidden_binary_pipes=True, original_game_process=False, GUI=False)
        (args.out/'report.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report, indent=2))
    finally:
        changed = [name for name, sha in pins.items() if not Path(name).exists() or digest(Path(name)) != sha]
        (args.out/'immutability-v1.json').write_text(json.dumps(dict(
            pinned_files=len(pins), unchanged=not changed, changed_files=changed), indent=2)+'\n')
        assert not changed, changed


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    for name in ['host-dir', 'definition-dir', 'scalar-dir', 'source-exe', 'out']:
        ap.add_argument('--'+name, type=Path, required=True)
    ap.add_argument('--reviewed-host-sha256', required=True)
    ap.add_argument('--execute-existing', action='store_true')
    args = ap.parse_args()
    target = (Path(__file__).resolve().parent.parent/'target').resolve()
    for name in ['host_dir', 'definition_dir', 'scalar_dir', 'out']:
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

"""Validate a pinned curve packet; --run starts only the reviewed owned host.

The814MB immutable request stream is referenced directly on every attempt.
No full input or output stream is copied into Python memory. Process failure
and immutability handling reuse the independently checked runtime-data runner.
"""
import argparse
from collections import Counter
import csv
import json
from pathlib import Path
import struct

import cell_runtime_curves_fixture as source
import cell_runtime_curves_inputs as inputs
import cell_runtime_curves_native_probe_v3 as probe
import cell_runtime_data_native_run as process
from cell_schema_probe import SourceImage, SOURCE_SHA256

HOST = '8a33668111028b95f389607fd714eba65ca6e094f3588db488ee65a100af1c40'
ASM = 'c21135e6b7cedc4a182b032c753afc634be1a2e781d4a3da403bcdd2305dd013'
PEER = 'd084ef9985c25c5177fcb8d032600ccce8c5aa761eb0a49fe9c67accdbb1a3e2'
INPUT = 'abf62621d25227958e06e6331aab7024d49438bb2284ad4aa60cd4bbee1ebbe6'
TABLE = '3077010ebc7d15cbd9ff4e7131fd8c32f018e0afcb4bb675bbd961b744145911'
WITNESSES = '6df4698fdd74e9c684d100eda268d408018aa646093eb5e06de0196142c35c3b'


def preflight(args):
    manifest = json.loads((args.host_dir/'manifest.json').read_text())
    linked = json.loads((args.host_dir/'linked-audit.json').read_text())
    root = json.loads(args.root_review.read_text())
    peer = json.loads(args.peer_review.read_text())
    prepared = json.loads((args.input_dir/'report.json').read_text())
    assert process.digest(args.peer_review) == PEER
    assert manifest['schema'] == 'bounded-original-cell-runtime-curves-host-v3'
    assert manifest['source_sha256'] == SOURCE_SHA256 == root['source_sha256'] == peer['source_sha256']
    assert manifest['host_source_sha256'] == ASM == root['source_assembly_sha256'] == peer['assembly_sha256']
    assert linked['host_exe_sha256'] == HOST == root['host_exe_sha256'] == peer['host_exe_sha256']
    assert root['source_assembly_byte_exact'] and root['source_spans_and_constants_exact']
    assert root['linked_audit_exact'] and root['no_blocking_static_issue_found'] and peer['no_blocking_static_issue_found']
    original = Path(root['original_prepared_dir'])
    assert process.digest(original/'manifest.json') == peer['manifest_sha256']
    assert process.digest(args.host_dir/'linked-audit.json') == peer['linked_audit_sha256']
    assert process.digest(args.host_dir/'host-x86.exe') == HOST and process.digest(args.host_dir/'host-x86.asm') == ASM
    assert process.digest(Path(probe.__file__)) == manifest['preparation_tool_sha256'] == peer['preparation_tool_sha256']
    image = SourceImage(args.source_exe)
    assert probe.audit_source(image) == manifest['code_allowlist']
    assert probe.host_source(image)[0] == (args.host_dir/'host-x86.asm').read_text()
    actual = probe.audit_linked(args.host_dir/'host-x86.exe', args.host_dir/'host-x86.map', image)
    assert all(linked.get(key) == value for key, value in actual.items())
    assert prepared['prepared'] and prepared['cases'] == 84544 and prepared['source_cases'] == 50367
    assert prepared['protocol'] == dict(input_bytes=9632, output_bytes=476)
    assert process.digest(args.input_dir/'input.bin') == prepared['hashes']['input.bin'] == INPUT
    assert process.digest(args.input_dir/'cases.tsv') == prepared['hashes']['cases.tsv'] == TABLE
    assert (args.input_dir/'input.bin').stat().st_size == prepared['input_bytes'] == 84544*9632
    assert process.digest(args.host_dir/'manifest.json') == prepared['host_manifest_sha256']
    assert process.digest(Path(inputs.__file__)) == prepared['tool_sha256']
    assert process.digest(Path(source.__file__)) == prepared['census_tool_sha256']
    assert process.digest(args.census_dir/'report.json') == prepared['census_report_sha256']
    assert process.digest(args.census_dir/'components.jsonl') == prepared['source_components_sha256']
    assert process.digest(args.census_dir/'nonrepresentable-times-v1.tsv') == WITNESSES
    assert process.digest(args.source_rows/'report.json') == prepared['source_rows_report_sha256']
    for name, expected in prepared['source_rows_files'].items():
        assert process.digest(args.source_rows/name) == expected
    return prepared, original


def inspect(input_path, table_path, output_path):
    count, remainder = divmod(input_path.stat().st_size, 9632)
    assert not remainder and output_path.stat().st_size == count*476
    controls, sticky, status, families, policies, finite_output, source_counts = (Counter() for _ in range(7))
    with input_path.open('rb') as inp, output_path.open('rb') as out, table_path.open(newline='') as table:
        rows = csv.DictReader(table, delimiter='\t')
        assert rows.fieldnames == inputs.COLUMNS
        for ordinal, metadata in enumerate(rows):
            req, reply = inp.read(9632), out.read(476)
            assert len(req) == 9632 and len(reply) == 476 and int(metadata['case']) == ordinal
            frame_bits, active, duration_bits, wrap, cw, policy, reserved1, reserved2 = struct.unpack_from('<8I', req)
            assert active <= 160 and policy < 24 and wrap in [0, 8] and cw in source.CONTEXTS
            assert not reserved1 and not reserved2 and source.finite(frame_bits) and source.finite(duration_bits)
            frame = source.number(frame_bits)
            flags, slot = probe.POLICIES[policy]
            times = [source.word(req, 32+60*i) for i in range(active)]
            upper = source.upper_index(times, frame, cw)
            selected = source.neighbors(active, upper, bool(wrap))
            assert source.word(reply, 464) == upper & 0xFFFFFFFF
            assert list(struct.unpack_from('<4I', reply, 448)) == selected
            assert int(metadata['upper']) == upper
            assert [int(metadata[k]) for k in ['previous', 'left', 'right', 'following']] == selected
            assert metadata['frame_bits'] == f'{frame_bits:08X}' and metadata['cw'] == f'{cw:04X}'
            assert metadata['flags'] == f'{flags:08X}' and int(metadata['slot']) == slot
            actual_cw, sw, mxcsr = struct.unpack_from('<HHI', reply, 468)
            assert actual_cw == cw and not sw & 0x3800 and mxcsr & ~0x3F == 0x1F80
            controls[f'{cw:04X}'] += 1
            sticky[f'{mxcsr&0x3F:02X}'] += 1
            status[f'{sw:04X}'] += 1
            families[metadata['family']] += 1
            policies[metadata['expected_policy']] += 1
            offset = probe.SLOTS[slot]
            if not active:
                assert reply[:440] == bytes([0xA5])*440
                assert source.word(reply, 440) == 0xDEADBEEF and source.word(reply, 444) == 0
            else:
                left, right = selected[1:3]
                expected_index = left if frame == source.unsigned_key(times[left], cw) else \
                    right if frame == source.unsigned_key(times[right], cw) else 0xFFFFFFFF
                assert source.word(reply, 440) == expected_index
                assert source.word(reply, 444) == int(expected_index != 0xFFFFFFFF)
                assert reply[:20] == req[32+60*left:32+60*left+20]
                extent = 40 if left == right else 8 if flags & 0x10 else 0
                if left == right:
                    assert reply[offset:offset+40] == req[32+60*left+20:32+60*left+60]
                for at in range(20, 440):
                    if not offset <= at < offset+extent:
                        assert reply[at] == 0xA5
                if extent:
                    is_finite = all(source.finite(source.word(reply, offset+4*i)) for i in range(2))
                    finite_output['finite' if is_finite else 'nonfinite'] += 1
                    if metadata['expected_policy'] == '-':
                        assert is_finite, f'unexpected finite-domain output at case{ordinal}'
            if metadata['family'].startswith('source_'):
                source_counts['cases'] += 1
                source_counts['components_above16rows_cases'] += active > 16
        assert ordinal+1 == count and not inp.read(1) and not out.read(1)
    return dict(cases=count, source_counts=dict(source_counts), controls=dict(controls),
        mxcsr_sticky_counts=dict(sticky), x87_status_counts=dict(status), families=dict(families),
        expected_policy_counts=dict(policies), native_scalar_output_counts=dict(finite_output),
        native_upper_neighbors_and_exact_flags_checked=True, whole_endpoint40_and_unwritten_bytes_checked=True)


def run(args, prepared, original):
    tools = Path(__file__).resolve().parent
    paths = [args.source_exe, args.root_review, args.peer_review,
        *[p for folder in [args.host_dir, original, args.input_dir, args.census_dir, args.source_rows]
          for p in folder.iterdir() if p.is_file()],
        *[tools/name for name in ['cell_runtime_curves_native_run.py', 'cell_runtime_curves_inputs.py',
            'cell_runtime_curves_fixture.py', 'cell_runtime_curves_native_probe_v3.py',
            'cell_runtime_data_native_run.py', 'cell_runtime_data_native_probe.py',
            'cell_schema_probe.py', 'cell_native_math_probe.py', 'native_crt_acos_probe.py']]]
    with process.pinned(paths, args.out):
        host = (args.host_dir/'host-x86.exe').resolve()
        protocol = args.out/'protocol'
        protocol.mkdir()
        with (args.input_dir/'input.bin').open('rb') as stream:
            reference = stream.read(9632)
        reference_path = protocol/'reference-input.bin'
        reference_path.write_bytes(reference)
        result, reference_output = process.reviewed_file_run(host, reference_path, protocol, 'reference')
        assert result['exit'] == 0 and result['stderr_bytes'] == 0 and result['output_bytes'] == 476
        expected = reference_output.read_bytes()  # One fixed-size reply only.
        checks = [('empty', b'', 0, b''), ('two_complete', reference+reference, 0, expected+expected)]
        for size in [1, 31, 32, 9631]:
            checks += [(f'partial{size}', reference[:size], 7, b''),
                       (f'complete_then_partial{size}', reference+reference[:size], 7, expected)]
        for label, at, value in [('count161', 4, 161), ('count_max', 4, 0xFFFFFFFF),
                ('reservedPC', 16, 0x17F), ('unmaskedCW', 16, 0x27E), ('highCW', 16, 0x1027F),
                ('bad_header_flags', 12, 9), ('bad_policy', 20, 24), ('reserved24', 24, 1), ('reserved28', 28, 1),
                ('frameInf', 0, 0x7F800000), ('frameNaN', 0, 0x7FC12345),
                ('durationInf', 8, 0xFF800000), ('durationSNaN', 8, 0x7F800001)]:
            changed = bytearray(reference)
            struct.pack_into('<I', changed, at, value)
            checks.append((label, bytes(changed), 7, b''))
        results = []
        for label, raw, exit_code, reply in checks:
            path = protocol/f'{label}-input.bin'
            path.write_bytes(raw)
            result, output = process.reviewed_file_run(host, path, protocol, label)
            assert result['exit'] == exit_code and result['stderr_bytes'] == 0
            assert output.read_bytes() == reply
            results.append(dict(case=label, exit=exit_code, bytes=result['output_bytes']))
        (protocol/'report.json').write_text(json.dumps(dict(checks=len(results), results=results), indent=2)+'\n')
        result, output = process.reviewed_file_run(host, args.input_dir/'input.bin', args.out, 'corpus', timeout=180)
        assert result['exit'] == 0 and result['stderr_bytes'] == 0
        summary = inspect(args.input_dir/'input.bin', args.input_dir/'cases.tsv', output)
        assert summary['cases'] == prepared['cases'] and summary['source_counts']['cases'] == prepared['source_cases']
        assert summary['expected_policy_counts'] == prepared['expected_policy_counts']
        summary.update(schema='full-channel-runtime-curves-native-run-v1', native_executed=True,
            source_sha256=SOURCE_SHA256, host_exe_sha256=HOST, host_source_sha256=ASM,
            input_sha256=INPUT, metadata_sha256=TABLE, native_output_sha256=process.digest(output),
            native_output_path=str(output.resolve()), protocol_checks=len(results),
            source_nondiscrete_time_witnesses=12, source_nondiscrete_time_components=6,
            witness_sha256=WITNESSES,
            scope='Only reviewed owned host runs sourceupper/neighbors/INFO+kind3 sampler on complete source channel rows. No suppliedupper,croppedrows,eventdispatch,POS/ROT,retailprocess,initializer,GUI or network.')
        (args.out/'report.json').write_text(json.dumps(summary, indent=2)+'\n')
        print(json.dumps(summary, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-exe', type=Path, default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    for name in ['host-dir', 'root-review', 'peer-review', 'input-dir', 'census-dir', 'source-rows', 'out']:
        parser.add_argument('--'+name, type=Path, required=True)
    parser.add_argument('--run', action='store_true')
    args = parser.parse_args()
    target = Path(__file__).resolve().parent.parent/'target'
    for name in ['host_dir', 'root_review', 'peer_review', 'input_dir', 'census_dir', 'source_rows', 'out']:
        getattr(args, name).resolve().relative_to(target.resolve())
    prepared, original = preflight(args)
    args.out.mkdir(parents=True, exist_ok=False)
    (args.out/'preflight.json').write_text(json.dumps(dict(host_exe_sha256=HOST, host_source_sha256=ASM,
        input_sha256=INPUT, metadata_sha256=TABLE, cases=prepared['cases'], source_cases=prepared['source_cases'],
        runner_sha256=process.digest(Path(__file__)), native_executed=False), indent=2)+'\n')
    if args.run:
        run(args, prepared, original)
    else:
        print('Preflight only: reviewed host/input/source pins valid; no process executed.')


if __name__ == '__main__':
    main()

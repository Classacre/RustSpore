"""Review and optionally execute the bounded original root-record constructor.

Only the owned fixture executable runs. Files, source/linked audits and all
prepared references are pinned. Its fail code71 includes allocation failures,
so no failed process is retried or silently classified as a native success.
"""
import argparse
import json
from pathlib import Path
import struct

import anim_root_binding_fixture as fixture
import anim_root_binding_native_probe_v3 as probe
from cell_runtime_data_native_run import digest, pinned, saved_file_run
from cell_schema_probe import SourceImage, SOURCE_SHA256

ROOT = Path(__file__).resolve().parent.parent
HOST = '74913f0962e3207027a8d9a3cb1913a72507f04845a756777068a4c979a2e1b2'
ASM = 'f90f85c41ced8120ae44c9746c7100f2932990bdb651e7bbd2ed39380d0010be'


def save(path, value):
    with path.open('x') as output:
        json.dump(value, output, indent=2)
        output.write('\n')


def review(args):
    manifest = json.loads((args.host_dir/'manifest.json').read_text())
    linked = json.loads((args.host_dir/'linked-audit.json').read_text())
    cohort = json.loads((args.input_dir/'manifest.json').read_text())
    assert manifest['source_sha256'] == SOURCE_SHA256 and manifest['v2_instructions_unchanged']
    assert digest(args.host_dir/'host-x86.exe') == linked['host_exe_sha256'] == HOST
    assert digest(args.host_dir/'host-x86.asm') == manifest['host_source_sha256'] == ASM
    assert digest(Path(probe.__file__)) == manifest['preparation_tool_sha256']
    for path, expected in manifest['preserved_v2_pins'].items():
        assert digest(ROOT/path) == expected, path
    for name, expected in cohort['files'].items():
        assert digest(args.input_dir/name) == expected, name
    for path, expected in cohort['sources'].items():
        assert digest(ROOT/path) == expected, path
    image = SourceImage(args.source_exe)
    assert probe.v2.audit(image) == manifest['source_audit']
    assert probe.host_source(image) == (args.host_dir/'host-x86.asm').read_text()
    actual = probe.linked_audit(args.host_dir/'host-x86.exe', args.host_dir/'host-x86.map', image)
    assert actual == linked
    count = 0
    with (args.input_dir/'inputs.bin').open('rb') as requests, (args.input_dir/'reference.bin').open('rb') as expected:
        while raw := requests.read(364):
            assert len(raw) == 364 and fixture.reference(raw) == expected.read(48), count
            count += 1
        assert expected.read(1) == b''
    rejected = (args.input_dir/'rejected.bin').read_bytes()
    assert len(rejected) == 6*364
    for index in range(6):
        try:
            fixture.reference(rejected[index*364:(index+1)*364])
        except ValueError:
            pass
        else:
            raise AssertionError(('negative accepted', index))
    assert count == cohort['records'] == 15113
    return manifest, cohort


def execute(args, manifest, cohort):
    folder = args.out/'native-v1'
    folder.mkdir(exist_ok=False)
    tools = ROOT/'tools'
    paths = [args.source_exe, Path(__file__), args.out/'preflight.json',
        *[path for path in args.host_dir.iterdir() if path.is_file()],
        args.input_dir/'manifest.json', *[args.input_dir/name for name in cohort['files']],
        *[ROOT/name for name in cohort['sources']],
        *[ROOT/name for name in manifest['preserved_v2_pins']],
        tools/'anim_root_binding_fixture.py', tools/'anim_root_binding_native_probe.py',
        tools/'anim_root_binding_native_probe_v3.py', tools/'cell_runtime_data_native_run.py',
        tools/'cell_schema_probe.py', tools/'native_crt_acos_probe.py',
        ROOT/'crates/spore-core/src/anim_root_binding.rs',
        ROOT/'crates/spore-bevy/examples/anim_root_binding_verify.rs.pending']
    paths = list(dict.fromkeys(path.resolve() for path in paths))
    host = (args.host_dir/'host-x86.exe').resolve()
    with pinned(paths, folder):
        protocol = folder/'protocol'
        protocol.mkdir()
        with (args.input_dir/'inputs.bin').open('rb') as source:
            smoke = source.read(364)
        first = fixture.reference(smoke)
        cases = [('empty', b'', 0, b''), ('one', smoke, 0, first)]
        for size in [1, 63, 363]:
            cases += [(f'partial{size}', smoke[:size], 71, b''),
                (f'complete_then_partial{size}', smoke+smoke[:size], 71, first)]
        rejected = (args.input_dir/'rejected.bin').read_bytes()
        for index in range(6):
            cases.append((f'unsupported{index}', rejected[index*364:(index+1)*364], 71, b''))
        for at, value, label in [(12, 256, 'channel_byte256'), (16, 256, 'channel_index256'),
                (28, 2, 'mirror2'), (32, 256, 'owner256'), (44, 256, 'secondary256'),
                (48, 1, 'reserved_header'), (76, 1, 'unused_descriptor'), (361, 1, 'tail_padding')]:
            raw = bytearray(smoke)
            if at == 361:
                raw[at] = value
            else:
                struct.pack_into('<I', raw, at, value)
            cases.append((label, bytes(raw), 71, b''))
        results = []
        for label, raw, code, expected in cases:
            request = protocol/f'{label}-input.bin'
            request.write_bytes(raw)
            status, output = saved_file_run(host, request, protocol, label)
            assert status['exit'] == code and status['stderr_bytes'] == 0, (label, status)
            assert output.read_bytes() == expected, label
            results.append(dict(case=label, exact=True, exit=status['exit']))
        save(protocol/'report.json', dict(checks=len(results), cases=results))
        status, output = saved_file_run(host, args.input_dir/'inputs.bin', folder, 'corpus', timeout=120)
        assert status['exit'] == 0 and status['stderr_bytes'] == 0, status
        assert output.stat().st_size == cohort['records']*48
        assert digest(output) == cohort['files']['reference.bin']
        report = dict(schema='original-root-record-constructor-native-run-v1', exact=True,
            native_executed=True, cases=cohort['records'], source_contexts=cohort['source_contexts'],
            source_clips=cohort['source_clips'], source_descriptors=cohort['source_descriptors'],
            protocol_checks=len(results), source_sha256=SOURCE_SHA256, host_exe_sha256=HOST,
            host_assembly_sha256=ASM, input_sha256=cohort['files']['inputs.bin'],
            output_path=str(output.resolve()), native_projection_sha256=digest(output),
            python_projection_sha256=cohort['files']['reference.bin'],
            retries=0, scratch_scope='Admitted mirror branches and constructor are integer-only with explicit entry values. No general floating-point/scratch-register or partial-GPR ABI guarantee.',
            scope=cohort['source_scope'])
        save(folder/'report.json', report)
        print(json.dumps(report, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host-dir', type=Path, required=True)
    parser.add_argument('--input-dir', type=Path, required=True)
    parser.add_argument('--source-exe', type=Path, default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--run', action='store_true')
    args = parser.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve())
    manifest, cohort = review(args)
    args.out.mkdir(parents=True, exist_ok=False)
    save(args.out/'preflight.json', dict(schema='reviewed-root-constructor-preflight-v1',
        host_exe_sha256=HOST, host_assembly_sha256=ASM, runner_sha256=digest(Path(__file__)),
        cases=cohort['records'], negative_cases=6, native_execution_requested=args.run))
    if args.run:
        execute(args, manifest, cohort)
    else:
        print(json.dumps(dict(preflight=True, cases=cohort['records'], native_executed=False)))


if __name__ == '__main__':
    main()

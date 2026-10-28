"""Review fresh root ROT requests and run only the already accepted fixture host.

Original packets stay frozen. A separate comparison view adapts metadata for
the existing strict ROT verifier without changing source rows or arithmetic.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import struct

import anim_rotation_native_probe as probe
import native_crt_acos_probe as crt
from anim_rotation_native_run import inspect, neighbors
from cell_runtime_data_native_run import digest, pinned, reviewed_file_run

ROOT = Path(__file__).resolve().parent.parent
HOST_DIR = ROOT/'target/animation/rotation-native-v2'
HOST = '972bf058d3cf7711b2e9f0ea5ff63c03d5e4e3f1a7a6dafa7fa17653e9d27f18'
ASM = 'b914ed8a7ff6cd972b60cfc3733628489895ec9a2e538a79988547f27f8e84c6'
INPUT = 'cbb39538e4c7290f4832b8bb4560f88bd230a401b736d84d93661c50fb9ab6b2'


def save(path, value):
    with path.open('x') as output:
        json.dump(value, output, indent=2)
        output.write('\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input-dir', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--run', action='store_true')
    args = parser.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve())
    manifest = json.loads((args.input_dir/'manifest.json').read_text())
    original_host = json.loads((HOST_DIR/'manifest.json').read_text())
    assert manifest['rows'] == 3702 and manifest['covered_clips'] == 215
    assert manifest['requested_clips'] == 217 and manifest['missing_clips'] == 2
    assert digest(HOST_DIR/'host-x86.exe') == HOST
    assert digest(HOST_DIR/'host-x86.asm') == ASM
    for name, expected in manifest['pins'].items():
        assert digest(ROOT/name) == expected, name
    for name, expected in manifest['files'].items():
        assert digest(args.input_dir/name) == expected, name
    sources = [Path(original_host['source_path']), Path(original_host['crt_source_path'])]
    assert digest(sources[0]) == original_host['source_sha256']
    assert digest(sources[1]) == original_host['crt_source_sha256']
    crt.decoder()  # Initialize the pinned local decoder runtime before audit.
    assert probe.audit_host(HOST_DIR/'host-x86.exe', HOST_DIR/'host-x86.map') == original_host['host_code_audit']
    raw = (args.input_dir/'inputs.bin').read_bytes()
    index = json.loads((args.input_dir/'input-index.json').read_text())
    assert digest(args.input_dir/'inputs.bin') == INPUT and len(raw) == 3702*1952
    assert len(index) == 3702
    for n, item in enumerate(index):
        record = raw[n*1952:(n+1)*1952]
        frame, count, duration, flags, cw, blend, enabled, reserved = struct.unpack_from('<fIf5I', record)
        assert item['row'] == n and 1 <= count <= 8 and flags in (0, 8)
        assert cw == 0x027f and blend == enabled == 1 and reserved == 0
        assert -2**24 <= frame <= 2**24 and 1 <= duration <= 2**24
        frames = [struct.unpack_from('<I', record, 32+i*120)[0] for i in range(count)]
        assert frames == sorted(frames) and frames[-1] <= min(2**24, duration-1)
        assert record[32+count*120:] == bytes((16-count)*120)
        assert hashlib.sha256(record[32:]).hexdigest() == item['source_row_window_sha256']
        upper = sum(key <= frame for key in frames)
        local = neighbors(count, upper, flags & 8)
        assert list(local) == item['local_neighbors'] and upper == item['local_upper']
        assert [item['local_source_indices'][i] for i in local] == item['full_neighbors']
        for i in range(count):
            for offset in [20, 24, 28, 32, 36, 104, 108]:
                assert struct.unpack_from('<I', record, 32+i*120+offset)[0] & 0x7f800000 != 0x7f800000
    args.out.mkdir(parents=True, exist_ok=False)
    save(args.out/'preflight.json', dict(schema='fresh-root-rot-cohort-preflight-v1',
        rows=3702, source_clips=215, requested_clips=217, excluded_clips=2,
        input_sha256=INPUT, host_exe_sha256=HOST, runner_sha256=digest(Path(__file__)),
        native_execution_requested=args.run))
    if not args.run:
        print(json.dumps(dict(preflight=True, native_executed=False, rows=3702)))
        return
    view = args.out/'comparison-input-view'
    view.mkdir()
    shutil.copyfile(args.input_dir/'inputs.bin', view/'inputs.bin')
    save(view/'manifest.json', dict(input_sha256=INPUT, source_packet=str(args.input_dir.resolve())))
    save(view/'input-index.json', [dict(item, family='source-first-root', source=item['key']) for item in index])
    paths = [Path(__file__), *sources, args.out/'preflight.json', args.input_dir/'manifest.json',
        *[ROOT/name for name in manifest['pins']], *[args.input_dir/name for name in manifest['files']],
        *[p for p in HOST_DIR.iterdir() if p.is_file()], *[p for p in view.iterdir() if p.is_file()],
        *[ROOT/'tools'/name for name in ['anim_rotation_native_probe.py', 'anim_rotation_native_run.py',
            'anim_rotation_compare.py', 'anim_quaternion_probe.py', 'native_crt_acos_probe.py',
            'cell_runtime_data_native_run.py']]]
    paths = list(dict.fromkeys(path.resolve() for path in paths))
    try:
        with pinned(paths, args.out):
            status, output = reviewed_file_run((HOST_DIR/'host-x86.exe').resolve(),
                args.input_dir/'inputs.bin', args.out, 'corpus')
            assert status['exit'] == 0 and status['stderr_bytes'] == 0 and not status['timeout'], status
            assert status['exception'] is None, status
            summary = inspect(raw, output.read_bytes())
            save(args.out/'report.json', dict(schema='fresh-root-rot-native-cohort-v1',
                native_executed=True, rows=3702, covered_source_clips=215, excluded_source_clips=2,
                raw_output=str(output.resolve()), raw_output_sha256=digest(output),
                structural_summary=summary, scope='Fresh original ROT100 replies at independently accepted POS frames. Strict source windows and host metadata checked. Rust numeric comparison, coordinate conversion, generated-body binding and visible pose remain separate.'))
            print(json.dumps(dict(native_executed=True, rows=3702, output=str(output), summary=summary)))
    except BaseException as error:
        save(args.out/'failure.json', dict(error=repr(error), true_failure_retained=True))
        raise


if __name__ == '__main__':
    main()

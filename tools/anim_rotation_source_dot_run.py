"""Run only the reviewed unchanged ROT host on pinned new dot-boundary inputs."""
import argparse
from collections import Counter
import json
from pathlib import Path
import struct
import time

import anim_quaternion_probe as quat
import anim_rotation_native_probe as probe
from anim_rotation_native_run import inspect, saved_run
from anim_rotation_source_dot_probe import ASM, HOST, corpus
import native_crt_acos_probe as crt

INPUT = 'a91eca29e4deff04b30fb77e2b5dd3ce028a9ef6a15200151c0edcd2274d12c0'
INDEX = 'd847dea0fad10ae07eaed694b33efc8f212ebce20358fe5cf945fe4cd96394f7'
ROWS = 3456


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host-dir', type=Path, required=True)
    parser.add_argument('--input-dir', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    target = (Path(__file__).resolve().parent.parent/'target').resolve()
    fixture, inputs, out = args.host_dir.resolve(), args.input_dir.resolve(), args.out.resolve()
    for path in [fixture, inputs, out]:
        path.relative_to(target)
    host_manifest = (fixture/'manifest.json').read_bytes()
    host_meta = json.loads(host_manifest)
    input_manifest = (inputs/'manifest.json').read_bytes()
    input_meta = json.loads(input_manifest)
    assert input_meta['schema'] == 'source-dot-overflow-unordered-rot-cohort-v1'
    assert input_meta['input_sha256'] == INPUT and input_meta['rows'] == ROWS
    assert input_meta['host_exe_sha256'] == HOST and input_meta['host_source_sha256'] == ASM
    assert host_meta['host_exe_sha256'] == HOST and host_meta['host_source_sha256'] == ASM
    for name, sha in [('host-x86.exe', HOST), ('host-x86.asm', ASM)]:
        assert quat.digest((fixture/name).read_bytes()) == sha
    originals = [(Path(host_meta['source_path']), quat.SOURCE_SHA256),
                 (Path(host_meta['crt_source_path']), crt.SOURCE_SHA256)]
    for path, sha in originals:
        assert quat.digest(path.read_bytes()) == sha
    crt.decoder()
    assert probe.audit_host(fixture/'host-x86.exe', fixture/'host-x86.map') == host_meta['host_code_audit']
    data = (inputs/'inputs.bin').read_bytes()
    index_bytes = (inputs/'input-index.json').read_bytes()
    index = json.loads(index_bytes)
    assert quat.digest(data) == INPUT and quat.digest(index_bytes) == INDEX
    assert len(data) == ROWS*probe.INPUT_BYTES and len(index) == ROWS
    expected_data, expected_index = corpus()
    assert data == expected_data and index == expected_index
    out.mkdir(parents=True, exist_ok=False)
    (out/'reviewed-host-manifest.json').write_bytes(host_manifest)
    (out/'reviewed-input-manifest.json').write_bytes(input_manifest)
    started = time.perf_counter()
    host = fixture/'host-x86.exe'
    smokes = {}
    for blend in [0, 1]:
        for kind in ['positive-infinity', 'negative-infinity', 'unordered']:
            row = next(i for i, item in enumerate(index) if item['blend'] == blend
                       and item['primary_dot'] == kind and item['frame_bits'] == '40E00000'
                       and item['cw'] == '027F' and item['previous_quaternion_mode'] == 1
                       and item['next_quaternion_mode'] == 1)
            request = data[row*probe.INPUT_BYTES:(row+1)*probe.INPUT_BYTES]
            result = saved_run(host, request, out, f'smoke-blend{blend}-{kind}')
            assert result.returncode == 0 and not result.stderr
            inspect(request, result.stdout)
            smokes[(blend, kind)] = (request, result.stdout)
    request, expected = smokes[(0, 'unordered')]
    negatives = [('empty', b'', 0, b'')]
    for size in [1, 31, probe.INPUT_BYTES-1]:
        negatives += [(f'partial{size}', request[:size], 7, b''),
                      (f'complete_then_partial{size}', request+request[:size], 7, expected)]
    for name, offset in [('quaternion_nan', 52), ('weight_nan', 68), ('slope_nan', 136)]:
        modified = bytearray(request)
        struct.pack_into('<I', modified, offset, 0x7FC00000)
        negatives.append((name, bytes(modified), 7, b''))
    negatives.append(('policy_change', request+smokes[(1, 'unordered')][0], 8, expected))
    checks = out/'protocol'
    checks.mkdir()
    protocol = []
    for name, request, code, expected in negatives:
        result = saved_run(host, request, checks, name)
        assert result.returncode == code and result.stdout == expected and not result.stderr, name
        protocol.append(dict(case=name, exit=code, exact=True,
            input_sha256=quat.digest(request), output_sha256=quat.digest(result.stdout)))
    (out/'protocol-checks.json').write_text(json.dumps(protocol, indent=2)+'\n')
    chunks, summaries = [], []
    for blend in [0, 1]:
        positions = [i for i, item in enumerate(index) if item['blend'] == blend]
        assert positions == list(range(positions[0], positions[-1]+1))
        request = data[positions[0]*probe.INPUT_BYTES:(positions[-1]+1)*probe.INPUT_BYTES]
        result = saved_run(host, request, out, f'corpus-blend{blend}')
        assert result.returncode == 0 and not result.stderr
        summaries.append(inspect(request, result.stdout))
        chunks.append(result.stdout)
    output = b''.join(chunks)
    (out/'output.bin').write_bytes(output)
    summary = inspect(data, output)
    for name, sha in [('host-x86.exe', HOST), ('host-x86.asm', ASM)]:
        assert quat.digest((fixture/name).read_bytes()) == sha
    for path, sha in originals:
        assert quat.digest(path.read_bytes()) == sha
    assert (fixture/'manifest.json').read_bytes() == host_manifest
    assert (inputs/'manifest.json').read_bytes() == input_manifest
    assert quat.digest((inputs/'inputs.bin').read_bytes()) == INPUT
    assert quat.digest((inputs/'input-index.json').read_bytes()) == INDEX
    report = dict(schema='reviewed-source-dot-boundary-rot-run-v1', rows=ROWS,
        input_sha256=INPUT, native_output_sha256=quat.digest(output), host_exe_sha256=HOST,
        protocol_checks=len(protocol), primary_dot_counts=dict(Counter(item['primary_dot'] for item in index)),
        elapsed_seconds=time.perf_counter()-started, original_sources_and_fixture_unchanged=True,
        hidden_binary_pipes=True, original_modules_loaded=False, retail_process_started=False, gui_used=False,
        summary=summary, by_blend=summaries,
        scope='Unchanged reviewed whole INFO+ROT100 branch; finite inputs deliberately produce positive/negative infinity or unordered intermediate dots under explicit policies. Native output bits remain raw. No retail actor/state or full-source sampling claim.')
    (out/'run.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k: v for k, v in report.items() if k not in {'summary', 'by_blend'}}, indent=2))
    print(json.dumps(summary['output_modes'], indent=2))


if __name__ == '__main__':
    main()

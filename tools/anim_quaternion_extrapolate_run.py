"""Run only the separately reviewed pinned source-extrapolation host hidden."""
import argparse
import json
from pathlib import Path
import struct
import time

import anim_quaternion_probe as base
from anim_quaternion_run import contexts
from native_crt_acos_run import run
import native_crt_acos_probe as crt

HOST = '230d8218b480fe9b3e269bb6bde6272a4b0af2f85e8ddaa82d7394dec4cec806'
ASM = '92d898cb7ef6828b987ac848985d04e01066329bec147db0892e1394ac02c33d'
INPUT = 'ea27fa9d532e14c04851696fded5817ab3e46f75bf1aab22b5671d3b821310b6'


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--host-dir', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    args = p.parse_args()
    target = (Path(__file__).resolve().parent.parent/'target').resolve()
    fixture, out = args.host_dir.resolve(), args.out.resolve()
    fixture.relative_to(target)
    out.relative_to(target)
    manifest_bytes = (fixture/'manifest.json').read_bytes()
    manifest = json.loads(manifest_bytes)
    assert manifest['schema'] == 'bounded-original-quaternion-extrapolation-host-v1'
    pins = {'host-x86.exe': HOST, 'host-x86.asm': ASM, 'inputs.bin': INPUT,
            'constants.bin': crt.WINDOW_SHA256}
    for name, sha in pins.items():
        assert base.digest((fixture/name).read_bytes()) == sha
    assert manifest['host_exe_sha256'] == HOST and manifest['host_source_sha256'] == ASM
    sources = [(Path(manifest['source_path']), base.SOURCE_SHA256),
               (Path(manifest['crt_source_path']), crt.SOURCE_SHA256)]
    for path, sha in sources:
        assert base.digest(path.read_bytes()) == sha
    crt.decoder()
    assert base.audit_host(fixture/'host-x86.exe', fixture/'host-x86.map') == manifest['host_code_audit']
    data = (fixture/'inputs.bin').read_bytes()
    inputs = list(struct.iter_unpack('<12I', data))
    assert len(inputs) == 86520 and len(set(inputs)) == 86520
    assert all(row[0] == 2 for row in inputs)
    out.mkdir(parents=True, exist_ok=False)
    (out/'reviewed-manifest.json').write_bytes(manifest_bytes)
    checks = out/'protocol'
    checks.mkdir()
    smoke_row = (2, 0x27F, 0, 0, 0, 0x3F800000, 0, 0, 0x3F800000, 0, 0xBF800000, 0)
    assert smoke_row in inputs
    smoke = struct.pack('<12I', *smoke_row)
    host, started = fixture/'host-x86.exe', time.perf_counter()
    result = run(host, smoke, out)
    (out/'smoke-input.bin').write_bytes(smoke)
    (out/'smoke-output.bin').write_bytes(result.stdout)
    (out/'smoke-stderr.bin').write_bytes(result.stderr)
    (out/'smoke-status.json').write_text(json.dumps(dict(exit=result.returncode,
        output_bytes=len(result.stdout), stderr_bytes=len(result.stderr)), indent=2)+'\n')
    assert result.returncode == 0 and len(result.stdout) == 32 and not result.stderr
    contexts([smoke_row], result.stdout)
    smoke_output = result.stdout
    negatives = [('empty', b'', 0, b'')]
    negatives += [(f'partial{n}', smoke[:n], 7, b'') for n in range(1, 48)]
    negatives += [(f'complete_then_partial{n}', smoke+smoke[:n], 7, smoke_output) for n in range(1, 48)]
    for name, index, value in [('negative_below_minus_one', 10, 0xBF800001),
        ('positive_above_two', 10, 0x40000001), ('fraction_positive_infinity', 10, 0x7F800000),
        ('fraction_negative_infinity', 10, 0xFF800000), ('fraction_nan', 10, 0x7FC00000),
        ('operation3', 0, 3), ('reserved_CW', 1, 0x17F), ('unmasked_CW', 1, 0x27E),
        ('component_infinity', 2, 0x7F800000), ('component_nan', 3, 0x7FC00000),
        ('component_above_four', 4, 0x40800001), ('flags2', 11, 2), ('slerp_NULL_flag', 11, 1)]:
        row = list(smoke_row)
        row[index] = value
        negatives.append((name, struct.pack('<12I', *row), 7, b''))
    protocol = []
    for name, request, code, expected in negatives:
        result = run(host, request, out)
        (checks/f'{name}-input.bin').write_bytes(request)
        (checks/f'{name}-output.bin').write_bytes(result.stdout)
        (checks/f'{name}-stderr.bin').write_bytes(result.stderr)
        (checks/f'{name}-status.json').write_text(json.dumps(dict(exit=result.returncode,
            output_bytes=len(result.stdout), stderr_bytes=len(result.stderr)), indent=2)+'\n')
        assert result.returncode == code and result.stdout == expected and not result.stderr, name
        protocol.append(dict(case=name, exit=code, input_sha256=base.digest(request),
                             output_sha256=base.digest(result.stdout), exact=True))
    (out/'protocol-checks.json').write_text(json.dumps(protocol, indent=2)+'\n')
    result = run(host, data, out)
    (out/'output.bin').write_bytes(result.stdout)
    (out/'stderr.bin').write_bytes(result.stderr)
    (out/'corpus-status.json').write_text(json.dumps(dict(exit=result.returncode,
        output_bytes=len(result.stdout), stderr_bytes=len(result.stderr)), indent=2)+'\n')
    assert result.returncode == 0 and not result.stderr
    state, state_bytes = contexts(inputs, result.stdout)
    (out/'contexts.bin').write_bytes(state_bytes)
    for name, sha in pins.items():
        assert base.digest((fixture/name).read_bytes()) == sha
    for path, sha in sources:
        assert base.digest(path.read_bytes()) == sha
    assert (fixture/'manifest.json').read_bytes() == manifest_bytes
    report = dict(schema='reviewed-source-quaternion-extrapolation-run-v1', rows=len(inputs),
        reviewed_manifest_sha256=base.digest(manifest_bytes), host_exe_sha256=HOST,
        source_sha256=base.SOURCE_SHA256, crt_source_sha256=crt.SOURCE_SHA256,
        input_sha256=INPUT, output_sha256=base.digest(result.stdout), protocol_checks=len(protocol),
        elapsed_seconds=time.perf_counter()-started, contexts=state,
        hidden_binary_pipes=True, original_module_load_requested=False, retail_process_started=False,
        gui_used=False, original_sources_and_fixture_unchanged=True,
        scope='Source slerp extrapolation[-1,2], all12 explicit masked CW contexts, SSE2 MXCSR1F80. No retail dispatch/whole curve claim.')
    (out/'run.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

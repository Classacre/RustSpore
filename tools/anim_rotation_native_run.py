"""Run only a separately reviewed pinned INFO+ROT100 source host hidden."""
import argparse
import bisect
from collections import Counter
import json
from pathlib import Path
import struct
import time

import anim_rotation_native_probe as probe
import anim_quaternion_probe as quat
import native_crt_acos_probe as crt
from native_crt_acos_run import run

HOST = '972bf058d3cf7711b2e9f0ea5ff63c03d5e4e3f1a7a6dafa7fa17653e9d27f18'
ASM = 'b914ed8a7ff6cd972b60cfc3733628489895ec9a2e538a79988547f27f8e84c6'
INPUT = '4c4c90b6745f7dcc30d598324671ee067afb563311b5d193b4940a5b4ff450b4'
ROWS = 30240


def neighbors(count, upper, wrap):
    right = min(upper, count-1)
    left = right if upper == count else max(upper-1, 0)
    previous = left-1 if left else (count-2) % count if wrap else left
    following = right+1 if right < count-1 else (right+2) % count if wrap else right
    return previous, left, right, following


def inspect(data, output):
    assert len(data) % probe.INPUT_BYTES == 0
    rows = len(data)//probe.INPUT_BYTES
    assert len(output) == rows*probe.OUTPUT_BYTES
    counts, states = Counter(), Counter()
    for n in range(rows):
        request = data[n*probe.INPUT_BYTES:(n+1)*probe.INPUT_BYTES]
        reply = output[n*probe.OUTPUT_BYTES:(n+1)*probe.OUTPUT_BYTES]
        frame, count, duration, flags, cw, blend, enabled, reserved = struct.unpack_from('<fIf5I', request)
        actual_index, exact = struct.unpack_from('<2I', reply, 200)
        actual_neighbors = struct.unpack_from('<4I', reply, 208)
        right, mxcsr, actual_cw, sw, reserved = struct.unpack_from('<2I2HI', reply, 224)
        assert actual_cw == cw and mxcsr & ~0x3F == 0x1F80 and sw & 0x3800 == 0 and reserved == 0
        states[(cw, mxcsr, sw)] += 1
        assert reply[20:100] == b'\xA5'*80
        if count == 0:
            assert reply[:200] == b'\xA5'*200 and exact == 0 and actual_index == 0xDEADBEEF
            assert actual_neighbors == (0xFFFFFFFF,)*4 and right == 0
            counts['empty'] += 1
            continue
        frames = [struct.unpack_from('<I', request, 32+i*120)[0] for i in range(count)]
        upper = bisect.bisect_right(frames, frame)
        expected = neighbors(count, upper, flags & 8)
        assert actual_neighbors == expected and right == expected[2], (n, actual_neighbors, expected, right)
        left, right = expected[1:3]
        expected_index = left if frame == frames[left] else right if frame == frames[right] else 0xFFFFFFFF
        assert actual_index == expected_index and exact == int(expected_index != 0xFFFFFFFF)
        assert reply[:20] == request[32+left*120:52+left*120]
        if left == right:
            assert reply[100:200] == request[52+left*120:152+left*120]
            counts['copied'] += 1
        elif not enabled:
            assert reply[100:200] == b'\xA5'*100
            counts['interpolation_disabled'] += 1
        else:
            assert reply[120:200] == b'\xA5'*80
            values = struct.unpack_from('<5I', reply, 100)
            counts['interpolated_nonfinite' if any(v & 0x7F800000 == 0x7F800000 for v in values)
                   else 'interpolated_finite'] += 1
    return dict(rows=rows, output_modes=dict(counts),
        context_counts=[dict(cw=f'{cw:04X}', mxcsr=f'{m:08X}', sw=f'{s:04X}', rows=c)
                        for (cw, m, s), c in sorted(states.items())])


def saved_run(host, data, out, prefix):
    result = run(host, data, out)
    (out/f'{prefix}-input.bin').write_bytes(data)
    (out/f'{prefix}-output.bin').write_bytes(result.stdout)
    (out/f'{prefix}-stderr.bin').write_bytes(result.stderr)
    (out/f'{prefix}-status.json').write_text(json.dumps(dict(exit=result.returncode,
        input_bytes=len(data), output_bytes=len(result.stdout), stderr_bytes=len(result.stderr)), indent=2)+'\n')
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--host-dir', type=Path, required=True)
    p.add_argument('--input-dir', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    args = p.parse_args()
    target = (Path(__file__).resolve().parent.parent/'target').resolve()
    fixture, inputs, out = args.host_dir.resolve(), args.input_dir.resolve(), args.out.resolve()
    for path in [fixture, inputs, out]:
        path.relative_to(target)
    manifest_bytes = (fixture/'manifest.json').read_bytes()
    manifest = json.loads(manifest_bytes)
    input_manifest_bytes = (inputs/'manifest.json').read_bytes()
    input_manifest = json.loads(input_manifest_bytes)
    assert manifest['schema'] == 'bounded-original-info-rot100-curve-host-v1'
    assert manifest['host_exe_sha256'] == HOST and manifest['host_source_sha256'] == ASM
    for name, sha in [('host-x86.exe', HOST), ('host-x86.asm', ASM)]:
        assert quat.digest((fixture/name).read_bytes()) == sha
    source_paths = [(Path(manifest['source_path']), quat.SOURCE_SHA256),
                    (Path(manifest['crt_source_path']), crt.SOURCE_SHA256)]
    for path, sha in source_paths:
        assert quat.digest(path.read_bytes()) == sha
    crt.decoder()
    assert probe.audit_host(fixture/'host-x86.exe', fixture/'host-x86.map') == manifest['host_code_audit']
    data = (inputs/'inputs.bin').read_bytes()
    assert quat.digest(data) == INPUT and input_manifest['input_sha256'] == INPUT
    assert len(data) == ROWS*probe.INPUT_BYTES
    index = json.loads((inputs/'input-index.json').read_text())
    assert len(index) == ROWS
    out.mkdir(parents=True, exist_ok=False)
    (out/'reviewed-host-manifest.json').write_bytes(manifest_bytes)
    (out/'reviewed-input-manifest.json').write_bytes(input_manifest_bytes)
    started = time.perf_counter()
    host = fixture/'host-x86.exe'
    smokes = []
    for blend in [0, 1]:
        n = next(i for i, item in enumerate(index) if item['blend'] == blend and item['family'] == 'four'
                 and item['frame_bits'] == '40E00000' and item['enabled'] == 1 and item['wrap'] == 0 and item['cw'] == '027F')
        smoke = data[n*probe.INPUT_BYTES:(n+1)*probe.INPUT_BYTES]
        result = saved_run(host, smoke, out, f'smoke-blend{blend}')
        assert result.returncode == 0 and not result.stderr and len(result.stdout) == probe.OUTPUT_BYTES
        inspect(smoke, result.stdout)
        smokes.append((smoke, result.stdout))
    smoke, expected = smokes[0]
    negatives = [('empty', b'', 0, b'')]
    for size in [1, 31, 32, probe.INPUT_BYTES-1]:
        negatives += [(f'partial{size}', smoke[:size], 7, b''),
                      (f'complete_then_partial{size}', smoke+smoke[:size], 7, expected)]
    for name, offset, value in [('count17', 4, 17), ('frame_nan', 0, 0x7FC00000),
        ('duration_nan', 8, 0x7FC00000), ('duration_too_short', 8, 0x3F800000),
        ('unknown_flags', 12, 1), ('reserved_PC', 16, 0x17F), ('unmasked_CW', 16, 0x27E),
        ('blend2', 20, 2), ('enabled2', 24, 2), ('reserved_header', 28, 1),
        ('key_too_large', 32, 0x1000001), ('unsorted_key', 32+120, 0),
        ('quaternion_nan', 52, 0x7FC00000), ('weight_nan', 68, 0x7FC00000),
        ('slope_nan', 136, 0x7FC00000)]:
        modified = bytearray(smoke)
        if name == 'unsorted_key':
            struct.pack_into('<I', modified, 32, 4)
        struct.pack_into('<I', modified, offset, value)
        negatives.append((name, bytes(modified), 7, b''))
    negatives.append(('policy_change', smokes[0][0]+smokes[1][0], 8, smokes[0][1]))
    checks = out/'protocol'
    checks.mkdir()
    protocol = []
    for name, request, code, expected in negatives:
        result = saved_run(host, request, checks, name)
        assert result.returncode == code and result.stdout == expected and not result.stderr, name
        protocol.append(dict(case=name, exit=code, input_sha256=quat.digest(request),
                             output_sha256=quat.digest(result.stdout), exact=True))
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
    for path, sha in source_paths:
        assert quat.digest(path.read_bytes()) == sha
    assert (fixture/'manifest.json').read_bytes() == manifest_bytes
    assert (inputs/'manifest.json').read_bytes() == input_manifest_bytes
    assert quat.digest((inputs/'inputs.bin').read_bytes()) == INPUT
    report = dict(schema='reviewed-info-rot100-native-run-v1', rows=ROWS, input_sha256=INPUT,
        native_output_sha256=quat.digest(output), host_exe_sha256=HOST,
        protocol_checks=len(protocol), elapsed_seconds=time.perf_counter()-started,
        source_sha256=quat.SOURCE_SHA256, crt_source_sha256=crt.SOURCE_SHA256,
        hidden_binary_pipes=True, original_modules_loaded=False, retail_process_started=False, gui_used=False,
        original_sources_and_fixture_unchanged=True, summary=summary, by_blend=summaries,
        scope='Original INFO+ROT100 branch under explicit bounded frame/CW/MXCSR/blend/descriptor contexts. Source windows and full source census remain separate; raw nonfinite outputs are retained.')
    (out/'run.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k: v for k, v in report.items() if k not in {'summary', 'by_blend'}}, indent=2))
    print(json.dumps(summary['output_modes'], indent=2))


if __name__ == '__main__':
    main()

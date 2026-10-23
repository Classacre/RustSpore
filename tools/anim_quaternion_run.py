"""Execute only the separately reviewed, pinned v2 quaternion fixture.

This uses hidden binary pipes, never loads the original modules and never
starts the retail game. Raw requests/replies and audits stay under target/.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct
import time

import anim_quaternion_probe as probe
from native_crt_acos_run import run
import native_crt_acos_probe as crt

REVIEWED_HOST = '7621203e6f41906c35febd00b9b24ad508417d8299561621b11c4c59d9317909'
REVIEWED_ASM = '54f0fea6efbb77c3a1c778857ebd6737c2dd52f16dbb36315f4cd15db4aedca7'
REVIEWED_INPUT = 'af3b78c5de8c0e1444bf5bb0e7c71b731293ec6de91fd6646afbbb4fa2e83a2d'
REVIEWED_TSV = 'd16df22646c2e7c9bd0430c4c8ec0874f770f555fca346f558f6e7bd76c08cd4'
ROWS = 86124


def digest(data):
    return hashlib.sha256(data).hexdigest()


def contexts(inputs, output):
    assert len(output) == len(inputs)*32
    counts, mxcsrs, statuses = Counter(), Counter(), Counter()
    results, state = bytearray(), bytearray()
    nonfinite = 0
    for n, row in enumerate(inputs):
        values = struct.unpack_from('<5I', output, n*32)
        mxcsr, cw, sw, reserved = struct.unpack_from('<IHHI', output, n*32+20)
        assert cw == row[1] and cw in probe.CONTROLS
        assert mxcsr & ~0x3F == 0x1F80 and sw & 0x3800 == 0 and reserved == 0
        if row[0] == 2:
            assert sw & 0x0400 == 0
        if row[0] != 0 or row[11] == 1:
            assert values[4] == probe.SENTINEL
        if any(v & 0x7F800000 == 0x7F800000 for v in values[:4]):
            assert row[0] == 0
            nonfinite += 1
        counts[cw] += 1
        mxcsrs[mxcsr] += 1
        statuses[sw] += 1
        results += output[n*32:n*32+20]
        state += output[n*32+20:n*32+28]
    return dict(control_word_counts={f'{k:04X}': v for k, v in sorted(counts.items())},
        mxcsr_counts={f'{k:08X}': v for k, v in sorted(mxcsrs.items())},
        x87_status_word_counts={f'{k:04X}': v for k, v in sorted(statuses.items())},
        finite_result_rows=len(inputs)-nonfinite, nonfinite_normalization_rows=nonfinite,
        result_bytes=len(results), result_bits_sha256=digest(results),
        context_bytes=len(state), context_sha256=digest(state)), bytes(state)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host-dir', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    target = (Path(__file__).resolve().parent.parent/'target').resolve()
    fixture, out = args.host_dir.resolve(), args.out.resolve()
    fixture.relative_to(target)
    out.relative_to(target)
    manifest_bytes = (fixture/'manifest.json').read_bytes()
    manifest = json.loads(manifest_bytes)
    assert manifest['schema'] == 'bounded-original-quaternion-host-v1'
    assert manifest['source_sha256'] == probe.SOURCE_SHA256
    assert manifest['crt_source_sha256'] == crt.SOURCE_SHA256
    assert manifest['host_exe_sha256'] == REVIEWED_HOST
    assert manifest['host_source_sha256'] == REVIEWED_ASM
    assert manifest['protocol']['input_rows'] == ROWS
    pins = {'host-x86.exe': REVIEWED_HOST, 'host-x86.asm': REVIEWED_ASM,
        'inputs.bin': REVIEWED_INPUT, 'inputs.tsv': REVIEWED_TSV,
        'constants.bin': crt.WINDOW_SHA256}
    files = {}
    for name, expected in pins.items():
        files[name] = (fixture/name).read_bytes()
        assert digest(files[name]) == expected, name
    source_path, dll_path = Path(manifest['source_path']), Path(manifest['crt_source_path'])
    assert digest(source_path.read_bytes()) == probe.SOURCE_SHA256
    assert digest(dll_path.read_bytes()) == crt.SOURCE_SHA256
    actual_imports = crt.PE(fixture/'host-x86.exe').imports()
    assert {dll.upper() for dll, _ in actual_imports.values()} == {'KERNEL32.DLL'}
    assert {name for _, name in actual_imports.values()} == crt.IMPORTS
    crt.decoder()  # Locate the pinned local inspection dependency first.
    assert probe.audit_host(fixture/'host-x86.exe', fixture/'host-x86.map') == manifest['host_code_audit']
    inputs = list(struct.iter_unpack('<12I', files['inputs.bin']))
    assert len(inputs) == ROWS and len(set(inputs)) == ROWS
    assert Counter(row[0] for row in inputs) == {int(k): v for k, v in manifest['protocol']['operation_counts'].items()}
    out.mkdir(parents=True, exist_ok=False)
    (out/'reviewed-manifest.json').write_bytes(manifest_bytes)
    checks = out/'protocol'
    checks.mkdir()
    # Source clamped dot=0, spherical interpolation under CW027F.
    smoke_row = (2, 0x27F, 0, 0, 0, 0x3F800000, 0, 0, 0x3F800000, 0, 0x3F000000, 0)
    assert smoke_row in inputs
    smoke = struct.pack('<12I', *smoke_row)
    host = fixture/'host-x86.exe'
    started = time.perf_counter()
    result = run(host, smoke, out)
    (out/'smoke-input.bin').write_bytes(smoke)
    (out/'smoke-output.bin').write_bytes(result.stdout)
    (out/'smoke-stderr.bin').write_bytes(result.stderr)
    assert result.returncode == 0 and len(result.stdout) == 32 and not result.stderr
    contexts([smoke_row], result.stdout)
    smoke_output = result.stdout
    negatives = [('empty', b'', 0, b'')]
    negatives += [(f'partial{n}', smoke[:n], 7, b'') for n in range(1, 48)]
    negatives += [(f'complete_then_partial{n}', smoke+smoke[:n], 7, smoke_output) for n in range(1, 48)]
    changes = [('operation3', 0, 3), ('reserved_precision', 1, 0x17F),
        ('unmasked_exception', 1, 0x27E), ('unexpected_control_bits', 1, 0x1227F),
        ('positive_infinity', 2, 0x7F800000), ('negative_infinity', 3, 0xFF800000),
        ('quiet_nan', 4, 0x7FC00000), ('signaling_nan', 5, 0x7F800001),
        ('above_four', 6, 0x40800001), ('negative_above_four', 7, 0xC0800001),
        ('fraction_above_one', 10, 0x3F800001), ('negative_fraction', 10, 0xBE800000),
        ('negative_subnormal_fraction', 10, 0x80000001), ('fraction_infinity', 10, 0x7F800000),
        ('fraction_nan', 10, 0x7FC00000), ('unknown_flags', 11, 2), ('null_length_on_slerp', 11, 1)]
    for name, index, value in changes:
        row = list(smoke_row)
        row[index] = value
        negatives.append((name, struct.pack('<12I', *row), 7, b''))
    invalid = struct.pack('<12I', *([3]+list(smoke_row[1:])))
    negatives += [('complete_then_invalid', smoke+invalid, 7, smoke_output),
                  ('invalid_then_complete', invalid+smoke, 7, b'')]
    protocol = []
    for name, data, exit_code, expected in negatives:
        result = run(host, data, out)
        (checks/f'{name}-input.bin').write_bytes(data)
        (checks/f'{name}-output.bin').write_bytes(result.stdout)
        (checks/f'{name}-stderr.bin').write_bytes(result.stderr)
        assert result.returncode == exit_code and result.stdout == expected and not result.stderr, name
        protocol.append(dict(case=name, exit=exit_code, input_bytes=len(data), reply_bytes=len(result.stdout),
            input_sha256=digest(data), reply_sha256=digest(result.stdout), stderr_sha256=digest(result.stderr), exact=True))
    protocol_bytes = (json.dumps(protocol, indent=2)+'\n').encode()
    (out/'protocol-checks.json').write_bytes(protocol_bytes)
    result = run(host, files['inputs.bin'], out)
    (out/'output.bin').write_bytes(result.stdout)
    (out/'stderr.bin').write_bytes(result.stderr)
    assert result.returncode == 0 and not result.stderr, (result.returncode, len(result.stdout))
    context, context_data = contexts(inputs, result.stdout)
    (out/'contexts.bin').write_bytes(context_data)
    smoke_index = inputs.index(smoke_row)
    assert result.stdout[smoke_index*32:(smoke_index+1)*32] == smoke_output
    assert (fixture/'manifest.json').read_bytes() == manifest_bytes
    for name, expected in pins.items():
        assert digest((fixture/name).read_bytes()) == expected, name
    assert digest(source_path.read_bytes()) == probe.SOURCE_SHA256
    assert digest(dll_path.read_bytes()) == crt.SOURCE_SHA256
    report = dict(schema='reviewed-original-quaternion-run-v1',
        reviewed_manifest_sha256=digest(manifest_bytes), host_exe_sha256=REVIEWED_HOST,
        host_source_sha256=REVIEWED_ASM, source_sha256=probe.SOURCE_SHA256,
        crt_source_sha256=crt.SOURCE_SHA256, original_x86_instructions_executed=True,
        original_module_load_requested=False, retail_process_started=False, gui_used=False,
        execution=dict(create_no_window=True, show_window=0, standard_input='binary pipe',
            standard_output='binary pipe', exit_code=result.returncode, elapsed_seconds=time.perf_counter()-started),
        rows=ROWS, input_sha256=REVIEWED_INPUT, native_output_sha256=digest(result.stdout),
        protocol_checks=len(protocol), protocol_checks_sha256=digest(protocol_bytes),
        contexts=context, fixture_artifacts_unchanged=True, original_sources_unchanged=True,
        scope='Explicit masked CW and SSE2 MXCSR1F80 finite binding, no retail dispatch or assembled animation claim.')
    (out/'run.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

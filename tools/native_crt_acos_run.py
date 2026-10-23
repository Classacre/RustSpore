"""Execute only the separately reviewed, SHA-pinned v1 finite SSE2 fixture.

Uses a hidden process and binary standard pipes. It never loads the source DLL
or starts the retail game. Every request, reply and check remains under target/.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import time

from native_crt_acos_probe import PE, IMPORTS, SOURCE_SHA256, WINDOW_SHA256, branch

REVIEWED_HOST = '6df1877f3683b8375b59825baa5793d2d8f382f148fd635e91772ff968558ecd'
REVIEWED_ASM = '864d4302007368780cd9b9362affd0bb90b227156e2f9a25f1de6ddbc689de90'
REVIEWED_INPUT = 'a87141fb22e124205e3c668675bf1e25314a3a0bacef25d41b8de11b566bdf2d'
REVIEWED_TSV = '8252a6b4f488545b2d3a25c3712db2869b4946a15282fb197a7b3405b49dba2e'
ROWS = 62624


def digest(data):
    return hashlib.sha256(data).hexdigest()


def run(host, data, cwd):
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = 0
    return subprocess.run([str(host)], input=data, capture_output=True, cwd=cwd,
        creationflags=subprocess.CREATE_NO_WINDOW, startupinfo=startup, timeout=60)


def contexts(data):
    counts, flags, status = Counter(), Counter(), Counter()
    packed = bytearray()
    results = bytearray()
    for value, mxcsr, cw, sw in struct.iter_unpack('<QIHH',data):
        assert cw == 0x27f
        assert mxcsr & ~0x3f == 0x1f80
        assert sw & 0x3800 == 0
        assert value & 0x7ff0000000000000 != 0x7ff0000000000000
        assert value >> 63 == 0
        counts[cw] += 1
        flags[mxcsr] += 1
        status[sw] += 1
        packed += struct.pack('<IHH',mxcsr,cw,sw)
        results += struct.pack('<Q',value)
    return dict(control_word_counts={f'{k:04X}':v for k,v in sorted(counts.items())},
        mxcsr_counts={f'{k:08X}':v for k,v in sorted(flags.items())},
        x87_status_word_counts={f'{k:04X}':v for k,v in sorted(status.items())},
        context_bytes=len(packed),context_sha256=digest(packed),
        result_bytes=len(results),result_bits_sha256=digest(results)),bytes(packed)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host-dir',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True)
    args = parser.parse_args()
    target = (Path(__file__).resolve().parent.parent/'target').resolve()
    fixture, out = args.host_dir.resolve(),args.out.resolve()
    fixture.relative_to(target)
    out.relative_to(target)
    manifest_bytes = (fixture/'manifest.json').read_bytes()
    manifest = json.loads(manifest_bytes)
    assert manifest['source_sha256'] == SOURCE_SHA256
    assert manifest['host_exe_sha256'] == REVIEWED_HOST
    assert manifest['host_source_sha256'] == REVIEWED_ASM
    assert manifest['protocol']['input_rows'] == ROWS
    pins = {'host-x86.exe':REVIEWED_HOST,'host-x86.asm':REVIEWED_ASM,
        'inputs.bin':REVIEWED_INPUT,'inputs.tsv':REVIEWED_TSV,'constants.bin':WINDOW_SHA256}
    files = {}
    for name, expected in pins.items():
        files[name] = (fixture/name).read_bytes()
        assert digest(files[name]) == expected,name
    source_path = Path(manifest['source_path'])
    assert digest(source_path.read_bytes()) == SOURCE_SHA256
    host = fixture/'host-x86.exe'
    actual_imports = PE(host).imports()
    assert {dll.upper() for dll,_ in actual_imports.values()} == {'KERNEL32.DLL'}
    assert {name for _,name in actual_imports.values()} == IMPORTS
    inputs = files['inputs.bin']
    assert len(inputs) == ROWS*8
    values = [x[0] for x in struct.iter_unpack('<Q',inputs)]
    assert len(set(values)) == ROWS
    assert Counter(branch(x) for x in values) == manifest['protocol']['branch_counts']
    out.mkdir(parents=True,exist_ok=False)
    (out/'reviewed-manifest.json').write_bytes(manifest_bytes)
    checks = out/'protocol'
    checks.mkdir()
    smoke = struct.pack('<Q',0x3fe0000000000000)
    start = time.perf_counter()
    result = run(host,smoke,out)
    (out/'smoke-input.bin').write_bytes(smoke)
    (out/'smoke-output.bin').write_bytes(result.stdout)
    (out/'smoke-stderr.bin').write_bytes(result.stderr)
    assert result.returncode == 0 and len(result.stdout) == 16 and not result.stderr
    contexts(result.stdout)
    smoke_output = result.stdout

    negatives = [('empty',b'',0,b'')]
    negatives += [(f'partial{n}',smoke[:n],7,b'') for n in range(1,8)]
    negatives += [(f'complete_then_partial{n}',smoke+smoke[:n],7,smoke_output) for n in range(1,8)]
    for name,bits in [('above_one_ulp',0x3ff0000000000001),('positive_infinity',0x7ff0000000000000),
        ('negative_infinity',0xfff0000000000000),('quiet_nan',0x7ff8000000000000),
        ('signaling_nan',0x7ff0000000000001),('negative_half',0xbfe0000000000000),
        ('negative_min_subnormal',0x8000000000000001),('negative_one',0xbff0000000000000),
        ('max_finite',0x7fefffffffffffff)]:
        negatives.append((name,struct.pack('<Q',bits),7,b''))
    bad = struct.pack('<Q',0x7ff0000000000000)
    negatives += [('complete_then_invalid',smoke+bad,7,smoke_output),
                  ('invalid_then_complete',bad+smoke,7,b'')]
    protocol = []
    for name,data,exit_code,expected in negatives:
        result = run(host,data,out)
        (checks/f'{name}-input.bin').write_bytes(data)
        (checks/f'{name}-output.bin').write_bytes(result.stdout)
        (checks/f'{name}-stderr.bin').write_bytes(result.stderr)
        assert result.returncode == exit_code and result.stdout == expected and not result.stderr,name
        protocol.append(dict(case=name,exit=exit_code,input_bytes=len(data),reply_bytes=len(result.stdout),
            input_sha256=digest(data),reply_sha256=digest(result.stdout),stderr_sha256=digest(result.stderr),exact=True))
    protocol_bytes = (json.dumps(protocol,indent=2)+'\n').encode()
    (out/'protocol-checks.json').write_bytes(protocol_bytes)

    # The entire fresh reviewed stream is supplied in one hidden piped run.
    (out/'input.bin').write_bytes(inputs)
    result = run(host,inputs,out)
    (out/'output.bin').write_bytes(result.stdout)
    (out/'stderr.bin').write_bytes(result.stderr)
    assert result.returncode == 0 and not result.stderr,(result.returncode,len(result.stdout))
    assert len(result.stdout) == 16*ROWS
    context, context_data = contexts(result.stdout)
    replies = list(struct.iter_unpack('<QIHH',result.stdout))
    subnormal_rows = [i for i,bits in enumerate(values)
        if 0 < bits & 0x7fffffffffffffff < 0x0010000000000000]
    assert [i for i,(_,_,_,sw) in enumerate(replies) if sw == 2] == subnormal_rows
    assert [values[i] for i,(_,mxcsr,_,_) in enumerate(replies) if mxcsr == 0x1f80] == [0x3ff0000000000000]
    (out/'contexts.bin').write_bytes(context_data)
    assert result.stdout[16*values.index(0x3fe0000000000000):16*(values.index(0x3fe0000000000000)+1)] == smoke_output
    assert struct.unpack_from('<Q',result.stdout,16*values.index(0x3ff0000000000000))[0] == 0
    rows = []
    for i,(bits,(value,_,_,_)) in enumerate(zip(values,struct.iter_unpack('<QIHH',result.stdout))):
        rows.append(f'{i:06d}\t{branch(bits)}\t{value:016x}\n')
    expected_tsv = ''.join(rows).encode('ascii')
    (out/'native-expected.tsv').write_bytes(expected_tsv)

    # Recheck every admitted executable/data artifact after execution.
    assert (fixture/'manifest.json').read_bytes() == manifest_bytes
    for name,expected in pins.items():
        assert digest((fixture/name).read_bytes()) == expected,name
    assert digest(source_path.read_bytes()) == SOURCE_SHA256
    canary = b'\xa5'*32
    inventory = dict(schema='reviewed-crt-acos-sse2-run-v1',source_sha256=SOURCE_SHA256,
        host_exe_sha256=REVIEWED_HOST,host_source_sha256=REVIEWED_ASM,
        reviewed_manifest_sha256=digest(manifest_bytes),original_x86_instructions_executed=True,
        original_dll_load_requested=False,retail_process_started=False,gui_used=False,
        execution=dict(create_no_window=True,show_window=0,standard_input='binary pipe',
            standard_output='binary pipe',standard_error='binary pipe',timeout_seconds=60),
        cases=ROWS,original_calls_per_case=2,original_calls=ROWS*2,
        smoke_original_calls=2,protocol_original_calls=2*sum(len(x[3])//16 for x in negatives),
        total_original_calls=2*(ROWS+1+sum(len(x[3])//16 for x in negatives)),
        hidden_host_processes=len(negatives)+2,
        branch_counts=manifest['protocol']['branch_counts'],host_exit=result.returncode,
        request_bytes=len(inputs),reply_bytes=len(result.stdout),input_sha256=digest(inputs),
        output_sha256=digest(result.stdout),stderr_sha256=digest(result.stderr),
        native_expected_tsv_sha256=digest(expected_tsv),**context,
        binary64_subnormal_input_cases=len(subnormal_rows),denormal_status_rows_exact=True,
        context_contract='SSE MXCSR control1F80, wrapper x87CW027F, final x87TOP0; not retail state',
        scratch_lane_patterns=['all-zero','all-one'],scratch_output_equality_checks=ROWS,
        canary_contract=dict(checks_passed=ROWS*8,checked_dwords_per_case=8,
            expected_bytes_per_case=32,expected_bytes_sha256=digest(canary),
            evidence='Successful exit means all compiled equality checks passed. Canary bytes are not transmitted in reply protocol.'),
        protocol_checks=len(protocol),protocol_rejection_checks=sum(x['exit']==7 for x in protocol),
        protocol_checks_sha256=digest(protocol_bytes),source_and_fixture_hashes_unchanged=True,
        elapsed_seconds=round(time.perf_counter()-start,3),
        scope='Reviewed finite SSE2 code and borrowed data in owned no-CRT host only; no legacy, retail dispatch, gameplay, DLL loading or visual fidelity claim.')
    (out/'inventory.json').write_text(json.dumps(inventory,indent=2)+'\n')
    print(json.dumps(inventory,indent=2))


if __name__ == '__main__':
    main()

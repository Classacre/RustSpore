"""Prepare a fresh [-1,2] source-slerp fixture, never execute it.

Reuses the reviewed primitive/CRT byte audit and immediate-result guards.
Only the owned input fraction admission changes from the accepted v2 host.
"""
import argparse
from collections import Counter
import difflib
import json
from pathlib import Path
import struct
import subprocess

import anim_quaternion_probe as base
import native_crt_acos_probe as crt
from cell_schema_probe import SourceImage, SOURCE_SHA256

ROOT = Path(__file__).resolve().parent.parent
OLD = '\n'.join(['mov eax,dword ptr [InputBuffer+40]', 'mov edx,eax', 'and eax,07FFFFFFFh',
    'cmp eax,03F800000h', 'ja HostFailure', 'test edx,080000000h', 'jz ValidFraction',
    'test eax,eax', 'jnz HostFailure', 'ValidFraction:'])
NEW = '\n'.join(['mov eax,dword ptr [InputBuffer+40]', 'mov edx,eax', 'and eax,07FFFFFFFh',
    'test edx,080000000h', 'jnz NegativeFraction', 'cmp eax,040000000h', 'ja HostFailure',
    'jmp ValidFraction', 'NegativeFraction:', 'cmp eax,03F800000h', 'ja HostFailure', 'ValidFraction:'])


def corpus():
    pairs = sorted({row[2:10] for row in base.corpus() if row[0] == 2})
    fractions = [0xBF800000, 0xBF7FFFFF, 0xBF400000, 0xBF000000, 0x80000001,
                 0x80000000, 0, 1, 0x3F000000, 0x3F800000, 0x3F800001,
                 0x3FC00000, 0x3FFFFFFF, 0x40000000]
    return [(2, cw, *pair, amount, 0) for pair in pairs for amount in fractions for cw in base.CONTROLS]


def prepare(args):
    out = args.out.resolve()
    out.relative_to((ROOT/'target').resolve())
    image, dll = SourceImage(args.exe), crt.PE(args.dll)
    assert base.digest(dll.data) == crt.SOURCE_SHA256
    crt.decoder()
    source, thunk = base.audit_source(image)
    crt_source, bounds = crt.audit_source(dll)
    constants = dll.read(crt.WINDOW[0], crt.WINDOW[1]-crt.WINDOW[0])
    assert base.digest(constants) == crt.WINDOW_SHA256
    original = base.host_source(image, dll)
    assert original.count(OLD) == 1
    assembly_text = original.replace(OLD, NEW)
    out.mkdir(parents=True, exist_ok=False)
    rows = corpus()
    inputs = b''.join(struct.pack('<12I', *row) for row in rows)
    (out/'inputs.bin').write_bytes(inputs)
    (out/'constants.bin').write_bytes(constants)
    (out/'v2-domain-only.diff').write_text(''.join(difflib.unified_diff(
        original.splitlines(True), assembly_text.splitlines(True),
        fromfile='accepted-v2-owned-host', tofile='extrapolation-owned-host')))
    asm, obj, exe, map_path = [out/p for p in ['host-x86.asm', 'host-x86.obj', 'host-x86.exe', 'host-x86.map']]
    asm.write_text(assembly_text)
    commands = [[str(args.sdk_bin_x86/'ml.exe'), '/nologo', '/c', '/coff', f'/Fo{obj}', str(asm)],
        [str(args.sdk_bin_x86/'link.exe'), '/NOLOGO', '/MACHINE:X86', '/SUBSYSTEM:CONSOLE',
         '/ENTRY:HostMain', '/BASE:0x24000000', '/DYNAMICBASE:NO', '/SAFESEH:NO',
         f'/OUT:{exe}', f'/MAP:{map_path}', str(obj), str(args.kernel32_lib)]]
    for command in commands:
        result = subprocess.run(command, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout+'\n'+result.stderr
    owned = base.audit_host(exe, map_path)
    manifest = dict(schema='bounded-original-quaternion-extrapolation-host-v1', host_executed=False,
        source_path=str(args.exe.resolve()), source_sha256=SOURCE_SHA256,
        crt_source_path=str(args.dll.resolve()), crt_source_sha256=crt.SOURCE_SHA256,
        source_code_allowlist=source, source_thunk=thunk,
        source_constants=[dict(address=f'{a:08X}', bytes=n, sha256=base.digest(image.read(a, n))) for a, n in base.CONSTANTS],
        crt_code_allowlist=crt_source, crt_indexed_read_bound_proof=bounds,
        constant_window=dict(start=f'{crt.WINDOW[0]:08X}', end_exclusive=f'{crt.WINDOW[1]:08X}', sha256=crt.WINDOW_SHA256),
        domain='All fixture cases call slerp with finite |components|<=4 and amount[-1,2]; all12 explicit masked CW contexts, MXCSR1F80. Host still also admits its accepted bounded normalize/add operations, but this corpus uses only operation2.',
        range_proof='Finite SSE dot bounded by64, original clamp -> abs-dot[-0,+1], explicit finite SSE2 CRT theta[0,pi/2]. Both amount and1-amount are in[-1,2]; every FSIN argument magnitude<=pi<4. Native C2=0 checked after every slerp.',
        owned_host_difference='Only integer fraction admission changed from accepted immediate-guard v2. Exact retained diff in v2-domain-only.diff. Original spans/thunk/constants/CRT bytes and all other owned instructions unchanged.',
        pages=[dict(address=f'{a:08X}', bytes=65536, initial='RW', final=p) for a, p in base.PAGES],
        host_code_audit=owned, host_source_sha256=base.digest(asm.read_bytes()), host_exe_sha256=base.digest(exe.read_bytes()),
        original_modules_loaded=False, compile_commands=commands,
        protocol=dict(input_bytes=48, output_bytes=32, input_rows=len(rows),
            input_sha256=base.digest(inputs), fraction_counts={f'{k:08X}': v for k, v in Counter(row[10] for row in rows).items()},
            checks='Two zero/all-one XMM calls per row; immediate12-byte result-tail sentinel before context stores; final canaries; cdecl ESP; TOP; CW; MXCSR1F80 control; slerp C2. Exact48-byte reads and integer domain/EOF guards inherited unchanged.',
            input_format='Same12little-endian words as accepted quaternion-v2; operation2 only.',
            output_format='Same32-byte quaternion/length-sentinel/context record as accepted quaternion-v2.'),
        next_step='Root must review this fresh exact fixture before any separately requested native execution.')
    (out/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps({k: manifest[k] for k in ['host_executed', 'host_source_sha256', 'host_exe_sha256', 'protocol']}, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('exe', type=Path)
    parser.add_argument('dll', type=Path)
    parser.add_argument('--sdk-bin-x86', type=Path, required=True)
    parser.add_argument('--kernel32-lib', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    prepare(parser.parse_args())


if __name__ == '__main__':
    main()

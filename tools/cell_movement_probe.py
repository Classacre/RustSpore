"""Translate the bounded original SSE axial block for x64 CPU conformance.

Only ABI/memory operands and branch labels change. Scalar instruction order is
read from the immutable source executable through Microsoft's SDK disassembler.
Generated source-derived assembly/DLL and synthetic fixtures stay ignored.
"""
import argparse
import ctypes
import hashlib
import json
from pathlib import Path
import random
import re
import struct
import subprocess

from cell_schema_probe import SourceImage, SOURCE_SHA256


def translated_kernel(listing):
    lines = [
        '.code', 'PUBLIC CellAxialKernel', 'CellAxialKernel PROC FRAME',
        'sub rsp, 56', '.allocstack 56',
        'movups xmmword ptr [rsp], xmm6', '.savexmm128 xmm6, 0',
        'movups xmmword ptr [rsp+16], xmm7', '.savexmm128 xmm7, 16', '.endprolog',
        'stmxcsr dword ptr [rsp+40]', 'mov dword ptr [rsp+44], 01F80h',
        'ldmxcsr dword ptr [rsp+44]',
        'movss xmm2, dword ptr [rdx]', 'movss xmm3, dword ptr [rdx+4]',
        'movss xmm0, dword ptr [rdx+8]',
    ]
    instructions = 0
    allowed = {'movss', 'movaps', 'mulss', 'subss', 'addss', 'comiss', 'jbe', 'jmp'}
    for line in listing.splitlines():
        match = re.match(r'\s*([0-9A-F]+): (\w+)\s+(.*)', line)
        if not match:
            continue
        address, operation, operands = match.groups()
        if not 0xe6f707 <= int(address, 16) < 0xe6f82a:
            continue
        assert operation in allowed, (address, operation)
        if operation in {'jbe', 'jmp'}:
            target = int(operands, 16)
            assert 0xe6f707 <= target < 0xe6f82a
            operands = f'L_{target:08X}'
        else:
            def global_memory(match):
                address = int(match[1], 16)
                assert 0x15a8c28 <= address <= 0x15a8c48 and address % 4 == 0
                return f'[r8+{address-0x15a8c28}]'
            operands = re.sub(r'ds:\[([0-9A-F]+)h\]', global_memory, operands)

            def actor_memory(match):
                offset = int(match[1], 16)
                assert offset in [0x4c, 0x50, 0x54]
                return f'[rcx+{offset-0x4c}]'
            operands = re.sub(r'\[esi\+([0-9A-F]+)h\]', actor_memory, operands)

            def stack_memory(match):
                offset = int(match[1], 16)
                return {4: '[r9]', 8: '[r9+4]', 12: '[r9+8]',
                    16: '[rsp+32]', 20: '[rsp+36]'}[offset]
            operands = re.sub(r'\[esp\+([0-9A-F]+)h?\]', stack_memory, operands)
            assert not re.search(r'\b(?:eax|ecx|edx|esi|esp)\b|\bds:', operands), operands
        lines += [f'L_{address}:', f'{operation} {operands}']
        instructions += 1
    assert instructions == 58, instructions
    lines += [
        'ldmxcsr dword ptr [rsp+40]', 'movups xmm6, xmmword ptr [rsp]',
        'movups xmm7, xmmword ptr [rsp+16]', 'add rsp, 56', 'ret', 'CellAxialKernel ENDP',
        'PUBLIC CellMxcsr', 'CellMxcsr PROC', 'stmxcsr dword ptr [rsp+8]',
        'mov eax, dword ptr [rsp+8]', 'ret', 'CellMxcsr ENDP', 'END',
    ]
    return '\n'.join(lines)+'\n', instructions


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('exe', type=Path)
    ap.add_argument('--sdk-bin', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    image = SourceImage(args.exe)
    assert ctypes.sizeof(ctypes.c_void_p) == 8, 'x64 Python is required'
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    args.out.mkdir(parents=True, exist_ok=False)
    output = args.out.resolve()
    listing = output/'source-axial-sdk.asm'
    subprocess.run([str(args.sdk_bin/'dumpbin.exe'), '/DISASM:NOBYTES',
        '/RANGE:0xE6F6D0,0xE6F868', f'/OUT:{listing}', str(args.exe)],
        check=True, capture_output=True)
    source, instructions = translated_kernel(listing.read_text())
    assembly = output/'axial-kernel-x64.asm'
    assembly.write_text(source)
    obj, dll = output/'axial-kernel.obj', output/'axial-kernel.dll'
    subprocess.run([str(args.sdk_bin/'ml64.exe'), '/nologo', '/c', f'/Fo{obj}', str(assembly)],
        check=True, capture_output=True)
    subprocess.run([str(args.sdk_bin/'link.exe'), '/NOLOGO', '/DLL', '/NOENTRY', '/MACHINE:X64',
        '/EXPORT:CellAxialKernel', '/EXPORT:CellMxcsr', f'/OUT:{dll}', str(obj)],
        check=True, capture_output=True)
    library = ctypes.CDLL(str(dll))
    point = ctypes.c_float*3
    basis = ctypes.c_float*9
    kernel = library.CellAxialKernel
    kernel.argtypes = [ctypes.POINTER(ctypes.c_float)]*4
    kernel.restype = None
    library.CellMxcsr.restype = ctypes.c_uint32
    source_basis = struct.unpack('<9f', image.read(0x15a8c28, 36))
    cases = []
    for flag in [0, 1, 2, 255]:
        for origin, target in [([1., 2., 3.], [11., 22., 33.]),
            ([1., 2., 3.], [11., 42., 33.]), ([1., 2., 3.], [11., 32., 33.]),
            ([-0., 0., -0.], [0., -0., -0.]), ([0., 0., 0.], [1.e-20, -2.e-20, 3.e-20])]:
            cases.append((origin, target, source_basis, flag))
    rng = random.Random(0xe6f707)
    for _ in range(4096):
        origin = [f32(rng.uniform(-100000, 100000)) for _ in range(3)]
        target = [f32(rng.uniform(-100000, 100000)) for _ in range(3)]
        axes = [f32(rng.uniform(-1.5, 1.5)) for _ in range(9)]
        cases.append((origin, target, axes, rng.choice([0, 1, 2, 255])))
    # Deliberate binary32 underflow/cancellation cases, under the declared
    # subnormal-preserving nearest policy. These are synthetic inputs.
    for tiny in [2.0**-126, 2.0**-140, 2.0**-149]:
        cases.append(([0., -0., 0.], [tiny, -tiny, tiny], [tiny]*9, 1))
    records = []
    before = library.CellMxcsr()
    for origin, target, axes, flag in cases:
        origin, target, axes = list(map(f32, origin)), list(map(f32, target)), list(map(f32, axes))
        actual = point()
        if flag:
            kernel(point(*origin), point(*target), basis(*axes), actual)
            expected = bytes(actual)
        else:
            expected = struct.pack('<3f', *target)
        records.append(struct.pack('<15fB', *origin, *target, *axes, flag)+expected)
    after = library.CellMxcsr()
    assert (before & 0xffc0) == (after & 0xffc0), 'kernel changed caller MXCSR control'
    fixture = output/'corpus.bin'
    fixture.write_bytes(b'RSCELLM1'+struct.pack('<I', len(records))+b''.join(records))
    report = dict(source_sha256=SOURCE_SHA256, source_range=['00E6F707', '00E6F82A'],
        decoded_instructions=instructions, cases=len(records), translated_kernel_cpu_execution=True,
        scope='Original bounded SSE operation/branch order, translated x64 ABI/memory operands; no game actor integrator or retail frame execution.',
        controlled_mxcsr='00001F80', caller_mxcsr_control_preserved=True,
        assembly_sha256=hashlib.sha256(assembly.read_bytes()).hexdigest(),
        dll_sha256=hashlib.sha256(dll.read_bytes()).hexdigest(),
        fixture_sha256=hashlib.sha256(fixture.read_bytes()).hexdigest())
    (output/'inventory.json').write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

"""Translate bounded E55E40 for explicit x87-policy CPU conformance.

The native scalar instruction/branch order comes from SDK disassembly. Runtime
environment/early-return vectors and floating-point control are explicit inputs.
Only ABI, stack layout, pointer operands and the separately verified field call
are replaced. No retail process or hardware-default control policy is assumed.
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


def translated_kernel(listing, constants):
    name = 'CellTurnKernel'
    lines = ['.code', f'PUBLIC {name}', f'{name} PROC FRAME', 'sub rsp, 88',
        '.allocstack 88', '.endprolog', 'fnstenv [rsp+32]', 'fninit',
        'fldcw word ptr [rcx+64]', 'stmxcsr dword ptr [rsp+64]',
        'mov dword ptr [rsp+68], 01F80h', 'ldmxcsr dword ptr [rsp+68]']
    special = {
        0xe55e40: ('movss', 'xmm0, dword ptr [rcx+36]'),
        0xe55e56: ('mov', 'r10d, dword ptr [rcx+52]'),
        0xe55e5c: ('mov', 'rax, rdx'),
        0xe55e62: ('mov', 'r11d, dword ptr [rcx+56]'),
        0xe55e6b: ('mov', 'r10d, dword ptr [rcx+60]'),
        0xe55e78: ('lea', 'rax, [rcx+24]'),
        0xe55e7c: ('lea', 'r10, [rcx]'),
        0xe55ef1: ('fld', 'dword ptr [rcx+36]'),
        0xe55efb: ('lea', 'rax, [rcx+36]'),
        0xe55f3a: ('lea', 'rax, [rcx+40]'),
        0xe55f4d: ('lea', 'rax, [rcx+12]'),
        0xe55f71: ('mov', 'rax, rdx'),
    }
    stack_memory = {0xe55f17: 4, 0xe55f2e: 8, 0xe55f34: 12,
        0xe55f51: 4, 0xe55f5b: 8, 0xe55f66: 12, 0xe55f75: 4,
        0xe55f7b: 4, 0xe55f81: 8, 0xe55f87: 8, 0xe55f8b: 12, 0xe55f91: 12}
    count = 0
    for line in listing.splitlines():
        match = re.match(r'\s*([0-9A-F]+): (\w+)\s*(.*)', line)
        if not match:
            continue
        address, operation, operands = match.groups()
        va = int(address, 16)
        if not 0xe55e40 <= va <= 0xe55fa1:
            continue
        if operation in {'push', 'add', 'sub'} or va in {0xe55f03, 0xe55f09, 0xe55f21}:
            continue
        if va in special:
            operation, operands = special[va]
        elif operation == 'ret':
            operation, operands = 'jmp', 'L_DONE'
        elif operation.startswith('j'):
            operands = f'L_{int(operands, 16):08X}'
        else:
            def global_memory(match):
                address = int(match[1], 16)
                assert address in {0x1485e24, 0x13ec4b0}, hex(address)
                constants.add(address)
                return f'[C_{address:08X}]'
            operands = re.sub(r'ds:\[([0-9A-F]+)h\]', global_memory, operands)

            def memory(match):
                body = match[1]
                if 'esp' in body:
                    if va in stack_memory:
                        return f'[rsp+{stack_memory[va]}]'
                    assert body == 'esp', (address, body)
                    return '[rsp]'
                mapping = {'eax': 'rax', 'ecx': 'r10', 'edx': 'r11'}
                return '['+re.sub(r'\b(eax|ecx|edx)\b', lambda m: mapping[m[1]], body)+']'
            operands = re.sub(r'\[([^\]]+)\]', memory, operands)
            operands = re.sub(r'\becx\b', 'r10d', operands)
            operands = re.sub(r'\bedx\b', 'r11d', operands)
            if operation == 'lea':
                operands = re.sub(r'^eax,', 'rax,', operands)
        assert not re.search(r'\b(?:esp|ecx|edx)\b|\bds:', operands), (address, operands)
        lines += [f'L_{address}:', f'{operation} {operands}']
        count += 1
    lines += ['L_DONE:', 'fldenv [rsp+32]', 'ldmxcsr dword ptr [rsp+64]',
        'add rsp, 88', 'ret', f'{name} ENDP', 'END']
    return lines, count


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('exe', type=Path)
    ap.add_argument('--sdk-bin', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    image = SourceImage(args.exe)
    assert ctypes.sizeof(ctypes.c_void_p) == 8, 'x64 Python required'
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    args.out.mkdir(parents=True, exist_ok=False)
    output = args.out.resolve()
    listing = output/'source-turn-sdk.asm'
    subprocess.run([str(args.sdk_bin/'dumpbin.exe'), '/DISASM:NOBYTES',
        '/RANGE:0xE55E40,0xE55FA2', f'/OUT:{listing}', str(args.exe)],
        check=True, capture_output=True)
    constants = set()
    lines, count = translated_kernel(listing.read_text(), constants)
    constants_source = ['.const']+[
        f'C_{a:08X} DWORD 0{struct.unpack("<I", image.read(a, 4))[0]:08X}h'
        for a in sorted(constants)]
    assert struct.unpack('<I', image.read(0x13ec4b0, 4))[0] == 0x322bcc77
    assembly = output/'turn-kernel-x64.asm'
    assembly.write_text('\n'.join(constants_source+lines)+'\n')
    obj, dll = output/'turn-kernel.obj', output/'turn-kernel.dll'
    for command in [
        [str(args.sdk_bin/'ml64.exe'), '/nologo', '/c', f'/Fo{obj}', str(assembly)],
        [str(args.sdk_bin/'link.exe'), '/NOLOGO', '/DLL', '/NOENTRY', '/MACHINE:X64',
            '/EXPORT:CellTurnKernel', f'/OUT:{dll}', str(obj)],
    ]:
        result = subprocess.run(command, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout+'\n'+result.stderr
    library = ctypes.CDLL(str(dll))
    kernel = library.CellTurnKernel
    kernel.argtypes = [ctypes.c_void_p]*2
    kernel.restype = None
    cases = []
    for target in [[0., 0., 0.], [1., 0., 0.], [.25, .5, .75], [3., 4., 0.],
        [-1., 2., -3.], [1.e-20, -1.e-20, 1.e-20]]:
        for step in [-2., 0., -0., 1.e-20, .5, 1., 5., 10000.]:
            cases.append([0., 0., 0., 1., 2., 3., *target, step, 3., 1., -3., -0., 1., -2.])
    for step in [0., -0.]:
        cases.append([float('nan')]*9+[step]+[float('nan')]*3+[-0., 1., -2.])
    rng = random.Random(0xe55e40)
    for _ in range(1024):
        size = 10.**rng.uniform(-10, 10)
        position = [f32(rng.uniform(-size, size)) for _ in range(3)]
        target = [f32(rng.uniform(-size, size)) for _ in range(3)]
        distance = sum((target[i]-position[i])**2 for i in range(3))**.5
        step = f32(distance*rng.uniform(-.5, 1.5))
        current = [f32(rng.uniform(-100., 100.)) for _ in range(3)]
        environment = [f32(rng.uniform(-100., 100.)) for _ in range(3)]
        cases.append([*position, *current, *target, step, *environment, 0., 0., 0.])
    for tiny in [2.**-126, 2.**-140, 2.**-149]:
        cases.append([0., -0., tiny, -0., tiny, -tiny, tiny, -tiny, 0., tiny, tiny, -tiny, 0., 0., 0., 0.])
    records = []
    per_policy = {}
    guard = bytes([0xa5])*64
    for precision_bits in [0, 0x200, 0x300]:
        for rounding_bits in [0, 0x400, 0x800, 0xc00]:
            control_word = 0x7f | precision_bits | rounding_bits
            for values in cases:
                parameters = ctypes.create_string_buffer(struct.pack('<16fH', *values, control_word))
                actual = ctypes.create_string_buffer(guard+bytes(12)+guard)
                kernel(parameters, ctypes.byref(actual, 64))
                assert actual.raw[:64] == guard and actual.raw[76:140] == guard
                records.append(parameters.raw[:66]+actual.raw[64:76])
            per_policy[f'{control_word:04X}'] = len(cases)
    fixture = output/'corpus.bin'
    fixture.write_bytes(b'RSCELLT1'+struct.pack('<I', len(records))+b''.join(records))
    report = dict(source_sha256=SOURCE_SHA256, source_range=['00E55E40', '00E55FA2'],
        translated_instructions=count, synthetic_cases=len(records), cases_per_explicit_control_word=per_policy,
        translated_kernel_cpu_execution=True, controlled_mxcsr='00001F80', native_control_word_established=False,
        scope='Complete finite E55E40 scalar order, translated ABI; sampled and early-return vectors explicit; no retail frame.',
        fixture_sha256=hashlib.sha256(fixture.read_bytes()).hexdigest(),
        assembly_sha256=hashlib.sha256(assembly.read_bytes()).hexdigest(),
        dll_sha256=hashlib.sha256(dll.read_bytes()).hexdigest())
    (output/'inventory.json').write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

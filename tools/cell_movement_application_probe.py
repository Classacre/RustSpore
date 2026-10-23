"""CPU conformance of original Cell E55FB0, with explicit disjoint buffers.

The SDK source listing supplies the instruction and branch order. Only x64
ABI/register widths, stack bookkeeping, and the environment-array base change.
The original game is not run. Source-derived output stays under ignored target/.
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
    lines = ['.code', 'PUBLIC CellApplyKernel', 'CellApplyKernel PROC FRAME',
        'sub rsp, 24', '.allocstack 24', '.endprolog',
        'stmxcsr dword ptr [rsp]', 'mov dword ptr [rsp+4], 01F80h',
        'ldmxcsr dword ptr [rsp+4]']
    # Incoming RCX: source-layout actor, RDX: independent delta, R8: parts array.
    # R10 replaces the source index pointer; R11D replaces its signed counter.
    omitted = {0xe55fb0, 0xe55fd0, 0xe55fdf, 0xe55fe5}
    count = 0
    for line in listing.splitlines():
        match = re.match(r'\s*([0-9A-F]+): (\w+)\s*(.*)', line)
        if not match:
            continue
        address, operation, operands = match.groups()
        va = int(address, 16)
        # REA's body has a deliberate noncontiguous alignment hole here.
        if not (0xe55fb0 <= va <= 0xe55ff3 or 0xe56000 <= va <= 0xe560dd):
            continue
        if operation == 'int' or operation in {'int3', 'push', 'pop'} or va in omitted:
            continue
        if operation == 'ret':
            operands = 'L_DONE'
            operation = 'jmp'
        elif operation.startswith('j'):
            target = int(operands, 16)
            assert 0xe55fb0 <= target <= 0xe560dd
            operands = f'L_{target:08X}'
        elif va == 0xe56084:
            assert operation == 'mov'
            operands = 'rax, rdx'
        else:
            assert operation in {'cmp', 'mov', 'xor', 'test', 'lea', 'movss',
                'movaps', 'mulss', 'addss', 'inc', 'add'}, (address, operation)

            def memory(match):
                body = match[1]
                if 'ebx' in body:
                    assert '4654h' in body
                    body = re.sub(r'\+0*4654h\b', '', body)
                    assert '4654h' not in body
                mapping = {'ecx': 'rcx', 'edx': 'rdx', 'esi': 'r10',
                    'ebx': 'r8', 'eax': 'rax'}
                body = re.sub(r'\b(ecx|edx|esi|ebx|eax)\b', lambda m: mapping[m[1]], body)
                assert 'esp' not in body, (address, body)
                return '['+body+']'
            operands = re.sub(r'\[([^\]]+)\]', memory, operands)
            operands = re.sub(r'\bedi\b', 'r11d', operands)
            if va == 0xe55fec:
                operands = operands.replace('esi,', 'r10,')
            if va == 0xe5605b:
                operands = operands.replace('esi,', 'r10,')
            if va == 0xe5602f:
                operands = operands.replace('eax,', 'rax,')
            assert not re.search(r'\b(?:esp|ebx|esi|edi)\b|\bds:', operands), (address, operands)
        lines += [f'L_{address}:', f'{operation} {operands}']
        count += 1
    lines += ['L_DONE:', 'ldmxcsr dword ptr [rsp]', 'add rsp, 24', 'ret',
        'CellApplyKernel ENDP', 'END']
    return '\n'.join(lines)+'\n', count


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('exe', type=Path)
    ap.add_argument('--sdk-bin', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    SourceImage(args.exe)  # Pins the immutable input SHA before disassembly.
    assert ctypes.sizeof(ctypes.c_void_p) == 8, 'x64 Python required'
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    args.out.mkdir(parents=True, exist_ok=False)
    output = args.out.resolve()
    listing = output/'source-apply-sdk.asm'
    subprocess.run([str(args.sdk_bin/'dumpbin.exe'), '/DISASM:NOBYTES',
        '/RANGE:0xE55FB0,0xE560DE', f'/OUT:{listing}', str(args.exe)],
        check=True, capture_output=True)
    source, count = translated_kernel(listing.read_text())
    assembly = output/'application-kernel-x64.asm'
    assembly.write_text(source)
    obj, dll = output/'application-kernel.obj', output/'application-kernel.dll'
    for command in [
        [str(args.sdk_bin/'ml64.exe'), '/nologo', '/c', f'/Fo{obj}', str(assembly)],
        [str(args.sdk_bin/'link.exe'), '/NOLOGO', '/DLL', '/NOENTRY', '/MACHINE:X64',
            '/EXPORT:CellApplyKernel', f'/OUT:{dll}', str(obj)],
    ]:
        result = subprocess.run(command, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout+'\n'+result.stderr
    library = ctypes.CDLL(str(dll))
    kernel = library.CellApplyKernel
    kernel.argtypes = [ctypes.c_void_p]*3
    kernel.restype = None
    cases = []
    for word in [0, 1, 2, 0xffffffff]:
        for part_count in [-(1 << 31), -1, 0, 1, 2, 4]:
            cases.append((word, part_count, [0, 1, 0, 0], -0.,
                [1., -1., 0.], [-0., 0., -0.], [[-0., 0., 0.], [0., -0., 0.]]))
    # Repeated small additions must not collapse to one scaled addition.
    cases.append((1, 3, [0, 1, 0], 1., [1., 2., -3.], [1., 2., 3.],
        [[16777216., 0., 0.], [10., 20., 30.]]))
    # Inactive malformed indices and unused NaN vectors do not get read.
    cases.append((0, 200, [-1], 1., [1., 2., 3.], [0., 0., 0.], [[float('nan')]*3]))
    cases.append((1, -1, [-1], 1., [1., 2., 3.], [0., 0., 0.], [[float('nan')]*3]))
    rng = random.Random(0xe55fb0)
    for _ in range(4096):
        parts = [[f32(rng.uniform(-100000, 100000)) for _ in range(3)] for _ in range(8)]
        indices = [rng.randrange(8) for _ in range(rng.randrange(2, 17))]
        cases.append((rng.choice([0, 1, 255, 0xffffffff]), len(indices), indices,
            f32(rng.uniform(-10, 10)), [f32(rng.uniform(-10000, 10000)) for _ in range(3)],
            [f32(rng.uniform(-100000, 100000)) for _ in range(3)], parts))
    for tiny in [2.**-126, 2.**-140, 2.**-149]:
        cases.append((1, 2, [0, 0], tiny, [tiny, -tiny, 1.], [0., -0., tiny], [[tiny]*3]))
    records = []
    for word, part_count, indices, scale, delta, actor_total, parts in cases:
        raw_actor = bytearray(0x254+4*len(indices))
        struct.pack_into('<3f', raw_actor, 0x90, *actor_total)
        struct.pack_into('<f', raw_actor, 0xbc, scale)
        # 24C is a distinct unknown actor word, not the 250 count.
        struct.pack_into('<I', raw_actor, 0x248, word)
        struct.pack_into('<i', raw_actor, 0x250, part_count)
        struct.pack_into(f'<{len(indices)}i', raw_actor, 0x254, *indices)
        actor = (ctypes.c_ubyte*len(raw_actor)).from_buffer_copy(raw_actor)
        delta_arg = (ctypes.c_float*3)(*delta)
        before_parts = struct.pack(f'<{3*len(parts)}f', *(v for p in parts for v in p))
        active = indices[:part_count] if word != 0 and part_count >= 2 else []
        assert not active or len(active) == part_count
        assert all(0 <= index < len(parts) for index in active)
        guard = bytes([0xa5])*64
        guarded_parts = guard+before_parts+guard
        part_arg = (ctypes.c_ubyte*len(guarded_parts)).from_buffer_copy(guarded_parts)
        kernel(actor, delta_arg, ctypes.byref(part_arg, len(guard)))
        assert bytes(part_arg)[:64] == guard and bytes(part_arg)[-64:] == guard
        record = struct.pack('<Iif6fI', word, part_count, scale, *delta, *actor_total, len(indices))
        record += struct.pack(f'<{len(indices)}iI', *indices, len(parts))+before_parts
        record += bytes(actor)[0x90:0x9c]+bytes(part_arg)[64:-64]
        records.append(record)
    fixture = output/'corpus.bin'
    fixture.write_bytes(b'RSCELLA1'+struct.pack('<I', len(records))+b''.join(records))
    report = dict(source_sha256=SOURCE_SHA256, source_range=['00E55FB0', '00E560DE'],
        translated_instructions=count, synthetic_cases=len(records),
        translated_kernel_cpu_execution=True, controlled_mxcsr='00001F80',
        scope='Complete E55FB0 finite scalar application, translated x64 ABI/environment base; no retail process or steering execution.',
        fixture_sha256=hashlib.sha256(fixture.read_bytes()).hexdigest(),
        assembly_sha256=hashlib.sha256(assembly.read_bytes()).hexdigest(),
        dll_sha256=hashlib.sha256(dll.read_bytes()).hexdigest())
    (output/'inventory.json').write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

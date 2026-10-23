"""Source-order CPU goldens for the bounded original Cell field sampler.

SDK instruction arithmetic is retained for coordinates, centered weights and
four-vector blending. ABI/memory operands are explicit; Python resolves only
the recovered row-major integer indices between these three bounded kernels.
No original game process is run. Generated source-derived files stay ignored.
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


def instructions(listing):
    result = []
    for line in listing.splitlines():
        match = re.match(r'\s*([0-9A-F]+): (\w+)\s*(.*)', line)
        if match:
            result.append((int(match[1], 16), match[2], match[3]))
    return result


def prologue(name):
    return [f'PUBLIC {name}', f'{name} PROC FRAME', 'sub rsp, 72', '.allocstack 72',
        'movups xmmword ptr [rsp+32], xmm6', '.savexmm128 xmm6, 32',
        'movups xmmword ptr [rsp+48], xmm7', '.savexmm128 xmm7, 48', '.endprolog',
        'stmxcsr dword ptr [rsp+24]', 'mov dword ptr [rsp+28], 01F80h',
        'ldmxcsr dword ptr [rsp+28]']


def epilogue(name):
    return [f'{name}_DONE:', 'ldmxcsr dword ptr [rsp+24]',
        'movups xmm6, xmmword ptr [rsp+32]', 'movups xmm7, xmmword ptr [rsp+48]',
        'add rsp, 72', 'ret', f'{name} ENDP']


def constants_memory(operands, constants):
    def reference(match):
        va = int(match[1], 16)
        constants.add(va)
        return f'[C_{va:08X}]'
    return re.sub(r'ds:\[([0-9A-F]+)h\]', reference, operands)


def coordinate_kernel(listing, constants):
    name = 'CellFlowCoordinate'
    lines = prologue(name)
    count = 0
    for va, operation, operands in instructions(listing):
        if not 0xe4db80 <= va <= 0xe4dc39 or operation in {'sub', 'add', 'ret'}:
            continue
        assert operation in {'movss', 'subss', 'mulss', 'xorps', 'maxss', 'minss'}
        replacements = {0x16b4c68: '[rdx]', 0x16b4c6c: '[rdx+4]',
            0x15a8b9c: '[r8]', 0x15a8ba0: '[r8+4]'}
        for address, memory in replacements.items():
            operands = operands.replace(f'ds:[{address:08X}h]', memory)
        operands = constants_memory(operands, constants)
        operands = re.sub(r'\beax\b', 'r9', operands)
        operands = re.sub(r'\becx\b', 'rcx', operands)
        operands = re.sub(r'\besp\b', 'rsp', operands)
        lines += [f'L_{va:08X}:', f'{operation} {operands}']
        count += 1
    return lines+epilogue(name), count


def weights_kernel(listing, constants):
    name = 'CellFlowWeights'
    lines = prologue(name)
    count = 0
    special = {
        0xe4dfd1: ('movss', 'xmm0, dword ptr [rcx]'),
        0xe4dfea: ('lea', 'r10, [rdx]'),
        0xe4dfee: ('movss', 'xmm0, dword ptr [rcx]'),
        0xe4dff4: ('lea', 'r11, [rdx+8]'),
        0xe4dff8: ('lea', 'r8, [rdx+12]'),
        0xe4e002: ('lea', 'rax, [rdx+4]'),
        0xe4e032: ('cmp', 'r10d, dword ptr [rcx+4]'),
        0xe4e059: ('mov', 'r9d, dword ptr [rcx+8]'),
    }
    for va, operation, operands in instructions(listing):
        if not 0xe4dfd0 <= va <= 0xe4e082 or operation in {'push', 'pop'}:
            continue
        if operation == 'ret':
            operation, operands = 'jmp', name+'_DONE'
        elif operation.startswith('j'):
            operands = f'L_{int(operands, 16):08X}'
        elif va in special:
            operation, operands = special[va]
        else:
            operands = constants_memory(operands, constants)

            def memory(match):
                mapping = {'eax': 'rax', 'ecx': 'r10', 'esi': 'r11', 'edx': 'r8'}
                return '['+re.sub(r'\b(eax|ecx|esi|edx)\b', lambda m: mapping[m[1]], match[1])+']'
            operands = re.sub(r'\[([^\]]+)\]', memory, operands)
            operands = re.sub(r'\becx\b', 'r10d', operands)
            operands = re.sub(r'\bedi\b', 'r9d', operands)
        assert not re.search(r'\b(?:esp|esi|edi|edx|ecx)\b|\bds:', operands), (hex(va), operands)
        lines += [f'L_{va:08X}:', f'{operation} {operands}']
        count += 1
    return lines+epilogue(name), count


def blend_kernel(listing):
    name = 'CellFlowBlend'
    lines = prologue(name)
    count = 0
    # Four row-major selected vectors at RCX; x0/x1/y0/y1 weights at RDX;
    # output at R8. Integer grid lookup happens between the native kernels.
    operands_at = {
        0xe4e2b3: '[rdx+8]', 0xe4e2b9: '[rdx]', 0xe4e2bf: '[rdx+4]', 0xe4e2c5: '[rdx+12]',
        0xe4e2d4: '[rcx]', 0xe4e2dd: '[rcx+4]', 0xe4e2e6: '[rcx+8]',
        0xe4e31c: '[rcx+12]', 0xe4e334: '[rcx+16]', 0xe4e341: '[rcx+20]',
        0xe4e361: '[rcx+24]', 0xe4e379: '[rcx+28]', 0xe4e386: '[rcx+32]',
        0xe4e39f: '[rcx+40]', 0xe4e3b0: '[rcx+44]', 0xe4e3b9: '[rcx+36]',
        0xe4e3ce: '[r8]', 0xe4e3d2: '[r8+4]', 0xe4e3d7: '[r8+8]',
    }
    for va, operation, operands in instructions(listing):
        if not 0xe4e2b3 <= va <= 0xe4e3d7 or operation not in {'movss', 'movaps', 'mulss', 'addss'}:
            continue
        if '[' in operands:
            assert va in operands_at, (hex(va), operands)
            operands = re.sub(r'\[[^\]]+\]', operands_at[va], operands)
        lines += [f'L_{va:08X}:', f'{operation} {operands}']
        count += 1
    return lines+epilogue(name), count


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
    listings = []
    for name, start, end in [('coordinate', 0xe4db80, 0xe4dc3a),
        ('weights', 0xe4dfd0, 0xe4e083), ('blend', 0xe4e220, 0xe4e3e1)]:
        listing = output/f'source-{name}-sdk.asm'
        subprocess.run([str(args.sdk_bin/'dumpbin.exe'), '/DISASM:NOBYTES',
            f'/RANGE:0x{start:X},0x{end:X}', f'/OUT:{listing}', str(args.exe)],
            check=True, capture_output=True)
        listings.append(listing.read_text())
    constants = set()
    coordinates, coordinate_count = coordinate_kernel(listings[0], constants)
    weights, weights_count = weights_kernel(listings[1], constants)
    blend, blend_count = blend_kernel(listings[2])
    const_lines = ['.const']+[
        f'C_{a:08X} DWORD 0{struct.unpack("<I", image.read(a, 4))[0]:08X}h'
        for a in sorted(constants)]
    assembly = output/'flow-kernels-x64.asm'
    assembly.write_text('\n'.join(const_lines+['.code']+coordinates+weights+blend+['END'])+'\n')
    obj, dll = output/'flow-kernels.obj', output/'flow-kernels.dll'
    for command in [
        [str(args.sdk_bin/'ml64.exe'), '/nologo', '/c', f'/Fo{obj}', str(assembly)],
        [str(args.sdk_bin/'link.exe'), '/NOLOGO', '/DLL', '/NOENTRY', '/MACHINE:X64',
            '/EXPORT:CellFlowCoordinate', '/EXPORT:CellFlowWeights', '/EXPORT:CellFlowBlend',
            f'/OUT:{dll}', str(obj)],
    ]:
        result = subprocess.run(command, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout+'\n'+result.stderr
    library = ctypes.CDLL(str(dll))
    for name, arg_count in [('CellFlowCoordinate', 4), ('CellFlowWeights', 2), ('CellFlowBlend', 3)]:
        kernel = getattr(library, name)
        kernel.argtypes = [ctypes.c_void_p]*arg_count
        kernel.restype = None
    coords = [-1., -0., 0., f32(.5-2.**-25), .5, f32(.5+2.**-24),
        1., 1.5, 14.5, 15., 19.5, 20., 21.]
    cases = [(p, [0., 0.], [1., 1.]) for p in ((x, y) for x in coords for y in coords)]
    rng = random.Random(0xe4e220)
    for _ in range(2048):
        origin = [f32(rng.uniform(-1000, 1000)) for _ in range(2)]
        scale = [f32(rng.choice([-1., 1.])*rng.uniform(.01, 10.)) for _ in range(2)]
        desired = [rng.uniform(-.25, 20.25), rng.uniform(-.25, 15.25)]
        point = [f32(origin[i]+desired[i]/scale[i]) for i in range(2)]
        cases.append((point, origin, scale))
    for tiny in [2.**-126, 2.**-140, 2.**-149]:
        cases.append(([tiny, -tiny], [0., -0.], [tiny, 1.]))
    records = []
    f2, f3, f4 = ctypes.c_float*2, ctypes.c_float*3, ctypes.c_float*4
    for ordinal, (point, origin, scale) in enumerate(cases):
        field = [f32(rng.uniform(-1000, 1000)) for _ in range(900)]
        if ordinal >= len(cases)-3:
            field = [f32(value*2.**-140) for value in field]
        actual_coordinate = f2()
        library.CellFlowCoordinate(f2(*point), f2(*origin), f2(*scale), actual_coordinate)
        axis_results = []
        for coordinate, extent in zip(actual_coordinate, [20, 15]):
            parameters = ctypes.create_string_buffer(struct.pack('<fii', coordinate, 0, extent))
            result = ctypes.create_string_buffer(16)
            library.CellFlowWeights(parameters, result)
            axis_results.append(struct.unpack('<iffi', result.raw))
        (ix, wx0, wx1, dx), (iy, wy0, wy1, dy) = axis_results
        selected = [ix+20*iy, ix+dx+20*iy, ix+20*(iy+dy), ix+dx+20*(iy+dy)]
        assert all(0 <= index < 300 for index in selected)
        corners = (ctypes.c_float*12)(*(field[index*3+axis] for index in selected for axis in range(3)))
        actual = f3()
        library.CellFlowBlend(corners, f4(wx0, wx1, wy0, wy1), actual)
        records.append(struct.pack('<906f', *point, *origin, *scale, *field)+bytes(actual))
    fixture = output/'corpus.bin'
    fixture.write_bytes(b'RSCELLF1'+struct.pack('<I', len(records))+b''.join(records))
    report = dict(source_sha256=SOURCE_SHA256,
        source_ranges=[['00E4DB80', '00E4DC3A'], ['00E4DFD0', '00E4E083'], ['00E4E2B3', '00E4E3DB']],
        translated_instructions=[coordinate_count, weights_count, blend_count],
        synthetic_cases=len(records), translated_kernel_cpu_execution=True, controlled_mxcsr='00001F80',
        scope='Source-order coordinate/weight/blend CPU kernels with explicit frame; host resolves original row-major integer indices; no retail process or field population.',
        fixture_sha256=hashlib.sha256(fixture.read_bytes()).hexdigest(),
        assembly_sha256=hashlib.sha256(assembly.read_bytes()).hexdigest(),
        dll_sha256=hashlib.sha256(dll.read_bytes()).hexdigest())
    (output/'inventory.json').write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

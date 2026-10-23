"""Prepare an allowlisted no-CRT x86 host for original bounded Cell math.

Copied executable bytes, native data, generated assembly, audit and host binary
are confined to ignored target/. Preparation does not execute the host. Its only
replacement routine is the explicit sampled-environment vector provider.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys

from cell_schema_probe import SourceImage, SOURCE_SHA256

CODE = [(0xe51230, 0xe51290), (0xe55650, 0xe55795), (0xe557a0, 0xe55b12),
    (0xe55b20, 0xe55e3d), (0xe55e40, 0xe55fa2), (0x1041e40, 0x1041e57)]
DATA = [0x13eb1bc, 0x13eb96c, 0x13ec4ac, 0x13ec4b0, 0x1471b2c, 0x1474728,
    0x1485c9c, 0x1485e24, 0x1485f70, 0x14861d0, 0x149a808]
INPUT_WORDS = [0x16b4be4, 0x16b4c08, 0x16b4c0c, 0x16b4c10]
STUB = 0xe4e220
IMPORTS = ['GetStdHandle', 'ReadFile', 'WriteFile', 'VirtualAlloc',
    'FlushInstructionCache', 'GetLastError', 'ExitProcess']


def audit_code(image):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent/'target/rea/capstone-runtime'))
    from capstone import Cs, CS_ARCH_X86, CS_MODE_32, CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    decoder = Cs(CS_ARCH_X86, CS_MODE_32)
    decoder.detail = True
    functions = []
    for start, end in CODE:
        decoded = list(decoder.disasm(image.read(start, end-start), start))
        assert sum(i.size for i in decoded) == end-start
        calls, absolute, relative = [], set(), set()
        for instruction in decoded:
            assert instruction.mnemonic not in {'int', 'sysenter', 'syscall', 'hlt', 'iret', 'iretd'}
            if instruction.group(CS_GRP_CALL) or instruction.group(CS_GRP_JUMP):
                assert len(instruction.operands) == 1 and instruction.operands[0].type == X86_OP_IMM, instruction.op_str
                target = instruction.operands[0].imm
                if instruction.group(CS_GRP_CALL):
                    assert target in {a for a, _ in CODE} | {STUB}, hex(target)
                    calls.append(dict(at=f'{instruction.address:08X}', target=f'{target:08X}'))
                else:
                    assert start <= target < end, (hex(instruction.address), hex(target))
            for operand in instruction.operands:
                if operand.type != X86_OP_MEM:
                    continue
                memory = operand.mem
                if memory.base == 0:
                    assert memory.index == 0
                    address = memory.disp & 0xffffffff
                    assert address in set(DATA) | set(INPUT_WORDS), hex(address)
                    absolute.add(f'{address:08X}')
                else:
                    relative.add(instruction.op_str)
        functions.append(dict(start=f'{start:08X}', end_exclusive=f'{end:08X}',
            bytes=end-start, sha256=hashlib.sha256(image.read(start, end-start)).hexdigest(),
            direct_calls=calls, indirect_calls_or_jumps=0, absolute_reads=sorted(absolute),
            register_relative_memory_operands=sorted(relative),
            disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in decoded]))
    return functions


def db(label, data):
    return [label+' LABEL BYTE']+[
        'DB '+','.join(f'0{value:02X}h' for value in data[at:at+16])
        for at in range(0, len(data), 16)]


def host_source(image):
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none',
        'EXTERN _GetStdHandle@4:PROC', 'EXTERN _ReadFile@20:PROC',
        'EXTERN _WriteFile@20:PROC', 'EXTERN _VirtualAlloc@16:PROC',
        'EXTERN _GetLastError@0:PROC',
        'EXTERN _FlushInstructionCache@12:PROC', 'EXTERN _ExitProcess@4:PROC', '.data',
        'InputBuffer BYTE 100 DUP(0)', 'ReplyBuffer BYTE 16 DUP(0)',
        'OutputBefore DWORD 4 DUP(0A5A5A5A5h)', 'OutputVector DWORD 3 DUP(0)',
        'OutputAfter DWORD 4 DUP(0A5A5A5A5h)', 'FakeRoot BYTE 0410Ch DUP(0)',
        'InputHandle DWORD 0', 'OutputHandle DWORD 0', 'Transferred DWORD 0',
        'SseControl DWORD 01F80h']
    for start, end in CODE:
        lines += db(f'Code_{start:08X}', image.read(start, end-start))
    for address in DATA:
        lines += db(f'Data_{address:08X}', image.read(address, 4))
    lines += ['.code', 'EnvironmentStub PROC', 'mov eax, dword ptr [esp+4]',
        'mov edx, dword ptr [InputBuffer+76]', 'mov dword ptr [eax], edx',
        'mov edx, dword ptr [InputBuffer+80]', 'mov dword ptr [eax+4], edx',
        'mov edx, dword ptr [InputBuffer+84]', 'mov dword ptr [eax+8], edx', 'ret',
        'EnvironmentStub ENDP', 'EnvironmentStubEnd LABEL BYTE',
        'ReadRecord PROC', 'push ebx', 'push esi', 'mov ebx, 100',
        'mov esi, OFFSET InputBuffer', 'ReadLoop:', 'mov Transferred,0', 'push 0', 'push OFFSET Transferred',
        'push ebx', 'push esi', 'push InputHandle', 'call _ReadFile@20', 'test eax,eax',
        'jnz ReadTransferred', 'call _GetLastError@0', 'cmp eax,109', 'jne HostFailure',
        'cmp ebx,100', 'jne HostFailure',
        'cmp Transferred,0', 'jne HostFailure', 'xor eax,eax', 'jmp ReadDone',
        'ReadTransferred:', 'mov eax, Transferred', 'test eax,eax', 'jnz ReadSome',
        'cmp ebx,100', 'jne HostFailure', 'xor eax,eax', 'jmp ReadDone', 'ReadSome:',
        'add esi,eax', 'sub ebx,eax', 'jnz ReadLoop', 'mov eax,1', 'ReadDone:',
        'pop esi', 'pop ebx', 'ret', 'ReadRecord ENDP', 'PUBLIC HostMain', 'HostMain PROC']
    code_pages = {start & ~0xffff for start, _ in CODE} | {STUB & ~0xffff}
    pages = code_pages | {a & ~0xffff for a in DATA+INPUT_WORDS}
    for page in sorted(pages):
        protection = '040h' if page in code_pages else '04h'
        lines += [f'push {protection}', 'push 03000h', 'push 010000h',
            f'push 0{page:08X}h', 'call _VirtualAlloc@16', f'cmp eax, 0{page:08X}h', 'jne HostFailure']
    lines += ['cld']
    for start, end in CODE:
        lines += [f'mov esi, OFFSET Code_{start:08X}', f'mov edi, 0{start:08X}h',
            f'mov ecx, {end-start}', 'rep movsb']
    for address in DATA:
        lines += [f'mov esi, OFFSET Data_{address:08X}', f'mov edi, 0{address:08X}h',
            'mov ecx,4', 'rep movsb']
    lines += ['mov esi, OFFSET EnvironmentStub', f'mov edi, 0{STUB:08X}h',
        'mov ecx, EnvironmentStubEnd-EnvironmentStub', 'rep movsb',
        'mov dword ptr [016B4BE4h], OFFSET FakeRoot',
        'push 0', 'push 0', 'push -1', 'call _FlushInstructionCache@12',
        'test eax,eax', 'jz HostFailure', 'push -10', 'call _GetStdHandle@4',
        'mov InputHandle,eax', 'push -11', 'call _GetStdHandle@4', 'mov OutputHandle,eax',
        'NextRecord:', 'call ReadRecord', 'test eax,eax', 'jz HostSuccess',
        'fninit', 'fldcw word ptr [InputBuffer+4]', 'ldmxcsr SseControl',
        'mov esi, OFFSET InputBuffer+88', 'mov edi,016B4C08h', 'mov ecx,3', 'rep movsd',
        'mov eax, dword ptr [InputBuffer]', 'cmp eax,0', 'je TurnCall',
        'cmp eax,1', 'je FlagellaCall', 'cmp eax,2', 'jne HostFailure',
        'mov eax,00E557A0h', 'jmp SwimCall', 'FlagellaCall:', 'mov eax,00E55B20h',
        'SwimCall:', 'push OFFSET OutputVector', 'push dword ptr [InputBuffer+72]',
        'push dword ptr [InputBuffer+68]', 'push dword ptr [InputBuffer+64]',
        'push dword ptr [InputBuffer+60]', 'push dword ptr [InputBuffer+56]',
        'push OFFSET InputBuffer+44', 'push OFFSET InputBuffer+32',
        'push OFFSET InputBuffer+20', 'push OFFSET InputBuffer+8', 'call eax',
        'add esp,40', 'movzx ebx,al', 'cmp dword ptr [InputBuffer],2', 'je GotOutput',
        'xor ebx,ebx', 'jmp GotOutput', 'TurnCall:', 'push OFFSET OutputVector',
        'push dword ptr [InputBuffer+68]', 'push dword ptr [InputBuffer+56]',
        'push OFFSET InputBuffer+44', 'push OFFSET InputBuffer+32', 'push OFFSET InputBuffer+20',
        'mov eax,00E55E40h', 'call eax', 'add esp,24', 'xor ebx,ebx', 'GotOutput:']
    for label in ['OutputBefore', 'OutputAfter']:
        for at in range(0, 16, 4):
            lines += [f'cmp dword ptr [{label}+{at}],0A5A5A5A5h', 'jne HostFailure']
    lines += ['mov esi, OFFSET OutputVector', 'mov edi, OFFSET ReplyBuffer', 'mov ecx,3',
        'rep movsd', 'mov dword ptr [ReplyBuffer+12],ebx', 'push 0', 'push OFFSET Transferred',
        'push 16', 'push OFFSET ReplyBuffer', 'push OutputHandle', 'call _WriteFile@20',
        'test eax,eax', 'jz HostFailure', 'cmp Transferred,16', 'jne HostFailure', 'jmp NextRecord',
        'HostSuccess:', 'push 0', 'call _ExitProcess@4', 'HostFailure::', 'push 7',
        'call _ExitProcess@4', 'HostMain ENDP', 'END HostMain']
    return '\n'.join(lines)+'\n', sorted(code_pages), sorted(pages)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('exe', type=Path)
    ap.add_argument('--sdk-bin-x86', type=Path, required=True)
    ap.add_argument('--kernel32-lib', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    image = SourceImage(args.exe)
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    args.out.mkdir(parents=True, exist_ok=False)
    output = args.out.resolve()
    audit = audit_code(image)
    source, code_pages, pages = host_source(image)
    assembly, obj, executable = output/'host-x86.asm', output/'host-x86.obj', output/'host-x86.exe'
    assembly.write_text(source)
    for command in [
        [str(args.sdk_bin_x86/'ml.exe'), '/nologo', '/c', '/coff', f'/Fo{obj}', str(assembly)],
        [str(args.sdk_bin_x86/'link.exe'), '/NOLOGO', '/MACHINE:X86', '/SUBSYSTEM:CONSOLE',
            '/ENTRY:HostMain', '/BASE:0x24000000', '/DYNAMICBASE:NO', '/SAFESEH:NO',
            f'/OUT:{executable}', str(obj), str(args.kernel32_lib)],
    ]:
        result = subprocess.run(command, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout+'\n'+result.stderr
    manifest = dict(source_sha256=SOURCE_SHA256, host_executed=False,
        host_image_base='24000000', imported_host_functions=IMPORTS,
        code_allowlist=audit,
        data_allowlist=[dict(address=f'{a:08X}', bytes=4, sha256=hashlib.sha256(image.read(a, 4)).hexdigest()) for a in DATA],
        explicit_input_words=[f'{a:08X}' for a in INPUT_WORDS],
        stub=dict(address=f'{STUB:08X}', behavior='Copy InputBuffer[76..88] to native output arg1; return arg1, cdecl.'),
        pages=[dict(address=f'{a:08X}', bytes=65536, protection='RWX' if a in code_pages else 'RW') for a in pages],
        protocol=dict(input_bytes=100, output_bytes=16,
            input_fields=['mode:u32@0', 'x87_cw:u16@4', 'reserved:u16@6', 'direction:f32x3@8',
                'position:f32x3@20', 'current:f32x3@32', 'target:f32x3@44', 'speed:f32@56',
                'gain6:f32@60', 'threshold7:f32@64', 'hint8:u32@68', 'gain9:f32@72',
                'sampled_environment:f32x3@76', 'zero_vector:f32x3@88'],
            modes={'0': 'E55E40', '1': 'E55B20', '2': 'E557A0'},
            output_fields=['delta:f32x3@0', 'jet_bool:u32@12 (zero in other modes)']),
        scoped_control_flow='All source direct calls are to allowlisted entries or stub; no source indirect call/jump. Branches stay within each allowlisted function. Register-relative operands retained in audit for pointer review.',
        host_source_sha256=hashlib.sha256(assembly.read_bytes()).hexdigest(),
        host_exe_sha256=hashlib.sha256(executable.read_bytes()).hexdigest())
    (output/'manifest.json').write_text(json.dumps(manifest, indent=2))
    print(json.dumps({key: manifest[key] for key in ['source_sha256', 'host_executed',
        'host_source_sha256', 'host_exe_sha256', 'imported_host_functions']}, indent=2))


if __name__ == '__main__':
    main()

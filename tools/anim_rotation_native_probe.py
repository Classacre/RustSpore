"""Prepare a bounded INFO+ROT100 source-curve host, never execute it.

Both original blend branches remain present. Immutable descriptors exclude POS
and deformation dispatch. Original source/CRT bytes and generated host remain
under target/. The checked memcpy fixture shim is separately audited.
"""
import argparse
import json
from pathlib import Path
import re
import struct
import subprocess

import anim_quaternion_probe as quat
import native_crt_acos_probe as crt
from cell_schema_probe import SourceImage, SOURCE_SHA256

ROOT = Path(__file__).resolve().parent.parent
CURVE = [(0x99F6A0, 0x99F8B6), (0x99FAF7, 0x9A0499), (0x9A0890, 0x9A0974),
         (0x99C6C0, 0x99C772), (0x99F630, 0x99F697)]
COPY = 0x11E0534
DATA = sorted(set(quat.CONSTANTS+[(0x13EB96C, 4), (0x13F4F98, 4),
    (0x14719E4, 4), (0x1471B2C, 4), (0x15514CC, 12),
    (0x15514F8, 4), (0x14477D8, 4), (0x1447818, 4)]))
INPUT_BYTES, OUTPUT_BYTES = 1952, 240


def audit_source(image):
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    calls_allowed = {0x99C6C0, 0x99F630, COPY} | {a for a, _ in quat.SPANS}
    branches, records = [], []
    boundaries = set()
    decoded = []
    for start, end in CURVE:
        code = list(crt.decoder().disasm(image.read(start, end-start), start))
        assert sum(i.size for i in code) == end-start
        boundaries.update(i.address for i in code)
        decoded.append(code)
    for (start, end), code in zip(CURVE, decoded):
        calls, jumps, memory = [], [], []
        for i in code:
            if i.group(CS_GRP_CALL) or i.group(CS_GRP_JUMP):
                assert i.operands[0].type == X86_OP_IMM
                target = i.operands[0].imm
                if i.group(CS_GRP_CALL):
                    assert target in calls_allowed, (hex(i.address), hex(target))
                    calls.append(dict(at=f'{i.address:08X}', target=f'{target:08X}'))
                elif target not in boundaries:
                    assert (i.address, target) in {(0x99F89E, 0x9A0499)}
                    jumps.append(dict(at=f'{i.address:08X}', target=f'{target:08X}',
                        reachable=False, proof='Immutable INFO0 skips enable dispatch; ROT02 skips enable dispatch; ROT12 lowtype2 cannot take preceding lowtype1 equality. Deformation fallthrough is also excluded.'))
                else:
                    jumps.append(dict(at=f'{i.address:08X}', target=f'{target:08X}', reachable=True))
            for value in i.operands:
                if value.type != X86_OP_MEM:
                    continue
                mem = value.mem
                displacement = mem.disp & 0xFFFFFFFF
                if not mem.base:
                    assert any(a <= displacement and displacement+value.size <= a+n for a, n in DATA)
                if displacement in {0x14477D8, 0x15514CC}:
                    assert i.address in {0x99F66B, 0x99F677}
                memory.append(i.op_str)
        records.append(dict(start=f'{start:08X}', end_exclusive=f'{end:08X}', bytes=end-start,
            sha256=quat.digest(image.read(start, end-start)), calls=calls, branches=jumps,
            memory_operands=sorted(set(memory)), indirect_calls_or_jumps=0,
            disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in code]))
    assert struct.unpack('<3I', image.read(0x15514CC, 12)) == (20, 80, 100)
    assert image.read(0x14477D8, 4) == struct.pack('<I', 0)
    assert image.read(0x1447818, 4) == struct.pack('<I', 100)
    assert image.read(0x15514F8, 4) == struct.pack('<I', 1)
    return records


def host_source(image, dll):
    code_pages = {a & ~0xFFFF for a, _ in CURVE+quat.SPANS+[quat.THUNK]} | {0x9A0000, COPY & ~0xFFFF, 0x785A0000}
    pages = sorted(code_pages | {a & ~0xFFFF for a, _ in DATA} | {0x13C0000, 0x78530000})
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none']
    for name, size in [('GetStdHandle', 4), ('ReadFile', 20), ('WriteFile', 20),
        ('VirtualAlloc', 16), ('VirtualProtect', 16), ('GetLastError', 0),
        ('FlushInstructionCache', 12), ('ExitProcess', 4)]:
        lines.append(f'EXTERN _{name}@{size}:PROC')
    lines += ['.const', 'DescriptorsOn DWORD 0,04F464E49h,0,120,0,0,0,0',
        'DWORD 012h,000544F52h,20,120,2,0,0,0',
        'DescriptorsOff DWORD 0,04F464E49h,0,120,0,0,0,0',
        'DWORD 002h,000544F52h,20,120,2,0,0,0']
    for a, b in CURVE+quat.SPANS+[quat.THUNK]:
        lines += crt.db(f'Source_{a:08X}', image.read(a, b-a))
    for a, n in DATA:
        lines += crt.db(f'Data_{a:08X}', image.read(a, n))
    for a, b in crt.CODE:
        lines += crt.db(f'Kernel_{a:08X}', dll.read(a, b-a))
    lines += crt.db('CrtConstants', dll.read(crt.WINDOW[0], crt.WINDOW[1]-crt.WINDOW[0]))
    lines += ['.data', 'InputBefore DWORD 4 DUP(0A5A5A5A5h)',
        f'InputBuffer BYTE {INPUT_BYTES} DUP(0)', 'InputAfter DWORD 4 DUP(0A5A5A5A5h)',
        'OutputBefore DWORD 4 DUP(0A5A5A5A5h)', 'OutputVector BYTE 200 DUP(0A5h)',
        'OutputAfter DWORD 4 DUP(0A5A5A5A5h)', 'ExpectedValue BYTE 228 DUP(0)',
        f'ReplyBuffer BYTE {OUTPUT_BYTES} DUP(0)', 'Channel BYTE 0E4h DUP(0)',
        'AnimHeader BYTE 0124h DUP(0)', 'InputHandle DWORD 0', 'OutputHandle DWORD 0',
        'Transferred DWORD 0', 'OldProtect DWORD 0', 'ActivePolicy DWORD 0FFFFFFFFh',
        'SseControl DWORD 01F80h', 'NativeStack DWORD 0', 'AlignedStack DWORD 0', 'UpperIndex DWORD 0',
        'ExactIndex DWORD 0', 'FrameScratch DWORD 0', 'Poison DWORD 0', '.code',
        'PUBLIC CopyStub, CopyStubEnd, ReadRecord, ReadRecordEnd, ValidateRecord, ValidateRecordEnd, RunOnce, RunOnceEnd, HostMain, HostEnd, HostFailure',
        # This exact memcpy shim is copied to the source CRT thunk address.
        # All its relative transfers stay internal; failure uses absolute EAX.
        'CopyStub PROC', 'push ebx', 'push esi', 'push edi',
        'mov ebx,dword ptr [esp+16]', 'mov esi,dword ptr [esp+20]',
        'cmp ebx,OFFSET OutputVector', 'je CopyInfo',
        'cmp ebx,OFFSET OutputVector+100', 'jne CopyFailure',
        'cmp dword ptr [esp+24],100', 'jne CopyFailure',
        'mov eax,esi', 'sub eax,OFFSET InputBuffer+52', 'jb CopyFailure', 'jmp CopyRow',
        'CopyInfo:', 'cmp dword ptr [esp+24],20', 'jne CopyFailure',
        'mov eax,esi', 'sub eax,OFFSET InputBuffer+32', 'jb CopyFailure', 'CopyRow:',
        'xor edx,edx', 'mov ecx,120', 'div ecx', 'test edx,edx', 'jnz CopyFailure',
        'cmp eax,dword ptr [InputBuffer+4]', 'jae CopyFailure',
        'mov edi,ebx', 'mov ecx,dword ptr [esp+24]', 'cld', 'rep movsb', 'mov eax,ebx',
        'pop edi', 'pop esi', 'pop ebx', 'ret',
        'CopyFailure:', 'mov eax,OFFSET HostFailure', 'jmp eax', 'CopyStub ENDP', 'CopyStubEnd LABEL BYTE',
        'ReadRecord PROC', 'push ebx', 'push esi', f'mov ebx,{INPUT_BYTES}', 'mov esi,OFFSET InputBuffer',
        'ReadLoop:', 'mov Transferred,0', 'push 0', 'push OFFSET Transferred', 'push ebx',
        'push esi', 'push InputHandle', 'call _ReadFile@20', 'test eax,eax', 'jnz ReadTransferred',
        'call _GetLastError@0', 'cmp eax,109', 'jne HostFailure', f'cmp ebx,{INPUT_BYTES}',
        'jne HostFailure', 'cmp Transferred,0', 'jne HostFailure', 'xor eax,eax', 'jmp ReadDone',
        'ReadTransferred:', 'mov eax,Transferred', 'test eax,eax', 'jnz ReadSome',
        f'cmp ebx,{INPUT_BYTES}', 'jne HostFailure', 'xor eax,eax', 'jmp ReadDone',
        'ReadSome:', 'add esi,eax', 'sub ebx,eax', 'jnz ReadLoop', 'mov eax,1',
        'ReadDone:', 'pop esi', 'pop ebx', 'ret', 'ReadRecord ENDP', 'ReadRecordEnd LABEL BYTE',
        'ValidateRecord PROC', 'cmp dword ptr [InputBuffer+4],16', 'ja HostFailure',
        'mov eax,dword ptr [InputBuffer+16]', 'mov edx,eax', 'and eax,0FFFFF0FFh',
        'cmp eax,07Fh', 'jne HostFailure', 'and edx,0300h', 'cmp edx,0100h', 'je HostFailure',
        'cmp dword ptr [InputBuffer+20],1', 'ja HostFailure',
        'cmp dword ptr [InputBuffer+24],1', 'ja HostFailure',
        'cmp dword ptr [InputBuffer+28],0', 'jne HostFailure',
        'mov eax,dword ptr [InputBuffer+12]', 'and eax,0FFFFFFF7h', 'jnz HostFailure',
        'mov eax,dword ptr [InputBuffer]', 'and eax,07FFFFFFFh', 'cmp eax,04B800000h', 'ja HostFailure',
        'mov eax,dword ptr [InputBuffer+8]', 'cmp eax,03F800000h', 'jb HostFailure',
        'cmp eax,04B800000h', 'ja HostFailure',
        'fninit', 'fldcw word ptr [InputBuffer+16]', 'ldmxcsr SseControl',
        'mov ecx,dword ptr [InputBuffer+4]', 'xor ebx,ebx', 'xor edi,edi',
        'mov esi,OFFSET InputBuffer+32', 'mov UpperIndex,0',
        'test ecx,ecx', 'jz ValidRows', 'ValidateRow:',
        'mov eax,dword ptr [esi]', 'cmp eax,01000000h', 'ja HostFailure',
        'cmp eax,ebx', 'jb HostFailure', 'mov ebx,eax',
        'fild dword ptr [esi]', 'fstp dword ptr [FrameScratch]',
        'movss xmm0,dword ptr [InputBuffer]', 'comiss xmm0,dword ptr [FrameScratch]',
        'jb UpperReady', 'inc UpperIndex', 'UpperReady:']
    for offset in [20, 24, 28, 32, 36, 104, 108]:
        lines += [f'mov eax,dword ptr [esi+{offset}]', 'and eax,07F800000h',
                  'cmp eax,07F800000h', 'je HostFailure']
    lines += ['add esi,120', 'dec ecx', 'jnz ValidateRow',
        # Source duration-1 must cover the final exact integer frame.
        'movss xmm0,dword ptr [InputBuffer+8]', 'subss xmm0,dword ptr [014861D0h]',
        'comiss xmm0,dword ptr [FrameScratch]', 'jb HostFailure', 'ValidRows:',
        'ret', 'ValidateRecord ENDP', 'ValidateRecordEnd LABEL BYTE',
        'RunOnce PROC', 'fninit', 'fldcw word ptr [InputBuffer+16]', 'ldmxcsr SseControl',
        'mov edi,OFFSET OutputVector', 'mov ecx,50', 'mov eax,0A5A5A5A5h', 'cld', 'rep stosd',
        'mov edi,OFFSET ReplyBuffer', 'mov ecx,60', 'mov eax,0FFFFFFFFh', 'rep stosd',
        'mov ExactIndex,0DEADBEEFh', 'mov eax,UpperIndex', 'mov dword ptr [ReplyBuffer+224],eax',
        'cmp dword ptr [InputBuffer+4],0', 'je NeighborsReady',
        'mov NativeStack,esp', 'push OFFSET ReplyBuffer+220', 'push OFFSET ReplyBuffer+212',
        'push OFFSET ReplyBuffer+208', 'push OFFSET ReplyBuffer+224', 'push OFFSET Channel',
        'mov eax,099C6C0h', 'call eax', 'add esp,20', 'cmp esp,NativeStack', 'jne HostFailure',
        'mov eax,dword ptr [ReplyBuffer+224]', 'mov dword ptr [ReplyBuffer+216],eax',
        'NeighborsReady:', 'cmp Poison,0', 'jne PoisonOnes']
    for i in range(8):
        lines += [f'pxor xmm{i},xmm{i}']
    lines += ['jmp ScratchReady', 'PoisonOnes:']
    for i in range(8):
        lines += [f'pcmpeqd xmm{i},xmm{i}']
    lines += ['ScratchReady:', 'mov NativeStack,esp', 'and esp,0FFFFFFF0h', 'mov AlignedStack,esp', 'sub esp,12',
        'push OFFSET ExactIndex', 'push OFFSET OutputVector', 'push OFFSET Channel',
        'push UpperIndex', 'push dword ptr [InputBuffer]', 'mov eax,099F6A0h', 'call eax',
        'add esp,32', 'cmp esp,AlignedStack', 'jne HostFailure', 'mov esp,NativeStack', 'movzx eax,al',
        'mov dword ptr [ReplyBuffer+204],eax', 'mov eax,ExactIndex',
        'mov dword ptr [ReplyBuffer+200],eax']
    for label in ['OutputBefore', 'OutputAfter', 'InputBefore', 'InputAfter']:
        for offset in range(0, 16, 4):
            lines += [f'cmp dword ptr [{label}+{offset}],0A5A5A5A5h', 'jne HostFailure']
    lines += ['stmxcsr dword ptr [ReplyBuffer+228]', 'fnstcw word ptr [ReplyBuffer+232]',
        'fnstsw word ptr [ReplyBuffer+234]', 'mov dword ptr [ReplyBuffer+236],0',
        'movzx eax,word ptr [ReplyBuffer+232]', 'cmp eax,dword ptr [InputBuffer+16]', 'jne HostFailure',
        'test word ptr [ReplyBuffer+234],03800h', 'jnz HostFailure',
        'mov eax,dword ptr [ReplyBuffer+228]', 'and eax,0FFFFFFC0h', 'cmp eax,01F80h', 'jne HostFailure',
        'mov esi,OFFSET OutputVector', 'mov edi,OFFSET ReplyBuffer', 'mov ecx,50', 'rep movsd',
        'ret', 'RunOnce ENDP', 'RunOnceEnd LABEL BYTE', 'HostMain PROC']
    for page in pages:
        lines += ['push 04h', 'push 03000h', 'push 010000h', f'push 0{page:08X}h',
                  'call _VirtualAlloc@16', f'cmp eax,0{page:08X}h', 'jne AllocationFailure']
        if page in code_pages:
            lines += [f'mov edi,0{page:08X}h', 'mov ecx,010000h', 'mov al,0CCh', 'cld', 'rep stosb']
    for a, b in CURVE+quat.SPANS+[quat.THUNK]:
        lines += [f'mov esi,OFFSET Source_{a:08X}', f'mov edi,0{a:08X}h', f'mov ecx,{b-a}', 'rep movsb']
    for a, n in DATA:
        lines += [f'mov esi,OFFSET Data_{a:08X}', f'mov edi,0{a:08X}h', f'mov ecx,{n}', 'rep movsb']
    for a, b in crt.CODE:
        lines += [f'mov esi,OFFSET Kernel_{a:08X}', f'mov edi,0{a:08X}h', f'mov ecx,{b-a}', 'rep movsb']
    lines += ['mov esi,OFFSET CrtConstants', f'mov edi,0{crt.WINDOW[0]:08X}h',
        f'mov ecx,{crt.WINDOW[1]-crt.WINDOW[0]}', 'rep movsb',
        'mov esi,OFFSET CopyStub', f'mov edi,0{COPY:08X}h', 'mov ecx,CopyStubEnd-CopyStub', 'rep movsb',
        f'mov dword ptr [0{quat.IAT:08X}h],0785A35E0h']
    for page in pages:
        if page == 0x1550000:
            continue  # First admitted request fixes policy, then makes it R.
        lines += ['push OFFSET OldProtect', f'push {"020h" if page in code_pages else "02h"}',
            'push 010000h', f'push 0{page:08X}h', 'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure']
    lines += ['push 0', 'push 0', 'push -1', 'call _FlushInstructionCache@12', 'test eax,eax', 'jz HostFailure',
        'push -10', 'call _GetStdHandle@4', 'mov InputHandle,eax',
        'push -11', 'call _GetStdHandle@4', 'mov OutputHandle,eax',
        'mov dword ptr [Channel+4],OFFSET AnimHeader', 'mov dword ptr [Channel+0D8h],OFFSET InputBuffer+32',
        'mov dword ptr [Channel+0DCh],2', 'NextRecord:', 'call ReadRecord', 'test eax,eax', 'jz HostSuccess',
        'call ValidateRecord', 'mov eax,dword ptr [InputBuffer+20]', 'cmp ActivePolicy,0FFFFFFFFh',
        'jne CheckPolicy', 'mov ActivePolicy,eax', 'mov dword ptr [015514F8h],eax',
        'push OFFSET OldProtect', 'push 02h', 'push 010000h', 'push 01550000h',
        'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure', 'jmp PolicyReady',
        'CheckPolicy:', 'cmp eax,ActivePolicy', 'jne PolicyFailure', 'PolicyReady:',
        'mov eax,dword ptr [InputBuffer+4]', 'mov dword ptr [Channel+0D4h],eax',
        'mov eax,dword ptr [InputBuffer+8]', 'mov dword ptr [AnimHeader+0120h],eax',
        'mov eax,dword ptr [InputBuffer+12]', 'mov dword ptr [AnimHeader+0118h],eax',
        'mov eax,OFFSET DescriptorsOn', 'cmp dword ptr [InputBuffer+24],0', 'jne DescriptorReady',
        'mov eax,OFFSET DescriptorsOff', 'DescriptorReady:', 'mov dword ptr [Channel+0E0h],eax',
        'mov Poison,0', 'call RunOnce', 'mov esi,OFFSET ReplyBuffer',
        'mov edi,OFFSET ExpectedValue', 'mov ecx,57', 'rep movsd', 'mov Poison,1', 'call RunOnce',
        'mov esi,OFFSET ReplyBuffer', 'mov edi,OFFSET ExpectedValue', 'mov ecx,57', 'repe cmpsd', 'jne HostFailure',
        'push 0', 'push OFFSET Transferred', f'push {OUTPUT_BYTES}', 'push OFFSET ReplyBuffer',
        'push OutputHandle', 'call _WriteFile@20', 'test eax,eax', 'jz HostFailure',
        f'cmp Transferred,{OUTPUT_BYTES}', 'jne HostFailure', 'jmp NextRecord',
        'HostSuccess:', 'push 0', 'call _ExitProcess@4', 'HostFailure::', 'push 7', 'call _ExitProcess@4',
        'AllocationFailure:', 'push 10', 'call _ExitProcess@4', 'PolicyFailure:', 'push 8', 'call _ExitProcess@4',
        'HostMain ENDP', 'HostEnd LABEL BYTE', 'END HostMain']
    return '\n'.join(lines)+'\n', pages, code_pages


def audit_host(exe, map_path):
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_REG, X86_REG_EAX
    wanted = {'CopyStub', 'CopyStubEnd', 'ReadRecord', 'ReadRecordEnd', 'ValidateRecord',
        'ValidateRecordEnd', 'RunOnce', 'RunOnceEnd', 'HostMain', 'HostEnd', 'HostFailure'}
    symbols = {}
    for line in map_path.read_text().splitlines():
        match = re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\w+)\s+([0-9A-Fa-f]{8})\b', line)
        if match and match[1] in wanted:
            symbols[match[1]] = int(match[2], 16)
    assert set(symbols) == wanted
    image = crt.PE(exe)
    imports = image.imports()
    assert {dll.upper() for dll, _ in imports.values()} == {'KERNEL32.DLL'}
    assert {name for _, name in imports.values()} == crt.IMPORTS
    start, end = symbols['CopyStub'], symbols['HostEnd']
    code = list(crt.decoder().disasm(image.read(start, end-start), start))
    assert sum(i.size for i in code) == end-start
    boundaries = {i.address for i in code}
    calls, branches = [], []
    for n, i in enumerate(code):
        assert i.mnemonic not in {'int', 'syscall', 'sysenter', 'hlt', 'iret', 'iretd'}
        if i.group(CS_GRP_JUMP):
            if i.operands[0].type == X86_OP_IMM:
                assert i.operands[0].imm in boundaries
                if i.address < symbols['CopyStubEnd']:
                    assert start <= i.operands[0].imm < symbols['CopyStubEnd']
                branches.append(dict(at=f'{i.address:08X}', target=f'{i.operands[0].imm:08X}'))
            else:
                assert i.address < symbols['CopyStubEnd'] and i.op_str == 'eax'
                assert code[n-1].mnemonic == 'mov' and code[n-1].operands[1].imm == symbols['HostFailure']
        if not i.group(CS_GRP_CALL):
            continue
        assert i.address >= symbols['CopyStubEnd']
        if i.operands[0].type == X86_OP_IMM:
            target = i.operands[0].imm
            if target in {symbols[k] for k in ['ReadRecord', 'ValidateRecord', 'RunOnce']}:
                calls.append(dict(at=f'{i.address:08X}', target=f'{target:08X}', kind='owned helper'))
            else:
                thunk = image.read(target, 6)
                assert thunk[:2] == b'\xFF\x25'
                dll, name = imports[struct.unpack('<I', thunk[2:])[0]]
                calls.append(dict(at=f'{i.address:08X}', dll=dll, name=name, kind='allowlisted host import'))
        else:
            assert i.operands[0].type == X86_OP_REG and i.operands[0].reg == X86_REG_EAX
            previous = code[n-1]
            assert previous.mnemonic == 'mov' and previous.operands[1].type == X86_OP_IMM
            target = previous.operands[1].imm
            assert target in {0x99F6A0, 0x99C6C0}
            calls.append(dict(at=f'{i.address:08X}', target=f'{target:08X}', kind='original source entry'))
    return dict(start=f'{start:08X}', end_exclusive=f'{end:08X}', sha256=quat.digest(image.read(start, end-start)),
        symbols={k: f'{v:08X}' for k, v in symbols.items()}, calls=calls, branches=branches,
        imports=[dict(iat=f'{a:08X}', dll=dll, name=name) for a, (dll, name) in imports.items()],
        copy_shim=dict(bytes=symbols['CopyStubEnd']-start, sha256=quat.digest(image.read(start, symbols['CopyStubEnd']-start)),
                       outbound_relative_transfers=0, failure='absolute EAX -> owned HostFailure'),
        disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in code])


def prepare(args):
    out = args.out.resolve()
    out.relative_to((ROOT/'target').resolve())
    image, dll = SourceImage(args.exe), crt.PE(args.dll)
    assert quat.digest(dll.data) == crt.SOURCE_SHA256
    crt.decoder()
    curve = audit_source(image)
    primitives, thunk = quat.audit_source(image)
    crt_code, bounds = crt.audit_source(dll)
    assert quat.digest(dll.read(crt.WINDOW[0], crt.WINDOW[1]-crt.WINDOW[0])) == crt.WINDOW_SHA256
    source, pages, code_pages = host_source(image, dll)
    out.mkdir(parents=True, exist_ok=False)
    asm, obj, exe, map_path = [out/name for name in ['host-x86.asm', 'host-x86.obj', 'host-x86.exe', 'host-x86.map']]
    asm.write_text(source)
    commands = [[str(args.sdk_bin_x86/'ml.exe'), '/nologo', '/c', '/coff', f'/Fo{obj}', str(asm)],
        [str(args.sdk_bin_x86/'link.exe'), '/NOLOGO', '/MACHINE:X86', '/SUBSYSTEM:CONSOLE',
         '/ENTRY:HostMain', '/BASE:0x24000000', '/DYNAMICBASE:NO', '/SAFESEH:NO',
         f'/OUT:{exe}', f'/MAP:{map_path}', str(obj), str(args.kernel32_lib)]]
    for command in commands:
        result = subprocess.run(command, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout+'\n'+result.stderr
    owned = audit_host(exe, map_path)
    manifest = dict(schema='bounded-original-info-rot100-curve-host-v1', host_executed=False,
        source_path=str(args.exe.resolve()), source_sha256=SOURCE_SHA256,
        crt_source_path=str(args.dll.resolve()), crt_source_sha256=crt.SOURCE_SHA256,
        source_curve_allowlist=curve, source_primitives=primitives, source_acos_thunk=thunk,
        source_data=[dict(address=f'{a:08X}', bytes=n, sha256=quat.digest(image.read(a, n))) for a, n in DATA],
        crt_code_allowlist=crt_code, crt_indexed_read_bound_proof=bounds, crt_window_sha256=crt.WINDOW_SHA256,
        descriptor_proof='Two immutable descriptor arrays: INFO0 slot0 offset0, ROT12 or ROT02 slot2 offset20, both stride120. They exclude lowtype1 POS and lowtype3 deformation paths. Endpoint helper can only copy INFO20->out0 or ROT100->out100 from checked aligned active rows.',
        selector_proof='Own upper selection admits sorted integer keys0..2^24, exact under all supported PCs/RCs. Compare finite frame against exact FILD/FST32 key values; upper counts keys<=frame. Count0..16. Source neighbors are called unchanged; frame before/after range uses endpoint copy. Interpolation has adjacent distinct integer times and t∈[-0,1]. Duration-1 must cover final key.',
        arithmetic_proof='Explicit masked PC24/53/64 x four RC, MXCSR1F80. Source quaternion finite inputs remain unnormalized. Ordered SSE dot overflow/unordered is safely clamped by original99CA1B/99CA47 to minus-one then abs-one; only finite abs-dot[-0,1] reaches reviewed CRT. All slerp fractions lie[-1,2], so FSIN magnitude<=pi<4. Native nonfinite results remain raw and bounded Rust errors must be separate exclusions.',
        memory_proof='Fixed stride120, count<=16, offsetsINFO0/ROT20 inside1952-byte input; rows sorted/time bounded and only actually used float fields are finite-guarded. Native output200 between separate16-byte canaries; checked before any context store. Reply/context is a separate buffer. Input canaries checked too. Copied shim validates every source/destination/length and row alignment/count before exact rep movsb.',
        blend_policy_proof='Each fresh host process accepts only one explicitly selected0/1 policy. First admitted request sets global015514F8 while context page is RW, then makes wholepage R before any source entry. All subsequent requests must match; policy changes exit8. Source-image initial1 is retained evidence, not assumed runtime state.',
        pages=[dict(address=f'{p:08X}', bytes=65536, initial='RW', final='RX' if p in code_pages else 'R',
                    finalization='before source entry;01550000 after first admitted policy') for p in pages],
        omitted_code='Code pages INT3-filled; unlisted POS/deformation/CRT dispatcher/domain/other code absent. Explicit checked memcpy shim at011E0534; unchanged acos thunk/IAT fixture binding to785A35E0. No original modules loaded.',
        host_code_audit=owned, host_source_sha256=quat.digest(asm.read_bytes()), host_exe_sha256=quat.digest(exe.read_bytes()),
        compile_commands=commands, protocol=dict(input_bytes=INPUT_BYTES, output_bytes=OUTPUT_BYTES,
            input='frame:f32@0,count:u32@4,duration:f32@8,flags:u32@12(0/8),CW:u32@16,blend:u32@20(0/1),enabled:u32@24(0/1),reserved0@28,16rowsINFO20+ROT100@32',
            output='native200@0,exactIndex@200,bool@204,prev/left/right/following@208..223,rightNeighbor@224,MXCSR@228,CW/SW@232/234,reserved0@236',
            per_record_checks='Two all-zero/all-one XMM native evaluations, first228bytes exact; input/output canaries, TOP0,CW unchanged,MXCSR1F80 controls, balanced aligned cdecl ESP.',
            exits='0 complete stream;7 domain/protocol/state/guard;8 attempted policy change;10 fixed-address allocation failure'),
        next_step='Root reviews fresh exact host/source/branches/guards/memory/imports/protocol and pending fixture inputs before separate native execution.')
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

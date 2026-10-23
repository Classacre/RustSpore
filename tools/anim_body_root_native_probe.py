"""Prepare, never execute, bounded unchanged native body root-selection spans.

Only fixed-capacity vector storage providers are owned replacements. Original
root votes, candidate order, policy branches and SSE comparisons are unchanged.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

from cell_schema_probe import SourceImage, SOURCE_SHA256
from cell_native_math_probe import db
from native_crt_acos_probe import PE, decoder, IMPORTS

ROOT = Path(__file__).resolve().parent.parent
CODE = [(0x9C32A0, 0x9C3355), (0x9C3857, 0x9C3A3B), (0x9C3A59, 0x9C3D0A)]
CONSTANTS = [(0x1447D08, 4), (0x13F5174, 4)]
GLOBALS = [0x166CFE4, 0x1551A84]
STUBS = {0x9C3355: 'VotesDone', 0x9C3D0A: 'RootDone',
         0xA2F530: 'ClearCandidates', 0x9C15F0: 'AppendCandidate',
         0xA2AF30: 'FillCounts', 0x8F3550: 'HostFailure',
         0xF47290: 'HostFailure', 0x11E0534: 'HostFailure'}
INPUT_BYTES, PROJECTION_BYTES, OUTPUT_BYTES = 8176, 2056, 2064
BLOCK_BYTES, FRAME_BYTES = 255*0x468, 0x1680
BUFFERS = [('Input', INPUT_BYTES), ('Definition', 1024), ('Blocks', BLOCK_BYTES),
           ('Counts', 1020), ('Candidates', 1020), ('Frame', FRAME_BYTES),
           ('Reply', OUTPUT_BYTES)]


def audit_source(image):
    sys.path.insert(0, str(ROOT/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    decoded = [list(decoder().disasm(image.read(a, b-a), a)) for a, b in CODE]
    boundaries = {i.address for part in decoded for i in part}
    result = []
    for (start, end), rows in zip(CODE, decoded):
        assert sum(i.size for i in rows) == end-start
        transfers, reads = [], []
        for i in rows:
            if i.group(CS_GRP_CALL) or i.group(CS_GRP_JUMP):
                assert i.operands[0].type == X86_OP_IMM
                target = i.operands[0].imm
                assert target in boundaries or target in STUBS
                transfers.append(dict(at=f'{i.address:08X}', operation=i.mnemonic,
                                      target=f'{target:08X}'))
            for op in i.operands:
                if op.type == X86_OP_MEM and not op.mem.base:
                    assert not op.mem.index and op.mem.disp in GLOBALS+[a for a, _ in CONSTANTS]
                    reads.append(dict(at=f'{i.address:08X}', address=f'{op.mem.disp:08X}'))
        result.append(dict(start=f'{start:08X}', end_exclusive=f'{end:08X}', bytes=end-start,
            sha256=hashlib.sha256(image.read(start, end-start)).hexdigest(),
            transfers=transfers, absolute_reads=reads,
            disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in rows]))
    assert image.read(0x1447D08, 4) == struct.pack('<I', 0x7F7FFFFF)
    assert image.read(0x13F5174, 4) == struct.pack('<I', 0xFF7FFFFF)
    return result


def host_source(image):
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none']
    for name, size in [('GetStdHandle', 4), ('ReadFile', 20), ('WriteFile', 20),
                       ('VirtualAlloc', 16), ('VirtualProtect', 16), ('GetLastError', 0),
                       ('FlushInstructionCache', 12), ('ExitProcess', 4)]:
        lines += [f'EXTERN _{name}@{size}:PROC']
    lines += ['.const']
    for a, b in CODE:
        lines += db(f'Original_{a:08X}', image.read(a, b-a))
    for a, size in CONSTANTS:
        lines += db(f'Constant_{a:08X}', image.read(a, size))
    lines += ['.data']
    for label, size in BUFFERS:
        lines += [f'{label}Before DWORD 4 DUP(0A5A5A5A5h)',
                  f'{label}Buffer BYTE {size} DUP(0)', f'{label}After DWORD 4 DUP(0A5A5A5A5h)']
    lines += [f'InputShadow BYTE {INPUT_BYTES} DUP(0)', 'DefinitionShadow BYTE 1024 DUP(0)',
        f'BlocksShadow BYTE {BLOCK_BYTES} DUP(0)', f'Expected BYTE {PROJECTION_BYTES} DUP(0)',
        'InputHandle DWORD 0', 'OutputHandle DWORD 0', 'Transferred DWORD 0',
        'OldProtect DWORD 0', 'NativeStack DWORD 0', 'Poison DWORD 0',
        'MaximumVotes DWORD 0', 'SseControl DWORD 01F80h', '.code',
        'PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,ReadRecord,ValidateRecord,RunOnce',
        'PUBLIC VotesDone,RootDone,ClearCandidates,AppendCandidate,FillCounts',
        'PUBLIC InputBuffer,DefinitionBuffer,BlocksBuffer,CountsBuffer,CandidatesBuffer,FrameBuffer,ReplyBuffer',
        'OwnedStart LABEL BYTE']
    for at, label in STUBS.items():
        lines += [f'PUBLIC Stub_{at:08X}', f'Stub_{at:08X} LABEL BYTE',
                  f'mov eax,OFFSET {label}', 'jmp eax']
    lines += ['ClearCandidates PROC', 'cmp dword ptr [esp],009C331Ah', 'jne HostFailure',
        'cmp ecx,OFFSET FrameBuffer+0100h+04ECh', 'jne HostFailure',
        'cmp dword ptr [esp+4],0', 'jne HostFailure',
        'mov eax,OFFSET CandidatesBuffer', 'cmp dword ptr [ecx],eax', 'jne HostFailure',
        'mov dword ptr [ecx+4],eax', 'ret 4', 'ClearCandidates ENDP',
        'AppendCandidate PROC', 'cmp dword ptr [esp],009C3334h', 'jne HostFailure',
        'cmp ecx,OFFSET FrameBuffer+0100h+04ECh', 'jne HostFailure',
        'cmp dword ptr [esp+4],OFFSET FrameBuffer+0100h+014h', 'jne HostFailure',
        'mov eax,dword ptr [ecx+4]', 'cmp eax,OFFSET CandidatesBuffer', 'jb HostFailure',
        'cmp eax,OFFSET CandidatesBuffer+1020', 'jae HostFailure',
        'mov edx,dword ptr [FrameBuffer+0100h+014h]',
        'cmp edx,dword ptr [InputBuffer+4]', 'jae HostFailure', 'mov dword ptr [eax],edx',
        'add eax,4', 'mov dword ptr [ecx+4],eax', 'ret 4', 'AppendCandidate ENDP',
        'FillCounts PROC', 'cmp dword ptr [esp],009C3BBEh', 'jne HostFailure',
        'cmp ecx,OFFSET FrameBuffer+0100h+0900h', 'jne HostFailure',
        'cmp dword ptr [esp+4],OFFSET FrameBuffer+0100h+0918h', 'jne HostFailure',
        'cmp dword ptr [esp+12],OFFSET FrameBuffer+0100h+02Ch', 'jne HostFailure',
        'cmp dword ptr [FrameBuffer+0100h+02Ch],0', 'jne HostFailure',
        'mov edx,dword ptr [esp+8]', 'cmp edx,dword ptr [InputBuffer+4]', 'jne HostFailure',
        'mov eax,dword ptr [esp+4]', 'FillLoop:', 'mov dword ptr [eax],0', 'add eax,4',
        'dec edx', 'jnz FillLoop', 'mov dword ptr [ecx+4],eax', 'ret 12', 'FillCounts ENDP',
        'ReadRecord PROC', 'push ebx', 'push esi', f'mov ebx,{INPUT_BYTES}',
        'mov esi,OFFSET InputBuffer', 'ReadLoop:', 'mov Transferred,0',
        'push 0', 'push OFFSET Transferred', 'push ebx', 'push esi', 'push InputHandle',
        'call _ReadFile@20', 'test eax,eax', 'jnz ReadTransferred',
        'call _GetLastError@0', 'cmp eax,109', 'jne HostFailure',
        f'cmp ebx,{INPUT_BYTES}', 'jne HostFailure', 'cmp Transferred,0', 'jne HostFailure',
        'xor eax,eax', 'jmp ReadDone', 'ReadTransferred:', 'mov eax,Transferred',
        'cmp eax,ebx', 'ja HostFailure', 'test eax,eax', 'jnz ReadSome',
        f'cmp ebx,{INPUT_BYTES}', 'jne HostFailure', 'xor eax,eax', 'jmp ReadDone',
        'ReadSome:', 'add esi,eax', 'sub ebx,eax', 'jnz ReadLoop', 'mov eax,1',
        'ReadDone:', 'pop esi', 'pop ebx', 'ret', 'ReadRecord ENDP',
        'ValidateRecord PROC', 'movzx eax,word ptr [InputBuffer]', 'mov edx,eax',
        'and eax,0F0FFh', 'cmp eax,07Fh', 'jne HostFailure',
        'and edx,0300h', 'cmp edx,0100h', 'je HostFailure',
        'cmp word ptr [InputBuffer+2],0', 'jne HostFailure',
        'mov ecx,dword ptr [InputBuffer+4]', 'test ecx,ecx', 'jz HostFailure',
        'cmp ecx,255', 'ja HostFailure', 'cmp dword ptr [InputBuffer+8],3', 'ja HostFailure',
        'cmp dword ptr [InputBuffer+12],0', 'jne HostFailure',
        'cmp dword ptr [InputBuffer+16],-1', 'jne HostFailure',
        'test byte ptr [InputBuffer+24],8', 'jz HostFailure',
        'xor ebx,ebx', 'mov esi,OFFSET InputBuffer+16', 'ValidateEdges:',
        'mov eax,dword ptr [esi]', 'cmp eax,-1', 'jne HasParent',
        'test ebx,ebx', 'jnz HostFailure', 'jmp ParentReady', 'HasParent:',
        'cmp eax,ecx', 'jae HostFailure', 'ParentReady:',
        'mov eax,dword ptr [esi+28]', 'cmp eax,-1', 'je SymmetryReady',
        'cmp eax,ecx', 'jae HostFailure', 'SymmetryReady:']
    for offset in [12, 16, 20]:
        lines += [f'mov eax,dword ptr [esi+{offset}]', 'and eax,07F800000h',
                  'cmp eax,07F800000h', 'je HostFailure']
    lines += ['inc ebx', 'add esi,32', 'cmp ebx,ecx', 'jb ValidateEdges',
        'xor ebx,ebx', 'ValidateChains:', 'mov eax,ebx', 'xor edx,edx', 'ChainLoop:',
        'cmp eax,-1', 'je ChainDone', 'cmp edx,ecx', 'jae HostFailure',
        'shl eax,5', 'mov eax,dword ptr [InputBuffer+16+eax]', 'inc edx', 'jmp ChainLoop',
        'ChainDone:', 'mov eax,ebx', 'shl eax,5', 'test byte ptr [InputBuffer+24+eax],4',
        'jz FootSafe', 'mov eax,dword ptr [InputBuffer+16+eax]', 'FootAncestor:',
        'cmp eax,-1', 'je HostFailure', 'mov edx,eax', 'shl edx,5',
        'test byte ptr [InputBuffer+24+edx],8', 'jnz FootSafe',
        'mov eax,dword ptr [InputBuffer+16+edx]', 'jmp FootAncestor', 'FootSafe:',
        'inc ebx', 'cmp ebx,ecx', 'jb ValidateChains', 'ret', 'ValidateRecord ENDP',
        'RunOnce PROC', 'mov NativeStack,esp']
    for label, size in BUFFERS:
        if label != 'Input':
            lines += [f'mov edi,OFFSET {label}Buffer', 'xor eax,eax', f'mov ecx,{size//4}',
                      'cld', 'rep stosd']
    lines += ['mov esi,OFFSET InputBuffer+16', 'mov edi,OFFSET BlocksBuffer',
        'mov ecx,dword ptr [InputBuffer+4]', 'BuildBlocks:', 'mov eax,dword ptr [esi]',
        'mov dword ptr [edi+01FCh],eax', 'mov eax,dword ptr [esi+4]',
        'mov dword ptr [edi+0150h],eax', 'mov eax,dword ptr [esi+8]',
        'mov dword ptr [edi+0154h],eax']
    for a, b in [(12, 0x10C), (16, 0x110), (20, 0x114), (24, 0x21C), (28, 0x200)]:
        lines += [f'mov eax,dword ptr [esi+{a}]', f'mov dword ptr [edi+0{b:X}h],eax']
    lines += ['add esi,32', 'add edi,0468h', 'dec ecx', 'jnz BuildBlocks',
        'mov dword ptr [DefinitionBuffer+0384h],OFFSET BlocksBuffer',
        'mov dword ptr [DefinitionBuffer+0388h],edi',
        'mov dword ptr [FrameBuffer+0100h+01540h],OFFSET DefinitionBuffer',
        'mov dword ptr [FrameBuffer+0100h+028h],-1',
        'mov dword ptr [FrameBuffer+0100h+01128h],OFFSET CountsBuffer',
        'mov dword ptr [FrameBuffer+0100h+01138h],OFFSET CountsBuffer',
        'mov dword ptr [FrameBuffer+0100h+04ECh],OFFSET CandidatesBuffer',
        'mov dword ptr [FrameBuffer+0100h+04F0h],OFFSET CandidatesBuffer',
        'mov dword ptr [FrameBuffer+0100h+04F4h],OFFSET CandidatesBuffer+1020',
        'mov dword ptr [FrameBuffer+0100h+04FCh],OFFSET CandidatesBuffer']
    for label, size in [('Input', INPUT_BYTES), ('Definition', 1024), ('Blocks', BLOCK_BYTES)]:
        lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                  f'mov ecx,{size//4}', 'cld', 'rep movsd']
    lines += ['fninit', 'fldcw word ptr [InputBuffer]', 'ldmxcsr SseControl',
        'cmp Poison,0', 'jne ScratchOnes']
    for n in range(8):
        lines += [f'pxor xmm{n},xmm{n}']
    lines += ['jmp ScratchReady', 'ScratchOnes:']
    for n in range(8):
        lines += [f'pcmpeqd xmm{n},xmm{n}']
    lines += ['ScratchReady:', 'mov ebx,OFFSET DefinitionBuffer', 'mov esi,dword ptr [InputBuffer+4]',
        'xor ebp,ebp', 'xor edi,edi', 'mov esp,OFFSET FrameBuffer+0100h',
        'mov eax,009C32A0h', 'jmp eax', 'VotesDone::', 'mov MaximumVotes,ebp',
        'mov eax,009C3857h', 'jmp eax', 'RootDone::',
        'cmp esp,OFFSET FrameBuffer+0100h', 'jne HostFailure', 'mov esp,NativeStack']
    for label, _ in BUFFERS:
        for side in ['Before', 'After']:
            for at in range(0, 16, 4):
                lines += [f'cmp dword ptr [{label}{side}+{at}],0A5A5A5A5h', 'jne HostFailure']
    for label, size in [('Input', INPUT_BYTES), ('Blocks', BLOCK_BYTES)]:
        lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                  f'mov ecx,{size//4}', 'cld', 'repe cmpsd', 'jne HostFailure']
    lines += ['xor ecx,ecx', 'CheckDefinition:', 'cmp ecx,03F4h', 'je NextDefinitionByte',
        'mov al,byte ptr [DefinitionBuffer+ecx]', 'cmp al,byte ptr [DefinitionShadow+ecx]',
        'jne HostFailure', 'NextDefinitionByte:', 'inc ecx', 'cmp ecx,1024', 'jb CheckDefinition',
        'cmp byte ptr [DefinitionBuffer+03F4h],1', 'ja HostFailure',
        'fnstcw word ptr [ReplyBuffer+2060]', 'fnstsw word ptr [ReplyBuffer+2062]',
        'stmxcsr dword ptr [ReplyBuffer+2056]', 'mov ax,word ptr [InputBuffer]',
        'cmp ax,word ptr [ReplyBuffer+2060]', 'jne HostFailure',
        'cmp word ptr [ReplyBuffer+2062],0', 'jne HostFailure',
        'mov eax,dword ptr [ReplyBuffer+2056]', 'and eax,0FFFFFFC0h', 'cmp eax,01F80h', 'jne HostFailure',
        'mov eax,dword ptr [FrameBuffer+0100h+010h]', 'mov dword ptr [ReplyBuffer],eax',
        'movzx eax,byte ptr [DefinitionBuffer+03F4h]', 'mov dword ptr [ReplyBuffer+4],eax',
        'mov eax,dword ptr [FrameBuffer+0100h+04F0h]', 'sub eax,OFFSET CandidatesBuffer',
        'cmp eax,1020', 'ja HostFailure', 'test eax,3', 'jnz HostFailure', 'shr eax,2',
        'mov dword ptr [ReplyBuffer+8],eax', 'mov ecx,eax',
        'mov esi,OFFSET CandidatesBuffer', 'mov edi,OFFSET ReplyBuffer+1036', 'cld', 'rep movsd',
        'mov eax,MaximumVotes', 'mov dword ptr [ReplyBuffer+12],eax',
        'mov esi,OFFSET CountsBuffer', 'mov edi,OFFSET ReplyBuffer+16', 'mov ecx,255', 'rep movsd',
        'ret', 'RunOnce ENDP', 'HostMain PROC']
    code_pages = {a & ~0xFFFF for a, _ in CODE} | {a & ~0xFFFF for a in STUBS}
    data_pages = {a & ~0xFFFF for a, _ in CONSTANTS} | {a & ~0xFFFF for a in GLOBALS}
    for page in sorted(code_pages | data_pages):
        lines += ['push 04h', 'push 03000h', 'push 010000h', f'push 0{page:08X}h',
                  'call _VirtualAlloc@16', f'cmp eax,0{page:08X}h', 'jne AllocationFailure']
        if page in code_pages:
            lines += [f'mov edi,0{page:08X}h', 'mov ecx,010000h', 'mov al,0CCh', 'cld', 'rep stosb']
    for a, b in CODE:
        lines += [f'mov esi,OFFSET Original_{a:08X}', f'mov edi,0{a:08X}h', f'mov ecx,{b-a}', 'rep movsb']
    for a in STUBS:
        lines += [f'mov esi,OFFSET Stub_{a:08X}', f'mov edi,0{a:08X}h', 'mov ecx,7', 'rep movsb']
    for a, size in CONSTANTS:
        lines += [f'mov esi,OFFSET Constant_{a:08X}', f'mov edi,0{a:08X}h', f'mov ecx,{size}', 'rep movsb']
    for page in sorted(code_pages | data_pages):
        lines += ['push OFFSET OldProtect', 'push 020h' if page in code_pages else 'push 02h',
                  'push 010000h', f'push 0{page:08X}h', 'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure']
    lines += ['push 0', 'push 0', 'push -1', 'call _FlushInstructionCache@12',
        'test eax,eax', 'jz HostFailure', 'push -10', 'call _GetStdHandle@4', 'mov InputHandle,eax',
        'push -11', 'call _GetStdHandle@4', 'mov OutputHandle,eax', 'NextRecord:',
        'call ReadRecord', 'test eax,eax', 'jz HostSuccess', 'call ValidateRecord']
    for page in sorted({a & ~0xFFFF for a in GLOBALS}):
        lines += ['push OFFSET OldProtect', 'push 04h', 'push 010000h', f'push 0{page:08X}h',
                  'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure']
    lines += ['mov eax,dword ptr [InputBuffer+8]', 'and eax,1', 'mov dword ptr [0166CFE4h],eax',
        'mov eax,dword ptr [InputBuffer+8]', 'shr eax,1', 'mov dword ptr [01551A84h],eax']
    for page in sorted({a & ~0xFFFF for a in GLOBALS}):
        lines += ['push OFFSET OldProtect', 'push 02h', 'push 010000h', f'push 0{page:08X}h',
                  'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure']
    lines += ['mov Poison,0', 'call RunOnce', 'mov esi,OFFSET ReplyBuffer', 'mov edi,OFFSET Expected',
        f'mov ecx,{PROJECTION_BYTES//4}', 'cld', 'rep movsd', 'mov Poison,1', 'call RunOnce',
        'mov esi,OFFSET ReplyBuffer', 'mov edi,OFFSET Expected', f'mov ecx,{PROJECTION_BYTES//4}',
        'repe cmpsd', 'jne HostFailure', 'push 0', 'push OFFSET Transferred', f'push {OUTPUT_BYTES}',
        'push OFFSET ReplyBuffer', 'push OutputHandle', 'call _WriteFile@20', 'test eax,eax', 'jz HostFailure',
        f'cmp Transferred,{OUTPUT_BYTES}', 'jne HostFailure', 'jmp NextRecord',
        'HostSuccess:', 'push 0', 'call _ExitProcess@4', 'HostFailure::', 'push 7', 'call _ExitProcess@4',
        'AllocationFailure:', 'push 10', 'call _ExitProcess@4', 'HostMain ENDP', 'OwnedEnd LABEL BYTE', 'END HostMain']
    return '\n'.join(lines)+'\n', code_pages, data_pages


def linked_audit(exe, map_path):
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_REG
    pe, symbols = PE(exe), {}
    for line in map_path.read_text().splitlines():
        match = re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\w+)\s+([0-9A-Fa-f]{8})\b', line)
        if match:
            symbols[match[1]] = int(match[2], 16)
    imports = pe.imports()
    assert {name for _, name in imports.values()} == IMPORTS
    assert {dll.upper() for dll, _ in imports.values()} == {'KERNEL32.DLL'}
    start, end = symbols['OwnedStart'], symbols['OwnedEnd']
    code = list(decoder().disasm(pe.read(start, end-start), start))
    assert sum(i.size for i in code) == end-start
    boundaries = {i.address for i in code}
    indirect_targets = set(STUBS.values()) | {'HostFailure'}
    for n, i in enumerate(code):
        assert i.mnemonic not in {'int', 'syscall', 'sysenter', 'hlt', 'iret', 'iretd'}
        if i.group(CS_GRP_JUMP):
            op = i.operands[0]
            if op.type == X86_OP_IMM:
                assert op.imm in boundaries
            else:
                assert op.type == X86_OP_REG and i.op_str == 'eax'
                prev = code[n-1]
                assert prev.mnemonic == 'mov' and prev.operands[1].type == X86_OP_IMM
                assert prev.operands[1].imm in {0x9C32A0, 0x9C3857} | {symbols[k] for k in indirect_targets}
        if i.group(CS_GRP_CALL):
            assert i.operands[0].type == X86_OP_IMM
            target = i.operands[0].imm
            if target in boundaries:
                assert target in {symbols[k] for k in ['ReadRecord', 'ValidateRecord', 'RunOnce']}
            else:
                thunk = pe.read(target, 6)
                assert thunk[:2] == b'\xFF\x25' and struct.unpack('<I', thunk[2:])[0] in imports
    for at, label in STUBS.items():
        assert pe.read(symbols[f'Stub_{at:08X}'], 7) == b'\xb8'+struct.pack('<I', symbols[label])+b'\xff\xe0'
    pe_at = struct.unpack_from('<I', pe.data, 0x3C)[0]
    section_count = struct.unpack_from('<H', pe.data, pe_at+6)[0]
    optional = struct.unpack_from('<H', pe.data, pe_at+20)[0]
    sections = []
    for i in range(section_count):
        at = pe_at+24+optional+40*i
        size, rva = struct.unpack_from('<II', pe.data, at+8)
        sections.append((pe.base+rva, pe.base+rva+size,
                         struct.unpack_from('<I', pe.data, at+36)[0]))
    assert pe.base == 0x24000000
    for name in ['HostMain', 'VotesDone', 'RootDone', 'ClearCandidates', 'AppendCandidate', 'FillCounts']:
        flags = next(flags for a, b, flags in sections if a <= symbols[name] < b)
        assert flags & 0xE0000000 == 0x60000000
    for first, (label, size) in enumerate(BUFFERS):
        at = symbols[label+'Buffer']
        flags = next(flags for a, b, flags in sections if a <= at-16 and at+size+16 <= b)
        assert flags & 0xE0000000 == 0xC0000000
        for other, other_size in BUFFERS[first+1:]:
            other_at = symbols[other+'Buffer']
            assert at+size+16 <= other_at-16 or other_at+other_size+16 <= at-16
    return dict(symbols={k: f'{v:08X}' for k, v in symbols.items() if not k.startswith('_')},
        owned_code_sha256=hashlib.sha256(pe.read(start, end-start)).hexdigest(),
        imports=[dict(iat=f'{a:08X}', dll=dll, name=name) for a, (dll, name) in imports.items()],
        disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in code])


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('exe', type=Path)
    p.add_argument('--sdk-bin-x86', type=Path, required=True)
    p.add_argument('--kernel32-lib', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    a.out.resolve().relative_to((ROOT/'target').resolve())
    image = SourceImage(a.exe)
    records = audit_source(image)
    text, code_pages, data_pages = host_source(image)
    a.out.mkdir(parents=True, exist_ok=False)
    asm, obj, exe, mapping = [a.out.resolve()/name for name in ['host-x86.asm', 'host-x86.obj', 'host-x86.exe', 'host-x86.map']]
    asm.write_text(text)
    commands = [[str(a.sdk_bin_x86/'ml.exe'), '/nologo', '/c', '/coff', f'/Fo{obj}', str(asm)],
        [str(a.sdk_bin_x86/'link.exe'), '/NOLOGO', '/MACHINE:X86', '/SUBSYSTEM:CONSOLE',
         '/ENTRY:HostMain', '/BASE:0x24000000', '/DYNAMICBASE:NO', '/SAFESEH:NO',
         f'/OUT:{exe}', f'/MAP:{mapping}', str(obj), str(a.kernel32_lib)]]
    for command in commands:
        result = subprocess.run(command, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout+'\n'+result.stderr
    manifest = dict(schema='bounded-original-body-root-selection-host-v1', host_executed=False,
        source_path=str(a.exe.resolve()), source_sha256=SOURCE_SHA256, code_allowlist=records,
        data_allowlist=[dict(address=f'{at:08X}', bytes=size,
            sha256=hashlib.sha256(image.read(at, size)).hexdigest()) for at, size in CONSTANTS],
        host_source_sha256=hashlib.sha256(asm.read_bytes()).hexdigest(),
        host_exe_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),
        linked_audit=linked_audit(exe, mapping), compile_commands=commands,
        input_bytes=INPUT_BYTES, projection_bytes=PROJECTION_BYTES, output_bytes=OUTPUT_BYTES,
        input='CW:u16@0,reserved:u16@2,count:u32@4(1..255),policyBits:u32@8(0..3),reserved:u32@12;255records32bytes@16:parent,presence,nonzero,centerXYZbits,paletteID,symmetryOrdinal.',
        output='rootCandidate:u32@0(FFFFFFFF unset),noFootCandidates:u32@4,candidateCount:u32@8,maximumVotes:u32@12;255zero-paddedvoteCounts:u32@16,255zero-paddedcandidateOrdinals:u32@1036;MXCSR@2056,CW/SW@2060/2062.',
        entry_proof='Owned gateway supplies the exact root-vote loop state at9C32A0, then7-byte boundary gateway at9C3355 enters unchanged9C3857 policy region. Stops at9C3D0A before unchecked candidate dereference/admission, retaining FFFFFFFF and presence-only-spine choices as source results. Matrix/symmetry regions and rootcap/anchor/postcopy changes are not executed or claimed.',
        storage_proof='Clear/append/fill providers check exact native returns, vector-object and byreference-value addresses, fixed count/capacity and ordinal bounds. Candidate clear changes only end; append preserves native list order. Fallback count fill writes the owned inline255-DWORD region. No actual allocation/copy/free provider runs: unreachable growth/free/zero-size-copy transfers map only to fatal exit.',
        topology_proof='Own validation bounds all parent/symmetry indices before following any chain, requires unique initial root0 and nonzero-spine admission, rejects cycles with<=count visits, and requires every foot to have a nonzero-spine ancestor before original negative-index vote access. Only finite center words are admitted; squared-distance overflow remains original SSE behavior.',
        memory_proof='Seven separate guarded input/definition/blocks/counts/candidates/frame/reply buffers. Native ESP is frame+100, with256-byte lower provider-stack reserve and enough room through native+1540. Every native subscript has<=255 bound. Full input/block shadows and definition shadow excluding only native3F4 byte are checked before output. Source code RX, constants/policy globals R during calls; unlisted code INT3. No protocol pointer is used.',
        arithmetic_proof='Explicit12masked CWs and initialMXCSR1F80; original root arithmetic uses SSE only. Exact nativeESP, unchangedCW, zeroSW and unchanged MXCSR control bits checked before result serialization; stickySSE flags retained. Two zero/all-one XMM scratch runs must match all2056projection bytes.',
        pages=[dict(address=f'{page:08X}', bytes=65536, initial='RW', final='RX' if page in code_pages else 'R') for page in sorted(code_pages|data_pages)],
        execution_scope='Preparation only. No game/original module/initializer/GUI/network/Cargo. New exact host/cohort requires root review before separate hidden execution.')
    (a.out/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps({k:manifest[k] for k in ['schema', 'host_executed', 'host_source_sha256', 'host_exe_sha256']}, indent=2))


if __name__ == '__main__':
    main()

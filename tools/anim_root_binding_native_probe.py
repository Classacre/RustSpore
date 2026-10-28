"""Prepare source-pinned ordinary-root constructor bytes and a no-CRT host.

This generator never compiles, links or executes. Root must review and perform
those actions separately. Unchanged native growth/fallback paths remain present
but their external callees are INT3 traps under the bounded input contract.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import sys
from cell_schema_probe import SourceImage, SOURCE_SHA256
from cell_native_math_probe import db
from native_crt_acos_probe import decoder

ROOT = Path(__file__).resolve().parent.parent
CODE = [(0x9a0e30, 0x9a1162), (0x99c340, 0x99c3a1)]
DATA = [(0x99c3a4, 24), (0x99c3bc, 32), (0x1485e24, 4)]
UNREACHED_CALLS = {0x9a0980, 0x9a0dd0}
BUFFERS = [('Input', 364), ('Frame', 0xd20), ('Definition', 1024),
           ('Blocks', 0x468), ('Channel', 0xe4), ('Descriptors', 24*32),
           ('Vector', 12), ('Records', 32), ('Stack', 1024), ('Reply', 48)]


def audit(image):
    decoder_instance = decoder()
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    decoded = [list(decoder_instance.disasm(image.read(a, b-a), a)) for a, b in CODE]
    boundaries = {i.address for rows in decoded for i in rows}
    transfers, absolute = [], []
    for (a, b), rows in zip(CODE, decoded):
        assert sum(i.size for i in rows) == b-a
        for i in rows:
            if i.group(CS_GRP_CALL) or i.group(CS_GRP_JUMP):
                op = i.operands[0]
                if op.type == X86_OP_IMM:
                    assert op.imm in boundaries | UNREACHED_CALLS
                    transfers.append(dict(at=f'{i.address:08X}', operation=i.mnemonic,
                                          target=f'{op.imm:08X}'))
                else:
                    assert i.address == 0x99c35d and op.type == X86_OP_MEM
                    transfers.append(dict(at=f'{i.address:08X}', operation=i.mnemonic,
                                          table='0099C3A4', entries=6))
            for op in i.operands:
                if op.type == X86_OP_MEM and not op.mem.base:
                    assert op.mem.disp in {p for p, _ in DATA}
                    absolute.append(dict(at=f'{i.address:08X}', address=f'{op.mem.disp:08X}'))
    table = struct.unpack('<6I', image.read(0x99c3a4, 24))
    assert all(t in boundaries for t in table)
    assert image.read(0x1485e24, 4) == bytes(4)
    return dict(code=[dict(start=f'{a:08X}', end_exclusive=f'{b:08X}', bytes=b-a,
                sha256=hashlib.sha256(image.read(a, b-a)).hexdigest()) for a, b in CODE],
        data=[dict(address=f'{a:08X}', bytes=n, sha256=hashlib.sha256(image.read(a, n)).hexdigest()) for a, n in DATA],
        transfers=transfers, absolute_data=absolute,
        disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for rows in decoded for i in rows])


def host_source(image):
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none']
    for name, n in [('GetStdHandle',4), ('ReadFile',20), ('WriteFile',20),
                    ('VirtualAlloc',16), ('VirtualProtect',16), ('FlushInstructionCache',12),
                    ('GetLastError',0), ('ExitProcess',4)]:
        lines += [f'EXTERN _{name}@{n}:PROC']
    lines += ['.const']
    for a, b in CODE:
        lines += db(f'Original_{a:08X}', image.read(a, b-a))
    for a, n in DATA:
        lines += db(f'Data_{a:08X}', image.read(a, n))
    lines += ['.data']
    for label, n in BUFFERS:
        lines += [f'{label}Before DWORD 4 DUP(0A5A5A5A5h)',
                  f'{label}Buffer BYTE {n} DUP(0)', f'{label}After DWORD 4 DUP(0A5A5A5A5h)']
    for label, n in BUFFERS:
        if label not in ('Stack', 'Reply', 'Records'):
            lines += [f'{label}Shadow BYTE {n} DUP(0)']
    lines += ['InputHandle DWORD 0', 'OutputHandle DWORD 0', 'Transferred DWORD 0',
              'OldProtect DWORD 0', 'SavedStack DWORD 0', 'Remaining DWORD 0',
              'Cursor DWORD 0', '.code', 'PUBLIC HostMain,HostFailure,RunOnce,ValidateRecord',
              'ValidateRecord PROC', 'cmp dword ptr [InputBuffer],1', 'je SelectorOK',
              'cmp dword ptr [InputBuffer],5', 'jne HostFailure', 'SelectorOK:',
              'cmp dword ptr [InputBuffer+4],0', 'jne HostFailure']
    for at in (12,16,32):
        lines += [f'cmp dword ptr [InputBuffer+{at}],255', 'ja HostFailure']
    lines += ['cmp dword ptr [InputBuffer+28],1', 'ja HostFailure',
              'cmp dword ptr [InputBuffer+40],24', 'ja HostFailure',
              'mov eax,dword ptr [InputBuffer+44]', 'cmp eax,-1', 'je SecondaryOK',
              'cmp eax,255', 'ja HostFailure', 'SecondaryOK:',
              'mov eax,dword ptr [InputBuffer+36]', 'and eax,032h', 'jz MirrorOK',
              'cmp eax,020h', 'jne HostFailure', 'MirrorOK:',
              'mov esi,OFFSET InputBuffer+48', 'mov ecx,16', 'ValidateReserved:',
              'cmp byte ptr [esi],0', 'jne HostFailure', 'inc esi', 'loop ValidateReserved',
              'mov eax,dword ptr [InputBuffer+40]', 'imul eax,12',
              'lea esi,InputBuffer+64[eax]', 'mov ecx,288', 'sub ecx,eax',
              'test ecx,ecx', 'jz PaddingDone', 'ValidatePadding:',
              'cmp byte ptr [esi],0', 'jne HostFailure', 'inc esi', 'loop ValidatePadding',
              'PaddingDone:', 'cmp word ptr [InputBuffer+361],0', 'jne HostFailure',
              'cmp byte ptr [InputBuffer+363],0', 'jne HostFailure',
              'xor ebx,ebx', 'xor ebp,ebp', 'xor edx,edx',
              'mov esi,OFFSET InputBuffer+64', 'ValidateDescriptor:',
              'cmp ebx,dword ptr [InputBuffer+40]', 'jae DescriptorsReady',
              'mov eax,dword ptr [esi]', 'and eax,15', 'cmp eax,3', 'jne DescriptorOK',
              'cmp dword ptr [esi+4],070E47545h', 'je HostFailure', 'DescriptorOK:',
              'mov eax,dword ptr [esi+8]', 'cmp eax,9', 'jae DescriptorNext',
              'movzx ecx,byte ptr [InputBuffer+352+eax]', 'cmp ecx,255', 'je ClaimOK',
              'cmp ecx,dword ptr [InputBuffer+32]', 'jne DescriptorNext', 'ClaimOK:',
              'mov ebp,1', 'test byte ptr [esi],16', 'jz DescriptorNext', 'mov edx,1',
              'DescriptorNext:', 'inc ebx', 'add esi,12', 'jmp ValidateDescriptor', 'DescriptorsReady:',
              'test edx,edx', 'jnz FallbackSafe', 'test ebp,ebp', 'jz FallbackSafe',
              'test byte ptr [InputBuffer+36],8', 'jnz HostFailure', 'FallbackSafe:', 'ret',
              'ValidateRecord ENDP', 'RunOnce PROC', 'mov SavedStack,esp']
    for label, n in BUFFERS:
        if label != 'Input':
            lines += [f'mov edi,OFFSET {label}Buffer', f'mov ecx,{n//4}', 'xor eax,eax', 'cld', 'rep stosd']
    lines += ['mov esi,OFFSET InputBuffer+64', 'mov edi,OFFSET DescriptorsBuffer',
              'mov ecx,dword ptr [InputBuffer+40]', 'test ecx,ecx', 'jz DescriptorsCopied',
              'CopyDescriptor:', 'mov eax,dword ptr [esi]', 'mov dword ptr [edi],eax',
              'mov eax,dword ptr [esi+4]', 'mov dword ptr [edi+4],eax',
              'mov eax,dword ptr [esi+8]', 'mov dword ptr [edi+16],eax',
              'add esi,12', 'add edi,32', 'loop CopyDescriptor', 'DescriptorsCopied:',
              'mov esi,OFFSET InputBuffer+352', 'mov edi,OFFSET FrameBuffer+24', 'mov ecx,9', 'rep movsb',
              'mov dword ptr [FrameBuffer+4],OFFSET DefinitionBuffer',
              'mov dword ptr [DefinitionBuffer+0384h],OFFSET BlocksBuffer',
              'mov dword ptr [FrameBuffer+8],OFFSET VectorBuffer',
              'mov dword ptr [VectorBuffer],OFFSET RecordsBuffer',
              'mov dword ptr [VectorBuffer+4],OFFSET RecordsBuffer',
              'mov dword ptr [VectorBuffer+8],OFFSET RecordsBuffer+32',
              'mov dword ptr [FrameBuffer+0910h],OFFSET ChannelBuffer',
              'mov dword ptr [ChannelBuffer+0E0h],OFFSET DescriptorsBuffer',
              'mov eax,dword ptr [InputBuffer+40]', 'mov dword ptr [ChannelBuffer+0DCh],eax',
              'mov eax,dword ptr [InputBuffer]', 'mov dword ptr [ChannelBuffer+08Ch],eax',
              'mov eax,dword ptr [InputBuffer+36]', 'mov dword ptr [ChannelBuffer+0ACh],eax',
              'mov eax,dword ptr [InputBuffer+32]', 'mov byte ptr [ChannelBuffer+0AEh],al',
              'mov eax,dword ptr [InputBuffer+44]', 'mov dword ptr [FrameBuffer+0D14h],eax']
    for label, n in BUFFERS:
        if label not in ('Stack', 'Reply', 'Records'):
            lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                      f'mov ecx,{n}', 'cld', 'rep movsb']
    lines += ['mov esp,OFFSET StackBuffer+1024',
              'push dword ptr [InputBuffer+24]', 'push dword ptr [InputBuffer+20]',
              'push dword ptr [InputBuffer+16]', 'push dword ptr [InputBuffer+12]',
              'push dword ptr [InputBuffer+4]', 'push dword ptr [InputBuffer+8]',
              'mov ecx,OFFSET FrameBuffer', 'mov eax,dword ptr [InputBuffer+28]',
              'mov edx,009A0E30h', 'call edx',
              'cmp esp,OFFSET StackBuffer+1024', 'jne HostFailure', 'mov esp,SavedStack']
    for label, _ in BUFFERS:
        for side in ('Before', 'After'):
            for at in range(0,16,4):
                lines += [f'cmp dword ptr [{label}{side}+{at}],0A5A5A5A5h', 'jne HostFailure']
    for label, n in BUFFERS:
        if label in ('Stack', 'Reply', 'Records'):
            continue
        if label in ('Frame', 'Vector'):
            lines += ['xor ecx,ecx', f'Check{label}:']
            if label == 'Frame':
                lines += ['cmp ecx,12', f'jb Compare{label}', 'cmp ecx,33', f'jb Next{label}']
            else:
                lines += ['cmp ecx,4', f'jb Compare{label}', 'cmp ecx,8', f'jb Next{label}']
            lines += [f'Compare{label}:', f'mov al,byte ptr [{label}Buffer+ecx]',
                      f'cmp al,byte ptr [{label}Shadow+ecx]', 'jne HostFailure',
                      f'Next{label}:', 'inc ecx', f'cmp ecx,{n}', f'jb Check{label}']
        else:
            lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                      f'mov ecx,{n}', 'cld', 'repe cmpsb', 'jne HostFailure']
    lines += ['mov esi,OFFSET RecordsBuffer+16', 'mov ecx,16', 'CheckRecordTail:',
              'cmp byte ptr [esi],0', 'jne HostFailure', 'inc esi', 'loop CheckRecordTail']
    lines += ['mov eax,dword ptr [VectorBuffer+4]', 'sub eax,OFFSET RecordsBuffer',
              'test eax,15', 'jnz HostFailure', 'cmp eax,16', 'ja HostFailure',
              'shr eax,4', 'mov dword ptr [ReplyBuffer],eax']
    for at, dst in ((12,4),(16,8),(20,12)):
        lines += [f'mov eax,dword ptr [FrameBuffer+{at}]', f'mov dword ptr [ReplyBuffer+{dst}],eax']
    lines += ['mov esi,OFFSET FrameBuffer+24', 'mov edi,OFFSET ReplyBuffer+16', 'mov ecx,9', 'rep movsb',
              'mov esi,OFFSET RecordsBuffer', 'mov edi,OFFSET ReplyBuffer+28', 'mov ecx,16', 'rep movsb',
              'ret', 'RunOnce ENDP', 'HostMain PROC']
    pages = sorted({a & ~0xffff for a,b in CODE} | {a & ~0xffff for a,n in DATA})
    for page in pages:
        lines += ['push 04h', 'push 03000h', 'push 010000h', f'push 0{page:08X}h',
                  'call _VirtualAlloc@16', f'cmp eax,0{page:08X}h', 'jne HostFailure',
                  f'mov edi,0{page:08X}h', 'mov ecx,010000h', 'mov al,0CCh', 'cld', 'rep stosb']
    for a,b in CODE:
        lines += [f'mov esi,OFFSET Original_{a:08X}', f'mov edi,0{a:08X}h', f'mov ecx,{b-a}', 'rep movsb']
    for a,n in DATA:
        lines += [f'mov esi,OFFSET Data_{a:08X}', f'mov edi,0{a:08X}h', f'mov ecx,{n}', 'rep movsb']
    for page in pages:
        protection = '020h' if page != 0x1480000 else '02h'
        lines += ['push OFFSET OldProtect', f'push {protection}', 'push 010000h', f'push 0{page:08X}h',
                  'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure']
    lines += ['push 0', 'push 0', 'push -1', 'call _FlushInstructionCache@12',
              'test eax,eax', 'jz HostFailure', 'push -10', 'call _GetStdHandle@4', 'mov InputHandle,eax',
              'push -11', 'call _GetStdHandle@4', 'mov OutputHandle,eax',
              'NextRecord:', 'mov Remaining,364', 'mov Cursor,OFFSET InputBuffer',
              'ReadMore:', 'mov Transferred,0', 'push 0', 'push OFFSET Transferred',
              'push Remaining', 'push Cursor', 'push InputHandle', 'call _ReadFile@20',
              'test eax,eax', 'jnz ReadOK', 'call _GetLastError@0', 'cmp eax,109', 'jne HostFailure',
              'ReadOK:', 'mov eax,Transferred', 'test eax,eax', 'jnz GotBytes',
              'cmp Remaining,364', 'jne HostFailure', 'push 0', 'call _ExitProcess@4',
              'GotBytes:', 'cmp eax,Remaining', 'ja HostFailure', 'add Cursor,eax',
              'sub Remaining,eax', 'jnz ReadMore', 'call ValidateRecord', 'call RunOnce',
              'mov Remaining,48', 'mov Cursor,OFFSET ReplyBuffer', 'WriteMore:', 'mov Transferred,0',
              'push 0', 'push OFFSET Transferred', 'push Remaining', 'push Cursor', 'push OutputHandle',
              'call _WriteFile@20', 'test eax,eax', 'jz HostFailure', 'mov eax,Transferred',
              'test eax,eax', 'jz HostFailure', 'cmp eax,Remaining', 'ja HostFailure',
              'add Cursor,eax', 'sub Remaining,eax', 'jnz WriteMore', 'jmp NextRecord',
              'HostMain ENDP', 'HostFailure PROC', 'push 71', 'call _ExitProcess@4', 'HostFailure ENDP', 'END']
    return '\n'.join(lines)+'\n'


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('exe', type=Path)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    a.out.resolve().relative_to((ROOT/'target').resolve())
    image = SourceImage(a.exe)
    source_audit = audit(image)
    a.out.mkdir(parents=True, exist_ok=False)
    asm = a.out/'host-x86.asm'
    asm.write_text(host_source(image))
    report = dict(schema='ordinary-root-constructor-host-preparation-v1',
        native_executed=False, compiled=False, linked=False, source_sha256=SOURCE_SHA256,
        source_audit=source_audit, host_source_sha256=hashlib.sha256(asm.read_bytes()).hexdigest(),
        input_bytes=364, output_bytes=48,
        input_contract='See anim_root_binding_fixture.py fixed protocol. flags1/5,root0,count<=24,source-reached mirror modes0/20,secondary-count0. No pseudo descriptors. Empty mask with admitted descriptors requires AC bit8 clear, verified by reference before native scheduling.',
        closure='Source9A0E30..9A1162 retained unchanged. Mirror99C340..99C3A1 and its exact tables/zero constant retained unchanged. Full mirror helper retained, but only integer default/zero paths admitted. No runtime hook or source stub. Growth9A0980 and fallback9A0DD0 remain INT3 pages: fixed32-byte capacity and empty-fallback exclusion prove unreachability for admitted cohorts.',
        memory='Guarded independent buffers,root0 only,24 descriptors,slot<9 before indexed claim access,at mostone ordinary16-byte record. Native stack has1024owned bytes. Count/flag/domain/padding/empty-fallback checked; no protocol pointers. Full immutable input/definition/blocks/channel/descriptors shadows; frame excludes only counters+claims bytes12..32 inclusive; vector excludes only end pointer bytes4..7; record tail stayszero. Linked audit remains pending.',
        pending=['Root source/host review', 'Independent source/data/CFG hash reproduction',
                 'Compile/link and linked code/import/section audit',
                 'Native positive/negative protocol and scratch/canary tests', 'Native/Python/Rust comparison'])
    (a.out/'manifest.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k:report[k] for k in ('schema','compiled','linked','native_executed','host_source_sha256')}, indent=2))


if __name__ == '__main__':
    main()

"""Prepare, never compile/execute, a bounded INFO+kind3 curve fixture host.

Only the common sampler, two-scalar branch, tail, neighbors and endpoint copy
helper are copied. The immutable descriptors cannot enter omitted POS/ROT.
All original bytes, generated assembly and optional linked audit stay ignored.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import sys

from cell_schema_probe import SourceImage, SOURCE_SHA256
from cell_native_math_probe import db
from native_crt_acos_probe import PE, decoder

CODE = [(0x99F6A0, 0x99FAF7), (0x9A0890, 0x9A0974),
        (0x99C6C0, 0x99C772), (0x99F630, 0x99F697)]
DATA = [(0x13EB96C, 4), (0x13F4F98, 4), (0x14719E4, 4), (0x14861D0, 4),
        (0x15514CC, 16)] + [(0x14477D8+0x20*slot, 4) for slot in [0, 3, 4, 5, 6, 7, 8]]
COPY = 0x11E0534
POLICIES = [(flags, slot) for flags in [0x83, 0x93, 0xA3, 0xB3] for slot in range(3, 9)]
SLOTS = [0, 20, 100, 200, 240, 280, 320, 360, 400]
INPUT_BYTES, OUTPUT_BYTES = 9632, 472
BUFFERS = [('Input', INPUT_BYTES), ('Output', 440), ('Channel', 0xE4),
           ('Header', 0x124), ('Neighbors', 16), ('Exact', 4), ('Reply', OUTPUT_BYTES)]
IMMUTABLE = {'Input', 'Channel', 'Header', 'Neighbors'}
IMPORTS = {'GetStdHandle', 'ReadFile', 'WriteFile', 'VirtualAlloc', 'VirtualProtect',
           'GetLastError', 'FlushInstructionCache', 'ExitProcess'}
OMITTED = {0x99F89E: 0x9A0499, 0x99F8A7: 0x99FAF7}


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def audit_source(image):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    instructions = [list(decoder().disasm(image.read(a, b-a), a)) for a, b in CODE]
    boundaries = {ins.address for rows in instructions for ins in rows}
    records = []
    for (start, end), rows in zip(CODE, instructions):
        assert sum(ins.size for ins in rows) == end-start
        transfers, omitted, memory = [], [], []
        for ins in rows:
            if ins.group(CS_GRP_CALL) or ins.group(CS_GRP_JUMP):
                assert ins.operands[0].type == X86_OP_IMM
                target = ins.operands[0].imm
                if ins.group(CS_GRP_CALL):
                    assert target in {0x99C6C0, 0x99F630, COPY}
                    transfers.append(dict(at=f'{ins.address:08X}', operation=ins.mnemonic, target=f'{target:08X}'))
                elif target not in boundaries:
                    assert OMITTED.get(ins.address) == target and ins.mnemonic == 'je'
                    omitted.append(dict(at=f'{ins.address:08X}', target=f'{target:08X}',
                        proof='INFO flags0 skips enable dispatch. Every other immutable descriptor has low kind3; the type1/type2 equal branches are false.'))
                else:
                    transfers.append(dict(at=f'{ins.address:08X}', operation=ins.mnemonic, target=f'{target:08X}'))
            for op in ins.operands:
                if op.type != X86_OP_MEM:
                    continue
                displacement = op.mem.disp & 0xFFFFFFFF
                if not op.mem.base and not op.mem.index:
                    assert any(a <= displacement and displacement+op.size <= a+n for a, n in DATA)
                    memory.append(dict(at=f'{ins.address:08X}', operand=ins.op_str, bytes=op.size))
                elif op.mem.disp >= 0x1000000:
                    assert (ins.address, displacement) in {
                        (0x99F8F7, 0x14477D8), (0x99F66B, 0x15514CC), (0x99F677, 0x14477D8)}
                    memory.append(dict(at=f'{ins.address:08X}', operand=ins.op_str, bytes=op.size,
                        proof='Immutable low kinds0/3 and slots0/3..8 bound these sparse source table reads.'))
        records.append(dict(start=f'{start:08X}', end_exclusive=f'{end:08X}', bytes=end-start,
            sha256=digest(image.read(start, end-start)), transfers=transfers,
            unreachable_conditional_edges=omitted, indirect_transfers=0, absolute_or_indexed_data=memory,
            disassembly=[f'{ins.address:08X}: {ins.bytes.hex()} {ins.mnemonic} {ins.op_str}' for ins in rows]))
    assert {int(item['at'], 16) for record in records for item in record['unreachable_conditional_edges']} == set(OMITTED)
    assert struct.unpack('<4I', image.read(0x15514CC, 16)) == (20, 80, 100, 40)
    assert [struct.unpack('<I', image.read(0x14477D8+0x20*i, 4))[0] for i in range(9)] == SLOTS
    return records


def host_source(image):
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none']
    for name, size in [('GetStdHandle', 4), ('ReadFile', 20), ('WriteFile', 20),
                       ('VirtualAlloc', 16), ('VirtualProtect', 16), ('GetLastError', 0),
                       ('FlushInstructionCache', 12), ('ExitProcess', 4)]:
        lines += [f'EXTERN _{name}@{size}:PROC']
    lines += ['.const', 'PUBLIC Descriptors,DescriptorsEnd,PolicyOffsets', 'Descriptors LABEL BYTE']
    for flags, slot in POLICIES:
        lines += ['DWORD 0,04F464E49h,0,60,0,0,0,0',
                  f'DWORD 0{flags:X}h,098C942B6h,20,60,{slot},0,0,0']
    lines += ['DescriptorsEnd LABEL BYTE', 'PolicyOffsets DWORD '+','.join(str(SLOTS[s]) for _, s in POLICIES)]
    for start, end in CODE:
        lines += [f'PUBLIC Source_{start:08X}'] + db(f'Source_{start:08X}', image.read(start, end-start))
    for address, count in DATA:
        lines += [f'PUBLIC Data_{address:08X}'] + db(f'Data_{address:08X}', image.read(address, count))
    lines += ['.data']
    for label, size in BUFFERS:
        lines += [f'PUBLIC {label}Buffer', f'{label}Before DWORD 4 DUP(0A5A5A5A5h)',
                  f'{label}Buffer BYTE {size} DUP(0)', f'{label}After DWORD 4 DUP(0A5A5A5A5h)']
        if label in IMMUTABLE:
            lines += [f'{label}Shadow BYTE {size} DUP(0)']
    lines += ['InputHandle DWORD 0', 'OutputHandle DWORD 0', 'Transferred DWORD 0',
        'OldProtect DWORD 0', 'SseControl DWORD 01F80h', 'NativeStack DWORD 0',
        'AlignedStack DWORD 0', 'NativeAL DWORD 0', 'OutputOffset DWORD 0', 'OutputExtent DWORD 0',
        '.code', 'PUBLIC OwnedStart,OwnedEnd,CopyStub,CopyStubEnd,HostMain,HostFailure,ReadRecord,CheckPreserved',
        'OwnedStart LABEL BYTE', 'CopyStub PROC', 'push ebx', 'push esi', 'push edi',
        'mov ebx,dword ptr [esp+16]', 'mov esi,dword ptr [esp+20]',
        'cmp ebx,OFFSET OutputBuffer', 'je CopyInfo',
        'mov eax,OFFSET OutputBuffer', 'add eax,OutputOffset', 'cmp ebx,eax', 'jne CopyFailure',
        'cmp dword ptr [esp+24],40', 'jne CopyFailure',
        'mov eax,esi', 'sub eax,OFFSET InputBuffer+52', 'jb CopyFailure', 'jmp CopyRow',
        'CopyInfo:', 'cmp dword ptr [esp+24],20', 'jne CopyFailure',
        'mov eax,esi', 'sub eax,OFFSET InputBuffer+32', 'jb CopyFailure', 'CopyRow:',
        'xor edx,edx', 'mov ecx,60', 'div ecx', 'test edx,edx', 'jnz CopyFailure',
        'cmp eax,dword ptr [InputBuffer+4]', 'jae CopyFailure',
        'mov edi,ebx', 'mov ecx,dword ptr [esp+24]', 'cld', 'rep movsb', 'mov eax,ebx',
        'pop edi', 'pop esi', 'pop ebx', 'ret',
        'CopyFailure:', 'mov eax,OFFSET HostFailure', 'jmp eax', 'CopyStub ENDP', 'CopyStubEnd LABEL BYTE',
        'CheckPreserved PROC', 'cmp ebx,011223344h', 'jne HostFailure',
        'cmp esi,022334455h', 'jne HostFailure', 'cmp edi,033445566h', 'jne HostFailure',
        'cmp ebp,044556677h', 'jne HostFailure', 'pushfd', 'pop eax', 'test eax,0400h',
        'jnz HostFailure', 'ret', 'CheckPreserved ENDP',
        'ReadRecord PROC', 'push ebx', 'push esi', f'mov ebx,{INPUT_BYTES}',
        'mov esi,OFFSET InputBuffer', 'ReadLoop:', 'mov Transferred,0', 'push 0',
        'push OFFSET Transferred', 'push ebx', 'push esi', 'push InputHandle', 'call _ReadFile@20',
        'test eax,eax', 'jnz ReadTransferred', 'call _GetLastError@0', 'cmp eax,109',
        'jne HostFailure', f'cmp ebx,{INPUT_BYTES}', 'jne HostFailure', 'cmp Transferred,0',
        'jne HostFailure', 'xor eax,eax', 'jmp ReadDone', 'ReadTransferred:',
        'mov eax,Transferred', 'cmp eax,ebx', 'ja HostFailure', 'test eax,eax', 'jnz ReadSome',
        f'cmp ebx,{INPUT_BYTES}', 'jne HostFailure', 'xor eax,eax', 'jmp ReadDone',
        'ReadSome:', 'add esi,eax', 'sub ebx,eax', 'jnz ReadLoop', 'mov eax,1',
        'ReadDone:', 'pop esi', 'pop ebx', 'ret', 'ReadRecord ENDP', 'HostMain PROC']
    code_pages = {a & ~0xFFFF for a, _ in CODE} | {COPY & ~0xFFFF}
    data_pages = {a & ~0xFFFF for a, _ in DATA}
    assert not code_pages & data_pages
    for page in sorted(code_pages | data_pages):
        lines += ['push 04h', 'push 03000h', 'push 010000h', f'push 0{page:08X}h',
                  'call _VirtualAlloc@16', f'cmp eax,0{page:08X}h', 'jne AllocationFailure']
        if page in code_pages:
            lines += [f'mov edi,0{page:08X}h', 'mov ecx,010000h', 'mov al,0CCh', 'cld', 'rep stosb']
    for start, end in CODE:
        lines += [f'mov esi,OFFSET Source_{start:08X}', f'mov edi,0{start:08X}h',
                  f'mov ecx,{end-start}', 'cld', 'rep movsb']
    for address, count in DATA:
        lines += [f'mov esi,OFFSET Data_{address:08X}', f'mov edi,0{address:08X}h',
                  f'mov ecx,{count}', 'rep movsb']
    lines += ['mov esi,OFFSET CopyStub', f'mov edi,0{COPY:08X}h',
              'mov ecx,CopyStubEnd-CopyStub', 'rep movsb']
    for page in sorted(code_pages | data_pages):
        lines += ['push OFFSET OldProtect', 'push 020h' if page in code_pages else 'push 02h',
                  'push 010000h', f'push 0{page:08X}h', 'call _VirtualProtect@16',
                  'test eax,eax', 'jz HostFailure']
    lines += ['push 0', 'push 0', 'push -1', 'call _FlushInstructionCache@12', 'test eax,eax', 'jz HostFailure',
        'push -10', 'call _GetStdHandle@4', 'mov InputHandle,eax', 'push -11',
        'call _GetStdHandle@4', 'mov OutputHandle,eax',
        'NextRecord:', 'call ReadRecord', 'test eax,eax', 'jz HostSuccess',
        'cmp dword ptr [InputBuffer+4],160', 'ja HostFailure',
        'mov eax,dword ptr [InputBuffer+24]', 'cmp eax,dword ptr [InputBuffer+4]', 'ja HostFailure',
        'mov eax,dword ptr [InputBuffer+16]', 'mov edx,eax', 'and eax,0FFFFF0FFh',
        'cmp eax,07Fh', 'jne HostFailure', 'and edx,0300h', 'cmp edx,0100h', 'je HostFailure',
        'cmp dword ptr [InputBuffer+20],24', 'jae HostFailure',
        'mov eax,dword ptr [InputBuffer+12]', 'and eax,0FFFFFFF7h', 'jnz HostFailure',
        'cmp dword ptr [InputBuffer+28],0', 'jne HostFailure']
    for at in [0, 8]:
        lines += [f'mov eax,dword ptr [InputBuffer+{at}]', 'and eax,07F800000h',
                  'cmp eax,07F800000h', 'je HostFailure']
    for label, size in BUFFERS:
        if label != 'Input':
            lines += [f'mov edi,OFFSET {label}Buffer', f'mov ecx,{size//4}',
                      'mov eax,0A5A5A5A5h', 'cld', 'rep stosd']
    lines += ['mov dword ptr [ChannelBuffer+4],OFFSET HeaderBuffer',
        'mov eax,dword ptr [InputBuffer+4]', 'mov dword ptr [ChannelBuffer+0D4h],eax',
        'mov dword ptr [ChannelBuffer+0D8h],OFFSET InputBuffer+32',
        'mov dword ptr [ChannelBuffer+0DCh],2', 'mov eax,dword ptr [InputBuffer+20]',
        'mov edx,eax', 'shl eax,6', 'add eax,OFFSET Descriptors', 'mov dword ptr [ChannelBuffer+0E0h],eax',
        'mov eax,dword ptr [PolicyOffsets+edx*4]', 'mov OutputOffset,eax',
        'mov eax,dword ptr [InputBuffer+12]', 'mov dword ptr [HeaderBuffer+0118h],eax',
        'mov eax,dword ptr [InputBuffer+8]', 'mov dword ptr [HeaderBuffer+0120h],eax',
        'mov dword ptr [ExactBuffer],0DEADBEEFh',
        'mov edi,OFFSET NeighborsBuffer', 'mov ecx,4', 'mov eax,0FFFFFFFFh', 'rep stosd']
    for label, size in BUFFERS:
        if label in IMMUTABLE and label != 'Neighbors':
            lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                      f'mov ecx,{size//4}', 'cld', 'rep movsd']
    lines += ['fninit', 'fldcw word ptr [InputBuffer+16]', 'ldmxcsr SseControl',
        'mov OutputExtent,0', 'cmp dword ptr [InputBuffer+4],0', 'je NeighborsReady',
        'mov eax,dword ptr [InputBuffer+24]', 'mov dword ptr [NeighborsBuffer+8],eax',
        'mov ebx,011223344h', 'mov esi,022334455h', 'mov edi,033445566h', 'mov ebp,044556677h',
        'mov NativeStack,esp', 'push OFFSET NeighborsBuffer+12', 'push OFFSET NeighborsBuffer+4',
        'push OFFSET NeighborsBuffer', 'push OFFSET NeighborsBuffer+8', 'push OFFSET ChannelBuffer',
        'mov eax,0099C6C0h', 'call eax', 'add esp,20', 'cmp esp,NativeStack', 'jne HostFailure',
        'call CheckPreserved']
    for at in range(0, 16, 4):
        lines += [f'mov eax,dword ptr [NeighborsBuffer+{at}]', 'cmp eax,dword ptr [InputBuffer+4]',
                  'jae HostFailure']
    lines += ['mov eax,dword ptr [NeighborsBuffer+4]', 'cmp eax,dword ptr [NeighborsBuffer+8]',
        'jne InterpolationExtent', 'mov OutputExtent,40', 'jmp NeighborsReady',
        'InterpolationExtent:', 'mov eax,dword ptr [ChannelBuffer+0E0h]',
        'test byte ptr [eax+32],010h', 'jz NeighborsReady', 'mov OutputExtent,8',
        'NeighborsReady:', 'mov esi,OFFSET NeighborsBuffer', 'mov edi,OFFSET NeighborsShadow',
        'mov ecx,4', 'cld', 'rep movsd',
        'mov ebx,011223344h', 'mov esi,022334455h', 'mov edi,033445566h', 'mov ebp,044556677h',
        'mov NativeStack,esp', 'and esp,0FFFFFFF0h', 'mov AlignedStack,esp', 'sub esp,12',
        'push OFFSET ExactBuffer', 'push OFFSET OutputBuffer', 'push OFFSET ChannelBuffer',
        'push dword ptr [InputBuffer+24]', 'push dword ptr [InputBuffer]',
        'mov eax,0099F6A0h', 'call eax', 'add esp,32', 'cmp esp,AlignedStack', 'jne HostFailure',
        'mov esp,NativeStack', 'movzx eax,al', 'mov NativeAL,eax', 'call CheckPreserved']
    # These guards and immutable comparisons precede every reply/context write.
    for label, size in BUFFERS:
        for side in ['Before', 'After']:
            for at in range(0, 16, 4):
                lines += [f'cmp dword ptr [{label}{side}+{at}],0A5A5A5A5h', 'jne HostFailure']
        if label in IMMUTABLE:
            lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                      f'mov ecx,{size//4}', 'cld', 'repe cmpsd', 'jne HostFailure']
    lines += ['xor ecx,ecx', 'OutputCheckLoop:', 'cmp dword ptr [InputBuffer+4],0',
        'je OutputCompare', 'cmp ecx,20', 'jb OutputCheckNext', 'mov edx,ecx',
        'sub edx,OutputOffset', 'cmp edx,OutputExtent', 'jb OutputCheckNext',
        'OutputCompare:', 'cmp dword ptr [OutputBuffer+ecx],0A5A5A5A5h', 'jne HostFailure',
        'OutputCheckNext:', 'add ecx,4', 'cmp ecx,440', 'jb OutputCheckLoop',
        'cmp dword ptr [InputBuffer+4],0', 'jne ExactChecked',
        'cmp dword ptr [ExactBuffer],0DEADBEEFh', 'jne HostFailure',
        'cmp NativeAL,0', 'jne HostFailure', 'ExactChecked:']
    for at in range(0, OUTPUT_BYTES, 4):
        lines += [f'cmp dword ptr [ReplyBuffer+{at}],0A5A5A5A5h', 'jne HostFailure']
    for start, end in CODE:
        lines += [f'mov esi,OFFSET Source_{start:08X}', f'mov edi,0{start:08X}h',
                  f'mov ecx,{end-start}', 'cld', 'repe cmpsb', 'jne HostFailure']
    for address, count in DATA:
        lines += [f'mov esi,OFFSET Data_{address:08X}', f'mov edi,0{address:08X}h',
                  f'mov ecx,{count}', 'repe cmpsb', 'jne HostFailure']
    lines += ['fnstcw word ptr [ReplyBuffer+464]', 'fnstsw word ptr [ReplyBuffer+466]',
        'stmxcsr dword ptr [ReplyBuffer+468]', 'mov ax,word ptr [ReplyBuffer+464]',
        'cmp ax,word ptr [InputBuffer+16]', 'jne HostFailure',
        'test word ptr [ReplyBuffer+466],03800h', 'jnz HostFailure',
        'mov eax,dword ptr [ReplyBuffer+468]', 'and eax,0FFFFFFC0h', 'cmp eax,01F80h', 'jne HostFailure',
        'mov esi,OFFSET OutputBuffer', 'mov edi,OFFSET ReplyBuffer', 'mov ecx,110', 'cld', 'rep movsd',
        'mov eax,dword ptr [ExactBuffer]', 'mov dword ptr [ReplyBuffer+440],eax',
        'mov eax,NativeAL', 'mov dword ptr [ReplyBuffer+444],eax',
        'mov esi,OFFSET NeighborsBuffer', 'mov edi,OFFSET ReplyBuffer+448', 'mov ecx,4', 'rep movsd',
        'push 0', 'push OFFSET Transferred', f'push {OUTPUT_BYTES}', 'push OFFSET ReplyBuffer',
        'push OutputHandle', 'call _WriteFile@20', 'test eax,eax', 'jz HostFailure',
        f'cmp Transferred,{OUTPUT_BYTES}', 'jne HostFailure', 'jmp NextRecord',
        'HostSuccess:', 'push 0', 'call _ExitProcess@4',
        'HostFailure::', 'push 7', 'call _ExitProcess@4',
        'AllocationFailure:', 'push 10', 'call _ExitProcess@4', 'HostMain ENDP', 'OwnedEnd LABEL BYTE', 'END HostMain']
    return '\n'.join(lines)+'\n', code_pages, data_pages


def audit_linked(path, mapping, image):
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_REG
    pe, symbols = PE(path), {}
    for line in mapping.read_text().splitlines():
        match = re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\w+)\s+([0-9A-Fa-f]{8})\b', line)
        if match:
            symbols[match[1]] = int(match[2], 16)
    assert pe.base == 0x24000000
    imports = pe.imports()
    assert {name for _, name in imports.values()} == IMPORTS
    assert {dll.upper() for dll, _ in imports.values()} == {'KERNEL32.DLL'}
    start, end = symbols['OwnedStart'], symbols['OwnedEnd']
    rows = list(decoder().disasm(pe.read(start, end-start), start))
    assert sum(ins.size for ins in rows) == end-start
    boundaries = {ins.address for ins in rows}
    copy_start, copy_end = symbols['CopyStub'], symbols['CopyStubEnd']
    for index, ins in enumerate(rows):
        assert ins.mnemonic not in {'int', 'syscall', 'sysenter', 'hlt', 'iret', 'iretd'}
        if ins.group(CS_GRP_JUMP):
            op = ins.operands[0]
            if op.type == X86_OP_IMM:
                assert op.imm in boundaries
                if copy_start <= ins.address < copy_end:
                    assert copy_start <= op.imm < copy_end
            else:
                assert op.type == X86_OP_REG and ins.mnemonic == 'jmp' and ins.op_str == 'eax'
                assert copy_start <= ins.address < copy_end
                previous = rows[index-1]
                assert previous.mnemonic == 'mov' and previous.operands[1].type == X86_OP_IMM
                assert previous.operands[1].imm == symbols['HostFailure']
        if ins.group(CS_GRP_CALL):
            assert not copy_start <= ins.address < copy_end
            op = ins.operands[0]
            if op.type == X86_OP_REG:
                assert ins.op_str == 'eax'
                previous = rows[index-1]
                assert previous.mnemonic == 'mov' and previous.operands[1].type == X86_OP_IMM
                assert previous.operands[1].imm in {0x99C6C0, 0x99F6A0}
            else:
                assert op.type == X86_OP_IMM
                if op.imm in boundaries:
                    assert op.imm in {symbols['ReadRecord'], symbols['CheckPreserved']}
                else:
                    raw = pe.read(op.imm, 6)
                    assert raw[:2] == b'\xff\x25' and struct.unpack('<I', raw[2:])[0] in imports
    at = struct.unpack_from('<I', pe.data, 0x3C)[0]
    count = struct.unpack_from('<H', pe.data, at+6)[0]
    optional = struct.unpack_from('<H', pe.data, at+20)[0]
    sections = []
    for index in range(count):
        row = at+24+optional+40*index
        size, rva = struct.unpack_from('<II', pe.data, row+8)
        sections.append((pe.base+rva, pe.base+rva+size, struct.unpack_from('<I', pe.data, row+36)[0]))
    def permission(address, size, expected):
        flags = next(flags for a, b, flags in sections if a <= address and address+size <= b)
        assert flags & 0xE0000000 == expected
    permission(start, end-start, 0x60000000)
    permission(symbols['Descriptors'], symbols['DescriptorsEnd']-symbols['Descriptors'], 0x40000000)
    assert symbols['DescriptorsEnd']-symbols['Descriptors'] == len(POLICIES)*64
    expected_descriptors = b''.join(struct.pack('<16I', 0, 0x4F464E49, 0, 60, 0, 0, 0, 0,
        flags, 0x98C942B6, 20, 60, slot, 0, 0, 0) for flags, slot in POLICIES)
    assert pe.read(symbols['Descriptors'], len(expected_descriptors)) == expected_descriptors
    permission(symbols['PolicyOffsets'], len(POLICIES)*4, 0x40000000)
    assert pe.read(symbols['PolicyOffsets'], len(POLICIES)*4) == struct.pack('<24I', *[SLOTS[s] for _, s in POLICIES])
    extents = []
    for label, size in BUFFERS:
        address = symbols[f'{label}Buffer']
        permission(address-16, size+32, 0xC0000000)
        assert all(address+size+16 <= low or high <= address-16 for low, high in extents)
        extents.append((address-16, address+size+16))
    for address, length in [(a, b-a) for a, b in CODE]+DATA:
        label = f'Source_{address:08X}' if any(address == a for a, _ in CODE) else f'Data_{address:08X}'
        permission(symbols[label], length, 0x40000000)
        assert pe.read(symbols[label], length) == image.read(address, length)
    copy_rows = [row for row in rows if copy_start <= row.address < copy_end]
    return dict(host_exe_sha256=digest(path.read_bytes()),
        symbols={k: f'{v:08X}' for k, v in symbols.items() if not k.startswith('_')},
        imports=[dict(address=f'{a:08X}', dll=dll, name=name) for a, (dll, name) in imports.items()],
        owned_code_sha256=digest(pe.read(start, end-start)), source_payloads_byte_exact=True,
        source_payloads_R_nonexecutable=True, descriptors_and_offsets_R_nonexecutable=True,
        owned_code_RX=True, guarded_buffers_RW_nonexecutable=True,
        copy_stub=dict(source_start=f'{copy_start:08X}', source_end_exclusive=f'{copy_end:08X}',
            bytes=copy_end-copy_start, sha256=digest(pe.read(copy_start, copy_end-copy_start)),
            outbound_relative_transfers=0, absolute_failure=f'{symbols["HostFailure"]:08X}',
            disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in copy_rows]),
        owned_disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in rows])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-exe', type=Path, default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--audit-linked', type=Path)
    parser.add_argument('--map', type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    args.out.resolve().relative_to((root/'target').resolve())
    image = SourceImage(args.source_exe)
    records = audit_source(image)
    if args.audit_linked:
        assert args.map is not None
        result = audit_linked(args.audit_linked, args.map, image)
        (args.out/'linked-audit.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps({k: v for k, v in result.items() if k not in {'symbols', 'owned_disassembly', 'copy_stub'}}, indent=2))
        return
    args.out.mkdir(parents=True, exist_ok=False)
    source, code_pages, data_pages = host_source(image)
    assembly = args.out.resolve()/'host-x86.asm'
    assembly.write_text(source)
    # Retain commands without invoking an assembler/linker or native payload.
    sdk = Path('C:/Program Files/Microsoft Visual Studio/18/Community/VC/Tools/MSVC/14.51.36231/bin/Hostx64/x86')
    kernel = 'C:/Program Files (x86)/Windows Kits/10/Lib/10.0.18362.0/um/x86/kernel32.lib'
    output = args.out.resolve()
    manifest = dict(schema='bounded-original-cell-runtime-curves-host-v2',
        source_sha256=SOURCE_SHA256, host_prepared=True, host_compiled=False, host_executed=False,
        host_image_base='24000000', preparation_tool_sha256=digest(Path(__file__).read_bytes()),
        host_source_sha256=digest(assembly.read_bytes()), code_allowlist=records,
        data_allowlist=[dict(address=f'{a:08X}', bytes=n, sha256=digest(image.read(a, n))) for a, n in DATA],
        pages=[dict(address=f'{a:08X}', bytes=65536, final_protection='RX' if a in code_pages else 'R')
               for a in sorted(code_pages | data_pages)],
        omitted_code=['0099FAF7..009A0890 (rotation and position)'],
        descriptor_policy=[dict(choice=i, flags=f'{flags:08X}', slot=slot, native_output_offset=SLOTS[slot])
                           for i, (flags, slot) in enumerate(POLICIES)],
        descriptor_proof='All24 arrays are immutable INFO0(offset0,slot0)+custom lowkind3(offset20,slot3..8),stride60. Source flags83/93/A3/B3 vary only enable10 and clamp-bypass20. INFO tag is nonzero; the sampler finds it before any row access. Custom tag is normalized to data1 because this branch never reads its tag.',
        bounds_proof='Count0..160, upper0..count. Exact unchanged99C6C0 selects and bounds four indices before full sampler call. Immutable count2, stride60, row offsets0/20 and slots0/3..8 limit every original read/store. Endpoint copies are20/40 bytes, interpolation writes INFO20 plus custom8 when enabled. Max source row ends at Input+9632 and max destination at Output+440.',
        stub=dict(address=f'{COPY:08X}', policy='Exact bounded memcpy only INFO20 to output0 or custom40 to selected output offset; source aligned to60-byte active rows. All relative branches stay inside copied stub; failure uses absolute EAX jump.'),
        guarded_buffers=[dict(label=label, bytes=size, immutable=label in IMMUTABLE) for label, size in BUFFERS],
        preservation='ESP checked before restoration; EBX/ESI/EDI/EBP and DF0 checked after each source call. Every canary, immutable shadow and unwritten output/reply byte checked before any context/reply write. x87 CW unchanged/TOP0 and MXCSR control1F80 checked; sticky flags retained.',
        protocol=dict(input_bytes=INPUT_BYTES, output_bytes=OUTPUT_BYTES, max_rows=160,
            input=['frame:f32@0 finite', 'count:u32@4(0..160)', 'duration:f32@8 finite',
                   'header_flags:u32@12(0 or8)', 'x87CW:u32@16(12 masked policies)',
                   'descriptor_choice:u32@20(0..23)', 'upper:u32@24(0..count)', 'reserved:u32@28=0',
                   'rows@32:160*(INFO20+custom40), raw source words retained'],
            output=['native destination440@0, initiallyA5', 'exact_index:i32@440', 'AL:u32@444',
                    'previous/left/right/following:u32@448/452/456/460', 'CW:u16@464', 'SW:u16@466', 'MXCSR:u32@468'],
            eof='Clean zero-byte EOF/ERROR_BROKEN_PIPE only between complete records; all partial records fail.',
            exits={'success': 0, 'protocol_or_guard': 7, 'fixed_allocation_before_first_record': 10}),
        imported_host_functions=sorted(IMPORTS), replacement_callbacks=1,
        compile_commands=[
            [str(sdk/'ml.exe'), '/nologo', '/c', '/coff', f'/Fo{output/"host-x86.obj"}', str(assembly)],
            [str(sdk/'link.exe'), '/NOLOGO', '/MACHINE:X86', '/SUBSYSTEM:CONSOLE', '/ENTRY:HostMain',
             '/BASE:0x24000000', '/DYNAMICBASE:NO', '/SAFESEH:NO', f'/OUT:{output/"host-x86.exe"}',
             f'/MAP:{output/"host-x86.map"}', str(output/'host-x86.obj'), kernel]])
    (output/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps({k: manifest[k] for k in ['schema', 'host_compiled', 'host_executed',
        'preparation_tool_sha256', 'host_source_sha256', 'protocol']}, indent=2))


if __name__ == '__main__':
    main()

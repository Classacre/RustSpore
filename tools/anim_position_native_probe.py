"""Prepare a strictly INFO+POS original-x86 ANIM conformance host.

No host execution occurs here. Quaternion/deformation code is not copied;
immutable descriptors make those dispatch paths unreachable. Every raw byte,
generated source and host binary stays under ignored target/.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys

from cell_schema_probe import SourceImage, SOURCE_SHA256
from cell_native_math_probe import db, IMPORTS


CODE = [(0x99f6a0, 0x99f8b6), (0x9a0499, 0x9a0974),
    (0x99c6c0, 0x99c772), (0x99f630, 0x99f697)]
DATA = [(0x13eb96c, 4), (0x13f4f98, 4), (0x14719e4, 4), (0x14861d0, 4),
    (0x15514cc, 8), (0x14477d8, 4), (0x14477f8, 4)]
COPY_STUB = 0x11e0534


def audit(image):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent/'target/rea/capstone-runtime'))
    from capstone import Cs, CS_ARCH_X86, CS_MODE_32, CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    decoder = Cs(CS_ARCH_X86, CS_MODE_32)
    decoder.detail = True
    records = []
    for start, end in CODE:
        decoded = list(decoder.disasm(image.read(start, end-start), start))
        assert sum(i.size for i in decoded) == end-start
        calls, forbidden, memory = [], [], []
        for instruction in decoded:
            if instruction.group(CS_GRP_CALL) or instruction.group(CS_GRP_JUMP):
                assert instruction.operands[0].type == X86_OP_IMM
                target = instruction.operands[0].imm
                if instruction.group(CS_GRP_CALL):
                    assert target in {0x99c6c0, 0x99f630, COPY_STUB}
                    calls.append(dict(at=f'{instruction.address:08X}', target=f'{target:08X}'))
                elif not any(a <= target < b for a, b in CODE):
                    assert instruction.address == 0x99f8a7 and target == 0x99faf7
                    forbidden.append(dict(at=f'{instruction.address:08X}', target=f'{target:08X}',
                        proof='Immutable INFO flags0 skip before dispatch; immutable POS flags11 take preceding type1 branch.'))
            for operand in instruction.operands:
                if operand.type != X86_OP_MEM:
                    continue
                displacement = operand.mem.disp & 0xffffffff
                if operand.mem.base == 0 or displacement in {0x14477d8, 0x15514cc}:
                    assert displacement in {a for a, _ in DATA}
                memory.append(instruction.op_str)
        records.append(dict(start=f'{start:08X}', end_exclusive=f'{end:08X}', bytes=end-start,
            sha256=hashlib.sha256(image.read(start, end-start)).hexdigest(), direct_calls=calls,
            forbidden_conditional_branches=forbidden, indirect_calls_or_jumps=0,
            memory_operands=sorted(set(memory)),
            disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in decoded]))
    assert struct.unpack('<2I', image.read(0x15514cc, 8)) == (20, 80)
    assert struct.unpack('<I', image.read(0x14477d8, 4))[0] == 0
    assert struct.unpack('<I', image.read(0x14477f8, 4))[0] == 20
    return records


def host_source(image):
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none',
        'EXTERN _GetStdHandle@4:PROC', 'EXTERN _ReadFile@20:PROC',
        'EXTERN _WriteFile@20:PROC', 'EXTERN _VirtualAlloc@16:PROC',
        'EXTERN _GetLastError@0:PROC', 'EXTERN _FlushInstructionCache@12:PROC',
        'EXTERN _ExitProcess@4:PROC', '.const',
        'Descriptors DWORD 0,04F464E49h,0,100,0,0,0,0',
        'DWORD 011h,000534F50h,20,100,1,0,0,0', '.data',
        'InputBefore DWORD 4 DUP(0A5A5A5A5h)', 'InputBuffer BYTE 1632 DUP(0)',
        'InputAfter DWORD 4 DUP(0A5A5A5A5h)',
        'OutputBefore DWORD 4 DUP(0A5A5A5A5h)', 'OutputVector DWORD 25 DUP(0A5A5A5A5h)',
        'OutputAfter DWORD 4 DUP(0A5A5A5A5h)', 'ReplyBuffer DWORD 31 DUP(0)',
        'Channel BYTE 0E4h DUP(0)', 'AnimHeader BYTE 0124h DUP(0)',
        'UpperIndex DWORD 0', 'ExactIndex DWORD 0', 'InputHandle DWORD 0',
        'OutputHandle DWORD 0', 'Transferred DWORD 0', 'NativeStack DWORD 0', 'SseControl DWORD 01F80h']
    for start, end in CODE:
        lines += db(f'Code_{start:08X}', image.read(start, end-start))
    for address, size in DATA:
        lines += db(f'Data_{address:08X}', image.read(address, size))
    lines += ['.code', 'PUBLIC CopyStub, CopyStubEnd, HostFailure', 'CopyStub PROC', 'push ebx', 'push esi', 'push edi',
        'mov ebx,dword ptr [esp+16]', 'mov esi,dword ptr [esp+20]',
        'cmp ebx,OFFSET OutputVector', 'je CopyInfo',
        'cmp ebx,OFFSET OutputVector+20', 'jne HostFailure',
        'cmp dword ptr [esp+24],80', 'jne HostFailure',
        'mov eax,esi', 'sub eax,OFFSET InputBuffer+52', 'jb HostFailure', 'jmp CopyRow',
        'CopyInfo:', 'cmp dword ptr [esp+24],20', 'jne HostFailure',
        'mov eax,esi', 'sub eax,OFFSET InputBuffer+32', 'jb HostFailure', 'CopyRow:',
        'xor edx,edx', 'mov ecx,100', 'div ecx', 'test edx,edx', 'jnz HostFailure',
        'cmp eax,dword ptr [InputBuffer+8]', 'jae HostFailure',
        'mov edi,ebx', 'mov ecx,dword ptr [esp+24]', 'rep movsb', 'mov eax,ebx',
        'pop edi', 'pop esi', 'pop ebx', 'ret', 'CopyStub ENDP', 'CopyStubEnd LABEL BYTE',
        'ReadRecord PROC', 'push ebx', 'push esi', 'mov ebx,1632', 'mov esi,OFFSET InputBuffer',
        'ReadLoop:', 'mov Transferred,0', 'push 0', 'push OFFSET Transferred', 'push ebx',
        'push esi', 'push InputHandle', 'call _ReadFile@20', 'test eax,eax', 'jnz ReadTransferred',
        'call _GetLastError@0', 'cmp eax,109', 'jne HostFailure', 'cmp ebx,1632', 'jne HostFailure',
        'cmp Transferred,0', 'jne HostFailure', 'xor eax,eax', 'jmp ReadDone', 'ReadTransferred:',
        'mov eax,Transferred', 'test eax,eax', 'jnz ReadSome', 'cmp ebx,1632', 'jne HostFailure',
        'xor eax,eax', 'jmp ReadDone', 'ReadSome:', 'add esi,eax', 'sub ebx,eax', 'jnz ReadLoop',
        'mov eax,1', 'ReadDone:', 'pop esi', 'pop ebx', 'ret', 'ReadRecord ENDP',
        'PUBLIC HostMain', 'HostMain PROC']
    # This routine is copied to a native address: keep all relative branches
    # internal, then leave through an absolute failure target when necessary.
    copy_start, copy_end = lines.index('CopyStub PROC'), lines.index('CopyStub ENDP')
    for index in range(copy_start, copy_end):
        lines[index] = lines[index].replace('HostFailure', 'CopyFailure')
    lines[copy_end:copy_end] = ['CopyFailure:', 'mov eax,OFFSET HostFailure', 'jmp eax']
    code_pages = {a & ~0xffff for a, _ in CODE} | {COPY_STUB & ~0xffff}
    pages = code_pages | {a & ~0xffff for a, _ in DATA}
    for page in sorted(pages):
        lines += [f'push {"040h" if page in code_pages else "04h"}', 'push 03000h',
            'push 010000h', f'push 0{page:08X}h', 'call _VirtualAlloc@16',
            f'cmp eax,0{page:08X}h', 'jne HostFailure']
        if page in code_pages:
            lines += [f'mov edi,0{page:08X}h', 'mov ecx,010000h', 'mov al,0CCh', 'cld', 'rep stosb']
    lines += ['cld']
    for start, end in CODE:
        lines += [f'mov esi,OFFSET Code_{start:08X}', f'mov edi,0{start:08X}h',
            f'mov ecx,{end-start}', 'rep movsb']
    for address, size in DATA:
        lines += [f'mov esi,OFFSET Data_{address:08X}', f'mov edi,0{address:08X}h',
            f'mov ecx,{size}', 'rep movsb']
    lines += ['mov esi,OFFSET CopyStub', f'mov edi,0{COPY_STUB:08X}h',
        'mov ecx,CopyStubEnd-CopyStub', 'rep movsb',
        'push 0', 'push 0', 'push -1', 'call _FlushInstructionCache@12', 'test eax,eax',
        'jz HostFailure', 'push -10', 'call _GetStdHandle@4', 'mov InputHandle,eax',
        'push -11', 'call _GetStdHandle@4', 'mov OutputHandle,eax',
        'mov dword ptr [Channel+4],OFFSET AnimHeader',
        'mov dword ptr [Channel+0D8h],OFFSET InputBuffer+32',
        'mov dword ptr [Channel+0DCh],2', 'mov dword ptr [Channel+0E0h],OFFSET Descriptors',
        'NextRecord:', 'call ReadRecord', 'test eax,eax', 'jz HostSuccess',
        'mov eax,dword ptr [InputBuffer+8]', 'cmp eax,1', 'jb HostFailure',
        'cmp eax,16', 'ja HostFailure', 'mov dword ptr [Channel+0D4h],eax',
        'cmp dword ptr [InputBuffer+4],eax', 'ja HostFailure',
        'movzx eax,word ptr [InputBuffer+20]', 'mov edx,eax', 'and edx,0F0FFh',
        'cmp edx,07Fh', 'jne HostFailure', 'and eax,0300h', 'cmp eax,0100h', 'je HostFailure']
    for offset in [0, 12]:
        lines += [f'mov eax,dword ptr [InputBuffer+{offset}]', 'and eax,07F800000h',
            'cmp eax,07F800000h', 'je HostFailure']
    lines += ['fninit', 'fldcw word ptr [InputBuffer+20]', 'ldmxcsr SseControl',
        'mov eax,dword ptr [InputBuffer+16]', 'mov dword ptr [AnimHeader+0118h],eax',
        'mov eax,dword ptr [InputBuffer+12]', 'mov dword ptr [AnimHeader+0120h],eax',
        'mov eax,dword ptr [InputBuffer+4]', 'mov UpperIndex,eax',
        'mov ExactIndex,0DEADBEEFh', 'mov edi,OFFSET OutputVector',
        'mov ecx,25', 'mov eax,0A5A5A5A5h', 'rep stosd',
        'mov NativeStack,esp', 'and esp,0FFFFFFF0h', 'sub esp,12',
        'push OFFSET ReplyBuffer+120', 'push OFFSET ReplyBuffer+112',
        'push OFFSET ReplyBuffer+108', 'push OFFSET UpperIndex', 'push OFFSET Channel',
        'mov eax,0099C6C0h', 'call eax', 'mov esp,NativeStack',
        'mov eax,UpperIndex', 'mov dword ptr [ReplyBuffer+116],eax',
        'mov NativeStack,esp', 'and esp,0FFFFFFF0h', 'sub esp,12',
        'push OFFSET ExactIndex', 'push OFFSET OutputVector', 'push OFFSET Channel',
        'push dword ptr [InputBuffer+4]', 'push dword ptr [InputBuffer]',
        'mov eax,0099F6A0h', 'call eax', 'mov esp,NativeStack', 'movzx ebx,al',
        'mov dword ptr [ReplyBuffer+104],ebx', 'mov eax,ExactIndex',
        'mov dword ptr [ReplyBuffer+100],eax']
    for label in ['OutputBefore', 'OutputAfter', 'InputBefore', 'InputAfter']:
        for offset in range(0, 16, 4):
            lines += [f'cmp dword ptr [{label}+{offset}],0A5A5A5A5h', 'jne HostFailure']
    lines += ['mov esi,OFFSET OutputVector', 'mov edi,OFFSET ReplyBuffer', 'mov ecx,25',
        'rep movsd', 'push 0', 'push OFFSET Transferred', 'push 124', 'push OFFSET ReplyBuffer',
        'push OutputHandle', 'call _WriteFile@20', 'test eax,eax', 'jz HostFailure',
        'cmp Transferred,124', 'jne HostFailure', 'jmp NextRecord',
        'HostSuccess:', 'push 0', 'call _ExitProcess@4',
        'HostFailure::', 'push 7', 'call _ExitProcess@4', 'HostMain ENDP', 'END HostMain']
    return '\n'.join(lines)+'\n', sorted(code_pages), sorted(pages)


def audit_copy_stub(exe, map_path):
    from capstone import Cs, CS_ARCH_X86, CS_MODE_32, CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM
    import re
    symbols = {}
    for line in map_path.read_text().splitlines():
        match = re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(CopyStub|CopyStubEnd|HostFailure)\s+([0-9A-Fa-f]{8})\b', line)
        if match:
            symbols[match[1]] = int(match[2], 16)
    assert set(symbols) == {'CopyStub', 'CopyStubEnd', 'HostFailure'}, symbols
    raw = exe.read_bytes()
    pe = struct.unpack_from('<I', raw, 60)[0]
    image_base = struct.unpack_from('<I', raw, pe+52)[0]
    count = struct.unpack_from('<H', raw, pe+6)[0]
    optional_size = struct.unpack_from('<H', raw, pe+20)[0]
    start, end = symbols['CopyStub'], symbols['CopyStubEnd']
    data = None
    for index in range(count):
        header = pe+24+optional_size+40*index
        _, rva, size, offset = struct.unpack_from('<4I', raw, header+8)
        if image_base+rva <= start and end <= image_base+rva+size:
            at = offset+start-image_base-rva
            data = raw[at:at+end-start]
    assert data is not None
    decoder = Cs(CS_ARCH_X86, CS_MODE_32)
    decoder.detail = True
    decoded = list(decoder.disasm(data, start))
    assert sum(i.size for i in decoded) == len(data)
    branches = []
    for index, instruction in enumerate(decoded):
        assert not instruction.group(CS_GRP_CALL)
        if not instruction.group(CS_GRP_JUMP):
            continue
        if instruction.operands[0].type == X86_OP_IMM:
            target = instruction.operands[0].imm
            assert start <= target < end
            branches.append(dict(at=f'{instruction.address:08X}', target=f'{target:08X}', internal=True))
        else:
            assert instruction.mnemonic == 'jmp' and instruction.op_str == 'eax' and index == len(decoded)-1
            previous = decoded[index-1]
            assert previous.mnemonic == 'mov' and previous.operands[1].type == X86_OP_IMM
            assert previous.operands[1].imm == symbols['HostFailure']
    return dict(source_start=f'{start:08X}', source_end_exclusive=f'{end:08X}',
        bytes=len(data), sha256=hashlib.sha256(data).hexdigest(), relative_branches=branches,
        outbound_relative_transfers=0, failure_transfer=f'absolute EAX jump to {symbols["HostFailure"]:08X}',
        disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in decoded])


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
    records = audit(image)
    source, code_pages, pages = host_source(image)
    assembly, obj, exe = output/'host-x86.asm', output/'host-x86.obj', output/'host-x86.exe'
    map_path = output/'host-x86.map'
    assembly.write_text(source)
    for command in [
        [str(args.sdk_bin_x86/'ml.exe'), '/nologo', '/c', '/coff', f'/Fo{obj}', str(assembly)],
        [str(args.sdk_bin_x86/'link.exe'), '/NOLOGO', '/MACHINE:X86', '/SUBSYSTEM:CONSOLE',
            '/ENTRY:HostMain', '/BASE:0x24000000', '/DYNAMICBASE:NO', '/SAFESEH:NO',
            f'/OUT:{exe}', f'/MAP:{map_path}', str(obj), str(args.kernel32_lib)],
    ]:
        result = subprocess.run(command, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout+'\n'+result.stderr
    copy_audit = audit_copy_stub(exe, map_path)
    manifest = dict(source_sha256=SOURCE_SHA256, host_executed=False, host_image_base='24000000',
        code_allowlist=records, omitted_code_range=['0099F8B6', '009A0499'],
        omitted_code_policy='INT3-filled; fixed read-only INFO0 and POS11 descriptors cannot dispatch to deformation/quaternion blocks.',
        descriptor_proof='99F890 tests enable: INFO0 ->9A089B. POS11 ->lowtype1 ->99F89E JZ9A0499; later type2/3 dispatch never reached.',
        descriptors=[dict(flags='00000000', tag='INFO', offset=0, stride=100, slot=0),
            dict(flags='00000011', tag='POS\\0', offset=20, stride=100, slot=1)],
        data_allowlist=[dict(address=f'{a:08X}', bytes=n, sha256=hashlib.sha256(image.read(a,n)).hexdigest()) for a,n in DATA],
        stub=dict(address=f'{COPY_STUB:08X}', behavior='Checked exact memcpy only INFO20->output0 or POS80->output20 from aligned in-range rows, return destination.', audit=copy_audit),
        imported_host_functions=IMPORTS,
        pages=[dict(address=f'{a:08X}', bytes=65536, protection='RWX' if a in code_pages else 'RW') for a in pages],
        protocol=dict(input_bytes=1632, output_bytes=124, max_rows=16,
            input_fields=['frame:f32@0','upper:u32@4','count:u32@8 (1..16)','duration:f32@12',
                'header_flags:u32@16','x87_CW:u16@20 (12 masked contexts only)','reserved@22..31',
                'rows@32:16*(INFO20+POS80), only count active'],
            output_fields=['native INFO20+POS80@0 (output initialized A5 bytes)','exact_index:i32@100','native_bool:u32@104',
                'previous:u32@108','left:u32@112','right:u32@116','following:u32@120']),
        host_source_sha256=hashlib.sha256(assembly.read_bytes()).hexdigest(),
        host_exe_sha256=hashlib.sha256(exe.read_bytes()).hexdigest())
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2))
    print(json.dumps({key:manifest[key] for key in ['source_sha256','host_executed',
        'host_source_sha256','host_exe_sha256','descriptor_proof']},indent=2))


if __name__ == '__main__':
    main()

"""Prepare, never execute, complete E563F0/E51760/E833B0 in an owned host.

The only provider is a checked fixed manager-handle resolution. Original game
initialization, actor construction, event dispatch and graphics are excluded.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import sys

from cell_native_math_probe import db
from cell_schema_probe import SourceImage, SOURCE_SHA256
from native_crt_acos_probe import PE, decoder

CODE = [(0xE51760, 0xE5178C), (0xE563F0, 0xE56436), (0xE833B0, 0xE83413)]
EPSILON = (0x1447BB8, 8)
MANAGER_GLOBAL, PROVIDER = 0x16B4BE8, 0xB71D00
INPUT_BYTES, OUTPUT_BYTES = 200, 28
BUFFERS = [('Input', 200), ('Actor', 0x250), ('Reference', 0x28),
           ('Object', 0x180), ('Owner', 0x2E8), ('Pose', 0x144),
           ('Manager', 0x16C), ('Reply', 28)]
IMPORTS = {'GetStdHandle', 'ReadFile', 'WriteFile', 'VirtualAlloc',
           'VirtualProtect', 'GetLastError', 'FlushInstructionCache', 'ExitProcess'}


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def audit_source(image):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    assert image.read(*EPSILON).hex() == '3a8c30e28e79453e'
    bodies = [list(decoder().disasm(image.read(a, b-a), a)) for a, b in CODE]
    boundaries = {i.address for rows in bodies for i in rows}
    result = []
    for (start, end), rows in zip(CODE, bodies):
        assert sum(i.size for i in rows) == end-start and rows[-1].mnemonic == 'ret'
        transfers, absolute = [], []
        for i in rows:
            if i.group(CS_GRP_CALL) or i.group(CS_GRP_JUMP):
                assert i.operands[0].type == X86_OP_IMM
                target = i.operands[0].imm
                assert target in boundaries or (i.address, target) == (0xE51783, PROVIDER)
                transfers.append(dict(at=f'{i.address:08X}', operation=i.mnemonic, target=f'{target:08X}'))
            for op in i.operands:
                if op.type == X86_OP_MEM and not op.mem.base and not op.mem.index:
                    assert (op.mem.disp, op.size) in [EPSILON, (MANAGER_GLOBAL, 4)]
                    absolute.append(dict(at=f'{i.address:08X}', address=f'{op.mem.disp:08X}', bytes=op.size))
        result.append(dict(start=f'{start:08X}', end_exclusive=f'{end:08X}', bytes=end-start,
            sha256=digest(image.read(start, end-start)), transfers=transfers,
            absolute_operands=absolute, indirect_transfers=0,
            disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in rows]))
    return result


def host_source(image):
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none']
    for name, n in [('GetStdHandle', 4), ('ReadFile', 20), ('WriteFile', 20),
                    ('VirtualAlloc', 16), ('VirtualProtect', 16), ('GetLastError', 0),
                    ('FlushInstructionCache', 12), ('ExitProcess', 4)]:
        lines += [f'EXTERN _{name}@{n}:PROC']
    lines += ['.const']
    for a, b in CODE:
        lines += [f'PUBLIC Source_{a:08X}'] + db(f'Source_{a:08X}', image.read(a, b-a))
    lines += ['PUBLIC SourceEpsilon'] + db('SourceEpsilon', image.read(*EPSILON))
    lines += ['.data']
    for label, size in BUFFERS:
        lines += [f'PUBLIC {label}Buffer', f'{label}Before DWORD 4 DUP(0A5A5A5A5h)',
                  f'{label}Buffer BYTE {size} DUP(0)', f'{label}After DWORD 4 DUP(0A5A5A5A5h)']
        if label != 'Reply':
            lines += [f'{label}Shadow BYTE {size} DUP(0)']
    lines += ['InputHandle DWORD 0', 'OutputHandle DWORD 0', 'Transferred DWORD 0',
        'OldProtect DWORD 0', 'NativeStack DWORD 0', 'ManagerCalls DWORD 0',
        'SseControl DWORD 01F80h', '.code',
        'PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,ReadRecord,ManagerProvider,ProviderJump',
        'OwnedStart LABEL BYTE', 'ProviderJump LABEL BYTE',
        'mov eax,OFFSET ManagerProvider', 'jmp eax',
        'ManagerProvider PROC', 'cmp dword ptr [esp],00E51788h', 'jne HostFailure',
        'cmp ecx,OFFSET ManagerBuffer+0168h', 'jne HostFailure',
        'mov eax,dword ptr [InputBuffer+8]', 'cmp eax,dword ptr [esp+4]', 'jne HostFailure',
        'cmp ManagerCalls,0', 'jne HostFailure', 'inc ManagerCalls',
        'mov eax,OFFSET ReferenceBuffer', 'ret 4', 'ManagerProvider ENDP',
        'ReadRecord PROC', 'push ebx', 'push esi', 'mov ebx,200',
        'mov esi,OFFSET InputBuffer', 'ReadLoop:', 'mov Transferred,0',
        'push 0', 'push OFFSET Transferred', 'push ebx', 'push esi',
        'push InputHandle', 'call _ReadFile@20', 'test eax,eax', 'jnz ReadTransferred',
        'call _GetLastError@0', 'cmp eax,109', 'jne HostFailure',
        'cmp ebx,200', 'jne HostFailure', 'cmp Transferred,0', 'jne HostFailure',
        'xor eax,eax', 'jmp ReadDone', 'ReadTransferred:', 'mov eax,Transferred',
        'cmp eax,ebx', 'ja HostFailure', 'test eax,eax', 'jnz ReadSome',
        'cmp ebx,200', 'jne HostFailure', 'xor eax,eax', 'jmp ReadDone',
        'ReadSome:', 'add esi,eax', 'sub ebx,eax', 'jnz ReadLoop', 'mov eax,1',
        'ReadDone:', 'pop esi', 'pop ebx', 'ret', 'ReadRecord ENDP', 'HostMain PROC']
    code_pages = {a & ~0xFFFF for a, _ in CODE} | {PROVIDER & ~0xFFFF}
    data_pages = {EPSILON[0] & ~0xFFFF, MANAGER_GLOBAL & ~0xFFFF}
    assert code_pages.isdisjoint(data_pages)
    for page in sorted(code_pages | data_pages):
        lines += ['push 04h', 'push 03000h', 'push 010000h', f'push 0{page:08X}h',
            'call _VirtualAlloc@16', f'cmp eax,0{page:08X}h', 'jne AllocationFailure']
        if page in code_pages:
            lines += [f'mov edi,0{page:08X}h', 'mov ecx,010000h', 'mov al,0CCh', 'cld', 'rep stosb']
    for a, b in CODE:
        lines += [f'mov esi,OFFSET Source_{a:08X}', f'mov edi,0{a:08X}h',
                  f'mov ecx,{b-a}', 'cld', 'rep movsb']
    lines += ['mov esi,OFFSET SourceEpsilon', f'mov edi,0{EPSILON[0]:08X}h',
        'mov ecx,8', 'rep movsb', f'mov dword ptr [0{MANAGER_GLOBAL:08X}h],OFFSET ManagerBuffer',
        'mov esi,OFFSET ProviderJump', f'mov edi,0{PROVIDER:08X}h', 'mov ecx,7', 'rep movsb']
    for page in sorted(code_pages | data_pages):
        lines += ['push OFFSET OldProtect', 'push 020h' if page in code_pages else 'push 02h',
            'push 010000h', f'push 0{page:08X}h', 'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure']
    lines += ['push 0', 'push 0', 'push -1', 'call _FlushInstructionCache@12',
        'test eax,eax', 'jz HostFailure', 'push -10', 'call _GetStdHandle@4',
        'mov InputHandle,eax', 'push -11', 'call _GetStdHandle@4', 'mov OutputHandle,eax',
        'NextRecord:', 'call ReadRecord', 'test eax,eax', 'jz HostSuccess',
        'cmp dword ptr [InputBuffer+12],1', 'ja HostFailure',
        'cmp dword ptr [InputBuffer+36],8', 'jg HostFailure',
        'mov eax,dword ptr [InputBuffer+16]', 'mov edx,eax', 'and eax,0FFFFF0FFh',
        'cmp eax,07Fh', 'jne HostFailure', 'and edx,0300h', 'cmp edx,0100h', 'je HostFailure']
    for at in range(20, 36, 4):
        lines += [f'cmp dword ptr [InputBuffer+{at}],0', 'jne HostFailure']
    for label, size in BUFFERS:
        if label != 'Input':
            lines += [f'mov edi,OFFSET {label}Buffer', f'mov ecx,{size//4}',
                      'mov eax,0A5A5A5A5h', 'cld', 'rep stosd']
    for src, dst in [(0, 0x18C), (4, 0xFC), (8, 0x248)]:
        lines += [f'mov eax,dword ptr [InputBuffer+{src}]', f'mov dword ptr [ActorBuffer+0{dst:X}h],eax']
    lines += ['xor eax,eax', 'cmp dword ptr [InputBuffer+12],0', 'je MissingObject',
        'mov eax,OFFSET ObjectBuffer', 'MissingObject:', 'mov dword ptr [ReferenceBuffer+24h],eax',
        'mov dword ptr [ObjectBuffer+17Ch],OFFSET OwnerBuffer',
        'mov dword ptr [OwnerBuffer+2E4h],OFFSET PoseBuffer',
        'mov esi,OFFSET InputBuffer+36', 'mov edi,OFFSET PoseBuffer+0A0h', 'mov ecx,41', 'cld', 'rep movsd',
        'mov ManagerCalls,0']
    for label, size in BUFFERS:
        if label != 'Reply':
            lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                      f'mov ecx,{size//4}', 'cld', 'rep movsd']
    lines += ['fninit', 'fldcw word ptr [InputBuffer+16]', 'ldmxcsr SseControl',
        'mov ebx,011223344h', 'mov esi,022334455h', 'mov edi,033445566h',
        'mov ebp,044556677h', 'mov NativeStack,esp', 'cld', 'push OFFSET ActorBuffer',
        'mov eax,00E563F0h', 'call eax', 'add esp,4', 'cmp esp,NativeStack', 'jne HostFailure',
        'cmp ebx,011223344h', 'jne HostFailure', 'cmp esi,022334455h', 'jne HostFailure',
        'cmp edi,033445566h', 'jne HostFailure', 'cmp ebp,044556677h', 'jne HostFailure',
        'pushfd', 'pop eax', 'test eax,0400h', 'jnz HostFailure']
    for label, size in BUFFERS:
        for side in ['Before', 'After']:
            for at in range(0, 16, 4):
                lines += [f'cmp dword ptr [{label}{side}+{at}],0A5A5A5A5h', 'jne HostFailure']
        if label != 'Reply':
            lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                      f'mov ecx,{size//4}', 'cld', 'repe cmpsd', 'jne HostFailure']
    for at in range(0, OUTPUT_BYTES, 4):
        lines += [f'cmp dword ptr [ReplyBuffer+{at}],0A5A5A5A5h', 'jne HostFailure']
    for a, b in CODE:
        lines += [f'mov esi,OFFSET Source_{a:08X}', f'mov edi,0{a:08X}h', f'mov ecx,{b-a}',
                  'cld', 'repe cmpsb', 'jne HostFailure']
    lines += ['mov esi,OFFSET SourceEpsilon', f'mov edi,0{EPSILON[0]:08X}h',
        'mov ecx,8', 'repe cmpsb', 'jne HostFailure',
        f'cmp dword ptr [0{MANAGER_GLOBAL:08X}h],OFFSET ManagerBuffer', 'jne HostFailure',
        'fnstsw ax', 'and eax,03800h', 'cmp eax,03800h', 'jne HostFailure',
        'fstp tbyte ptr [ReplyBuffer]', 'mov word ptr [ReplyBuffer+10],0',
        'fld tbyte ptr [ReplyBuffer]', 'fstp dword ptr [ReplyBuffer+12]',
        'mov eax,ManagerCalls', 'cmp eax,1', 'ja HostFailure', 'mov dword ptr [ReplyBuffer+16],eax',
        'fnstcw word ptr [ReplyBuffer+20]', 'fnstsw word ptr [ReplyBuffer+22]',
        'stmxcsr dword ptr [ReplyBuffer+24]', 'mov ax,word ptr [InputBuffer+16]',
        'cmp ax,word ptr [ReplyBuffer+20]', 'jne HostFailure',
        'test word ptr [ReplyBuffer+22],03800h', 'jnz HostFailure',
        'mov eax,dword ptr [ReplyBuffer+24]', 'and eax,0FFFFFFC0h', 'cmp eax,01F80h', 'jne HostFailure',
        'push 0', 'push OFFSET Transferred', 'push 28', 'push OFFSET ReplyBuffer',
        'push OutputHandle', 'call _WriteFile@20', 'test eax,eax', 'jz HostFailure',
        'cmp Transferred,28', 'jne HostFailure', 'jmp NextRecord',
        'HostSuccess:', 'push 0', 'call _ExitProcess@4', 'HostFailure::',
        'push 7', 'call _ExitProcess@4', 'AllocationFailure:', 'push 10',
        'call _ExitProcess@4', 'HostMain ENDP', 'OwnedEnd LABEL BYTE', 'END HostMain']
    return '\n'.join(lines)+'\n', code_pages, data_pages


def audit_linked(path, mapping, image):
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_REG
    pe, symbols = PE(path), {}
    for line in mapping.read_text().splitlines():
        m = re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\w+)\s+([0-9A-Fa-f]{8})\b', line)
        if m:
            symbols[m[1]] = int(m[2], 16)
    assert pe.base == 0x24000000
    imports = pe.imports()
    assert {n for _, n in imports.values()} == IMPORTS
    assert {dll.upper() for dll, _ in imports.values()} == {'KERNEL32.DLL'}
    start, end = symbols['OwnedStart'], symbols['OwnedEnd']
    rows = list(decoder().disasm(pe.read(start, end-start), start))
    assert sum(i.size for i in rows) == end-start
    boundaries = {i.address for i in rows}
    for n, i in enumerate(rows):
        assert i.mnemonic not in {'int', 'syscall', 'sysenter', 'hlt', 'iret', 'iretd'}
        if i.group(CS_GRP_JUMP):
            if i.operands[0].type == X86_OP_IMM:
                assert i.operands[0].imm in boundaries
            else:
                assert i.address == symbols['ProviderJump']+5 and i.op_str == 'eax'
                assert rows[n-1].operands[1].imm == symbols['ManagerProvider']
        if i.group(CS_GRP_CALL):
            if i.operands[0].type == X86_OP_REG:
                assert i.op_str == 'eax' and rows[n-1].mnemonic == 'mov'
                assert rows[n-1].operands[1].imm == 0xE563F0
            else:
                assert i.operands[0].type == X86_OP_IMM
                target = i.operands[0].imm
                if target in boundaries:
                    assert target == symbols['ReadRecord']
                else:
                    thunk = pe.read(target, 6)
                    assert thunk[:2] == b'\xFF\x25' and struct.unpack('<I', thunk[2:])[0] in imports
    at = struct.unpack_from('<I', pe.data, 0x3C)[0]
    count = struct.unpack_from('<H', pe.data, at+6)[0]
    opt = struct.unpack_from('<H', pe.data, at+20)[0]
    sections = []
    for n in range(count):
        row = at+24+opt+n*40
        size, rva = struct.unpack_from('<II', pe.data, row+8)
        sections.append((pe.base+rva, pe.base+rva+size, struct.unpack_from('<I', pe.data, row+36)[0]))
    def permission(address, size, expected):
        flags = next(f for a, b, f in sections if a <= address and address+size <= b)
        assert flags & 0xE0000000 == expected
    permission(start, end-start, 0x60000000)
    spans = []
    for label, size in BUFFERS:
        a = symbols[label+'Buffer']
        permission(a-16, size+32, 0xC0000000)
        assert all(a+size+16 <= low or high <= a-16 for low, high in spans)
        spans.append((a-16, a+size+16))
    for a, b in CODE:
        permission(symbols[f'Source_{a:08X}'], b-a, 0x40000000)
        assert pe.read(symbols[f'Source_{a:08X}'], b-a) == image.read(a, b-a)
    permission(symbols['SourceEpsilon'], 8, 0x40000000)
    assert pe.read(symbols['SourceEpsilon'], 8) == image.read(*EPSILON)
    assert pe.read(symbols['ProviderJump'], 7) == b'\xB8'+struct.pack('<I', symbols['ManagerProvider'])+b'\xFF\xE0'
    return dict(host_exe_sha256=digest(path.read_bytes()),
        owned_code_sha256=digest(pe.read(start, end-start)),
        symbols={k: f'{v:08X}' for k, v in symbols.items() if not k.startswith('_')},
        imports=[dict(iat=f'{a:08X}', dll=d, name=n) for a, (d, n) in imports.items()],
        source_code_constants_R_nonexecutable=True, owned_code_RX=True,
        guarded_buffers_RW_nonexecutable=True, source_payloads_exact=True,
        disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in rows])


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--source-exe', type=Path, default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    p.add_argument('--out', type=Path, required=True)
    p.add_argument('--audit-linked', type=Path)
    p.add_argument('--map', type=Path)
    args = p.parse_args()
    root = Path(__file__).resolve().parent.parent
    args.out.resolve().relative_to((root/'target').resolve())
    image = SourceImage(args.source_exe)
    records = audit_source(image)
    if args.audit_linked:
        assert args.map
        report = audit_linked(args.audit_linked, args.map, image)
        (args.out/'linked-audit.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({k: v for k, v in report.items() if k not in {'symbols', 'disassembly'}}, indent=2))
        return
    args.out.mkdir(parents=True, exist_ok=False)
    assembly, code_pages, data_pages = host_source(image)
    out = args.out.resolve()
    (out/'host-x86.asm').write_text(assembly)
    sdk = Path('C:/Program Files/Microsoft Visual Studio/18/Community/VC/Tools/MSVC/14.51.36231/bin/Hostx64/x86')
    kernel = 'C:/Program Files (x86)/Windows Kits/10/Lib/10.0.18362.0/um/x86/kernel32.lib'
    report = dict(schema='bounded-original-cell-data2-consumer-host-v1',
        source_path=str(args.source_exe.resolve()), source_sha256=SOURCE_SHA256,
        host_compiled=False, host_executed=False, preparation_tool_sha256=digest(Path(__file__).read_bytes()),
        host_source_sha256=digest((out/'host-x86.asm').read_bytes()), code_allowlist=records,
        epsilon_sha256=digest(image.read(*EPSILON)),
        pages=[dict(address=f'{a:08X}', bytes=65536, final='RX' if a in code_pages else 'R')
               for a in sorted(code_pages | data_pages)],
        protocol=dict(input_bytes=200, output_bytes=28, modeled_bytes=20,
            input='state18C:i32@0,FC:u32@4,handle248:u32@8,resolvedObjectPresent:bool32@12,CW:u32@16,reserved20..32=0,aggregate164@36',
            output='retained80@0,reservedZero:u16@10,FST32@12,managerCalls:u32@16,CW:u16@20,SW:u16@22,MXCSR:u32@24'),
        provider='Only B71D00: checks returnE51788,manager+168,raw handle,at mostonecall; returns fixed owned Reference+24 object/zero and RET4. No actor construction or actual manager map is inferred.',
        bounds='Signed aggregatecount<=8 (nonpositive scan none), all pointers owned; raw floats unrestricted. All source writes are to private call-stack argument. Full input/actor/reference/object/owner/pose/manager shadows and every canary/reply byte checked before serialization. NativeESP,EBX/ESI/EDI/EBP,DF0,returnTOP1/finalTOP0,CW andMXCSR1F80 checked.',
        execution_scope='Preparation/linked audit only. No original module, initializer, game, GUI or network execution. Root and peer review required before hidden source-kernel execution.',
        compile_commands=[
            [str(sdk/'ml.exe'), '/nologo', '/c', '/coff', f'/Fo{out/"host-x86.obj"}', str(out/'host-x86.asm')],
            [str(sdk/'link.exe'), '/NOLOGO', '/MACHINE:X86', '/SUBSYSTEM:CONSOLE', '/ENTRY:HostMain',
             '/BASE:0x24000000', '/DYNAMICBASE:NO', '/SAFESEH:NO', f'/OUT:{out/"host-x86.exe"}',
             f'/MAP:{out/"host-x86.map"}', str(out/'host-x86.obj'), kernel]])
    (out/'manifest.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k: report[k] for k in ['schema', 'host_source_sha256', 'host_compiled', 'host_executed']}, indent=2))


if __name__ == '__main__':
    main()

"""Prepare, never execute, bounded original32-slot capability preparation."""
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
CODE = [(0x9FC210, 0x9FC2E8), (0x9FC2F0, 0x9FC3DA),
        (0x9B2DA0, 0x9B2E11), (0x9B2E20, 0x9B2E5C)]
INPUT_BYTES, OUTPUT_BYTES = 1264, 192
GLOBAL, MAP, NODES, MEMBERS = 0x166D060, 0x1660000, 0x1660010, 0x1660100


def audit_source(image):
    sys.path.insert(0, str(ROOT/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    boundaries = set()
    decoded = []
    for start, end in CODE:
        code = list(decoder().disasm(image.read(start, end-start), start))
        assert sum(i.size for i in code) == end-start
        boundaries.update(i.address for i in code)
        decoded.append(code)
    records = []
    for (start, end), code in zip(CODE, decoded):
        transfers = []
        for i in code:
            assert not any(name.startswith(('xmm', 'st(')) for name in
                           [i.reg_name(r) for r in i.regs_access()[0]+i.regs_access()[1]])
            if i.group(CS_GRP_CALL) or i.group(CS_GRP_JUMP):
                assert i.operands[0].type == X86_OP_IMM
                assert i.operands[0].imm in boundaries
                transfers.append(dict(at=f'{i.address:08X}', target=f'{i.operands[0].imm:08X}',
                                      kind='call' if i.group(CS_GRP_CALL) else 'jump'))
            for o in i.operands:
                if o.type == X86_OP_MEM and not o.mem.base:
                    assert not o.mem.index and o.mem.disp == GLOBAL
        records.append(dict(start=f'{start:08X}', end_exclusive=f'{end:08X}', bytes=end-start,
            sha256=hashlib.sha256(image.read(start, end-start)).hexdigest(), transfers=transfers,
            disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in code]))
    return records


def host_source(image):
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none']
    for name, size in [('GetStdHandle', 4), ('ReadFile', 20), ('WriteFile', 20),
                       ('VirtualAlloc', 16), ('VirtualProtect', 16), ('GetLastError', 0),
                       ('FlushInstructionCache', 12), ('ExitProcess', 4)]:
        lines += [f'EXTERN _{name}@{size}:PROC']
    lines += ['.const']
    for a, b in CODE:
        lines += db(f'Original_{a:08X}', image.read(a, b-a))
    lines += ['.data']
    for label, size in [('Input', INPUT_BYTES), ('Block', 168), ('Validation', 128), ('Reply', OUTPUT_BYTES)]:
        lines += [f'{label}Before DWORD 4 DUP(0A5A5A5A5h)',
                  f'{label}Buffer BYTE {size} DUP(0)', f'{label}After DWORD 4 DUP(0A5A5A5A5h)']
    lines += ['InputShadow BYTE 1264 DUP(0)', 'Expected BYTE 168 DUP(0)',
        'InputHandle DWORD 0', 'OutputHandle DWORD 0', 'Transferred DWORD 0',
        'OldProtect DWORD 0', 'NativeStack DWORD 0', 'AppendAt DWORD 0',
        'ScanAt DWORD 0', 'Poison DWORD 0', 'SseControl DWORD 01F80h', '.code',
        'PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,ReadRecord,ValidateRecord,RunOnce',
        'OwnedStart LABEL BYTE', 'ReadRecord PROC', 'push ebx', 'push esi',
        f'mov ebx,{INPUT_BYTES}', 'mov esi,OFFSET InputBuffer', 'ReadLoop:',
        'mov Transferred,0', 'push 0', 'push OFFSET Transferred', 'push ebx', 'push esi',
        'push InputHandle', 'call _ReadFile@20', 'test eax,eax', 'jnz ReadTransferred',
        'call _GetLastError@0', 'cmp eax,109', 'jne HostFailure',
        f'cmp ebx,{INPUT_BYTES}', 'jne HostFailure', 'cmp Transferred,0', 'jne HostFailure',
        'xor eax,eax', 'jmp ReadDone', 'ReadTransferred:', 'mov eax,Transferred',
        'test eax,eax', 'jnz ReadSome', f'cmp ebx,{INPUT_BYTES}', 'jne HostFailure',
        'xor eax,eax', 'jmp ReadDone', 'ReadSome:', 'add esi,eax', 'sub ebx,eax',
        'jnz ReadLoop', 'mov eax,1', 'ReadDone:', 'pop esi', 'pop ebx', 'ret', 'ReadRecord ENDP',
        'ValidateRecord PROC', 'movzx eax,word ptr [InputBuffer]', 'mov edx,eax',
        'and eax,0F0FFh', 'cmp eax,07Fh', 'jne HostFailure',
        'and edx,0300h', 'cmp edx,0100h', 'je HostFailure',
        'cmp word ptr [InputBuffer+2],0', 'jne HostFailure',
        'cmp dword ptr [InputBuffer+4],8', 'ja HostFailure',
        'cmp dword ptr [InputBuffer+8],0', 'jne HostFailure',
        'cmp dword ptr [InputBuffer+12],0', 'jne HostFailure',
        'mov esi,OFFSET InputBuffer+176', 'xor ebx,ebx', 'xor edi,edi', 'ValidateTable:',
        'cmp dword ptr [esi+4],32', 'ja HostFailure',
        'cmp edi,dword ptr [InputBuffer+4]', 'jae TableKeyReady',
        'mov eax,dword ptr [esi]', 'test edi,edi', 'jz FirstKey',
        'cmp eax,ebx', 'jbe HostFailure', 'FirstKey:', 'mov ebx,eax', 'TableKeyReady:',
        'add esi,136', 'inc edi', 'cmp edi,8', 'jb ValidateTable',
        'mov esi,OFFSET InputBuffer+16', 'mov edi,OFFSET ValidationBuffer', 'mov ecx,32',
        'cld', 'rep movsd', 'xor eax,eax', 'FindInitialEnd:',
        'cmp dword ptr [ValidationBuffer+eax*4],0', 'je InitialEndFound',
        'inc eax', 'cmp eax,32', 'jb FindInitialEnd', 'jmp HostFailure',
        'InitialEndFound:', 'mov AppendAt,eax', 'mov ScanAt,0', 'ValidateExpansion:',
        'mov eax,ScanAt', 'mov ebx,dword ptr [ValidationBuffer+eax*4]',
        'test ebx,ebx', 'jz ExpansionDone', 'xor edx,edx',
        'mov esi,OFFSET InputBuffer+176', 'FindAggregate:',
        'cmp edx,dword ptr [InputBuffer+4]', 'jae NextExpansion',
        'cmp ebx,dword ptr [esi]', 'je FoundAggregate',
        'inc edx', 'add esi,136', 'jmp FindAggregate', 'FoundAggregate:',
        'mov ecx,dword ptr [esi+4]', 'mov eax,AppendAt', 'add eax,ecx',
        'cmp eax,32', 'ja HostFailure', 'mov AppendAt,eax',
        'sub eax,ecx', 'lea edi,[ValidationBuffer+eax*4]', 'add esi,8', 'rep movsd',
        'NextExpansion:', 'inc ScanAt', 'cmp ScanAt,32', 'jb ValidateExpansion',
        'ExpansionDone:', 'ret', 'ValidateRecord ENDP',
        'RunOnce PROC', 'mov edi,OFFSET BlockBuffer', 'mov eax,0A5A5A5A5h', 'mov ecx,42',
        'cld', 'rep stosd', 'mov esi,OFFSET InputBuffer+16', 'mov edi,OFFSET BlockBuffer+8',
        'mov ecx,40', 'rep movsd', 'fninit', 'fldcw word ptr [InputBuffer]', 'ldmxcsr SseControl',
        'cmp Poison,0', 'jne ScratchOnes']
    for n in range(8):
        lines += [f'pxor xmm{n},xmm{n}']
    lines += ['jmp ScratchReady', 'ScratchOnes:']
    for n in range(8):
        lines += [f'pcmpeqd xmm{n},xmm{n}']
    lines += ['ScratchReady:', 'mov NativeStack,esp', 'push OFFSET BlockBuffer',
        'mov eax,009FC2F0h', 'call eax', 'add esp,4', 'cmp esp,NativeStack', 'jne HostFailure']
    for label in ['Input', 'Block', 'Validation', 'Reply']:
        for side in ['Before', 'After']:
            for at in range(0, 16, 4):
                lines += [f'cmp dword ptr [{label}{side}+{at}],0A5A5A5A5h', 'jne HostFailure']
    lines += ['mov esi,OFFSET InputBuffer', 'mov edi,OFFSET InputShadow', 'mov ecx,316',
        'repe cmpsd', 'jne HostFailure', 'stmxcsr dword ptr [ReplyBuffer+168]',
        'fnstcw word ptr [ReplyBuffer+172]', 'fnstsw word ptr [ReplyBuffer+174]',
        'mov ax,word ptr [InputBuffer]', 'cmp ax,word ptr [ReplyBuffer+172]', 'jne HostFailure',
        'cmp dword ptr [ReplyBuffer+168],01F80h', 'jne HostFailure',
        'cmp word ptr [ReplyBuffer+174],0', 'jne HostFailure',
        'mov esi,OFFSET BlockBuffer', 'mov edi,OFFSET ReplyBuffer', 'mov ecx,42', 'rep movsd',
        'mov dword ptr [ReplyBuffer+176],0', 'mov dword ptr [ReplyBuffer+180],0',
        'mov dword ptr [ReplyBuffer+184],0', 'mov dword ptr [ReplyBuffer+188],0',
        'ret', 'RunOnce ENDP', 'HostMain PROC']
    pages = {a & ~0xFFFF for a, _ in CODE} | {MAP}
    for page in sorted(pages):
        lines += ['push 04h', 'push 03000h', 'push 010000h', f'push 0{page:08X}h',
            'call _VirtualAlloc@16', f'cmp eax,0{page:08X}h', 'jne AllocationFailure']
        if page != MAP:
            lines += [f'mov edi,0{page:08X}h', 'mov ecx,010000h', 'mov al,0CCh', 'cld', 'rep stosb']
    for a, b in CODE:
        lines += [f'mov esi,OFFSET Original_{a:08X}', f'mov edi,0{a:08X}h',
            f'mov ecx,{b-a}', 'rep movsb']
    for page in sorted(pages-{MAP}):
        lines += ['push OFFSET OldProtect', 'push 020h', 'push 010000h',
            f'push 0{page:08X}h', 'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure']
    lines += ['push 0', 'push 0', 'push -1', 'call _FlushInstructionCache@12',
        'test eax,eax', 'jz HostFailure', 'push -10', 'call _GetStdHandle@4', 'mov InputHandle,eax',
        'push -11', 'call _GetStdHandle@4', 'mov OutputHandle,eax', 'NextRecord:',
        'call ReadRecord', 'test eax,eax', 'jz HostSuccess', 'call ValidateRecord',
        'mov esi,OFFSET InputBuffer', 'mov edi,OFFSET InputShadow', 'mov ecx,316', 'cld', 'rep movsd',
        'push OFFSET OldProtect', 'push 04h', 'push 010000h', f'push 0{MAP:08X}h',
        'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure',
        f'mov edi,0{MAP:08X}h', 'xor eax,eax', 'mov ecx,60', 'rep stosd',
        f'mov dword ptr [0{GLOBAL:08X}h],0{MAP:08X}h', 'cmp dword ptr [InputBuffer+4],0',
        'je EmptyTree', f'mov dword ptr [0{MAP+12:08X}h],0{NODES:08X}h', 'EmptyTree:',
        'mov esi,OFFSET InputBuffer+176', f'mov edi,0{NODES:08X}h', 'xor ebx,ebx', 'BuildNodes:',
        'cmp ebx,dword ptr [InputBuffer+4]', 'jae NodesReady',
        'mov eax,ebx', 'inc eax', 'cmp eax,dword ptr [InputBuffer+4]', 'jae LastNode',
        'imul eax,eax,28', f'add eax,0{NODES:08X}h', 'mov dword ptr [edi],eax', 'LastNode:',
        'mov eax,dword ptr [esi]', 'mov dword ptr [edi+16],eax',
        'mov eax,ebx', 'shl eax,7', f'add eax,0{MEMBERS:08X}h', 'mov dword ptr [edi+20],eax',
        'mov ecx,dword ptr [esi+4]', 'lea edx,[eax+ecx*4]', 'mov dword ptr [edi+24],edx',
        'push edi', 'mov edi,eax', 'push esi', 'add esi,8', 'rep movsd', 'pop esi', 'pop edi',
        'add esi,136', 'add edi,28', 'inc ebx', 'jmp BuildNodes', 'NodesReady:',
        'push OFFSET OldProtect', 'push 02h', 'push 010000h', f'push 0{MAP:08X}h',
        'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure',
        'mov Poison,0', 'call RunOnce', 'mov esi,OFFSET ReplyBuffer', 'mov edi,OFFSET Expected',
        'mov ecx,42', 'rep movsd', 'mov Poison,1', 'call RunOnce',
        'mov esi,OFFSET ReplyBuffer', 'mov edi,OFFSET Expected', 'mov ecx,42', 'repe cmpsd', 'jne HostFailure',
        'push 0', 'push OFFSET Transferred', f'push {OUTPUT_BYTES}', 'push OFFSET ReplyBuffer',
        'push OutputHandle', 'call _WriteFile@20', 'test eax,eax', 'jz HostFailure',
        f'cmp Transferred,{OUTPUT_BYTES}', 'jne HostFailure', 'jmp NextRecord',
        'HostSuccess:', 'push 0', 'call _ExitProcess@4', 'HostFailure::', 'push 7', 'call _ExitProcess@4',
        'AllocationFailure:', 'push 10', 'call _ExitProcess@4', 'HostMain ENDP',
        'OwnedEnd LABEL BYTE', 'END HostMain']
    return '\n'.join(lines)+'\n'


def linked_audit(exe, map_path):
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_REG
    pe, symbols = PE(exe), {}
    for line in map_path.read_text().splitlines():
        match = re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\w+)\s+([0-9A-Fa-f]{8})\b', line)
        if match:
            symbols[match[1]] = int(match[2], 16)
    imports = pe.imports()
    assert {n for _, n in imports.values()} == IMPORTS
    assert {n.upper() for n, _ in imports.values()} == {'KERNEL32.DLL'}
    start, end = symbols['OwnedStart'], symbols['OwnedEnd']
    code = list(decoder().disasm(pe.read(start, end-start), start))
    assert sum(i.size for i in code) == end-start
    boundaries = {i.address for i in code}
    transfers = []
    for n, i in enumerate(code):
        assert i.mnemonic not in {'int', 'syscall', 'sysenter', 'hlt', 'iret', 'iretd'}
        if i.group(CS_GRP_JUMP):
            assert i.operands[0].type == X86_OP_IMM and i.operands[0].imm in boundaries
        if i.group(CS_GRP_CALL):
            op = i.operands[0]
            if op.type == X86_OP_IMM:
                if op.imm in boundaries:
                    assert op.imm in {symbols[k] for k in ['ReadRecord', 'ValidateRecord', 'RunOnce']}
                else:
                    thunk = pe.read(op.imm, 6)
                    assert thunk[:2] == b'\xFF\x25' and struct.unpack('<I', thunk[2:])[0] in imports
            else:
                assert op.type == X86_OP_REG and i.op_str == 'eax'
                previous = code[n-1]
                assert previous.mnemonic == 'mov' and previous.operands[1].imm == 0x9FC2F0
            transfers.append(f'{i.address:08X}: {i.mnemonic} {i.op_str}')
    return dict(owned_start=f'{start:08X}', owned_end_exclusive=f'{end:08X}',
        symbols={k: f'{v:08X}' for k, v in symbols.items() if not k.startswith('_')},
        owned_code_sha256=hashlib.sha256(pe.read(start, end-start)).hexdigest(),
        imports=[dict(iat=f'{a:08X}', dll=dll, name=name) for a, (dll, name) in imports.items()],
        calls=transfers, disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in code])


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('exe', type=Path)
    p.add_argument('--sdk-bin-x86', type=Path, required=True)
    p.add_argument('--kernel32-lib', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    a.out.resolve().relative_to((ROOT/'target').resolve())
    image = SourceImage(a.exe)
    records, text = audit_source(image), host_source(image)
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
    linked = linked_audit(exe, mapping)
    manifest = dict(schema='bounded-original-body-capabilities-host-v1', host_executed=False,
        source_path=str(a.exe.resolve()), source_sha256=SOURCE_SHA256, code_allowlist=records,
        host_source_sha256=hashlib.sha256(asm.read_bytes()).hexdigest(),
        host_exe_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(), linked_audit=linked,
        compile_commands=commands, input_bytes=INPUT_BYTES, output_bytes=OUTPUT_BYTES,
        input='CW:u16@0,reserved:u16@2,aggregateCount:u32@4(0..8),reserved0@8/12,32IDs@16,32numbers@144,8records136bytes@176:key,count(0..32),32members.',
        output='native capability block168bytes (presence/nonzero masks,32IDs,32numbers);MXCSR@168,CW/SW@172/174,reserved0@176..191.',
        capacity_proof='Own validation scans changing32-ID array with exact first-zero append rule and bounds every entire member vector before writes. Full initial32 without zero is rejected before original undefined append-index load9FC230. Source raw initial count<=8; exact expanded32 remains admitted.',
        tree_proof='One fixed64KiB source-global page. Map1660000,8nodes at1660010 stride28,right child onlynext active node,leftNULL,unsignedsortedunique keys,member arrays1660100+i*128,extent0..32. Lookup can visit<=8acyclic nodes; sentinelmap+4. Entire page becomes R before every original call, RW only for next owned configuration.',
        memory_proof='Input1264,Block168,Validation128,Reply192 have8separate16-byte canaries. Original receives only Block pointer. Original calls/branches close within four exact spans; no imports/indirect native transfers. All used pointers are owned/fixed; every input byte has a postcall shadow check before result/context stores.',
        state_proof='Two original calls with zero/all-one XMM scratch per record compare all168native bytes; source is integer-only. Explicit12masked CWs andMXCSR1F80, exactESP,CW,zeroSW,unchangedMXCSR and canaries checked before copies.',
        pages=[dict(address=f'{page:08X}', bytes=65536, initial='RW', final='R' if page==MAP else 'RX')
               for page in sorted({a&~0xFFFF for a,_ in CODE}|{MAP})],
        execution_scope='Preparation only; original game/modules/initializers are not loaded or started. Root reviews new exact host/cohort before separate hidden execution.')
    (a.out/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps({k: manifest[k] for k in ['schema', 'host_executed', 'host_source_sha256', 'host_exe_sha256']}, indent=2))


if __name__ == '__main__':
    main()

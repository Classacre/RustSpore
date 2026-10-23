"""Prepare, never execute, bounded original E56330/E4ECC0 scaling bytes.

The explicit twenty-word scale table is copied to the original data address and
made read-only before each native entry. Original source rows/constants and
code stay hash-pinned; all owned actor/difficulty/output buffers are separate.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

from cell_native_math_probe import db
from cell_schema_probe import SOURCE_SHA256, SourceImage
from native_crt_acos_probe import PE, decoder

CODE = [(0xe4ecc0, 0xe4ecfb), (0xe56330, 0xe563e6)]
CONSTANTS = [(0x14719e4, 4), (0x1471b2c, 4), (0x14846d0, 0x248),
             (0x1485e8c, 4), (0x14861d0, 4)]
TABLE, GLOBAL = 0x1484680, 0x16b4be4
INPUT_BYTES, OUTPUT_BYTES = 112, 32
IMPORTS = {'GetStdHandle', 'ReadFile', 'WriteFile', 'VirtualAlloc', 'VirtualProtect',
           'GetLastError', 'FlushInstructionCache', 'ExitProcess'}
BUFFERS = [('Input', INPUT_BYTES), ('Actor', 0x35c), ('Root', 0x5194),
           ('Difficulty', 0x80), ('Value', 16), ('Reply', OUTPUT_BYTES)]


def audit_source(image):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    records = []
    allowed_displacements = {GLOBAL, TABLE, 0x14846c4, *[a for a, _ in CONSTANTS]}
    for start, end in CODE:
        rows = list(decoder().disasm(image.read(start, end-start), start))
        assert sum(row.size for row in rows) == end-start
        boundaries = {row.address for row in rows}
        transfers, absolute = [], []
        for ins in rows:
            if ins.group(CS_GRP_CALL) or ins.group(CS_GRP_JUMP):
                op = ins.operands[0]
                assert op.type == X86_OP_IMM
                if ins.group(CS_GRP_CALL):
                    assert (ins.address, op.imm) == (0xe56353, 0xe4ecc0)
                else:
                    assert op.imm in boundaries
                transfers.append(dict(at=f'{ins.address:08X}', operation=ins.mnemonic, target=f'{op.imm:08X}'))
            for op in ins.operands:
                if op.type == X86_OP_MEM and not op.mem.base:
                    if ins.mnemonic == 'lea' and op.mem.disp == 0:
                        assert ins.address == 0xe4ece8 and op.mem.scale == 8
                        continue
                    assert op.mem.disp in allowed_displacements
                    absolute.append(dict(at=f'{ins.address:08X}', displacement=f'{op.mem.disp:08X}',
                                         index=ins.reg_name(op.mem.index), scale=op.mem.scale))
        records.append(dict(start=f'{start:08X}', end_exclusive=f'{end:08X}', bytes=end-start,
            sha256=hashlib.sha256(image.read(start, end-start)).hexdigest(), transfers=transfers,
            indirect_transfers=0, absolute_operands=absolute,
            disassembly=[f'{ins.address:08X}: {ins.bytes.hex()} {ins.mnemonic} {ins.op_str}' for ins in rows]))
    # Index-selected original words and thresholds are audited independently,
    # including the final threshold1000 that the bounded loop can read.
    pairs = [(struct.unpack('<i', image.read(0x14846e4 + row*0x1c, 4))[0],
              struct.unpack('<I', image.read(0x14846c4 + row*0x1c, 4))[0]) for row in range(1, 21)]
    assert pairs == [(row*50, (row+1)//2) for row in range(1, 21)]
    assert struct.unpack('<I', image.read(0x1484910, 4))[0] == 10
    return records, pairs


def host_source(image):
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none']
    for name, size in [('GetStdHandle', 4), ('ReadFile', 20), ('WriteFile', 20),
                       ('VirtualAlloc', 16), ('VirtualProtect', 16), ('GetLastError', 0),
                       ('FlushInstructionCache', 12), ('ExitProcess', 4)]:
        lines += [f'EXTERN _{name}@{size}:PROC']
    lines += ['.const']
    for start, end in CODE:
        lines += db(f'Original_{start:08X}', image.read(start, end-start))
    for start, count in CONSTANTS:
        lines += db(f'Constant_{start:08X}', image.read(start, count))
    lines += ['.data']
    for label, size in BUFFERS:
        lines += [f'{label}Before DWORD 4 DUP(0A5A5A5A5h)',
                  f'{label}Buffer BYTE {size} DUP(0)', f'{label}After DWORD 4 DUP(0A5A5A5A5h)']
        if label not in {'Value', 'Reply'}:
            lines += [f'{label}Shadow BYTE {size} DUP(0)']
    lines += ['InputHandle DWORD 0', 'OutputHandle DWORD 0', 'Transferred DWORD 0',
        'OldProtect DWORD 0', 'SseControl DWORD 01F80h', 'NativeStack DWORD 0', 'CallStack DWORD 0',
        'StageIndex DWORD 0', 'Ratio DWORD 0', '.code',
        'PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure',
        'PUBLIC InputBuffer,ActorBuffer,RootBuffer,DifficultyBuffer,ValueBuffer,ReplyBuffer',
        'OwnedStart LABEL BYTE', 'CheckPreserved PROC',
        'cmp ebx,011223344h', 'jne HostFailure', 'cmp esi,022334455h', 'jne HostFailure',
        'cmp edi,033445566h', 'jne HostFailure', 'cmp ebp,044556677h', 'jne HostFailure',
        'ret', 'CheckPreserved ENDP', 'ReadRecord PROC', 'push ebx', 'push esi',
        f'mov ebx,{INPUT_BYTES}', 'mov esi,OFFSET InputBuffer', 'ReadLoop:', 'mov Transferred,0',
        'push 0', 'push OFFSET Transferred', 'push ebx', 'push esi', 'push InputHandle',
        'call _ReadFile@20', 'test eax,eax', 'jnz ReadTransferred', 'call _GetLastError@0',
        'cmp eax,109', 'jne HostFailure', f'cmp ebx,{INPUT_BYTES}', 'jne HostFailure',
        'cmp Transferred,0', 'jne HostFailure', 'xor eax,eax', 'jmp ReadDone',
        'ReadTransferred:', 'mov eax,Transferred', 'cmp eax,ebx', 'ja HostFailure',
        'test eax,eax', 'jnz ReadSome', f'cmp ebx,{INPUT_BYTES}', 'jne HostFailure',
        'xor eax,eax', 'jmp ReadDone', 'ReadSome:', 'add esi,eax', 'sub ebx,eax',
        'jnz ReadLoop', 'mov eax,1', 'ReadDone:', 'pop esi', 'pop ebx', 'ret',
        'ReadRecord ENDP', 'HostMain PROC']
    code_pages = {start & ~0xffff for start, _ in CODE}
    data_pages = {start & ~0xffff for start, _ in CONSTANTS} | {GLOBAL & ~0xffff}
    for page in sorted(code_pages | data_pages):
        lines += ['push 04h', 'push 03000h', 'push 010000h', f'push 0{page:08X}h',
            'call _VirtualAlloc@16', f'cmp eax,0{page:08X}h', 'jne AllocationFailure']
        if page in code_pages:
            lines += [f'mov edi,0{page:08X}h', 'mov ecx,010000h', 'mov al,0CCh', 'cld', 'rep stosb']
    for start, end in CODE:
        lines += [f'mov esi,OFFSET Original_{start:08X}', f'mov edi,0{start:08X}h',
                  f'mov ecx,{end-start}', 'cld', 'rep movsb']
    for start, count in CONSTANTS:
        lines += [f'mov esi,OFFSET Constant_{start:08X}', f'mov edi,0{start:08X}h',
                  f'mov ecx,{count}', 'rep movsb']
    lines += [f'mov dword ptr [0{GLOBAL:08X}h],OFFSET RootBuffer']
    for page in sorted(code_pages | data_pages):
        lines += ['push OFFSET OldProtect', 'push 020h' if page in code_pages else 'push 02h',
            'push 010000h', f'push 0{page:08X}h', 'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure']
    lines += ['push 0', 'push 0', 'push -1', 'call _FlushInstructionCache@12', 'test eax,eax', 'jz HostFailure',
        'push -10', 'call _GetStdHandle@4', 'mov InputHandle,eax',
        'push -11', 'call _GetStdHandle@4', 'mov OutputHandle,eax',
        'NextRecord:', 'call ReadRecord', 'test eax,eax', 'jz HostSuccess',
        'movzx eax,word ptr [InputBuffer+8]', 'mov edx,eax', 'and eax,0F0FFh',
        'cmp eax,07Fh', 'jne HostFailure', 'and edx,0300h', 'cmp edx,0100h', 'je HostFailure',
        'cmp word ptr [InputBuffer+10],0', 'jne HostFailure',
        'cmp dword ptr [InputBuffer+12],0', 'jne HostFailure']
    for offset in range(96, 112, 4):
        lines += [f'cmp dword ptr [InputBuffer+{offset}],0', 'jne HostFailure']
    lines += ['cmp dword ptr [InputBuffer],-1', 'je RangeValid',
        'cmp dword ptr [InputBuffer],20', 'jae HostFailure',
        'cmp dword ptr [InputBuffer+4],1000', 'ja HostFailure', 'RangeValid:',
        'push OFFSET OldProtect', 'push 04h', 'push 010000h', 'push 01480000h',
        'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure',
        'mov esi,OFFSET InputBuffer+16', f'mov edi,0{TABLE:08X}h', 'mov ecx,20', 'cld', 'rep movsd',
        'push OFFSET OldProtect', 'push 02h', 'push 010000h', 'push 01480000h',
        'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure']
    for label, size in BUFFERS:
        if label != 'Input':
            lines += [f'mov edi,OFFSET {label}Buffer', f'mov ecx,{size//4}',
                      'mov eax,0A5A5A5A5h', 'cld', 'rep stosd']
    lines += ['mov eax,dword ptr [InputBuffer]', 'mov dword ptr [ActorBuffer+0358h],eax',
        'mov dword ptr [RootBuffer+05190h],OFFSET DifficultyBuffer',
        'mov eax,dword ptr [InputBuffer+4]', 'mov dword ptr [DifficultyBuffer+01Ch],eax']
    for label, size in BUFFERS:
        if label not in {'Value', 'Reply'}:
            lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                      f'mov ecx,{size//4}', 'rep movsd']
    lines += ['mov StageIndex,0FFFFFFFFh', 'mov Ratio,0', 'fninit',
              'fldcw word ptr [InputBuffer+8]', 'ldmxcsr SseControl',
              'cmp dword ptr [InputBuffer],-1', 'je StageReady']
    def preserved():
        return ['mov ebx,011223344h', 'mov esi,022334455h', 'mov edi,033445566h', 'mov ebp,044556677h']
    lines += preserved()+['mov NativeStack,esp', 'mov edx,dword ptr [InputBuffer+4]',
        'mov eax,00E4ECC0h', 'call eax', 'cmp esp,NativeStack', 'jne HostFailure',
        'call CheckPreserved', 'cmp eax,20', 'jae HostFailure', 'mov StageIndex,eax', 'StageReady:']
    lines += preserved()+['mov NativeStack,esp', 'push OFFSET ActorBuffer', 'mov CallStack,esp',
        'mov eax,00E56330h', 'call eax', 'cmp esp,CallStack', 'jne HostFailure',
        'mov ecx,dword ptr [esp]', 'add esp,4', 'cmp esp,NativeStack', 'jne HostFailure',
        'cmp dword ptr [InputBuffer],-1', 'je RatioReady', 'mov Ratio,ecx', 'RatioReady:',
        'fst dword ptr [ValueBuffer+12]', 'fstp tbyte ptr [ValueBuffer]', 'call CheckPreserved']
    for label, size in BUFFERS:
        if label not in {'Value', 'Reply'}:
            lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                      f'mov ecx,{size//4}', 'cld', 'repe cmpsd', 'jne HostFailure']
    lines += ['mov esi,OFFSET InputBuffer+16', f'mov edi,0{TABLE:08X}h',
              'mov ecx,20', 'repe cmpsd', 'jne HostFailure']
    for start, size in CONSTANTS:
        lines += [f'mov esi,OFFSET Constant_{start:08X}', f'mov edi,0{start:08X}h',
                  f'mov ecx,{size}', 'repe cmpsb', 'jne HostFailure']
    for label, _ in BUFFERS:
        for side in ['Before', 'After']:
            for offset in range(0, 16, 4):
                lines += [f'cmp dword ptr [{label}{side}+{offset}],0A5A5A5A5h', 'jne HostFailure']
    lines += ['cmp word ptr [ValueBuffer+10],0A5A5h', 'jne HostFailure']
    for offset in range(0, OUTPUT_BYTES, 4):
        lines += [f'cmp dword ptr [ReplyBuffer+{offset}],0A5A5A5A5h', 'jne HostFailure']
    lines += ['fnstcw word ptr [ReplyBuffer+24]', 'fnstsw word ptr [ReplyBuffer+26]',
        'stmxcsr dword ptr [ReplyBuffer+28]', 'mov ax,word ptr [ReplyBuffer+24]',
        'cmp ax,word ptr [InputBuffer+8]', 'jne HostFailure',
        'test word ptr [ReplyBuffer+26],03800h', 'jnz HostFailure',
        'mov eax,dword ptr [ReplyBuffer+28]', 'and eax,0FFFFFFC0h', 'cmp eax,01F80h', 'jne HostFailure',
        'mov esi,OFFSET ValueBuffer', 'mov edi,OFFSET ReplyBuffer', 'mov ecx,4', 'cld', 'rep movsd',
        'mov word ptr [ReplyBuffer+10],0', 'mov eax,StageIndex', 'mov dword ptr [ReplyBuffer+16],eax',
        'mov eax,Ratio', 'mov dword ptr [ReplyBuffer+20],eax',
        'push 0', 'push OFFSET Transferred', f'push {OUTPUT_BYTES}', 'push OFFSET ReplyBuffer', 'push OutputHandle',
        'call _WriteFile@20', 'test eax,eax', 'jz HostFailure', f'cmp Transferred,{OUTPUT_BYTES}', 'jne HostFailure',
        'jmp NextRecord', 'HostSuccess:', 'push 0', 'call _ExitProcess@4',
        'HostFailure::', 'push 7', 'call _ExitProcess@4', 'AllocationFailure:', 'push 10', 'call _ExitProcess@4',
        'HostMain ENDP', 'OwnedEnd LABEL BYTE', 'END HostMain']
    return '\n'.join(lines)+'\n', code_pages, data_pages


def linked_audit(path, mapping):
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
    assert sum(row.size for row in rows) == end-start
    boundaries = {ins.address for ins in rows}
    for index, ins in enumerate(rows):
        assert ins.mnemonic not in {'int', 'syscall', 'sysenter', 'hlt', 'iret', 'iretd'}
        if ins.group(CS_GRP_JUMP):
            assert ins.operands[0].type == X86_OP_IMM and ins.operands[0].imm in boundaries
        if ins.group(CS_GRP_CALL):
            op = ins.operands[0]
            if op.type == X86_OP_REG:
                assert ins.op_str == 'eax'
                previous = rows[index-1]
                assert previous.mnemonic == 'mov' and previous.operands[1].type == X86_OP_IMM
                assert previous.operands[1].imm in {start for start, _ in CODE}
            else:
                assert op.type == X86_OP_IMM
                if op.imm in boundaries:
                    assert op.imm in {symbols['ReadRecord'], symbols['CheckPreserved']}
                else:
                    raw = pe.read(op.imm, 6)
                    assert raw[:2] == b'\xff\x25' and struct.unpack('<I', raw[2:])[0] in imports
    at = struct.unpack_from('<I', pe.data, 0x3c)[0]
    count = struct.unpack_from('<H', pe.data, at+6)[0]
    optional = struct.unpack_from('<H', pe.data, at+20)[0]
    sections = []
    for index in range(count):
        row = at+24+optional+40*index
        size, rva = struct.unpack_from('<II', pe.data, row+8)
        sections.append((pe.base+rva, pe.base+rva+size, struct.unpack_from('<I', pe.data, row+36)[0]))
    for label in ['OwnedStart', 'HostMain', 'CheckPreserved']:
        flags = next(flags for a, b, flags in sections if a <= symbols[label] < b)
        assert flags & 0xe0000000 == 0x60000000
    extents = []
    for label, size in BUFFERS:
        address = symbols[f'{label}Buffer']
        flags = next(flags for a, b, flags in sections if a <= address and address+size <= b)
        assert flags & 0xe0000000 == 0xc0000000
        assert all(address+size+16 <= low or high <= address-16 for low, high in extents)
        extents.append((address-16, address+size+16))
    return dict(symbols={k: f'{v:08X}' for k, v in symbols.items() if not k.startswith('_')},
        imports=[dict(address=f'{a:08X}', dll=dll, name=name) for a, (dll, name) in imports.items()],
        owned_code_sha256=hashlib.sha256(pe.read(start, end-start)).hexdigest(),
        source_and_owned_code_final_RX=True, distinct_guarded_buffers_RW_nonexecutable=True,
        replacement_stubs=0, owned_disassembly=[f'{ins.address:08X}: {ins.mnemonic} {ins.op_str}' for ins in rows])


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('exe', type=Path)
    ap.add_argument('--sdk-bin-x86', type=Path, required=True)
    ap.add_argument('--kernel32-lib', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    image = SourceImage(args.exe)
    native, pairs = audit_source(image)
    source, code_pages, data_pages = host_source(image)
    args.out.mkdir(parents=True, exist_ok=False)
    out = args.out.resolve()
    assembly, obj, exe, mapping = [out/name for name in ['host-x86.asm', 'host-x86.obj', 'host-x86.exe', 'host-x86.map']]
    assembly.write_text(source)
    commands = [[str(args.sdk_bin_x86/'ml.exe'), '/nologo', '/c', '/coff', f'/Fo{obj}', str(assembly)],
        [str(args.sdk_bin_x86/'link.exe'), '/NOLOGO', '/MACHINE:X86', '/SUBSYSTEM:CONSOLE',
         '/ENTRY:HostMain', '/BASE:0x24000000', '/DYNAMICBASE:NO', '/SAFESEH:NO',
         f'/OUT:{exe}', f'/MAP:{mapping}', str(obj), str(args.kernel32_lib)]]
    for command in commands:
        result = subprocess.run(command, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout+'\n'+result.stderr
    linked = linked_audit(exe, mapping)
    table = image.read(TABLE, 80)
    manifest = dict(schema='bounded-original-cell-movement-scaling-host-v1', host_executed=False,
        source_sha256=SOURCE_SHA256, code_allowlist=native,
        data_allowlist=[dict(address=f'{address:08X}', bytes=size,
            sha256=hashlib.sha256(image.read(address, size)).hexdigest()) for address, size in CONSTANTS],
        explicit_scale_table=dict(address=f'{TABLE:08X}', bytes=80,
            original_sha256=hashlib.sha256(table).hexdigest(), original_bits=[f'{word:08X}' for word in struct.unpack('<20I', table)],
            policy='Each request supplies20rawbinary32words. Only owned setup temporarily makes1480000RW to replace exactly80bytes; entire page R before source entries. Other original row/constants verified byte-exact after every case. Original-table and synthetic override cohorts remain separate.'),
        original_progress_rows=pairs, raw1000_special_word=10,
        explicit_global=dict(address=f'{GLOBAL:08X}', policy='Immutable pointer to guarded RootBuffer; +5190 points to guarded DifficultyBuffer, whose +1C is explicit signed progress. Whole original global pageR before native entries.'),
        host_image_base='24000000', host_source_sha256=hashlib.sha256(assembly.read_bytes()).hexdigest(),
        host_exe_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(), linked_audit=linked, compile_commands=commands,
        pages=[dict(address=f'{page:08X}', bytes=65536, initial='RW', final='RX' if page in code_pages else 'R')
               for page in sorted(code_pages | data_pages)],
        protocol=dict(input_bytes=INPUT_BYTES, output_bytes=OUTPUT_BYTES, modeled_bytes=24, metadata_bytes=8,
            input='actorSizeIndex:i32@0,progress:i32@4,CW:u16@8,reserved0@10..15,20scaleBits:u32@16..95,reserved0@96..111',
            output='retainedExtended80:10bytes@0,reserved0:u16@10,committedBinary32:u32@12,selectedStageIndex:u32@16,committedRatio:u32@20,CW/SW:u16@24/26,MXCSR:u32@28',
            bounds='Actor index-1 skips progress validation/lookup and returns literal one; stageIndexFFFFFFFF/ratio0 explicitly mark unused outputs. Otherwise actor0..19/progress0..1000 only. All12maskedCWpolicies;MXCSR1F80controls. All table word bit patterns accepted; nonfinite quotient branch outcomes retained.'),
        entry_proof='For actor!=-1 call whole E4ECC0 with EDXprogress to record selected index, then whole E56330 with guarded actor. Per-call exact ESP and four callee-saved registers checked. E56330 overwrites its original actor stack argument with ratio on the active path; capture that exact committed word before caller stack cleanup. Owned FST32(nonpop),FSTP80 preserve native retained return and commit separately.',
        memory_proof='Native code has no indirect transfer or imported call and no replacement stub. For admitted progress loop<=20; computed selected word addresses14846E0..14848F4 and thresholds1484700..1484914 stay original immutable rows. Special1000 reads1484910. Actor numerator0..19 and denominator1..10 stay within supplied80byte table. No negative/out-of-table native index executed.',
        guard_proof='Input/actor/root/difficulty/value/reply each have16byte leading/trailing guards. Full immutable input/constructed-buffer shadows,80byte table and every original constant/row span checked after native call. Retained return is written only by owned bounded stores into16byte ValueBuffer; untouched padding and all output guards checked before FP metadata stores. CWunchanged,TOP0,MXCSRcontrols1F80; sticky flags retained separately.',
        execution_scope='Preparation only. No game/CRT/retail module or initializer/archive/liveactor loaded; no desktop/window/network. New root review required before hidden fixture execution.')
    (out/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps({k: manifest[k] for k in ['schema', 'host_executed', 'host_source_sha256', 'host_exe_sha256']}, indent=2))


if __name__ == '__main__':
    main()

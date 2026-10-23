"""Prepare, never execute, original Cell AI/style/base-speed getter bytes.

The complete three functions use an explicit difficulty object and checked
three-word creation-key provider. No game initializer/archive/actor is loaded.
Raw original bytes, generated assembly and executable remain under target/.
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

CODE = [(0xe52770, 0xe527b9), (0xe67cb0, 0xe67d15), (0xe67d20, 0xe67d9c)]
CONSTANT = (0x14861d0, 4)
PROVIDER = 0xe67a50
GLOBAL = 0x16b4be4
INPUT_BYTES, OUTPUT_BYTES = 832, 40
IMPORTS = {'GetStdHandle', 'ReadFile', 'WriteFile', 'VirtualAlloc', 'VirtualProtect',
           'GetLastError', 'FlushInstructionCache', 'ExitProcess'}
BUFFERS = [('Input', INPUT_BYTES), ('Definition', 796), ('Actor', 0x108),
           ('Cache', 0x1290), ('Root', 0x5194), ('Difficulty', 0x80),
           ('Speeds', 16), ('Reply', OUTPUT_BYTES)]


def audit_source(image):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    result = []
    calls = {(0xe67cca, 0xe52770), (0xe67cd9, 0xe52770),
             (0xe67d05, PROVIDER), (0xe67d6c, PROVIDER)}
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
                    assert (ins.address, op.imm) in calls
                else:
                    assert op.imm in boundaries
                transfers.append(dict(at=f'{ins.address:08X}', operation=ins.mnemonic,
                                      target=f'{op.imm:08X}'))
            for op in ins.operands:
                if op.type == X86_OP_MEM and not op.mem.base:
                    assert not op.mem.index and op.mem.disp in {GLOBAL, CONSTANT[0]}
                    absolute.append(dict(at=f'{ins.address:08X}', address=f'{op.mem.disp:08X}'))
        result.append(dict(start=f'{start:08X}', end_exclusive=f'{end:08X}', bytes=end-start,
            sha256=hashlib.sha256(image.read(start, end-start)).hexdigest(),
            transfers=transfers, indirect_transfers=0, absolute_reads=absolute,
            disassembly=[f'{ins.address:08X}: {ins.mnemonic} {ins.op_str}' for ins in rows]))
    assert image.read(*CONSTANT) == struct.pack('<f', 1.)
    return result


def host_source(image):
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none']
    for name, size in [('GetStdHandle', 4), ('ReadFile', 20), ('WriteFile', 20),
                       ('VirtualAlloc', 16), ('VirtualProtect', 16), ('GetLastError', 0),
                       ('FlushInstructionCache', 12), ('ExitProcess', 4)]:
        lines += [f'EXTERN _{name}@{size}:PROC']
    lines += ['.const']
    for start, end in CODE:
        lines += db(f'Original_{start:08X}', image.read(start, end-start))
    lines += db('OriginalOne', image.read(*CONSTANT))
    lines += ['.data']
    for label, size in BUFFERS:
        lines += [f'{label}Before DWORD 4 DUP(0A5A5A5A5h)',
                  f'{label}Buffer BYTE {size} DUP(0)', f'{label}After DWORD 4 DUP(0A5A5A5A5h)']
        if label not in {'Speeds', 'Reply'}:
            lines += [f'{label}Shadow BYTE {size} DUP(0)']
    lines += ['InputHandle DWORD 0', 'OutputHandle DWORD 0', 'Transferred DWORD 0',
              'OldProtect DWORD 0', 'SseControl DWORD 01F80h', 'NativeStack DWORD 0',
              'SelectedOffset DWORD 0', 'CacheCalls DWORD 0', 'ExpectedCalls DWORD 0',
              '.code', 'PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,CacheProvider,CacheStub',
              'PUBLIC InputBuffer,DefinitionBuffer,ActorBuffer,CacheBuffer,RootBuffer,DifficultyBuffer,SpeedsBuffer,ReplyBuffer',
              'OwnedStart LABEL BYTE',
              'CacheStub PROC', 'mov eax,OFFSET CacheProvider', 'jmp eax', 'CacheStub ENDP',
              'CacheProvider PROC', 'cmp dword ptr [esp],00E67D0Ah', 'je ProviderStyle',
              'cmp dword ptr [esp],00E67D71h', 'jne HostFailure', 'jmp ProviderKey',
              'ProviderStyle:', 'cmp CacheCalls,0', 'jne HostFailure', 'ProviderKey:',
              'cmp dword ptr [InputBuffer],0', 'je HostFailure',
              'mov eax,dword ptr [InputBuffer]', 'cmp dword ptr [esp+4],eax', 'jne HostFailure',
              'mov eax,dword ptr [InputBuffer+4]', 'cmp dword ptr [esp+8],eax', 'jne HostFailure',
              'mov eax,dword ptr [InputBuffer+8]', 'cmp dword ptr [esp+12],eax', 'jne HostFailure',
              'inc CacheCalls', 'cmp CacheCalls,2', 'ja HostFailure',
              'mov eax,OFFSET CacheBuffer', 'ret', 'CacheProvider ENDP',
              'CheckPreserved PROC', 'cmp ebx,011223344h', 'jne HostFailure',
              'cmp esi,022334455h', 'jne HostFailure', 'cmp edi,033445566h', 'jne HostFailure',
              'cmp ebp,044556677h', 'jne HostFailure', 'ret', 'CheckPreserved ENDP',
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
              'ReadDone:', 'pop esi', 'pop ebx', 'ret', 'ReadRecord ENDP', 'HostMain PROC']
    code_pages = {a & ~0xffff for a, _ in CODE}
    data_pages = {CONSTANT[0] & ~0xffff, GLOBAL & ~0xffff}
    for page in sorted(code_pages | data_pages):
        lines += ['push 04h', 'push 03000h', 'push 010000h', f'push 0{page:08X}h',
                  'call _VirtualAlloc@16', f'cmp eax,0{page:08X}h', 'jne AllocationFailure']
        if page in code_pages:
            lines += [f'mov edi,0{page:08X}h', 'mov ecx,010000h', 'mov al,0CCh', 'cld', 'rep stosb']
    for start, end in CODE:
        lines += [f'mov esi,OFFSET Original_{start:08X}', f'mov edi,0{start:08X}h',
                  f'mov ecx,{end-start}', 'cld', 'rep movsb']
    lines += ['mov esi,OFFSET CacheStub', f'mov edi,0{PROVIDER:08X}h', 'mov ecx,7', 'rep movsb',
              'mov esi,OFFSET OriginalOne', f'mov edi,0{CONSTANT[0]:08X}h', 'mov ecx,4', 'rep movsb',
              f'mov dword ptr [0{GLOBAL:08X}h],OFFSET RootBuffer']
    for page in sorted(code_pages | data_pages):
        lines += ['push OFFSET OldProtect', 'push 020h' if page in code_pages else 'push 02h',
                  'push 010000h', f'push 0{page:08X}h', 'call _VirtualProtect@16',
                  'test eax,eax', 'jz HostFailure']
    lines += ['push 0', 'push 0', 'push -1', 'call _FlushInstructionCache@12',
              'test eax,eax', 'jz HostFailure', 'push -10', 'call _GetStdHandle@4',
              'mov InputHandle,eax', 'push -11', 'call _GetStdHandle@4', 'mov OutputHandle,eax',
              'NextRecord:', 'call ReadRecord', 'test eax,eax', 'jz HostSuccess',
              'movzx eax,word ptr [InputBuffer+24]', 'mov edx,eax', 'and eax,0F0FFh',
              'cmp eax,07Fh', 'jne HostFailure', 'and edx,0300h', 'cmp edx,0100h', 'je HostFailure',
              'cmp word ptr [InputBuffer+26],0', 'jne HostFailure',
              'cmp dword ptr [InputBuffer+28],0', 'jne HostFailure',
              'cmp dword ptr [InputBuffer+828],0', 'jne HostFailure']
    for label, size in BUFFERS:
        if label != 'Input':
            lines += [f'mov edi,OFFSET {label}Buffer', f'mov ecx,{size//4}',
                      'mov eax,0A5A5A5A5h', 'cld', 'rep stosd']
    lines += ['mov esi,OFFSET InputBuffer+32', 'mov edi,OFFSET DefinitionBuffer',
              'mov ecx,199', 'rep movsd']
    for src, dst in [(0, 0xfc), (4, 0x100), (8, 0x104)]:
        lines += [f'mov eax,dword ptr [InputBuffer+{src}]',
                  f'mov dword ptr [ActorBuffer+0{dst:X}h],eax']
    lines += ['mov eax,dword ptr [InputBuffer+16]', 'mov dword ptr [CacheBuffer+01254h],eax',
              'mov eax,dword ptr [InputBuffer+20]', 'mov dword ptr [CacheBuffer+01258h],eax',
              'mov dword ptr [RootBuffer+05190h],OFFSET DifficultyBuffer',
              'mov eax,dword ptr [InputBuffer+12]', 'mov dword ptr [DifficultyBuffer+07Ch],eax']
    for label, size in BUFFERS:
        if label not in {'Speeds', 'Reply'}:
            lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                      f'mov ecx,{size//4}', 'rep movsd']
    lines += ['mov CacheCalls,0', 'fninit', 'fldcw word ptr [InputBuffer+24]', 'ldmxcsr SseControl']
    def preserved():
        return ['mov ebx,011223344h', 'mov esi,022334455h', 'mov edi,033445566h', 'mov ebp,044556677h']
    lines += preserved() + ['mov NativeStack,esp', 'push OFFSET DefinitionBuffer',
              'mov eax,00E52770h', 'call eax', 'add esp,4', 'cmp esp,NativeStack', 'jne HostFailure',
              'call CheckPreserved', 'sub eax,OFFSET DefinitionBuffer',
              'cmp eax,0E0h', 'je SelectedValid', 'cmp eax,0194h', 'je SelectedValid',
              'cmp eax,0248h', 'jne HostFailure', 'SelectedValid:', 'mov SelectedOffset,eax',
              'mov dword ptr [ReplyBuffer+4],eax',
              'mov ecx,OFFSET DefinitionBuffer', 'cmp dword ptr [InputBuffer],0',
              'jne StyleDefinition', 'xor ecx,ecx', 'StyleDefinition:']
    lines += preserved() + ['mov NativeStack,esp', 'push ecx', 'push OFFSET ActorBuffer',
              'mov eax,00E67CB0h', 'call eax', 'add esp,8', 'cmp esp,NativeStack', 'jne HostFailure',
              'call CheckPreserved', 'mov dword ptr [ReplyBuffer],eax']
    lines += preserved() + ['mov NativeStack,esp', 'push OFFSET SpeedsBuffer', 'push OFFSET ActorBuffer',
              'mov eax,00E67D20h', 'call eax', 'add esp,8', 'cmp esp,NativeStack', 'jne HostFailure',
              'call CheckPreserved', 'xor eax,eax', 'cmp dword ptr [InputBuffer],0', 'je ExpectedReady',
              'inc eax', 'mov ecx,SelectedOffset', 'cmp dword ptr [DefinitionBuffer+ecx+16],0',
              'jne ExpectedReady', 'inc eax', 'ExpectedReady:', 'mov ExpectedCalls,eax',
              'cmp CacheCalls,eax', 'jne HostFailure']
    for label, size in BUFFERS:
        if label not in {'Speeds', 'Reply'}:
            lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                      f'mov ecx,{size//4}', 'cld', 'repe cmpsd', 'jne HostFailure']
    for label, _ in BUFFERS:
        for side in ['Before', 'After']:
            for offset in range(0, 16, 4):
                lines += [f'cmp dword ptr [{label}{side}+{offset}],0A5A5A5A5h', 'jne HostFailure']
    # Reply metadata has never been used as the native writable speed destination.
    for offset in range(8, OUTPUT_BYTES, 4):
        lines += [f'cmp dword ptr [ReplyBuffer+{offset}],0A5A5A5A5h', 'jne HostFailure']
    lines += ['fnstcw word ptr [ReplyBuffer+28]', 'fnstsw word ptr [ReplyBuffer+30]',
              'stmxcsr dword ptr [ReplyBuffer+32]', 'mov ax,word ptr [ReplyBuffer+28]',
              'cmp ax,word ptr [InputBuffer+24]', 'jne HostFailure',
              'test word ptr [ReplyBuffer+30],03800h', 'jnz HostFailure',
              'cmp dword ptr [ReplyBuffer+32],01F80h', 'jne HostFailure',
              'mov eax,CacheCalls', 'mov dword ptr [ReplyBuffer+8],eax',
              'mov esi,OFFSET SpeedsBuffer', 'mov edi,OFFSET ReplyBuffer+12', 'mov ecx,4', 'cld', 'rep movsd',
              'mov dword ptr [ReplyBuffer+36],0', 'push 0', 'push OFFSET Transferred',
              f'push {OUTPUT_BYTES}', 'push OFFSET ReplyBuffer', 'push OutputHandle', 'call _WriteFile@20',
              'test eax,eax', 'jz HostFailure', f'cmp Transferred,{OUTPUT_BYTES}', 'jne HostFailure',
              'jmp NextRecord', 'HostSuccess:', 'push 0', 'call _ExitProcess@4',
              'HostFailure::', 'push 7', 'call _ExitProcess@4',
              'AllocationFailure:', 'push 10', 'call _ExitProcess@4',
              'HostMain ENDP', 'OwnedEnd LABEL BYTE', 'END HostMain']
    return '\n'.join(lines)+'\n', code_pages, data_pages


def linked_audit(path, mapping):
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_REG
    pe = PE(path)
    symbols = {}
    for line in mapping.read_text().splitlines():
        m = re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\w+)\s+([0-9A-Fa-f]{8})\b', line)
        if m:
            symbols[m[1]] = int(m[2], 16)
    imports = pe.imports()
    assert {name for _, name in imports.values()} == IMPORTS
    assert {dll.upper() for dll, _ in imports.values()} == {'KERNEL32.DLL'}
    assert pe.base == 0x24000000
    start, end = symbols['OwnedStart'], symbols['OwnedEnd']
    rows = list(decoder().disasm(pe.read(start, end-start), start))
    assert sum(row.size for row in rows) == end-start
    boundaries = {row.address for row in rows}
    for index, ins in enumerate(rows):
        assert ins.mnemonic not in {'int', 'syscall', 'sysenter', 'hlt', 'iret', 'iretd'}
        if ins.group(CS_GRP_JUMP):
            op = ins.operands[0]
            if op.type == X86_OP_IMM:
                assert op.imm in boundaries
            else:
                assert op.type == X86_OP_REG and ins.op_str == 'eax'
                previous = rows[index-1]
                assert previous.mnemonic == 'mov' and previous.operands[1].type == X86_OP_IMM
                assert previous.operands[1].imm == symbols['CacheProvider']
        if ins.group(CS_GRP_CALL):
            op = ins.operands[0]
            if op.type == X86_OP_REG:
                assert ins.op_str == 'eax'
                previous = rows[index-1]
                assert previous.mnemonic == 'mov' and previous.operands[1].type == X86_OP_IMM
                assert previous.operands[1].imm in {a for a, _ in CODE}
            else:
                assert op.type == X86_OP_IMM
                if op.imm in boundaries:
                    assert op.imm in {symbols['ReadRecord'], symbols['CheckPreserved']}
                else:
                    raw = pe.read(op.imm, 6)
                    assert raw[:2] == b'\xff\x25' and struct.unpack('<I', raw[2:])[0] in imports
    stub = b'\xb8'+struct.pack('<I', symbols['CacheProvider'])+b'\xff\xe0'
    assert pe.read(symbols['CacheStub'], 7) == stub
    pe_at = struct.unpack_from('<I', pe.data, 0x3c)[0]
    n = struct.unpack_from('<H', pe.data, pe_at+6)[0]
    optional = struct.unpack_from('<H', pe.data, pe_at+20)[0]
    sections = []
    for i in range(n):
        at = pe_at+24+optional+40*i
        virtual_size, rva = struct.unpack_from('<II', pe.data, at+8)
        sections.append((pe.base+rva, pe.base+rva+virtual_size,
                         struct.unpack_from('<I', pe.data, at+36)[0]))
    for label in ['CacheProvider', 'CacheStub', 'HostMain']:
        flags = next(flags for a, b, flags in sections if a <= symbols[label] < b)
        assert flags & 0xe0000000 == 0x60000000
    for label, size in BUFFERS:
        at = symbols[f'{label}Buffer']
        flags = next(flags for a, b, flags in sections if a <= at and at+size <= b)
        assert flags & 0xe0000000 == 0xc0000000
    for first, (label, size) in enumerate(BUFFERS):
        start_buffer = symbols[f'{label}Buffer']-16
        end_buffer = start_buffer+size+32
        for other, other_size in BUFFERS[first+1:]:
            other_start = symbols[f'{other}Buffer']-16
            assert end_buffer <= other_start or other_start+other_size+32 <= start_buffer
    return dict(symbols={k: f'{v:08X}' for k, v in symbols.items() if not k.startswith('_')},
        imports=[dict(address=f'{a:08X}', dll=dll, name=name) for a, (dll, name) in imports.items()],
        cache_stub=dict(copy_to=f'{PROVIDER:08X}', hex=stub.hex(), relative_transfers=0),
        owned_code_sha256=hashlib.sha256(pe.read(start, end-start)).hexdigest(),
        source_and_owned_code_final_RX=True, distinct_guarded_buffers_RW_nonexecutable=True,
        owned_disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in rows])


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('exe', type=Path)
    ap.add_argument('--sdk-bin-x86', type=Path, required=True)
    ap.add_argument('--kernel32-lib', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    image = SourceImage(args.exe)
    native = audit_source(image)
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
    manifest = dict(schema='bounded-original-cell-movement-source-host-v1', host_executed=False,
        source_sha256=SOURCE_SHA256, code_allowlist=native,
        data_allowlist=[dict(address=f'{CONSTANT[0]:08X}', bytes=4,
            sha256=hashlib.sha256(image.read(*CONSTANT)).hexdigest(), value_bits='3F800000')],
        explicit_global=dict(address=f'{GLOBAL:08X}', policy='Immutable pointer to guarded RootBuffer; only +5190 points to guarded DifficultyBuffer, whose +7C is explicit raw request difficulty. Entire original global page R before entries.'),
        host_image_base='24000000', host_source_sha256=hashlib.sha256(assembly.read_bytes()).hexdigest(),
        host_exe_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(), linked_audit=linked,
        compile_commands=commands,
        pages=[dict(address=f'{page:08X}', bytes=65536, initial='RW', final='RX' if page in code_pages else 'R')
               for page in sorted(code_pages | data_pages)],
        protocol=dict(input_bytes=INPUT_BYTES, output_bytes=OUTPUT_BYTES,
            input='actorFC/100/104:u32@0/4/8,difficulty:u32@12,cachedMask:u32@16,cachedSpeed:f32bits@20,CW:u16@24,reserved0@26..31,definition796@32,reserved0@828..831',
            output='style:u32@0,selectedAIoffset:u32@4,cacheProviderCalls:u32@8,fourSpeedBits:u32@12..27,CW:u16@28,SW:u16@30,MXCSR:u32@32,reserved0:u32@36',
            modeled_bytes=32, metadata_bytes=8,
            bounds='Exactly796-byte definitions; scalar actor/cache words and raw difficulty bits. All12maskedx87 policies, MXCSR1F80. Nonfinite cache-speed words remain a separate input/output-policy cohort, not silently rejected by native host.'),
        entry_proof='Calls full E52770 then E67CB0 then E67D20 separately; this is a fixture protocol, not a claimed combined game function. Exact ESP and EBX/ESI/EDI/EBP checked after each call. E67CB0 receives NULL definition iff actorFC0, exercising its unused-reference early exit. Selected pointer must be definition+E0/194/248.',
        provider_proof='Only original E67A50 is replaced with seven-byte MOV EAX,ownedCacheProvider/JMP EAX. Callback checks exact native return E67D0A/E67D71, all3 byvalue key words, nonzeroFC and max2calls. Style lookup only first; final total equals1speed plus1iffselected source style0. No pointer accepted in protocol.',
        guard_proof='Input/definition/actor/cache/root/difficulty/speeds/reply have separate16-byte leading/trailing canaries. Full readonly-input and constructed-buffer shadows checked. Native writes only guarded16byte speeds. All guards plus untouched reply suffix checked before post-call FP metadata stores. CWunchanged,TOP0,MXCSRexact1F80; x87 sticky flags retained separately.',
        execution_scope='Preparation only. No game/CRT/retail module loaded, no initializer/archive/liveactor, no desktop/window/network. Root review is required before hidden fixture execution.')
    (out/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps({k: manifest[k] for k in ['schema', 'host_executed', 'host_source_sha256', 'host_exe_sha256']}, indent=2))


if __name__ == '__main__':
    main()

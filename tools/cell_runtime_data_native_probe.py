"""Prepare, never compile or execute, three bounded runtime-data kernels.

The parent reviews this packet, compiles its MASM with the retained commands,
then invokes --audit-linked before considering any hidden fixture execution.
Original bytes, assembly and binaries remain under ignored target/.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import sys

from cell_native_math_probe import db
from cell_schema_probe import SOURCE_SHA256, SourceImage
from native_crt_acos_probe import PE, decoder

CODE = [(0x99C170, 0x99C207), (0x99C210, 0x99C2C2), (0xE833B0, 0xE83413)]
EPSILON = (0x1447BB8, 8)
INPUT_BYTES, OUTPUT_BYTES, AGGREGATE_BYTES = 200, 180, 164
BUFFERS = [('Input', INPUT_BYTES), ('Object', 0x180), ('Owner', 0x2E8),
           ('Pose', 0x2BC), ('Value', 16), ('Reply', OUTPUT_BYTES)]
IMPORTS = {'GetStdHandle', 'ReadFile', 'WriteFile', 'VirtualAlloc', 'VirtualProtect',
           'GetLastError', 'FlushInstructionCache', 'ExitProcess'}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def audit_source(image):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    records = []
    assert image.read(*EPSILON).hex() == '3a8c30e28e79453e'
    for start, end in CODE:
        rows = list(decoder().disasm(image.read(start, end-start), start))
        assert sum(row.size for row in rows) == end-start
        assert rows[-1].mnemonic == 'ret'
        boundaries = {row.address for row in rows}
        transfers, absolute = [], []
        for ins in rows:
            assert not ins.group(CS_GRP_CALL)
            if ins.group(CS_GRP_JUMP):
                op = ins.operands[0]
                assert op.type == X86_OP_IMM and op.imm in boundaries
                transfers.append(dict(at=f'{ins.address:08X}', operation=ins.mnemonic,
                                      target=f'{op.imm:08X}'))
            for op in ins.operands:
                if op.type == X86_OP_MEM and not op.mem.base and not op.mem.index:
                    assert (op.mem.disp, op.size) == EPSILON
                    absolute.append(dict(at=f'{ins.address:08X}', address=f'{op.mem.disp:08X}',
                                         bytes=op.size))
        records.append(dict(start=f'{start:08X}', end_exclusive=f'{end:08X}', bytes=end-start,
            sha256=digest(image.read(start, end-start)), calls=0, indirect_transfers=0,
            transfers=transfers, absolute_operands=absolute,
            disassembly=[f'{ins.address:08X}: {ins.bytes.hex()} {ins.mnemonic} {ins.op_str}' for ins in rows]))
    return records


def host_source(image):
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none']
    for name, size in [('GetStdHandle', 4), ('ReadFile', 20), ('WriteFile', 20),
                       ('VirtualAlloc', 16), ('VirtualProtect', 16), ('GetLastError', 0),
                       ('FlushInstructionCache', 12), ('ExitProcess', 4)]:
        lines += [f'EXTERN _{name}@{size}:PROC']
    lines += ['.const']
    for start, end in CODE:
        lines += [f'PUBLIC Original_{start:08X}'] + db(f'Original_{start:08X}', image.read(start, end-start))
    lines += ['PUBLIC Constant_01447BB8'] + db('Constant_01447BB8', image.read(*EPSILON))
    lines += ['.data']
    for label, size in BUFFERS:
        lines += [f'PUBLIC {label}Buffer', f'{label}Before DWORD 4 DUP(0A5A5A5A5h)']
        if label == 'Pose':
            lines += ['PoseBuffer BYTE 160 DUP(0)', 'PUBLIC AggregateBuffer',
                      'AggregateBuffer BYTE 164 DUP(0)', 'PoseSuffix BYTE 376 DUP(0)']
            assert 160 + 164 + 376 == size
        else:
            lines += [f'{label}Buffer BYTE {size} DUP(0)']
        lines += [f'{label}After DWORD 4 DUP(0A5A5A5A5h)']
        if label not in {'Value', 'Reply'}:
            lines += [f'{label}Shadow BYTE {size} DUP(0)']
    lines += ['InputHandle DWORD 0', 'OutputHandle DWORD 0', 'Transferred DWORD 0',
        'OldProtect DWORD 0', 'SseControl DWORD 01F80h', 'NativeStack DWORD 0',
        'CallStack DWORD 0', 'StatusWord DWORD 0', 'ExpectedStatus DWORD 0',
        'WriteRowOffset DWORD 0FFFFFFFFh', 'Inserted DWORD 0', '.code',
        'PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,ReadRecord,CheckPreserved',
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
    code_pages = {start & ~0xFFFF for start, _ in CODE}
    data_pages = {EPSILON[0] & ~0xFFFF}
    for page in sorted(code_pages | data_pages):
        lines += ['push 04h', 'push 03000h', 'push 010000h', f'push 0{page:08X}h',
            'call _VirtualAlloc@16', f'cmp eax,0{page:08X}h', 'jne AllocationFailure']
        if page in code_pages:
            lines += [f'mov edi,0{page:08X}h', 'mov ecx,010000h', 'mov al,0CCh', 'cld', 'rep stosb']
    for start, end in CODE:
        lines += [f'mov esi,OFFSET Original_{start:08X}', f'mov edi,0{start:08X}h',
                  f'mov ecx,{end-start}', 'cld', 'rep movsb']
    lines += ['mov esi,OFFSET Constant_01447BB8', 'mov edi,01447BB8h', 'mov ecx,8', 'rep movsb']
    for page in sorted(code_pages | data_pages):
        lines += ['push OFFSET OldProtect', 'push 020h' if page in code_pages else 'push 02h',
            'push 010000h', f'push 0{page:08X}h', 'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure']
    lines += ['push 0', 'push 0', 'push -1', 'call _FlushInstructionCache@12', 'test eax,eax', 'jz HostFailure',
        'push -10', 'call _GetStdHandle@4', 'mov InputHandle,eax',
        'push -11', 'call _GetStdHandle@4', 'mov OutputHandle,eax',
        'NextRecord:', 'call ReadRecord', 'test eax,eax', 'jz HostSuccess',
        'cmp dword ptr [InputBuffer],2', 'ja HostFailure',
        'movzx eax,word ptr [InputBuffer+8]', 'mov edx,eax', 'and eax,0F0FFh',
        'cmp eax,07Fh', 'jne HostFailure', 'and edx,0300h', 'cmp edx,0100h', 'je HostFailure',
        'cmp word ptr [InputBuffer+10],0', 'jne HostFailure',
        'cmp dword ptr [InputBuffer+28],0', 'jne HostFailure',
        'cmp dword ptr [InputBuffer+32],0', 'jne HostFailure',
        'mov eax,dword ptr [InputBuffer+36]', 'cmp eax,8', 'jbe CountValid',
        'cmp dword ptr [InputBuffer],0', 'jne HostFailure', 'test eax,eax', 'jns HostFailure',
        'CountValid:']
    for label, size in BUFFERS:
        if label != 'Input':
            lines += [f'mov edi,OFFSET {label}Buffer', f'mov ecx,{size//4}',
                      'mov eax,0A5A5A5A5h', 'cld', 'rep stosd']
    lines += ['mov dword ptr [ObjectBuffer+017Ch],OFFSET OwnerBuffer',
        'mov dword ptr [OwnerBuffer+02E4h],OFFSET PoseBuffer',
        'mov esi,OFFSET InputBuffer+36', 'mov edi,OFFSET AggregateBuffer', 'mov ecx,41', 'rep movsd']
    for label, size in BUFFERS:
        if label not in {'Value', 'Reply'}:
            lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                      f'mov ecx,{size//4}', 'rep movsd']
    # Independent integer-only scan establishes the permitted write region.
    lines += ['mov WriteRowOffset,0FFFFFFFFh', 'mov Inserted,0', 'mov ExpectedStatus,0',
        'mov ecx,dword ptr [AggregateBuffer]', 'xor edx,edx', 'test ecx,ecx', 'jle ScanAbsent',
        'mov edi,dword ptr [InputBuffer+4]', 'mov esi,OFFSET AggregateBuffer+4',
        'ScanLoop:', 'cmp dword ptr [esi],edi', 'je ScanFound', 'inc edx', 'add esi,20',
        'cmp edx,ecx', 'jb ScanLoop', 'ScanAbsent:',
        'cmp dword ptr [InputBuffer],0', 'je ScanReady', 'cmp ecx,8', 'jae ScanReady',
        'mov edx,ecx', 'mov Inserted,1', 'ScanFound:',
        'imul edx,edx,20', 'add edx,4', 'mov WriteRowOffset,edx', 'mov ExpectedStatus,1', 'ScanReady:',
        'fninit', 'fldcw word ptr [InputBuffer+8]', 'ldmxcsr SseControl',
        'mov ebx,011223344h', 'mov esi,022334455h', 'mov edi,033445566h', 'mov ebp,044556677h',
        'mov NativeStack,esp', 'cmp dword ptr [InputBuffer],0', 'jne DispatchWriter',
        'push OFFSET ValueBuffer+4', 'push dword ptr [InputBuffer+4]', 'push OFFSET ObjectBuffer',
        'mov CallStack,esp', 'mov eax,00E833B0h', 'call eax',
        'cmp esp,CallStack', 'jne HostFailure', 'add esp,12', 'jmp CallComplete',
        'DispatchWriter:', 'push dword ptr [InputBuffer+24]',
        'cmp dword ptr [InputBuffer],1', 'je DispatchScalar',
        'push dword ptr [InputBuffer+20]', 'push dword ptr [InputBuffer+16]',
        'push dword ptr [InputBuffer+12]', 'push dword ptr [InputBuffer+4]',
        'mov ecx,OFFSET AggregateBuffer', 'mov eax,0099C210h', 'call eax', 'jmp CallComplete',
        'DispatchScalar:', 'push dword ptr [InputBuffer+16]', 'push dword ptr [InputBuffer+12]',
        'push dword ptr [InputBuffer+4]', 'mov ecx,OFFSET AggregateBuffer',
        'mov eax,0099C170h', 'call eax', 'CallComplete:',
        'cmp esp,NativeStack', 'jne HostFailure', 'movzx eax,al', 'mov StatusWord,eax',
        'cmp eax,ExpectedStatus', 'jne HostFailure', 'call CheckPreserved',
        'pushfd', 'pop eax', 'test eax,0400h', 'jnz HostFailure']
    for label, size in BUFFERS:
        if label not in {'Pose', 'Value', 'Reply'}:
            lines += [f'mov esi,OFFSET {label}Buffer', f'mov edi,OFFSET {label}Shadow',
                      f'mov ecx,{size//4}', 'cld', 'repe cmpsd', 'jne HostFailure']
    lines += ['mov esi,OFFSET PoseBuffer', 'mov edi,OFFSET PoseShadow', 'mov ecx,40',
        'cld', 'repe cmpsd', 'jne HostFailure',
        'mov esi,OFFSET PoseBuffer+0144h', 'mov edi,OFFSET PoseShadow+0144h', 'mov ecx,94',
        'repe cmpsd', 'jne HostFailure',
        'mov eax,dword ptr [InputBuffer+36]', 'add eax,Inserted',
        'cmp dword ptr [AggregateBuffer],eax', 'jne HostFailure', 'xor ecx,ecx',
        'AggregateCheckLoop:', 'cmp dword ptr [InputBuffer],0', 'je AggregateCompare',
        'test ecx,ecx', 'jnz AggregateMaybeRow', 'cmp Inserted,1', 'je AggregateCheckNext',
        'AggregateMaybeRow:', 'mov edx,ecx', 'sub edx,WriteRowOffset', 'cmp edx,20', 'jae AggregateCompare',
        'test edx,edx', 'jnz AggregateRowField', 'cmp Inserted,1', 'je AggregateCheckNext',
        'jmp AggregateCompare', 'AggregateRowField:', 'cmp edx,4', 'je AggregateCheckNext',
        'cmp edx,8', 'je AggregateCheckNext', 'cmp edx,16', 'je AggregateCheckNext',
        'cmp edx,12', 'jne AggregateCompare', 'cmp dword ptr [InputBuffer],2', 'je AggregateCheckNext',
        'AggregateCompare:', 'mov eax,dword ptr [AggregateBuffer+ecx]',
        'cmp eax,dword ptr [PoseShadow+0A0h+ecx]', 'jne HostFailure',
        'AggregateCheckNext:', 'add ecx,4', 'cmp ecx,164', 'jb AggregateCheckLoop']
    for start, end in CODE:
        lines += [f'mov esi,OFFSET Original_{start:08X}', f'mov edi,0{start:08X}h',
                  f'mov ecx,{end-start}', 'cld', 'repe cmpsb', 'jne HostFailure']
    lines += ['mov esi,OFFSET Constant_01447BB8', 'mov edi,01447BB8h',
              'mov ecx,8', 'repe cmpsb', 'jne HostFailure']
    for label, _ in BUFFERS:
        for side in ['Before', 'After']:
            for offset in range(0, 16, 4):
                lines += [f'cmp dword ptr [{label}{side}+{offset}],0A5A5A5A5h', 'jne HostFailure']
    for offset in [0, 8, 12]:
        lines += [f'cmp dword ptr [ValueBuffer+{offset}],0A5A5A5A5h', 'jne HostFailure']
    lines += ['cmp dword ptr [InputBuffer],0', 'je ValueChecked',
              'cmp dword ptr [ValueBuffer+4],0A5A5A5A5h', 'jne HostFailure', 'ValueChecked:']
    for offset in range(0, OUTPUT_BYTES, 4):
        lines += [f'cmp dword ptr [ReplyBuffer+{offset}],0A5A5A5A5h', 'jne HostFailure']
    lines += ['fnstcw word ptr [ReplyBuffer+172]', 'fnstsw word ptr [ReplyBuffer+174]',
        'stmxcsr dword ptr [ReplyBuffer+176]', 'mov ax,word ptr [ReplyBuffer+172]',
        'cmp ax,word ptr [InputBuffer+8]', 'jne HostFailure',
        'test word ptr [ReplyBuffer+174],03800h', 'jnz HostFailure',
        'mov eax,dword ptr [ReplyBuffer+176]', 'and eax,0FFFFFFC0h', 'cmp eax,01F80h', 'jne HostFailure',
        'mov eax,StatusWord', 'mov dword ptr [ReplyBuffer],eax',
        'mov eax,dword ptr [ValueBuffer+4]', 'mov dword ptr [ReplyBuffer+4],eax',
        'mov esi,OFFSET AggregateBuffer', 'mov edi,OFFSET ReplyBuffer+8', 'mov ecx,41', 'cld', 'rep movsd',
        'push 0', 'push OFFSET Transferred', f'push {OUTPUT_BYTES}', 'push OFFSET ReplyBuffer',
        'push OutputHandle', 'call _WriteFile@20', 'test eax,eax', 'jz HostFailure',
        f'cmp Transferred,{OUTPUT_BYTES}', 'jne HostFailure', 'jmp NextRecord',
        'HostSuccess:', 'push 0', 'call _ExitProcess@4',
        'HostFailure::', 'push 7', 'call _ExitProcess@4',
        'AllocationFailure:', 'push 10', 'call _ExitProcess@4',
        'HostMain ENDP', 'OwnedEnd LABEL BYTE', 'END HostMain']
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
                assert previous.operands[1].imm in {a for a, _ in CODE}
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
    for label in ['OwnedStart', 'HostMain', 'CheckPreserved', 'ReadRecord']:
        flags = next(flags for a, b, flags in sections if a <= symbols[label] < b)
        assert flags & 0xE0000000 == 0x60000000
    extents = []
    for label, size in BUFFERS:
        address = symbols[f'{label}Buffer']
        flags = next(flags for a, b, flags in sections if a <= address and address+size <= b)
        assert flags & 0xE0000000 == 0xC0000000
        assert all(address+size+16 <= low or high <= address-16 for low, high in extents)
        extents.append((address-16, address+size+16))
    assert symbols['AggregateBuffer'] == symbols['PoseBuffer'] + 0xA0
    for address, length in [(a, b-a) for a, b in CODE] + [EPSILON]:
        label = f'Constant_{address:08X}' if (address, length) == EPSILON else f'Original_{address:08X}'
        stored = pe.read(symbols[label], length)
        assert stored == image.read(address, length)
        flags = next(flags for a, b, flags in sections if a <= symbols[label] and symbols[label]+length <= b)
        assert flags & 0xE0000000 == 0x40000000
    return dict(symbols={k: f'{v:08X}' for k, v in symbols.items() if not k.startswith('_')},
        imports=[dict(address=f'{a:08X}', dll=dll, name=name) for a, (dll, name) in imports.items()],
        owned_code_sha256=digest(pe.read(start, end-start)), source_payloads_byte_exact=True,
        source_payloads_R_nonexecutable=True, owned_code_RX=True, guarded_buffers_RW_nonexecutable=True,
        replacement_stubs=0, owned_disassembly=[f'{ins.address:08X}: {ins.mnemonic} {ins.op_str}' for ins in rows])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('exe', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--sdk-bin-x86', type=Path)
    parser.add_argument('--kernel32-lib', type=Path)
    parser.add_argument('--audit-linked', type=Path)
    parser.add_argument('--map', type=Path)
    args = parser.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    image = SourceImage(args.exe)
    native = audit_source(image)
    if args.audit_linked:
        if not args.map:
            parser.error('--map is required with --audit-linked')
        manifest = json.loads((args.out/'manifest.json').read_text())
        assembly = args.out/'host-x86.asm'
        expected, _, _ = host_source(image)
        assert assembly.read_text() == expected
        assert digest(assembly.read_bytes()) == manifest['host_source_sha256']
        result = audit_linked(args.audit_linked, args.map, image)
        result.update(host_exe_sha256=digest(args.audit_linked.read_bytes()),
                      host_map_sha256=digest(args.map.read_bytes()), host_executed=False)
        (args.out/'linked-audit.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps({k: result[k] for k in ['host_exe_sha256', 'owned_code_sha256', 'host_executed']}, indent=2))
        return
    if not args.sdk_bin_x86 or not args.kernel32_lib:
        parser.error('--sdk-bin-x86 and --kernel32-lib record the parent compilation commands')
    source, code_pages, data_pages = host_source(image)
    args.out.mkdir(parents=True, exist_ok=False)
    out = args.out.resolve()
    assembly, obj, exe, mapping = [out/name for name in ['host-x86.asm', 'host-x86.obj', 'host-x86.exe', 'host-x86.map']]
    assembly.write_text(source)
    commands = [[str(args.sdk_bin_x86/'ml.exe'), '/nologo', '/c', '/coff', f'/Fo{obj}', str(assembly)],
        [str(args.sdk_bin_x86/'link.exe'), '/NOLOGO', '/MACHINE:X86', '/SUBSYSTEM:CONSOLE',
         '/ENTRY:HostMain', '/BASE:0x24000000', '/DYNAMICBASE:NO', '/SAFESEH:NO',
         f'/OUT:{exe}', f'/MAP:{mapping}', str(obj), str(args.kernel32_lib)]]
    manifest = dict(schema='bounded-original-cell-runtime-data-host-v1', source_sha256=SOURCE_SHA256,
        host_compiled=False, host_executed=False, code_allowlist=native,
        data_allowlist=[dict(address='01447BB8', bytes=8, sha256=digest(image.read(*EPSILON)),
                             raw_hex=image.read(*EPSILON).hex(), interpretation='binary64 1e-8')],
        host_image_base='24000000', host_source_sha256=digest(assembly.read_bytes()),
        preparation_tool_sha256=digest(Path(__file__).read_bytes()), compile_commands=commands,
        imports=sorted(IMPORTS),
        pages=[dict(address=f'{page:08X}', bytes=65536, initial='RW', final='RX' if page in code_pages else 'R')
               for page in sorted(code_pages | data_pages)],
        guarded_buffers=[dict(name=name, bytes=size, before_guard=16, after_guard=16) for name, size in BUFFERS],
        protocol=dict(input_bytes=200, output_bytes=180, modeled_bytes=172, metadata_bytes=8,
            input='mode:u32@0(0reader/1scalar/2pair),ID:u32@4,CW:u16@8,reserved0:u16@10,rawKind:u32@12,firstBits:u32@16,secondBits:u32@20,weightBits:u32@24,reserved0:u32@28/32,count+8x20aggregate:164bytes@36',
            output='nativeALzeroextended:u32@0,valueBits:u32@4(readerFSTP32;writersunchangedA5A5A5A5),aggregate:164bytes@8,CW:u16@172,SW:u16@174,MXCSR:u32@176',
            bounds='Reader signed counts<=0 or0..8; writer unsigned counts0..8. All other count words rejected before native entry. Arbitrary ID/kind/float-bit payloads. Twelve exact masked CWs, MXCSR1F80 controls. Reserved bytes zero. Partial records always fail, EOF/BROKEN_PIPE only between fresh records.'),
        aggregate_view=dict(buffer='Pose', offset=160, bytes=164, label='AggregateBuffer',
                            policy='Interior count-plus-eight-row view, not a separate overlapping guarded allocation. Pose prefix160 and suffix376 stay unchanged.'),
        source_entry_proof='Three complete original no-call functions only. E833B0 cdecl receives owned object/ID/valuePointer, follows Object+17C toOwner andOwner+2E4 toPose, then reads count atPose+A0. 99C170/210 thiscall receive Pose+A0 and four/five explicit words; RET16/20 checked via actualESP before any cleanup. All four callee-saved registers must retain independent sentinels. Reader FSTP32 leaves no retained80 return; internal epsilon/add/div precision remains explicit.',
        memory_proof='Bounded fakeObject384 bytes and fakeOwner744 bytes have exact final-field pointers and whole shadows. Pose700 bytes holds an interior Aggregate164 at+A0, count4 plus eight20-byte rows; whole prefix160/suffix376 are shadow-checked unchanged. All native scan indices are<=7; original writes fit the selected row. Host determines first ID match independently, enforces nativeAL and count increment, and checks every nonpermitted aggregate DWORD against its original shadow (including scalarrow+C). Immutable input/object/owner shadows, source code and epsilon checked per case.',
        guard_proof='Six separated buffers each have16-byte before/after canaries. Value buffer unused words and every reply DWORD are checked before owned output/FPmetadata stores. ESP,callee-savedregs,DF0,CWunchanged,TOP0,MXCSRcontrols1F80 validated; stickyflags retained. Source bytes placed only at reviewed fixed addresses, gapsINT3, pagesRX; epsilon pageR. Source payloads remainR/nonexec inside host image; linked audit must prove it before execution.',
        limitations='No authoredcurve evaluation, application/controller/clipselection, game initializer, retail process, DLL load, GUI, network or replacement callback. Raw native nonfinite outcomes retained; Rust reader finite-policy cases are a separate positive error gate. Initial packet is prepare-only; no assembler/compiler/native execution performed by this tool.',
        review_status='Pending parent source/ASM review, parent compilation, linked audit and independent peer review before any native fixture.')
    (out/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    (out/'compile-commands.json').write_text(json.dumps(commands, indent=2)+'\n')
    print(json.dumps({k: manifest[k] for k in ['schema', 'host_compiled', 'host_executed', 'host_source_sha256']}, indent=2))


if __name__ == '__main__':
    main()

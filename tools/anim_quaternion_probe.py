"""Prepare, never execute, a bounded original-byte quaternion fixture.

Original bytes and the generated x86 host stay under target/. The fixture
binds the source acos IAT slot to the separately audited finite SSE2 wrapper;
it does not load the original modules or establish the retail dispatch/state.
"""
import argparse
from collections import Counter
import hashlib
import itertools
import json
from pathlib import Path
import re
import struct
import subprocess

from cell_schema_probe import SourceImage, SOURCE_SHA256
import native_crt_acos_probe as crt

ROOT = Path(__file__).resolve().parent.parent
SPANS = [(0x99C810, 0x99C8D0), (0x99C8D0, 0x99C993), (0x99C9A0, 0x99CB6C)]
THUNK = (0x11E06B8, 0x11E06BE)
IAT = 0x13CC48C
CONSTANTS = [(0x13EB1BC, 4), (0x13EB8B0, 16), (0x13EBCA8, 4),
             (0x1485E24, 4), (0x14861D0, 4)]
CONTROLS = [pc | rc | 0x7F for pc in [0, 0x200, 0x300]
            for rc in [0, 0x400, 0x800, 0xC00]]
PAGES = [(0x990000, 'RX'), (0x11E0000, 'RX'), (0x13C0000, 'R'),
         (0x13E0000, 'R'), (0x1480000, 'R'), (0x785A0000, 'RX'),
         (0x78530000, 'R')]
SENTINEL = 0x7FC0BEEF


def digest(data):
    return hashlib.sha256(data).hexdigest()


def audit_source(image):
    from capstone import CS_GRP_CALL, CS_GRP_JUMP, CS_AC_WRITE
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    allowed = set('sub mov fld fst fmul fmulp faddp fxch fsqrt fstp movss '
                  'test je ucomiss lahf jnp divss movaps mulss jmp add ret '
                  'addss xorps comiss jbe subss ja call fsin fld1 fdivrp fsub'.split())
    records, boundaries = [], set()
    decoded = []
    for start, end in SPANS:
        code = list(crt.decoder().disasm(image.read(start, end-start), start))
        assert sum(i.size for i in code) == end-start
        boundaries.update(i.address for i in code)
        decoded.append(code)
    for (start, end), code in zip(SPANS, decoded):
        calls, branches, reads, pointers, stack = [], [], [], [], []
        for i in code:
            assert i.mnemonic in allowed, (hex(i.address), i.mnemonic)
            if i.group(CS_GRP_CALL) or i.group(CS_GRP_JUMP):
                assert i.operands[0].type == X86_OP_IMM
                target = i.operands[0].imm
                if i.group(CS_GRP_CALL):
                    assert i.address == 0x99CA88 and target == THUNK[0]
                    calls.append(dict(at=f'{i.address:08X}', target=f'{target:08X}'))
                else:
                    assert start <= target < end and target in boundaries
                    branches.append(dict(at=f'{i.address:08X}', target=f'{target:08X}'))
            for value in i.operands:
                if value.type != X86_OP_MEM:
                    continue
                m = value.mem
                assert not m.index
                base = i.reg_name(m.base)
                row = dict(at=f'{i.address:08X}', operand=i.op_str, bytes=value.size,
                           write=bool(value.access & CS_AC_WRITE))
                if base == 'esp':
                    assert 0 <= m.disp <= 0x44
                    stack.append(row)
                elif base in {'eax', 'ecx'}:
                    assert m.disp in {0, 4, 8, 12}
                    pointers.append(dict(**row, register=base, displacement=m.disp))
                else:
                    assert not base and not row['write']
                    address = m.disp & 0xFFFFFFFF
                    assert any(a <= address and address+value.size <= a+n for a, n in CONSTANTS)
                    reads.append(dict(**row, address=f'{address:08X}'))
        records.append(dict(start=f'{start:08X}', end_exclusive=f'{end:08X}',
            bytes=end-start, sha256=digest(image.read(start, end-start)),
            calls=calls, relative_jumps=branches, absolute_constant_reads=reads,
            argument_pointer_operands=pointers, stack_operands=stack,
            disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in code]))
    thunk = image.read(THUNK[0], THUNK[1]-THUNK[0])
    assert thunk == b'\xFF\x25'+struct.pack('<I', IAT)
    actual = {a: image.read(a, n) for a, n in CONSTANTS}
    assert actual[0x13EB1BC] == struct.pack('<I', 0xBF800000)
    assert actual[0x13EB8B0] == struct.pack('<4I', *([0x80000000]*4))
    assert actual[0x13EBCA8] == struct.pack('<I', 0x38D1B717)
    assert actual[0x1485E24] == struct.pack('<I', 0)
    assert actual[0x14861D0] == struct.pack('<I', 0x3F800000)
    return records, dict(start=f'{THUNK[0]:08X}', end_exclusive=f'{THUNK[1]:08X}',
        bytes=len(thunk), sha256=digest(thunk),
        instruction='JMP DWORD PTR [013CC48C]', fixture_iat_target='785A35E0')


def fbits(value):
    return struct.unpack('<I', struct.pack('<f', value))[0]


def corpus():
    """Deterministic bit-pattern inputs, including nonunit/zero/subnormal rows."""
    cases = set()
    zero, identity = (0, 0, 0, 0), (0, 0, 0, 0x3F800000)
    normalize_values = set(itertools.product([0, 0x80000000], repeat=4))
    edges = [0, 1, 0x007FFFFF, 0x00800000, 0x00800001, 0x0D800000,
             0x20000000, 0x3EFFFFFF, 0x3F000000, 0x3F3504F3, 0x3F7FFFFF,
             0x3F800000, 0x3F800001, 0x40000000, 0x40800000]
    for bit in edges:
        for sign in [0, 0x80000000]:
            for component in range(4):
                row = list(zero)
                row[component] = bit | sign
                normalize_values.add(tuple(row))
                row[(component+1) % 4] = 0x3F800000
                normalize_values.add(tuple(row))
    state = 0xCABBA6E5
    for _ in range(512):
        value = []
        for component in range(4):
            state = (state*1664525+1013904223) & 0xFFFFFFFF
            # Wide finite exponents, bounded |component|<4. No unit filter.
            value.append((state & 0x807FFFFF) | (((state >> 24) % 129) << 23))
        normalize_values.add(tuple(value))
    for row in sorted(normalize_values):
        for null_length in [0, 1]:
            cases.add((0, row, zero, 0, null_length))

    pairs = set()
    seeds = [zero, identity, (0, 0, 0, 0xBF800000), (0x3F800000, 0, 0, 0),
             (0, 0x3F800000, 0, 0), (0, 0, 0x3F800000, 0),
             tuple(fbits(x) for x in [.25, -.5, .75, -1.25]),
             tuple(fbits(x) for x in [4., -4., 4., -4.]),
             tuple(fbits(x) for x in [-0., 0., -0., 0.]), (1, 0x80000001, 0, 0)]
    pairs.update(itertools.product(seeds, repeat=2))
    # With left.w=1 these hit clamp, sign, threshold, CRT and zero-dot edges.
    for magnitude in [0, 1, 0x007FFFFF, 0x00800000, 0x33800000, 0x3D800000,
                      0x3F000000, 0x3F5D8000, 0x3F7FE000, 0x3F7FF971,
                      0x3F7FF972, 0x3F7FF973, 0x3F7FFFFF, 0x3F800000,
                      0x3F800001, 0x40000000, 0x40800000]:
        for sign in [0, 0x80000000]:
            pairs.add((identity, (0, 0, 0, magnitude | sign)))
    for _ in range(384):
        values = []
        for component in range(8):
            state = (state*1664525+1013904223) & 0xFFFFFFFF
            values.append(fbits(((state >> 8) % 8193-4096)/2048))
        pairs.add((tuple(values[:4]), tuple(values[4:])))
    fractions = [0, 0x80000000, 1, 0x00800000, 0x33800000, 0x3E800000,
                 0x3F000000, 0x3F400000, 0x3F7FFFFF, 0x3F800000]
    for left, right in sorted(pairs):
        cases.add((1, left, right, 0, 0))
        for fraction in fractions:
            cases.add((2, left, right, fraction, 0))
    rows = []
    for operation, left, right, amount, flags in sorted(cases):
        for control in CONTROLS:
            rows.append((operation, control, *left, *right, amount, flags))
    return rows


def host_source(image, dll):
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none']
    for name, size in [('GetStdHandle', 4), ('ReadFile', 20), ('WriteFile', 20),
        ('VirtualAlloc', 16), ('VirtualProtect', 16), ('GetLastError', 0),
        ('FlushInstructionCache', 12), ('ExitProcess', 4)]:
        lines.append(f'EXTERN _{name}@{size}:PROC')
    lines += ['.const']
    for start, end in SPANS+[THUNK]:
        lines += crt.db(f'Source_{start:08X}', image.read(start, end-start))
    for address, count in CONSTANTS:
        lines += crt.db(f'Const_{address:08X}', image.read(address, count))
    for start, end in crt.CODE:
        lines += crt.db(f'Kernel_{start:08X}', dll.read(start, end-start))
    lines += crt.db('OriginalCrtConstants', dll.read(crt.WINDOW[0], crt.WINDOW[1]-crt.WINDOW[0]))
    lines += ['.data', 'InputBuffer BYTE 48 DUP(0)', 'ExpectedValue BYTE 20 DUP(0)',
        'ReplyBefore DWORD 4 DUP(0A5A5A5A5h)', 'ReplyBuffer BYTE 32 DUP(0)',
        'ReplyAfter DWORD 4 DUP(0A5A5A5A5h)', 'InputHandle DWORD 0', 'OutputHandle DWORD 0',
        'Transferred DWORD 0', 'OldProtect DWORD 0', 'SseControl DWORD 01F80h',
        'SavedStack DWORD 0', 'Poison DWORD 0', '.code',
        'PUBLIC ReadRecord, ReadRecordEnd, ValidateRecord, ValidateRecordEnd, RunOnce, RunOnceEnd, HostMain, HostEnd, HostFailure',
        'ReadRecord PROC', 'push ebx', 'push esi', 'mov ebx,48', 'mov esi,OFFSET InputBuffer',
        'ReadLoop:', 'mov Transferred,0', 'push 0', 'push OFFSET Transferred', 'push ebx',
        'push esi', 'push InputHandle', 'call _ReadFile@20', 'test eax,eax', 'jnz ReadTransferred',
        'call _GetLastError@0', 'cmp eax,109', 'jne HostFailure', 'cmp ebx,48',
        'jne HostFailure', 'cmp Transferred,0', 'jne HostFailure', 'xor eax,eax', 'jmp ReadDone',
        'ReadTransferred:', 'mov eax,Transferred', 'test eax,eax', 'jnz ReadSome',
        'cmp ebx,48', 'jne HostFailure', 'xor eax,eax', 'jmp ReadDone', 'ReadSome:',
        'add esi,eax', 'sub ebx,eax', 'jnz ReadLoop', 'mov eax,1', 'ReadDone:',
        'pop esi', 'pop ebx', 'ret', 'ReadRecord ENDP', 'ReadRecordEnd LABEL BYTE',
        'ValidateRecord PROC', 'cmp dword ptr [InputBuffer],2', 'ja HostFailure',
        # Exact masked PC24/53/64 x RC-nearest/down/up/zero admission.
        'mov eax,dword ptr [InputBuffer+4]', 'mov edx,eax', 'and eax,0FFFFF0FFh',
        'cmp eax,07Fh', 'jne HostFailure', 'and edx,0300h', 'cmp edx,0100h', 'je HostFailure',
        # All source memory reads remain in finite, |component|<=4 inputs.
        'mov ecx,8', 'mov edx,OFFSET InputBuffer+8', 'ValidateComponent:',
        'mov eax,dword ptr [edx]', 'and eax,07FFFFFFFh', 'cmp eax,040800000h', 'ja HostFailure',
        'add edx,4', 'dec ecx', 'jnz ValidateComponent',
        'mov eax,dword ptr [InputBuffer+40]', 'mov edx,eax', 'and eax,07FFFFFFFh',
        'cmp eax,03F800000h', 'ja HostFailure', 'test edx,080000000h', 'jz ValidFraction',
        'test eax,eax', 'jnz HostFailure', 'ValidFraction:',
        'cmp dword ptr [InputBuffer+44],1', 'ja HostFailure',
        'cmp dword ptr [InputBuffer+44],0', 'je ValidFlags',
        'cmp dword ptr [InputBuffer],0', 'jne HostFailure', 'ValidFlags:',
        'ret', 'ValidateRecord ENDP', 'ValidateRecordEnd LABEL BYTE',
        'RunOnce PROC', 'mov SavedStack,esp', 'fninit',
        'fldcw word ptr [InputBuffer+4]', 'ldmxcsr SseControl',
        f'mov dword ptr [ReplyBuffer+16],0{SENTINEL:08X}h',
        'mov dword ptr [ReplyBuffer+20],0A5A5A5A5h',
        'mov dword ptr [ReplyBuffer+24],0A5A5A5A5h',
        'mov dword ptr [ReplyBuffer+28],0A5A5A5A5h',
        'cmp Poison,0', 'jne PoisonOnes']
    for i in range(8):
        lines += [f'pxor xmm{i},xmm{i}']
    lines += ['jmp ScratchReady', 'PoisonOnes:']
    for i in range(8):
        lines += [f'pcmpeqd xmm{i},xmm{i}']
    lines += ['ScratchReady:', 'cmp dword ptr [InputBuffer],0', 'jne NotNormalize',
        'cmp dword ptr [InputBuffer+44],0', 'jne NullLength',
        'push OFFSET ReplyBuffer+16', 'jmp LengthReady', 'NullLength:', 'push 0',
        'LengthReady:', 'push OFFSET InputBuffer+8', 'push OFFSET ReplyBuffer',
        'mov eax,099C810h', 'call eax', 'add esp,12', 'jmp OriginalDone',
        'NotNormalize:', 'cmp dword ptr [InputBuffer],1', 'jne Spherical',
        'push OFFSET InputBuffer+24', 'push OFFSET InputBuffer+8', 'push OFFSET ReplyBuffer',
        'mov eax,099C8D0h', 'call eax', 'add esp,12', 'jmp OriginalDone',
        'Spherical:', 'push dword ptr [InputBuffer+40]', 'push OFFSET InputBuffer+24',
        'push OFFSET InputBuffer+8', 'push OFFSET ReplyBuffer',
        'mov eax,099C9A0h', 'call eax', 'add esp,16', 'OriginalDone:',
        'cmp esp,SavedStack', 'jne HostFailure',
        'cmp dword ptr [ReplyBuffer+20],0A5A5A5A5h', 'jne HostFailure',
        'cmp dword ptr [ReplyBuffer+24],0A5A5A5A5h', 'jne HostFailure',
        'cmp dword ptr [ReplyBuffer+28],0A5A5A5A5h', 'jne HostFailure',
        'mov dword ptr [ReplyBuffer+28],0',
        'stmxcsr dword ptr [ReplyBuffer+20]',
        'fnstcw word ptr [ReplyBuffer+24]', 'fnstsw word ptr [ReplyBuffer+26]',
        'mov ax,word ptr [InputBuffer+4]', 'cmp word ptr [ReplyBuffer+24],ax', 'jne HostFailure',
        'movzx eax,word ptr [ReplyBuffer+26]', 'and eax,03800h', 'jnz HostFailure',
        'cmp dword ptr [InputBuffer],2', 'jne C2Ready',
        'test word ptr [ReplyBuffer+26],0400h', 'jnz HostFailure', 'C2Ready:',
        'mov eax,dword ptr [ReplyBuffer+20]', 'and eax,0FFFFFFC0h', 'cmp eax,01F80h',
        'jne HostFailure', 'ret', 'RunOnce ENDP', 'RunOnceEnd LABEL BYTE', 'HostMain PROC']
    for page, _ in PAGES:
        lines += ['push 04h', 'push 03000h', 'push 010000h', f'push 0{page:08X}h',
            'call _VirtualAlloc@16', f'cmp eax,0{page:08X}h', 'jne HostFailure']
    # Every unlisted source-code byte is a trapping gap. Data allocation is
    # initially zero; only the enumerated constants and explicit IAT are set.
    for page, protection in PAGES:
        if protection == 'RX':
            lines += [f'mov edi,0{page:08X}h', 'mov ecx,010000h', 'mov al,0CCh', 'cld', 'rep stosb']
    for start, end in SPANS+[THUNK]:
        lines += [f'mov esi,OFFSET Source_{start:08X}', f'mov edi,0{start:08X}h',
                  f'mov ecx,{end-start}', 'rep movsb']
    for address, count in CONSTANTS:
        lines += [f'mov esi,OFFSET Const_{address:08X}', f'mov edi,0{address:08X}h',
                  f'mov ecx,{count}', 'rep movsb']
    lines += [f'mov dword ptr [0{IAT:08X}h],0785A35E0h']
    for start, end in crt.CODE:
        lines += [f'mov esi,OFFSET Kernel_{start:08X}', f'mov edi,0{start:08X}h',
                  f'mov ecx,{end-start}', 'rep movsb']
    lines += ['mov esi,OFFSET OriginalCrtConstants', f'mov edi,0{crt.WINDOW[0]:08X}h',
              f'mov ecx,{crt.WINDOW[1]-crt.WINDOW[0]}', 'rep movsb']
    for page, protection in PAGES:
        access = '020h' if protection == 'RX' else '02h'
        lines += ['push OFFSET OldProtect', f'push {access}', 'push 010000h',
                  f'push 0{page:08X}h', 'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure']
    lines += ['push 0', 'push 0', 'push -1', 'call _FlushInstructionCache@12', 'test eax,eax',
        'jz HostFailure', 'push -10', 'call _GetStdHandle@4', 'mov InputHandle,eax',
        'push -11', 'call _GetStdHandle@4', 'mov OutputHandle,eax', 'NextRecord:',
        'call ReadRecord', 'test eax,eax', 'jz HostSuccess', 'call ValidateRecord',
        'mov Poison,0', 'call RunOnce']
    for offset in range(0, 20, 4):
        lines += [f'mov eax,dword ptr [ReplyBuffer+{offset}]', f'mov dword ptr [ExpectedValue+{offset}],eax']
    lines += ['mov Poison,1', 'call RunOnce']
    for offset in range(0, 20, 4):
        lines += [f'mov eax,dword ptr [ReplyBuffer+{offset}]',
                  f'cmp eax,dword ptr [ExpectedValue+{offset}]', 'jne HostFailure']
    for label in ['ReplyBefore', 'ReplyAfter']:
        for offset in range(0, 16, 4):
            lines += [f'cmp dword ptr [{label}+{offset}],0A5A5A5A5h', 'jne HostFailure']
    lines += ['push 0', 'push OFFSET Transferred', 'push 32', 'push OFFSET ReplyBuffer',
        'push OutputHandle', 'call _WriteFile@20', 'test eax,eax', 'jz HostFailure',
        'cmp Transferred,32', 'jne HostFailure', 'jmp NextRecord', 'HostSuccess:',
        'push 0', 'call _ExitProcess@4', 'HostFailure::', 'push 7', 'call _ExitProcess@4',
        'HostMain ENDP', 'HostEnd LABEL BYTE', 'END HostMain']
    return '\n'.join(lines)+'\n'


def audit_host(exe, map_path):
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_REG, X86_REG_EAX
    wanted = {'ReadRecord', 'ReadRecordEnd', 'ValidateRecord', 'ValidateRecordEnd',
              'RunOnce', 'RunOnceEnd', 'HostMain', 'HostEnd', 'HostFailure'}
    symbols = {}
    for line in map_path.read_text().splitlines():
        match = re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\w+)\s+([0-9A-Fa-f]{8})\b', line)
        if match and match[1] in wanted:
            symbols[match[1]] = int(match[2], 16)
    assert symbols.keys() == wanted, symbols
    image = crt.PE(exe)
    imports = image.imports()
    assert {dll.upper() for dll, _ in imports.values()} == {'KERNEL32.DLL'}
    assert {name for _, name in imports.values()} == crt.IMPORTS
    start, end = symbols['ReadRecord'], symbols['HostEnd']
    code = list(crt.decoder().disasm(image.read(start, end-start), start))
    assert sum(i.size for i in code) == end-start
    boundaries = {i.address for i in code}
    calls, branches = [], []
    for n, i in enumerate(code):
        assert i.mnemonic not in {'int', 'syscall', 'sysenter', 'hlt', 'iret', 'iretd'}
        if i.group(CS_GRP_JUMP):
            assert i.operands[0].type == X86_OP_IMM and i.operands[0].imm in boundaries
            branches.append(dict(at=f'{i.address:08X}', target=f'{i.operands[0].imm:08X}'))
        if not i.group(CS_GRP_CALL):
            continue
        operand = i.operands[0]
        if operand.type == X86_OP_IMM:
            target = operand.imm
            if target in {symbols['ReadRecord'], symbols['ValidateRecord'], symbols['RunOnce']}:
                calls.append(dict(at=f'{i.address:08X}', target=f'{target:08X}', kind='owned helper'))
            else:
                thunk = image.read(target, 6)
                assert thunk[:2] == b'\xFF\x25'
                dll, name = imports[struct.unpack('<I', thunk[2:])[0]]
                calls.append(dict(at=f'{i.address:08X}', kind='allowlisted host import', dll=dll, name=name))
        else:
            assert operand.type == X86_OP_REG and operand.reg == X86_REG_EAX
            previous = code[n-1]
            assert previous.mnemonic == 'mov' and previous.operands[0].reg == X86_REG_EAX
            assert previous.operands[1].type == X86_OP_IMM
            target = previous.operands[1].imm
            assert target in {a for a, _ in SPANS}
            calls.append(dict(at=f'{i.address:08X}', target=f'{target:08X}', kind='original primitive'))
    assert {c['target'] for c in calls if c['kind'] == 'original primitive'} == {f'{a:08X}' for a, _ in SPANS}
    return dict(start=f'{start:08X}', end_exclusive=f'{end:08X}',
        sha256=digest(image.read(start, end-start)),
        symbols={k: f'{v:08X}' for k, v in symbols.items()},
        imported_functions=[dict(iat=f'{slot:08X}', dll=dll, name=name)
                            for slot, (dll, name) in sorted(imports.items())],
        calls=calls, relative_jumps=branches,
        disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in code])


def prepare(args):
    output = args.out.resolve()
    output.relative_to((ROOT/'target').resolve())
    image, dll = SourceImage(args.exe), crt.PE(args.dll)
    assert digest(dll.data) == crt.SOURCE_SHA256 and dll.base == 0x78520000
    constants = dll.read(crt.WINDOW[0], crt.WINDOW[1]-crt.WINDOW[0])
    assert digest(constants) == crt.WINDOW_SHA256
    crt.decoder()
    source_code, thunk = audit_source(image)
    crt_code, bounds = crt.audit_source(dll)
    rows = corpus()
    data = b''.join(struct.pack('<12I', *row) for row in rows)
    tsv = ''.join(f'{n:06d}\t'+ ' '.join(f'{v:08x}' for v in row)+'\n'
                  for n, row in enumerate(rows)).encode('ascii')
    output.mkdir(parents=True, exist_ok=False)
    (output/'inputs.bin').write_bytes(data)
    (output/'inputs.tsv').write_bytes(tsv)
    (output/'constants.bin').write_bytes(constants)
    assembly, obj, exe = [output/name for name in ['host-x86.asm', 'host-x86.obj', 'host-x86.exe']]
    map_path = output/'host-x86.map'
    assembly.write_text(host_source(image, dll))
    commands = [
        [str(args.sdk_bin_x86/'ml.exe'), '/nologo', '/c', '/coff', f'/Fo{obj}', str(assembly)],
        [str(args.sdk_bin_x86/'link.exe'), '/NOLOGO', '/MACHINE:X86', '/SUBSYSTEM:CONSOLE',
         '/ENTRY:HostMain', '/BASE:0x24000000', '/DYNAMICBASE:NO', '/SAFESEH:NO',
         f'/OUT:{exe}', f'/MAP:{map_path}', str(obj), str(args.kernel32_lib)],
    ]
    for command in commands:
        result = subprocess.run(command, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout+'\n'+result.stderr
    owned = audit_host(exe, map_path)
    manifest = dict(schema='bounded-original-quaternion-host-v1', host_executed=False,
        source_path=str(args.exe.resolve()), source_sha256=SOURCE_SHA256,
        crt_source_path=str(args.dll.resolve()), crt_source_sha256=crt.SOURCE_SHA256,
        source_code_allowlist=source_code, source_thunk=thunk,
        source_constants=[dict(address=f'{a:08X}', bytes=n, sha256=digest(image.read(a, n)))
                          for a, n in CONSTANTS],
        crt_code_allowlist=crt_code, crt_indexed_read_bound_proof=bounds,
        crt_constant_window=dict(start=f'{crt.WINDOW[0]:08X}', end_exclusive=f'{crt.WINDOW[1]:08X}',
                                 bytes=len(constants), sha256=crt.WINDOW_SHA256),
        arithmetic_policy='Explicit masked PC24/53/64 x all four RC modes, MXCSR1F80; finite |components|<=4, fraction[-0,+1]. No retail dispatch/state claim.',
        slerp_range_proof='Integer-admitted finite components bounded by4 make SSE dot finite. Source clamps abs(dot) into[-0,+1]. Original finite CRT returns theta in[0,pi/2]; fraction[-0,+1] bounds all three FSIN arguments below2. Near-dot branch bypasses CRT/FSIN. C2=0 is also checked after every slerp.',
        memory_bound_proof='Only original source reads through fixed local quaternion buffers, their four offsets0/4/8/12, bounded own stack locals, listed readonly constants, and readonly explicit IAT. Original output points to first16 bytes of a20-byte result region. Normalize optional length output points at result+16 or NULL. Prefix16-byte and final16-byte canaries are checked after both calls. The next12 bytes at result+20/+24/+28 are separately set to sentinels before each call and checked immediately after it, before context stores may overwrite those bytes. Native cdecl argument stack writes do not alias request bytes.',
        omitted_code_policy='All source/CRT code pages INT3-filled, then only listed unmodified spans copied. No original module loaded, startup/dispatch/domain/unrelated functions absent.',
        fixture_binding=dict(iat='013CC48C', target='785A35E0', policy='Reviewed finite SSE2 wrapper; source thunk unchanged, fixture IAT data supplied explicitly.'),
        pages=[dict(address=f'{a:08X}', bytes=65536, initial='RW', final=p) for a, p in PAGES],
        host_image_base='24000000', host_code_audit=owned,
        host_source_sha256=digest(assembly.read_bytes()), host_exe_sha256=digest(exe.read_bytes()),
        compile_commands=commands, original_modules_loaded=False,
        protocol=dict(input_bytes=48, input_fields=['operation:u32@0(0=normalize,1=hemisphere-add,2=slerp)',
            'control_word:u32@4', 'left:f32x4@8', 'right:f32x4@24', 'fraction:f32@40',
            'flags:u32@44(0,or1=normalize NULL length)'],
            output_bytes=32, output_fields=['quaternion:f32x4@0', 'length:f32@16(or7FC0BEEF sentinel)',
                'post_MXCSR:u32@20', 'post_CW:u16@24', 'post_SW:u16@26', 'reserved:u32@28=0'],
            per_input_calls=2, scratch_lane_patterns=['all-zero', 'all-one'],
            checks=['both result/length bit-identical', 'x87 TOP restored to zero', 'requested CW unchanged',
                'MXCSR control bits remain1F80', 'slerp C2 cleared', 'cdecl ESP restored',
                'all12 context/reserved guard bytes preserved before context stores', 'reply canaries preserved'],
            nonfinite_output_policy='Finite tiny normalization may produce native SSE infinities/NaNs after reciprocal overflow. Retain raw native output; Rust bounded API reports NonFiniteOutput and comparison records these separately, without claiming finite bit conformance for excluded results.',
            input_rows=len(rows), operation_counts=dict(Counter(row[0] for row in rows)),
            control_word_counts={f'{k:04X}': v for k, v in Counter(row[1] for row in rows).items()},
            input_sha256=digest(data), input_tsv_sha256=digest(tsv)),
        next_step='Root must review the fresh exact manifest/source/audits/hashes before a separate execution action.')
    (output/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps({k: manifest[k] for k in ['source_sha256', 'host_executed',
        'host_source_sha256', 'host_exe_sha256', 'protocol']}, indent=2))


def compare(args):
    fixture = args.fixture.resolve()
    manifest = json.loads((fixture/'manifest.json').read_text())
    data = (fixture/'inputs.bin').read_bytes()
    assert digest(data) == manifest['protocol']['input_sha256']
    inputs = list(struct.iter_unpack('<12I', data))
    native, rust = args.native.read_bytes(), args.rust.read_bytes()
    assert len(native) == len(inputs)*32 and len(rust) == len(inputs)*24
    expected, exclusions, problems = bytearray(), [], []
    context_counts = Counter()
    for n, row in enumerate(inputs):
        op, control, *unused = row
        values = struct.unpack_from('<5I', native, n*32)
        mxcsr, cw, sw, reserved = struct.unpack_from('<IHHI', native, n*32+20)
        assert cw == control and sw & 0x3800 == 0 and mxcsr & ~0x3F == 0x1F80 and reserved == 0
        if op == 2:
            assert sw & 0x0400 == 0
        context_counts[(cw, mxcsr, sw)] += 1
        nonfinite = any(v & 0x7F800000 == 0x7F800000 for v in values[:4])
        if nonfinite:
            assert op == 0
            packed = struct.pack('<6I', 0, 0, 0, 0, 0, 1)
            exclusions.append(n)
        else:
            packed = struct.pack('<6I', *values, 0)
        expected += packed
        if rust[n*24:(n+1)*24] != packed:
            problems.append(dict(row=n, operation=op, control_word=f'{cw:04X}',
                input=[f'{v:08X}' for v in row], expected=packed.hex(),
                actual=rust[n*24:(n+1)*24].hex()))
    report = dict(schema='original-quaternion-conformance-comparison-v1', rows=len(inputs),
        finite_result_rows=len(inputs)-len(exclusions), nonfinite_normalization_rows=len(exclusions),
        nonfinite_normalization_indices=exclusions, input_sha256=digest(data),
        native_output_sha256=digest(native), expected_result_sha256=digest(expected),
        rust_result_sha256=digest(rust), exact=not problems, problems=problems,
        source_sha256=SOURCE_SHA256, crt_source_sha256=crt.SOURCE_SHA256,
        host_exe_sha256=manifest['host_exe_sha256'],
        source_context_counts=[dict(cw=f'{cw:04X}', mxcsr=f'{mxcsr:08X}', sw=f'{sw:04X}', rows=count)
                               for (cw, mxcsr, sw), count in sorted(context_counts.items())],
        scope='Bounded explicit source primitives plus finite SSE2 CRT binding, not retail dispatch, full authored rotation channel evaluation or an assembled creature animation.')
    args.report.resolve().relative_to((ROOT/'target').resolve())
    with args.report.open('x') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps({k: v for k, v in report.items() if k not in {'problems', 'source_context_counts', 'nonfinite_normalization_indices'}}, indent=2))
    if problems:
        raise AssertionError(problems[:5])


def audit_rational(args):
    """Third oracle for normalize/add/linear slerp; no software FSIN claim."""
    from anim_body_pose_probe import arithmetic
    from native_x87_probe import operand, quantize, sqrt_quantize, commit
    fixture = args.fixture.resolve()
    manifest = json.loads((fixture/'manifest.json').read_text())
    data = (fixture/'inputs.bin').read_bytes()
    assert digest(data) == manifest['protocol']['input_sha256']
    rows = list(struct.iter_unpack('<12I', data))
    native = args.native.read_bytes()
    assert len(native) == len(rows)*32
    counts, paths, reductions = Counter(), Counter(), Counter()
    problems, cache = [], {}

    def dot(left, right):
        products = [arithmetic('mulss', r, l) for l, r in zip(left, right)]
        value = products[0]
        for product in products[1:]:
            value = arithmetic('addss', value, product)
        return value

    for n, row in enumerate(rows):
        operation, control = row[:2]
        left, right = row[2:6], row[6:10]
        actual = struct.unpack_from('<5I', native, n*32)
        if operation == 0:
            precision = {0: 24, 0x200: 53, 0x300: 64}[control & 0x300]
            mode = (control >> 10) & 3
            x, y, z, w = map(operand, left)
            q = lambda v: quantize(v, precision, mode)[0]
            value = q(w*w)
            value = q(q(x*x)+value)
            value = q(q(y*y)+value)
            value = q(q(z*z)+value)
            length = commit(sqrt_quantize(value, precision, mode)[0], mode) if value else 0
            expected_length = length if row[11] == 0 else SENTINEL
            if not length:
                expected = left
            else:
                reciprocal = arithmetic('divss', 0x3F800000, length)
                if reciprocal == 0x7F800000:
                    counts['normalize-nonfinite-reciprocal'] += 1
                    if (not any(v & 0x7F800000 == 0x7F800000 for v in actual[:4])
                            or actual[4] != expected_length):
                        problems.append(dict(row=n, reason='reciprocal-overflow boundary differs'))
                    continue
                expected = tuple(arithmetic('mulss', reciprocal, v) for v in left)
            expected = (*expected, expected_length)
            counts['normalize-finite'] += 1
        else:
            key = (operation, left, right, row[10])
            if key not in cache:
                raw_dot = dot(left, right)
                if operation == 1:
                    opcode = 'subss' if operand(raw_dot) < 0 else 'addss'
                    first, second = (left, right) if opcode == 'subss' else (right, left)
                    expected = (*[arithmetic(opcode, a, b) for a, b in zip(first, second)], SENTINEL)
                    cache[key] = ('hemisphere-add', expected, None)
                else:
                    clamped = (0xBF800000 if operand(raw_dot) <= -1 else
                               0x3F800000 if operand(raw_dot) > 1 else raw_dot)
                    negative = operand(clamped) < 0
                    absolute = clamped ^ 0x80000000 if negative else clamped
                    gap = arithmetic('subss', 0x3F800000, absolute)
                    near = operand(gap) <= operand(0x38D1B717)
                    if near:
                        first = arithmetic('subss', 0x3F800000, row[10])
                        first = arithmetic('mulss', first, 0xBF800000 if negative else 0x3F800000)
                        expected = (*[arithmetic('addss', arithmetic('mulss', a, first),
                                                arithmetic('mulss', b, row[10]))
                                      for a, b in zip(left, right)], SENTINEL)
                        cache[key] = ('slerp-near-finite', expected, 'bypassed')
                    else:
                        # Source bounded theta/FSIN is checked by the actual
                        # original-byte fixture, not a mathematical sine here.
                        binary64 = struct.unpack('<Q', struct.pack('<d', float(operand(absolute))))[0]
                        cache[key] = ('slerp-spherical-not-rational', None, crt.branch(binary64))
            label, expected, reduction = cache[key]
            counts[label] += 1
            if operation == 2:
                paths['near-dot-linear' if expected is not None else 'spherical'] += 1
                reductions[reduction] += 1
            if expected is None:
                continue
        if tuple(actual) != tuple(expected):
            problems.append(dict(row=n, operation=operation, control_word=f'{control:04X}',
                expected=[f'{v:08X}' for v in expected], actual=[f'{v:08X}' for v in actual]))
    report = dict(schema='native-quaternion-rational-crosscheck-v1', rows=len(rows),
        evaluated_finite_rows=sum(counts[k] for k in ['normalize-finite', 'hemisphere-add', 'slerp-near-finite']),
        evaluated_nonfinite_boundary_rows=counts['normalize-nonfinite-reciprocal'],
        classified_without_evaluating_rows=counts['slerp-spherical-not-rational'],
        exact_for_evaluated_rows=not problems, problems=problems, operation_counts=dict(counts),
        input_derived_slerp_paths=dict(paths), input_derived_crt_reductions=dict(reductions),
        input_sha256=digest(data), native_output_sha256=digest(native), source_sha256=SOURCE_SHA256,
        scope='Integer/Fraction source-order PC/RC normalize, sqrt/store and exact integer SSE reciprocal/products, hemisphere add and near-dot slerp. No native floating arithmetic supplies expected finite bits. Spherical source paths are classified but not rationally evaluated; hardware FSIN is checked separately against Rust.')
    args.report.resolve().relative_to((ROOT/'target').resolve())
    with args.report.open('x') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps({k: v for k, v in report.items() if k != 'problems'}, indent=2))
    if problems:
        raise AssertionError(problems[:5])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    p = sub.add_parser('prepare', help='read, audit, assemble/link only; never execute')
    p.add_argument('exe', type=Path)
    p.add_argument('dll', type=Path)
    p.add_argument('--sdk-bin-x86', type=Path, required=True)
    p.add_argument('--kernel32-lib', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    c = sub.add_parser('compare', help='compare already-produced binary outputs; never execute')
    c.add_argument('fixture', type=Path)
    c.add_argument('--native', type=Path, required=True)
    c.add_argument('--rust', type=Path, required=True)
    c.add_argument('--report', type=Path, required=True)
    r = sub.add_parser('audit-rational', help='independent exact integer/Fraction cross-check; never execute')
    r.add_argument('fixture', type=Path)
    r.add_argument('--native', type=Path, required=True)
    r.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    {'prepare': prepare, 'compare': compare, 'audit-rational': audit_rational}[args.command](args)


if __name__ == '__main__':
    main()

"""Prepare, never execute, an allowlisted finite original-CRT SSE2 fixture.

Original code/data, generated MASM, the no-CRT host and audit stay in target/.
The source DLL is only read. Execution requires a separate reviewed action.
"""
import argparse
from collections import Counter
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

SOURCE_SHA256 = 'ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185'
WINDOW_SHA256 = '76210f1fef0a723d62dbcf92c1a7ce9ce1697b99f6a14530692fa8e3871f07df'
WINDOW = (0x78534400, 0x78536050)
# The intrinsic wrapper plus finite kernel spans. All domain/NaN handlers,
# ordinary-acos entry, legacy dispatch and unrelated code remain INT3 gaps.
CODE = [(0x785a35e0, 0x785a35f8), (0x785a35fe, 0x785a3a38), (0x785a3abd, 0x785a3b22)]
IMPORTS = {'GetStdHandle', 'ReadFile', 'WriteFile', 'VirtualAlloc', 'VirtualProtect',
           'FlushInstructionCache', 'GetLastError', 'ExitProcess'}
MASK64 = (1 << 64) - 1
ROOT = Path(__file__).resolve().parent.parent


class PE:
    def __init__(self, path):
        self.path = Path(path)
        self.data = self.path.read_bytes()
        pe = struct.unpack_from('<I', self.data, 0x3c)[0]
        assert self.data[pe:pe+4] == b'PE\0\0'
        assert struct.unpack_from('<H', self.data, pe+4)[0] == 0x14c
        optional = pe+24
        assert struct.unpack_from('<H', self.data, optional)[0] == 0x10b
        self.base = struct.unpack_from('<I', self.data, optional+28)[0]
        self.directories = [struct.unpack_from('<II', self.data, optional+96+8*i) for i in range(16)]
        count, = struct.unpack_from('<H', self.data, pe+6)
        size, = struct.unpack_from('<H', self.data, pe+20)
        self.sections = []
        for i in range(count):
            at = optional+size+40*i
            vs, rva, length, raw = struct.unpack_from('<4I', self.data, at+8)
            self.sections.append((self.base+rva, raw, length, vs,
                                  self.data[at:at+8].split(b'\0')[0].decode('ascii')))

    def read(self, address, length):
        for va, raw, size, virtual, _ in self.sections:
            if va <= address and address+length <= va+max(size, virtual):
                offset = address-va
                available = max(0, min(length, size-offset))
                return self.data[raw+offset:raw+offset+available]+b'\0'*(length-available)
        raise ValueError(f'PE address outside sections: {address:08X}+{length}')

    def u32(self, address):
        return struct.unpack('<I', self.read(address, 4))[0]

    def text(self, address):
        data = bytearray()
        for i in range(512):
            char = self.read(address+i, 1)[0]
            if not char:
                return data.decode('ascii')
            data.append(char)
        raise ValueError('unbounded PE string')

    def imports(self):
        out = {}
        rva, size = self.directories[1]
        assert rva and size
        for i in range(size // 20):
            descriptor = struct.unpack('<5I', self.read(self.base+rva+20*i, 20))
            if not any(descriptor):
                return out
            source, _, _, name, iat = descriptor
            dll = self.text(self.base+name)
            for j in range(256):
                entry = self.u32(self.base+(source or iat)+4*j)
                if not entry:
                    break
                assert not entry & 0x80000000, 'ordinal import not allowlisted'
                out[self.base+iat+4*j] = (dll, self.text(self.base+entry+2))
            else:
                raise ValueError('unbounded import list')
        raise ValueError('missing import descriptor terminator')

    def export(self, name):
        rva, _ = self.directories[0]
        values = struct.unpack('<IIHH7I', self.read(self.base+rva, 40))
        count, functions, names, ordinals = values[7], values[8], values[9], values[10]
        for i in range(count):
            if self.text(self.base+self.u32(self.base+names+4*i)) == name:
                index, = struct.unpack('<H', self.read(self.base+ordinals+2*i, 2))
                return self.base+self.u32(self.base+functions+4*index)
        raise ValueError(f'export not found: {name}')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def decoder():
    sys.path.insert(0, str(ROOT/'target/rea/capstone-runtime'))
    from capstone import Cs, CS_ARCH_X86, CS_MODE_32
    result = Cs(CS_ARCH_X86, CS_MODE_32)
    result.detail = True
    return result


def branch(bits):
    q = (bits >> 44) & 0x7ffff
    a = (q-0x3fb00) & 0xffffffff
    if a < 0x3bb:
        return 'table'
    a = (a-0x3bb) & 0xffffffff
    if a < 0x41:
        return 'sqrt-table'
    a = (a+0x3bbb) & 0xffffffff
    if a < 0x3800:
        return 'small'
    a = (a-0x3bfc) & 0xffffffff
    if a < 4:
        return 'near-one'
    a = (a+0x3fefc) & 0xffffffff
    return 'tiny' if a < 0x3ff00 else 'one'


def table_bounds():
    # In the sqrt/table branch, truncating 38 significand bits gives
    # 0 <= x-xhi < 2^-15. A conservative 2^-48 envelope dominates the
    # binary64 rounding errors in the few pre-sqrt operations.
    lo, hi = Fraction(443, 512), Fraction(2047, 2048)
    error = Fraction(1, 1 << 48)
    lower = 1-hi*hi-Fraction(1, 1 << 14)-error
    upper = 1-lo*lo+error
    assert lower > (Fraction(3, 100)+Fraction(1, 1 << 52))**2
    assert upper < (Fraction(51, 100)-Fraction(1, 1 << 52))**2
    # Outward-enclosing binary64 bounds for 0.03 and 0.51. No native CRT
    # arithmetic supplies this address-range proof.
    bounds = [0x3f9eb851eb851eb8, 0x3fe051eb851eb852]
    indices = [2*((((bits << 2) & MASK64) >> 48)-0xfec0) for bits in bounds]
    accesses = {
        0x785a366f: (0x78535770, 2, 0, 952, 8),
        0x785a367c: (0x78534870, 4, 0, 952, 16),
        0x785a379f: (0x78535770, 4, *indices, 8),
        0x785a37f0: (0x78534870, 8, *indices, 16),
    }
    out = {}
    for address, (base, scale, minimum, maximum, size) in accesses.items():
        start, end = base+scale*minimum, base+scale*maximum+size
        assert WINDOW[0] <= start < end <= WINDOW[1]
        out[address] = dict(base=f'{base:08X}', scale=scale, index_min=minimum,
            index_max=maximum, start_inclusive=f'{start:08X}', end_exclusive=f'{end:08X}',
            bytes_per_read=size)
    return out, dict(input_interval=['443/512', '2047/2048 (excluded)'],
        truncation_gap_upper='2^-15 (excluded)', pre_sqrt_error_envelope='2^-48',
        squared_result_lower=str(lower), squared_result_upper=str(upper),
        proven_sqrt_interval=['3/100 (strict)', '51/100 (strict)'],
        index_bounds=indices)


def audit_source(image):
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM, X86_REG_EDX, X86_REG_ESP, X86_REG_EBP
    allowed = set('push mov sub and fstp movq call leave ret movlpd xorpd movsd psrlq movd pinsrw cmp jae mulsd subsd sqrtsd andpd movapd orpd addsd divsd psllq pshufd subpd pmovmskb andnpd shr neg pextrw add pxor unpcklpd mulpd addpd cmpltsd jb or je fld'.split())
    ranges, proof = table_bounds()
    records = []
    boundaries = set()
    decoded_ranges = []
    for start, end in CODE:
        instructions = list(decoder().disasm(image.read(start, end-start), start))
        assert sum(i.size for i in instructions) == end-start
        boundaries.update(i.address for i in instructions)
        decoded_ranges.append(instructions)
    for (start, end), instructions in zip(CODE, decoded_ranges):
        calls, jumps, stack, reads, indexed = [], [], set(), [], []
        for i in instructions:
            assert i.mnemonic in allowed, (hex(i.address), i.mnemonic)
            if i.group(CS_GRP_CALL) or i.group(CS_GRP_JUMP):
                assert i.operands[0].type == X86_OP_IMM
                target = i.operands[0].imm
                assert target in boundaries, (hex(i.address), hex(target))
                if i.group(CS_GRP_CALL):
                    assert i.address == 0x785a35f1 and target == 0x785a35fe
                    calls.append(dict(at=f'{i.address:08X}', target=f'{target:08X}'))
                else:
                    jumps.append(dict(at=f'{i.address:08X}', target=f'{target:08X}'))
            for operand in i.operands:
                if operand.type != X86_OP_MEM:
                    continue
                m = operand.mem
                if m.base in {X86_REG_ESP, X86_REG_EBP}:
                    assert m.index == 0
                    stack.add(i.op_str)
                elif m.index:
                    assert m.base == 0 and m.index == X86_REG_EDX
                    bounds = ranges[i.address]
                    assert m.scale == bounds['scale']
                    assert m.disp & 0xffffffff == int(bounds['base'], 16)
                    assert operand.size == bounds['bytes_per_read']
                    indexed.append(dict(at=f'{i.address:08X}', **bounds))
                else:
                    assert m.base == 0
                    address = m.disp & 0xffffffff
                    assert WINDOW[0] <= address and address+operand.size <= WINDOW[1]
                    reads.append(dict(at=f'{i.address:08X}', address=f'{address:08X}', bytes=operand.size))
        records.append(dict(start=f'{start:08X}', end_exclusive=f'{end:08X}',
            bytes=end-start, sha256=digest(image.read(start, end-start)), calls=calls,
            relative_jumps=jumps, indirect_calls_or_jumps=0, absolute_constant_reads=reads,
            indexed_constant_reads=indexed, stack_operands=sorted(stack),
            disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in instructions]))
    return records, proof


def corpus():
    values = {0, 1, 0x8000000000000000, 0x000fffffffffffff, 0x0010000000000000,
              0x3ff0000000000000}
    state = 0xace5eed
    for q in range(0x3c300, 0x3ff00):
        state = (state*6364136223846793005+1442695040888963407) & MASK64
        values.update({q << 44, (q << 44) | ((1 << 44)-1), (q << 44) | (state & ((1 << 44)-1))})
    for bits in [0, 1, 0x3c30000000000000, 0x3fb0000000000000, 0x3febb00000000000,
                 0x3fefc00000000000, 0x3ff0000000000000]:
        for delta in range(-16, 17):
            if 0 <= bits+delta <= 0x3ff0000000000000:
                values.add(bits+delta)
    # Additional dense source-like binary32 abs-dot inputs and near-one
    # mantissas exercise reductions and source table-index transitions.
    for _ in range(8192):
        state = (state*6364136223846793005+1442695040888963407) & MASK64
        value = (state >> 32) / 0xffffffff
        value = struct.unpack('<f', struct.pack('<f', value))[0]
        values.add(struct.unpack('<Q', struct.pack('<d', value))[0])
        values.add(0x3fefc00000000000 | (state & ((1 << 46)-1)))
    return sorted(values)


def db(label, data):
    return [label+' LABEL BYTE']+['DB '+','.join(f'0{v:02X}h' for v in data[i:i+16])
                                for i in range(0, len(data), 16)]


def host_source(image):
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none']
    for name, size in [('GetStdHandle',4),('ReadFile',20),('WriteFile',20),
        ('VirtualAlloc',16),('VirtualProtect',16),('GetLastError',0),
        ('FlushInstructionCache',12),('ExitProcess',4)]:
        lines.append(f'EXTERN _{name}@{size}:PROC')
    lines += ['.const']
    for start, end in CODE:
        lines += db(f'Kernel_{start:08X}', image.read(start, end-start))
    lines += db('OriginalConstants', image.read(WINDOW[0], WINDOW[1]-WINDOW[0]))
    lines += ['.data', 'InputBuffer QWORD 0', 'ExpectedValue QWORD 0',
        'ReplyBefore DWORD 4 DUP(0A5A5A5A5h)', 'ReplyBuffer BYTE 16 DUP(0)',
        'ReplyAfter DWORD 4 DUP(0A5A5A5A5h)', 'InputHandle DWORD 0',
        'OutputHandle DWORD 0', 'Transferred DWORD 0', 'OldProtect DWORD 0',
        'SseControl DWORD 01F80h', 'X87Control WORD 027Fh', 'Poison DWORD 0', '.code',
        'PUBLIC ReadRecord, ReadRecordEnd, RunOnce, RunOnceEnd, HostMain, HostEnd, HostFailure',
        'ReadRecord PROC', 'push ebx', 'push esi', 'mov ebx,8', 'mov esi,OFFSET InputBuffer',
        'ReadLoop:', 'mov Transferred,0', 'push 0', 'push OFFSET Transferred', 'push ebx',
        'push esi', 'push InputHandle', 'call _ReadFile@20', 'test eax,eax', 'jnz ReadTransferred',
        'call _GetLastError@0', 'cmp eax,109', 'jne HostFailure', 'cmp ebx,8',
        'jne HostFailure', 'cmp Transferred,0', 'jne HostFailure', 'xor eax,eax', 'jmp ReadDone',
        'ReadTransferred:', 'mov eax,Transferred', 'test eax,eax', 'jnz ReadSome',
        'cmp ebx,8', 'jne HostFailure', 'xor eax,eax', 'jmp ReadDone', 'ReadSome:',
        'add esi,eax', 'sub ebx,eax', 'jnz ReadLoop', 'mov eax,1', 'ReadDone:',
        'pop esi', 'pop ebx', 'ret', 'ReadRecord ENDP', 'ReadRecordEnd LABEL BYTE',
        'RunOnce PROC', 'fninit', 'fldcw X87Control', 'ldmxcsr SseControl',
        'cmp Poison,0', 'jne PoisonOnes']
    for i in range(8):
        lines += [f'pxor xmm{i},xmm{i}']
    lines += ['jmp ScratchReady', 'PoisonOnes:']
    for i in range(8):
        lines += [f'pcmpeqd xmm{i},xmm{i}']
    lines += ['ScratchReady:', 'fld qword ptr [InputBuffer]', 'mov eax,0785A35E0h',
        'call eax', 'fstp qword ptr [ReplyBuffer]', 'stmxcsr dword ptr [ReplyBuffer+8]',
        'fnstcw word ptr [ReplyBuffer+12]', 'fnstsw word ptr [ReplyBuffer+14]',
        'cmp word ptr [ReplyBuffer+12],027Fh', 'jne HostFailure',
        'movzx eax,word ptr [ReplyBuffer+14]', 'and eax,03800h', 'jnz HostFailure',
        'mov eax,dword ptr [ReplyBuffer+8]', 'and eax,0FFFFFFC0h', 'cmp eax,01F80h',
        'jne HostFailure', 'ret', 'RunOnce ENDP', 'RunOnceEnd LABEL BYTE', 'HostMain PROC']
    # Allocate writable only for initialization; permissions are reduced before
    # any original instruction can execute. One code page and one data page.
    for page in [0x785a0000, 0x78530000]:
        lines += ['push 04h', 'push 03000h', 'push 010000h', f'push 0{page:08X}h',
            'call _VirtualAlloc@16', f'cmp eax,0{page:08X}h', 'jne HostFailure']
    lines += ['mov edi,0785A0000h', 'mov ecx,010000h', 'mov al,0CCh', 'cld', 'rep stosb']
    for start, end in CODE:
        lines += [f'mov esi,OFFSET Kernel_{start:08X}', f'mov edi,0{start:08X}h',
            f'mov ecx,{end-start}', 'rep movsb']
    lines += ['mov esi,OFFSET OriginalConstants', f'mov edi,0{WINDOW[0]:08X}h',
        f'mov ecx,{WINDOW[1]-WINDOW[0]}', 'rep movsb']
    for page, protect in [(0x785a0000, '020h'), (0x78530000, '02h')]:
        lines += ['push OFFSET OldProtect', f'push {protect}', 'push 010000h',
            f'push 0{page:08X}h', 'call _VirtualProtect@16', 'test eax,eax', 'jz HostFailure']
    lines += ['push 0', 'push 0', 'push -1', 'call _FlushInstructionCache@12', 'test eax,eax',
        'jz HostFailure', 'push -10', 'call _GetStdHandle@4', 'mov InputHandle,eax',
        'push -11', 'call _GetStdHandle@4', 'mov OutputHandle,eax', 'NextRecord:',
        'call ReadRecord', 'test eax,eax', 'jz HostSuccess',
        # Integer guards admit exactly finite [0,1], including negative zero.
        'mov eax,dword ptr [InputBuffer+4]', 'mov edx,eax', 'and eax,07FFFFFFFh',
        'cmp eax,03FF00000h', 'ja HostFailure', 'jb BelowOne',
        'cmp dword ptr [InputBuffer],0', 'jne HostFailure', 'BelowOne:',
        'test edx,080000000h', 'jz ValidInput', 'or eax,dword ptr [InputBuffer]',
        'jnz HostFailure', 'ValidInput:', 'mov Poison,0', 'call RunOnce',
        'mov eax,dword ptr [ReplyBuffer]', 'mov dword ptr [ExpectedValue],eax',
        'mov eax,dword ptr [ReplyBuffer+4]', 'mov dword ptr [ExpectedValue+4],eax',
        'mov Poison,1', 'call RunOnce', 'mov eax,dword ptr [ReplyBuffer]',
        'cmp eax,dword ptr [ExpectedValue]', 'jne HostFailure',
        'mov eax,dword ptr [ReplyBuffer+4]', 'cmp eax,dword ptr [ExpectedValue+4]', 'jne HostFailure']
    for label in ['ReplyBefore', 'ReplyAfter']:
        for offset in range(0,16,4):
            lines += [f'cmp dword ptr [{label}+{offset}],0A5A5A5A5h', 'jne HostFailure']
    lines += ['push 0', 'push OFFSET Transferred', 'push 16', 'push OFFSET ReplyBuffer',
        'push OutputHandle', 'call _WriteFile@20', 'test eax,eax', 'jz HostFailure',
        'cmp Transferred,16', 'jne HostFailure', 'jmp NextRecord', 'HostSuccess:',
        'push 0', 'call _ExitProcess@4', 'HostFailure::', 'push 7', 'call _ExitProcess@4',
        'HostMain ENDP', 'HostEnd LABEL BYTE', 'END HostMain']
    return '\n'.join(lines)+'\n'


def audit_host(exe, map_path):
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_REG, X86_REG_EAX
    symbols = {}
    wanted = {'ReadRecord','ReadRecordEnd','RunOnce','RunOnceEnd','HostMain','HostEnd','HostFailure'}
    for line in map_path.read_text().splitlines():
        match = re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\w+)\s+([0-9A-Fa-f]{8})\b',line)
        if match and match[1] in wanted:
            symbols[match[1]] = int(match[2],16)
    assert symbols.keys() == wanted, symbols
    image = PE(exe)
    imports = image.imports()
    assert {dll.upper() for dll, _ in imports.values()} == {'KERNEL32.DLL'}
    assert {name for _, name in imports.values()} == IMPORTS
    start, end = symbols['ReadRecord'], symbols['HostEnd']
    instructions = list(decoder().disasm(image.read(start,end-start),start))
    assert sum(i.size for i in instructions) == end-start
    boundaries = {i.address for i in instructions}
    calls, branches = [], []
    for n, i in enumerate(instructions):
        assert i.mnemonic not in {'int','syscall','sysenter','hlt','iret','iretd'}
        if i.group(CS_GRP_JUMP):
            assert i.operands[0].type == X86_OP_IMM
            target = i.operands[0].imm
            assert target in boundaries
            branches.append(dict(at=f'{i.address:08X}',target=f'{target:08X}'))
        if not i.group(CS_GRP_CALL):
            continue
        operand = i.operands[0]
        if operand.type == X86_OP_IMM:
            target = operand.imm
            if target in {symbols['RunOnce'],symbols['ReadRecord']}:
                calls.append(dict(at=f'{i.address:08X}',target=f'{target:08X}',kind='owned helper'))
            else:
                thunk = image.read(target,6)
                assert thunk[:2] == b'\xff\x25'
                slot, = struct.unpack('<I',thunk[2:])
                dll, name = imports[slot]
                calls.append(dict(at=f'{i.address:08X}',kind='allowlisted host import',dll=dll,name=name))
        else:
            assert operand.type == X86_OP_REG and operand.reg == X86_REG_EAX
            previous = instructions[n-1]
            assert previous.mnemonic == 'mov' and previous.operands[0].reg == X86_REG_EAX
            assert previous.operands[1].type == X86_OP_IMM and previous.operands[1].imm == CODE[0][0]
            calls.append(dict(at=f'{i.address:08X}',target=f'{CODE[0][0]:08X}',kind='finite original wrapper'))
    assert sum(c['kind']=='finite original wrapper' for c in calls) == 1
    return dict(start=f'{start:08X}',end_exclusive=f'{end:08X}',sha256=digest(image.read(start,end-start)),
        symbols={name:f'{address:08X}' for name,address in symbols.items()},
        imported_functions=[dict(iat=f'{slot:08X}',dll=dll,name=name) for slot,(dll,name) in sorted(imports.items())],
        calls=calls, relative_jumps=branches,
        disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in instructions])


def prepare(args):
    output = args.out.resolve()
    output.relative_to((ROOT/'target').resolve())
    image = PE(args.dll)
    assert image.base == 0x78520000
    assert digest(image.data) == SOURCE_SHA256
    assert image.export('_CIacos') == 0x7859a6e0
    constants = image.read(WINDOW[0],WINDOW[1]-WINDOW[0])
    assert digest(constants) == WINDOW_SHA256
    decoder()  # Import the pinned local inspection dependency before use.
    code, bound_proof = audit_source(image)
    output.mkdir(parents=True,exist_ok=False)
    values = corpus()
    input_data = b''.join(struct.pack('<Q',bits) for bits in values)
    tsv = ''.join(f'{i:06d}\t{bits:016x}\n' for i,bits in enumerate(values)).encode('ascii')
    (output/'inputs.bin').write_bytes(input_data)
    (output/'inputs.tsv').write_bytes(tsv)
    (output/'constants.bin').write_bytes(constants)
    assembly, obj, exe = output/'host-x86.asm',output/'host-x86.obj',output/'host-x86.exe'
    map_path = output/'host-x86.map'
    assembly.write_text(host_source(image))
    commands = [
        [str(args.sdk_bin_x86/'ml.exe'),'/nologo','/c','/coff',f'/Fo{obj}',str(assembly)],
        [str(args.sdk_bin_x86/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE',
            '/ENTRY:HostMain','/BASE:0x24000000','/DYNAMICBASE:NO','/SAFESEH:NO',
            f'/OUT:{exe}',f'/MAP:{map_path}',str(obj),str(args.kernel32_lib)],
    ]
    for command in commands:
        result = subprocess.run(command,capture_output=True,text=True)
        assert result.returncode == 0,result.stdout+'\n'+result.stderr
    owned = audit_host(exe,map_path)
    manifest = dict(schema='bounded-original-crt-acos-sse2-host-v1',
        source_path=str(args.dll.resolve()),source_sha256=SOURCE_SHA256,host_executed=False,
        arithmetic_policy='finite [-0,+1], SSE2 MXCSR1F80, x87 wrapper CW027F only; not retail dispatch',
        code_allowlist=code,constant_window=dict(start=f'{WINDOW[0]:08X}',end_exclusive=f'{WINDOW[1]:08X}',
            bytes=len(constants),sha256=WINDOW_SHA256),indexed_read_bound_proof=bound_proof,
        omitted_code_policy='Native code page INT3-filled. Only three listed spans copied. No original CRT dispatcher, legacy, domain/NaN handler or original import call is present.',
        endpoint_fallthrough_proof='At785A3A32 JE785A3ABD is always taken after finite host validation on the only q>=3FF00 case: exact +1. The false fallthrough785A3A38 is INT3.',
        pages=[dict(address='785A0000',bytes=65536,initial='RW',final='RX'),
               dict(address='78530000',bytes=65536,initial='RW',final='R')],
        host_image_base='24000000',host_code_audit=owned,
        host_source_sha256=digest(assembly.read_bytes()),host_exe_sha256=digest(exe.read_bytes()),
        compile_commands=commands,original_module_loaded=False,
        protocol=dict(input_bytes=8,input='binary64 bits, integer validated finite [-0,+1]',
            output_bytes=16,output_fields=['result:u64@0','post_MXCSR:u32@8','post_CW:u16@12','post_SW:u16@14'],
            per_input_calls=2,scratch_lane_patterns=['all-zero','all-one'],
            checks=['both outputs bit-identical','x87 TOP restored to zero','CW027F unchanged',
                    'MXCSR control bits remain1F80','reply canaries preserved'],
            input_rows=len(values),branch_counts=dict(Counter(branch(bits) for bits in values)),
            input_sha256=digest(input_data),input_tsv_sha256=digest(tsv)),
        next_step='Root must review fresh manifest/code/data/branch/import audit before any separate host execution.')
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({key:manifest[key] for key in ['source_sha256','host_executed','host_source_sha256',
        'host_exe_sha256','protocol']},indent=2))


def compare(args):
    out = args.fixture.resolve()
    manifest = json.loads((out/'manifest.json').read_text())
    source = (out/'inputs.bin').read_bytes()
    assert digest(source) == manifest['protocol']['input_sha256']
    values = [row[0] for row in struct.iter_unpack('<Q',source)]
    native = args.native.read_bytes()
    assert len(native) == len(values)*16
    rows = []
    flags = Counter()
    for i,(bits,(result,mxcsr,cw,sw)) in enumerate(zip(values,struct.iter_unpack('<QIHH',native))):
        assert mxcsr & ~0x3f == 0x1f80 and cw == 0x27f and sw & 0x3800 == 0
        flags[mxcsr & 0x3f] += 1
        rows.append(f'{i:06d}\t{branch(bits)}\t{result:016x}\n')
    expected = ''.join(rows).encode('ascii')
    rust = args.rust.read_bytes()
    report = dict(schema='crt-acos-sse2-conformance-comparison-v1',rows=len(rows),
        source_sha256=SOURCE_SHA256,host_exe_sha256=manifest['host_exe_sha256'],
        input_sha256=digest(source),native_sha256=digest(native),native_expected_tsv_sha256=digest(expected),
        rust_tsv_sha256=digest(rust),exact_match=expected==rust,
        source_sse_exception_flag_counts={f'{k:02X}':v for k,v in sorted(flags.items())},
        scope='Finite explicit SSE2 MXCSR1F80 only; no retail dispatch/legacy claim.')
    args.report.resolve().relative_to((ROOT/'target').resolve())
    with args.report.open('x') as stream:
        json.dump(report,stream,indent=2)
        stream.write('\n')
    print(json.dumps(report,indent=2))
    if expected != rust:
        got = rust.splitlines()
        for i,(a,b) in enumerate(zip(expected.splitlines(),got)):
            if a != b:
                raise AssertionError(f'first mismatch row{i}: native={a!r} rust={b!r}')
        raise AssertionError('output lengths differ')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command',required=True)
    p = sub.add_parser('prepare',help='read, inspect, assemble and link only; never execute host')
    p.add_argument('dll',type=Path)
    p.add_argument('--sdk-bin-x86',type=Path,required=True)
    p.add_argument('--kernel32-lib',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True)
    c = sub.add_parser('compare',help='compare already-produced outputs; never execute host')
    c.add_argument('fixture',type=Path)
    c.add_argument('--native',type=Path,required=True)
    c.add_argument('--rust',type=Path,required=True)
    c.add_argument('--report',type=Path,required=True)
    args = parser.parse_args()
    (prepare if args.command=='prepare' else compare)(args)


if __name__ == '__main__':
    main()

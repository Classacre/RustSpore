"""Add public audit symbols to frozen v2 constructor preparation; never build/run.

Source and owned instructions are unchanged. This version adds only MASM PUBLIC
directives and zero-byte code-boundary labels, plus a separate linked audit.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct

import anim_root_binding_native_probe as v2
from cell_schema_probe import SourceImage, SOURCE_SHA256
from native_crt_acos_probe import PE, decoder

ROOT = Path(__file__).resolve().parent.parent
V2_TOOL_SHA256 = '2d6d242a91563003575fb25f67e0b5e6a68ff502f6b3468c180ae826c45dd5b6'
V2_ASM_SHA256 = '03b719e34b7a9e7354d450a3df140941c1a7fab1fc7694f15770659af9173872'
V2_HOST = ROOT/'target/animation/root-binding-constructor-host-v2'
V2_INPUT = ROOT/'target/animation/root-binding-constructor-inputs-v2'
IMPORTS = {'GetStdHandle', 'ReadFile', 'WriteFile', 'VirtualAlloc', 'VirtualProtect',
           'FlushInstructionCache', 'GetLastError', 'ExitProcess'}
HOST_BASE = 0x27000000


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def host_source(image):
    assert digest(v2.__file__) == V2_TOOL_SHA256
    frozen = V2_HOST/'host-x86.asm'
    assert digest(frozen) == V2_ASM_SHA256
    source = v2.host_source(image)
    assert source == frozen.read_text()
    publics = ['OwnedStart', 'OwnedEnd']
    publics += [f'Original_{a:08X}' for a, _ in v2.CODE]
    publics += [f'Data_{a:08X}' for a, _ in v2.DATA]
    for label, _ in v2.BUFFERS:
        publics += [label+'Buffer', label+'Before', label+'After']
        if label not in ('Stack', 'Reply', 'Records'):
            publics += [label+'Shadow']
    publics += ['InputHandle','OutputHandle','Transferred','OldProtect','SavedStack','Remaining','Cursor']
    lines = source.splitlines()
    code = lines.index('.code')
    additions = ['PUBLIC '+name for name in publics]
    lines[code:code] = additions
    start = lines.index('ValidateRecord PROC')
    lines.insert(start, 'OwnedStart LABEL BYTE')
    lines.insert(lines.index('END'), 'OwnedEnd LABEL BYTE')
    result = '\n'.join(lines)+'\n'
    # Mechanical proof that only declarations and zero-byte labels changed.
    stripped = [line for line in result.splitlines()
                if line not in additions and line not in ('OwnedStart LABEL BYTE','OwnedEnd LABEL BYTE')]
    assert stripped == source.splitlines()
    return result


def linked_audit(path, mapping, image):
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_REG, X86_REG_EDX
    pe = PE(path)
    symbols = {}
    for line in mapping.read_text().splitlines():
        match = re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]+\s+(\S+)\s+([0-9A-Fa-f]{8})\b', line)
        if match:
            symbols[match[1]] = int(match[2],16)
    assert pe.base == HOST_BASE
    start, end = symbols['OwnedStart'], symbols['OwnedEnd']
    rows = list(decoder().disasm(pe.read(start,end-start),start))
    assert rows and sum(i.size for i in rows) == end-start
    boundaries = {i.address for i in rows}
    imports = pe.imports()
    assert len(imports) == 8
    assert {name for dll,name in imports.values() if dll.lower() == 'kernel32.dll'} == IMPORTS
    transfers, native_calls = [], 0
    for index,row in enumerate(rows):
        if not (row.group(CS_GRP_CALL) or row.group(CS_GRP_JUMP)):
            continue
        op = row.operands[0]
        if op.type == X86_OP_REG:
            assert row.mnemonic == 'call' and op.reg == X86_REG_EDX
            assert index > 0 and rows[index-1].bytes == bytes.fromhex('ba300e9a00')
            native_calls += 1
            transfers.append(dict(at=f'{row.address:08X}', operation='call', target='009A0E30',
                                  proof='Immediately preceding MOV EDX,009A0E30; no intervening instruction.'))
        else:
            assert op.type == X86_OP_IMM
            target = op.imm
            if target not in boundaries:
                thunk = pe.read(target,6)
                assert row.mnemonic == 'call' and thunk[:2] == bytes.fromhex('ff25')
                assert struct.unpack_from('<I',thunk,2)[0] in imports
            transfers.append(dict(at=f'{row.address:08X}', operation=row.mnemonic, target=f'{target:08X}'))
    assert native_calls == 1
    at = struct.unpack_from('<I',pe.data,0x3c)[0]
    optional = at+24
    assert pe.base+struct.unpack_from('<I',pe.data,optional+16)[0] == symbols['HostMain']
    assert struct.unpack_from('<H',pe.data,optional+70)[0] & 0x40 == 0
    count = struct.unpack_from('<H',pe.data,at+6)[0]
    optional_size = struct.unpack_from('<H',pe.data,at+20)[0]
    sections = []
    for i in range(count):
        header = optional+optional_size+i*40
        size,rva = struct.unpack_from('<II',pe.data,header+8)
        sections.append((pe.base+rva,pe.base+rva+size,struct.unpack_from('<I',pe.data,header+36)[0]))
    def permissions(address,size):
        return next(flags for low,high,flags in sections if low<=address and address+size<=high)&0xe0000000
    assert permissions(start,end-start) == 0x60000000
    blob_records = []
    for a,b in v2.CODE:
        address = symbols[f'Original_{a:08X}']
        assert pe.read(address,b-a) == image.read(a,b-a)
        assert permissions(address,b-a) == 0x40000000
        blob_records.append(dict(kind='source_code',address=f'{a:08X}',linked_address=f'{address:08X}',
                                 bytes=b-a,sha256=hashlib.sha256(pe.read(address,b-a)).hexdigest()))
    for a,n in v2.DATA:
        address = symbols[f'Data_{a:08X}']
        assert pe.read(address,n) == image.read(a,n)
        assert permissions(address,n) == 0x40000000
        blob_records.append(dict(kind='source_data',address=f'{a:08X}',linked_address=f'{address:08X}',
                                 bytes=n,sha256=hashlib.sha256(pe.read(address,n)).hexdigest()))
    extents = []
    def add_extent(name,address,size):
        assert permissions(address,size) == 0xc0000000
        assert all(address+size<=lo or hi<=address for _,lo,hi in extents)
        extents.append((name,address,address+size))
    for label,n in v2.BUFFERS:
        address = symbols[label+'Buffer']
        assert address-symbols[label+'Before'] == 16
        assert symbols[label+'After']-address == n
        assert pe.read(address-16,16) == bytes.fromhex('a5'*16)
        assert pe.read(address+n,16) == bytes.fromhex('a5'*16)
        add_extent(label+'GuardedBuffer',address-16,n+32)
        if label not in ('Stack','Reply','Records'):
            add_extent(label+'Shadow',symbols[label+'Shadow'],n)
    for name in ['InputHandle','OutputHandle','Transferred','OldProtect','SavedStack','Remaining','Cursor']:
        add_extent(name,symbols[name],4)
    return dict(schema='ordinary-root-constructor-v3-linked-audit-v1',host_exe_sha256=digest(path),
        host_map_sha256=digest(mapping),source_sha256=SOURCE_SHA256,
        owned_code_sha256=hashlib.sha256(pe.read(start,end-start)).hexdigest(),
        source_and_data_exact=True,section_permissions_checked=True,buffers_and_shadows_nonoverlapping=True,
        native_calls=native_calls,host_executed=False,
        imports=[dict(iat=f'{address:08X}',dll=dll,name=name) for address,(dll,name) in imports.items()],
        symbols={name:f'{address:08X}' for name,address in symbols.items() if not name.startswith('_')},
        blobs=blob_records,transfers=transfers,
        extents=[dict(name=name,start=f'{lo:08X}',end_exclusive=f'{hi:08X}') for name,lo,hi in extents],
        disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in rows],
        abi_scope='Custom owned gateway checks exact native ESP and immutable memory. No partial GPR preservation or retail-context claim.')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    p.add_argument('--out',type=Path,required=True)
    p.add_argument('--audit-exe',type=Path)
    p.add_argument('--audit-map',type=Path)
    a = p.parse_args()
    a.out.resolve().relative_to((ROOT/'target').resolve())
    image = SourceImage(a.source_exe)
    text = host_source(image)
    source_audit = v2.audit(image)
    if a.audit_exe is not None:
        assert a.audit_map is not None
        manifest = json.loads((a.out/'manifest.json').read_text())
        assert (a.out/'host-x86.asm').read_text() == text
        assert digest(a.out/'host-x86.asm') == manifest['host_source_sha256']
        assert digest(__file__) == manifest['preparation_tool_sha256']
        assert source_audit == manifest['source_audit']
        report = linked_audit(a.audit_exe,a.audit_map,image)
        with (a.out/'linked-audit.json').open('x') as f:
            json.dump(report,f,indent=2);f.write('\n')
        print(json.dumps({k:report[k] for k in ['schema','host_exe_sha256','owned_code_sha256','host_executed']},indent=2))
        return
    a.out.mkdir(parents=True,exist_ok=False)
    (a.out/'host-x86.asm').write_text(text)
    preserved = [Path(v2.__file__),V2_HOST/'manifest.json',V2_HOST/'host-x86.asm',
                 V2_INPUT/'manifest.json',V2_INPUT/'inputs.bin',V2_INPUT/'reference.bin',V2_INPUT/'rejected.bin']
    report = dict(schema='ordinary-root-constructor-host-preparation-v3',source_sha256=SOURCE_SHA256,
        host_executed=False,compiled=False,linked=False,source_audit=source_audit,
        host_source_sha256=digest(a.out/'host-x86.asm'),preparation_tool_sha256=digest(__file__),
        preserved_v2_pins={str(path.relative_to(ROOT)):digest(path) for path in preserved},
        v2_instructions_unchanged=True,change='PUBLIC directives and zero-byte OwnedStart/OwnedEnd labels only.',
        input_bytes=364,output_bytes=48,
        compile_commands_templates=[['ml.exe','/nologo','/c','/coff','/Fohost-x86.obj','host-x86.asm'],
            ['link.exe','/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain',
             '/BASE:0x27000000','/DYNAMICBASE:NO','/SAFESEH:NO','/OUT:host-x86.exe','/MAP:host-x86.map',
             'host-x86.obj','kernel32.lib']],
        linked_audit_command='python tools/anim_root_binding_native_probe_v3.py --out HOST_DIR --audit-exe HOST_DIR/host-x86.exe --audit-map HOST_DIR/host-x86.map',
        remaining_gates=['Root compile/link and linked audit','Root reviewed hidden runner and protocol checks',
                         'Native/Python/Rust equality; actual actor association remains unsupported'],
        abi_scope='No partial GPR preservation claim. Exact native ESP and immutable-buffer guards are checked.')
    with (a.out/'manifest.json').open('x') as f:
        json.dump(report,f,indent=2);f.write('\n')
    print(json.dumps({k:report[k] for k in ['schema','host_source_sha256','compiled','linked','host_executed']},indent=2))


if __name__ == '__main__':
    main()

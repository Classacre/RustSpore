"""Prepare a source-root coordinate conversion plus POS/ROT accumulation host.

Never compiles or executes. Source samples must come from the paired native
curve fixture; finalized rest/reference fields remain explicit caller inputs.
No staged evaluator, child constraints or final visible pose is claimed.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct

from cell_schema_probe import SourceImage, SOURCE_SHA256
from cell_native_math_probe import db
from native_crt_acos_probe import PE, decoder

ROOT = Path(__file__).resolve().parent.parent
CODE = [(0x9b0ca0,0x9b1b82),(0x9b0b90,0x9b0c91),(0x99bd80,0x99be6b),
        (0x99d74f,0x99dbfe),(0x99c8d0,0x99c993),(0x5992b0,0x5992cf)]
EXIT = 0x99dbfe
GLOBALS = {0x1551a70,0x1551a74}
IMPORTS = {'GetStdHandle','ReadFile','WriteFile','VirtualAlloc','VirtualProtect',
           'FlushInstructionCache','GetLastError','ExitProcess'}
BUFFERS = [('Input',144),('Owner',1024),('OwnerDefinition',1024),('Root',0x468),
    ('RootReference',0x468),('RootBox',4),('Pose',0x2bc),('Wrapper',0x1f0),
    ('Channel',0xe4),('PositionDescriptor',32),('RotationDescriptor',32),
    ('ReferencePosition',12),('ReferenceRotation',16),('Converted',28),
    ('SecondaryPosition',12),('Frame',0x3400),('Reply',176)]
MUTABLE = {'Owner','Pose','Frame','Converted','Reply'}
BASE = 0x29000000


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def source_audit(image):
    engine = decoder()
    from capstone import CS_GRP_CALL,CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM,X86_OP_MEM
    parts = [list(engine.disasm(image.read(a,b-a),a)) for a,b in CODE]
    boundaries = {i.address for rows in parts for i in rows}
    external,constants,transfers = set(),{},[]
    for (a,b),rows in zip(CODE,parts):
        assert sum(i.size for i in rows)==b-a
        for row in rows:
            if row.group(CS_GRP_CALL) or row.group(CS_GRP_JUMP):
                op = row.operands[0]
                assert op.type == X86_OP_IMM
                if op.imm not in boundaries and op.imm != EXIT:
                    assert row.mnemonic == 'call'
                    external.add(op.imm)
                transfers.append(dict(at=f'{row.address:08X}',operation=row.mnemonic,target=f'{op.imm:08X}'))
            for op in row.operands:
                if op.type == X86_OP_MEM and not op.mem.base:
                    assert not op.mem.index
                    constants[op.mem.disp] = max(constants.get(op.mem.disp,0),op.size)
    # These calls are excluded by exact source flag/owner/global contracts.
    assert external == {0x99be70,0x9a48b0,0x9a4df0,0x9b0150,0x9b0530,
                        0x9b05e0,0x9b0ac0,0x9b75b0,0x9cda70}
    assert GLOBALS <= constants.keys()
    records = [dict(start=f'{a:08X}',end_exclusive=f'{b:08X}',bytes=b-a,
                    sha256=hashlib.sha256(image.read(a,b-a)).hexdigest()) for a,b in CODE]
    data = [dict(address=f'{a:08X}',bytes=n,sha256=hashlib.sha256(image.read(a,n)).hexdigest())
            for a,n in sorted(constants.items()) if a not in GLOBALS]
    return dict(code=records,data=data,transfers=transfers,
        unreachable_calls=[f'{a:08X}' for a in sorted(external)],
        globals={f'{a:08X}':0 for a in sorted(GLOBALS)},
        disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for rows in parts for i in rows])


def host_source(image,audit):
    lines = ['.686p','.xmm','.model flat','option casemap:none']
    for name,n in [('GetStdHandle',4),('ReadFile',20),('WriteFile',20),('VirtualAlloc',16),
                   ('VirtualProtect',16),('FlushInstructionCache',12),('GetLastError',0),('ExitProcess',4)]:
        lines += [f'EXTERN _{name}@{n}:PROC']
    lines += ['.const']
    for a,b in CODE:
        lines += [f'PUBLIC Original_{a:08X}']+db(f'Original_{a:08X}',image.read(a,b-a))
    for row in audit['data']:
        a,n = int(row['address'],16),row['bytes']
        lines += [f'PUBLIC Data_{a:08X}']+db(f'Data_{a:08X}',image.read(a,n))
    lines += ['.data']
    for name,n in BUFFERS:
        lines += [f'PUBLIC {name}Before,{name}Buffer,{name}After',f'{name}Before DWORD 4 DUP(0A5A5A5A5h)',
                  f'{name}Buffer BYTE {n} DUP(0)',f'{name}After DWORD 4 DUP(0A5A5A5A5h)']
        if name not in ('Converted','Reply'):
            lines += [f'PUBLIC {name}Shadow',f'{name}Shadow BYTE {n} DUP(0)']
    lines += ['InputHandle DWORD 0','OutputHandle DWORD 0','Transferred DWORD 0','OldProtect DWORD 0',
              'SavedStack DWORD 0','Remaining DWORD 0','Cursor DWORD 0','Poison DWORD 0',
              'Expected BYTE 156 DUP(0)','SseControl DWORD 01F80h','.code',
              'PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,MappingFailure,ValidateRecord,RunOnce,OriginalDone',
              'OwnedStart LABEL BYTE','ValidateRecord PROC',
              'mov eax,dword ptr [InputBuffer]','mov edx,eax','and eax,0FFFFF0FFh','cmp eax,07Fh','jne HostFailure',
              'and edx,0300h','cmp edx,0100h','je HostFailure',
              'mov eax,dword ptr [InputBuffer+4]','cmp eax,031h','je PositionFlagsOK',
              'cmp eax,071h','je PositionFlagsOK','cmp eax,0B1h','jne HostFailure','PositionFlagsOK:',
              'mov eax,dword ptr [InputBuffer+8]','and eax,0FFFFFFEFh','jnz HostFailure',
              'cmp dword ptr [InputBuffer+12],1','ja HostFailure',
              'mov eax,dword ptr [InputBuffer+60]','test eax,0FF000000h','jnz HostFailure',
              'and eax,6','cmp eax,6','jne HostFailure',
              'mov eax,dword ptr [InputBuffer+40]','and eax,0FFFFFFF7h','jnz HostFailure',
              'mov ecx,4','ValidateFloatsA:','mov eax,dword ptr [InputBuffer+ecx*4]',
              'and eax,07F800000h','cmp eax,07F800000h','je HostFailure','inc ecx','cmp ecx,10','jb ValidateFloatsA',
              'mov ecx,11','ValidateFloatsB:','cmp ecx,15','je NextFloatB',
              'mov eax,dword ptr [InputBuffer+ecx*4]','and eax,07F800000h','cmp eax,07F800000h','je HostFailure',
              'NextFloatB:','inc ecx','cmp ecx,36','jb ValidateFloatsB','ret','ValidateRecord ENDP',
              'RunOnce PROC','mov SavedStack,esp','cld']
    for name,n in BUFFERS:
        if name != 'Input':
            lines += [f'mov edi,OFFSET {name}Buffer',f'mov ecx,{n//4}','xor eax,eax','rep stosd']
    lines += ['mov dword ptr [OwnerBuffer],OFFSET OwnerDefinitionBuffer',
              'mov dword ptr [OwnerBuffer+8],OFFSET RootBoxBuffer',
              'mov dword ptr [RootBoxBuffer],OFFSET RootReferenceBuffer',
              'mov dword ptr [PoseBuffer],OFFSET RootBuffer',
              'mov dword ptr [WrapperBuffer],OFFSET ChannelBuffer',
              'mov eax,dword ptr [InputBuffer+8]','mov dword ptr [ChannelBuffer+088h],eax',
              'mov eax,dword ptr [InputBuffer+4]','mov dword ptr [PositionDescriptorBuffer],eax',
              'mov dword ptr [RotationDescriptorBuffer],032h',
              'mov eax,dword ptr [InputBuffer+20]','mov dword ptr [OwnerBuffer+070h],eax',
              'mov eax,dword ptr [InputBuffer+36]','mov dword ptr [OwnerDefinitionBuffer+0374h],eax',
              'mov eax,dword ptr [InputBuffer+40]','mov dword ptr [RootBuffer+0154h],eax',
              'mov eax,dword ptr [InputBuffer+44]','mov dword ptr [RootBuffer+0434h],eax']
    for src,label,at,n in [(24,'OwnerDefinition',0x364,3),(48,'Root',0x10c,3),(64,'Root',0x118,4),
                           (92,'RootReference',0x118,4),(80,'ReferencePosition',0,3),(92,'ReferenceRotation',0,4),
                           (108,'Wrapper',0x18,3),(124,'Wrapper',0x68,4)]:
        lines += [f'mov esi,OFFSET InputBuffer+{src}',f'mov edi,OFFSET {label}Buffer+0{at:X}h',f'mov ecx,{n}','rep movsd']
    lines += ['mov eax,dword ptr [InputBuffer+120]','mov dword ptr [WrapperBuffer+024h],eax',
              'mov eax,dword ptr [InputBuffer+140]','mov dword ptr [WrapperBuffer+078h],eax']
    for name,n in BUFFERS:
        if name not in ('Converted','Reply'):
            lines += [f'mov esi,OFFSET {name}Buffer',f'mov edi,OFFSET {name}Shadow',f'mov ecx,{n//4}','rep movsd']
    lines += ['fninit','fldcw word ptr [InputBuffer]','ldmxcsr SseControl','movd xmm0,Poison','pshufd xmm0,xmm0,0']
    for i in range(1,8):
        lines += [f'movaps xmm{i},xmm0']
    lines += ['mov esp,OFFSET FrameBuffer+0800h',
              'push dword ptr [InputBuffer+12]','push OFFSET ConvertedBuffer+12','push OFFSET ConvertedBuffer',
              'push OFFSET ReferenceRotationBuffer','push OFFSET ReferencePositionBuffer','push OFFSET PoseBuffer',
              'push OFFSET OwnerBuffer','push OFFSET WrapperBuffer+068h','push OFFSET RotationDescriptorBuffer',
              'push OFFSET WrapperBuffer+018h','push OFFSET PositionDescriptorBuffer','push OFFSET SecondaryPositionBuffer',
              'push OFFSET ChannelBuffer+088h','mov eax,009B0CA0h','call eax','add esp,52',
              'cmp esp,OFFSET FrameBuffer+0800h','jne HostFailure',
              'mov esi,OFFSET ConvertedBuffer','mov edi,OFFSET FrameBuffer+0858h','mov ecx,7','rep movsd',
              'mov eax,dword ptr [InputBuffer+16]','mov dword ptr [FrameBuffer+0840h],eax',
              'mov dword ptr [FrameBuffer+0850h],OFFSET PositionDescriptorBuffer',
              'mov dword ptr [FrameBuffer+0834h],OFFSET RotationDescriptorBuffer',
              'mov dword ptr [FrameBuffer+0838h],4','mov eax,dword ptr [InputBuffer+60]',
              'mov dword ptr [FrameBuffer+0828h],eax',
              'mov byte ptr [FrameBuffer+0813h],1','mov byte ptr [FrameBuffer+0812h],2',
              'mov dword ptr [FrameBuffer+0818h],OFFSET ChannelBuffer+088h',
              'mov dword ptr [FrameBuffer+02E0Ch],OFFSET OwnerBuffer',
              'mov esi,OFFSET PoseBuffer','mov edi,OFFSET WrapperBuffer','mov ebp,OFFSET OwnerBuffer',
              'mov ebx,2','mov eax,0099D74Fh','jmp eax','OriginalDone::',
              'cmp esp,OFFSET FrameBuffer+0800h','jne HostFailure','mov esp,SavedStack','cld']
    for name,n in BUFFERS:
        if name in ('Converted','Reply'):
            continue
        if name in ('Owner','Pose','Frame'):
            lines += ['xor ecx,ecx',f'Check{name}:']
            if name == 'Owner':
                lines += ['cmp ecx,01B8h',f'je Next{name}']
            elif name == 'Pose':
                lines += ['cmp ecx,048h',f'jb Compare{name}','cmp ecx,0A0h',f'jb Next{name}']
            else:
                # One bounded owned stack/local slab; immutable high arguments
                # and all other frame words are still shadow checked.
                lines += ['cmp ecx,0500h',f'jb Compare{name}','cmp ecx,0B00h',f'jb Next{name}',
                          'cmp ecx,02E0Ch',f'je Next{name}']
            lines += [f'Compare{name}:',f'mov eax,dword ptr [{name}Buffer+ecx]',
                      f'cmp eax,dword ptr [{name}Shadow+ecx]','jne HostFailure',
                      f'Next{name}:','add ecx,4',f'cmp ecx,{n}',f'jb Check{name}']
        else:
            lines += [f'mov esi,OFFSET {name}Buffer',f'mov edi,OFFSET {name}Shadow',f'mov ecx,{n//4}','repe cmpsd','jne HostFailure']
    for name,_ in BUFFERS:
        for side in ('Before','After'):
            for at in range(0,16,4):
                lines += [f'cmp dword ptr [{name}{side}+{at}],0A5A5A5A5h','jne HostFailure']
    for label,at,dest,n in [('Converted',0,0,7),('Pose',0x48,28,22),('Frame',0xa28,116,9)]:
        lines += [f'mov esi,OFFSET {label}Buffer+0{at:X}h',f'mov edi,OFFSET ReplyBuffer+{dest}',f'mov ecx,{n}','rep movsd']
    lines += ['mov eax,dword ptr [OwnerBuffer+01B8h]','mov dword ptr [ReplyBuffer+152],eax',
              'fnstcw word ptr [ReplyBuffer+156]','fnstsw word ptr [ReplyBuffer+158]','stmxcsr dword ptr [ReplyBuffer+160]',
              'mov ax,word ptr [InputBuffer]','cmp ax,word ptr [ReplyBuffer+156]','jne HostFailure',
              'test word ptr [ReplyBuffer+158],03800h','jnz HostFailure',
              'mov eax,dword ptr [ReplyBuffer+160]','and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure',
              'ret','RunOnce ENDP','HostMain PROC','cld']
    code_pages = {a&~0xffff for a,b in CODE}|{EXIT&~0xffff}
    code_pages.update(int(a,16)&~0xffff for a in audit['unreachable_calls'])
    data_pages = {int(r['address'],16)&~0xffff for r in audit['data']}|{a&~0xffff for a in GLOBALS}
    pages = sorted(code_pages|data_pages)
    for page in pages:
        lines += ['push 04h','push 03000h','push 010000h',f'push 0{page:08X}h','call _VirtualAlloc@16',
                  f'cmp eax,0{page:08X}h','jne MappingFailure',f'mov edi,0{page:08X}h','mov ecx,010000h',
                  'mov al,0CCh' if page in code_pages else 'xor eax,eax','rep stosb']
    for a,b in CODE:
        lines += [f'mov esi,OFFSET Original_{a:08X}',f'mov edi,0{a:08X}h',f'mov ecx,{b-a}','rep movsb']
    for row in audit['data']:
        a,n = int(row['address'],16),row['bytes']
        lines += [f'mov esi,OFFSET Data_{a:08X}',f'mov edi,0{a:08X}h',f'mov ecx,{n}','rep movsb']
    lines += [f'mov byte ptr [0{EXIT:08X}h],0B8h','mov eax,OFFSET OriginalDone',
              f'mov dword ptr [0{EXIT+1:08X}h],eax',f'mov word ptr [0{EXIT+5:08X}h],0E0FFh']
    for page in pages:
        lines += ['push OFFSET OldProtect','push 020h' if page in code_pages else 'push 02h',
                  'push 010000h',f'push 0{page:08X}h','call _VirtualProtect@16','test eax,eax','jz MappingFailure']
    lines += ['push 0','push 0','push -1','call _FlushInstructionCache@12','test eax,eax','jz HostFailure',
              'push -10','call _GetStdHandle@4','mov InputHandle,eax','push -11','call _GetStdHandle@4','mov OutputHandle,eax',
              'NextRecord:','mov Remaining,144','mov Cursor,OFFSET InputBuffer','ReadMore:','mov Transferred,0',
              'push 0','push OFFSET Transferred','push Remaining','push Cursor','push InputHandle','call _ReadFile@20',
              'test eax,eax','jnz ReadOK','call _GetLastError@0','cmp eax,109','jne HostFailure','ReadOK:',
              'mov eax,Transferred','test eax,eax','jnz GotBytes','cmp Remaining,144','jne HostFailure',
              'push 0','call _ExitProcess@4','GotBytes:','cmp eax,Remaining','ja HostFailure',
              'add Cursor,eax','sub Remaining,eax','jnz ReadMore','call ValidateRecord',
              'mov Poison,0','call RunOnce','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected','mov ecx,156','rep movsb',
              'mov Poison,0FFFFFFFFh','call RunOnce','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected',
              'mov ecx,156','repe cmpsb','jne HostFailure','mov Remaining,176','mov Cursor,OFFSET ReplyBuffer','WriteMore:',
              'mov Transferred,0','push 0','push OFFSET Transferred','push Remaining','push Cursor','push OutputHandle',
              'call _WriteFile@20','test eax,eax','jz HostFailure','mov eax,Transferred','test eax,eax','jz HostFailure',
              'cmp eax,Remaining','ja HostFailure','add Cursor,eax','sub Remaining,eax','jnz WriteMore','jmp NextRecord',
              'HostMain ENDP','HostFailure PROC','push 71','call _ExitProcess@4','HostFailure ENDP',
              'MappingFailure PROC','push 10','call _ExitProcess@4','MappingFailure ENDP','OwnedEnd LABEL BYTE','END']
    return '\n'.join(lines)+'\n'


def linked_audit(path,mapping,image,audit):
    from capstone import CS_GRP_CALL,CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM,X86_OP_REG,X86_REG_EAX
    pe = PE(path);symbols = {}
    for line in mapping.read_text().splitlines():
        match = re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]+\s+(\S+)\s+([0-9A-Fa-f]{8})\b',line)
        if match:symbols[match[1]] = int(match[2],16)
    assert pe.base == BASE
    first,last = symbols['OwnedStart'],symbols['OwnedEnd']
    rows = list(decoder().disasm(pe.read(first,last-first),first))
    assert sum(row.size for row in rows)==last-first
    boundaries = {row.address for row in rows};imports = pe.imports()
    assert len(imports)==8
    assert {name for dll,name in imports.values() if dll.lower()=='kernel32.dll'}==IMPORTS
    def imported(row):
        if row.mnemonic!='call' or row.operands[0].type!=X86_OP_IMM:return None
        target = row.operands[0].imm
        if target in boundaries:return None
        thunk = pe.read(target,6);assert thunk[:2]==bytes.fromhex('ff25')
        return imports[struct.unpack_from('<I',thunk,2)[0]][1]
    entries = []
    for index,row in enumerate(rows):
        if not(row.group(CS_GRP_CALL) or row.group(CS_GRP_JUMP)):continue
        op = row.operands[0]
        if op.type==X86_OP_REG:
            assert op.reg==X86_REG_EAX and index>0
            previous = rows[index-1]
            assert previous.mnemonic=='mov' and previous.operands[1].type==X86_OP_IMM
            target = previous.operands[1].imm
            assert (row.mnemonic,target) in (('call',0x9b0ca0),('jmp',0x99d74f))
            entries.append(dict(at=f'{row.address:08X}',operation=row.mnemonic,target=f'{target:08X}'))
        else:
            assert op.type==X86_OP_IMM
            assert op.imm in boundaries or imported(row) in IMPORTS
    assert len(entries)==2
    at = struct.unpack_from('<I',pe.data,0x3c)[0];optional=at+24
    assert pe.base+struct.unpack_from('<I',pe.data,optional+16)[0]==symbols['HostMain']
    assert not struct.unpack_from('<H',pe.data,optional+70)[0]&0x40
    count = struct.unpack_from('<H',pe.data,at+6)[0];size = struct.unpack_from('<H',pe.data,at+20)[0]
    sections=[]
    for i in range(count):
        header=optional+size+40*i;extent,rva=struct.unpack_from('<II',pe.data,header+8)
        sections.append((pe.base+rva,pe.base+rva+extent,struct.unpack_from('<I',pe.data,header+36)[0]))
    def permission(address,n):
        return next(flags for lo,hi,flags in sections if lo<=address and address+n<=hi)&0xe0000000
    assert permission(first,last-first)==0x60000000
    for a,b in CODE:
        linked = symbols[f'Original_{a:08X}']
        assert pe.read(linked,b-a)==image.read(a,b-a) and permission(linked,b-a)==0x40000000
    for row in audit['data']:
        a,n = int(row['address'],16),row['bytes'];linked=symbols[f'Data_{a:08X}']
        assert pe.read(linked,n)==image.read(a,n) and permission(linked,n)==0x40000000
    extents=[]
    def add_extent(name,low,n):
        assert permission(low,n)==0xc0000000
        assert all(low+n<=lo or hi<=low for _,lo,hi in extents)
        extents.append((name,low,low+n))
    for name,n in BUFFERS:
        address=symbols[name+'Buffer']
        assert address-symbols[name+'Before']==16 and symbols[name+'After']-address==n
        assert pe.read(address-16,16)==bytes.fromhex('a5'*16)
        assert pe.read(address+n,16)==bytes.fromhex('a5'*16)
        add_extent(name,address-16,n+32)
        if name not in ('Converted','Reply'):add_extent(name+'Shadow',symbols[name+'Shadow'],n)
    first_read = next(row.address for row in rows if imported(row)=='ReadFile')
    incoming=[]
    for index,row in enumerate(rows):
        if not(row.group(CS_GRP_CALL) or row.group(CS_GRP_JUMP)):continue
        if row.operands[0].type!=X86_OP_IMM or row.operands[0].imm!=symbols['MappingFailure']:continue
        assert symbols['HostMain']<row.address<first_read and index>=2
        name=imported(rows[index-2]);assert name in ('VirtualAlloc','VirtualProtect')
        assert (name=='VirtualAlloc' and rows[index-1].mnemonic=='cmp' and row.mnemonic in ('jne','jnz')) or \
               (name=='VirtualProtect' and rows[index-1].bytes==bytes.fromhex('85c0') and row.mnemonic in ('je','jz'))
        incoming.append(dict(at=f'{row.address:08X}',api=name))
    assert incoming and sum(x['api']=='VirtualAlloc' for x in incoming)==sum(x['api']=='VirtualProtect' for x in incoming)
    for label,code in [('MappingFailure',10),('HostFailure',71)]:
        index=next(i for i,row in enumerate(rows) if row.address==symbols[label])
        assert rows[index].mnemonic=='push' and rows[index].operands[0].imm==code
        assert imported(rows[index+1])=='ExitProcess'
    return dict(schema='root-POS-ROT-conversion-accumulation-linked-audit-v1',native_executed=False,
        host_exe_sha256=sha(path),host_map_sha256=sha(mapping),
        owned_code_sha256=hashlib.sha256(pe.read(first,last-first)).hexdigest(),
        source_and_data_byte_exact=True,sections_checked=True,guarded_buffers_shadows_nonoverlapping=True,
        partial_native_entries=entries,mapping_failure_pre_input=incoming,
        imports=[dict(iat=f'{a:08X}',dll=dll,name=name) for a,(dll,name) in imports.items()],
        symbols={k:f'{v:08X}' for k,v in symbols.items() if not k.startswith('_')},
        disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in rows],
        limits='Partial source entry/exit with checkedESP; no partialGPRABI promise or staged/livepose claim.')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    p.add_argument('--out',type=Path,required=True)
    p.add_argument('--audit-exe',type=Path)
    p.add_argument('--audit-map',type=Path)
    a = p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
    image = SourceImage(a.source_exe);audit = source_audit(image)
    if a.audit_exe is not None:
        assert a.audit_map is not None
        manifest=json.loads((a.out/'manifest.json').read_text())
        assert sha(__file__)==manifest['preparation_tool_sha256']
        assert (a.out/'host-x86.asm').read_text()==host_source(image,audit)
        assert sha(a.out/'host-x86.asm')==manifest['host_source_sha256']
        assert audit==manifest['source_audit']
        report=linked_audit(a.audit_exe,a.audit_map,image,audit)
        with (a.out/'linked-audit.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
        print(json.dumps({k:report[k] for k in ('schema','host_exe_sha256','native_executed')},indent=2));return
    a.out.mkdir(parents=True,exist_ok=False)
    (a.out/'host-x86.asm').write_text(host_source(image,audit))
    manifest = dict(schema='root-POS-ROT-conversion-accumulation-host-preparation-v1',source_sha256=SOURCE_SHA256,
        compiled=False,linked=False,native_executed=False,source_audit=audit,
        host_source_sha256=sha(a.out/'host-x86.asm'),preparation_tool_sha256=sha(__file__),
        input_bytes=144,output_bytes=176,
        input='CW0:u32,POSflags4(31/71/B1),CHANflags8(0/10),mirror12(0/1),blend16,scale20,definitionXYZ24..32,size374@36,rootmask154@40(0/8),rootsize434@44,restXYZ48..56,wholelow24bindingmask60(requiresPOSbit1/ROTbit2),restQuat64..76,referenceXYZ80..88,referenceQuat92..104,sampledPOS108..116,POSweight120,sampledROT124..136,ROTweight140.',
        output='convertedPOS/ROT28@0,pose48..9F88@28,rootcache36@116,ownerbitmap152,CW156/SW158/MXCSR160,reserved12@164.',
        domain='Exact217firstrootflagdomain,rootordinal0,secondarynone,positive/zero/negative finite explicitblend andweights,finite sample/rest/reference fields. Wrapperflags10=0;owner260/261=0;debugglobals1551A70/74=0 explicitfixturepolicy. No protocolpointers.',
        branch_proof='CHANbit0clear excludesrelative9B0530/9B05E0/9B0150/9B0AC0paths;POSflags31/71/B1exclude200/100/orientation-specialcases;ROT32takesrawmirrorcopy+99BD80composition. SourcePOS/ROTaccumulation begins99D74Fwithrootindex0,validPOS/ROTdescriptors. Sourceexit99DBFE precedesscalardataafterbothaccumulators; allotherunreachedcallsremainINT3.',
        memory='Guardedbuffers;immutableinputs/rest/reference/wrapper/channel/descriptors. Posewrites48..9F,ownerbitmapword1B8only. OwnedFramescratch500..AFFand2E0Cargumentdeclared;allotherFramewordsshadowchecked. NativeESPFrame+800checkedonbothcalls/exit. TwoXMMpoisonscompare156projectionbytes;CW/TOP/MXCSRcontrolschecked.',
        pending=['Root source/branch/memory review','Root compile/link and linked audit of generated binary/imports/sections/control and source/data blobs',
                 'Root joins paired accepted POS/new ROT replies into144-byte requests with explicit source-bound rest/reference context',
                 'Root compiles/runs only after admission; independent numerical reference/Rust pending'],
        compile_commands_templates=[['ml.exe','/nologo','/c','/coff','/Fohost-x86.obj','host-x86.asm'],
            ['link.exe','/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/BASE:0x29000000',
             '/DYNAMICBASE:NO','/SAFESEH:NO','/OUT:host-x86.exe','/MAP:host-x86.map','host-x86.obj','kernel32.lib']],
        limits='Coordinateconversion+weightedaccumulators only. No9F4730staged-evaluation,childconstraints,9BD370finalcommit,liveactor/clock orvisibleanimationclaim.')
    (a.out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({k:manifest[k] for k in ('schema','compiled','linked','native_executed','host_source_sha256')},indent=2))


if __name__=='__main__':main()

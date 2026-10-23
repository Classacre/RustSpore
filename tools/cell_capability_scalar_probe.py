"""Prepare, never execute, the original scalar-cache plus property-curve host.

Every native byte and generated binary stays ignored under target/. Counts,
weights, property points, the mutable curve ID and FP policy are explicit. The
owned Globals/property references do not initialize the game or an archive.
"""
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
import cell_capability_curve_probe as curve
from native_crt_acos_probe import PE, decoder

CODE = [(0xe524a4, 0xe52726), *curve.CODE]
CONSTANTS = [(0x1485e24, 4), (curve.EPSILON, 4)]
CALLS = {0xe52503: 0, 0xe52562: 1, 0xe525a2: 2, 0xe525dc: 3,
         0xe52670: 4, 0xe52692: 5, 0xe526bd: 6, 0xe526dd: 7, 0xe52708: 8}
IDS = [0x77540c34, 0x2c574746, 0xbbab1bf5, 0x2ef68471,
       0xe5015d0c, None, 0x03ab589e, 0xaee2835e, 0x92baae19]
IMPORTS = {'GetStdHandle', 'ReadFile', 'WriteFile', 'VirtualAlloc', 'VirtualProtect',
           'GetLastError', 'FlushInstructionCache', 'ExitProcess'}
INPUT_BYTES, OUTPUT_BYTES = 1488, 240


def audit_source(image):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    records = curve.audit(image)
    start, end = CODE[0]
    code = list(decoder().disasm(image.read(start, end-start), start))
    assert sum(ins.size for ins in code) == end-start
    boundaries = {ins.address for ins in code}
    direct = []
    known_calls = {0x7b2350, 0xe4cca0, 0xe82160, 0xe837f0}
    for ins in code:
        if ins.group(CS_GRP_CALL) or ins.group(CS_GRP_JUMP):
            assert ins.operands[0].type == X86_OP_IMM
            target = ins.operands[0].imm
            if ins.group(CS_GRP_CALL):
                assert target in known_calls
                if target == 0xe837f0: assert ins.address in CALLS
                direct.append(dict(at=f'{ins.address:08X}', target=f'{target:08X}'))
            else: assert target in boundaries
        for operand in ins.operands:
            if operand.type != X86_OP_MEM or operand.mem.base: continue
            assert not operand.mem.index and operand.mem.disp in {0x1485e24, 0x16b4c64}
    assert image.read(0x1485e24, 4) == bytes(4)
    return [dict(start=f'{start:08X}', end_exclusive=f'{end:08X}', bytes=end-start,
        sha256=hashlib.sha256(image.read(start,end-start)).hexdigest(), direct_calls=direct,
        indirect_calls=0, disassembly=[f'{ins.address:08X}: {ins.mnemonic} {ins.op_str}' for ins in code]), *records]


def host_source(image):
    lines = ['.686p', '.xmm', '.model flat', 'option casemap:none']
    for name,size in [('GetStdHandle',4),('ReadFile',20),('WriteFile',20),('VirtualAlloc',16),
                      ('VirtualProtect',16),('GetLastError',0),('FlushInstructionCache',12),('ExitProcess',4)]:
        lines += [f'EXTERN _{name}@{size}:PROC']
    lines += ['.const', 'PUBLIC ManagerVtable,ListVtable,FakeManager,FakeList',
        'ManagerVtable DWORD 11 DUP(0),OFFSET ManagerLookup',
        'ListVtable DWORD 0,OFFSET ListRelease,8 DUP(0),OFFSET GetProperty',
        'FakeManager DWORD OFFSET ManagerVtable', 'FakeList DWORD OFFSET ListVtable']
    for start,end in CODE: lines += db(f'Original_{start:08X}',image.read(start,end-start))
    for start,count in CONSTANTS: lines += db(f'Constant_{start:08X}',image.read(start,count))
    lines += ['.data']
    for label,size in [('Input',INPUT_BYTES),('Cache',0x1290),('Globals',0x94),('Property',20),('Reference',16),('Reply',OUTPUT_BYTES)]:
        lines += [f'{label}Before DWORD 4 DUP(0A5A5A5A5h)',f'{label}Buffer BYTE {size} DUP(0)',f'{label}After DWORD 4 DUP(0A5A5A5A5h)']
    lines += ['InputShadow BYTE 1488 DUP(0)', 'GlobalsShadow BYTE 094h DUP(0)',
        'InputHandle DWORD 0','OutputHandle DWORD 0','Transferred DWORD 0','OldProtect DWORD 0',
        'SseControl DWORD 01F80h','NativeStack DWORD 0','ScalarStack DWORD 0',
        'CurrentSlot DWORD 0','CurrentID DWORD 0','ExpectedCurveRef DWORD 0',
        'CurveState DWORD 0','CurveCalls DWORD 0','CurveLookups DWORD 0',
        'CurveProperties DWORD 0','CurveReleases DWORD 0','GlobalsConstructors DWORD 0',
        'GlobalsLookups DWORD 0','GlobalsReleases DWORD 0','ExpectedCurveCalls DWORD 0',
        'ExpectedGlobalsCalls DWORD 0','PinnedID DWORD 0','IDPinned DWORD 0',
        '.code','PUBLIC OwnedStart,OwnedEnd,ScalarWrapper,ScalarWrapperEnd,GlobalsGetterStub,GlobalsReleaseStub',
        'PUBLIC CurveEnter,ManagerLookup,GetProperty,ListRelease,ReferenceConstruct,GlobalsLookup,GlobalsRelease',
        'PUBLIC HostMain,HostFailure,ManagerProvider,ReferenceProvider', 'OwnedStart LABEL BYTE',
        'GlobalsGetterStub PROC','mov eax,OFFSET GlobalsLookup','jmp eax','GlobalsGetterStub ENDP',
        'GlobalsReleaseStub PROC','mov eax,OFFSET GlobalsRelease','jmp eax','GlobalsReleaseStub ENDP',
        'ScalarWrapper PROC','sub esp,030h','push ebx','push ebp','push edi',
        'mov ScalarStack,esp','mov esi,OFFSET CacheBuffer','mov ebx,1',
        'mov eax,00E524A4h','jmp eax','ScalarWrapper ENDP','ScalarWrapperEnd LABEL BYTE',
        'ReferenceConstruct PROC','mov eax,dword ptr [esp]','cmp eax,00E5251Ah','je FirstConstruct',
        'cmp eax,00E5263Bh','jne HostFailure','mov eax,ScalarStack','add eax,044h','jmp ConstructPointer',
        'FirstConstruct:','mov eax,ScalarStack','add eax,0Ch','ConstructPointer:',
        'cmp ecx,eax','jne HostFailure','inc GlobalsConstructors','cmp GlobalsConstructors,2','ja HostFailure',
        'mov dword ptr [eax],0','ret','ReferenceConstruct ENDP',
        'GlobalsLookup PROC','mov eax,dword ptr [esp]','cmp eax,00E52524h','je FirstGlobals',
        'cmp eax,00E52645h','jne HostFailure','mov eax,ScalarStack','add eax,044h','jmp GlobalsPointer',
        'FirstGlobals:','mov eax,ScalarStack','add eax,0Ch','GlobalsPointer:',
        'cmp dword ptr [esp+4],eax','jne HostFailure','cmp dword ptr [eax],0','jne HostFailure',
        'inc GlobalsLookups','cmp GlobalsLookups,2','ja HostFailure',
        'mov dword ptr [eax],OFFSET ReferenceBuffer','inc dword ptr [ReferenceBuffer+12]',
        'mov eax,OFFSET GlobalsBuffer','ret','GlobalsLookup ENDP',
        'GlobalsRelease PROC','mov eax,dword ptr [esp]','cmp eax,00E5271Fh','je FirstRelease',
        'cmp eax,00E5265Dh','jne HostFailure','mov eax,ScalarStack','add eax,044h','jmp ReleasePointer',
        'FirstRelease:','mov eax,ScalarStack','add eax,0Ch','ReleasePointer:',
        'cmp ecx,eax','jne HostFailure','cmp dword ptr [eax],OFFSET ReferenceBuffer','jne HostFailure',
        'cmp dword ptr [ReferenceBuffer+12],0','jle HostFailure','dec dword ptr [ReferenceBuffer+12]',
        'inc GlobalsReleases','cmp GlobalsReleases,2','ja HostFailure','ret','GlobalsRelease ENDP',
        'CurveEnter PROC','cmp dword ptr [esp],00E83801h','jne HostFailure',
        'cmp CurveState,0','jne HostFailure','cmp CurveCalls,7','jae HostFailure',
        'mov eax,dword ptr [esp+16]']
    for call,slot in CALLS.items(): lines += [f'cmp eax,0{call+5:08X}h',f'je CurveSlot{slot}']
    lines += ['jmp HostFailure']
    for slot,identifier in enumerate(IDS):
        lines += [f'CurveSlot{slot}:',f'mov CurrentSlot,{slot}']
        lines += ['mov edx,dword ptr [InputBuffer+4]'] if identifier is None else [f'mov edx,0{identifier:08X}h']
        lines += ['jmp CurveSelected']
    lines += ['CurveSelected:','cmp dword ptr [esp+20],edx','jne HostFailure',
        'mov CurrentID,edx','mov eax,CurveCalls','shl eax,3',
        'mov dword ptr [ReplyBuffer+eax+152],edx','mov edx,dword ptr [esp+24]',
        'mov dword ptr [ReplyBuffer+eax+156],edx','inc CurveCalls','mov CurveState,1',
        'lea eax,[esp+8]','cmp dword ptr [eax],0','jne HostFailure','mov ExpectedCurveRef,eax',
        'mov eax,OFFSET FakeManager','ret','CurveEnter ENDP',
        'ManagerLookup PROC','cmp ecx,OFFSET FakeManager','jne HostFailure',
        'cmp CurveState,1','jne HostFailure','cmp dword ptr [esp+4],0C951EB63h','jne HostFailure',
        'cmp dword ptr [esp+8],0','jne HostFailure','mov eax,dword ptr [esp+12]',
        'cmp eax,ExpectedCurveRef','jne HostFailure','cmp dword ptr [eax],0','jne HostFailure',
        'inc CurveLookups','cmp dword ptr [InputBuffer+28],0','je MissingList',
        'mov dword ptr [eax],OFFSET FakeList','mov CurveState,2','mov eax,1','ret 12',
        'MissingList:','mov dword ptr [eax],0','mov CurveState,0','xor eax,eax','ret 12','ManagerLookup ENDP',
        'GetProperty PROC','cmp ecx,OFFSET FakeList','jne HostFailure',
        'cmp CurveState,2','jne HostFailure','mov eax,dword ptr [esp+4]',
        'cmp eax,CurrentID','jne HostFailure','inc CurveProperties','mov CurveState,3',
        'mov eax,CurrentSlot','imul eax,eax,144','add eax,OFFSET InputBuffer+192',
        'mov ecx,dword ptr [eax+4]','mov dword ptr [PropertyBuffer+8],ecx',
        'mov dword ptr [PropertyBuffer+12],0','mov dword ptr [PropertyBuffer+16],030h',
        'cmp dword ptr [eax],1','je InlineProperty','add eax,16',
        'mov dword ptr [PropertyBuffer],eax','mov dword ptr [PropertyBuffer+4],0','jmp PropertyReady',
        'InlineProperty:','mov edx,dword ptr [eax+16]','mov dword ptr [PropertyBuffer],edx',
        'mov edx,dword ptr [eax+20]','mov dword ptr [PropertyBuffer+4],edx',
        'mov dword ptr [PropertyBuffer+16],0','mov word ptr [PropertyBuffer+18],cx',
        'PropertyReady:','mov eax,OFFSET PropertyBuffer','ret 4','GetProperty ENDP',
        'ListRelease PROC','cmp ecx,OFFSET FakeList','jne HostFailure','cmp CurveState,3','jne HostFailure',
        'inc CurveReleases','mov CurveState,0','xor eax,eax','ret','ListRelease ENDP',
        'ReadRecord PROC','push ebx','push esi',f'mov ebx,{INPUT_BYTES}','mov esi,OFFSET InputBuffer',
        'ReadLoop:','mov Transferred,0','push 0','push OFFSET Transferred','push ebx','push esi','push InputHandle',
        'call _ReadFile@20','test eax,eax','jnz ReadTransferred','call _GetLastError@0','cmp eax,109','jne HostFailure',
        f'cmp ebx,{INPUT_BYTES}','jne HostFailure','cmp Transferred,0','jne HostFailure','xor eax,eax','jmp ReadDone',
        'ReadTransferred:','mov eax,Transferred','test eax,eax','jnz ReadSome',f'cmp ebx,{INPUT_BYTES}','jne HostFailure',
        'xor eax,eax','jmp ReadDone','ReadSome:','add esi,eax','sub ebx,eax','jnz ReadLoop','mov eax,1',
        'ReadDone:','pop esi','pop ebx','ret','ReadRecord ENDP',
        'HostMain PROC']
    code_pages={start&~0xffff for start,_ in CODE}|{0xe40000}
    data_pages={a&~0xffff for a,_ in CONSTANTS}|{0x16b0000}
    for page in sorted(code_pages|data_pages):
        lines += ['push 04h','push 03000h','push 010000h',f'push 0{page:08X}h','call _VirtualAlloc@16',
                  f'cmp eax,0{page:08X}h','jne AllocationFailure']
        if page in code_pages: lines += [f'mov edi,0{page:08X}h','mov ecx,010000h','mov al,0CCh','cld','rep stosb']
    for start,end in CODE:
        lines += [f'mov esi,OFFSET Original_{start:08X}',f'mov edi,0{start:08X}h',f'mov ecx,{end-start}','rep movsb']
    for start,count in CONSTANTS:
        lines += [f'mov esi,OFFSET Constant_{start:08X}',f'mov edi,0{start:08X}h',f'mov ecx,{count}','rep movsb']
    for label,address in [('GlobalsGetterStub',0xe4cca0),('GlobalsReleaseStub',0xe82160)]:
        lines += [f'mov esi,OFFSET {label}',f'mov edi,0{address:08X}h','mov ecx,7','rep movsb']
    for page in sorted((code_pages|data_pages)-{0x16b0000}):
        protection='020h' if page in code_pages else '02h'
        lines += ['push OFFSET OldProtect',f'push {protection}','push 010000h',f'push 0{page:08X}h',
                  'call _VirtualProtect@16','test eax,eax','jz HostFailure']
    lines += ['push 0','push 0','push -1','call _FlushInstructionCache@12','test eax,eax','jz HostFailure',
        'push -10','call _GetStdHandle@4','mov InputHandle,eax','push -11','call _GetStdHandle@4','mov OutputHandle,eax',
        'NextRecord:','call ReadRecord','test eax,eax','jz HostSuccess',
        'movzx eax,word ptr [InputBuffer]','mov edx,eax','and eax,0F0FFh','cmp eax,07Fh','jne HostFailure',
        'and edx,0300h','cmp edx,0100h','je HostFailure','cmp word ptr [InputBuffer+2],0','jne HostFailure',
        'cmp dword ptr [InputBuffer+28],1','ja HostFailure',
        'mov esi,OFFSET InputBuffer+32','mov ecx,13','ValidateCount:','cmp dword ptr [esi],128','ja HostFailure',
        'add esi,4','dec ecx','jnz ValidateCount',
        'mov esi,OFFSET InputBuffer+172','mov ecx,5','ValidateReserved:','cmp dword ptr [esi],0','jne HostFailure',
        'add esi,4','dec ecx','jnz ValidateReserved',
        'mov esi,OFFSET InputBuffer+192','mov ecx,9','ValidateProperty:',
        'cmp dword ptr [esi],1','ja HostFailure','cmp dword ptr [esi+4],16','ja HostFailure',
        'cmp dword ptr [esi],1','jne ArrayExtent','cmp dword ptr [esi+4],1','ja HostFailure','ArrayExtent:',
        'cmp dword ptr [esi+8],0','jne HostFailure','cmp dword ptr [esi+12],0','jne HostFailure',
        'add esi,144','dec ecx','jnz ValidateProperty',
        'mov eax,dword ptr [InputBuffer+4]']
    for identifier in IDS:
        if identifier is not None: lines += [f'cmp eax,0{identifier:08X}h','je HostFailure']
    lines += ['cmp IDPinned,0','jne CheckPinnedID','mov PinnedID,eax','mov IDPinned,1',
        'mov dword ptr [016B4C64h],eax','push OFFSET OldProtect','push 02h','push 010000h','push 016B0000h',
        'call _VirtualProtect@16','test eax,eax','jz HostFailure','jmp IDReady',
        'CheckPinnedID:','cmp eax,PinnedID','jne PolicyFailure','IDReady:',
        'mov esi,OFFSET InputBuffer','mov edi,OFFSET InputShadow','mov ecx,372','cld','rep movsd',
        'mov edi,OFFSET CacheBuffer','mov ecx,0x1290/4','mov eax,0A5A5A5A5h','rep stosd',
        'mov esi,OFFSET InputBuffer+32','mov edi,OFFSET CacheBuffer+01204h','mov ecx,35','rep movsd',
        'mov edi,OFFSET GlobalsBuffer','mov ecx,37','mov eax,0A5A5A5A5h','rep stosd']
    for src,dst in [(8,0x74),(12,0x78),(16,0x7c),(20,0x80),(24,0x90)]:
        lines += [f'mov eax,dword ptr [InputBuffer+{src}]',f'mov dword ptr [GlobalsBuffer+0{dst:X}h],eax']
    lines += ['mov esi,OFFSET GlobalsBuffer','mov edi,OFFSET GlobalsShadow','mov ecx,37','rep movsd',
        'mov edi,OFFSET PropertyBuffer','xor eax,eax','mov ecx,5','rep stosd',
        'mov edi,OFFSET ReferenceBuffer','mov ecx,4','rep stosd',
        'mov edi,OFFSET ReplyBuffer','mov eax,0A5A5A5A5h','mov ecx,60','rep stosd',
        'mov CurveState,0','mov CurveCalls,0','mov CurveLookups,0','mov CurveProperties,0','mov CurveReleases,0',
        'mov GlobalsConstructors,0','mov GlobalsLookups,0','mov GlobalsReleases,0',
        'mov ExpectedCurveCalls,6','mov ExpectedGlobalsCalls,2',
        'mov eax,dword ptr [CacheBuffer+01224h]','or eax,dword ptr [CacheBuffer+01228h]',
        'or eax,dword ptr [CacheBuffer+0122Ch]','jz ExpectedReady',
        'mov ExpectedCurveCalls,7','mov ExpectedGlobalsCalls,1','ExpectedReady:',
        'fninit','fldcw word ptr [InputBuffer]','ldmxcsr SseControl',
        'mov NativeStack,esp','push 0','push 0','call ScalarWrapper','add esp,8',
        'cmp esp,NativeStack','jne HostFailure','cmp CurveState,0','jne HostFailure',
        'mov eax,ExpectedCurveCalls','cmp CurveCalls,eax','jne HostFailure','cmp CurveLookups,eax','jne HostFailure',
        'cmp dword ptr [InputBuffer+28],0','jne PresentCounts','xor eax,eax','PresentCounts:',
        'cmp CurveProperties,eax','jne HostFailure','cmp CurveReleases,eax','jne HostFailure',
        'mov eax,ExpectedGlobalsCalls','cmp GlobalsConstructors,eax','jne HostFailure',
        'cmp GlobalsLookups,eax','jne HostFailure','cmp GlobalsReleases,eax','jne HostFailure',
        'mov esi,OFFSET ReferenceBuffer','mov ecx,4','CheckReference:','cmp dword ptr [esi],0','jne HostFailure',
        'add esi,4','dec ecx','jnz CheckReference',
        'mov esi,OFFSET InputBuffer','mov edi,OFFSET InputShadow','mov ecx,372','repe cmpsd','jne HostFailure',
        'mov esi,OFFSET GlobalsBuffer','mov edi,OFFSET GlobalsShadow','mov ecx,37','repe cmpsd','jne HostFailure',
        'mov esi,OFFSET CacheBuffer','mov ecx,0x1204/4','CheckCachePrefix:',
        'cmp dword ptr [esi],0A5A5A5A5h','jne HostFailure','add esi,4','dec ecx','jnz CheckCachePrefix']
    changed={0x1248,0x1250,0x1251}|set(range(0x124c,0x1250))|set(range(0x1254,0x1274))
    for offset in range(0x1204,0x1290):
        if offset not in changed:
            lines += [f'mov al,byte ptr [CacheBuffer+0{offset:X}h]',
                      f'cmp al,byte ptr [InputBuffer+{32+offset-0x1204}]','jne HostFailure']
    for label in ['Input','Cache','Globals','Property','Reference','Reply']:
        for side in ['Before','After']:
            for offset in range(0,16,4):
                lines += [f'cmp dword ptr [{label}{side}+{offset}],0A5A5A5A5h','jne HostFailure']
    lines += ['stmxcsr dword ptr [ReplyBuffer+208]','fnstcw word ptr [ReplyBuffer+212]',
        'fnstsw word ptr [ReplyBuffer+214]','movzx eax,word ptr [ReplyBuffer+212]',
        'cmp ax,word ptr [InputBuffer]','jne HostFailure',
        'test word ptr [ReplyBuffer+214],03800h','jnz HostFailure',
        'mov eax,dword ptr [ReplyBuffer+208]','and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure',
        'mov esi,OFFSET CacheBuffer+01204h','mov edi,OFFSET ReplyBuffer','mov ecx,35','rep movsd']
    for label,offset in [('CurveCalls',140),('GlobalsLookups',144),('GlobalsReleases',148),
                         ('GlobalsConstructors',216),('CurveLookups',220),('CurveProperties',224),('CurveReleases',228)]:
        lines += [f'mov eax,{label}',f'mov dword ptr [ReplyBuffer+{offset}],eax']
    lines += ['mov dword ptr [ReplyBuffer+232],0','mov dword ptr [ReplyBuffer+236],0',
        'push 0','push OFFSET Transferred',f'push {OUTPUT_BYTES}','push OFFSET ReplyBuffer','push OutputHandle',
        'call _WriteFile@20','test eax,eax','jz HostFailure',f'cmp Transferred,{OUTPUT_BYTES}','jne HostFailure','jmp NextRecord',
        'HostSuccess:','push 0','call _ExitProcess@4','HostFailure::','push 7','call _ExitProcess@4',
        'AllocationFailure:','push 10','call _ExitProcess@4','PolicyFailure:','push 8','call _ExitProcess@4',
        'HostMain ENDP','OwnedEnd LABEL BYTE',
        'ORG 0C950h','ManagerProvider PROC','mov eax,OFFSET CurveEnter','jmp eax','ManagerProvider ENDP',
        'ORG 0141350h','ReferenceProvider PROC','mov eax,OFFSET ReferenceConstruct','jmp eax','ReferenceProvider ENDP',
        'END HostMain']
    # MASM uses decimal default; avoid Python-style integer notation.
    source='\n'.join(lines)+'\n'
    source=source.replace('0x1290/4',str(0x1290//4)).replace('0x1204/4',str(0x1204//4))
    return source,code_pages,data_pages


def linked_audit(path,map_path):
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_REG
    pe=PE(path);symbols={}
    for line in map_path.read_text().splitlines():
        m=re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\w+)\s+([0-9A-Fa-f]{8})\b',line)
        if m:symbols[m[1]]=int(m[2],16)
    assert symbols['ManagerProvider']==0x67d950
    assert symbols['ReferenceProvider']==0x7b2350
    fixed=[]
    for label,target,address in [('ManagerProvider','CurveEnter',0x67d950),
        ('ReferenceProvider','ReferenceConstruct',0x7b2350),
        ('GlobalsGetterStub','GlobalsLookup',None),('GlobalsReleaseStub','GlobalsRelease',None)]:
        at=symbols[label];raw=pe.read(at,7)
        assert raw==b'\xb8'+struct.pack('<I',symbols[target])+b'\xff\xe0'
        fixed.append(dict(label=label,linked=f'{at:08X}',native_address=f'{address:08X}'if address else
            '00E4CCA0'if label=='GlobalsGetterStub'else'00E82160',hex=raw.hex(),absolute_target=f'{symbols[target]:08X}',relative_transfers=0))
    imports=pe.imports()
    assert {name for _,name in imports.values()}==IMPORTS
    assert {name.upper()for name,_ in imports.values()}=={'KERNEL32.DLL'}
    start,end=symbols['OwnedStart'],symbols['OwnedEnd']
    code=list(decoder().disasm(pe.read(start,end-start),start));assert sum(i.size for i in code)==end-start
    boundaries={i.address for i in code};calls=[]
    for index,ins in enumerate(code):
        assert ins.mnemonic not in {'int','syscall','sysenter','hlt','iret','iretd'}
        if ins.group(CS_GRP_JUMP):
            op=ins.operands[0]
            if op.type==X86_OP_IMM:assert op.imm in boundaries
            else:
                assert op.type==X86_OP_REG and ins.op_str=='eax'
                previous=code[index-1];assert previous.mnemonic=='mov'and previous.operands[1].type==X86_OP_IMM
                assert previous.operands[1].imm in {symbols['GlobalsLookup'],symbols['GlobalsRelease'],0xe524a4}
        if ins.group(CS_GRP_CALL):
            op=ins.operands[0];assert op.type==X86_OP_IMM
            if op.imm in boundaries:assert op.imm in {symbols['ReadRecord'],symbols['ScalarWrapper']}
            else:
                raw=pe.read(op.imm,6);assert raw[:2]==b'\xff\x25'
                assert struct.unpack('<I',raw[2:])[0]in imports
            calls.append(dict(at=f'{ins.address:08X}',target=f'{op.imm:08X}'))
    # Inspect actual PE characteristics for every fixed stub and immutable vtable.
    pe_at=struct.unpack_from('<I',pe.data,0x3c)[0];n=struct.unpack_from('<H',pe.data,pe_at+6)[0]
    size=struct.unpack_from('<H',pe.data,pe_at+20)[0];sections=[]
    for i in range(n):
        at=pe_at+24+size+40*i;vs,rva=struct.unpack_from('<II',pe.data,at+8)
        sections.append((pe.base+rva,pe.base+rva+vs,struct.unpack_from('<I',pe.data,at+36)[0]))
    for label in ['ManagerProvider','ReferenceProvider','GlobalsGetterStub','GlobalsReleaseStub']:
        flags=next(f for a,b,f in sections if a<=symbols[label]<b);assert flags&0xe0000000==0x60000000
    for label in ['ManagerVtable','ListVtable','FakeManager','FakeList']:
        flags=next(f for a,b,f in sections if a<=symbols[label]<b);assert flags&0xe0000000==0x40000000
    return dict(symbols={k:f'{v:08X}'for k,v in symbols.items()if not k.startswith('_')},
        owned_code_sha256=hashlib.sha256(pe.read(start,end-start)).hexdigest(),calls=calls,
        fixed_stubs=fixed,immutable_vtables=True,linked_stubs_RX_nonwritable=True,
        imports=[dict(address=f'{a:08X}',dll=dll,name=name)for a,(dll,name)in imports.items()],
        owned_disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}'for i in code])


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('exe',type=Path)
    ap.add_argument('--sdk-bin-x86',type=Path,required=True);ap.add_argument('--kernel32-lib',type=Path,required=True)
    ap.add_argument('--out',type=Path,required=True);args=ap.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    image=SourceImage(args.exe);native=audit_source(image);source,code_pages,data_pages=host_source(image)
    args.out.mkdir(parents=True,exist_ok=False);out=args.out.resolve()
    assembly,obj,exe,mapping=[out/name for name in ['host-x86.asm','host-x86.obj','host-x86.exe','host-x86.map']]
    assembly.write_text(source)
    commands=[[str(args.sdk_bin_x86/'ml.exe'),'/nologo','/c','/coff',f'/Fo{obj}',str(assembly)],
        [str(args.sdk_bin_x86/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain',
         '/BASE:0x00670000','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{exe}',f'/MAP:{mapping}',str(obj),str(args.kernel32_lib)]]
    for command in commands:
        result=subprocess.run(command,capture_output=True,text=True);assert result.returncode==0,result.stdout+'\n'+result.stderr
    linked=linked_audit(exe,mapping)
    manifest=dict(schema='bounded-original-cell-scalar-cache-host-v1',host_executed=False,source_sha256=SOURCE_SHA256,
        code_allowlist=native,data_allowlist=[dict(address=f'{a:08X}',bytes=n,sha256=hashlib.sha256(image.read(a,n)).hexdigest())for a,n in CONSTANTS],
        explicit_global=dict(address='016B4C64',policy='First admitted request sets supplied ID, then entire page becomes R before source entry; later changes reject with exit8.'),
        host_image_base='00670000',host_source_sha256=hashlib.sha256(assembly.read_bytes()).hexdigest(),
        host_exe_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),linked_audit=linked,compile_commands=commands,
        pages=[dict(address=f'{a:08X}',bytes=65536,initial='RW',final='RX'if a in code_pages else'R',
            finalization='before native entry;016B0000 after first admitted ID')for a in sorted(code_pages|data_pages)],
        protocol=dict(input_bytes=INPUT_BYTES,output_bytes=OUTPUT_BYTES,
            input='CW:u16@0,reserved:u16@2,mutableCurveID:u32@4,Globals74/78/7C/80/90:f32@8..27,listPresent:u32@28,cache1204..128F:140bytes@32,reserved0@172..191,9slots144bytes@192:storage:u32(0array/1inline),count:u32,reserved8bytes,16Vector2rows@16',
            output='cachetail140@0,curveCalls@140,globalsGets/Releases@144/148,7ID/querypairs@152,MXCSR@208,CW/SW@212/214,globalsCtors@216,curveGets/props/releases@220/224/228,reserved0@232/236',
            bounds='13counts0..128;points0..16;inline0..1;listPresent0/1;12maskedCWpolicies;explicitID distinct from eight other fixed curveIDs. Native nonfinite data/results retained; finite API policy checked separately.',
            curve_slot_ids=[f'{x:08X}'if x is not None else'explicit mutable ID'for x in IDS]),
        entry_proof='Owned wrapper reproduces SUBESP30/PUSH EBX/EBP/EDI, supplies EBX1/ESI guarded cache, and absolute-jumps to E524A4. Caller reserves both original argument slots. Native epilogue restores frame and returns normally; exact outer ESP checked before any restoration.',
        provider_proof='CurveEnter checks exact E837F0 getter return, scalar callsite/ID, empty local reference and <=7calls, captures exact ID/query pairs. Immutable vtables reach checked manager/property/release only; property points are bounded input arrays or20byte inline object. Globals shims check exact source callsites and two original stack-reference slots; refcount returns zero. All addresses are owned and no request supplies pointers.',
        guard_proof='Input/cache/Globals/property/reference/reply each have separate16byte canaries. Full input and Globals shadows, cache prefix and every unwritten tail byte checked before context stores. Postcall CW unchanged, TOP0, MXCSRcontrols1F80; sticky flags may vary.',
        execution_scope='Preparation only. No game/CRT/retail module loaded, no initializer/archive/actor, no desktop/window/network. Root must review fresh source/linked boundaries and hashes before separate hidden execution.')
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({k:manifest[k]for k in ['schema','host_executed','host_source_sha256','host_exe_sha256']},indent=2))


if __name__=='__main__':main()

"""Prepare, never execute, the bounded original optional-foot construction.

All driver/minimum/point/rotation/append/constructor/capability arithmetic is
unchanged original code. Owned providers supply fixed storage, bounded memset
and excluded debug-name formatting. New artifacts require separate review.
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
from native_crt_acos_probe import PE, decoder, IMPORTS

ROOT=Path(__file__).resolve().parent.parent
CODE=[(0x99BE70,0x99BFD1),(0x9B2DA0,0x9B2E11),(0x9B2E20,0x9B2E5C),
    (0x9B2EA0,0x9B2EE6),(0x9B2F90,0x9B31F5),(0x9C84B0,0x9C855D),
    (0x9C8560,0x9C8657),(0x9C8660,0x9C8704),(0x9CBAB0,0x9CBE61),(0x9CBE70,0x9CBF97)]
CONSTANTS=[(0x13EB8B0,16),(0x14861D0,4),(0x1471B2C,4),(0x13EB970,4),(0x1447F20,4),(0x14719E4,4)]
THRESHOLD=0x155223C
STUBS={0x9CB900:'GrowFixed',0x938150:'DebugName',0x11E052E:'ZeroConstructorStack'}
# Read/write projection only. Debug strings and unrelated constructor padding
# are deliberately excluded; original code still writes its actual fields.
FIELDS=[0x104,0x108,0x1F8,0x1FC,0x200,0x21C,0x208]+list(range(0x10C,0x150,4))+list(range(0x344,0x36C,4))+[0x36C,0x378,0x37C]
FIELD_INDEX={offset:168+4*i for i,offset in enumerate(FIELDS)}
RECORD_BYTES=168+4*len(FIELDS)
INPUT_BYTES=16+255*RECORD_BYTES
PROJECTION_BYTES=16+255*RECORD_BYTES
OUTPUT_BYTES=PROJECTION_BYTES+8
BLOCK_BYTES=255*0x468
FRAME_BYTES=4096
BUFFERS=[('Input',INPUT_BYTES),('Definition',1024),('Blocks',BLOCK_BYTES),('Frame',FRAME_BYTES),('Reply',OUTPUT_BYTES)]


def audit_source(image):
    sys.path.insert(0,str(ROOT/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL,CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM,X86_OP_MEM
    decoded=[list(decoder().disasm(image.read(a,b-a),a)) for a,b in CODE]
    boundaries={i.address for part in decoded for i in part}
    result=[]
    for (start,end),rows in zip(CODE,decoded):
        assert sum(i.size for i in rows)==end-start
        transfers,reads=[],[]
        for ins in rows:
            if ins.group(CS_GRP_CALL) or ins.group(CS_GRP_JUMP):
                assert ins.operands[0].type==X86_OP_IMM
                target=ins.operands[0].imm
                assert target in boundaries or target in STUBS,(hex(ins.address),hex(target))
                transfers.append(dict(at=f'{ins.address:08X}',operation=ins.mnemonic,target=f'{target:08X}'))
            for op in ins.operands:
                if op.type==X86_OP_MEM and not op.mem.base:
                    assert not op.mem.index and op.mem.disp in [a for a,_ in CONSTANTS]+[THRESHOLD]
                    reads.append(dict(at=f'{ins.address:08X}',address=f'{op.mem.disp:08X}',bytes=op.size))
        result.append(dict(start=f'{start:08X}',end_exclusive=f'{end:08X}',bytes=end-start,
            sha256=hashlib.sha256(image.read(start,end-start)).hexdigest(),transfers=transfers,absolute_reads=reads,
            disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in rows]))
    assert image.read(0x14861D0,4)==struct.pack('<I',0x3F800000)
    assert image.read(0x1471B2C,4)==struct.pack('<I',0x3F000000)
    assert image.read(0x13EB970,4)==struct.pack('<I',0x3C23D70A)
    assert image.read(0x1447F20,4)==struct.pack('<I',0x7F7FFFFF)
    assert image.read(0x14719E4,4)==struct.pack('<I',0x40000000)
    return result


def host_source(image):
    lines=['.686p','.xmm','.model flat','option casemap:none']
    for name,size in [('GetStdHandle',4),('ReadFile',20),('WriteFile',20),('VirtualAlloc',16),
        ('VirtualProtect',16),('GetLastError',0),('FlushInstructionCache',12),('ExitProcess',4)]:
        lines += [f'EXTERN _{name}@{size}:PROC']
    lines+=['.const']
    for a,b in CODE:lines+=db(f'Original_{a:08X}',image.read(a,b-a))
    for a,size in CONSTANTS:lines+=db(f'Constant_{a:08X}',image.read(a,size))
    lines+=['.data']
    for label,size in BUFFERS:
        lines += [f'{label}Before DWORD 4 DUP(0A5A5A5A5h)',f'{label}Buffer BYTE {size} DUP(0)',f'{label}After DWORD 4 DUP(0A5A5A5A5h)']
    lines += [f'InputShadow BYTE {INPUT_BYTES} DUP(0)',f'BlocksShadow BYTE {BLOCK_BYTES} DUP(0)',
        'DefinitionShadow BYTE 1024 DUP(0)',f'Expected BYTE {PROJECTION_BYTES} DUP(0)',
        'InputHandle DWORD 0','OutputHandle DWORD 0','Transferred DWORD 0','OldProtect DWORD 0',
        'NativeStack DWORD 0','Poison DWORD 0','InitialCandidates DWORD 0','GrowCalls DWORD 0',
        'SseControl DWORD 01F80h','.code',
        'PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,ReadRecord,ValidateRecord,RunOnce,OriginalDone',
        'PUBLIC GrowFixed,DebugName,ZeroConstructorStack',
        'PUBLIC InputBuffer,DefinitionBuffer,BlocksBuffer,FrameBuffer,ReplyBuffer','OwnedStart LABEL BYTE']
    for at,label in STUBS.items():
        lines += [f'PUBLIC Stub_{at:08X}',f'Stub_{at:08X} LABEL BYTE',f'mov eax,OFFSET {label}','jmp eax']
    lines += ['GrowFixed PROC','cmp dword ptr [esp],009CBB09h','jne HostFailure',
        'cmp ecx,OFFSET DefinitionBuffer+0384h','jne HostFailure',
        'cmp dword ptr [ecx],OFFSET BlocksBuffer','jne HostFailure',
        'cmp dword ptr [ecx+8],OFFSET BlocksBuffer+287640','jne HostFailure',
        'mov eax,dword ptr [esp+4]','cmp eax,255','ja HostFailure',
        'imul eax,eax,0468h','add eax,OFFSET BlocksBuffer','mov edx,eax',
        'sub edx,dword ptr [ecx+4]','cmp edx,0468h','je GrowthReady',
        'cmp edx,08D0h','jne HostFailure','GrowthReady:',
        'inc GrowCalls','mov edx,GrowCalls','cmp edx,InitialCandidates','ja HostFailure',
        'mov dword ptr [ecx+4],eax','ret 4','GrowFixed ENDP',
        'DebugName PROC','mov eax,dword ptr [esp]','cmp eax,009CBB48h','je FirstDebugName',
        'cmp eax,009CBB5Fh','jne HostFailure','cmp dword ptr [esp+12],014483B4h','jne HostFailure',
        'mov edx,dword ptr [esp+16]','cmp edx,dword ptr [InputBuffer+4]','jb HostFailure',
        'cmp edx,255','jae HostFailure','imul edx,edx,0468h','add edx,OFFSET BlocksBuffer+0220h',
        'cmp edx,dword ptr [esp+4]','jne HostFailure','jmp DebugBounds',
        'FirstDebugName:','cmp dword ptr [esp+12],013F5B58h','jne HostFailure',
        'cmp dword ptr [esp+16],014483C4h','jne HostFailure','mov edx,dword ptr [esp+4]',
        'cmp edx,OFFSET BlocksBuffer','jb HostFailure','mov eax,edx','sub eax,OFFSET BlocksBuffer',
        'xor edx,edx','mov ecx,0468h','div ecx','test edx,edx','jnz HostFailure',
        'cmp eax,dword ptr [InputBuffer+4]','jb HostFailure','cmp eax,255','jae HostFailure',
        'DebugBounds:','cmp dword ptr [esp+8],0104h','jne HostFailure',
        'mov eax,dword ptr [esp+4]','cmp eax,OFFSET BlocksBuffer','jb HostFailure',
        'lea edx,[eax+0104h]','cmp edx,dword ptr [DefinitionBuffer+0388h]','ja HostFailure',
        'mov byte ptr [eax],0','xor eax,eax','ret','DebugName ENDP',
        'ZeroConstructorStack PROC','cmp dword ptr [esp],009B305Ch','jne HostFailure',
        'cmp dword ptr [esp+8],0','jne HostFailure','cmp dword ptr [esp+12],0A8h','jne HostFailure',
        'mov edx,dword ptr [esp+4]','cmp edx,OFFSET FrameBuffer+0200h','jb HostFailure',
        'lea eax,[edx+0A8h]','cmp eax,OFFSET FrameBuffer+0800h','ja HostFailure',
        'push edi','mov edi,edx','mov ecx,0A8h','xor eax,eax','cld','rep stosb','pop edi',
        'mov eax,edx','ret','ZeroConstructorStack ENDP',
        'ReadRecord PROC','push ebx','push esi',f'mov ebx,{INPUT_BYTES}',
        'mov esi,OFFSET InputBuffer','ReadLoop:','mov Transferred,0',
        'push 0','push OFFSET Transferred','push ebx','push esi','push InputHandle','call _ReadFile@20',
        'test eax,eax','jnz ReadTransferred','call _GetLastError@0','cmp eax,109','jne HostFailure',
        f'cmp ebx,{INPUT_BYTES}','jne HostFailure','cmp Transferred,0','jne HostFailure','xor eax,eax','jmp ReadDone',
        'ReadTransferred:','mov eax,Transferred','cmp eax,ebx','ja HostFailure','test eax,eax','jnz ReadSome',
        f'cmp ebx,{INPUT_BYTES}','jne HostFailure','xor eax,eax','jmp ReadDone',
        'ReadSome:','add esi,eax','sub ebx,eax','jnz ReadLoop','mov eax,1','ReadDone:',
        'pop esi','pop ebx','ret','ReadRecord ENDP',
        'ValidateRecord PROC','movzx eax,word ptr [InputBuffer]','mov edx,eax','and eax,0F0FFh',
        'cmp eax,07Fh','jne HostFailure','and edx,0300h','cmp edx,0100h','je HostFailure',
        'cmp word ptr [InputBuffer+2],0','jne HostFailure','mov ecx,dword ptr [InputBuffer+4]',
        'test ecx,ecx','jz HostFailure','cmp ecx,255','ja HostFailure',
        'mov eax,dword ptr [InputBuffer+8]','and eax,07F800000h','cmp eax,07F800000h','je HostFailure',
        'cmp dword ptr [InputBuffer+12],1','ja HostFailure','mov InitialCandidates,0',
        'xor ebx,ebx','mov esi,OFFSET InputBuffer+16','ValidateBlocks:']
    for offset in [0,4]:
        lines += [f'cmp dword ptr [esi+{offset}],255','ja HostFailure']
    lines += ['mov eax,dword ptr [esi+4]','mov edx,dword ptr [esi]','not edx','test eax,edx','jnz HostFailure',
        'test eax,4','jnz HostFailure',f'cmp dword ptr [esi+{FIELD_INDEX[0x1F8]}],ebx','jne HostFailure',
        f'cmp dword ptr [esi+{FIELD_INDEX[0x21C]}],ebx','jne HostFailure',
        f'mov eax,dword ptr [esi+{FIELD_INDEX[0x1FC]}]','cmp eax,-1','jne ValidateParent',
        'test ebx,ebx','jnz HostFailure','jmp ParentReady','ValidateParent:','cmp eax,ecx','jae HostFailure',
        'ParentReady:',f'mov eax,dword ptr [esi+{FIELD_INDEX[0x200]}]','cmp eax,-1','je SymmetryReady',
        'cmp eax,ecx','jae HostFailure','SymmetryReady:']
    for offset in range(0x10C,0x150,4):
        lines += [f'mov eax,dword ptr [esi+{FIELD_INDEX[offset]}]','and eax,07FFFFFFFh',
            'cmp eax,07F800000h','jae HostFailure']
        if offset in list(range(0x118,0x138,4)):
            lines += ['cmp eax,040000000h','ja HostFailure']
        else:
            lines += ['cmp eax,071800000h','ja HostFailure']
    for offset in [0x138,0x13C,0x140]:
        lines += [f'cmp dword ptr [esi+{FIELD_INDEX[offset]}],0','jle HostFailure']
    lines += [f'cmp dword ptr [esi+{FIELD_INDEX[0x208]}],255','ja HostFailure',
        f'cmp dword ptr [esi+{FIELD_INDEX[0x208]}],0','jne NextBlock','test byte ptr [esi],2','jz NextBlock',
        'test byte ptr [esi],4','jnz NextBlock','test byte ptr [esi],060h','jnz HostFailure',
        'inc InitialCandidates','NextBlock:','inc ebx',f'add esi,{RECORD_BYTES}','cmp ebx,ecx','jb ValidateBlocks',
        'mov eax,InitialCandidates','shl eax,1','add eax,ecx','cmp eax,255','ja HostFailure',
        # Every declared child count must agree with these bounded parents.
        'xor ebx,ebx','ValidateChildren:','xor edi,edi','xor ebp,ebp','mov esi,OFFSET InputBuffer+16',
        'CountChildren:',f'cmp dword ptr [esi+{FIELD_INDEX[0x1FC]}],ebx','jne ChildCountNext',
        'inc edi','ChildCountNext:',f'add esi,{RECORD_BYTES}','inc ebp','cmp ebp,ecx','jb CountChildren',
        f'imul eax,ebx,{RECORD_BYTES}',f'cmp edi,dword ptr [InputBuffer+16+{FIELD_INDEX[0x208]}+eax]','jne HostFailure',
        # All parents are bounded before walking; each chain must terminate.
        'mov eax,ebx','xor edx,edx','WalkParents:','cmp eax,-1','je WalkDone',
        'cmp edx,ecx','jae HostFailure',f'imul eax,eax,{RECORD_BYTES}',
        f'mov eax,dword ptr [InputBuffer+16+{FIELD_INDEX[0x1FC]}+eax]','inc edx','jmp WalkParents',
        'WalkDone:','inc ebx','cmp ebx,ecx','jb ValidateChildren',
        # Fixed unused records are zero, including unused capability bytes.
        f'imul eax,ecx,{RECORD_BYTES}','lea esi,[InputBuffer+16+eax]',f'mov ecx,{255*RECORD_BYTES}',
        'sub ecx,eax','jz PaddingDone','PaddingLoop:','cmp byte ptr [esi],0','jne HostFailure',
        'inc esi','dec ecx','jnz PaddingLoop','PaddingDone:','ret','ValidateRecord ENDP',
        'RunOnce PROC','mov NativeStack,esp','mov GrowCalls,0']
    for label,size in BUFFERS:
        if label!='Input':
            fill='0A7A7A7A7h' if label=='Blocks' else '0'
            lines += [f'mov edi,OFFSET {label}Buffer',f'mov eax,{fill}',f'mov ecx,{size//4}','cld','rep stosd']
    lines += ['mov esi,OFFSET InputBuffer+16','mov edi,OFFSET BlocksBuffer',
        'mov ebx,dword ptr [InputBuffer+4]','BuildBlocks:','push esi','push edi',
        'add edi,0150h','mov ecx,42','rep movsd','pop edi','pop esi']
    for offset in FIELDS:
        lines += [f'mov eax,dword ptr [esi+{FIELD_INDEX[offset]}]',f'mov dword ptr [edi+0{offset:X}h],eax']
    lines += [f'add esi,{RECORD_BYTES}','add edi,0468h','dec ebx','jnz BuildBlocks',
        'mov dword ptr [DefinitionBuffer+0384h],OFFSET BlocksBuffer','mov dword ptr [DefinitionBuffer+0388h],edi',
        f'mov dword ptr [DefinitionBuffer+038Ch],OFFSET BlocksBuffer+{BLOCK_BYTES}',
        'mov eax,dword ptr [InputBuffer+12]','mov byte ptr [DefinitionBuffer+03F5h],al']
    for label,size in [('Input',INPUT_BYTES),('Definition',1024),('Blocks',BLOCK_BYTES)]:
        lines += [f'mov esi,OFFSET {label}Buffer',f'mov edi,OFFSET {label}Shadow',f'mov ecx,{size//4}','cld','rep movsd']
    lines += ['fninit','fldcw word ptr [InputBuffer]','ldmxcsr SseControl','cmp Poison,0','jne ScratchOnes']
    for n in range(8):lines += [f'pxor xmm{n},xmm{n}']
    lines += ['jmp ScratchReady','ScratchOnes:']
    for n in range(8):lines += [f'pcmpeqd xmm{n},xmm{n}']
    lines += ['ScratchReady:','mov esp,OFFSET FrameBuffer+0800h','push OFFSET DefinitionBuffer',
        'push OFFSET OriginalDone','mov eax,009CBE70h','jmp eax','OriginalDone::',
        'cmp esp,OFFSET FrameBuffer+07FCh','jne HostFailure','add esp,4','mov esp,NativeStack']
    for label,_ in BUFFERS:
        for side in ['Before','After']:
            for at in range(0,16,4):lines += [f'cmp dword ptr [{label}{side}+{at}],0A5A5A5A5h','jne HostFailure']
    lines += ['mov esi,OFFSET InputBuffer','mov edi,OFFSET InputShadow',f'mov ecx,{INPUT_BYTES//4}',
        'cld','repe cmpsd','jne HostFailure','xor ecx,ecx','CheckDefinition:',
        'cmp ecx,0388h','jb DefinitionUnchanged','cmp ecx,038Ch','jb NextDefinitionByte',
        'cmp ecx,03C8h','jb DefinitionUnchanged','cmp ecx,03CCh','jb NextDefinitionByte',
        'cmp ecx,03F5h','je NextDefinitionByte','DefinitionUnchanged:',
        'mov al,byte ptr [DefinitionBuffer+ecx]','cmp al,byte ptr [DefinitionShadow+ecx]','jne HostFailure',
        'NextDefinitionByte:','inc ecx','cmp ecx,1024','jb CheckDefinition',
        'mov eax,dword ptr [DefinitionBuffer+0388h]','sub eax,OFFSET BlocksBuffer',
        'xor edx,edx','mov ecx,0468h','div ecx','test edx,edx','jnz HostFailure',
        'cmp eax,dword ptr [InputBuffer+4]','jb HostFailure','cmp eax,255','ja HostFailure',
        'mov dword ptr [ReplyBuffer],eax','sub eax,dword ptr [InputBuffer+4]',
        'cmp eax,dword ptr [DefinitionBuffer+03C8h]','jne HostFailure',
        'mov dword ptr [ReplyBuffer+4],eax','movzx eax,byte ptr [DefinitionBuffer+03F5h]',
        'cmp eax,1','ja HostFailure','mov dword ptr [ReplyBuffer+8],eax',
        'mov eax,GrowCalls','mov dword ptr [ReplyBuffer+12],eax',
        # Source prefix unchanged except child count; unused tail unchanged.
        'xor ebx,ebx','mov esi,OFFSET BlocksBuffer','mov edi,OFFSET BlocksShadow','CheckSourceBlock:',
        'xor ecx,ecx','CheckSourceByte:','cmp ecx,0208h','jb SourceByteUnchanged',
        'cmp ecx,020Ch','jb NextSourceByte','SourceByteUnchanged:',
        'mov al,byte ptr [esi+ecx]','cmp al,byte ptr [edi+ecx]','jne HostFailure',
        'NextSourceByte:','inc ecx','cmp ecx,0468h','jb CheckSourceByte',
        'add esi,0468h','add edi,0468h','inc ebx','cmp ebx,dword ptr [InputBuffer+4]','jb CheckSourceBlock',
        'mov esi,dword ptr [DefinitionBuffer+0388h]','mov ecx,OFFSET BlocksBuffer',f'add ecx,{BLOCK_BYTES}',
        'sub ecx,esi','jz TailChecked','CheckUnusedTail:','cmp byte ptr [esi],0A7h','jne HostFailure',
        'inc esi','dec ecx','jnz CheckUnusedTail','TailChecked:',
        'mov esi,OFFSET BlocksBuffer','mov edi,OFFSET ReplyBuffer+16','mov ebx,dword ptr [ReplyBuffer]','ProjectBlock:',
        'push esi','push edi','add esi,0150h','mov ecx,42','cld','rep movsd','pop edi','pop esi']
    for offset in FIELDS:
        lines += [f'mov eax,dword ptr [esi+0{offset:X}h]',f'mov dword ptr [edi+{FIELD_INDEX[offset]}],eax']
    lines += ['add esi,0468h',f'add edi,{RECORD_BYTES}','dec ebx','jnz ProjectBlock',
        f'fnstcw word ptr [ReplyBuffer+{PROJECTION_BYTES+4}]',f'fnstsw word ptr [ReplyBuffer+{PROJECTION_BYTES+6}]',
        f'stmxcsr dword ptr [ReplyBuffer+{PROJECTION_BYTES}]',
        'mov ax,word ptr [InputBuffer]',f'cmp ax,word ptr [ReplyBuffer+{PROJECTION_BYTES+4}]','jne HostFailure',
        f'test word ptr [ReplyBuffer+{PROJECTION_BYTES+6}],03800h','jnz HostFailure',
        f'mov eax,dword ptr [ReplyBuffer+{PROJECTION_BYTES}]','and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure']
    for side in ['Before','After']:
        for at in range(0,16,4):lines += [f'cmp dword ptr [Reply{side}+{at}],0A5A5A5A5h','jne HostFailure']
    lines += ['ret','RunOnce ENDP','HostMain PROC']
    code_pages={a & ~0xFFFF for a,_ in CODE}|{a & ~0xFFFF for a in STUBS}
    data_pages={a & ~0xFFFF for a,_ in CONSTANTS}|{THRESHOLD & ~0xFFFF}
    assert not code_pages & data_pages
    for page in sorted(code_pages|data_pages):
        lines += ['push 04h','push 03000h','push 010000h',f'push 0{page:08X}h','call _VirtualAlloc@16',
            f'cmp eax,0{page:08X}h','jne AllocationFailure']
        if page in code_pages:
            lines += [f'mov edi,0{page:08X}h','mov ecx,010000h','mov al,0CCh','cld','rep stosb']
    for at,end in CODE:
        lines += [f'mov esi,OFFSET Original_{at:08X}',f'mov edi,0{at:08X}h',f'mov ecx,{end-at}','cld','rep movsb']
    for at in STUBS:
        lines += [f'mov esi,OFFSET Stub_{at:08X}',f'mov edi,0{at:08X}h','mov ecx,7','rep movsb']
    for at,size in CONSTANTS:
        lines += [f'mov esi,OFFSET Constant_{at:08X}',f'mov edi,0{at:08X}h',f'mov ecx,{size}','rep movsb']
    for page in sorted(code_pages|data_pages):
        lines += ['push OFFSET OldProtect',f'push {"020h" if page in code_pages else "02h"}',
            'push 010000h',f'push 0{page:08X}h','call _VirtualProtect@16','test eax,eax','jz HostFailure']
        if page in code_pages:
            lines += ['push 010000h',f'push 0{page:08X}h','push -1','call _FlushInstructionCache@12',
                'test eax,eax','jz HostFailure']
    lines += ['push -10','call _GetStdHandle@4','mov InputHandle,eax',
        'push -11','call _GetStdHandle@4','mov OutputHandle,eax','MainLoop:',
        'call ReadRecord','test eax,eax','jz HostSuccess','call ValidateRecord',
        'push OFFSET OldProtect','push 04h','push 010000h','push 01550000h','call _VirtualProtect@16',
        'test eax,eax','jz HostFailure','mov eax,dword ptr [InputBuffer+8]','mov dword ptr [0155223Ch],eax',
        'push OFFSET OldProtect','push 02h','push 010000h','push 01550000h','call _VirtualProtect@16',
        'test eax,eax','jz HostFailure','mov Poison,0','call RunOnce',
        'mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected',f'mov ecx,{PROJECTION_BYTES//4}','cld','rep movsd',
        'mov Poison,1','call RunOnce','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected',
        f'mov ecx,{PROJECTION_BYTES//4}','cld','repe cmpsd','jne HostFailure',
        'mov Transferred,0','push 0','push OFFSET Transferred',f'push {OUTPUT_BYTES}','push OFFSET ReplyBuffer',
        'push OutputHandle','call _WriteFile@20','test eax,eax','jz HostFailure',
        f'cmp Transferred,{OUTPUT_BYTES}','jne HostFailure','jmp MainLoop',
        'HostSuccess:','push 0','call _ExitProcess@4','HostFailure::','push 7','call _ExitProcess@4',
        'AllocationFailure:','push 10','call _ExitProcess@4','HostMain ENDP','OwnedEnd LABEL BYTE','END HostMain']
    return '\n'.join(lines)+'\n',code_pages,data_pages


def linked_audit(exe,map_path):
    from capstone import CS_GRP_CALL,CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM,X86_OP_REG
    pe,symbols=PE(exe),{}
    for line in map_path.read_text().splitlines():
        match=re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\w+)\s+([0-9A-Fa-f]{8})\b',line)
        if match:symbols[match[1]]=int(match[2],16)
    imports=pe.imports();assert {name for _,name in imports.values()}==IMPORTS
    assert {dll.upper() for dll,_ in imports.values()}=={'KERNEL32.DLL'}
    start,end=symbols['OwnedStart'],symbols['OwnedEnd'];code=list(decoder().disasm(pe.read(start,end-start),start))
    assert sum(i.size for i in code)==end-start
    boundaries={i.address for i in code}
    for n,ins in enumerate(code):
        assert ins.mnemonic not in {'int','syscall','sysenter','hlt','iret','iretd'}
        if ins.group(CS_GRP_JUMP):
            op=ins.operands[0]
            if op.type==X86_OP_IMM:assert op.imm in boundaries
            else:
                assert op.type==X86_OP_REG and ins.op_str=='eax'
                previous=code[n-1];assert previous.mnemonic=='mov' and previous.operands[1].type==X86_OP_IMM
                assert previous.operands[1].imm in {0x9CBE70}|{symbols[v] for v in STUBS.values()}
        if ins.group(CS_GRP_CALL):
            assert ins.operands[0].type==X86_OP_IMM;target=ins.operands[0].imm
            if target in boundaries:assert target in {symbols[v] for v in ['ReadRecord','ValidateRecord','RunOnce']}
            else:
                thunk=pe.read(target,6);assert thunk[:2]==b'\xFF\x25' and struct.unpack('<I',thunk[2:])[0] in imports
    for at,label in STUBS.items():
        assert pe.read(symbols[f'Stub_{at:08X}'],7)==b'\xB8'+struct.pack('<I',symbols[label])+b'\xFF\xE0'
    assert pe.base==0x24000000
    pe_at=struct.unpack_from('<I',pe.data,0x3C)[0]
    section_count=struct.unpack_from('<H',pe.data,pe_at+6)[0];optional=struct.unpack_from('<H',pe.data,pe_at+20)[0]
    sections=[]
    for i in range(section_count):
        at=pe_at+24+optional+40*i;size,rva=struct.unpack_from('<II',pe.data,at+8)
        sections.append((pe.base+rva,pe.base+rva+size,struct.unpack_from('<I',pe.data,at+36)[0]))
    for name in ['HostMain','OriginalDone','GrowFixed','DebugName','ZeroConstructorStack']:
        flags=next(flags for a,b,flags in sections if a<=symbols[name]<b);assert flags & 0xE0000000==0x60000000
    for first,(label,size) in enumerate(BUFFERS):
        at=symbols[label+'Buffer'];flags=next(flags for a,b,flags in sections if a<=at-16 and at+size+16<=b)
        assert flags & 0xE0000000==0xC0000000
        for other,other_size in BUFFERS[first+1:]:
            other_at=symbols[other+'Buffer'];assert at+size+16<=other_at-16 or other_at+other_size+16<=at-16
    return dict(symbols={k:f'{v:08X}' for k,v in symbols.items() if not k.startswith('_')},
        owned_code_sha256=hashlib.sha256(pe.read(start,end-start)).hexdigest(),
        imports=[dict(iat=f'{a:08X}',dll=dll,name=name) for a,(dll,name) in imports.items()],
        disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in code])


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('exe',type=Path);p.add_argument('--sdk-bin-x86',type=Path,required=True)
    p.add_argument('--kernel32-lib',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
    a.out.resolve().relative_to((ROOT/'target').resolve());image=SourceImage(a.exe)
    records=audit_source(image);text,code_pages,data_pages=host_source(image)
    a.out.mkdir(parents=True,exist_ok=False)
    asm,obj,exe,mapping=[a.out.resolve()/name for name in ['host-x86.asm','host-x86.obj','host-x86.exe','host-x86.map']]
    asm.write_text(text)
    commands=[[str(a.sdk_bin_x86/'ml.exe'),'/nologo','/c','/coff',f'/Fo{obj}',str(asm)],
        [str(a.sdk_bin_x86/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain',
        '/BASE:0x24000000','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{exe}',f'/MAP:{mapping}',str(obj),str(a.kernel32_lib)]]
    for command in commands:
        result=subprocess.run(command,capture_output=True,text=True)
        assert result.returncode==0,result.stdout+'\n'+result.stderr
    manifest=dict(schema='bounded-original-optional-foot-construction-host-v1',host_executed=False,
        source_path=str(a.exe.resolve()),source_sha256=SOURCE_SHA256,code_allowlist=records,
        data_allowlist=[dict(address=f'{at:08X}',bytes=size,sha256=hashlib.sha256(image.read(at,size)).hexdigest()) for at,size in CONSTANTS],
        host_source_sha256=hashlib.sha256(asm.read_bytes()).hexdigest(),host_exe_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),
        linked_audit=linked_audit(exe,mapping),compile_commands=commands,
        input_bytes=INPUT_BYTES,record_bytes=RECORD_BYTES,projection_bytes=PROJECTION_BYTES,output_bytes=OUTPUT_BYTES,
        field_offsets=[f'{v:03X}' for v in FIELDS],field_record_offsets={f'{v:03X}':i for v,i in FIELD_INDEX.items()},
        input='CW:u16@0,reserved:u16@2,count:u32@4,finiteThresholdBits@8,initial3F5bool@12;255fixedrecords@16.Each record contains native168-byte capability projection at0 plus explicitfield-offsetDWORDs at168. Unused records zero.',
        output='count@0,appendedBlocks/nonzeroFootCount@4,final3F5bool@8,appendBatches@12;255zero-paddedrecords@16 in identical record layout. MXCSR andCW/SW follow projection.',
        entry_proof='All ten complete original spans unchanged, direct calls/branches closed over instruction boundaries or three declared providers.9CBE70 receives one owned definition and returns through an owned literal return address; exact nativeESP checked.',
        domain_proof='Source-prefix ordinal/palette identity, unique root0, bounded parent/symmetry, acyclic parent chains, exact childcounts, nonzeroFootCount0, finite consumed geometry, positive extents, quaternionsabs<=2, other consumed coordinatesabs<=2^100. Unconsumed anchor/scale/field words are opaque bit-for-bit inputs, including nonfinites. Eligible leaf limbs must have !(presence&60). count+2*initialCandidates<=255. Threshold finite; original overflow/unordered minima/threshold comparisons are preserved.',
        progress_proof='Every selected eligible leaf has no mouth/grasp, so9C8660 returns its own1F8 ordinal. Originalappend increments that parent208 at9CBC73. It therefore ceases to be a leaf. New blocks have foot-only capabilities and cannot enter the limb minimum. Thus append batches<=initialCandidates; mirrored insertion adds at most2blocks. GrowFixed independently enforces these limits.',
        provider_proof='GrowFixed checks9CBB09,exact owned vector/capacity,newcount difference1or2 and progress bound; it changes only end pointer. Originalappend immediately runs original9B2F90 before any new-slot read. DebugName checks the two exact original returns,args,format pointers,ordinal and104-byte destination, writes only excluded nameNUL; debug payload is outside projection and not consumed. ZeroConstructorStack checks9B305C,sizeA8,value0 and bounded owned-frame destination, providing exact memset semantics.',
        memory_proof='Input/definition/255blocks/frame/reply each have16-byte canaries checked before serialization. Full input immutable. Full source-prefix bytes immutable except native208. Unused post-result blocks remainA7. Definition shadow excludes only388end,3C8count,3F5bool. FrameESP+800 with>=1536byte lower reserve beyond memset lower bound; constructor deepest finite stack below512bytes. Source RX,constants/threshold R; unlisted sourcepage bytes INT3. No caller pointer enters the protocol.',
        arithmetic_proof='Explicit12maskedx87CWs andMXCSR1F80, exactunchangedCW,emptyx87TOP andunchangedSSEcontrol checked. x87/SSEstickyflags retained as context metadata, not claimed propagated by Rust. Sourceq<=2,coordinate<=2^100 bounds rotation/append anchor terms below binary32 overflow; extent products and threshold may overflow and keep native unordered branches. Zero/all-one XMM scratch runs must match every projectionbyte.',
        limits='Projected optional construction fields only. Debug strings, unrelated constructor padding, allocator behavior and full finalized body/animation are not claimed. Source32-slot capability input is supplied explicitly from separately accepted native projection.',
        pages=[dict(address=f'{page:08X}',bytes=65536,initial='RW',final='RX' if page in code_pages else 'R') for page in sorted(code_pages|data_pages)],
        execution_scope='Preparation only. No original modules/game/GUI/Cargo/network. Root and independent review required before hidden execution of pinned host/cohort.')
    (a.out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({k:manifest[k] for k in ['schema','host_executed','record_bytes','projection_bytes','host_source_sha256','host_exe_sha256']},indent=2))


if __name__=='__main__':main()

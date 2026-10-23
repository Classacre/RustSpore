"""Prepare, never execute, three bounded original finalizer field operations.

Reroot and local-orientation spans are declared instruction boundaries inside
9C3150, not complete functions.9B5DA0 is complete. No allocator, CRT, replacement
math, caller pointer, original module, game or GUI is used by the owned host.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

from anim_body_optional_native_probe import FIELDS as OPTIONAL_FIELDS
from cell_schema_probe import SourceImage, SOURCE_SHA256
from cell_native_math_probe import db
from native_crt_acos_probe import PE, decoder, IMPORTS

ROOT = Path(__file__).resolve().parent.parent
CODE = [(0x9B5DA0,0x9B5FF2),(0x9C3D80,0x9C407F),(0x9C4463,0x9C4598)]
EXITS = {0x9C407F:'OriginalRerootDone',0x9C4598:'OriginalLocalDone'}
FIELDS = OPTIONAL_FIELDS+[0x204,0x20C]
FIELD_INDEX = {at:168+4*i for i,at in enumerate(FIELDS)}
RECORD_BYTES = 168+4*len(FIELDS)
INPUT_BYTES = 32+255*RECORD_BYTES
PROJECTION_BYTES = INPUT_BYTES
OUTPUT_BYTES = PROJECTION_BYTES+8
BLOCK_BYTES = 255*0x468
FRAME_BYTES = 0x3000
BUFFERS = [('Input',INPUT_BYTES),('Definition',1024),('Blocks',BLOCK_BYTES),
    ('Frame',FRAME_BYTES),('Reply',OUTPUT_BYTES)]

def sha(raw): return hashlib.sha256(raw).hexdigest()

def audit_source(image):
    sys.path.insert(0,str(ROOT/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL, CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM
    result=[]
    for start,end in CODE:
        raw=image.read(start,end-start)
        instructions=list(decoder().disasm(raw,start))
        boundaries={i.address for i in instructions}
        assert sum(i.size for i in instructions)==end-start
        transfers=[]
        for ins in instructions:
            assert not ins.group(CS_GRP_CALL)
            if ins.group(CS_GRP_JUMP):
                assert ins.operands[0].type==X86_OP_IMM
                target=ins.operands[0].imm
                assert target in boundaries,(hex(ins.address),hex(target))
                transfers.append(dict(at=f'{ins.address:08X}',operation=ins.mnemonic,target=f'{target:08X}'))
            for op in ins.operands:
                if op.type==X86_OP_MEM:
                    assert op.mem.base or op.mem.index,'No source absolute data permitted'
        result.append(dict(start=f'{start:08X}',end_exclusive=f'{end:08X}',bytes=len(raw),
            instructions=len(instructions),sha256=sha(raw),calls=0,absolute_data_dependencies=0,
            transfers=transfers,disassembly=[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in instructions]))
    return result

def host_source(image):
    lines=['.686p','.xmm','.model flat','option casemap:none']
    for name,size in [('GetStdHandle',4),('ReadFile',20),('WriteFile',20),('VirtualAlloc',16),
        ('VirtualProtect',16),('GetLastError',0),('FlushInstructionCache',12),('ExitProcess',4)]:
        lines += [f'EXTERN _{name}@{size}:PROC']
    lines += ['.const']
    for start,end in CODE: lines += db(f'Original_{start:08X}',image.read(start,end-start))
    lines += ['.data']
    for label,size in BUFFERS:
        lines += [f'{label}Before DWORD 4 DUP(0A5A5A5A5h)',f'{label}Buffer BYTE {size} DUP(0)',
            f'{label}After DWORD 4 DUP(0A5A5A5A5h)']
    lines += [f'InputShadow BYTE {INPUT_BYTES} DUP(0)',f'BlocksShadow BYTE {BLOCK_BYTES} DUP(0)',
        'DefinitionShadow BYTE 1024 DUP(0)',f'Expected BYTE {PROJECTION_BYTES} DUP(0)',
        'InputHandle DWORD 0','OutputHandle DWORD 0','Transferred DWORD 0','OldProtect DWORD 0',
        'NativeStack DWORD 0','Poison DWORD 0','CurrentIndex DWORD 0','CurrentWalk DWORD 0',
        'CalculatedPresence DWORD 0','CalculatedNonzero DWORD 0','SseControl DWORD 01F80h',
        '.code','PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,RunOnce,ValidateRecord,ReadRecord',
        'PUBLIC OriginalRerootDone,OriginalLocalDone,OriginalRelabelDone,OriginalDone',
        'PUBLIC InputBuffer,DefinitionBuffer,BlocksBuffer,FrameBuffer,ReplyBuffer','OwnedStart LABEL BYTE',
        'MaskId PROC']
    for value,bit in [(0x746F6F72,1),(0x626D696C,2),(0x746F6F66,4),(0x6E697073,8),
        (0x74667370,16),(0x74756F6D,32),(0x70737267,64),(0x7274736E,128)]:
        lines += [f'cmp eax,0{value:08X}h',f'jne MaskNext{bit}',f'mov eax,{bit}','ret',f'MaskNext{bit}:']
    lines += ['xor eax,eax','ret','MaskId ENDP',
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
        'cmp word ptr [InputBuffer+2],0','jne HostFailure','cmp dword ptr [InputBuffer+4],2','ja HostFailure',
        'mov ecx,dword ptr [InputBuffer+8]','test ecx,ecx','jz HostFailure','cmp ecx,255','ja HostFailure',
        'cmp dword ptr [InputBuffer+28],0','jne HostFailure',
        'cmp dword ptr [InputBuffer+4],0','jne NotRerootHeader',
        'cmp dword ptr [InputBuffer+12],ecx','jae HostFailure',
        'cmp dword ptr [InputBuffer+16],0','jne HostFailure','cmp dword ptr [InputBuffer+20],0','jne HostFailure',
        'cmp dword ptr [InputBuffer+24],0','jne HostFailure','jmp HeaderReady',
        'NotRerootHeader:','cmp dword ptr [InputBuffer+12],0','jne HostFailure',
        'cmp dword ptr [InputBuffer+4],2','je RelabelHeader',
        'cmp dword ptr [InputBuffer+16],0','jne HostFailure','cmp dword ptr [InputBuffer+20],0','jne HostFailure',
        'cmp dword ptr [InputBuffer+24],0','jne HostFailure','jmp HeaderReady',
        'RelabelHeader:','cmp dword ptr [InputBuffer+16],ecx','jae HostFailure',
        'cmp dword ptr [InputBuffer+20],ecx','jae HostFailure','cmp dword ptr [InputBuffer+24],1','ja HostFailure',
        'HeaderReady:','xor ebx,ebx','mov esi,OFFSET InputBuffer+32','ValidateBlocks:',
        f'cmp dword ptr [esi+{FIELD_INDEX[0x1F8]}],ebx','jne HostFailure',
        f'mov eax,dword ptr [esi+{FIELD_INDEX[0x1FC]}]','cmp eax,-1','jne ParentValue',
        'test ebx,ebx','jnz HostFailure','jmp ParentReady','ParentValue:',
        'test ebx,ebx','jz HostFailure','cmp eax,dword ptr [InputBuffer+8]','jae HostFailure','ParentReady:',
        f'mov eax,dword ptr [esi+{FIELD_INDEX[0x200]}]','cmp eax,-1','je SymReady',
        'cmp eax,dword ptr [InputBuffer+8]','jae HostFailure','SymReady:',
        f'cmp dword ptr [esi+{FIELD_INDEX[0x208]}],255','ja HostFailure',
        'mov CalculatedPresence,0','mov CalculatedNonzero,0','xor edi,edi','ValidateMaskLoop:',
        'mov eax,dword ptr [esi+edi*4+8]','test eax,eax','jz ValidateMasksReady','call MaskId',
        'or CalculatedPresence,eax','cmp byte ptr [esi+edi+136],0','je ValidateMaskNext',
        'or CalculatedNonzero,eax','ValidateMaskNext:','inc edi','cmp edi,32','jb ValidateMaskLoop',
        'ValidateMasksReady:','mov eax,CalculatedPresence','cmp eax,dword ptr [esi]','jne HostFailure',
        'mov eax,CalculatedNonzero','cmp eax,dword ptr [esi+4]','jne HostFailure',
        'xor edx,edx','mov edi,OFFSET InputBuffer+32','xor eax,eax','CountChildLoop:',
        f'cmp dword ptr [edi+{FIELD_INDEX[0x1FC]}],ebx','jne CountChildNext','inc eax','CountChildNext:',
        f'add edi,{RECORD_BYTES}','inc edx','cmp edx,dword ptr [InputBuffer+8]','jb CountChildLoop',
        f'cmp eax,dword ptr [esi+{FIELD_INDEX[0x208]}]','jne HostFailure',
        'mov eax,ebx','xor edx,edx','ValidateParentChain:','cmp eax,-1','je ValidParentChain',
        'cmp edx,dword ptr [InputBuffer+8]','jae HostFailure',f'imul eax,eax,{RECORD_BYTES}',
        f'mov eax,dword ptr [eax+InputBuffer+32+{FIELD_INDEX[0x1FC]}]','inc edx','jmp ValidateParentChain',
        'ValidParentChain:','cmp dword ptr [InputBuffer+4],1','jne NotLocalValidation',
        'test ebx,ebx','jz LocalValuesReady']
    for at in range(0x118,0x128,4):
        lines += [f'mov eax,dword ptr [esi+{FIELD_INDEX[at]}]','and eax,07FFFFFFFh','cmp eax,040000000h','ja HostFailure']
    lines += [f'mov eax,dword ptr [esi+{FIELD_INDEX[0x1FC]}]',f'imul eax,eax,{RECORD_BYTES}',
        'add eax,OFFSET InputBuffer+32']
    for at in range(0x118,0x128,4):
        lines += [f'mov edx,dword ptr [eax+{FIELD_INDEX[at]}]','and edx,07FFFFFFFh','cmp edx,040000000h','ja HostFailure']
    lines += ['LocalValuesReady:','NotLocalValidation:','cmp dword ptr [InputBuffer+4],2','jne RangeValuesReady',
        f'mov edi,dword ptr [esi+{FIELD_INDEX[0x204]}]','cmp edi,dword ptr [InputBuffer+8]','ja HostFailure',
        f'mov edx,dword ptr [esi+{FIELD_INDEX[0x208]}]','add edx,edi',
        'cmp edx,dword ptr [InputBuffer+8]','ja HostFailure','RangeChildren:',
        'cmp edi,edx','je RangeValuesReady',f'imul eax,edi,{RECORD_BYTES}',
        f'cmp dword ptr [eax+InputBuffer+32+{FIELD_INDEX[0x1FC]}],ebx','jne HostFailure',
        'inc edi','jmp RangeChildren','RangeValuesReady:','inc ebx',f'add esi,{RECORD_BYTES}',
        'cmp ebx,dword ptr [InputBuffer+8]','jb ValidateBlocks',
        'mov edi,esi',f'mov ecx,{INPUT_BYTES}','add ecx,OFFSET InputBuffer','sub ecx,edi','xor eax,eax',
        'cld','repe scasb','jne HostFailure',
        'cmp dword ptr [InputBuffer+4],0','jne Validated',
        'test byte ptr [InputBuffer+36],8','jz HostFailure',
        'mov ebx,dword ptr [InputBuffer+12]',f'imul eax,ebx,{RECORD_BYTES}',
        'test byte ptr [eax+InputBuffer+36],8','jz HostFailure',
        'test ebx,ebx','jz Validated','ValidateReversedChain:',f'imul esi,ebx,{RECORD_BYTES}',
        'add esi,OFFSET InputBuffer+32']
    for at in range(0x344,0x35C,4):
        lines += [f'mov eax,dword ptr [esi+{FIELD_INDEX[at]}]','and eax,07FFFFFFFh','cmp eax,071800000h','ja HostFailure']
    lines += ['test byte ptr [esi],8','jz ReverseOriginReady']
    for at in range(0x144,0x150,4):
        lines += [f'mov eax,dword ptr [esi+{FIELD_INDEX[at]}]','and eax,07FFFFFFFh','cmp eax,071800000h','ja HostFailure']
    lines += ['ReverseOriginReady:',f'mov ebx,dword ptr [esi+{FIELD_INDEX[0x1FC]}]',
        'cmp ebx,-1','jne ValidateReversedChain','Validated:','ret','ValidateRecord ENDP',
        'RunOnce PROC','mov NativeStack,esp','cld','mov edi,OFFSET FrameBuffer','xor eax,eax',
        f'mov ecx,{FRAME_BYTES//4}','rep stosd','mov edi,OFFSET DefinitionBuffer','mov ecx,256','rep stosd',
        'mov edi,OFFSET BlocksBuffer','mov eax,0A7A7A7A7h',f'mov ecx,{BLOCK_BYTES//4}','rep stosd',
        'mov dword ptr [DefinitionBuffer+0384h],OFFSET BlocksBuffer','mov eax,dword ptr [InputBuffer+8]',
        'imul eax,eax,0468h','add eax,OFFSET BlocksBuffer','mov dword ptr [DefinitionBuffer+0388h],eax',
        'xor ebx,ebx','mov esi,OFFSET InputBuffer+32','mov edi,OFFSET BlocksBuffer','UnpackBlocks:',
        'push esi','push edi','add edi,0150h','mov ecx,42','rep movsd','pop edi','pop esi']
    for at in FIELDS:
        lines += [f'mov eax,dword ptr [esi+{FIELD_INDEX[at]}]',f'mov dword ptr [edi+0{at:X}h],eax']
    lines += [f'add esi,{RECORD_BYTES}','add edi,0468h','inc ebx','cmp ebx,dword ptr [InputBuffer+8]','jb UnpackBlocks',
        'mov esi,OFFSET InputBuffer','mov edi,OFFSET InputShadow',f'mov ecx,{INPUT_BYTES}','rep movsb',
        'mov esi,OFFSET BlocksBuffer','mov edi,OFFSET BlocksShadow',f'mov ecx,{BLOCK_BYTES}','rep movsb',
        'mov esi,OFFSET DefinitionBuffer','mov edi,OFFSET DefinitionShadow','mov ecx,1024','rep movsb',
        'fninit','fldcw word ptr [InputBuffer]','ldmxcsr SseControl','movd xmm0,Poison','pshufd xmm0,xmm0,0']
    for i in range(1,8): lines += [f'movaps xmm{i},xmm0']
    lines += ['cmp dword ptr [InputBuffer+4],0','je StartReroot',
        'cmp dword ptr [InputBuffer+4],1','je StartLocal',
        'cmp dword ptr [InputBuffer+24],0','jne OriginalDone','mov CurrentWalk,0','jmp StartRelabel',
        'StartReroot:','mov ebp,OFFSET DefinitionBuffer','xor ebx,ebx','mov esi,dword ptr [InputBuffer+12]',
        'imul ecx,esi,0468h','add ecx,OFFSET BlocksBuffer','lea edi,[ecx+0158h]','mov edx,edi','xor eax,eax',
        'mov esp,OFFSET FrameBuffer+0800h','mov dword ptr [esp+10h],esi','mov dword ptr [esp+1544h],0',
        'mov dword ptr [esp+20h],0','push 009C3D80h','ret',
        'OriginalRerootDone::','cmp esp,OFFSET FrameBuffer+0800h','jne HostFailure','jmp OriginalDone',
        'StartLocal:','mov CurrentIndex,1','LocalNext:','mov esi,CurrentIndex','cmp esi,dword ptr [InputBuffer+8]',
        'jae OriginalDone','imul esi,esi,0468h','add esi,OFFSET BlocksBuffer',
        'mov eax,dword ptr [esi+01FCh]','imul eax,eax,0468h','add esi,0120h',
        'mov esp,OFFSET FrameBuffer+0800h','mov dword ptr [esp+70h],OFFSET BlocksBuffer','push 009C4463h','ret',
        'OriginalLocalDone::','cmp esp,OFFSET FrameBuffer+0800h','jne HostFailure','inc CurrentIndex','jmp LocalNext',
        'StartRelabel:','mov ecx,CurrentWalk','mov eax,dword ptr [ecx*4+InputBuffer+16]',
        'mov esp,OFFSET FrameBuffer+0800h','push OFFSET DefinitionBuffer','push OFFSET OriginalRelabelDone',
        'push 009B5DA0h','ret','OriginalRelabelDone::','cmp esp,OFFSET FrameBuffer+07FCh','jne HostFailure',
        'inc CurrentWalk','cmp CurrentWalk,2','jb StartRelabel','OriginalDone::',
        'mov esp,NativeStack','pushfd','pop eax','test eax,0400h','jnz HostFailure','cld',
        f'fnstcw word ptr [ReplyBuffer+{PROJECTION_BYTES}]',f'fnstsw word ptr [ReplyBuffer+{PROJECTION_BYTES+2}]',
        f'stmxcsr dword ptr [ReplyBuffer+{PROJECTION_BYTES+4}]',
        f'mov ax,word ptr [ReplyBuffer+{PROJECTION_BYTES}]','cmp ax,word ptr [InputBuffer]','jne HostFailure',
        f'test word ptr [ReplyBuffer+{PROJECTION_BYTES+2}],03800h','jnz HostFailure',
        f'mov eax,dword ptr [ReplyBuffer+{PROJECTION_BYTES+4}]','and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure',
        'mov esi,OFFSET InputBuffer','mov edi,OFFSET InputShadow',f'mov ecx,{INPUT_BYTES}','repe cmpsb','jne HostFailure',
        'mov esi,OFFSET DefinitionBuffer','mov edi,OFFSET DefinitionShadow','mov ecx,1024','repe cmpsb','jne HostFailure',
        'xor ebx,ebx','mov esi,OFFSET BlocksBuffer','mov edi,OFFSET BlocksShadow','ShadowBlocks:',
        'xor edx,edx','ShadowBytes:','mov eax,dword ptr [InputBuffer+4]','cmp eax,1','je LocalShadowMask',
        'cmp edx,0150h','jb ShadowCompare','cmp edx,01F8h','jb ShadowAllowed',
        'cmp eax,2','je ShadowCompare','cmp edx,01FCh','jb ShadowCompare','cmp edx,0200h','jb ShadowAllowed',
        'cmp edx,0208h','jb ShadowCompare','cmp edx,020Ch','jb ShadowAllowed',
        'cmp edx,0344h','jb ShadowCompare','cmp edx,036Ch','jb ShadowAllowed','jmp ShadowCompare',
        'LocalShadowMask:','cmp edx,0128h','jb ShadowCompare','cmp edx,0138h','jb ShadowAllowed',
        'ShadowCompare:','mov al,byte ptr [esi+edx]','cmp al,byte ptr [edi+edx]','jne HostFailure',
        'ShadowAllowed:','inc edx','cmp edx,0468h','jb ShadowBytes','add esi,0468h','add edi,0468h',
        'inc ebx','cmp ebx,255','jb ShadowBlocks']
    for label,size in BUFFERS:
        for guard in ['Before','After']:
            for at in range(0,16,4):
                lines += [f'cmp dword ptr [{label}{guard}+{at}],0A5A5A5A5h','jne HostFailure']
    lines += ['mov edi,OFFSET ReplyBuffer','xor eax,eax',f'mov ecx,{PROJECTION_BYTES//4}','rep stosd',
        'mov esi,OFFSET InputBuffer','mov edi,OFFSET ReplyBuffer','mov ecx,8','rep movsd',
        'xor ebx,ebx','mov esi,OFFSET BlocksBuffer','mov edi,OFFSET ReplyBuffer+32','PackBlocks:',
        'push esi','push edi','add esi,0150h','mov ecx,42','rep movsd','pop edi','pop esi']
    for at in FIELDS:
        lines += [f'mov eax,dword ptr [esi+0{at:X}h]',f'mov dword ptr [edi+{FIELD_INDEX[at]}],eax']
    lines += ['add esi,0468h',f'add edi,{RECORD_BYTES}','inc ebx','cmp ebx,dword ptr [InputBuffer+8]',
        'jb PackBlocks','ret','RunOnce ENDP',
        'HostMain PROC','cld','push -10','call _GetStdHandle@4','mov InputHandle,eax',
        'push -11','call _GetStdHandle@4','mov OutputHandle,eax']
    pages=sorted({at&~0xFFFF for start,end in CODE for at in [start,end-1]})
    for page in pages:
        lines += ['push 4','push 03000h','push 010000h',f'push 0{page:08X}h','call _VirtualAlloc@16',
            f'cmp eax,0{page:08X}h','jne AllocationFailure','mov edi,eax','mov eax,0CCCCCCCCh',
            'mov ecx,04000h','rep stosd']
    for start,end in CODE:
        lines += [f'mov esi,OFFSET Original_{start:08X}',f'mov edi,0{start:08X}h',f'mov ecx,{end-start}','rep movsb']
    for at,label in EXITS.items():
        lines += [f'mov byte ptr [0{at:08X}h],0B8h',f'mov eax,OFFSET {label}',
            f'mov dword ptr [0{at+1:08X}h],eax',f'mov word ptr [0{at+5:08X}h],0E0FFh']
    for page in pages:
        lines += ['push OFFSET OldProtect','push 020h','push 010000h',f'push 0{page:08X}h',
            'call _VirtualProtect@16','test eax,eax','jz HostFailure',
            'push 010000h',f'push 0{page:08X}h','push -1','call _FlushInstructionCache@12',
            'test eax,eax','jz HostFailure']
    lines += ['MainLoop:','call ReadRecord','test eax,eax','jz HostSuccess','call ValidateRecord',
        'mov Poison,0','call RunOnce','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected',
        f'mov ecx,{PROJECTION_BYTES}','rep movsb','mov Poison,0FFFFFFFFh','call RunOnce',
        'mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected',f'mov ecx,{PROJECTION_BYTES}',
        'repe cmpsb','jne HostFailure','mov esi,OFFSET ReplyBuffer',f'mov ebx,{OUTPUT_BYTES}',
        'WriteLoop:','mov Transferred,0','push 0','push OFFSET Transferred','push ebx','push esi',
        'push OutputHandle','call _WriteFile@20','test eax,eax','jz HostFailure','mov eax,Transferred',
        'test eax,eax','jz HostFailure','cmp eax,ebx','ja HostFailure','add esi,eax','sub ebx,eax',
        'jnz WriteLoop','jmp MainLoop','HostSuccess:','push 0','call _ExitProcess@4',
        'AllocationFailure:','push 10','call _ExitProcess@4','HostFailure::','push 7','call _ExitProcess@4',
        'HostMain ENDP','OwnedEnd LABEL BYTE','END HostMain']
    return '\n'.join(lines)+'\n', pages

def audit_linked(path, mapping, image):
    pe=PE(path)
    symbols={}
    for line in mapping.read_text().splitlines():
        match=re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]+\s+(\S+)\s+([0-9A-Fa-f]{8})\b',line)
        if match: symbols[match[1]]=int(match[2],16)
    assert pe.base==0x25000000
    start,end=symbols['OwnedStart'],symbols['OwnedEnd']
    instructions=list(decoder().disasm(pe.read(start,end-start),start))
    assert sum(i.size for i in instructions)==end-start
    boundaries={i.address for i in instructions}
    imports=pe.imports()
    assert len(imports)==8
    for _,(dll,name) in imports.items():
        assert dll.lower()=='kernel32.dll' and name in IMPORTS
    from capstone import CS_GRP_CALL,CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM
    for ins in instructions:
        if ins.group(CS_GRP_CALL) or ins.group(CS_GRP_JUMP):
            assert ins.operands[0].type==X86_OP_IMM
            target=ins.operands[0].imm
            if target not in boundaries:
                thunk=pe.read(target,6)
                assert ins.mnemonic=='call' and thunk[:2]==b'\xFF\x25'
                assert struct.unpack_from('<I',thunk,2)[0] in imports
    for a,b in CODE:
        assert pe.read(symbols[f'Original_{a:08X}'],b-a)==image.read(a,b-a)
    for label,size in BUFFERS:
        assert symbols[label+'Buffer']-symbols[label+'Before']==16
        assert symbols[label+'After']-symbols[label+'Buffer']==size
    pe_at=struct.unpack_from('<I',pe.data,0x3C)[0]
    section_count=struct.unpack_from('<H',pe.data,pe_at+6)[0]
    optional=struct.unpack_from('<H',pe.data,pe_at+20)[0]
    sections=[]
    for i in range(section_count):
        at=pe_at+24+optional+40*i
        size,rva=struct.unpack_from('<II',pe.data,at+8)
        sections.append((pe.base+rva,pe.base+rva+size,struct.unpack_from('<I',pe.data,at+36)[0]))
    def flags(at,size=1):
        return next(value for first,last,value in sections if first<=at and at+size<=last)&0xE0000000
    assert flags(start,end-start)==0x60000000
    for first,last in CODE:
        assert flags(symbols[f'Original_{first:08X}'],last-first)==0x40000000
    for i,(label,size) in enumerate(BUFFERS):
        address=symbols[label+'Buffer']
        assert flags(address-16,size+32)==0xC0000000
        for other,other_size in BUFFERS[i+1:]:
            second=symbols[other+'Buffer']
            assert address+size+16<=second-16 or second+other_size+16<=address-16
    return dict(host_exe_sha256=sha(pe.data),owned_code_sha256=sha(pe.read(start,end-start)),
        symbols={k:f'{v:08X}' for k,v in symbols.items() if not k.startswith('_')},
        imports=[dict(iat=f'{at:08X}',dll=dll,name=name) for at,(dll,name) in imports.items()],
        section_permissions_checked=True,buffers_nonoverlapping=True,
        disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in instructions])

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('exe',type=Path);p.add_argument('--sdk-bin-x86',type=Path,required=True)
    p.add_argument('--kernel32-lib',type=Path,required=True);p.add_argument('--out',type=Path,required=True)
    a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
    image=SourceImage(a.exe);source=audit_source(image);text,pages=host_source(image)
    a.out.mkdir(parents=True,exist_ok=False)
    asm,obj,exe,mapping=[a.out.resolve()/name for name in ['host-x86.asm','host-x86.obj','host-x86.exe','host-x86.map']]
    asm.write_text(text)
    commands=[[str(a.sdk_bin_x86/'ml.exe'),'/nologo','/c','/coff',f'/Fo{obj}',str(asm)],
        [str(a.sdk_bin_x86/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain',
            '/BASE:0x25000000','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{exe}',f'/MAP:{mapping}',str(obj),str(a.kernel32_lib)]]
    saved_commands=[]
    for command in commands:
        completed=subprocess.run(command,capture_output=True,text=True)
        saved_commands.append(dict(arguments=command,exit=completed.returncode,stdout=completed.stdout,stderr=completed.stderr))
        (a.out/'compile-status.json').write_text(json.dumps(saved_commands,indent=2)+'\n')
        assert completed.returncode==0,completed.stdout+'\n'+completed.stderr
    linked=audit_linked(exe,mapping,image)
    report=dict(schema='bounded-original-body-finalizer-fields-host-v1',host_executed=False,
        source_sha256=SOURCE_SHA256,code_allowlist=source,host_source_sha256=sha(asm.read_bytes()),
        host_exe_sha256=linked['host_exe_sha256'],preparation_tool_sha256=sha(Path(__file__).read_bytes()),
        linked_audit=linked,compile_commands=commands,input_bytes=INPUT_BYTES,record_bytes=RECORD_BYTES,
        projection_bytes=PROJECTION_BYTES,output_bytes=OUTPUT_BYTES,field_offsets=[f'{at:03X}' for at in FIELDS],
        input='CW:u16@0/reserved0:u16@2,mode:u32@4,count@8,selectedRoot@12,start0@16,start1@20,noFootBool@24,reserved0@28;255records@32. Each record is168-byte capabilities plus39 explicit field DWORDs; unused records zero.',
        output='Identical32-byte header and255 fixed zero-padded projected records. CW:u16,SW:u16,MXCSR:u32 follow projection.',
        entry_proof='Mode0 enters unchanged9C3D80 with EAX0,EBX0,ECXselectedBlock,EDX=EDI=selectedIDs,EBPownedDef,ESPframe800; stack10selected,20oldRootByteOffset0,1544oldRoot0. Mode1 enters9C4463 per nonroot with EAXparent*468,ESIblock+120,stack70ownedArray. Mode2 calls complete9B5DA0 twice with explicit sequential starts/ownedDef unless noFootBool1. Two declared owned seven-byte exit trampolines immediately follow the partial spans; no whole-finalizer execution is claimed.',
        bounds_proof='Count1..255,ordinalidentity,unique root0,all parents/symmetries bounded,acyclic,exact directchildcounts,masks equal native first-zero32-slot tables. Mode0initial/selectednonzeroSpin,consumed chain anchors/origins finiteabs<=2^100; root0 opaque anchors are unconsumed. Mode1only consumed absolute quaternions finiteabs<=2,rootlocal opaque. Mode2starts/count/ranges bounded and every childrange member has exact owner parent. Unused inputrecords zero.',
        memory_proof='No protocol pointer; fixed owned input/1024-byteDef/255*468blocks/3000hframe/reply,16-byte guards. FrameESP800,all partialspan stack accesses within0..1544 and completehelper depth16bytes. Input/Def shadows immutable; full-block shadow permits only mode0caps,parent1FC,count208,anchors344..368;mode1local128..134;mode2caps. Original code has no absolute memory operands/calls and all direct branches remain inside each original span. Only owned exit trampolines and exact original fragments are placed in RX pages9B0000/9C0000; gaps INT3.',
        control_proof='FNINIT,12maskedCWs,MXCSR1F80,DF0,exact returnESP,unchangedCW,emptyTOP,unchangedSSEcontrol and DF checked. Sticky flags retained as metadata only. Zero/all-one XMM scratch runs compare all projected bytes. Partial-function GPR preservation is not claimed.',
        scope='Projected source-bound finalizer fields only. Root selection/order/source intake are separate accepted operations; no rest-matrix execution, allocator, full finalizer/wrapper, selector classes, runtime channels, evaluated pose or game loop claimed.',
        pages=[dict(address=f'{at:08X}',bytes=65536,initial='RW',final='RX') for at in pages],
        review_required_before_execution=True)
    (a.out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({key:report[key] for key in ['schema','host_executed','input_bytes','output_bytes','host_source_sha256','host_exe_sha256']},indent=2))

if __name__=='__main__': main()

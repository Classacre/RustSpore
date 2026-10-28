"""Prepare/audit original finite first-duration clock spans; never execute.

Explicit partial-entry/exit contracts avoid the event reset and sampling calls.
Crossing and loop bodies are absent/INT3 and checked unreachable in the domain.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import sys
from cell_native_math_probe import db
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import PE,decoder

ROOT=Path(__file__).resolve().parent.parent
CODE=[(0x9A2BB0,0x9A2BCC),(0x9A1CDB,0x9A1CFF),(0x9A1D4E,0x9A1D57),(0x9A1D69,0x9A1DB2)]
EXITS={0x9A2BCC:'DeltaDone',0x9A1CFF:'HostFailure',0x9A1D57:'HostFailure',0x9A1DB2:'FrameDone'}
ONE=(0x14861D0,4)
INPUT_BYTES,OUTPUT_BYTES=64,28
BUFFERS=[('Input',64),('Clip',0xC8),('Resource',0x130),('Context',16),('Frame',0x1000),('Reply',28)]
IMPORTS={'GetStdHandle','ReadFile','WriteFile','VirtualAlloc','VirtualProtect','GetLastError','FlushInstructionCache','ExitProcess'}
def sha(raw):return hashlib.sha256(raw).hexdigest()

def audit_source(image):
    sys.path.insert(0,str(ROOT/'target/rea/capstone-runtime'))
    from capstone import CS_GRP_CALL,CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM,X86_OP_MEM
    decoded=[list(decoder().disasm(image.read(a,b-a),a))for a,b in CODE]
    boundaries={i.address for rows in decoded for i in rows}
    result=[]
    for (a,b),rows in zip(CODE,decoded):
        assert sum(i.size for i in rows)==b-a
        transfers=[]
        for i in rows:
            assert not i.group(CS_GRP_CALL)
            if i.group(CS_GRP_JUMP):
                assert i.operands[0].type==X86_OP_IMM and i.operands[0].imm in boundaries|set(EXITS)
                transfers.append(dict(at=f'{i.address:08X}',target=f'{i.operands[0].imm:08X}'))
            for op in i.operands:
                if op.type==X86_OP_MEM and not op.mem.base and not op.mem.index:assert(op.mem.disp,op.size)==ONE
        result.append(dict(start=f'{a:08X}',end_exclusive=f'{b:08X}',bytes=b-a,sha256=sha(image.read(a,b-a)),
            transfers=transfers,disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in rows]))
    assert image.read(*ONE)==bytes.fromhex('0000803f')
    return result

def host_source(image):
    lines=['.686p','.xmm','.model flat','option casemap:none']
    for n,s in [('GetStdHandle',4),('ReadFile',20),('WriteFile',20),('VirtualAlloc',16),('VirtualProtect',16),('GetLastError',0),('FlushInstructionCache',12),('ExitProcess',4)]:lines.append(f'EXTERN _{n}@{s}:PROC')
    lines+=['.const']
    for a,b in CODE:lines += [f'PUBLIC Original_{a:08X}']+db(f'Original_{a:08X}',image.read(a,b-a))
    lines+=['PUBLIC OriginalOne']+db('OriginalOne',image.read(*ONE))+['.data']
    for n,s in BUFFERS:
        lines += [f'PUBLIC {n}Buffer,{n}Before,{n}After',f'{n}Before DWORD 4 DUP(0A5A5A5A5h)',f'{n}Buffer BYTE {s} DUP(0)',f'{n}After DWORD 4 DUP(0A5A5A5A5h)']
        if n!='Reply':lines += [f'{n}Shadow BYTE {s} DUP(0)']
    lines+=['InputHandle DWORD 0','OutputHandle DWORD 0','Transferred DWORD 0','OldProtect DWORD 0','NativeStack DWORD 0',
        'NextTime DWORD 0','Poison DWORD 0','Expected BYTE 20 DUP(0)','SseControl DWORD 01F80h','.code',
        'PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,DeltaDone,FrameDone','OwnedStart LABEL BYTE',
        'ReadRecord PROC','push ebx','push esi','mov ebx,64','mov esi,OFFSET InputBuffer','ReadLoop:',
        'mov Transferred,0','push 0','push OFFSET Transferred','push ebx','push esi','push InputHandle','call _ReadFile@20',
        'test eax,eax','jnz ReadTransferred','call _GetLastError@0','cmp eax,109','jne HostFailure',
        'cmp ebx,64','jne HostFailure','cmp Transferred,0','jne HostFailure','xor eax,eax','jmp ReadDone','ReadTransferred:',
        'mov eax,Transferred','cmp eax,ebx','ja HostFailure','test eax,eax','jnz ReadSome','cmp ebx,64','jne HostFailure',
        'xor eax,eax','jmp ReadDone','ReadSome:','add esi,eax','sub ebx,eax','jnz ReadLoop','mov eax,1',
        'ReadDone:','pop esi','pop ebx','ret','ReadRecord ENDP',
        'ValidateRecord PROC','cmp dword ptr [InputBuffer],1','ja HostFailure','movzx eax,word ptr [InputBuffer+4]',
        'mov edx,eax','and eax,0F0FFh','cmp eax,07Fh','jne HostFailure','and edx,0300h','cmp edx,0100h','je HostFailure',
        'cmp word ptr [InputBuffer+6],0','jne HostFailure','cmp dword ptr [InputBuffer+32],0','jne HostFailure',
        'mov esi,OFFSET InputBuffer+36','mov ecx,7','ValidateReserved:','cmp dword ptr [esi],0','jne HostFailure',
        'add esi,4','loop ValidateReserved','mov eax,dword ptr [InputBuffer+8]','and eax,07F800000h',
        'cmp eax,07F800000h','je HostFailure','mov eax,dword ptr [InputBuffer+24]','test eax,080000000h','jnz HostFailure',
        'mov edx,eax','and edx,07FF00000h','cmp edx,07FF00000h','je HostFailure','or eax,dword ptr [InputBuffer+20]',
        'jz HostFailure','mov eax,dword ptr [InputBuffer+28]','cmp eax,03F800000h','jb HostFailure',
        'cmp eax,07F800000h','jae HostFailure','cmp dword ptr [InputBuffer],0','je Validated',
        'mov eax,dword ptr [InputBuffer+16]','and eax,07FF00000h','cmp eax,07FF00000h','je HostFailure',
        'Validated:','ret','ValidateRecord ENDP','RunOnce PROC','mov NativeStack,esp','cld']
    for n,s in BUFFERS:
        if n!='Input':lines += [f'mov edi,OFFSET {n}Buffer',f'mov ecx,{s//4}','mov eax,0A5A5A5A5h','rep stosd']
    lines+=['mov dword ptr [ClipBuffer],OFFSET ResourceBuffer','mov dword ptr [ClipBuffer+8],OFFSET ContextBuffer',
        'mov byte ptr [ClipBuffer+0ADh],0','mov eax,dword ptr [InputBuffer+12]','mov dword ptr [ClipBuffer+0B0h],eax',
        'mov eax,dword ptr [InputBuffer+16]','mov dword ptr [ClipBuffer+0B4h],eax',
        'mov eax,dword ptr [InputBuffer+20]','mov dword ptr [ClipBuffer+0B8h],eax',
        'mov eax,dword ptr [InputBuffer+24]','mov dword ptr [ClipBuffer+0BCh],eax',
        'mov eax,dword ptr [InputBuffer+28]','mov dword ptr [ResourceBuffer+0120h],eax',
        'mov edi,OFFSET ContextBuffer','xor eax,eax','mov ecx,4','rep stosd',
        'mov eax,dword ptr [InputBuffer+8]','mov NextTime,eax',
        'mov dword ptr [FrameBuffer+0804h],OFFSET ClipBuffer','mov dword ptr [FrameBuffer+0808h],eax',
        'mov dword ptr [FrameBuffer+080Ch],0','mov dword ptr [FrameBuffer+0810h],0']
    for n,s in BUFFERS:
        if n!='Reply':lines += [f'mov esi,OFFSET {n}Buffer',f'mov edi,OFFSET {n}Shadow',f'mov ecx,{s//4}','rep movsd']
    lines += ['fninit','fldcw word ptr [InputBuffer+4]','ldmxcsr SseControl','movd xmm0,Poison','pshufd xmm0,xmm0,0']
    for i in range(1,8):lines += [f'movaps xmm{i},xmm0']
    lines += ['cmp dword ptr [InputBuffer],0','je PrepareClock','mov esp,OFFSET FrameBuffer+0800h','push 009A2BB0h','ret',
        'DeltaDone::','cmp esp,OFFSET FrameBuffer+07F4h','jne HostFailure','mov eax,dword ptr [esp]',
        'mov NextTime,eax','mov esp,NativeStack','xor ecx,ecx','CheckDeltaFrame:',
        'cmp ecx,07F4h','jb CompareDeltaFrame','cmp ecx,0800h','jb NextDeltaFrame','CompareDeltaFrame:',
        'mov eax,dword ptr [FrameBuffer+ecx]','cmp eax,dword ptr [FrameShadow+ecx]','jne HostFailure',
        'NextDeltaFrame:','add ecx,4','cmp ecx,01000h','jb CheckDeltaFrame','PrepareClock:',
        'mov eax,NextTime','mov edx,eax','and edx,07FFFFFFFh','cmp edx,07F800000h','jae HostFailure',
        'test eax,080000000h','jz TimeNonnegative','test edx,edx','jnz HostFailure','TimeNonnegative:',
        'mov edi,OFFSET FrameBuffer','mov ecx,0400h','mov eax,0A5A5A5A5h','rep stosd',
        'mov eax,NextTime','mov dword ptr [FrameBuffer+081Ch],eax',
        'mov dword ptr [FrameBuffer+0820h],0','mov dword ptr [FrameBuffer+0824h],0',
        'mov esi,OFFSET FrameBuffer','mov edi,OFFSET FrameShadow','mov ecx,0400h','rep movsd',
        'mov esi,OFFSET ClipBuffer','mov esp,OFFSET FrameBuffer+0800h','push 009A1CDBh','ret',
        'FrameDone::','cmp esp,OFFSET FrameBuffer+07E0h','jne HostFailure','mov esp,NativeStack',
        'pushfd','pop eax','test eax,0400h','jnz HostFailure','cld']
    for n,s in BUFFERS:
        if n not in {'Clip','Frame','Reply'}:lines += [f'mov esi,OFFSET {n}Buffer',f'mov edi,OFFSET {n}Shadow',f'mov ecx,{s//4}','repe cmpsd','jne HostFailure']
    lines += ['xor ecx,ecx','CheckClip:','cmp ecx,0B0h','je NextClip','cmp ecx,0B4h','je NextClip',
        'mov eax,dword ptr [ClipBuffer+ecx]','cmp eax,dword ptr [ClipShadow+ecx]','jne HostFailure',
        'NextClip:','add ecx,4','cmp ecx,0C8h','jb CheckClip','xor ecx,ecx','CheckFrame:',
        'cmp ecx,07E0h','jb CompareFrame','cmp ecx,0800h','jb NextFrame','cmp ecx,0804h','je NextFrame',
        'cmp ecx,0818h','je NextFrame','CompareFrame:','mov eax,dword ptr [FrameBuffer+ecx]',
        'cmp eax,dword ptr [FrameShadow+ecx]','jne HostFailure','NextFrame:','add ecx,4','cmp ecx,01000h','jb CheckFrame']
    for n,s in BUFFERS:
        for guard in ['Before','After']:
            for at in range(0,16,4):lines += [f'cmp dword ptr [{n}{guard}+{at}],0A5A5A5A5h','jne HostFailure']
    for at in range(0,28,4):lines += [f'cmp dword ptr [ReplyBuffer+{at}],0A5A5A5A5h','jne HostFailure']
    lines += ['fnstcw word ptr [ReplyBuffer+20]','fnstsw word ptr [ReplyBuffer+22]','stmxcsr dword ptr [ReplyBuffer+24]',
        'mov ax,word ptr [ReplyBuffer+20]','cmp ax,word ptr [InputBuffer+4]','jne HostFailure',
        'test word ptr [ReplyBuffer+22],03800h','jnz HostFailure','mov eax,dword ptr [ReplyBuffer+24]',
        'and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure','mov eax,NextTime','mov dword ptr [ReplyBuffer],eax',
        'mov eax,dword ptr [ClipBuffer+0B0h]','mov dword ptr [ReplyBuffer+4],eax',
        'mov eax,dword ptr [ClipBuffer+0B4h]','mov dword ptr [ReplyBuffer+8],eax',
        'mov eax,dword ptr [FrameBuffer+0818h]','mov dword ptr [ReplyBuffer+12],eax',
        'mov eax,dword ptr [FrameBuffer+07E4h]','mov dword ptr [ReplyBuffer+16],eax','ret','RunOnce ENDP',
        'HostMain PROC','cld','push -10','call _GetStdHandle@4','mov InputHandle,eax','push -11','call _GetStdHandle@4','mov OutputHandle,eax']
    for page in [0x9A0000,0x1480000]:
        lines += ['push 4','push 03000h','push 010000h',f'push 0{page:X}h','call _VirtualAlloc@16',f'cmp eax,0{page:X}h','jne AllocationFailure','mov edi,eax','mov ecx,04000h','mov eax,0CCCCCCCCh','rep stosd']
    for a,b in CODE:lines += [f'mov esi,OFFSET Original_{a:08X}',f'mov edi,0{a:X}h',f'mov ecx,{b-a}','rep movsb']
    lines += ['mov esi,OFFSET OriginalOne','mov edi,014861D0h','mov ecx,4','rep movsb']
    for at,label in EXITS.items():lines += [f'mov byte ptr [0{at:X}h],0B8h',f'mov eax,OFFSET {label}',f'mov dword ptr [0{at+1:X}h],eax',f'mov word ptr [0{at+5:X}h],0E0FFh']
    for page,flags in [(0x9A0000,0x20),(0x1480000,2)]:
        lines += ['push OFFSET OldProtect',f'push 0{flags:X}h','push 010000h',f'push 0{page:X}h','call _VirtualProtect@16','test eax,eax','jz HostFailure']
    lines += ['push 010000h','push 009A0000h','push -1','call _FlushInstructionCache@12','test eax,eax','jz HostFailure',
        'MainLoop:','call ReadRecord','test eax,eax','jz HostSuccess','call ValidateRecord','mov Poison,0','call RunOnce',
        'mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected','mov ecx,20','rep movsb','mov Poison,0FFFFFFFFh','call RunOnce',
        'mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected','mov ecx,20','repe cmpsb','jne HostFailure',
        'mov esi,OFFSET ReplyBuffer','mov ebx,28','WriteLoop:','mov Transferred,0','push 0','push OFFSET Transferred',
        'push ebx','push esi','push OutputHandle','call _WriteFile@20','test eax,eax','jz HostFailure',
        'mov eax,Transferred','test eax,eax','jz HostFailure','cmp eax,ebx','ja HostFailure','add esi,eax','sub ebx,eax','jnz WriteLoop','jmp MainLoop',
        'HostSuccess:','push 0','call _ExitProcess@4','AllocationFailure:','push 10','call _ExitProcess@4',
        'HostFailure::','push 7','call _ExitProcess@4','HostMain ENDP','OwnedEnd LABEL BYTE','END HostMain']
    return '\n'.join(lines)+'\n'

def linked_audit(path,mapping,image):
    pe=PE(path);assert pe.base==0x27000000
    symbols={}
    for line in mapping.read_text().splitlines():
        m=re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]+\s+(\S+)\s+([0-9A-Fa-f]{8})\b',line)
        if m:symbols[m[1]]=int(m[2],16)
    first,last=symbols['OwnedStart'],symbols['OwnedEnd'];rows=list(decoder().disasm(pe.read(first,last-first),first))
    assert sum(i.size for i in rows)==last-first
    imports=pe.imports();assert len(imports)==8 and {n for d,n in imports.values()if d.lower()=='kernel32.dll'}==IMPORTS
    from capstone import CS_GRP_CALL,CS_GRP_JUMP
    from capstone.x86 import X86_OP_IMM
    boundaries={i.address for i in rows}
    for i in rows:
        if i.group(CS_GRP_CALL)or i.group(CS_GRP_JUMP):
            assert i.operands[0].type==X86_OP_IMM
            target=i.operands[0].imm
            if target not in boundaries:
                raw=pe.read(target,6);assert i.mnemonic=='call'and raw[:2]==b'\xff\x25'and struct.unpack_from('<I',raw,2)[0]in imports
    at=struct.unpack_from('<I',pe.data,0x3C)[0];count=struct.unpack_from('<H',pe.data,at+6)[0];optional=struct.unpack_from('<H',pe.data,at+20)[0];sections=[]
    for index in range(count):
        row=at+24+optional+40*index;size,rva=struct.unpack_from('<II',pe.data,row+8);sections.append((pe.base+rva,pe.base+rva+size,struct.unpack_from('<I',pe.data,row+36)[0]))
    def flags(address,size):return next(f for a,b,f in sections if a<=address and address+size<=b)&0xE0000000
    assert flags(first,last-first)==0x60000000
    for a,b in CODE:
        address=symbols[f'Original_{a:08X}'];assert pe.read(address,b-a)==image.read(a,b-a)and flags(address,b-a)==0x40000000
    assert pe.read(symbols['OriginalOne'],4)==image.read(*ONE)and flags(symbols['OriginalOne'],4)==0x40000000
    extents=[]
    for n,size in BUFFERS:
        a=symbols[n+'Buffer'];assert a-symbols[n+'Before']==16 and symbols[n+'After']-a==size
        assert flags(a-16,size+32)==0xC0000000 and all(a+size+16<=lo or hi<=a-16 for lo,hi in extents);extents.append((a-16,a+size+16))
    return dict(host_exe_sha256=sha(pe.data),host_map_sha256=sha(mapping.read_bytes()),owned_code_sha256=sha(pe.read(first,last-first)),
        source_bytes_exact=True,sections_and_guards_checked=True,host_executed=False,
        imports=[dict(iat=f'{a:08X}',dll=d,name=n)for a,(d,n)in imports.items()],symbols={n:f'{a:08X}'for n,a in symbols.items()if not n.startswith('_')},
        disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in rows])

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True)
    p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    p.add_argument('--audit-exe',type=Path);p.add_argument('--audit-map',type=Path);args=p.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve());image=SourceImage(args.source_exe);source=audit_source(image);assembly=host_source(image)
    if args.audit_exe is not None:
        assert args.audit_map is not None;m=json.loads((args.out/'manifest.json').read_text());assert (args.out/'host-x86.asm').read_text()==assembly
        assert sha((args.out/'host-x86.asm').read_bytes())==m['host_source_sha256'];report=linked_audit(args.audit_exe,args.audit_map,image)
        with (args.out/'linked-audit.json').open('x')as f:json.dump(report,f,indent=2)
    else:
        args.out.mkdir(parents=True,exist_ok=False);(args.out/'host-x86.asm').write_text(assembly)
        report=dict(schema='bounded-original-cell-runtime-clock-host-v1',source_sha256=SOURCE_SHA256,code_allowlist=source,
            source_bytes=sum(b-a for a,b in CODE),constant=dict(address='014861D0',bytes=4,raw_hex='0000803f'),
            host_source_sha256=sha((args.out/'host-x86.asm').read_bytes()),preparation_tool_sha256=sha(Path(__file__).read_bytes()),
            compiled=False,host_executed=False,protocol=dict(input_bytes=64,projection_bytes=20,output_bytes=28,
                input='mode:u32@0(0abs/1delta),CW:u16@4,reserved0:u16@6,timeOrDelta32@8,previousTime64@12,duration64@20,resourceFrames32@28,loop0@32,reserved0@36..60',
                output='nextTime32@0,storedTime64@4,normalized32@12,frame32@16,CW:u16@20,SW:u16@22,MXCSR:u32@24'),
            domain='Finite delta/absolute and reached previousTime64. Positive finite duration64; frames finite>=1; committed time32 finite nonnegative including negativezero and strictly below duration64. Loop word zero. Twelve exact masked x87 CWs,MXCSR1F80. Other paths rejected before output; ignored previousTime64 in mode0 arbitrary.',
            source_contract='Delta source9A2BB0..2BCC starts with ESP=Frame800 and explicit four argument words; exits ESP7F4 after FSTP32. Clock source9A1CDB starts ESP800,ESI=Clip,emptyx87; time@ESP1C,flags@20/24 zero,Clip+8->Context16,Clip+0->Resource. Exact JCbypass proves no crossing; Clip+AD0 proves no loop. Source setup completes32bytes of sampling arguments and exits at9A1DB2,ESP7E0 before CALL9A1590. No replacement callback.',
            memory='Six separated16byte-guarded buffers. Input/Resource/Context whole shadows; Clip whole shadow exceptB0/B4 time store. Delta Frame whole shadow except7F4..800; clock Frame whole shadow except7E0..800,804ratio,818normalized. Reply poison and all guards checked before projection. ESP/DF/CW/TOP/MXCSRcontrols checked; zero/allones XMM projections agree.',
            exclusions='No9A1690 event reset, no99BCB0 fraction helper, no crossing counters/globals, no9A1590 sampler, no original process/GUI/callback. Source gaps are INT3; only four declared seven-byte absolute exits are written.',
            exits={f'{a:08X}':n for a,n in EXITS.items()},pages=[dict(address='009A0000',final='RX'),dict(address='01480000',final='R')],
            compile_commands=[['ml.exe','/nologo','/c','/coff','/Fohost-x86.obj','host-x86.asm'],['link.exe','/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/BASE:0x27000000','/DYNAMICBASE:NO','/SAFESEH:NO','/OUT:host-x86.exe','/MAP:host-x86.map','host-x86.obj','kernel32.lib']],root_review_required=True)
        (args.out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:report[k]for k in ['host_executed']},indent=2))
if __name__=='__main__':main()

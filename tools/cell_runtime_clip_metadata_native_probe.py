"""Prepare/audit two exact9A3140 metadata spans. Never execute."""
import argparse,hashlib,json,struct,sys
from pathlib import Path
import cell_runtime_clock_native_probe as base
from cell_schema_probe import SourceImage,SOURCE_SHA256
from cell_native_math_probe import db
ROOT=Path(__file__).resolve().parent.parent
CODE=[(0x9A3198,0x9A31C5),(0x9A32D4,0x9A32E7)]
EXITS={0x9A31C5:'DurationDone',0x9A32E7:'LoopDone'}
BUFFERS=[('Input',32),('Clip',0xC8),('Resource',0x130),('Frame',0x1000),('Reply',24)]
def sha(x):return hashlib.sha256(x).hexdigest()
def audit_source(image):
 base.CODE=CODE;base.EXITS=EXITS
 return base.audit_source(image)
def host_source(image):
 l=['.686p','.xmm','.model flat','option casemap:none']
 for n,s in [('GetStdHandle',4),('ReadFile',20),('WriteFile',20),('VirtualAlloc',16),('VirtualProtect',16),('GetLastError',0),('FlushInstructionCache',12),('ExitProcess',4)]:l+=[f'EXTERN _{n}@{s}:PROC']
 l+=['.const']
 for a,b in CODE:l+=[f'PUBLIC Original_{a:08X}']+db(f'Original_{a:08X}',image.read(a,b-a))
 l+=['PUBLIC OriginalOne']+db('OriginalOne',image.read(*base.ONE))+['.data']
 for n,s in BUFFERS:
  l +=[f'PUBLIC {n}Buffer,{n}Before,{n}After',f'{n}Before DWORD 4 DUP(0A5A5A5A5h)',f'{n}Buffer BYTE {s} DUP(0)',f'{n}After DWORD 4 DUP(0A5A5A5A5h)']
  if n!='Reply':l +=[f'{n}Shadow BYTE {s} DUP(0)']
 l+=['InputHandle DWORD 0','OutputHandle DWORD 0','Transferred DWORD 0','OldProtect DWORD 0','NativeStack DWORD 0','SseControl DWORD 01F80h','Poison DWORD 0','Expected BYTE 16 DUP(0)', '.code','PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,DurationDone,LoopDone','OwnedStart LABEL BYTE',
 'ReadRecord PROC','push ebx','push esi','mov ebx,32','mov esi,OFFSET InputBuffer','ReadLoop:','mov Transferred,0','push 0','push OFFSET Transferred','push ebx','push esi','push InputHandle','call _ReadFile@20','test eax,eax','jnz ReadTransferred','call _GetLastError@0','cmp eax,109','jne HostFailure','cmp ebx,32','jne HostFailure','cmp Transferred,0','jne HostFailure','xor eax,eax','jmp ReadDone','ReadTransferred:','mov eax,Transferred','cmp eax,ebx','ja HostFailure','test eax,eax','jnz ReadSome','cmp ebx,32','jne HostFailure','xor eax,eax','jmp ReadDone','ReadSome:','add esi,eax','sub ebx,eax','jnz ReadLoop','mov eax,1','ReadDone:','pop esi','pop ebx','ret','ReadRecord ENDP',
 'ValidateRecord PROC','movzx eax,word ptr [InputBuffer]','mov edx,eax','and eax,0F0FFh','cmp eax,07Fh','jne HostFailure','and edx,0300h','cmp edx,0100h','je HostFailure','cmp word ptr [InputBuffer+2],0','jne HostFailure','mov esi,OFFSET InputBuffer+16','mov ecx,4','Reserved:','cmp dword ptr [esi],0','jne HostFailure','add esi,4','loop Reserved','mov eax,dword ptr [InputBuffer+8]','cmp eax,03F800000h','jbe HostFailure','cmp eax,07F800000h','jae HostFailure','mov eax,dword ptr [InputBuffer+12]','test eax,eax','jz HostFailure','cmp eax,07F800000h','jae HostFailure','ret','ValidateRecord ENDP',
 'RunOnce PROC','mov NativeStack,esp','cld']
 for n,s in BUFFERS:
  if n!='Input':l +=[f'mov edi,OFFSET {n}Buffer',f'mov ecx,{s//4}','mov eax,0A5A5A5A5h','rep stosd']
 l+=['mov dword ptr [ClipBuffer],OFFSET ResourceBuffer','mov eax,dword ptr [InputBuffer+4]','mov dword ptr [ResourceBuffer+0118h],eax','mov eax,dword ptr [InputBuffer+8]','mov dword ptr [ResourceBuffer+0120h],eax','mov eax,dword ptr [InputBuffer+12]','mov dword ptr [ResourceBuffer+011Ch],eax']
 for n,s in BUFFERS:
  if n!='Reply':l +=[f'mov esi,OFFSET {n}Buffer',f'mov edi,OFFSET {n}Shadow',f'mov ecx,{s//4}','rep movsd']
 l+=['fninit','fldcw word ptr [InputBuffer]','ldmxcsr SseControl','movd xmm0,Poison','pshufd xmm0,xmm0,0','mov ebx,OFFSET ClipBuffer','mov eax,OFFSET ResourceBuffer','mov esp,OFFSET FrameBuffer+0800h','push 009A3198h','ret',
 'DurationDone::','cmp esp,OFFSET FrameBuffer+0800h','jne HostFailure','mov esp,NativeStack','mov edx,OFFSET ResourceBuffer','mov ebx,OFFSET ClipBuffer','push 009A32D4h','ret',
 'LoopDone::','cmp esp,NativeStack','jne HostFailure','pushfd','pop eax','test eax,0400h','jnz HostFailure','cld']
 for n,s in BUFFERS:
  if n in {'Input','Resource'}:l +=[f'mov esi,OFFSET {n}Buffer',f'mov edi,OFFSET {n}Shadow',f'mov ecx,{s//4}','repe cmpsd','jne HostFailure']
 l+=['xor ecx,ecx','CheckClip:','cmp ecx,0ADh','je NextClip','cmp ecx,0B8h','jb CompareClip','cmp ecx,0C0h','jb NextClip','CompareClip:','mov al,byte ptr [ClipBuffer+ecx]','cmp al,byte ptr [ClipShadow+ecx]','jne HostFailure','NextClip:','inc ecx','cmp ecx,0C8h','jb CheckClip','xor ecx,ecx','CheckFrame:','cmp ecx,07FCh','je NextFrame','cmp ecx,0820h','je NextFrame','mov eax,dword ptr [FrameBuffer+ecx]','cmp eax,dword ptr [FrameShadow+ecx]','jne HostFailure','NextFrame:','add ecx,4','cmp ecx,01000h','jb CheckFrame']
 for n,s in BUFFERS:
  for g in ['Before','After']:
   for a in range(0,16,4):l +=[f'cmp dword ptr [{n}{g}+{a}],0A5A5A5A5h','jne HostFailure']
 for a in range(0,24,4):l +=[f'cmp dword ptr [ReplyBuffer+{a}],0A5A5A5A5h','jne HostFailure']
 l+=['mov eax,dword ptr [FrameBuffer+0820h]','test eax,eax','jz HostFailure','cmp eax,07F800000h','jae HostFailure','mov dword ptr [ReplyBuffer],eax','mov eax,dword ptr [ClipBuffer+0B8h]','mov dword ptr [ReplyBuffer+4],eax','mov eax,dword ptr [ClipBuffer+0BCh]','mov dword ptr [ReplyBuffer+8],eax','movzx eax,byte ptr [ClipBuffer+0ADh]','mov dword ptr [ReplyBuffer+12],eax','fnstcw word ptr [ReplyBuffer+16]','fnstsw word ptr [ReplyBuffer+18]','stmxcsr dword ptr [ReplyBuffer+20]','mov ax,word ptr [ReplyBuffer+16]','cmp ax,word ptr [InputBuffer]','jne HostFailure','test word ptr [ReplyBuffer+18],03800h','jnz HostFailure','mov eax,dword ptr [ReplyBuffer+20]','and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure','ret','RunOnce ENDP',
 'HostMain PROC','cld','push -10','call _GetStdHandle@4','mov InputHandle,eax','push -11','call _GetStdHandle@4','mov OutputHandle,eax']
 for page in [0x9A0000,0x1480000]:
  l+=['push 4','push 03000h','push 010000h',f'push 0{page:X}h','call _VirtualAlloc@16',f'cmp eax,0{page:X}h','jne AllocationFailure','mov edi,eax','mov ecx,04000h','mov eax,0CCCCCCCCh','rep stosd']
 for a,b in CODE:l +=[f'mov esi,OFFSET Original_{a:08X}',f'mov edi,0{a:X}h',f'mov ecx,{b-a}','rep movsb']
 l+=['mov esi,OFFSET OriginalOne','mov edi,014861D0h','mov ecx,4','rep movsb']
 for a,n in EXITS.items():l +=[f'mov byte ptr [0{a:X}h],0B8h',f'mov eax,OFFSET {n}',f'mov dword ptr [0{a+1:X}h],eax',f'mov word ptr [0{a+5:X}h],0E0FFh']
 for page,flags in [(0x9A0000,0x20),(0x1480000,2)]:l+=['push OFFSET OldProtect',f'push 0{flags:X}h','push 010000h',f'push 0{page:X}h','call _VirtualProtect@16','test eax,eax','jz HostFailure']
 l+=['push 010000h','push 009A0000h','push -1','call _FlushInstructionCache@12','test eax,eax','jz HostFailure','MainLoop:','call ReadRecord','test eax,eax','jz HostSuccess','call ValidateRecord','mov Poison,0','call RunOnce','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected','mov ecx,16','rep movsb','mov Poison,0FFFFFFFFh','call RunOnce','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected','mov ecx,16','repe cmpsb','jne HostFailure','mov esi,OFFSET ReplyBuffer','mov ebx,24','WriteLoop:','mov Transferred,0','push 0','push OFFSET Transferred','push ebx','push esi','push OutputHandle','call _WriteFile@20','test eax,eax','jz HostFailure','mov eax,Transferred','test eax,eax','jz HostFailure','cmp eax,ebx','ja HostFailure','add esi,eax','sub ebx,eax','jnz WriteLoop','jmp MainLoop','HostSuccess:','push 0','call _ExitProcess@4','AllocationFailure:','push 10','call _ExitProcess@4','HostFailure::','cmp NativeStack,0','je FailureRealStack','mov esp,NativeStack','FailureRealStack:','push 7','call _ExitProcess@4','HostMain ENDP','OwnedEnd LABEL BYTE','END HostMain']
 return '\n'.join(l)+'\n'
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--audit-exe',type=Path);p.add_argument('--audit-map',type=Path);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());image=SourceImage(a.source_exe);source=audit_source(image);asm=host_source(image)
 if a.audit_exe:
  assert a.audit_map and (a.out/'host-x86.asm').read_text()==asm
  base.BUFFERS=BUFFERS;r=base.linked_audit(a.audit_exe,a.audit_map,image)
  pe=base.PE(a.audit_exe);off=struct.unpack_from('<I',pe.data,0x3c)[0];entry=pe.base+struct.unpack_from('<I',pe.data,off+24+16)[0];assert entry==int(r['symbols']['HostMain'],16);r['pe_entrypoint_hostmain']=True
  with(a.out/'linked-audit.json').open('x')as f:json.dump(r,f,indent=2)
 else:
  a.out.mkdir(parents=True,exist_ok=False);(a.out/'host-x86.asm').write_text(asm)
  r=dict(schema='source-cell-clip-metadata-host-v1',source_sha256=SOURCE_SHA256,code_allowlist=source,source_bytes=sum(b-a for a,b in CODE),host_source_sha256=sha((a.out/'host-x86.asm').read_bytes()),preparation_tool_sha256=sha(Path(__file__).read_bytes()),base_auditor_sha256=sha(Path(base.__file__).read_bytes()),compiled=False,host_executed=False,input_bytes=32,projection_bytes=16,output_bytes=24,input='CW:u16@0,reserved0:u16@2,resourceFlags32@4,frames32@8,frameInterval32@12,reserved0@16..28',output='durationCommit32@0,duration64@4,loop32@12,CW:u16@16,SW:u16@18,MXCSR32@20',domain='Twelve masked CWs; MXCSR1F80; finite frames>1,finite interval>0; rounded duration product finite positive. All32 resource flag words allowed.',contract='Partial entry9A3198 EAX=nonnullResource,EBX=Clip,ESP=Frame800,emptyx87; exits9A31C5. JMP9A31B0 skipsXORPS at9A31B2. SourcewritesFrame820 andClipB8..BF. Second entry9A32D4 EDX=Resource,EBX=Clip,realstack; exits9A32E7 afterloopADwrite. Noallocation/reference/binding/selection/events/counters claim.',memory='Five16byte guardedbuffers; wholeInput/Resource shadows,Clip shadowexceptAD/B8..BF,Frame shadowexcept7FC/820; Replypoison. TwoXMMpoisons agree; DF,ESP,CW,TOP,MXCSR controls checked.',exclusions='No full9A3140 constructor gate; no nullresource path; no channel allocation/wrapper setup; no live actor resource selection. Positive duration arithmetic only.',exits={f'{a:08X}':n for a,n in EXITS.items()},compile_commands=[['ml.exe','/nologo','/c','/coff','/Fohost-x86.obj','host-x86.asm'],['link.exe','/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/BASE:0x27000000','/DYNAMICBASE:NO','/SAFESEH:NO','/OUT:host-x86.exe','/MAP:host-x86.map','host-x86.obj','kernel32.lib']])
  (a.out/'manifest.json').write_text(json.dumps(r,indent=2)+'\n')
 print(json.dumps({'host_executed':False,'source_bytes':64}))
if __name__=='__main__':main()

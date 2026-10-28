"""Prepare1049originalB full fresh aid+boundedvectorreset gate. Never execute."""
import argparse,hashlib,json,re,struct,sys
from pathlib import Path
import cell_runtime_clip_metadata_native_probe as template
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder
ROOT=Path(__file__).resolve().parent.parent
CODE=[(0x7D0230,0x7D0272),(0x9A1690,0x9A16ED),(0x9A1A20,0x9A1A3A),(0x9A1A40,0x9A1AC6),(0x9A1B40,0x9A1B78),(0x9A2FA0,0x9A313D),(0x9A33C0,0x9A34C5)]
TRAPS={0xABB720:'reset0cannotgrow',0x9A17C0:'emptywrappercopy',0x11E0534:'emptywrappererase',0xF47290:'emptywrappererase',0x99CD90:'resourceprecleared',0x9C4B50:'ownerprecleared',0x99EDE0:'bindingprecleared'}
SERIAL=0x15514E0
BUFFERS=[('Input',64),('Clip',0xE0),('Events',128),('Frame',0x1000),('Reply',64)]
def sha(raw):return hashlib.sha256(raw).hexdigest()
def audit_source(image):
 sys.path.insert(0,str(ROOT/'target/rea/capstone-runtime'));from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM,X86_OP_MEM
 rows=[list(decoder().disasm(image.read(a,b-a),a))for a,b in CODE];boundaries={i.address for rr in rows for i in rr};report=[];called=set()
 for(a,b),rr in zip(CODE,rows):
  assert sum(i.size for i in rr)==b-a;edges=[]
  for i in rr:
   if i.group(CS_GRP_CALL)or i.group(CS_GRP_JUMP):
    assert i.operands[0].type==X86_OP_IMM;t=i.operands[0].imm;assert t in boundaries or(i.group(CS_GRP_CALL)and t in TRAPS);edges.append(dict(at=f'{i.address:08X}',target=f'{t:08X}',unreachable_reason=TRAPS.get(t)))
    if i.group(CS_GRP_CALL):called.add(t)
   for op in i.operands:
    if op.type==X86_OP_MEM and not op.mem.base and not op.mem.index:assert(op.mem.disp,op.size)in[(0x14861D0,4),(SERIAL,4)]
  report.append(dict(start=f'{a:08X}',end_exclusive=f'{b:08X}',bytes=b-a,sha256=sha(image.read(a,b-a)),transfers=edges,disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in rr]))
 assert set(TRAPS)<=called and sum(b-a for a,b in CODE)==1049;return report
def host_source(image):
 saved=(template.CODE,template.EXITS,template.BUFFERS)
 try:template.CODE=CODE;template.EXITS={};template.BUFFERS=BUFFERS;text=template.host_source(image)
 finally:template.CODE,template.EXITS,template.BUFFERS=saved
 a=text.index('ReadRecord PROC');b=text.index('ReadRecord ENDP')+len('ReadRecord ENDP');text=text[:a]+re.sub(r'\b32\b','64',text[a:b])+text[b:];text=text.replace('Expected BYTE 16 DUP(0)','Expected BYTE 56 DUP(0)')
 v=['ValidateRecord PROC','movzx eax,word ptr [InputBuffer+4]','mov edx,eax','and eax,0F0FFh','cmp eax,07Fh','jne HostFailure','and edx,0300h','cmp edx,0100h','je HostFailure','cmp word ptr [InputBuffer+6],0','jne HostFailure','cmp dword ptr [InputBuffer+8],1','ja HostFailure','mov eax,dword ptr [InputBuffer+16]','cmp eax,16','ja HostFailure','cmp eax,dword ptr [InputBuffer+12]','jb HostFailure','cmp dword ptr [InputBuffer+8],0','jne ValidEventMode','test eax,eax','jnz HostFailure','ValidEventMode:','mov esi,OFFSET InputBuffer+20','mov ecx,11','Reserved:','cmp dword ptr [esi],0','jne HostFailure','add esi,4','loop Reserved','ret','ValidateRecord ENDP']
 a=text.index('ValidateRecord PROC');b=text.index('ValidateRecord ENDP')+len('ValidateRecord ENDP');text=text[:a]+'\n'.join(v)+text[b:]
 r=['RunOnce PROC','mov NativeStack,esp','cld']
 for n,size in BUFFERS:
  if n!='Input':r +=[f'mov edi,OFFSET {n}Buffer',f'mov ecx,{size//4}','mov eax,0A5A5A5A5h','rep stosd']
 r +=['mov edi,01550000h','mov ecx,04000h','mov eax,0A5A5A5A5h','rep stosd','mov eax,dword ptr [InputBuffer]','mov dword ptr [015514E0h],eax','mov esi,01550000h','mov edi,OFFSET SerialShadow','mov ecx,04000h','rep movsd','mov dword ptr [FrameBuffer+0800h],OFFSET ConstructorDone']
 for n,size in BUFFERS:
  if n!='Reply':r +=[f'mov esi,OFFSET {n}Buffer',f'mov edi,OFFSET {n}Shadow',f'mov ecx,{size//4}','rep movsd']
 r +=['fninit','fldcw word ptr [InputBuffer+4]','ldmxcsr SseControl','mov ecx,OFFSET ClipBuffer','mov esp,OFFSET FrameBuffer+0800h','push 009A33C0h','ret','ConstructorDone::','cmp esp,OFFSET FrameBuffer+0804h','jne HostFailure','mov esp,NativeStack','cmp eax,OFFSET ClipBuffer','jne HostFailure','pushfd','pop eax','test eax,0400h','jnz HostFailure','cld','xor ecx,ecx','CheckConstructorClip:']
 allowed=set([0,4,8,12,16,20,24,28,32,*range(0x2C,0xAC,4),0xB0,0xB4,0xB8,0xBC,0xC0,0xC4,0xC8,0xCC,0xD0,0xDC])
 # Compare all bytes outside the exactsource write footprint. Clockprojection
 # independentlycompares important initialized values; otherfloatfields excluded.
 for at in sorted({off+i for off in allowed for i in range(4)}|{0xAC,0xAD}):r +=[f'cmp ecx,{at}', 'je NextConstructorClip']
 r +=['mov al,byte ptr [ClipBuffer+ecx]','cmp al,byte ptr [ClipShadow+ecx]','jne HostFailure','NextConstructorClip:','inc ecx','cmp ecx,0E0h','jb CheckConstructorClip','xor ecx,ecx','CheckConstructorFrame:','cmp ecx,07BCh','jb CompareConstructorFrame','cmp ecx,0800h','jb NextConstructorFrame','CompareConstructorFrame:','mov eax,dword ptr [FrameBuffer+ecx]','cmp eax,dword ptr [FrameShadow+ecx]','jne HostFailure','NextConstructorFrame:','add ecx,4','cmp ecx,01000h','jb CheckConstructorFrame']
 for at in[0,4,8,0x14,0x18,0x1C,0x20,0xB0,0xB4,0xB8,0xBC,0xC4,0xC8,0xCC,0xD0]:r +=[f'cmp dword ptr [ClipBuffer+{at}],0','jne HostFailure']
 r +=['cmp dword ptr [ClipBuffer+0Ch],0FFFFFFFFh','jne HostFailure','cmp word ptr [ClipBuffer+0ACh],0','jne HostFailure','cmp dword ptr [ClipBuffer+0C0h],1','jne HostFailure','cmp dword ptr [InputBuffer+8],0','je EventResetDone','mov dword ptr [ClipBuffer+0C8h],OFFSET EventsBuffer','mov eax,dword ptr [InputBuffer+12]','shl eax,3','add eax,OFFSET EventsBuffer','mov dword ptr [ClipBuffer+0CCh],eax','mov eax,dword ptr [InputBuffer+16]','shl eax,3','add eax,OFFSET EventsBuffer','mov dword ptr [ClipBuffer+0D0h],eax','mov esi,OFFSET ClipBuffer','mov edi,OFFSET ClipShadow','mov ecx,038h','rep movsd','mov edi,OFFSET FrameBuffer','mov ecx,0400h','mov eax,0A5A5A5A5h','rep stosd','mov dword ptr [FrameBuffer+0800h],OFFSET ResetDone','mov dword ptr [FrameBuffer+0804h],0','mov esi,OFFSET FrameBuffer','mov edi,OFFSET FrameShadow','mov ecx,0400h','rep movsd','mov ecx,OFFSET ClipBuffer+0C8h','mov esp,OFFSET FrameBuffer+0800h','push 009A1690h','ret','ResetDone::','cmp esp,OFFSET FrameBuffer+0808h','jne HostFailure','mov esp,NativeStack','pushfd','pop eax','test eax,0400h','jnz HostFailure','cld','xor ecx,ecx','CheckResetClip:','cmp ecx,0CCh','je NextResetClip','mov eax,dword ptr [ClipBuffer+ecx]','cmp eax,dword ptr [ClipShadow+ecx]','jne HostFailure','NextResetClip:','add ecx,4','cmp ecx,0E0h','jb CheckResetClip','cmp dword ptr [ClipBuffer+0CCh],OFFSET EventsBuffer','jne HostFailure','xor ecx,ecx','CheckResetFrame:','cmp ecx,07D8h','jb CompareResetFrame','cmp ecx,0800h','jb NextResetFrame','CompareResetFrame:','mov eax,dword ptr [FrameBuffer+ecx]','cmp eax,dword ptr [FrameShadow+ecx]','jne HostFailure','NextResetFrame:','add ecx,4','cmp ecx,01000h','jb CheckResetFrame','EventResetDone:','xor ecx,ecx','CheckSerialPage:','cmp ecx,014E0h','je NextSerial','mov eax,dword ptr [01550000h+ecx]','cmp eax,dword ptr [SerialShadow+ecx]','jne HostFailure','NextSerial:','add ecx,4','cmp ecx,010000h','jb CheckSerialPage','mov eax,dword ptr [InputBuffer]','inc eax','cmp eax,dword ptr [015514E0h]','jne HostFailure','mov eax,dword ptr [InputBuffer]','cmp eax,dword ptr [ClipBuffer+010h]','jne HostFailure']
 for n,size in BUFFERS:
  if n in['Input','Events']:r +=[f'mov esi,OFFSET {n}Buffer',f'mov edi,OFFSET {n}Shadow',f'mov ecx,{size//4}','repe cmpsd','jne HostFailure']
  for g in['Before','After']:
   for at in range(0,16,4):r +=[f'cmp dword ptr [{n}{g}+{at}],0A5A5A5A5h','jne HostFailure']
 for at in range(0,64,4):r +=[f'cmp dword ptr [ReplyBuffer+{at}],0A5A5A5A5h','jne HostFailure']
 r +=['mov eax,dword ptr [ClipBuffer+0Ch]','mov dword ptr [ReplyBuffer],eax','mov eax,dword ptr [ClipBuffer+010h]','mov dword ptr [ReplyBuffer+4],eax','mov eax,dword ptr [ClipBuffer+014h]','mov dword ptr [ReplyBuffer+8],eax','movzx eax,byte ptr [ClipBuffer+0ACh]','mov dword ptr [ReplyBuffer+12],eax','movzx eax,byte ptr [ClipBuffer+0ADh]','mov dword ptr [ReplyBuffer+16],eax']
 for dst,src in[(20,0xB0),(24,0xB4),(28,0xB8),(32,0xBC),(36,0xC0),(40,0xC4)]:r +=[f'mov eax,dword ptr [ClipBuffer+{src}]',f'mov dword ptr [ReplyBuffer+{dst}],eax']
 r +=['mov eax,dword ptr [ClipBuffer+0CCh]','sub eax,dword ptr [ClipBuffer+0C8h]','shr eax,3','mov dword ptr [ReplyBuffer+44],eax','mov eax,dword ptr [ClipBuffer+0D0h]','sub eax,dword ptr [ClipBuffer+0C8h]','shr eax,3','mov dword ptr [ReplyBuffer+48],eax','mov eax,dword ptr [015514E0h]','mov dword ptr [ReplyBuffer+52],eax','fnstcw word ptr [ReplyBuffer+56]','fnstsw word ptr [ReplyBuffer+58]','stmxcsr dword ptr [ReplyBuffer+60]','mov ax,word ptr [ReplyBuffer+56]','cmp ax,word ptr [InputBuffer+4]','jne HostFailure','test word ptr [ReplyBuffer+58],03800h','jnz HostFailure','cmp dword ptr [ReplyBuffer+60],01F80h','jne HostFailure','ret','RunOnce ENDP']
 a=text.index('RunOnce PROC');b=text.index('RunOnce ENDP')+len('RunOnce ENDP');text=text[:a]+'\n'.join(r)+text[b:];text=text.replace('.code\n','SerialShadow BYTE 65536 DUP(0)\n.code\n',1).replace('HostFailure,DurationDone,LoopDone','HostFailure,ConstructorDone,ResetDone')
 main=['HostMain PROC','cld','push -10','call _GetStdHandle@4','mov InputHandle,eax','push -11','call _GetStdHandle@4','mov OutputHandle,eax']
 for page in[0x7D0000,0x9A0000,0x1480000,0x1550000]:main +=['push 4','push 03000h','push 010000h',f'push 0{page:X}h','call _VirtualAlloc@16',f'cmp eax,0{page:X}h','jne AllocationFailure','mov edi,eax','mov ecx,04000h','mov eax,0CCCCCCCCh','rep stosd']
 for a,b in CODE:main +=[f'mov esi,OFFSET Original_{a:08X}',f'mov edi,0{a:X}h',f'mov ecx,{b-a}','rep movsb']
 main +=['mov esi,OFFSET OriginalOne','mov edi,014861D0h','mov ecx,4','rep movsb']
 for page,flags in[(0x7D0000,0x20),(0x9A0000,0x20),(0x1480000,2)]:main +=['push OFFSET OldProtect',f'push 0{flags:X}h','push 010000h',f'push 0{page:X}h','call _VirtualProtect@16','test eax,eax','jz HostFailure']
 for page in[0x7D0000,0x9A0000]:main +=['push 010000h',f'push 0{page:X}h','push -1','call _FlushInstructionCache@12','test eax,eax','jz HostFailure']
 a=text.index('HostMain PROC');b=text.index('MainLoop:');tail=text[b:].replace('mov ecx,16','mov ecx,56').replace('mov ebx,24','mov ebx,64');return text[:a]+'\n'.join(main)+'\n'+tail

def linked_audit(exe,mapping,image):
 base=template.base;saved=(base.CODE,base.BUFFERS)
 try:base.CODE=CODE;base.BUFFERS=BUFFERS;r=base.linked_audit(exe,mapping,image);pe=base.PE(exe);nt=struct.unpack_from('<I',pe.data,60)[0];assert pe.base+struct.unpack_from('<I',pe.data,nt+40)[0]==int(r['symbols']['HostMain'],16);r['pe_entrypoint_hostmain']=True;return r
 finally:base.CODE,base.BUFFERS=saved
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--audit-exe',type=Path);p.add_argument('--audit-map',type=Path);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());image=SourceImage(a.source_exe);source=audit_source(image);asm=host_source(image)
 if a.audit_exe:
  assert a.audit_map and(a.out/'host-x86.asm').read_text()==asm;r=linked_audit(a.audit_exe,a.audit_map,image)
  with(a.out/'linked-audit.json').open('x')as f:json.dump(r,f,indent=2)
 else:
  a.out.mkdir(parents=True,exist_ok=False);(a.out/'host-x86.asm').write_text(asm);m=dict(schema='source-cell-full-fresh-aid-host-v1',source_sha256=SOURCE_SHA256,source_bytes=1049,code_allowlist=source,unreachable_calls={f'{k:08X}':v for k,v in TRAPS.items()},host_source_sha256=sha((a.out/'host-x86.asm').read_bytes()),preparation_tool_sha256=sha(Path(__file__).read_bytes()),template_sha256=sha(Path(template.__file__).read_bytes()),auditor_sha256=sha(Path(template.base.__file__).read_bytes()),input_bytes=64,projection_bytes=56,output_bytes=64,input='globalserial32@0,CW:u16@4,reserved0:u16@6,mode0fresh/1freshthenpendingreset@8,eventcount@12,eventcapacity@16,reserved0@20..60',output='selection32@0,serial32@4,sampledwrapper32@8,active32@12,loop32@16,time64@20,duration64@28,cycle32@36,crossing32@40,eventlength32@44,eventcapacity32@48,nextglobalserial32@52,CW:u16@56,SW:u16@58,MXCSR32@60',domain='Twelve maskedCW,MXCSR1F80; explicitglobalserialsnapshotall32bits; mode0requirescountcapacity0; mode1requires0<=count<=capacity<=16. Eight-bytependingrecordsaretrivialboundedstorage,valuesunreadbyreset0.',source_contract='Full9A33C0 ECX=poisoned224BClip,ESPFrame800,returnConstructorDone. Constructorpreclearsresource/owner/binding/wrappers/eventvectorbeforefull9A2FA0(1); nullreferencesavoidreleasecalls; emptywrappercopy/erasehelpersfullsourceincludedandgrowth/copy/freeedgesunreachable. Fullreset9A1690(0)->7D0230 included. ConstructorreturnsClip,E SP804;optionalnonemptyresetreturnResetDone,ESP808. Fullsourcebytesunchanged,nomockcallbacks.',memory='Five16Bguardedbuffers. Input/Eventswholeshadows. ConstructorClipwholeshadowexceptexactsourcefixedwritefootprint; projectionadmitsscalarclock/serial/vector-sizefieldsonly. Afterinjectingboundedpendingvector,ClipwholeshadowexceptCC; capacityandrecordsunchanged. Framewholeshadowexcept7BC..800ctor or7D8..800reset. Originalglobalserialpage1550000RWfullyshadowedexcept4Bat15514E0. Replypoison,ESP/DF/CW/TOP/MXCSRchecked.',global_serial=dict(address='015514E0',bytes=4,initial='explicitinputsnapshot',after='sourceINCwrapping32'),compiled=False,host_executed=False,exclusions='Freshnewallocationonly.Noresourcebinding/recycledreferences/activeclockadvance/eventdispatch/liveactor/schedulingadmission.RootaccumfloatvaluesoutsideclockprojectionarenotmodeledbypublicAPI.',compile_commands=[['ml.exe','/nologo','/c','/coff','/Fohost-x86.obj','host-x86.asm'],['link.exe','/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/BASE:0x27000000','/DYNAMICBASE:NO','/SAFESEH:NO','/OUT:host-x86.exe','/MAP:host-x86.map','host-x86.obj','kernel32.lib']]);(a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
 print(json.dumps(dict(host_executed=False,source_bytes=1049)))
if __name__=='__main__':main()

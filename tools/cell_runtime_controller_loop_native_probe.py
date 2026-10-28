"""Prepare fullA00310/9FFE30 setter/handle boundary; never execute."""
import argparse,hashlib,json,re,struct,sys
from pathlib import Path
import cell_runtime_clip_metadata_native_probe as template
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder
ROOT=Path(__file__).resolve().parent.parent
CODE=[(0x9FFE30,0x9FFE99),(0xA00310,0xA0035E)]
BUFFERS=[('Input',64),('Controller',0xFD0),('ClipA',0xE0),('ClipB',0xE0),('Frame',0x1000),('Reply',36)]
def sha(raw):return hashlib.sha256(raw).hexdigest()
def audit_source(image):
 sys.path.insert(0,str(ROOT/'target/rea/capstone-runtime'));from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM,X86_OP_MEM
 rows=[list(decoder().disasm(image.read(a,b-a),a))for a,b in CODE];boundaries={i.address for rr in rows for i in rr};report=[];calls=[]
 for(a,b),rr in zip(CODE,rows):
  assert sum(i.size for i in rr)==b-a and rr[-1].mnemonic=='ret';transfers=[]
  for i in rr:
   if i.group(CS_GRP_CALL)or i.group(CS_GRP_JUMP):
    assert i.operands[0].type==X86_OP_IMM and i.operands[0].imm in boundaries;transfers.append(dict(at=f'{i.address:08X}',target=f'{i.operands[0].imm:08X}'))
    if i.group(CS_GRP_CALL):calls.append(i.operands[0].imm)
   for op in i.operands:
    if op.type==X86_OP_MEM:assert op.mem.base or op.mem.index
  report.append(dict(start=f'{a:08X}',end_exclusive=f'{b:08X}',bytes=b-a,sha256=sha(image.read(a,b-a)),transfers=transfers,disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in rr]))
 assert calls==[0x9FFE30];return report
def host_source(image):
 saved=(template.CODE,template.EXITS,template.BUFFERS)
 try:template.CODE=CODE;template.EXITS={};template.BUFFERS=BUFFERS;text=template.host_source(image)
 finally:template.CODE,template.EXITS,template.BUFFERS=saved
 # Reuse only reviewed IO/import/link scaffold; source183B and owned contracts
 # are independently audited below. UnusedOriginalOne remains a readonly pin.
 a=text.index('ReadRecord PROC');b=text.index('ReadRecord ENDP')+len('ReadRecord ENDP');text=text[:a]+re.sub(r'\b32\b','64',text[a:b])+text[b:];text=text.replace('Expected BYTE 16 DUP(0)','Expected BYTE 28 DUP(0)')
 validate=['ValidateRecord PROC','cmp dword ptr [InputBuffer+4],15','ja HostFailure','cmp dword ptr [InputBuffer+12],15','ja HostFailure','mov eax,dword ptr [InputBuffer+4]','cmp eax,dword ptr [InputBuffer+12]','je HostFailure']
 for at in[28,32]:validate +=[f'cmp dword ptr [InputBuffer+{at}],1','ja HostFailure']
 for at in[36,40,44,48]:validate +=[f'cmp dword ptr [InputBuffer+{at}],255','ja HostFailure']
 validate +=['cmp dword ptr [InputBuffer+56],0','jne HostFailure','cmp dword ptr [InputBuffer+60],0','jne HostFailure','xor eax,eax','cmp dword ptr [InputBuffer+24],1','sete al','cmp eax,dword ptr [InputBuffer+52]','jne HostFailure','mov eax,dword ptr [InputBuffer]','test eax,eax','jz Validated','mov edx,eax','and edx,255','mov ecx,dword ptr [InputBuffer+4]','inc ecx','cmp edx,ecx','jne HostFailure','shr eax,8','cmp eax,dword ptr [InputBuffer+8]','jne Validated','mov eax,dword ptr [InputBuffer+20]','test eax,eax','jz Validated','and eax,255','mov ecx,dword ptr [InputBuffer+12]','inc ecx','cmp eax,ecx','jne HostFailure','Validated:','ret','ValidateRecord ENDP']
 a=text.index('ValidateRecord PROC');b=text.index('ValidateRecord ENDP')+len('ValidateRecord ENDP');text=text[:a]+'\n'.join(validate)+text[b:]
 r=['RunOnce PROC','mov NativeStack,esp','cld']
 for n,size in BUFFERS:
  if n!='Input':r +=[f'mov edi,OFFSET {n}Buffer',f'mov ecx,{size//4}','mov eax,0A5A5A5A5h','rep stosd']
 for slot,index,epoch,link,present,initial,clip in [('Primary',4,8,20,28,36,'ClipA'),('Secondary',12,16,None,32,40,'ClipB')]:
  r +=[f'mov eax,dword ptr [InputBuffer+{index}]','imul eax,0F0h','add eax,OFFSET ControllerBuffer+8',f'mov {slot}Slot,eax',f'mov edx,dword ptr [InputBuffer+{epoch}]','mov dword ptr [eax+0B4h],edx']
  if link is not None:r +=[f'mov edx,dword ptr [InputBuffer+{link}]','mov dword ptr [eax+06Ch],edx']
  else:r +=['mov dword ptr [eax+06Ch],0']
  r +=['mov dword ptr [eax],0',f'cmp dword ptr [InputBuffer+{present}],0',f'je {slot}NoAid',f'mov dword ptr [eax],OFFSET {clip}Buffer',f'{slot}NoAid:',f'mov edx,dword ptr [InputBuffer+{initial}]','mov byte ptr [eax+0D4h],dl']
 r +=['mov eax,dword ptr [InputBuffer+44]','mov byte ptr [ClipABuffer+0ADh],al','mov eax,dword ptr [InputBuffer+48]','mov byte ptr [ClipBBuffer+0ADh],al','mov dword ptr [FrameBuffer+0800h],OFFSET SetterDone','mov dword ptr [FrameBuffer+0804h],OFFSET ControllerBuffer','mov eax,dword ptr [InputBuffer]','mov dword ptr [FrameBuffer+0808h],eax','mov eax,dword ptr [InputBuffer+52]','mov dword ptr [FrameBuffer+080Ch],eax']
 for n,size in BUFFERS:
  if n!='Reply':r +=[f'mov esi,OFFSET {n}Buffer',f'mov edi,OFFSET {n}Shadow',f'mov ecx,{size//4}','rep movsd']
 r +=['fninit','fldcw FixedControl','ldmxcsr SseControl','mov esp,OFFSET FrameBuffer+0800h','push 00A00310h','ret','SetterDone::','cmp esp,OFFSET FrameBuffer+0804h','jne HostFailure','mov esp,NativeStack','mov SetterResult,eax','pushfd','pop eax','test eax,0400h','jnz HostFailure','cld','mov eax,dword ptr [FrameBuffer+07F8h]','mov ResolvedPrimary,eax','mov eax,dword ptr [FrameBuffer+07FCh]','mov ResolvedSecondary,eax','cmp ResolvedPrimary,0','je NoResolvedPrimary','mov eax,PrimarySlot','cmp eax,ResolvedPrimary','jne HostFailure','cmp SetterResult,1','jne HostFailure','jmp ResolutionChecked','NoResolvedPrimary:','cmp SetterResult,0','jne HostFailure','cmp ResolvedSecondary,0','jne HostFailure','ResolutionChecked:','cmp ResolvedSecondary,0','je SecondaryChecked','mov eax,SecondarySlot','cmp eax,ResolvedSecondary','jne HostFailure','SecondaryChecked:','mov esi,OFFSET InputBuffer','mov edi,OFFSET InputShadow','mov ecx,16','repe cmpsd','jne HostFailure','xor ecx,ecx','CheckController:','mov eax,OFFSET ControllerBuffer','add eax,ecx','cmp ResolvedPrimary,0','je CompareController','mov edx,ResolvedPrimary','add edx,0D4h','cmp eax,edx','je NextController','CompareController:','mov al,byte ptr [ControllerBuffer+ecx]','cmp al,byte ptr [ControllerShadow+ecx]','jne HostFailure','NextController:','inc ecx','cmp ecx,0FD0h','jb CheckController']
 for name,present in [('ClipA',28),('ClipB',32)]:
  # Primary pointsClipA only; linkedClipB mustremain entirely unchanged.
  r +=['xor ecx,ecx',f'Check{name}:']
  if name=='ClipA':r +=['cmp ResolvedPrimary,0',f'je Compare{name}','cmp dword ptr [InputBuffer+28],0',f'je Compare{name}','cmp ecx,0ADh',f'je Next{name}']
  r +=[f'Compare{name}:',f'mov al,byte ptr [{name}Buffer+ecx]',f'cmp al,byte ptr [{name}Shadow+ecx]','jne HostFailure',f'Next{name}:','inc ecx','cmp ecx,0E0h',f'jb Check{name}']
 r +=['xor ecx,ecx','CheckFrame:','cmp ecx,07E0h','jb CompareFrame','cmp ecx,0800h','jb NextFrame','CompareFrame:','mov eax,dword ptr [FrameBuffer+ecx]','cmp eax,dword ptr [FrameShadow+ecx]','jne HostFailure','NextFrame:','add ecx,4','cmp ecx,01000h','jb CheckFrame']
 for n,size in BUFFERS:
  for g in['Before','After']:
   for at in range(0,16,4):r +=[f'cmp dword ptr [{n}{g}+{at}],0A5A5A5A5h','jne HostFailure']
 for at in range(0,36,4):r +=[f'cmp dword ptr [ReplyBuffer+{at}],0A5A5A5A5h','jne HostFailure']
 r +=['mov eax,SetterResult','mov dword ptr [ReplyBuffer],eax','mov eax,PrimarySlot','movzx edx,byte ptr [eax+0D4h]','mov dword ptr [ReplyBuffer+4],edx','mov eax,SecondarySlot','movzx edx,byte ptr [eax+0D4h]','mov dword ptr [ReplyBuffer+8],edx','movzx eax,byte ptr [ClipABuffer+0ADh]','mov dword ptr [ReplyBuffer+12],eax','movzx eax,byte ptr [ClipBBuffer+0ADh]','mov dword ptr [ReplyBuffer+16],eax','xor eax,eax','cmp ResolvedPrimary,0','setne al','mov dword ptr [ReplyBuffer+20],eax','xor eax,eax','cmp ResolvedSecondary,0','setne al','mov dword ptr [ReplyBuffer+24],eax','fnstcw word ptr [ReplyBuffer+28]','fnstsw word ptr [ReplyBuffer+30]','stmxcsr dword ptr [ReplyBuffer+32]','cmp word ptr [ReplyBuffer+28],027Fh','jne HostFailure','test word ptr [ReplyBuffer+30],03800h','jnz HostFailure','cmp dword ptr [ReplyBuffer+32],01F80h','jne HostFailure','ret','RunOnce ENDP']
 a=text.index('RunOnce PROC');b=text.index('RunOnce ENDP')+len('RunOnce ENDP');text=text[:a]+'\n'.join(r)+text[b:]
 text=text.replace('.code\n','FixedControl WORD 027Fh\nPrimarySlot DWORD 0\nSecondarySlot DWORD 0\nSetterResult DWORD 0\nResolvedPrimary DWORD 0\nResolvedSecondary DWORD 0\n.code\n',1).replace('HostFailure,DurationDone,LoopDone','HostFailure,SetterDone')
 main=['HostMain PROC','cld','push -10','call _GetStdHandle@4','mov InputHandle,eax','push -11','call _GetStdHandle@4','mov OutputHandle,eax']
 for page in[0x9F0000,0xA00000]:main +=['push 4','push 03000h','push 010000h',f'push 0{page:X}h','call _VirtualAlloc@16',f'cmp eax,0{page:X}h','jne AllocationFailure','mov edi,eax','mov ecx,04000h','mov eax,0CCCCCCCCh','rep stosd']
 for a,b in CODE:main +=[f'mov esi,OFFSET Original_{a:08X}',f'mov edi,0{a:X}h',f'mov ecx,{b-a}','rep movsb']
 for page in[0x9F0000,0xA00000]:main +=['push OFFSET OldProtect','push 020h','push 010000h',f'push 0{page:X}h','call _VirtualProtect@16','test eax,eax','jz HostFailure']
 main +=['push 020000h','push 009F0000h','push -1','call _FlushInstructionCache@12','test eax,eax','jz HostFailure']
 a=text.index('HostMain PROC');b=text.index('MainLoop:');tail=text[b:].replace('mov ecx,16','mov ecx,28').replace('mov ebx,24','mov ebx,36');text=text[:a]+'\n'.join(main)+'\n'+tail
 return text
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
  a.out.mkdir(parents=True,exist_ok=False);(a.out/'host-x86.asm').write_text(asm);m=dict(schema='source-cell-full-controller-loop-setter-host-v1',source_sha256=SOURCE_SHA256,source_bytes=183,code_allowlist=source,host_source_sha256=sha((a.out/'host-x86.asm').read_bytes()),preparation_tool_sha256=sha(Path(__file__).read_bytes()),template_sha256=sha(Path(template.__file__).read_bytes()),auditor_sha256=sha(Path(template.base.__file__).read_bytes()),input_bytes=64,projection_bytes=28,output_bytes=36,input='handle@0,primaryindex@4,primaryepoch@8,secondaryindex@12,secondaryepoch@16,linkhandle@20,propertymode@24,primaryaidbool@28,secondaryaidbool@32,primaryslotloopbyte@36,secondaryslotloopbyte@40,primaryaidloopbyte@44,secondaryaidloopbyte@48,sourcepolicybyte(mode==1)@52,reserved0@56/60',output='success@0,primaryslotloop@4,secondaryslotloop@8,primaryaidloop@12,secondaryaidloop@16,resolvedprimarybool@20,resolvedsecondarybool@24,CW:u16@28,SW:u16@30,MXCSR@32',domain='Two distinctindexslots0..15,rawinitialloopbytes0..255,sourcepolicy(mode==1),handle0 orlow8==primaryindex+1; onlyifprimaryepochmatches,nonzerolinklow8 must==secondaryindex+1. AidpointersnullorownClipA/B. Nootherpointertargets.',source_contract='Both183originalbytes/fullfunctionsunchanged. EntryA00310 ESPFrame800,returnSetterDone at800,argsqueue/handle/sourcebyte804/808/80C. Callsfull9FFE30 atoriginalVA. ExitESP804 thenrealstackrestore. SourceoutpairremainsFrame7F8/7FC; onlyresolvedprimaryD4 andprimaryAidAD writable. Linkedsecondarynevermutated. Sourcepropertymode mapping is measuredq53; rawsettergate callswithindependentlycheckedbyte.',memory='Six16B guardedbuffers. WholeInput/ClipBshadow; ControllerwholeexceptresolvedprimaryD4; ClipAwholeexceptADwhenprimaryresolvesandhasAid; Framewholeexcept7E0..800; Replypoison. DF/ESP/CW/TOP/MXCSRchecked. No originalallocation/import/callback. UnusedreadonlyOriginalOne constant retained onlyforreusedstaticlinkauditor.',compiled=False,host_executed=False,exclusions='Noactualcontrollercreation/actorresourceadmission/freshcounterinitialization/eventdispatch orloopclockexecution. Topologyexplicit; sourceepochmismatchperformszero writes.',compile_commands=[['ml.exe','/nologo','/c','/coff','/Fohost-x86.obj','host-x86.asm'],['link.exe','/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/BASE:0x27000000','/DYNAMICBASE:NO','/SAFESEH:NO','/OUT:host-x86.exe','/MAP:host-x86.map','host-x86.obj','kernel32.lib']]);(a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
 print(json.dumps(dict(host_executed=False,source_bytes=183)))
if __name__=='__main__':main()

"""Prepare full fresh activation/crossing low-sparse/high-CRT hybrid. Never execute."""
import argparse,hashlib,json,re,struct,sys
from pathlib import Path
import cell_runtime_fresh_aid_native_probe as fresh
import cell_runtime_controller_loop_native_probe as setter
import cell_runtime_loop_clock_native_probe as arithmetic
from cell_native_math_probe import db
from native_crt_acos_probe import decoder,PE
ROOT=Path(__file__).resolve().parent.parent
BASE=0x7C0000;OWNED_BASE=0x27000000
APP_CODE=sorted(fresh.CODE+setter.CODE+[(0x9A3198,0x9A31C5),(0x9A32D4,0x9A32E7),(0x9A2BE0,0x9A2C70),(0x9A7F30,0x9A7F9D),(0x9A7FA0,0x9A7FB5),(0x9A1CB0,0x9A1DCA),(0x9A2BB0,0x9A2BD6),(0x9A1590,0x9A168F),(0x11E04D0,0x11E04EF),(0x99BCB0,0x99BCE1)])
CRT_CODE=arithmetic.CRT_CODE;CODE=APP_CODE+CRT_CODE
DATA=arithmetic.DATA;ONE=(0x14861D0,4)
EXITS={0x9A31C5:'MetadataDurationDone',0x9A32E7:'MetadataLoopDone',0x785A3429:'HostFailure',0x785A940A:'HostFailure'}
TRAPS=dict(fresh.TRAPS,**{})
TRAPS.update({0x9A14C0:'fourzerosamplingmasks',0x99F6A0:'fourzerosamplingmasks',0xAD88C0:'freshowner0',0x9C8780:'freshowner0'})
BUFFERS=[('Input',256),('Clip',0xE0),('Events',128),('Frame',0x1000),('Reply',864),('Resource',0x150),('Context',16),('Wrappers',16*0x1F0),('Controller',0xFD0),('State',64)]
LOW_PAGES={0x7D0000:0x60000020,0x990000:0x60000020,0x9A0000:0x60000020,0x9F0000:0x60000020,0xA00000:0x60000020,0x11E0000:0x60000020,0x13C0000:0x40000040,0x1480000:0x40000040,0x1550000:0xC0000040,0x1660000:0xC0000040}
HIGH_PAGES={0x78530000:2,0x785A0000:0x20,0x785B0000:2}
IMPORTS={'GetStdHandle','ReadFile','WriteFile','GetLastError','ExitProcess','GetModuleHandleA','VirtualAlloc','VirtualProtect','FlushInstructionCache'}
def sha(raw):return hashlib.sha256(raw).hexdigest()
def audit_source(images):
 sys.path.insert(0,str(ROOT/'target/rea/capstone-runtime'));from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM,X86_OP_MEM
 rows=[list(decoder().disasm(images.read(a,b-a),a))for a,b in CODE];bounds={i.address for rr in rows for i in rr};reports=[]
 for(a,b),rr in zip(CODE,rows):
  assert sum(i.size for i in rr)==b-a;edges=[]
  for i in rr:
   if i.group(CS_GRP_CALL)or i.group(CS_GRP_JUMP):
    op=i.operands[0]
    if op.type==X86_OP_IMM:t=op.imm;assert t in bounds|set(EXITS)|set(TRAPS)
    else:assert i.address==0x99BCC1 and op.type==X86_OP_MEM and op.mem.disp==0x13CC4C0;t=0x13CC4C0
    edges.append(dict(at=f'{i.address:08X}',kind=i.mnemonic,target=f'{t:08X}',exclusion=TRAPS.get(t)))
   for op in i.operands:
    if op.type==X86_OP_MEM and not op.mem.base and not op.mem.index:assert(op.mem.disp,op.size)in DATA+[ONE,(0x15514E0,4),(0x166C240,4),(0x13CC4C0,4),(0x785BD7B0,4)]
  reports.append(dict(start=f'{a:08X}',end_exclusive=f'{b:08X}',bytes=b-a,sha256=sha(images.read(a,b-a)),transfers=edges,disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in rr]))
 assert sum(b-a for a,b in CODE)==2442;return reports

def host_source(images):
 saved=(fresh.CODE,fresh.BUFFERS)
 try:fresh.CODE=CODE;fresh.BUFFERS=BUFFERS;text=fresh.host_source(images)
 finally:fresh.CODE,fresh.BUFFERS=saved
 # Keep the admitted freshconstructor/per-record prefix, not its old startup.
 a=text.index('RunOnce PROC');b=text.index('mov esi,OFFSET InputBuffer',text.index('EventResetDone:',a));prefix=text[a:b]
 g=['mov edi,01660000h','mov ecx,04000h','mov eax,0A5A5A5A5h','rep stosd','mov eax,dword ptr [InputBuffer+20]','mov dword ptr [0166C240h],eax','mov esi,01660000h','mov edi,OFFSET GenerationShadow','mov ecx,04000h','rep movsd','mov ProviderCalls,0','mov SnapshotIndex,0']
 prefix=prefix.replace('mov dword ptr [FrameBuffer+0800h],OFFSET ConstructorDone','\n'.join(g)+'\nmov dword ptr [FrameBuffer+0800h],OFFSET ConstructorDone',1)
 text=text.replace('Expected BYTE 56 DUP(0)','Expected BYTE 864 DUP(0)')
 a=text.index('ReadRecord PROC');b=text.index('ReadRecord ENDP')+len('ReadRecord ENDP');text=text[:a]+re.sub(r'\b64\b','256',text[a:b])+text[b:]
 # Newstrictrecorddomain. Runtime guard beforeeachclock repeats argument/division
 # only to exclude negative/nonfinite/indefinite paths; originalcallstillruns.
 v=['ValidateRecord PROC','movzx eax,word ptr [InputBuffer+4]','mov edx,eax','and eax,0F0FFh','cmp eax,07Fh','jne HostFailure','and edx,0300h','cmp edx,0100h','je HostFailure','cmp word ptr [InputBuffer+6],0','jne HostFailure','cmp dword ptr [InputBuffer+8],1','jne HostFailure','cmp dword ptr [InputBuffer+12],0','jne HostFailure','cmp dword ptr [InputBuffer+16],16','ja HostFailure','mov eax,dword ptr [InputBuffer+28]','test eax,eax','jz HostFailure','cmp eax,07F800000h','jae HostFailure','mov eax,dword ptr [InputBuffer+32]','cmp eax,03F800000h','jbe HostFailure','cmp eax,07F800000h','jae HostFailure','mov eax,dword ptr [InputBuffer+36]','dec eax','cmp eax,15','ja HostFailure','cmp dword ptr [InputBuffer+40],8','jne HostFailure']
 for at in[44,48,52,56]:v +=[f'cmp dword ptr [InputBuffer+{at}],0','jne HostFailure']
 for at in[60,64,68]:v +=[f'cmp dword ptr [InputBuffer+{at}],1','jne HostFailure']
 v+=['mov esi,OFFSET InputBuffer+72','mov ecx,14','HeaderReserved:','cmp dword ptr [esi],0','jne HostFailure','add esi,4','loop HeaderReserved','mov esi,OFFSET InputBuffer+128','mov ecx,8','ValidateSteps:','cmp dword ptr [esi],1','ja HostFailure','mov eax,dword ptr [esi+4]','and eax,07FFFFFFFh','cmp eax,07F800000h','jae HostFailure','mov eax,dword ptr [esi+8]','cmp eax,dword ptr [InputBuffer+16]','ja HostFailure','cmp dword ptr [esi+12],0','jne HostFailure','add esi,16','loop ValidateSteps','ret','ValidateRecord ENDP']
 a=text.index('ValidateRecord PROC');b=text.index('ValidateRecord ENDP')+len('ValidateRecord ENDP');text=text[:a]+'\n'.join(v)+text[b:]
 def shadows(names):
  out=[]
  for n,size in BUFFERS:
   if n in names:out +=[f'mov esi,OFFSET {n}Buffer',f'mov edi,OFFSET {n}Shadow',f'mov ecx,{size//4}','rep movsd']
  return out
 def frame(label,low,high,extra=0xffffffff,args=None):
  q=['mov edi,OFFSET FrameBuffer','mov ecx,0400h','mov eax,0A5A5A5A5h','rep stosd',f'mov FrameLow,{low}',f'mov FrameHigh,{high}',f'mov FrameExtra,{extra}']
  for at,value in(args or{}).items():q +=[f'mov dword ptr [FrameBuffer+{at}],{value}']
  q+=shadows(['Frame']);return q
 def enter(address,esp):return[f'mov esp,OFFSET FrameBuffer+{esp}',f'push 0{address:X}h','ret']
 def restore(label,esp):return[label+'::',f'cmp esp,OFFSET FrameBuffer+{esp}','jne HostFailure','mov esp,NativeStack','pushfd','pop edx','test edx,0400h','jnz HostFailure','cld']
 r=prefix.splitlines()
 # Explicit binding snapshot: ownership/refcounts/full9A3140remainexcluded.
 r+=['mov dword ptr [ClipBuffer],OFFSET ResourceBuffer','mov dword ptr [ClipBuffer+8],OFFSET ContextBuffer','mov dword ptr [ClipBuffer+014h],OFFSET WrappersBuffer','mov dword ptr [ClipBuffer+018h],OFFSET WrappersBuffer','mov eax,dword ptr [InputBuffer+36]','imul eax,01F0h','add eax,OFFSET WrappersBuffer','mov dword ptr [ClipBuffer+01Ch],eax','mov dword ptr [ClipBuffer+020h],OFFSET WrappersBuffer+7936','mov edi,OFFSET ContextBuffer','xor eax,eax','mov ecx,4','rep stosd']
 for dst,src in[(0x118,24),(0x11C,28),(0x120,32),(0x144,36)]:r +=[f'mov eax,dword ptr [InputBuffer+{src}]',f'mov dword ptr [ResourceBuffer+{dst}],eax']
 r+=shadows(['Resource','Context','Wrappers','Clip'])+['mov ClipMask,OFFSET MaskDuration','mov ControllerWrite,0']
 r+=frame('metadata',0x7fc,0x800,0x820)+['mov ebx,OFFSET ClipBuffer','mov eax,OFFSET ResourceBuffer']+enter(0x9A3198,0x800)+restore('MetadataDurationDone',0x800)+['call GuardCurrent','mov eax,dword ptr [ClipBuffer+0BCh]','test eax,080000000h','jnz HostFailure','mov edx,eax','and edx,07FF00000h','cmp edx,07FF00000h','je HostFailure','or eax,dword ptr [ClipBuffer+0B8h]','jz HostFailure']
 r+=shadows(['Clip'])+['mov ClipMask,OFFSET MaskLoop']+frame('loopdefault',0x7fc,0x800)+['mov edx,OFFSET ResourceBuffer','mov ebx,OFFSET ClipBuffer']+enter(0x9A32D4,0x800)+restore('MetadataLoopDone',0x800)+['call GuardCurrent']
 r+=['mov edi,OFFSET ControllerBuffer','mov ecx,0FD0h/4','xor eax,eax','rep stosd','mov dword ptr [ControllerBuffer+8],OFFSET ClipBuffer']+shadows(['Controller','Clip'])+['mov ClipMask,OFFSET MaskLoop','mov ControllerWrite,1']
 r+=frame('setter',0x7e0,0x800,args={0x800:'OFFSET SetterDone',0x804:'OFFSET ControllerBuffer',0x808:'1',0x80c:'1'})+enter(0xA00310,0x800)+restore('SetterDone',0x804)+['cmp eax,1','jne HostFailure','cmp byte ptr [ClipBuffer+0ADh],1','jne HostFailure','cmp byte ptr [ControllerBuffer+0DCh],1','jne HostFailure','call GuardCurrent','mov ControllerWrite,0']+shadows(['Clip','Controller'])
 r+=['mov ClipMask,OFFSET MaskActivation','mov ClockBase,OFFSET FrameBuffer+0850h','mov ExpectedProviderCalls,1']+frame('activation',0x810,0x880,args={0x880:'OFFSET ActivationDone',0x884:'OFFSET ClipBuffer',0x888:'0'})+enter(0x9A2BE0,0x880)+restore('ActivationDone',0x884)+['cmp al,1','jne HostFailure','cmp dword ptr [ClipBuffer+0C0h],2','jne HostFailure','cmp dword ptr [ClipBuffer+0C4h],0','jne HostFailure','cmp byte ptr [ClipBuffer+0ACh],1','jne HostFailure','cmp ProviderCalls,1','jne HostFailure','fld dword ptr [FrameBuffer+0868h]','fstp tbyte ptr [ReplyBuffer]','call GuardCurrent','call CaptureSnapshot','mov SnapshotIndex,1','StepLoop:','mov eax,SnapshotIndex','dec eax','shl eax,4','add eax,OFFSET InputBuffer+128','mov StepPointer,eax','mov eax,dword ptr [eax+8]','shl eax,3','add eax,OFFSET EventsBuffer','mov dword ptr [ClipBuffer+0CCh],eax']+shadows(['Clip'])+['mov ClipMask,OFFSET MaskClock','mov ClockBase,OFFSET FrameBuffer+0800h','mov eax,SnapshotIndex','inc eax','mov ExpectedProviderCalls,eax','call ValidateNextTime','mov eax,StepPointer','cmp dword ptr [eax],0','jne PrepareDelta']
 r+=frame('absolute',0x7c0,0x814,0x818,args={0x814:'OFFSET StepDone',0x818:'OFFSET ClipBuffer',0x820:'1',0x824:'1'})+['mov eax,StepPointer','mov eax,dword ptr [eax+4]','mov dword ptr [FrameBuffer+081Ch],eax']+shadows(['Frame'])+enter(0x9A1CB0,0x814)
 r+=['PrepareDelta:']+frame('delta',0x7c0,0x828,args={0x828:'OFFSET DeltaDone',0x82c:'OFFSET ClipBuffer',0x834:'1',0x838:'1'})+['mov eax,StepPointer','mov eax,dword ptr [eax+4]','mov dword ptr [FrameBuffer+0830h],eax']+shadows(['Frame'])+enter(0x9A2BB0,0x828)+restore('DeltaDone',0x82c)+['jmp ClockReturned']+restore('StepDone',0x818)+['ClockReturned:','mov eax,SnapshotIndex','imul eax,96','add eax,OFFSET ReplyBuffer','fstp tbyte ptr [eax]','mov eax,dword ptr [FrameBuffer+081Ch]','cmp eax,dword ptr [StateBuffer+12]','jne HostFailure','mov eax,ProviderCalls','cmp eax,ExpectedProviderCalls','jne HostFailure','call GuardCurrent','call CaptureSnapshot','inc SnapshotIndex','cmp SnapshotIndex,9','jb StepLoop','cmp ProviderCalls,9','jne HostFailure']
 r+=['mov eax,dword ptr [InputBuffer]','add eax,2','cmp eax,dword ptr [015514E0h]','jne HostFailure','ret','RunOnce ENDP']
 a=text.index('RunOnce PROC');b=text.index('RunOnce ENDP')+len('RunOnce ENDP');text=text[:a]+'\n'.join(r)+text[b:]
 # Providerforwards actualsourceargs to full finiteCRTbytes; no numericstub.
 provider=['ModfProvider::','mov eax,ClockBase','sub eax,020h','cmp esp,eax','jne HostFailure','cmp dword ptr [esp],0099BCC7h','jne HostFailure','mov eax,ClockBase','sub eax,010h','cmp eax,dword ptr [esp+12]','jne HostFailure','inc ProviderCalls','mov eax,ProviderCalls','cmp eax,ExpectedProviderCalls','jne HostFailure','mov eax,dword ptr [esp+4]','mov edx,dword ptr [esp+8]','push dword ptr [esp+12]','push edx','push eax','push OFFSET ModfReturn','push 0785A3350h','ret','ModfReturn::','mov eax,ClockBase','sub eax,02Ch','cmp esp,eax','jne HostFailure','fstp tbyte ptr [StateBuffer]','fld tbyte ptr [StateBuffer]','add esp,12','ret']
 helper=['ValidateNextTime PROC','mov eax,StepPointer','cmp dword ptr [eax],0','jne ValidateDelta','mov eax,dword ptr [eax+4]','mov dword ptr [StateBuffer+12],eax','jmp ValidateCommitted','ValidateDelta:','fld dword ptr [eax+4]','fadd qword ptr [ClipBuffer+0B0h]','fstp dword ptr [StateBuffer+12]','ValidateCommitted:','mov eax,dword ptr [StateBuffer+12]','mov edx,eax','and edx,07FFFFFFFh','cmp edx,07F800000h','jae HostFailure','test eax,080000000h','jz ValidateQuotient','test edx,edx','jnz HostFailure','ValidateQuotient:','fld dword ptr [StateBuffer+12]','fdiv qword ptr [ClipBuffer+0B8h]','fstp dword ptr [StateBuffer+16]','mov eax,dword ptr [StateBuffer+16]','and eax,07FFFFFFFh','cmp eax,04F000000h','jae HostFailure','ret','ValidateNextTime ENDP']
 helper+=['GuardCurrent PROC','cmp dword ptr [ClipBuffer+4],0','jne HostFailure','pushfd','pop eax','test eax,0400h','jnz HostFailure','cld','xor ecx,ecx','GuardClip:','mov eax,ClipMask','cmp byte ptr [eax+ecx],0','jne NextGuardClip','mov al,byte ptr [ClipBuffer+ecx]','cmp al,byte ptr [ClipShadow+ecx]','jne HostFailure','NextGuardClip:','inc ecx','cmp ecx,0E0h','jb GuardClip','xor ecx,ecx','GuardFrame:','cmp ecx,FrameLow','jb CompareGuardFrame','cmp ecx,FrameHigh','jb NextGuardFrame','cmp ecx,FrameExtra','je NextGuardFrame','CompareGuardFrame:','mov eax,dword ptr [FrameBuffer+ecx]','cmp eax,dword ptr [FrameShadow+ecx]','jne HostFailure','NextGuardFrame:','add ecx,4','cmp ecx,01000h','jb GuardFrame','xor ecx,ecx','GuardResource:','cmp ecx,012Ch','je NextResource','mov eax,dword ptr [ResourceBuffer+ecx]','cmp eax,dword ptr [ResourceShadow+ecx]','jne HostFailure','NextResource:','add ecx,4','cmp ecx,0150h','jb GuardResource','xor ecx,ecx','GuardController:','cmp ControllerWrite,0','je CompareController','cmp ecx,0DCh','je NextController','CompareController:','mov al,byte ptr [ControllerBuffer+ecx]','cmp al,byte ptr [ControllerShadow+ecx]','jne HostFailure','NextController:','inc ecx','cmp ecx,0FD0h','jb GuardController']
 for n,size in BUFFERS:
  if n in['Input','Events','Context','Wrappers']:helper +=[f'mov esi,OFFSET {n}Buffer',f'mov edi,OFFSET {n}Shadow',f'mov ecx,{size//4}','repe cmpsd','jne HostFailure']
  for guard in['Before','After']:
   for at in range(0,16,4):helper +=[f'cmp dword ptr [{n}{guard}+{at}],0A5A5A5A5h','jne HostFailure']
 for lo,page,shadow,skip in [('Serial',0x1550000,'SerialShadow',0x14e0),('Generation',0x1660000,'GenerationShadow',0xc240)]:helper +=['xor ecx,ecx',f'Guard{lo}Page:',f'cmp ecx,{skip}',f'je Next{lo}Page',f'mov eax,dword ptr [0{page:X}h+ecx]',f'cmp eax,dword ptr [{shadow}+ecx]','jne HostFailure',f'Next{lo}Page:','add ecx,4','cmp ecx,010000h',f'jb Guard{lo}Page']
 for at in[10,20,24,28,32,36,40,44,48,52,56,60]:
  if at==10:helper +=['cmp word ptr [StateBuffer+10],0A5A5h','jne HostFailure']
  else:helper +=[f'cmp dword ptr [StateBuffer+{at}],0A5A5A5A5h','jne HostFailure']
 helper+=['ret','GuardCurrent ENDP','CaptureSnapshot PROC','mov eax,SnapshotIndex','imul eax,96','add eax,OFFSET ReplyBuffer','mov edi,eax','mov word ptr [edi+10],0','mov eax,ClockBase','mov edx,dword ptr [eax+018h]','mov dword ptr [edi+12],edx','mov edx,dword ptr [eax-01Ch]','mov dword ptr [edi+16],edx','mov edx,dword ptr [eax+01Ch]','mov dword ptr [edi+20],edx']
 for dst,src in[(24,0xB0),(28,0xB4),(32,0xC0),(36,0xC4),(48,0x10),(80,0xB8),(84,0xBC)]:helper +=[f'mov eax,dword ptr [ClipBuffer+{src}]',f'mov dword ptr [edi+{dst}],eax']
 for dst,src in[(40,'ResourceBuffer+012Ch'),(44,'0166C240h'),(52,'015514E0h'),(76,'ResourceBuffer+0120h')]:helper +=[f'mov eax,dword ptr [{src}]',f'mov dword ptr [edi+{dst}],eax']
 helper+=['mov eax,dword ptr [ClipBuffer+0CCh]','sub eax,dword ptr [ClipBuffer+0C8h]','shr eax,3','mov dword ptr [edi+56],eax','mov eax,dword ptr [ClipBuffer+0D0h]','sub eax,dword ptr [ClipBuffer+0C8h]','shr eax,3','mov dword ptr [edi+60],eax','movzx eax,byte ptr [ClipBuffer+0ACh]','mov dword ptr [edi+64],eax','movzx eax,byte ptr [ClipBuffer+0ADh]','mov dword ptr [edi+68],eax','mov eax,dword ptr [ClipBuffer+0Ch]','mov dword ptr [edi+72],eax','fnstcw word ptr [edi+88]','fnstsw word ptr [edi+90]','stmxcsr dword ptr [edi+92]','mov ax,word ptr [edi+88]','cmp ax,word ptr [InputBuffer+4]','jne HostFailure','test word ptr [edi+90],03800h','jnz HostFailure','mov eax,dword ptr [edi+92]','and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure','ret','CaptureSnapshot ENDP']
 text=text.replace('RunOnce PROC\n','\n'.join(provider+helper)+'\nRunOnce PROC\n',1)
 extra=['GenerationShadow BYTE 65536 DUP(0)','ClockBase DWORD 0','SnapshotIndex DWORD 0','StepPointer DWORD 0','ProviderCalls DWORD 0','ExpectedProviderCalls DWORD 0','ClipMask DWORD 0','ControllerWrite DWORD 0','FrameLow DWORD 0','FrameHigh DWORD 0','FrameExtra DWORD 0']
 text=text.replace('.code\n','\n'.join(extra)+'\nPUBLIC MaskDuration,MaskLoop,MaskActivation,MaskClock,Original_01485E24,Original_785347F0,Original_78534800,InputShadow,ClipShadow,EventsShadow,FrameShadow,ResourceShadow,ContextShadow,WrappersShadow,ControllerShadow,StateShadow,SerialShadow,GenerationShadow,NativeStack,ClockBase,SnapshotIndex,StepPointer,ProviderCalls,ExpectedProviderCalls,ClipMask,ControllerWrite,FrameLow,FrameHigh,FrameExtra,InputHandle,OutputHandle,Transferred,OldProtect,SseControl,Poison,Expected,ModfProvider,ModfReturn,GuardCurrent,CaptureSnapshot,ValidateNextTime\n.code\n',1)
 for name,offsets in [('MaskDuration',list(range(0xB8,0xC0))),('MaskLoop',[0xAD]),('MaskActivation',list(range(0xC,0x14))+[0xAC]+list(range(0xB0,0xB8))+list(range(0xC0,0xC8))+list(range(0xCC,0xD0))),('MaskClock',list(range(0xB0,0xB8))+list(range(0xC0,0xC8))+list(range(0xCC,0xD0)))]:
  mask=bytes(int(i in offsets)for i in range(0xE0));text=text.replace('.data\n','\n'.join(db(name,mask))+'\n.data\n',1)
 for at,n in DATA:text=text.replace('.data\n','\n'.join(db(f'Original_{at:08X}',images.read(at,n)))+'\n.data\n',1)
 text=text.replace('EXTERN _GetStdHandle@4:PROC','EXTERN _GetModuleHandleA@4:PROC\nEXTERN _GetStdHandle@4:PROC',1)
 text=text.replace('PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,ConstructorDone,ResetDone','PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,ConstructorDone,ResetDone,MetadataDurationDone,MetadataLoopDone,SetterDone,ActivationDone,StepDone,DeltaDone',1)
 main=['HostMain PROC','cld','push 0','call _GetModuleHandleA@4',f'cmp eax,0{BASE:X}h','jne HostFailure']
 for a,b in APP_CODE:main +=[f'mov esi,OFFSET Original_{a:08X}',f'mov edi,0{a:X}h',f'mov ecx,{b-a}','repe cmpsb','jne HostFailure']
 main+=['cmp dword ptr [014861D0h],03F800000h','jne HostFailure','cmp dword ptr [01485E24h],0','jne HostFailure','cmp dword ptr [013CC4C0h],OFFSET ModfProvider','jne HostFailure']
 for page in HIGH_PAGES:main +=['push 4','push 03000h','push 010000h',f'push 0{page:X}h','call _VirtualAlloc@16',f'cmp eax,0{page:X}h','jne AllocationFailure','mov edi,eax','mov ecx,04000h','mov eax,0CCCCCCCCh','rep stosd']
 for a,b in CRT_CODE:main +=[f'mov esi,OFFSET Original_{a:08X}',f'mov edi,0{a:X}h',f'mov ecx,{b-a}','rep movsb']
 for a,n in DATA:
  if a>=0x78000000:main +=[f'mov esi,OFFSET Original_{a:08X}',f'mov edi,0{a:X}h',f'mov ecx,{n}','rep movsb']
 main +=['mov dword ptr [0785BD7B0h],1']
 for at,label in EXITS.items():
  if at>=0x78000000:main +=[f'mov byte ptr [0{at:X}h],0B8h',f'mov eax,OFFSET {label}',f'mov dword ptr [0{at+1:X}h],eax',f'mov word ptr [0{at+5:X}h],0E0FFh']
 for page,flags in HIGH_PAGES.items():
  main +=['push OFFSET OldProtect',f'push 0{flags:X}h','push 010000h',f'push 0{page:X}h','call _VirtualProtect@16','test eax,eax','jz HostFailure']
  if flags==0x20:main +=['push 010000h',f'push 0{page:X}h','push -1','call _FlushInstructionCache@12','test eax,eax','jz HostFailure']
 for a,b in CRT_CODE:main +=[f'mov esi,OFFSET Original_{a:08X}',f'mov edi,0{a:X}h',f'mov ecx,{b-a}','repe cmpsb','jne HostFailure']
 main +=['cmp dword ptr [0785BD7B0h],1','jne HostFailure','push -10','call _GetStdHandle@4','mov InputHandle,eax','push -11','call _GetStdHandle@4','mov OutputHandle,eax']
 a=text.index('HostMain PROC');b=text.index('MainLoop:');tail=text[b:].replace('mov ecx,56','mov ecx,864').replace('mov ebx,64','mov ebx,864');return text[:a]+'\n'.join(main)+'\n'+tail

def linked_audit(exe,mapping,images):
 pe=PE(exe);assert pe.base==OWNED_BASE;symbols={}
 for line in mapping.read_text().splitlines():
  m=re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]+\s+(\S+)\s+([0-9A-Fa-f]{8})\b',line)
  if m:symbols[m[1]]=int(m[2],16)
 first,last=symbols['OwnedStart'],symbols['OwnedEnd'];rows=list(decoder().disasm(pe.read(first,last-first),first));assert sum(i.size for i in rows)==last-first;bounds={i.address for i in rows};imports=pe.imports();assert len(imports)==9 and {name for dll,name in imports.values()if dll.lower()=='kernel32.dll'}==IMPORTS
 from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM
 for i in rows:
  if i.group(CS_GRP_CALL)or i.group(CS_GRP_JUMP):
   assert i.operands[0].type==X86_OP_IMM;t=i.operands[0].imm
   if t not in bounds:raw=pe.read(t,6);assert i.mnemonic=='call'and raw[:2]==b'\xff\x25'and struct.unpack_from('<I',raw,2)[0]in imports
 nt=struct.unpack_from('<I',pe.data,60)[0];opt=nt+24;n=struct.unpack_from('<H',pe.data,nt+6)[0];os=struct.unpack_from('<H',pe.data,nt+20)[0];sections=[]
 for k in range(n):
  at=opt+os+40*k;vs,rva,rs,fo=struct.unpack_from('<4I',pe.data,at+8);sections.append((pe.base+rva,pe.base+rva+vs,struct.unpack_from('<I',pe.data,at+36)[0]))
 def flags(a,n):return next(f for lo,hi,f in sections if lo<=a and a+n<=hi)&0xe0000000
 assert flags(first,last-first)==0x60000000 and pe.base+struct.unpack_from('<I',pe.data,opt+16)[0]==symbols['HostMain']
 for a,b in CODE:
  at=symbols[f'Original_{a:08X}'];assert pe.read(at,b-a)==images.read(a,b-a)and flags(at,b-a)==0x40000000
 for a,n in DATA:
  at=symbols[f'Original_{a:08X}'];assert pe.read(at,n)==images.read(a,n)and flags(at,n)==0x40000000
 assert pe.read(symbols['OriginalOne'],4)==images.read(*ONE)and flags(symbols['OriginalOne'],4)==0x40000000
 ext=[]
 for name,n in BUFFERS:
  at=symbols[name+'Buffer'];assert symbols[name+'Before']==at-16 and symbols[name+'After']==at+n and flags(at-16,n+32)==0xc0000000
  for lo,size in[(at-16,n+32)]+([]if name=='Reply'else[(symbols[name+'Shadow'],n)]):assert all(lo+size<=a or b<=lo for a,b in ext);ext.append((lo,lo+size))
 for name in['SerialShadow','GenerationShadow']:
  lo=symbols[name];assert flags(lo,65536)==0xc0000000 and all(lo+65536<=a or b<=lo for a,b in ext);ext.append((lo,lo+65536))
 return dict(schema='fresh-loop-owned-linked-audit-v1',host_exe_sha256=sha(pe.data),host_map_sha256=sha(mapping.read_bytes()),owned_code_sha256=sha(pe.read(first,last-first)),source_bytes_exact=True,source_bytes=2442,entrypoint_hostmain=True,owned_control_closed=True,buffer_shadow_sections_and_extents_closed=True,native_executed=False,symbols={k:f'{v:08X}'for k,v in symbols.items()if not k.startswith('_')},imports=[dict(iat=f'{a:08X}',dll=d,name=n)for a,(d,n)in imports.items()],disassembly=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in rows])

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--source-crt',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll'));p.add_argument('--audit-exe',type=Path);p.add_argument('--audit-map',type=Path);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());images=arithmetic.Images(a.source_exe,a.source_crt);proof=arithmetic.import_export_proof(images);code=audit_source(images);asm=host_source(images);
 if a.audit_exe:
  assert a.audit_map and(a.out/'host-x86.asm').read_text()==asm;r=linked_audit(a.audit_exe,a.audit_map,images)
  with(a.out/'owned-linked-audit.json').open('x')as f:json.dump(r,f,indent=2)
  print('Ownedlinked static audit PASS; no execution');return
 a.out.mkdir(parents=True,exist_ok=False);(a.out/'host-x86.asm').write_text(asm)
 m=dict(schema='fresh-full-activation-loop-hybrid-preparation-v1',source_bytes=2442,source_sha256=arithmetic.SOURCE_SHA256,crt_sha256=arithmetic.CRT_SHA,import_export=proof,code_allowlist=code,host_source_sha256=sha((a.out/'host-x86.asm').read_bytes()),preparation_tool_sha256=sha(Path(__file__).read_bytes()),fresh_scaffold_sha256=sha(Path(fresh.__file__).read_bytes()),input_bytes=256,output_bytes=864,snapshots=9,snapshot_bytes=96,projection_bytes_per_snapshot=88,primary_base=f'{BASE:08X}',owned_compile_base=f'{OWNED_BASE:08X}',low_pages={f'{k:08X}':f'{v:08X}'for k,v in LOW_PAGES.items()},high_runtime_pages={f'{k:08X}':f'{v:08X}'for k,v in HIGH_PAGES.items()},exits={f'{k:08X}':v for k,v in EXITS.items()},unreachable_calls={f'{k:08X}':v for k,v in TRAPS.items()},compiled=False,native_executed=False,actual_carrier_required=True,limits='Fullfreshctor+fullactivation+fullclock+fullzeromasksampler+fullshift/helper/finiteCRT. Explicitbinding/unboundowner/globalisolatedsnapshots/sourceCRTdispatch; no actualcontrollercreation/liveactor/sharedresourceinterference/eventdispatch/samplingadmission.',compile_commands=[['ml.exe','/nologo','/c','/coff','/Fohost-x86.obj','host-x86.asm'],['link.exe','/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/BASE:0x27000000','/DYNAMICBASE:NO','/FIXED','/SAFESEH:NO','/OUT:host-x86.exe','/MAP:host-x86.map','host-x86.obj','kernel32.lib']])
 (a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(dict(source_bytes=2442,native_executed=False,carrier_required=True)))
if __name__=='__main__':main()

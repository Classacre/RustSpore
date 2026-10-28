"""Prepare full fresh/default/clock/nonzero-channel hybrid; never build/run."""
import argparse,json,re,struct,hashlib
from pathlib import Path
import cell_runtime_fresh_loop_native_probe as old
import cell_runtime_fresh_sampled_channel_source as source
from cell_native_math_probe import db
arithmetic=old.arithmetic
ROOT=old.ROOT;BASE=0x3F0000;OWNED_BASE=old.OWNED_BASE
APP_CODE=source.APP;CRT_CODE=source.CRT;CODE=APP_CODE+CRT_CODE
_sizes={}
for a,n in source.DATA+[(source.acos.WINDOW[0],source.acos.WINDOW[1]-source.acos.WINDOW[0])]:_sizes[a]=max(_sizes.get(a,0),n)
DATA=sorted(_sizes.items());ONE=old.ONE;EXITS=source.EXITS;TRAPS=source.TRAPS
RESOURCE_BYTES=72616;RAW_NODE=556;PREFIX=448;OUTPUT_BYTES=PREFIX+9*RAW_NODE
BUFFERS=[(n,RESOURCE_BYTES if n=='Resource'else OUTPUT_BYTES if n=='Reply'else size)for n,size in old.BUFFERS]+[('Epochs',16*46*4)]
LOW_PAGES=dict(old.LOW_PAGES);LOW_PAGES.update({0x400000:0x60000020,0x13E0000:0x40000040,0x13F0000:0x40000040,0x1440000:0x40000040,0x1470000:0x40000040})
HIGH_PAGES=old.HIGH_PAGES;IMPORTS=old.IMPORTS

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def source_cases():
 base=ROOT/'target/source-cell/runtime-fresh-loop-inputs-v1';rows=[];seen=set()
 for case in json.loads((base/'cases.json').read_text()):
  if case['clip']in seen:continue
  seen.add(case['clip']);p=base/case['clip'];raw=p.read_bytes();u=lambda a:struct.unpack_from('<I',raw,a)[0];channels=[]
  for i in range(u(324)):
   at=u(u(328)+4*i);channels.append(dict(at=at,keys=u(at+216),descriptors=u(at+224),count=u(at+212)))
  mask=bytearray(444)
  for i in range(u(channels[0]['at']+220)):
   d=channels[0]['descriptors']+32*i;kind=u(d)&15;slot=u(d+16);assert kind<4 and slot<9;offset=[0,20,100,200,240,280,320,360,400][slot];n=[20,80,100,40][kind];mask[4+offset:4+offset+n]=bytes([1])*n
  rows.append(dict(path=p,raw=raw,channels=channels,table=u(328),flags=u(280),interval=u(284),frames=u(288),mask=bytes(mask)))
 assert len(rows)==5 and max(len(r['raw'])for r in rows)==RESOURCE_BYTES;return rows

def shadow(names):
 q=[]
 for n,size in BUFFERS:
  if n in names:q +=[f'mov esi,OFFSET {n}Buffer',f'mov edi,OFFSET {n}Shadow',f'mov ecx,{size//4}','rep movsd']
 return q

def frame(low,high,args=None,extra=0xffffffff):
 q=['mov edi,OFFSET FrameBuffer','mov ecx,1024','mov eax,0A5A5A5A5h','rep stosd',f'mov FrameLow,{low}',f'mov FrameHigh,{high}',f'mov FrameExtra,{extra}']
 for at,value in(args or{}).items():q +=[f'mov dword ptr [FrameBuffer+{at}],{value}']
 return q+shadow(['Frame'])

def entry(address,sp):return[f'mov esp,OFFSET FrameBuffer+{sp}',f'push 0{address:X}h','ret']
def returned(label,sp):return[label+'::',f'cmp esp,OFFSET FrameBuffer+{sp}','jne HostFailure','mov esp,HelperStack','pushfd','pop edx','test edx,0400h','jnz HostFailure','cld']

def host_source(images):
 cases=source_cases();saved={n:getattr(old,n)for n in['BASE','APP_CODE','CRT_CODE','CODE','DATA','EXITS','TRAPS','BUFFERS','LOW_PAGES']}
 try:
  for n in saved:setattr(old,n,globals()[n])
  text=old.host_source(images)
 finally:
  for n,v in saved.items():setattr(old,n,v)
 # Correct only owned IO sizes and source mask/domain declarations.
 text=text.replace('Expected BYTE 864 DUP(0)',f'Expected BYTE {OUTPUT_BYTES} DUP(0)')
 text=text.replace('push 864',f'push {OUTPUT_BYTES}').replace('cmp ecx,864',f'cmp ecx,{OUTPUT_BYTES}')
 text=text.replace('mov ecx,864',f'mov ecx,{OUTPUT_BYTES}').replace('mov ebx,864',f'mov ebx,{OUTPUT_BYTES}')
 text=text.replace('mov ecx,216',f'mov ecx,{OUTPUT_BYTES//4}')
 a=text.index('ValidateRecord PROC');b=text.index('ValidateRecord ENDP',a)
 part=text[a:b]
 for at in[44,52]:part=part.replace(f'cmp dword ptr [InputBuffer+{at}],0',f'cmp dword ptr [InputBuffer+{at}],1')
 part=part.replace('mov esi,OFFSET InputBuffer+72\nmov ecx,14','cmp dword ptr [InputBuffer+16],16\njne HostFailure\ncmp dword ptr [InputBuffer+72],4\nja HostFailure\ncmp dword ptr [InputBuffer+80],1\njne HostFailure\nmov esi,OFFSET InputBuffer+84\nmov ecx,11')
 part=part.replace('ret\n','call ValidateSourceCase\nret\n');text=text[:a]+part+text[b:]
 # RealXMM poison input is explicit; it is not inherited as a claim.
 poison=['movd xmm0,Poison','pshufd xmm0,xmm0,0']+[f'movaps xmm{i},xmm0'for i in range(1,8)]
 text=text.replace('RunOnce PROC\n','RunOnce PROC\n'+'\n'.join(poison)+'\n',1)
 # Preserve original fixed table and policy bytes while individually seeding globals.
 reset=[]
 for a,n in DATA:
  if a&~65535==0x1550000:reset +=[f'mov esi,OFFSET Original_{a:08X}',f'mov edi,0{a:X}h',f'mov ecx,{n}','rep movsb']
 reset +=['mov eax,dword ptr [InputBuffer+76]','mov dword ptr [015514DCh],eax','mov ConstructorCount,0','mov WrapperMode,0']
 text=text.replace('mov esi,01550000h\nmov edi,OFFSET SerialShadow','\n'.join(reset)+'\nmov esi,01550000h\nmov edi,OFFSET SerialShadow',1)
 # Borrowed selected resource layout is copied from immutable exactsource bytes.
 insert='mov dword ptr [ClipBuffer],OFFSET ResourceBuffer'
 text=text.replace(insert,'call InitializeResource\n'+insert,1)
 text=text.replace('mov ClipMask,OFFSET MaskDuration','mov ClipMask,OFFSET MaskNone\ncall InitializeWrappers\nmov WrapperMode,4\nmov esi,OFFSET InputBuffer+44\nmov edi,OFFSET ContextBuffer\nmov ecx,4\nrep movsd\n'+'\n'.join(shadow(['Wrappers','Epochs','Context','Clip']))+'\ncall CaptureConstructor\nmov ClipMask,OFFSET MaskDuration',1)
 # Sourceframe feeds original sampler within the fullclock, never an injected request.
 text=text.replace('mov FrameLow,2064','mov FrameLow,1024').replace('mov FrameLow,1984','mov FrameLow,1024')
 text=text.replace('fstp tbyte ptr [ReplyBuffer]','fstp tbyte ptr [ReplyBuffer+448]')
 text=text.replace('imul eax,96\nadd eax,OFFSET ReplyBuffer','imul eax,556\nadd eax,OFFSET ReplyBuffer+448')
 # Guard payload's legalfullslot union; independentRust verifies all partialwrites.
 a=text.index('GuardCurrent PROC');b=text.index('GuardCurrent ENDP',a)
 guard=text[a:b].replace('cmp ecx,0150h',f'cmp ecx,{RESOURCE_BYTES}')
 for n,size in[('Events',128),('Wrappers',7936)]:
  pattern='\n'.join([f'mov esi,OFFSET {n}Buffer',f'mov edi,OFFSET {n}Shadow',f'mov ecx,{size//4}','repe cmpsd','jne HostFailure'])
  assert pattern in guard;guard=guard.replace(pattern,f'call Guard{n}',1)
 guard=guard.replace('cmp ecx,5344\nje NextSerialPage','cmp ecx,5340\nje NextSerialPage\ncmp ecx,5344\nje NextSerialPage')
 guard+='call GuardEpochs\nmov eax,dword ptr [InputBuffer+76]\nadd eax,ConstructorCount\ncmp eax,dword ptr [015514DCh]\njne HostFailure\n'
 # Append the extra guard before the existing RET, not after a returning path.
 guard=guard.replace('ret\ncall GuardEpochs','call GuardEpochs');guard+='ret\n'
 text=text[:a]+guard+text[b:]
 # Replace snapshot builder with raw payload/events, context at548.
 a=text.index('CaptureSnapshot PROC');b=text.index('CaptureSnapshot ENDP',a)+len('CaptureSnapshot ENDP')
 lines=['CaptureSnapshot PROC','mov eax,SnapshotIndex','imul eax,556','add eax,OFFSET ReplyBuffer+448','mov edi,eax','mov word ptr [edi+10],0','mov eax,ClockBase','mov edx,dword ptr [eax+018h]','mov dword ptr [edi+12],edx','mov edx,dword ptr [eax-01Ch]','mov dword ptr [edi+16],edx','mov edx,dword ptr [eax+01Ch]','mov dword ptr [edi+20],edx']
 for dst,src in[(24,0xB0),(28,0xB4),(32,0xC0),(36,0xC4),(48,0x10),(80,0xB8),(84,0xBC)]:lines +=[f'mov eax,dword ptr [ClipBuffer+{src}]',f'mov dword ptr [edi+{dst}],eax']
 for dst,src in[(40,'ResourceBuffer+012Ch'),(44,'0166C240h'),(52,'015514E0h'),(76,'ResourceBuffer+0120h')]:lines +=[f'mov eax,dword ptr [{src}]',f'mov dword ptr [edi+{dst}],eax']
 lines +=['mov eax,dword ptr [ClipBuffer+0CCh]','sub eax,OFFSET EventsBuffer','shr eax,3','cmp eax,2','ja HostFailure','mov dword ptr [edi+56],eax','mov dword ptr [edi+528],eax','mov dword ptr [edi+60],16','mov dword ptr [edi+64],1','mov dword ptr [edi+68],1','mov dword ptr [edi+72],0','push edi','add edi,88','mov esi,OFFSET WrappersBuffer+4','mov ecx,110','rep movsd','pop edi','push edi','add edi,532','mov esi,OFFSET EventsBuffer','mov ecx,4','rep movsd','pop edi','fnstcw word ptr [edi+548]','fnstsw word ptr [edi+550]','stmxcsr dword ptr [edi+552]','mov ax,word ptr [edi+548]','cmp ax,word ptr [InputBuffer+4]','jne HostFailure','test word ptr [edi+550],03800h','jnz HostFailure','mov eax,dword ptr [edi+552]','and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure','ret','CaptureSnapshot ENDP']
 text=text[:a]+'\n'.join(lines)+text[b:]
 extra=procedures(cases)
 text=text.replace('RunOnce PROC\n','\n'.join(extra)+'\nRunOnce PROC\n',1)
 # Validate each readonly importbinding before any ReadRecord.
 startup=['cmp dword ptr [013CC478h],OFFSET MemsetProvider','jne HostFailure','cmp dword ptr [013CC47Ch],OFFSET MemcpyProvider','jne HostFailure','cmp dword ptr [013CC48Ch],0785A35E0h','jne HostFailure']
 text=text.replace('push -10\ncall _GetStdHandle@4','\n'.join(startup)+'\npush -10\ncall _GetStdHandle@4',1)
 # Rdata caseimages/masks, RW own control fields; all declarations short.
 declarations=['PUBLIC '+x for x in['MaskNone','CurrentWrapper','HelperStack','WrapperMode','CurrentWrapperIndex','CurrentKeyCount','ConstructorCount','CaseMask','MemsetProvider','MemcpyProvider','WrapperCtorDone','BorrowedChannelInstalled','EpochAndCacheInitialized','CaptureConstructor','GuardWrappers','GuardEvents','GuardEpochs','ValidateSourceCase','InitializeResource','InitializeWrappers']]
 const=['MaskNone BYTE 224 DUP(0)']
 for i,c in enumerate(cases):const +=[f'PUBLIC SourceCase{i},SourceMask{i}',*db(f'SourceCase{i}',c['raw']),*db(f'SourceMask{i}',c['mask'])]
 pos=text.index('.data\n');text=text[:pos]+'\n'.join(const)+'\n'+text[pos:]
 fields=['WrapperMode DWORD 0','CurrentWrapperIndex DWORD 0','CurrentKeyCount DWORD 0','ConstructorCount DWORD 0','CaseMask DWORD 0','CurrentWrapper DWORD 0','HelperStack DWORD 0']
 text=text.replace('.code\n','\n'.join(fields+declarations)+'\n.code\n',1)
 text='\n'.join('PUBLIC '+s.strip()if line.startswith('PUBLIC ')else line for line in text.splitlines()for s in(line[7:].split(',')if line.startswith('PUBLIC ')else[line]))+'\n'
 assert max(map(len,text.splitlines()))<512
 return text

def procedures(cases):
 q=['ValidateSourceCase PROC','mov eax,dword ptr [InputBuffer+72]']
 for i in range(5):q +=[f'cmp eax,{i}',f'je ValidateCase{i}']
 q+=['jmp HostFailure']
 for i,c in enumerate(cases):
  q +=[f'ValidateCase{i}:']
  for at,value in[(24,c['flags']),(28,c['interval']),(32,c['frames']),(36,len(c['channels']))]:q +=[f'cmp dword ptr [InputBuffer+{at}],0{value:X}h','jne HostFailure']
  q+=['ret']
 q+=['ValidateSourceCase ENDP','InitializeResource PROC','mov eax,dword ptr [InputBuffer+72]']
 for i in range(5):q +=[f'cmp eax,{i}',f'je InitializeCase{i}']
 q+=['jmp HostFailure']
 for i,c in enumerate(cases):
  q +=[f'InitializeCase{i}:',f'mov esi,OFFSET SourceCase{i}','mov edi,OFFSET ResourceBuffer',f'mov ecx,{len(c["raw"])//4}','rep movsd',f'mov CaseMask,OFFSET SourceMask{i}',f'mov dword ptr [ResourceBuffer+0148h],OFFSET ResourceBuffer+{c["table"]}']
  for k,ch in enumerate(c['channels']):
   for off,value in[(c['table']+k*4,f'OFFSET ResourceBuffer+{ch["at"]}'),(ch['at']+4,'OFFSET ResourceBuffer'),(ch['at']+216,f'OFFSET ResourceBuffer+{ch["keys"]}'),(ch['at']+224,f'OFFSET ResourceBuffer+{ch["descriptors"]}')]:q +=[f'mov dword ptr [ResourceBuffer+{off}],{value}']
  q+=['ret']
 q+=['InitializeResource ENDP','CaptureConstructor PROC','mov esi,OFFSET WrappersBuffer+4','mov edi,OFFSET ReplyBuffer','mov ecx,110','rep movsd','mov eax,dword ptr [015514DCh]','mov dword ptr [ReplyBuffer+440],eax','mov eax,dword ptr [WrappersBuffer+01BCh]','mov dword ptr [ReplyBuffer+444],eax','ret','CaptureConstructor ENDP','InitializeWrappers PROC','mov HelperStack,esp','mov CurrentWrapperIndex,0','InitializeWrapperLoop:','mov eax,CurrentWrapperIndex','imul eax,01F0h','add eax,OFFSET WrappersBuffer','mov CurrentWrapper,eax','mov WrapperMode,1']
 q+=frame(0x700,0x804,{0x800:'OFFSET WrapperCtorDone'})+shadow(['Wrappers','Epochs'])+['mov ecx,CurrentWrapper']+entry(0x9A1450,0x800)+returned('WrapperCtorDone',0x804)
 q +=['cmp eax,CurrentWrapper','jne HostFailure','inc ConstructorCount','call GuardCurrent','mov WrapperMode,2']
 q+=frame(0x7fc,0x800,{0x820:'0',0x824:'0'})+['mov eax,CurrentWrapperIndex','mov dword ptr [FrameBuffer+0820h],eax','imul eax,01F0h','mov dword ptr [FrameBuffer+0824h],eax']+shadow(['Frame','Wrappers'])+['mov ebx,OFFSET ClipBuffer']+entry(0x9A3214,0x800)+returned('BorrowedChannelInstalled',0x800)
 q+=['mov CurrentKeyCount,ecx','cmp ecx,46','ja HostFailure','test ecx,ecx','jz HostFailure','call GuardCurrent','mov eax,CurrentWrapperIndex','imul eax,184','add eax,OFFSET EpochsBuffer','mov edx,CurrentWrapper','mov dword ptr [edx+01DCh],eax','mov ecx,CurrentKeyCount','lea ecx,[eax+ecx*4]','mov dword ptr [edx+01E0h],ecx','add eax,184','mov dword ptr [edx+01E4h],eax','mov WrapperMode,3']
 q+=frame(0x7fc,0x800,{0x820:'0',0x824:'0'},0x824)+['mov eax,CurrentWrapperIndex','mov dword ptr [FrameBuffer+0820h],eax','imul eax,01F0h','mov dword ptr [FrameBuffer+0824h],eax']+shadow(['Frame','Wrappers','Epochs'])+['mov edi,CurrentWrapper','lea ebp,[edi+01DCh]']+entry(0x9A328F,0x800)+returned('EpochAndCacheInitialized',0x800)
 q+=['call GuardCurrent','mov edx,CurrentWrapper','cmp dword ptr [edx+01BCh],0','jne HostFailure','inc CurrentWrapperIndex','mov eax,CurrentWrapperIndex','cmp eax,dword ptr [InputBuffer+36]','jb InitializeWrapperLoop','mov WrapperMode,0','ret','InitializeWrappers ENDP']
 q+=memory_providers()+guards();return q

def memory_providers():
 return ['MemsetProvider::','cmp WrapperMode,1','jne HostFailure','cmp esp,OFFSET FrameBuffer+07D8h','jne HostFailure','cmp dword ptr [esp],0099EAC9h','jne HostFailure','mov eax,CurrentWrapper','add eax,4','cmp dword ptr [esp+4],eax','jne HostFailure','cmp dword ptr [esp+8],0','jne HostFailure','cmp dword ptr [esp+12],440','jne HostFailure','push edi','push ecx','mov edi,eax','xor eax,eax','mov ecx,110','rep stosd','mov eax,dword ptr [esp+12]','pop ecx','pop edi','ret',
 'MemcpyProvider::','cmp dword ptr [esp+12],0','je MemcpyZero','cmp WrapperMode,4','jne HostFailure','mov eax,ClockBase','sub eax,02E8h','cmp esp,eax','jne HostFailure','cmp dword ptr [esp],0099F685h','jne HostFailure','mov eax,dword ptr [esp+12]','cmp eax,20','je MemcpySize','cmp eax,40','je MemcpySize','cmp eax,80','je MemcpySize','cmp eax,100','jne HostFailure','MemcpySize:','mov edx,dword ptr [esp+4]','cmp edx,OFFSET WrappersBuffer+4','jb HostFailure','add eax,edx','jc HostFailure','cmp eax,OFFSET WrappersBuffer+444','ja HostFailure','mov edx,dword ptr [esp+8]','cmp edx,OFFSET ResourceBuffer','jb HostFailure','mov eax,dword ptr [esp+12]','add eax,edx','jc HostFailure',f'cmp eax,OFFSET ResourceBuffer+{RESOURCE_BYTES}','ja HostFailure','push esi','push edi','push ecx','mov edi,dword ptr [esp+16]','mov esi,dword ptr [esp+20]','mov ecx,dword ptr [esp+24]','rep movsb','mov eax,dword ptr [esp+16]','pop ecx','pop edi','pop esi','ret','MemcpyZero:','cmp WrapperMode,1','jne HostFailure','cmp esp,OFFSET FrameBuffer+07DCh','jne HostFailure','cmp dword ptr [esp],009A0B96h','jne HostFailure','cmp dword ptr [esp+4],0','jne HostFailure','cmp dword ptr [esp+8],0','jne HostFailure','xor eax,eax','ret']

def guards():
 q=['GuardEvents PROC','mov edx,dword ptr [ClipBuffer+0CCh]','sub edx,OFFSET EventsBuffer','cmp edx,16','ja HostFailure','test edx,7','jnz HostFailure','xor ecx,ecx','GuardEventByte:','cmp ecx,edx','jb NextEventByte','mov al,byte ptr [EventsBuffer+ecx]','cmp al,byte ptr [EventsShadow+ecx]','jne HostFailure','NextEventByte:','inc ecx','cmp ecx,128','jb GuardEventByte','ret','GuardEvents ENDP',
 'GuardWrappers PROC','xor ecx,ecx','GuardWrapperByte:','mov eax,ecx','xor edx,edx','mov ebx,496','div ebx','cmp WrapperMode,4','je GuardSamplingByte','cmp eax,CurrentWrapperIndex','jne CompareWrapperByte','cmp WrapperMode,1','je GuardCtorByte','cmp WrapperMode,2','je GuardInstallByte','cmp WrapperMode,3','jne CompareWrapperByte','cmp edx,444','jb CompareWrapperByte','cmp edx,448','jb NextWrapperByte','jmp CompareWrapperByte','GuardInstallByte:','cmp edx,4','jb NextWrapperByte','jmp CompareWrapperByte','GuardCtorByte:','cmp edx,444','jb NextWrapperByte','cmp edx,448','jb CompareWrapperByte','cmp edx,488','jb NextWrapperByte','jmp CompareWrapperByte','GuardSamplingByte:','test eax,eax','jnz CompareWrapperByte','cmp edx,444','jb CheckSampleMask','cmp edx,448','jb NextWrapperByte','jmp CompareWrapperByte','CheckSampleMask:','mov eax,CaseMask','cmp byte ptr [eax+edx],0','jne NextWrapperByte','CompareWrapperByte:','mov al,byte ptr [WrappersBuffer+ecx]','cmp al,byte ptr [WrappersShadow+ecx]','jne HostFailure','NextWrapperByte:','inc ecx','cmp ecx,7936','jb GuardWrapperByte','ret','GuardWrappers ENDP',
 'GuardEpochs PROC','mov edx,0FFFFFFFFh','mov ebx,0FFFFFFFFh','cmp WrapperMode,3','jne GuardEpochReady','mov edx,CurrentWrapperIndex','imul edx,184','mov ebx,CurrentKeyCount','shl ebx,2','add ebx,edx','GuardEpochReady:','xor ecx,ecx','GuardEpochByte:','cmp ecx,edx','jb CompareEpochByte','cmp ecx,ebx','jb NextEpochByte','CompareEpochByte:','mov al,byte ptr [EpochsBuffer+ecx]','cmp al,byte ptr [EpochsShadow+ecx]','jne HostFailure','NextEpochByte:','inc ecx','cmp ecx,2944','jb GuardEpochByte','ret','GuardEpochs ENDP']
 return q

def audit_source(images):
 return source.audit(images)[0]

def linked_audit(exe,mapping,images):
 saved={n:getattr(old,n)for n in['CODE','DATA','BUFFERS']}
 try:
  for n in saved:setattr(old,n,globals()[n])
  report=old.linked_audit(exe,mapping,images)
 finally:
  for n,v in saved.items():setattr(old,n,v)
 from anim_root_pose_sparse_carrier_audit_v2 import Image
 view=Image(exe);pe=old.PE(exe);symbols={k:int(v,16)for k,v in report['symbols'].items()}
 for i,c in enumerate(source_cases()):
  assert pe.read(symbols[f'SourceCase{i}'],len(c['raw']))==c['raw'] and view.perm(symbols[f'SourceCase{i}'],len(c['raw']))==0x40000000
  assert pe.read(symbols[f'SourceMask{i}'],len(c['mask']))==c['mask'] and view.perm(symbols[f'SourceMask{i}'],len(c['mask']))==0x40000000
 report.update(schema='fresh-source-default-full-channel-owned-linked-v1',source_bytes=10403,game_bytes=8979,crt_bytes=1424,exact_source_cases=True,source_case_hashes=[sha(c['path'])for c in source_cases()],source_callback_CFG_only=True,fullbinder_claim=False)
 return report

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--audit-exe',type=Path);p.add_argument('--audit-map',type=Path);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());images=old.arithmetic.Images(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll'));source.audit(images);source.imports(images);asm=host_source(images);
 if a.audit_exe:
  assert a.audit_map and(a.out/'host-x86.asm').read_text()==asm;r=linked_audit(a.audit_exe,a.audit_map,images)
  with(a.out/'owned-linked-audit.json').open('x')as f:json.dump(r,f,indent=2)
  print('static linked PASS');return
 a.out.mkdir(parents=True,exist_ok=False);(a.out/'host-x86.asm').write_text(asm)
 m=dict(schema='fresh-source-default-activation-whole-sampler-hybrid-prep-v1',source_bytes=10403,game_bytes=8979,crt_bytes=1424,source_sha256=old.arithmetic.SOURCE_SHA256,crt_sha256=old.arithmetic.CRT_SHA,host_source_sha256=sha(a.out/'host-x86.asm'),preparation_tool_sha256=sha(__file__),input_bytes=256,output_bytes=OUTPUT_BYTES,prefix_bytes=PREFIX,snapshots=9,raw_snapshot_bytes=RAW_NODE,defined_projection_snapshot_bytes=544,base=f'{BASE:08X}',owned_base=f'{OWNED_BASE:08X}',low_pages={f'{a:08X}':f'{v:08X}'for a,v in LOW_PAGES.items()},high_pages={f'{a:08X}':f'{v:08X}'for a,v in HIGH_PAGES.items()},actual_counts=[len(c['channels'])for c in source_cases()],compiled=False,native_executed=False,carrier_required=True,scope='Fresh source-defined wrapper payload and epoch/cache initialization, privateactivation/fullclock, genuinefirstchannelnonzero masks/fullsampler/events. Preallocatedbuffer binding partial entries; no fullbinder/lifetime/actor/eventdispatch admission.')
 (a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(m['host_source_sha256'])
if __name__=='__main__':main()

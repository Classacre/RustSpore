"""Prepare fixed-fixture bounded 9E7C80 mode-2 host and sparse PE; never execute."""
import argparse,hashlib,json,struct
from pathlib import Path
import anim_root_pose_sparse_carrier_v3 as carrier
from cell_native_math_probe import db
from cell_schema_probe import SourceImage
ROOT=Path(__file__).resolve().parent.parent
FIX=ROOT/'target/animation/cid-child-mode2-fixtures-v1'
AUDIT=ROOT/'target/animation/cid-child-mode2-source-audit-v1.json'
CONTRACT=ROOT/'target/animation/cid-child-mode2-host-contract-v1.json'
BASE=0x670000
CODE=[(0x9e7c80,0x9e7cc3),(0x9e7cd0,0x9e82d6),(0x99be70,0x99bfd1)]
DATA=[0x13eb1bc,0x13eb8b0,0x14719e4]
SIZES={'Input':4,'CID':116,'ParentDef':1128,'ChildDef':1128,'Parent':700,'Children':255*700,'Reply':60,'Stack':65536}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def fixture_bytes():
 m=json.loads((FIX/'manifest.json').read_text());assert sha(FIX/'fixtures.json')==m['fixtures_sha256'];f=json.loads((FIX/'fixtures.json').read_text());assert len(f['fixtures'])==64 and f['synthetic_stage_state']
 out=bytearray()
 for row in f['fixtures']:
  s=row['synthetic_stage'];n=row['source_child_count'];assert 1<=n<=255 and s['flags_1ac_bytes']==[0,0,255,255] and s['mode_238']==2 and s['other_child_flags_low3']==4 and s['cid_scale_bits']==0x3f800000
  assert not any(s['initial_position_bits']+s['initial_quaternion_bits']+s['initial_counters'])
  vals=[n]+row['parent_definition_344_bits']+row['child_definition_128_bits']+row['child_definition_350_bits']+s['child_quaternion_19c_bits']+s['child_position_1c8_bits'];assert len(vals)==18
  assert all((v&0x7f800000)!=0x7f800000 for v in vals[1:]);out+=struct.pack('<18I',*vals)
 return bytes(out)
def proof(image):
 a=json.loads(AUDIT.read_text());assert a['source_sha256']==hashlib.sha256(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe').read_bytes()).hexdigest()
 records={int(r['start'],16):r for group in a['source_spans'] for r in group}
 for x,y in CODE:assert hashlib.sha256(image.read(x,y-x)).hexdigest()==records[x]['sha256']
 data=[dict(address=f'{x:08X}',bytes=4)for x in DATA]
 for x in DATA:assert image.read(x,4).hex()==a['absolute_constants'][f'{x:08X}']
 assert all(y<=BASE or x>=BASE+4096 for x,y in CODE) and all(x>=BASE+4096 or x+4<=BASE for x in DATA)
 return dict(data=data)
def pages(image,audit,done):
 pages={BASE:bytearray(65536),0x990000:bytearray(b'\xcc'*65536),0x9e0000:bytearray(b'\xcc'*65536),0x13e0000:bytearray(65536),0x1470000:bytearray(65536)}
 for x,y in CODE:pages[x&~65535][x&65535:(x&65535)+y-x]=image.read(x,y-x)
 for x in DATA:pages[x&~65535][x&65535:(x&65535)+4]=image.read(x,4)
 assert pages[0x9e0000][0x6890]==0xcc
 return[(x,bytes(b),0x60000020 if x in (0x990000,0x9e0000) else 0x40000040)for x,b in sorted(pages.items())]
def host(image,fixtures):
 q=['.686p','.xmm','.model flat','option casemap:none']
 q+=['EXTERN _'+n+':PROC' for n in ['GetModuleHandleA@4','GetStdHandle@4','ReadFile@20','WriteFile@20','GetLastError@0','ExitProcess@4']]
 q+=['.const','PUBLIC Fixtures']+db('Fixtures',fixtures)
 for x,y in CODE:q+=['PUBLIC Original_%08X'%x]+db('Original_%08X'%x,image.read(x,y-x))
 for x in DATA:q+=['PUBLIC Data_%08X'%x]+db('Data_%08X'%x,image.read(x,4))
 q+=['.data','NativeStack DWORD 0','RequestedCSR DWORD 01F80h','RequestedCW WORD 027Fh','Poison DWORD 0','InputHandle DWORD 0','OutputHandle DWORD 0','Transferred DWORD 0','Remaining DWORD 0','Cursor DWORD 0','Expected BYTE 60 DUP(0)','PUBLIC NativeStack,Poison,Expected']
 for name,size in SIZES.items():q += [f'PUBLIC {name}Before,{name}Buffer,{name}After,{name}Shadow',f'{name}Before DWORD 4 DUP(0A5A5A5A5h)',f'{name}Buffer BYTE {size} DUP(0)',f'{name}After DWORD 4 DUP(0A5A5A5A5h)',f'{name}Shadow BYTE {size} DUP(0)']
 q+=['.code','PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,OriginalDone,RunOnce','OwnedStart LABEL BYTE','RunOnce PROC','mov NativeStack,esp','cld','cmp dword ptr [InputBuffer],64','jae HostFailure']
 for name,size in SIZES.items():
  if name=='Input':continue
  q += [f'mov edi,OFFSET {name}Buffer',f'mov ecx,{size}','mov eax,0A5h','rep stosb']
 q+=['mov eax,dword ptr [InputBuffer]','imul eax,eax,72','lea esi,[Fixtures+eax]','mov eax,dword ptr [esi]','cmp eax,1','jb HostFailure','cmp eax,255','ja HostFailure','mov dword ptr [ParentDefBuffer+208h],eax','mov dword ptr [CIDBuffer+70h],03F800000h','mov dword ptr [ParentBuffer],OFFSET ParentDefBuffer','mov dword ptr [ParentBuffer+0Ch],OFFSET ChildrenBuffer','mov dword ptr [ChildrenBuffer],OFFSET ChildDefBuffer',
 'lea edi,[ParentDefBuffer+344h]','lea esi,[esi+4]','mov ecx,12','rep movsb','lea edi,[ChildDefBuffer+128h]','mov ecx,16','rep movsb','lea edi,[ChildDefBuffer+350h]','mov ecx,12','rep movsb','lea edi,[ChildrenBuffer+19Ch]','mov ecx,16','rep movsb','lea edi,[ChildrenBuffer+1C8h]','mov ecx,12','rep movsb',
 'mov dword ptr [ChildrenBuffer+1ACh],0FFFF0000h','mov dword ptr [ChildrenBuffer+238h],2','mov ecx,1','SkipInit:','cmp ecx,dword ptr [ParentDefBuffer+208h]','jae InitDone','imul eax,ecx,700','mov byte ptr [ChildrenBuffer+eax+1ACh],4','inc ecx','jmp SkipInit','InitDone:',
 'mov edi,OFFSET ReplyBuffer','mov ecx,60','xor eax,eax','rep stosb','mov dword ptr [ReplyBuffer+40],0FFFF0004h']
 for name,size in SIZES.items():
  if name in ('Reply','Stack'):continue
  q += [f'mov esi,OFFSET {name}Buffer',f'mov edi,OFFSET {name}Shadow',f'mov ecx,{size}','rep movsb']
 q+=['fninit','fldcw RequestedCW','ldmxcsr RequestedCSR','movd xmm0,Poison','pshufd xmm0,xmm0,0']+[f'movaps xmm{i},xmm0' for i in range(1,8)]
 q+=['mov esp,OFFSET StackBuffer+65536','push OFFSET ReplyBuffer+40','push OFFSET ReplyBuffer+24','push OFFSET ReplyBuffer+12','push OFFSET ReplyBuffer+8','push OFFSET ReplyBuffer+4','push OFFSET ReplyBuffer','push OFFSET ParentBuffer','push OFFSET CIDBuffer','mov eax,009E7C80h','call eax','add esp,32','cmp esp,OFFSET StackBuffer+65536','jne HostFailure','mov esp,NativeStack','pushfd','pop eax','test eax,0400h','jnz HostFailure','cld','fnstcw word ptr [ReplyBuffer+52]','fnstsw word ptr [ReplyBuffer+54]','stmxcsr dword ptr [ReplyBuffer+56]','cmp word ptr [ReplyBuffer+52],027Fh','jne HostFailure','test word ptr [ReplyBuffer+54],03800h','jnz HostFailure','mov eax,dword ptr [ReplyBuffer+56]','and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure','cmp dword ptr [ReplyBuffer+40],0FFFF0004h','jne HostFailure']
 for name in ('Input','CID','ParentDef','ChildDef','Parent'):
  size=SIZES[name];q += [f'mov esi,OFFSET {name}Buffer',f'mov edi,OFFSET {name}Shadow',f'mov ecx,{size}','repe cmpsb','jne HostFailure']
 q+=['xor ecx,ecx','NodeGuard:','cmp ecx,01ACh','jb NodeCompare','cmp ecx,01B0h','jb NodeNext','cmp ecx,0210h','jb NodeCompare','cmp ecx,0214h','jb NodeNext','NodeCompare:','mov al,byte ptr [ChildrenBuffer+ecx]','cmp al,byte ptr [ChildrenShadow+ecx]','jne HostFailure','NodeNext:','inc ecx',f'cmp ecx,{SIZES["Children"]}','jb NodeGuard']
 for name in SIZES:
  for side in ('Before','After'):
   for at in (0,4,8,12):q += [f'cmp dword ptr [{name}{side}+{at}],0A5A5A5A5h','jne HostFailure']
 q+=['cmp dword ptr [ReplyBuffer],1','jne HostFailure','cmp dword ptr [ReplyBuffer+4],1','jne HostFailure','cmp dword ptr [ReplyBuffer+8],1','jne HostFailure','mov eax,dword ptr [ChildrenBuffer+1ACh]','mov dword ptr [ReplyBuffer+44],eax','cmp eax,0FFFF0004h','jne HostFailure','mov eax,dword ptr [ChildrenBuffer+210h]','mov dword ptr [ReplyBuffer+48],eax','cmp eax,0BF800000h','jne HostFailure','ret','RunOnce ENDP',
 'HostMain PROC','cld','push 0','call _GetModuleHandleA@4',f'cmp eax,0{BASE:X}h','jne HostFailure']
 for x,y in CODE:q += [f'mov esi,OFFSET Original_{x:08X}',f'mov edi,0{x:08X}h',f'mov ecx,{y-x}','repe cmpsb','jne HostFailure']
 for x in DATA:q += [f'mov esi,OFFSET Data_{x:08X}',f'mov edi,0{x:08X}h','mov ecx,4','repe cmpsb','jne HostFailure']
 q+=['cmp byte ptr [009E6890h],0CCh','jne HostFailure','push -10','call _GetStdHandle@4','mov InputHandle,eax','push -11','call _GetStdHandle@4','mov OutputHandle,eax','NextCase:','mov Remaining,4','mov Cursor,OFFSET InputBuffer','ReadAgain:','push 0','push OFFSET Transferred','push Remaining','push Cursor','push InputHandle','call _ReadFile@20','test eax,eax','jnz ReadOK','call _GetLastError@0','cmp eax,109','jne HostFailure','mov Transferred,0','ReadOK:','mov eax,Transferred','test eax,eax','jnz ReadBytes','cmp Remaining,4','jne HostFailure','push 0','call _ExitProcess@4','ReadBytes:','cmp eax,Remaining','ja HostFailure','sub Remaining,eax','add Cursor,eax','cmp Remaining,0','jne ReadAgain','mov Poison,0','call RunOnce','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected','mov ecx,60','rep movsb','mov Poison,-1','call RunOnce','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected','mov ecx,52','repe cmpsb','jne HostFailure','mov Remaining,60','mov Cursor,OFFSET ReplyBuffer','WriteAgain:','push 0','push OFFSET Transferred','push Remaining','push Cursor','push OutputHandle','call _WriteFile@20','test eax,eax','jz HostFailure','mov eax,Transferred','test eax,eax','jz HostFailure','cmp eax,Remaining','ja HostFailure','sub Remaining,eax','add Cursor,eax','cmp Remaining,0','jne WriteAgain','jmp NextCase','HostFailure::','cmp NativeStack,0','je FailureExit','mov esp,NativeStack','FailureExit:','push 71','call _ExitProcess@4','HostMain ENDP','OriginalDone LABEL BYTE','OwnedEnd LABEL BYTE','END HostMain']
 return '\n'.join(q)+'\n'
def build(exe,mapping,image):
 old=(carrier.BASE,carrier.source_pages)
 try:carrier.BASE,carrier.source_pages=BASE,pages;return carrier.build(exe,mapping,image,proof(image))
 finally:carrier.BASE,carrier.source_pages=old
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--host-dir',type=Path);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());image=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));fixture=fixture_bytes();audit=proof(image)
 if a.host_dir:
  assert (a.host_dir/'host-x86.asm').read_text()==host(image,fixture);blob,review=build(a.host_dir/'host-x86.exe',a.host_dir/'host-x86.map',image);a.out.mkdir(parents=True,exist_ok=False);(a.out/'carrier-x86.exe').write_bytes(blob);review.update(native_executed=False,execution_admitted=False,carrier_sha256=sha(a.out/'carrier-x86.exe'));(a.out/'carrier-review.json').write_text(json.dumps(review,indent=2)+'\n');return
 a.out.mkdir(parents=True,exist_ok=False);(a.out/'host-x86.asm').write_text(host(image,fixture));(a.out/'manifest.json').write_text(json.dumps(dict(schema='cid-child-mode2-host-v1',tool_sha256=sha(__file__),asm_sha256=sha(a.out/'host-x86.asm'),fixture_sha256=hashlib.sha256(fixture).hexdigest(),source_audit=audit,host_contract_sha256=sha(CONTRACT),fixtures=64,input='little-endian u32 case ordinal; EOF only at boundary',reply='60 bytes counters,position,quaternion,aggregate flags,child flags,child +210,CW,SW,MXCSR',synthetic_stage=True,native_executed=False,execution_admitted=False,compile_commands=[['ml.exe','/nologo','/c','/coff','/Fohost-x86.obj','host-x86.asm'],['link.exe','/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/BASE:0x29000000','/FIXED','/DYNAMICBASE:NO','/SAFESEH:NO','/OUT:host-x86.exe','/MAP:host-x86.map','host-x86.obj','kernel32.lib']],limits='Independent actual-linked audit, runner review, compiler and execution all pending.'),indent=2)+'\n')
if __name__=='__main__':main()

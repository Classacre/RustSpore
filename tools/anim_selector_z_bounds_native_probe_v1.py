"""Prepare full original9B6920 + measured9C48DB..4992 field publisher.
No compile/link/native execution. Real owned stack, exact retained80 exit.
"""
import argparse,json,struct,hashlib,re
from pathlib import Path
import anim_root_pose_sparse_carrier_v3 as carrier
from cell_schema_probe import SourceImage
from native_crt_acos_probe import decoder
from cell_native_math_probe import db
from anim_body_finalizer_native_probe import FIELDS
ROOT=Path(__file__).resolve().parent.parent
BASE=0x990000
SIZES={'Header':32,'Payload':82668,'Definition':1024,'Blocks':287640,'State':16,'Reply':52}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def proof(image,out):
 records=[];constant={}
 engine=decoder();from capstone.x86 import X86_OP_MEM
 for number,entry,ranges,partial in[(67,0x9b6920,[(0x9b6920,0x9b699d),(0x9b69a0,0x9b75b0)],False),(None,0x9c48db,[(0x9c48db,0x9c4992)],True)]:
  path=ROOT/(f'target/rea/20261027-source-runtime-v4/{number}-analyze_function.json'if number else'target/rea/20261022-anim-rig-focused-v1/57-analyze_function.json');e=json.loads(json.loads(path.read_text())['content'][0]['text']);v=e['raw_result']['value'];assert int(v['procedure']['address'],16)==(entry if not partial else 0x9c48c0)
  boundaries={int(x.split(':')[0],16)for x in v['assembly']};assert all(a in boundaries and (b in boundaries or b==0x9b699d or b==0x9b75b0)for a,b in ranges)
  spans=[]
  for a,b in ranges:
   raw=image.read(a,b-a);rows=list(engine.disasm(raw,a));assert sum(i.size for i in rows)==len(raw);name=f'source-{a:08X}-{b:08X}.bin';(out/name).write_bytes(raw)
   for row in rows:
    for op in row.operands:
     if op.type==X86_OP_MEM and not op.mem.base and not op.mem.index:constant[op.mem.disp]=max(op.size,constant.get(op.mem.disp,0))
   spans.append(dict(start=f'{a:08X}',end_exclusive=f'{b:08X}',bytes=b-a,sha256=hashlib.sha256(raw).hexdigest(),blob=name))
  records.append(dict(entry=f'{entry:08X}',partial_entry=partial,query=str(path),query_sha256=sha(path),evidence_id=e['evidence_id'],ranges=spans))
 for a,n in constant.items():(out/f'data-{a:08X}.bin').write_bytes(image.read(a,n))
 return dict(schema='original-body-Z-producer-and-fieldwriter-source-v1',native_executed=False,functions=records,source_bytes=3396,full_bbox_bytes=3213,caller_slice_bytes=183,owned_exit=dict(address='009C4992',bytes=7,target='WriterDone'),data=[dict(address=f'{a:08X}',bytes=n,sha256=hashlib.sha256(image.read(a,n)).hexdigest())for a,n in sorted(constant.items())],ABI='Enter9C48DB withESI=Definition,realESP=S,x87empty,DF0. Originalcodepushesall3arguments andcallsfull9B6920. Exit9C4992 atESP=S-12 withone retainedx87 value(TOP7),Definition358..36Cpublished. No Original9C3150/370/constraint/actor claim.')
def assembly(out,p):
 lines=['.686p','.xmm','.model flat','option casemap:none']+['EXTERN _'+x+':PROC'for x in('GetModuleHandleA@4','GetStdHandle@4','ReadFile@20','WriteFile@20','GetLastError@0','ExitProcess@4')]+['.const']
 for r in p['functions']:
  for s in r['ranges']:lines+=['PUBLIC Original_'+s['start']]+db('Original_'+s['start'],(out/s['blob']).read_bytes())
 for r in p['data']:lines+=['PUBLIC Data_'+r['address']]+db('Data_'+r['address'],(out/f'data-{r["address"]}.bin').read_bytes())
 lines+=['.data','PUBLIC DriverStack,StackLow,StackWindow,SourceStack,CurrentBlock,CurrentPolicy,CurrentCW,Expected,Poison',
 'DriverStack DWORD 0','StackLow DWORD 0','StackWindow DWORD 0','SourceStack DWORD 0','CurrentBlock DWORD 0','CurrentPolicy DWORD 0','CurrentCW DWORD 0','Poison DWORD 0','Expected BYTE 44 DUP(0)','RequestedMxcsr DWORD 01F80h','InputHandle DWORD 0','OutputHandle DWORD 0','Remaining DWORD 0','Transferred DWORD 0','Cursor DWORD 0','AllowEnd DWORD 0']
 for name,n in SIZES.items():lines += [f'PUBLIC {name}Before,{name}Buffer,{name}After,{name}Shadow',f'{name}Before DWORD 4 DUP(0A5A5A5A5h)',f'{name}Buffer BYTE {n} DUP(0)',f'{name}After DWORD 4 DUP(0A5A5A5A5h)',f'{name}Shadow BYTE {n} DUP(0)']
 lines+=['.code','PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,OriginalDone,WriterDone,ReadExact,PrepareBody,RunPolicy','OwnedStart LABEL BYTE','ReadExact PROC','ReadAgain:','push 0','push OFFSET Transferred','push Remaining','push Cursor','push InputHandle','call _ReadFile@20','test eax,eax','jnz ReadSucceeded','call _GetLastError@0','cmp eax,109','jne HostFailure','mov Transferred,0','ReadSucceeded:','mov eax,Transferred','test eax,eax','jnz BytesRead','cmp AllowEnd,1','jne HostFailure','cmp Remaining,32','jne HostFailure','push 0','call _ExitProcess@4','BytesRead:','cmp eax,Remaining','ja HostFailure','sub Remaining,eax','add Cursor,eax','cmp Remaining,0','jne ReadAgain','ret','ReadExact ENDP',
 'PrepareBody PROC','mov edi,OFFSET DefinitionBuffer','xor eax,eax','mov ecx,1024','rep stosb','mov edi,OFFSET BlocksBuffer','mov ecx,287640','rep stosb','mov dword ptr [DefinitionBuffer+384h],OFFSET BlocksBuffer','mov eax,dword ptr [HeaderBuffer+4]','imul eax,eax,0468h','add eax,OFFSET BlocksBuffer','mov dword ptr [DefinitionBuffer+388h],eax','mov CurrentBlock,0','BuildBlock:','mov eax,CurrentBlock','imul esi,eax,324','add esi,OFFSET PayloadBuffer','imul edi,eax,0468h','add edi,OFFSET BlocksBuffer']
 for i,at in enumerate(FIELDS):lines += [f'mov eax,dword ptr [esi+{168+i*4}]',f'mov dword ptr [edi+0{at:X}h],eax']
 for at in(0x10c,0x110,0x114,0x118,0x11c,0x120,0x124,0x138,0x13c,0x140):
  lines += [f'mov eax,dword ptr [edi+0{at:X}h]']
  if at>=0x138:lines+=['test eax,080000000h','jnz HostFailure','test eax,07FFFFFFFh','jz HostFailure']
  lines+=['and eax,07FFFFFFFh',f'cmp eax,0{0x40000000 if 0x118<=at<=0x124 else 0x49800000:X}h','ja HostFailure']
 lines+=['inc CurrentBlock','mov eax,CurrentBlock','cmp eax,dword ptr [HeaderBuffer+4]','jb BuildBlock']
 for name,n in SIZES.items():
  if name in('Reply','State'):continue
  lines += [f'mov esi,OFFSET {name}Buffer',f'mov edi,OFFSET {name}Shadow',f'mov ecx,{n}','rep movsb']
 lines+=['ret','PrepareBody ENDP','RunPolicy PROC','mov DriverStack,esp','mov eax,CurrentCW','mov edx,eax','and eax,0FFFFF0FFh','cmp eax,07Fh','jne HostFailure','and edx,0300h','cmp edx,0100h','je HostFailure','fninit','fldcw word ptr CurrentCW','ldmxcsr RequestedMxcsr','movd xmm0,Poison','pshufd xmm0,xmm0,0']
 for i in range(1,8):lines += [f'movaps xmm{i},xmm0']
 lines+=['sub esp,0320h','mov StackLow,esp','mov edi,esp','mov ecx,0C8h','mov eax,0A5A5A5A5h','rep stosd','mov eax,StackLow','add eax,16','mov StackWindow,eax','add eax,0180h','mov SourceStack,eax','mov esp,eax','mov esi,OFFSET DefinitionBuffer','mov eax,009C48DBh','jmp eax',
 'WriterDone::','mov eax,SourceStack','sub eax,12','cmp esp,eax','jne HostFailure','fnstsw word ptr [StateBuffer+10]','mov ax,word ptr [StateBuffer+10]','and ax,03800h','cmp ax,03800h','jne HostFailure','fstp tbyte ptr [StateBuffer]','mov esp,DriverStack']
 # Source stack window is real stack. Scratch belowS is allowed for sourcecallee;
 # aboveS onlyS+4..23 holds actual published-call scratch.
 lines+=['mov esi,SourceStack','mov ecx,4','CheckStackFirst:','cmp byte ptr [esi],0A5h','jne HostFailure','inc esi','loop CheckStackFirst','mov esi,SourceStack','add esi,24','mov edi,StackWindow','add edi,0300h','CheckStackUpper:','cmp esi,edi','jae StackUpperDone','cmp byte ptr [esi],0A5h','jne HostFailure','inc esi','jmp CheckStackUpper','StackUpperDone:']
 for pexpr in('StackLow','StackWindow'):
  lines += [f'mov esi,{pexpr}']
  if pexpr=='StackWindow':lines += ['add esi,0300h']
  for at in range(0,16,4):lines += [f'cmp dword ptr [esi+{at}],0A5A5A5A5h','jne HostFailure']
 for name,n in SIZES.items():
  if name in('Reply','State','Definition'):continue
  lines += [f'mov esi,OFFSET {name}Buffer',f'mov edi,OFFSET {name}Shadow',f'mov ecx,{n}','repe cmpsb','jne HostFailure']
 lines+=['xor ecx,ecx','CheckDefinition:','cmp ecx,0358h','jb CompareDefinition','cmp ecx,0370h','jb NextDefinition','CompareDefinition:','mov al,byte ptr [DefinitionBuffer+ecx]','cmp al,byte ptr [DefinitionShadow+ecx]','jne HostFailure','NextDefinition:','inc ecx','cmp ecx,1024','jb CheckDefinition']
 for name in SIZES:
  for side in('Before','After'):
   for at in range(0,16,4):lines += [f'cmp dword ptr [{name}{side}+{at}],0A5A5A5A5h','jne HostFailure']
 lines+=['mov eax,dword ptr [HeaderBuffer+12]','mov dword ptr [ReplyBuffer],eax','mov eax,CurrentCW','mov dword ptr [ReplyBuffer+4],eax','mov esi,OFFSET DefinitionBuffer+358h','mov edi,OFFSET ReplyBuffer+8','mov ecx,24','rep movsb','mov esi,OFFSET StateBuffer','mov ecx,10','rep movsb','mov word ptr [ReplyBuffer+42],0','fnstcw word ptr [ReplyBuffer+44]','fnstsw word ptr [ReplyBuffer+46]','stmxcsr dword ptr [ReplyBuffer+48]','mov ax,word ptr CurrentCW','cmp ax,word ptr [ReplyBuffer+44]','jne HostFailure','test word ptr [ReplyBuffer+46],03800h','jnz HostFailure','mov eax,dword ptr [ReplyBuffer+48]','and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure','ret','RunPolicy ENDP',
 'HostMain PROC','cld','push 0','call _GetModuleHandleA@4',f'cmp eax,0{BASE:08X}h','jne HostFailure']
 for r in p['functions']:
  for s in r['ranges']:lines += [f'mov esi,OFFSET Original_{s["start"]}',f'mov edi,0{s["start"]}h',f'mov ecx,{s["bytes"]}','repe cmpsb','jne HostFailure']
 for r in p['data']:lines += [f'mov esi,OFFSET Data_{r["address"]}',f'mov edi,0{r["address"]}h',f'mov ecx,{r["bytes"]}','repe cmpsb','jne HostFailure']
 lines+=['cmp byte ptr [009C4992h],0B8h','jne HostFailure','mov eax,OFFSET WriterDone','cmp dword ptr [009C4993h],eax','jne HostFailure','cmp word ptr [009C4997h],0E0FFh','jne HostFailure','push -10','call _GetStdHandle@4','mov InputHandle,eax','push -11','call _GetStdHandle@4','mov OutputHandle,eax',
 'NextGroup:','mov Remaining,32','mov Cursor,OFFSET HeaderBuffer','mov AllowEnd,1','call ReadExact','cmp dword ptr [HeaderBuffer],0315A4242h','jne HostFailure','cmp dword ptr [HeaderBuffer+4],1','jb HostFailure','cmp dword ptr [HeaderBuffer+4],255','ja HostFailure','cmp dword ptr [HeaderBuffer+8],1','jb HostFailure','cmp dword ptr [HeaderBuffer+8],12','ja HostFailure']
 for at in(16,20,24,28):lines += [f'cmp dword ptr [HeaderBuffer+{at}],0','jne HostFailure']
 lines+=['mov edi,OFFSET PayloadBuffer','mov ecx,82668','xor eax,eax','rep stosb','mov eax,dword ptr [HeaderBuffer+4]','imul eax,eax,324','mov edx,dword ptr [HeaderBuffer+8]','shl edx,2','add eax,edx','mov Remaining,eax','mov Cursor,OFFSET PayloadBuffer','mov AllowEnd,0','call ReadExact','call PrepareBody','mov CurrentPolicy,0','NextPolicy:','mov eax,dword ptr [HeaderBuffer+4]','imul eax,eax,324','mov edx,CurrentPolicy','mov eax,dword ptr [PayloadBuffer+eax+edx*4]','mov CurrentCW,eax','mov Poison,0','call RunPolicy','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected','mov ecx,44','rep movsb','mov Poison,-1','call RunPolicy','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected','mov ecx,44','repe cmpsb','jne HostFailure','mov Remaining,52','mov Cursor,OFFSET ReplyBuffer','WriteAgain:','push 0','push OFFSET Transferred','push Remaining','push Cursor','push OutputHandle','call _WriteFile@20','test eax,eax','jz HostFailure','mov eax,Transferred','test eax,eax','jz HostFailure','cmp eax,Remaining','ja HostFailure','sub Remaining,eax','add Cursor,eax','cmp Remaining,0','jne WriteAgain','inc CurrentPolicy','mov eax,CurrentPolicy','cmp eax,dword ptr [HeaderBuffer+8]','jb NextPolicy','jmp NextGroup','HostMain ENDP',
 'OriginalDone LABEL BYTE','HostFailure PROC','cmp DriverStack,0','je FailureStackReady','mov esp,DriverStack','FailureStackReady::','push 71','call _ExitProcess@4','HostFailure ENDP','OwnedEnd LABEL BYTE','END']
 return '\n'.join(lines)+'\n'
def symbols(path):
 d={}
 for line in path.read_text().splitlines():
  m=re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]+\s+(\S+)\s+([0-9A-Fa-f]{8})\b',line)
  if m:d[m[1]]=int(m[2],16)
 return d

def pages(image,p,mapping):
 sy=symbols(mapping);pages={0x990000:bytearray(b'\xcc'*65536)};code={0x990000}
 for r in p['functions']:
  for s in r['ranges']:
   a=int(s['start'],16);code.add(a&~0xffff);pages.setdefault(a&~0xffff,bytearray(b'\xcc'*65536));pages[a&~0xffff][a&0xffff:(a&0xffff)+s['bytes']]=image.read(a,s['bytes'])
 for r in p['data']:
  a=int(r['address'],16);pages.setdefault(a&~0xffff,bytearray(65536));pages[a&~0xffff][a&0xffff:(a&0xffff)+r['bytes']]=image.read(a,r['bytes'])
 pages[0x9c0000][0x4992:0x4999]=b'\xb8'+struct.pack('<I',sy['WriterDone'])+b'\xff\xe0'
 return[(a,bytes(b),0x60000020 if a in code else 0x40000040)for a,b in sorted(pages.items())]
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--owned-exe',type=Path);p.add_argument('--owned-map',type=Path);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());image=SourceImage(a.source_exe)
 if a.owned_exe:
  source=json.loads((a.out/'source-closure.json').read_text());old=(carrier.BASE,carrier.source_pages)
  try:carrier.BASE=BASE;carrier.source_pages=lambda im,pr,done:pages(im,pr,a.owned_map);blob,r=carrier.build(a.owned_exe,a.owned_map,image,source)
  finally:carrier.BASE,carrier.source_pages=old
  (a.out/'Z-carrier-x86.exe').write_bytes(blob);r.update(native_executed=False,execution_admitted=False,carrier_sha256=sha(a.out/'Z-carrier-x86.exe'));(a.out/'carrier-build-review.json').write_text(json.dumps(r,indent=2)+'\n');return
 a.out.mkdir(parents=True,exist_ok=False);p=proof(image,a.out);(a.out/'source-closure.json').write_text(json.dumps(p,indent=2)+'\n');(a.out/'host-x86.asm').write_text(assembly(a.out,p));m=dict(schema='original-Z-producer-fieldwriter-native-preparation-v1',tool_sha256=sha(__file__),asm_sha256=sha(a.out/'host-x86.asm'),source_bytes=3396,source_full_helper_bytes=3213,source_partial_caller_bytes=183,owned_exit_bytes=7,buffers=SIZES,model_bytes=44,reply_bytes=52,source_scope='Full9B6920 plus actualcaller9C48DB..4992 only. Caller sourcebuildsall3arguments itself, publishes6Definitionfields; originalrestwrapper/finalizer/370/constraints notexecuted.',stack_scope='RealStackWindow768B+two16Bguards,SourceS=window+180h. OriginalexitESP=S-12,retainedx87TOP7->capture80->TOP0. BelowSsourcecall scratch allowed; aboveS onlyS+4..23 writable. Failure restoresDriverStack beforeWindowsAPI.',compiled=False,native_executed=False,execution_admitted=False,compile_commands=[['ml.exe','/nologo','/c','/coff','/Fohost-x86.obj','host-x86.asm'],['link.exe','/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/BASE:0x29000000','/FIXED','/DYNAMICBASE:NO','/SAFESEH:NO','/OUT:host-x86.exe','/MAP:host-x86.map','host-x86.obj','kernel32.lib']]);(a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
if __name__=='__main__':main()

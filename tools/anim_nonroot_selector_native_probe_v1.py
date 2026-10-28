"""Prepare a guarded grouped actual-body full9B2150 selector native gate.
No compile/link/execution. Proprietary bytes emitted only beneath target/.
"""
import argparse,json,hashlib,struct
from pathlib import Path
import anim_root_pose_sparse_carrier_v3 as carrier
from cell_native_math_probe import db
from cell_schema_probe import SourceImage
from native_crt_acos_probe import decoder
from anim_body_finalizer_native_probe import FIELDS
ROOT=Path(__file__).resolve().parent.parent
BASE=0x990000
INPUTS=ROOT/'target/animation/nonroot-selector-inputs-v1'
SIZES={'Header':32,'Payload':90812,'Definition':1024,'Blocks':255*0x468,'Query':32,'Indices':1020,'Reply':1040}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def source_proof(image,out):
 p=json.loads((INPUTS/'source-closure.json').read_text());p=json.loads(json.dumps(p))
 known=json.loads((ROOT/'target/animation/body-capabilities-native-v1/manifest.json').read_text())['code_allowlist']
 for a,b in((0x9b2da0,0x9b2e11),(0x9b2e20,0x9b2e5c)):
  raw=image.read(a,b-a);found=next(r for r in known if int(r['start'],16)==a);assert hashlib.sha256(raw).hexdigest()==found['sha256'];rows=list(decoder().disasm(raw,a));assert sum(i.size for i in rows)==len(raw)
  name=f'source-{a:08X}-{b:08X}.bin';p['functions'].append(dict(entry=f'{a:08X}',accepted_capability_helper=True,ranges=[dict(start=f'{a:08X}',end_exclusive=f'{b:08X}',bytes=b-a,sha256=hashlib.sha256(raw).hexdigest(),blob=name)]));p['source_bytes']+=len(raw)
 for r in p['functions']:
  for span in r['ranges']:
   a=int(span['start'],16);raw=image.read(a,span['bytes']);assert hashlib.sha256(raw).hexdigest()==span['sha256'];(out/span['blob']).write_bytes(raw)
 for r in p['data']:
  a=int(r['address'],16);raw=image.read(a,r['bytes']);assert hashlib.sha256(raw).hexdigest()==r['sha256'];(out/f'data-{a:08X}.bin').write_bytes(raw)
 (out/'source-011E0534-011E053A.bin').write_bytes(image.read(0x11e0534,6))
 p['targeted_import_proof']=targeted_import_proof(image)
 p['owned_IAT_binding']=dict(address='013CC47C',target='MemcpyProvider',reason='Approved bounded value-free CRTcopy ABI; original6Bthunk remains byteexact')
 p['owned_trap_page']='00990000 (first4KiBheaders; restINT3, no proprietary code there)'
 return p

def targeted_import_proof(image):
 nt=struct.unpack_from('<I',image.data,60)[0];op=nt+24;base=struct.unpack_from('<I',image.data,op+28)[0];rva,size=struct.unpack_from('<II',image.data,op+104)
 def text(address):
  out=bytearray()
  for i in range(256):
   b=image.read(address+i,1)[0]
   if not b:return out.decode('ascii')
   out.append(b)
  raise AssertionError('unbounded import string')
 for i in range(size//20):
  oft,stamp,chain,name,ft=struct.unpack('<5I',image.read(base+rva+i*20,20))
  if not name:break
  for j in range(2048):
   value=struct.unpack('<I',image.read(base+(oft or ft)+j*4,4))[0]
   if not value:break
   if base+ft+j*4!=0x13cc47c:continue
   assert not value&0x80000000
   dll=text(base+name);symbol=text(base+value+2)
   assert(dll,symbol)==('MSVCR90.dll','memcpy')
   return dict(IAT='013CC47C',dll=dll,symbol=symbol,exact_targeted_import=True)
 raise AssertionError('targeted memcpy import not verified')

def host_source(out,p):
 lines=['.686p','.xmm','.model flat','option casemap:none']
 lines += ['EXTERN _'+x+':PROC'for x in('GetModuleHandleA@4','GetStdHandle@4','ReadFile@20','WriteFile@20','GetLastError@0','ExitProcess@4')]+['.const']
 for r in p['functions']:
  for s in r['ranges']:lines += ['PUBLIC Original_'+s['start']]+db('Original_'+s['start'],(out/s['blob']).read_bytes())
 lines += ['PUBLIC Original_011E0534']+db('Original_011E0534',(out/'source-011E0534-011E053A.bin').read_bytes())
 for r in p['data']:lines += ['PUBLIC Data_'+r['address']]+db('Data_'+r['address'],(out/f'data-{r["address"]}.bin').read_bytes())
 lines+=['.data','PUBLIC DriverStack,ProviderCalls,ReturnedCount,CurrentBlock,CurrentQuery,NeedBytes,Poison,Expected',
 'DriverStack DWORD 0','ProviderCalls DWORD 0','ReturnedCount DWORD 0','CurrentBlock DWORD 0','CurrentQuery DWORD 0','NeedBytes DWORD 0','Poison DWORD 0','Expected BYTE 1032 DUP(0)','RequestedMxcsr DWORD 01F80h','InputHandle DWORD 0','OutputHandle DWORD 0','Transferred DWORD 0','Remaining DWORD 0','Cursor DWORD 0','AllowEnd DWORD 0']
 for name,n in SIZES.items():lines += [f'PUBLIC {name}Before,{name}Buffer,{name}After,{name}Shadow',f'{name}Before DWORD 4 DUP(0A5A5A5A5h)',f'{name}Buffer BYTE {n} DUP(0)',f'{name}After DWORD 4 DUP(0A5A5A5A5h)',f'{name}Shadow BYTE {n} DUP(0)']
 lines+=['.code','PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,OriginalDone,ReadExact,MemcpyProvider,ValidateHeader,PrepareBody,ValidateQuery,RunQuery','OwnedStart LABEL BYTE',
 'ReadExact PROC','ReadAgain:','push 0','push OFFSET Transferred','push Remaining','push Cursor','push InputHandle','call _ReadFile@20','test eax,eax','jnz ReadSucceeded','call _GetLastError@0','cmp eax,109','jne HostFailure','mov Transferred,0','ReadSucceeded:','mov eax,Transferred','test eax,eax','jnz BytesRead','cmp AllowEnd,1','jne HostFailure','cmp Remaining,32','jne HostFailure','push 0','call _ExitProcess@4','BytesRead:','cmp eax,Remaining','ja HostFailure','sub Remaining,eax','add Cursor,eax','cmp Remaining,0','jne ReadAgain','ret','ReadExact ENDP',
 'ValidateHeader PROC','cmp dword ptr [HeaderBuffer],0314C4553h','jne HostFailure','cmp dword ptr [HeaderBuffer+4],1','jb HostFailure','cmp dword ptr [HeaderBuffer+4],255','ja HostFailure','cmp dword ptr [HeaderBuffer+8],1','jb HostFailure','cmp dword ptr [HeaderBuffer+8],256','ja HostFailure','mov eax,dword ptr [HeaderBuffer+12]','mov edx,eax','and eax,0FFFFF0FFh','cmp eax,07Fh','jne HostFailure','and edx,0300h','cmp edx,0100h','je HostFailure']
 for at in(20,24,28):lines += [f'cmp dword ptr [HeaderBuffer+{at}],0','jne HostFailure']
 lines += ['ret','ValidateHeader ENDP','PrepareBody PROC','mov edi,OFFSET DefinitionBuffer','xor eax,eax','mov ecx,1024','rep stosb','mov edi,OFFSET BlocksBuffer','mov ecx,287640','rep stosb','mov dword ptr [DefinitionBuffer+384h],OFFSET BlocksBuffer','mov eax,dword ptr [HeaderBuffer+4]','imul eax,eax,0468h','add eax,OFFSET BlocksBuffer','mov dword ptr [DefinitionBuffer+388h],eax',
 'mov dword ptr [DefinitionBuffer+360h],07FC0CAFEh','mov dword ptr [DefinitionBuffer+36Ch],07FC0CAFEh','mov CurrentBlock,0','BuildBlock:','mov eax,CurrentBlock','imul esi,eax,324','add esi,OFFSET PayloadBuffer','imul edi,eax,0468h','add edi,OFFSET BlocksBuffer','mov edx,edi','push esi','push edi','lea edi,[edi+150h]','mov ecx,168','rep movsb','pop edi','pop esi']
 for i,at in enumerate(FIELDS):lines += [f'mov eax,dword ptr [esi+{168+4*i}]',f'mov dword ptr [edi+0{at:X}h],eax']
 lines += ['mov dword ptr [edi+210h],0DEADFA11h','mov eax,CurrentBlock','cmp dword ptr [edi+1F8h],eax','jne HostFailure']
 for at in(0x10c,0x110,0x114,0x138,0x13c,0x140):
  lines += [f'mov eax,dword ptr [edi+0{at:X}h]']
  if at>=0x138:lines += ['test eax,080000000h','jnz HostFailure','test eax,07FFFFFFFh','jz HostFailure']
  lines += ['and eax,07FFFFFFFh','cmp eax,049800000h','ja HostFailure']
 # Recompute both masks with already accepted unchanged native integer helpers;
 # reject mismatches before selector rather than inventing capability masks.
 lines += ['lea ecx,[edi+150h]','mov eax,009B2E20h','call eax','mov eax,CurrentBlock','imul esi,eax,324','add esi,OFFSET PayloadBuffer','imul edi,eax,0468h','add edi,OFFSET BlocksBuffer','mov eax,dword ptr [esi]','cmp eax,dword ptr [edi+150h]','jne HostFailure','mov eax,dword ptr [esi+4]','cmp eax,dword ptr [edi+154h]','jne HostFailure','mov eax,CurrentBlock','test eax,eax','jnz ValidateNonRoot','cmp dword ptr [edi+1FCh],-1','jne HostFailure','test byte ptr [edi+154h],8','jz HostFailure','jmp ParentChecked',
 'ValidateNonRoot:','mov ecx,CurrentBlock','xor edx,edx','WalkParent:','imul eax,ecx,0468h','mov eax,dword ptr [BlocksBuffer+eax+1FCh]','cmp eax,-1','je ParentEnd','cmp eax,dword ptr [HeaderBuffer+4]','jae HostFailure','cmp eax,ecx','je HostFailure','mov ecx,eax','inc edx','cmp edx,dword ptr [HeaderBuffer+4]','jae HostFailure','jmp WalkParent','ParentEnd:','test ecx,ecx','jnz HostFailure','ParentChecked:','inc CurrentBlock','mov eax,CurrentBlock','cmp eax,dword ptr [HeaderBuffer+4]','jb BuildBlock']
 for name,n in SIZES.items():
  if name in('Query','Indices','Reply'):continue
  lines += [f'mov esi,OFFSET {name}Buffer',f'mov edi,OFFSET {name}Shadow',f'mov ecx,{n}','rep movsb']
 lines += ['ret','PrepareBody ENDP','ValidateQuery PROC','mov eax,dword ptr [QueryBuffer]','mov edx,eax','and edx,0FFF90FC3h','jnz HostFailure','mov edx,eax','and edx,0801C00h','test edx,edx','jz HierarchyValid','cmp eax,01000h','jne HostFailure','HierarchyValid:','mov edx,eax','and edx,02000Ch']
 for name,mask,values in [('XMask',0x2000c,[0,4,8,12,0x20000,0x20004,0x20008]),('YMask',0x40030,[0,0x10,0x20,0x30,0x40000,0x40010,0x40020]),('Rank',0xe000,[0,0x2000,0x4000,0x6000,0x8000])]:
  lines += ['mov edx,eax',f'and edx,0{mask:X}h']
  for value in values:lines += [f'cmp edx,0{value:X}h',f'je {name}Valid']
  lines += ['jmp HostFailure',name+'Valid:']
 for at in(8,12,24):lines += [f'cmp dword ptr [QueryBuffer+{at}],0','jne HostFailure']
 lines += ['cmp dword ptr [QueryBuffer+16],1','ja HostFailure','cmp dword ptr [QueryBuffer+20],255','ja HostFailure','ret','ValidateQuery ENDP',
 'MemcpyProvider PROC','cmp dword ptr [esp],009B221Bh','jne HostFailure','cmp dword ptr [esp+4],OFFSET IndicesBuffer','jne HostFailure','lea eax,[esp+160h]','cmp dword ptr [esp+8],eax','jne HostFailure','mov ecx,dword ptr [esp+12]','test ecx,ecx','jz HostFailure','test ecx,3','jnz HostFailure','cmp ecx,1020','ja HostFailure','mov edx,dword ptr [QueryBuffer+20]','shl edx,2','cmp ecx,edx','ja HostFailure','mov edx,dword ptr [HeaderBuffer+4]','shl edx,2','cmp ecx,edx','ja HostFailure','mov edx,OFFSET IndicesBuffer','add edx,ecx','cmp edx,eax','jbe CopyRangesValid','add eax,ecx','cmp eax,OFFSET IndicesBuffer','ja HostFailure','CopyRangesValid:','inc ProviderCalls','cmp ProviderCalls,1','jne HostFailure','pushfd','pop eax','test eax,0400h','jnz HostFailure','push esi','push edi','mov edi,dword ptr [esp+12]','mov esi,dword ptr [esp+16]','mov ecx,dword ptr [esp+20]','mov eax,edi','rep movsb','pop edi','pop esi','ret','MemcpyProvider ENDP',
 'RunQuery PROC','mov DriverStack,esp','call ValidateQuery','mov esi,OFFSET QueryBuffer','mov edi,OFFSET QueryShadow','mov ecx,32','rep movsb','mov edi,OFFSET IndicesBuffer','mov ecx,255','mov eax,-1','rep stosd','mov ProviderCalls,0','fninit','fldcw word ptr [HeaderBuffer+12]','ldmxcsr RequestedMxcsr','movd xmm0,Poison','pshufd xmm0,xmm0,0']
 for i in range(1,8):lines += [f'movaps xmm{i},xmm0']
 lines += ['push 0','push dword ptr [QueryBuffer+16]','push OFFSET QueryBuffer','push OFFSET DefinitionBuffer','push dword ptr [QueryBuffer+20]','push OFFSET IndicesBuffer','mov eax,009B2150h','call eax','add esp,24','cmp esp,DriverStack','jne HostFailure','mov ReturnedCount,eax','cmp eax,dword ptr [QueryBuffer+20]','ja HostFailure','cmp eax,dword ptr [HeaderBuffer+4]','ja HostFailure','test eax,eax','jz NoCopyExpected','cmp ProviderCalls,1','jne HostFailure','jmp CopyChecked','NoCopyExpected:','cmp ProviderCalls,0','jne HostFailure','CopyChecked:','xor ecx,ecx','CheckIndices:','cmp ecx,ReturnedCount','jae CheckUnused','mov eax,dword ptr [IndicesBuffer+ecx*4]','cmp eax,dword ptr [HeaderBuffer+4]','jae HostFailure','jmp NextIndex','CheckUnused:','cmp dword ptr [IndicesBuffer+ecx*4],-1','jne HostFailure','NextIndex:','inc ecx','cmp ecx,255','jb CheckIndices']
 for name,n in SIZES.items():
  if name in('Indices','Reply'):continue
  lines += [f'mov esi,OFFSET {name}Buffer',f'mov edi,OFFSET {name}Shadow',f'mov ecx,{n}','repe cmpsb','jne HostFailure']
 for name in SIZES:
  for side in('Before','After'):
   for at in range(0,16,4):lines += [f'cmp dword ptr [{name}{side}+{at}],0A5A5A5A5h','jne HostFailure']
 lines += ['pushfd','pop eax','test eax,0400h','jnz HostFailure','mov eax,dword ptr [HeaderBuffer+16]','mov dword ptr [ReplyBuffer],eax','mov eax,dword ptr [QueryBuffer+28]','mov dword ptr [ReplyBuffer+4],eax','mov eax,ReturnedCount','mov dword ptr [ReplyBuffer+8],eax','mov esi,OFFSET IndicesBuffer','mov edi,OFFSET ReplyBuffer+12','mov ecx,1020','rep movsb','fnstcw word ptr [ReplyBuffer+1032]','fnstsw word ptr [ReplyBuffer+1034]','stmxcsr dword ptr [ReplyBuffer+1036]','mov ax,word ptr [HeaderBuffer+12]','cmp ax,word ptr [ReplyBuffer+1032]','jne HostFailure','test word ptr [ReplyBuffer+1034],03800h','jnz HostFailure','mov eax,dword ptr [ReplyBuffer+1036]','and eax,0FFFFFFC0h','cmp eax,01F80h','jne HostFailure','ret','RunQuery ENDP',
 'HostMain PROC','cld','push 0','call _GetModuleHandleA@4',f'cmp eax,0{BASE:08X}h','jne HostFailure']
 for r in p['functions']:
  for s in r['ranges']:lines += [f'mov esi,OFFSET Original_{s["start"]}',f'mov edi,0{s["start"]}h',f'mov ecx,{s["bytes"]}','repe cmpsb','jne HostFailure']
 lines += ['mov esi,OFFSET Original_011E0534','mov edi,011E0534h','mov ecx,6','repe cmpsb','jne HostFailure']
 for r in p['data']:lines += [f'mov esi,OFFSET Data_{r["address"]}',f'mov edi,0{r["address"]}h',f'mov ecx,{r["bytes"]}','repe cmpsb','jne HostFailure']
 lines += ['cmp dword ptr [013CC47Ch],OFFSET MemcpyProvider','jne HostFailure','push -10','call _GetStdHandle@4','mov InputHandle,eax','push -11','call _GetStdHandle@4','mov OutputHandle,eax',
 'NextGroup:','mov Remaining,32','mov Cursor,OFFSET HeaderBuffer','mov AllowEnd,1','call ReadExact','call ValidateHeader','mov edi,OFFSET PayloadBuffer','mov ecx,90812','xor eax,eax','rep stosb','mov eax,dword ptr [HeaderBuffer+4]','imul eax,eax,324','mov edx,dword ptr [HeaderBuffer+8]','shl edx,5','add eax,edx','mov NeedBytes,eax','mov Remaining,eax','mov Cursor,OFFSET PayloadBuffer','mov AllowEnd,0','call ReadExact','call PrepareBody','mov CurrentQuery,0','NextQuery:','mov eax,dword ptr [HeaderBuffer+4]','imul eax,eax,324','mov edx,CurrentQuery','shl edx,5','add eax,edx','add eax,OFFSET PayloadBuffer','mov esi,eax','mov edi,OFFSET QueryBuffer','mov ecx,32','rep movsb','mov Poison,0','call RunQuery','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected','mov ecx,1032','rep movsb','mov Poison,-1','call RunQuery','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected','mov ecx,1032','repe cmpsb','jne HostFailure','mov Remaining,1040','mov Cursor,OFFSET ReplyBuffer','WriteAgain:','push 0','push OFFSET Transferred','push Remaining','push Cursor','push OutputHandle','call _WriteFile@20','test eax,eax','jz HostFailure','mov eax,Transferred','test eax,eax','jz HostFailure','cmp eax,Remaining','ja HostFailure','sub Remaining,eax','add Cursor,eax','cmp Remaining,0','jne WriteAgain','inc CurrentQuery','mov eax,CurrentQuery','cmp eax,dword ptr [HeaderBuffer+8]','jb NextQuery','jmp NextGroup','HostMain ENDP',
 'OriginalDone LABEL BYTE','HostFailure PROC','cmp DriverStack,0','je FailureStackReady','mov esp,DriverStack','FailureStackReady::','push 71','call _ExitProcess@4','HostFailure ENDP','OwnedEnd LABEL BYTE','END']
 return '\n'.join(lines)+'\n'

def source_pages(image,p,unused,mapping):
 symbols=carrier_symbols(mapping);pages={0x990000:bytearray(b'\xcc'*65536)};code={0x990000}
 for r in p['functions']:
  for s in r['ranges']:
   a=int(s['start'],16);code.add(a&~0xffff);pages.setdefault(a&~0xffff,bytearray(b'\xcc'*65536));pages[a&~0xffff][a&0xffff:(a&0xffff)+s['bytes']]=image.read(a,s['bytes'])
 code.add(0x11e0000);pages[0x11e0000]=bytearray(b'\xcc'*65536);pages[0x11e0000][0x534:0x53a]=image.read(0x11e0534,6)
 for r in p['data']:
  a=int(r['address'],16);pages.setdefault(a&~0xffff,bytearray(65536));pages[a&~0xffff][a&0xffff:(a&0xffff)+r['bytes']]=image.read(a,r['bytes'])
 pages.setdefault(0x13c0000,bytearray(65536));struct.pack_into('<I',pages[0x13c0000],0xc47c,symbols['MemcpyProvider'])
 return[(a,bytes(b),0x60000020 if a in code else 0x40000040)for a,b in sorted(pages.items())]
def carrier_symbols(mapping):
 import re
 result={}
 for line in mapping.read_text().splitlines():
  m=re.match(r'\s*[0-9a-fA-F]{4}:[0-9a-fA-F]+\s+(\S+)\s+([0-9a-fA-F]{8})\b',line)
  if m:result[m[1]]=int(m[2],16)
 return result

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--owned-exe',type=Path);p.add_argument('--owned-map',type=Path);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());image=SourceImage(a.source_exe)
 if a.owned_exe:
  proof=json.loads((a.out/'source-closure.json').read_text());old=(carrier.BASE,carrier.source_pages)
  try:carrier.BASE=BASE;carrier.source_pages=lambda im,pr,d:source_pages(im,pr,d,a.owned_map);blob,review=carrier.build(a.owned_exe,a.owned_map,image,proof)
  finally:carrier.BASE,carrier.source_pages=old
  (a.out/'selector-carrier-x86.exe').write_bytes(blob);review.update(native_executed=False,execution_admitted=False,carrier_sha256=sha(a.out/'selector-carrier-x86.exe'));(a.out/'carrier-build-review.json').write_text(json.dumps(review,indent=2)+'\n');return
 a.out.mkdir(parents=True,exist_ok=False);proof=source_proof(image,a.out);(a.out/'source-closure.json').write_text(json.dumps(proof,indent=2)+'\n');(a.out/'host-x86.asm').write_text(host_source(a.out,proof))
 m=dict(schema='actual-body-full-mode0-selector-native-preparation-v1',tool_sha256=sha(__file__),asm_sha256=sha(a.out/'host-x86.asm'),carrier_mechanics_sha256=sha(carrier.__file__),source_bytes=proof['source_bytes'],source_closure_sha256=sha(a.out/'source-closure.json'),buffers=SIZES,input_scope='Grouped actual finalized324Bbodyrecords,then32Bqueries.1641/1644aliases,3Zexclusions retained. No zerochild substitution.',copy_contract='Original6Bthunk kept. OwnedIATcopy provider exactreturn9B221B,destIndices,srcEntryESP+160,sizepositive/multiple4/<=1020/capacity4/bodycount4,nonoverlap,once-only,ESI/EDI saved,EBX/EBP untouched; no numeric/private algorithm substitute.',opaque_fields='Definition360/36C quietNaN canaries may be read by discarded noZ calculation; decision unobserved under Zmask0. Block210 opaque canary; allclassmasks0. No original producer/value claim.',compiled=False,native_executed=False,execution_admitted=False,compile_commands=[['ml.exe','/nologo','/c','/coff','/Fohost-x86.obj','host-x86.asm'],['link.exe','/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/BASE:0x29000000','/FIXED','/DYNAMICBASE:NO','/SAFESEH:NO','/OUT:host-x86.exe','/MAP:host-x86.map','host-x86.obj','kernel32.lib']],remaining=['Independent actual-linked source/PE/provider/frame/guards audit','Rootcompile/peer/protocol/native155996/Rustcomparison','OriginalDefinitionZbounds3selectors','Bindingconstructor/AEownership/pruning/liveclipadmission/actor association'])
 (a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
if __name__=='__main__':main()

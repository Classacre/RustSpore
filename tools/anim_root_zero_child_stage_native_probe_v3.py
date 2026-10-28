"""Complete zero-child full-entry native transport plus sparse carrier preparation.
Never compile/execute. Uses approved adjacent-section carrier metadata mechanics.
"""
import argparse,hashlib,json,struct,re
from pathlib import Path
import anim_root_zero_child_stage_native_probe_v2 as frozen_v2
import anim_root_zero_child_stage_prepare_v1 as core
import anim_root_pose_sparse_carrier_v3 as carrier
from cell_schema_probe import SourceImage
ROOT=Path(__file__).resolve().parent.parent
BASE=0x990000
SOURCE=core.SOURCE
SIZES={'Input':1256,'Root':700,'Definition':524,'Context':128,'Body':748,'Orientation':16,'DefaultPosition':12,'Reply':736}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def proof():return json.loads((SOURCE/'source-closure.json').read_text())
def source_pages(image,audit,done):
 code={int(s['start'],16)&~0xffff for r in audit['source_records'] for s in r['ranges']}
 code|={int(a,16)&~0xffff for a in audit['excluded_calls']}
 data={int(r['address'],16)&~0xffff for r in audit['data']};assert not code&data
 pages={a:bytearray(b'\xcc'*65536 if a in code else bytes(65536)) for a in code|data}
 for r in audit['source_records']:
  for s in r['ranges']:
   a=int(s['start'],16);raw=image.read(a,s['bytes']);assert hashlib.sha256(raw).hexdigest()==s['sha256'];pages[a&~0xffff][a&0xffff:(a&0xffff)+len(raw)]=raw
 for r in audit['data']:
  a=int(r['address'],16);raw=image.read(a,r['bytes']);assert hashlib.sha256(raw).hexdigest()==r['sha256'];pages[a&~0xffff][a&0xffff:(a&0xffff)+len(raw)]=raw
 assert all(not(BASE<=int(s['start'],16)<BASE+4096) for r in audit['source_records'] for s in r['ranges'])
 return [(a,bytes(pages[a]),0x60000020 if a in code else 0x40000040) for a in sorted(pages)]
def host_source(fragment):
 assert sha(frozen_v2.__file__)=='5a046df9e8ed4f446da10c8806695d5fbbab0f0621923b878ad48d7ab60d7099'
 lines=fragment.splitlines();lines.pop();assert lines[-1]=='StageOwnedEnd LABEL BYTE'
 index=lines.index('.const');lines[index:index]=['EXTERN _'+n+':PROC' for n in ['GetModuleHandleA@4','GetStdHandle@4','ReadFile@20','WriteFile@20','GetLastError@0','ExitProcess@4']]
 lines=[('PUBLIC '+line.split()[0]+'\n'+line) if re.match(r'(Original|Data)_[0-9A-F]{8} LABEL BYTE',line) else line for line in lines]
 idx=lines.index('.code');lines[idx:idx]=['InputHandle DWORD 0','OutputHandle DWORD 0','Transferred DWORD 0','Remaining DWORD 0','Cursor DWORD 0','Expected BYTE 736 DUP(0)']
 idx=lines.index('StageOwnedStart LABEL BYTE');lines[idx:idx]=['PUBLIC OwnedStart,OwnedEnd,HostMain,HostFailure,OriginalDone','OwnedStart LABEL BYTE']
 # Canonicalize owned definition pointer only in projected reply, not source state.
 idx=lines.index('fnstcw word ptr [ReplyBuffer+728]');lines[idx:idx]=['mov dword ptr [ReplyBuffer],0']
 lines+=['HostMain PROC','cld','push 0','call _GetModuleHandleA@4',f'cmp eax,0{BASE:08X}h','jne HostFailure']
 p=proof()
 for r in p['source_records']:
  for s in r['ranges']:
   lines += [f'mov esi,OFFSET Original_{s["start"]}',f'mov edi,0{s["start"]}h',f'mov ecx,{s["bytes"]}','repe cmpsb','jne HostFailure']
 for r in p['data']:lines += [f'mov esi,OFFSET Data_{r["address"]}',f'mov edi,0{r["address"]}h',f'mov ecx,{r["bytes"]}','repe cmpsb','jne HostFailure']
 for a in p['excluded_calls']:lines += [f'cmp byte ptr [0{a}h],0CCh','jne HostFailure']
 lines += ['push -10','call _GetStdHandle@4','mov InputHandle,eax','push -11','call _GetStdHandle@4','mov OutputHandle,eax',
 'NextRecord:','mov Remaining,1256','mov Cursor,OFFSET InputBuffer','ReadMore:','push 0','push OFFSET Transferred','push Remaining','push Cursor','push InputHandle','call _ReadFile@20','test eax,eax','jnz ReadSucceeded','call _GetLastError@0','cmp eax,109','jne HostFailure','mov Transferred,0',
 'ReadSucceeded:','mov eax,Transferred','test eax,eax','jnz ReadReceived','cmp Remaining,1256','jne HostFailure','push 0','call _ExitProcess@4',
 'ReadReceived:','cmp eax,Remaining','ja HostFailure','sub Remaining,eax','add Cursor,eax','cmp Remaining,0','jne ReadMore',
 'mov Poison,0','call RunStage','test eax,eax','jnz HostFailure','mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected','mov ecx,736','rep movsb','mov Poison,0FFFFFFFFh','call RunStage','test eax,eax','jnz HostFailure',
 'mov esi,OFFSET ReplyBuffer','mov edi,OFFSET Expected','mov ecx,728','repe cmpsb','jne HostFailure',
 'mov Remaining,736','mov Cursor,OFFSET ReplyBuffer','WriteMore:','push 0','push OFFSET Transferred','push Remaining','push Cursor','push OutputHandle','call _WriteFile@20','test eax,eax','jz HostFailure','mov eax,Transferred','test eax,eax','jz HostFailure','cmp eax,Remaining','ja HostFailure','sub Remaining,eax','add Cursor,eax','cmp Remaining,0','jne WriteMore','jmp NextRecord','HostMain ENDP',
 'OriginalDone LABEL BYTE','HostFailure PROC','push 71','call _ExitProcess@4','HostFailure ENDP','OwnedEnd LABEL BYTE','END']
 text='\n'.join(lines)+'\n'
 public='PUBLIC NativeStack,Poison,Expected,'+','.join(n+'Shadow' for n in SIZES)+'\n'
 text=text.replace('.code\n',public+'.code\n',1)
 assert text.replace(public,'',1)==frozen_v2.host_source(fragment)
 return text
def build(owned,mapping,image):
 old=(carrier.BASE,carrier.source_pages)
 try:
  carrier.BASE,carrier.source_pages=BASE,source_pages
  return carrier.build(owned,mapping,image,proof())
 finally:carrier.BASE,carrier.source_pages=old

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--owned-exe',type=Path);p.add_argument('--owned-map',type=Path);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
 if a.owned_exe:
  assert a.owned_map;image=SourceImage(a.source_exe);blob,review=build(a.owned_exe,a.owned_map,image);(a.out/'stage-carrier-x86.exe').write_bytes(blob);review.update(native_executed=False,execution_admitted=False,carrier_sha256=sha(a.out/'stage-carrier-x86.exe'));(a.out/'carrier-build-review.json').write_text(json.dumps(review,indent=2)+'\n');return
 core.prepare(a.out);fragment=(a.out/'stage-host-core-x86.asm').read_text();(a.out/'host-x86.asm').write_text(host_source(fragment));m=json.loads((a.out/'manifest.json').read_text());m.update(schema='zero-child-root-stage-complete-native-preparation-v3',full_transport_prepared=True,tool_sha256=sha(__file__),carrier_mechanics_sha256=sha(carrier.__file__),asm_sha256=sha(a.out/'host-x86.asm'),carrier_base=f'{BASE:08X}',opaque_pointer_projection='Only replyrootword0 canonicalzero after source returns; native root pointer remainsownedbound.',compile_commands=[['ml.exe','/nologo','/c','/coff','/Fohost-x86.obj','host-x86.asm'],['link.exe','/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/BASE:0x29000000','/FIXED','/DYNAMICBASE:NO','/SAFESEH:NO','/OUT:host-x86.exe','/MAP:host-x86.map','host-x86.obj','kernel32.lib']],missing=['Root compile/link/carrier build','Independent linked audit+peer and protocol/native/reference comparisons'])
 (a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
if __name__=='__main__':main()

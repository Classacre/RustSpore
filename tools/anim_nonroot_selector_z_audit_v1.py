"""Independent actual-linked combined original Z publisher and full selector audit."""
import argparse,json,struct,hashlib
from pathlib import Path
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms,u32,roundup,IMPORTS
from native_crt_acos_probe import decoder
from cell_schema_probe import SourceImage
ROOT=Path(__file__).resolve().parent.parent
BASE=0x990000;OLD=0x29000000
SIZES={'Header':32,'Payload':90812,'Definition':1024,'Blocks':287640,'Query':32,'Indices':1020,'Reply':1040,'State':16,'Producer':52}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def audit(a):
 new,old=Image(a.carrier),Image(a.owned_exe);s=syms(a.owned_map);original=SourceImage(a.source_exe);proof=json.loads((a.prep/'source-closure.json').read_text())
 assert new.base==BASE and old.base==OLD
 assert struct.unpack_from('<H',new.b,new.nt+22)[0]&1 and not struct.unpack_from('<H',new.b,new.op+70)[0]&0x40
 assert new.b[:new.nt]==old.b[:old.nt] and all(d==(0,0)for i,d in enumerate(new.dirs())if i not in(1,12))
 ni,ne=new.imports();oi,oe=old.imports();assert ni==oi and ne==oe and set(ni.values())==IMPORTS and len(ni)==6
 for i in(1,12):rva,n=old.dirs()[i];assert new.dirs()[i]==(rva+OLD-BASE,n)
 assert BASE+u32(new.b,new.op+16)==s['HostMain']
 high=[z for z in new.sections if z['rs']and z['va']>=OLD];assert len(high)==len(old.sections)
 for before,after in zip(old.sections,high):
  assert all(before[k]==after[k]for k in('name','va','vs','rs','flags'))
  expected=bytearray(old.read(before['va'],before['vs']))
  for address in oe:
   if before['va']<=address<before['va']+before['vs']:
    j=address-before['va'];struct.pack_into('<I',expected,j,u32(expected,j)+OLD-BASE)
  assert new.read(after['va'],after['vs'])==expected
  assert new.b[after['raw']+after['vs']:after['raw']+after['rs']]==old.b[before['raw']+before['vs']:before['raw']+before['rs']]
 assert all(not(s['OwnedStart']<=at<s['OwnedEnd'])for at in oe)
 codebytes=new.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart']);assert codebytes==old.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart'])
 engine=decoder();from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM,X86_OP_MEM,X86_OP_REG,X86_REG_EAX,X86_REG_EBX,X86_REG_EBP
 pages={0x990000:bytearray(b'\xcc'*65536)};codepages={0x990000};native_rows=[];spans=0;source_bytes=6
 for r in proof['functions']:
  for span in r['ranges']:
   address=int(span['start'],16);n=span['bytes'];raw=original.read(address,n);assert hashlib.sha256(raw).hexdigest()==span['sha256'];source_bytes+=n;spans+=1
   assert new.read(s['Original_'+span['start']],n)==raw and new.perm(s['Original_'+span['start']],n)==0x40000000
   p=address&~0xffff;codepages.add(p);pages.setdefault(p,bytearray(b'\xcc'*65536));pages[p][address&0xffff:(address&0xffff)+n]=raw;part=list(engine.disasm(raw,address));assert sum(i.size for i in part)==n;native_rows+=part
 raw=original.read(0x11e0534,6);thunk=list(engine.disasm(raw,0x11e0534));assert len(thunk)==1 and thunk[0].mnemonic=='jmp'and thunk[0].operands[0].type==X86_OP_MEM and thunk[0].operands[0].mem.disp==0x13cc47c
 assert new.read(s['Original_011E0534'],6)==raw;codepages.add(0x11e0000);pages[0x11e0000]=bytearray(b'\xcc'*65536);pages[0x11e0000][0x534:0x53a]=raw
 for r in proof['data']:
  address=int(r['address'],16);n=r['bytes'];raw=original.read(address,n);assert hashlib.sha256(raw).hexdigest()==r['sha256'];assert new.read(s['Data_'+r['address']],n)==raw
  pages.setdefault(address&~0xffff,bytearray(65536));pages[address&~0xffff][address&0xffff:(address&0xffff)+n]=raw
 pages.setdefault(0x13c0000,bytearray(65536));struct.pack_into('<I',pages[0x13c0000],0xc47c,s['MemcpyProvider'])
 pages[0x9c0000][0x4992:0x4999]=b'\xb8'+struct.pack('<I',s['ProducerDone'])+b'\xff\xe0'
 payload=[z for z in new.sections if z['rs']and z['va']<OLD];assert len(payload)==len(pages)
 for z in payload:
  page=z['va']&~0xffff;skip=4096 if page==BASE else 0
  assert z['va']==page+skip and z['vs']==65536-skip and new.read(z['va'],z['vs'])==pages[page][skip:]
  assert z['flags']==(0x60000020 if page in codepages else 0x40000040)
 assert new.read(0x13cc47c,4)==struct.pack('<I',s['MemcpyProvider'])and new.perm(0x13cc47c,4)==0x40000000
 bounds={i.address for i in native_rows}|{0x9c4992};unreachable={0x9b03c0,0x9b2ef0}
 for r in native_rows:
  if r.group(CS_GRP_CALL)or r.group(CS_GRP_JUMP):assert r.operands[0].type==X86_OP_IMM and r.operands[0].imm in bounds|unreachable|{0x11e0534}
 assert all(new.read(x,1)==b'\xcc'for x in unreachable)
 gaps=[z for z in new.sections if not z['rs']];expected=[];allpayload=[z for z in new.sections if z['rs']]
 for b,z in zip(allpayload,allpayload[1:]):
  end=b['va']+roundup(b['vs'],4096)
  if end<z['va']:expected.append((end,z['va']-end))
 assert[(z['va'],z['vs'])for z in gaps]==expected and all(z['flags']==0x80 and z['raw']==0 for z in gaps)
 assert u32(new.b,new.op+12)==sum(z['vs']for z in gaps)
 ext=[]
 for name,n in SIZES.items():
  at=s[name+'Buffer'];assert at-s[name+'Before']==16 and s[name+'After']-at==n
  for lo,count in[(at-16,n+32),(s[name+'Shadow'],n)]:
   assert new.perm(lo,count)==0xc0000000 and all(lo+count<=x or y<=lo for x,y in ext);ext.append((lo,lo+count))
  assert new.read(at-16,16)==new.read(at+n,16)==b'\xa5'*16
 rows=list(engine.disasm(codebytes,s['OwnedStart']));assert sum(r.size for r in rows)==len(codebytes);ownedbounds={r.address for r in rows};entries=[];calls=[]
 for i,r in enumerate(rows):
  if not(r.group(CS_GRP_CALL)or r.group(CS_GRP_JUMP)):continue
  op=r.operands[0]
  if op.type==X86_OP_REG:
   assert r.mnemonic in('call','jmp')and op.reg==X86_REG_EAX and rows[i-1].mnemonic=='mov'and rows[i-1].operands[1].type==X86_OP_IMM
   target=rows[i-1].operands[1].imm;assert target in(0x9b2150,0x9b2e20,0x9c48db);entries.append(target);continue
  assert op.type==X86_OP_IMM
  if op.imm in ownedbounds:continue
  assert r.mnemonic=='call';t=new.read(op.imm,6);assert t[:2]==b'\xff\x25';calls.append((r.address,ni[u32(t,2)]))
 assert sorted(entries)==[0x9b2150,0x9b2e20,0x9c48db]
 module=next(x for x,n in calls if n=='GetModuleHandleA');read=next(x for x,n in calls if n=='ReadFile');assert s['HostMain']<module
 # ReadExact PROC lies earlier in image; verify startup's actual call to ReadExact
 # occurs only after byte/IAT checks, not by comparing ReadFile instruction VAs.
 main=[r for r in rows if s['HostMain']<=r.address<s['HostFailure']]
 idx=next(i for i,r in enumerate(main)if r.address==module);assert main[idx+1].mnemonic=='cmp'and main[idx+1].operands[1].imm==BASE
 first_read=next(r.address for r in main if r.mnemonic=='call'and r.operands[0].imm==s['ReadExact'])
 assert sum(r.mnemonic=='repe cmpsb'for r in main if module<r.address<first_read)==spans+len(proof['data'])+1
 provider=[r for r in rows if s['MemcpyProvider']<=r.address<s['RunQuery']]
 assert provider[0].mnemonic=='cmp'and provider[0].operands[0].mem.disp==0 and provider[0].operands[1].imm==0x9b221b
 assert any(r.mnemonic=='cmp'and r.op_str==f'dword ptr [esp + 4], 0x{s["IndicesBuffer"]:x}'for r in provider)
 assert any(r.mnemonic=='lea'and r.op_str=='eax, [esp + 0x160]'for r in provider)
 assert any(r.mnemonic=='cmp'and r.op_str=='ecx, 0x3fc'for r in provider)
 assert any(r.mnemonic=='test'and r.op_str=='ecx, 3'for r in provider)
 assert any(r.mnemonic=='lea'and r.op_str=='edx, [esi*4]'for r in provider)
 assert any(r.mnemonic=='cmp'and r.op_str=='ecx, edx'for r in provider)
 assert [r.op_str for r in provider if r.mnemonic=='push'and r.operands[0].type==X86_OP_REG]==['esi','edi']
 assert [r.op_str for r in provider if r.mnemonic=='pop'and r.operands[0].type==X86_OP_REG]==['eax','edi','esi']
 assert all(not(r.operands and r.operands[0].type==X86_OP_REG and r.operands[0].reg in(X86_REG_EBX,X86_REG_EBP))for r in provider)
 assert any(r.mnemonic=='rep movsb'for r in provider)and provider[-1].mnemonic=='ret'
 validate=[r for r in rows if s['ValidateQuery']<=r.address<s['MemcpyProvider']]
 assert any(r.mnemonic=='and'and r.op_str=='edx, 0xfff90f83'for r in validate)
 assert any(r.mnemonic=='cmp'and r.op_str=='eax, 0x1000'for r in validate)
 producer=[r for r in rows if s['RunProducer']<=r.address<s['HostMain']]
 assert any(r.mnemonic=='add'and r.op_str=='esi, 0x24'for r in producer)
 assert not any(r.mnemonic=='add'and r.op_str=='esi, 0x18'for r in producer)
 calls_to_producer=[r.address for r in main if r.mnemonic=='call'and r.operands[0].imm==s['RunProducer']]
 calls_to_selector=[r.address for r in main if r.mnemonic=='call'and r.operands[0].imm==s['RunQuery']]
 assert len(calls_to_producer)==2 and len(calls_to_selector)==2 and max(calls_to_producer)<min(calls_to_selector)
 assert new.read(0x9c4992,7)==b'\xb8'+struct.pack('<I',s['ProducerDone'])+b'\xff\xe0'
 assert source_bytes==8089
 failure=[r for r in rows if s['HostFailure']<=r.address<s['OwnedEnd']]
 assert failure[0].mnemonic=='cmp'and failure[0].operands[0].mem.disp==s['DriverStack']
 assert any(r.mnemonic=='mov'and r.op_str==f'esp, dword ptr [0x{s["DriverStack"]:x}]'for r in failure)
 return dict(schema='actual-linked-original-Z-publisher-all1644-selector-audit-v1',static_pass=True,native_executed=False,carrier_sha256=sha(a.carrier),owned_sha256=sha(a.owned_exe),map_sha256=sha(a.owned_map),source_bytes=source_bytes,native_pages=len(pages),virtual_gap_sections=len(gaps),adjacent_sections=True,source_and_slack_exact=True,owned_instructions_unchanged=True,original_memcpy_thunk_exact=True,readonly_IAT_provider_binding=True,provider_return_stack_pointer_size_nonoverlap_guard_instructions_present=True,provider_ESI_EDI_saved_EBX_EBP_untouched=True,source_CFG_closed_with_two_typed_unreachable_callees=True,original_producer_runs_twice_before_any_query=True,upper_stack_guard_offset=36,owned_control_closed=True,guards_closed=True,full_entries=[f'{x:08X}'for x in entries],header_bytes=new.headers,limits='Static linked gate only. All1644aliases/61policies; independent fullpartialABI/operationorder/provider peer and native/Rust/protocol acceptance required. No actor/constructor/wholebuilder admission.')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--carrier',type=Path,required=True);p.add_argument('--owned-exe',type=Path,required=True);p.add_argument('--owned-map',type=Path,required=True);p.add_argument('--prep',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));a=p.parse_args();r=audit(a)
 with a.out.open('x')as f:json.dump(r,f,indent=2);f.write('\n')
 print(json.dumps(r,indent=2))
if __name__=='__main__':main()

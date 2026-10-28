"""Independent actual-linked Z helper/caller sparse carrier audit. Static only."""
import argparse,json,struct,hashlib
from pathlib import Path
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms,u32,roundup,IMPORTS
from native_crt_acos_probe import decoder
from cell_schema_probe import SourceImage
ROOT=Path(__file__).resolve().parent.parent
BASE=0x990000;OLD=0x29000000
SIZES={'Header':32,'Payload':82668,'Definition':1024,'Blocks':287640,'State':16,'Reply':52}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def audit(a):
 new,old=Image(a.carrier),Image(a.owned_exe);s=syms(a.owned_map);source=SourceImage(a.source_exe);p=json.loads((a.prep/'source-closure.json').read_text())
 assert new.base==BASE and old.base==OLD and new.b[:new.nt]==old.b[:old.nt]
 assert struct.unpack_from('<H',new.b,new.nt+22)[0]&1 and not struct.unpack_from('<H',new.b,new.op+70)[0]&0x40
 assert all(d==(0,0)for i,d in enumerate(new.dirs())if i not in(1,12))
 ni,ne=new.imports();oi,oe=old.imports();assert ni==oi and ne==oe and set(ni.values())==IMPORTS and len(ni)==6
 for i in(1,12):r,n=old.dirs()[i];assert new.dirs()[i]==(r+OLD-BASE,n)
 assert BASE+u32(new.b,new.op+16)==s['HostMain']
 high=[z for z in new.sections if z['rs']and z['va']>=OLD];assert len(high)==len(old.sections)
 for b,z in zip(old.sections,high):
  assert all(b[k]==z[k]for k in('name','va','vs','rs','flags'));expected=bytearray(old.read(b['va'],b['vs']))
  for at in oe:
   if b['va']<=at<b['va']+b['vs']:j=at-b['va'];struct.pack_into('<I',expected,j,u32(expected,j)+OLD-BASE)
  assert new.read(z['va'],z['vs'])==expected
  assert new.b[z['raw']+z['vs']:z['raw']+z['rs']]==old.b[b['raw']+b['vs']:b['raw']+b['rs']]
 assert all(not(s['OwnedStart']<=at<s['OwnedEnd'])for at in oe)
 code=new.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart']);assert code==old.read(s['OwnedStart'],len(code))
 engine=decoder();from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM,X86_OP_REG,X86_REG_EAX
 pages={BASE:bytearray(b'\xcc'*65536)};codepages={BASE};native=[];total=0;spans=0
 for f in p['functions']:
  assert f['entry']in('009B6920','009C48DB')
  for z in f['ranges']:
   at=int(z['start'],16);n=z['bytes'];raw=source.read(at,n);assert hashlib.sha256(raw).hexdigest()==z['sha256'];total+=n;spans+=1
   assert new.read(s['Original_'+z['start']],n)==raw and new.perm(s['Original_'+z['start']],n)==0x40000000
   page=at&~0xffff;pages.setdefault(page,bytearray(b'\xcc'*65536));codepages.add(page);pages[page][at&0xffff:(at&0xffff)+n]=raw
   rows=list(engine.disasm(raw,at));assert sum(x.size for x in rows)==n;native+=rows
 assert total==3396 and spans==3
 for z in p['data']:
  at=int(z['address'],16);n=z['bytes'];raw=source.read(at,n);assert hashlib.sha256(raw).hexdigest()==z['sha256'];assert new.read(s['Data_'+z['address']],n)==raw
  pages.setdefault(at&~0xffff,bytearray(65536))[at&0xffff:(at&0xffff)+n]=raw
 exitraw=b'\xb8'+struct.pack('<I',s['WriterDone'])+b'\xff\xe0';pages[0x9c0000][0x4992:0x4999]=exitraw;assert new.read(0x9c4992,7)==exitraw
 payload=[z for z in new.sections if z['rs']and z['va']<OLD];assert len(payload)==len(pages)
 for z in payload:
  page=z['va']&~0xffff;skip=4096 if page==BASE else 0
  assert z['va']==page+skip and z['vs']==65536-skip and new.read(z['va'],z['vs'])==pages[page][skip:]
  assert z['flags']==(0x60000020 if page in codepages else 0x40000040)
 bounds={r.address for r in native}|{0x9c4992}
 for r in native:
  if r.group(CS_GRP_CALL)or r.group(CS_GRP_JUMP):assert r.operands[0].type==X86_OP_IMM and r.operands[0].imm in bounds
 assert [(r.address,r.operands[0].imm)for r in native if r.group(CS_GRP_CALL)]==[(0x9c490d,0x9b6920)]
 for b,z in zip(new.sections,new.sections[1:]):assert b['va']+roundup(b['vs'],4096)==z['va']
 assert new.sections[0]['va']==BASE+4096
 gaps=[z for z in new.sections if not z['rs']];assert all(z['flags']==0x80 and z['raw']==0 for z in gaps)
 assert u32(new.b,new.op+12)==sum(z['vs']for z in gaps)
 ext=[]
 for name,n in SIZES.items():
  at=s[name+'Buffer'];assert at-s[name+'Before']==16 and s[name+'After']-at==n
  for lo,count in[(at-16,n+32),(s[name+'Shadow'],n)]:
   assert new.perm(lo,count)==0xc0000000 and all(lo+count<=x or y<=lo for x,y in ext);ext.append((lo,lo+count))
  assert new.read(at-16,16)==new.read(at+n,16)==b'\xa5'*16
 rows=list(engine.disasm(code,s['OwnedStart']));assert sum(r.size for r in rows)==len(code);owned={r.address for r in rows};entries=[];calls=[]
 for i,r in enumerate(rows):
  if not(r.group(CS_GRP_CALL)or r.group(CS_GRP_JUMP)):continue
  op=r.operands[0]
  if op.type==X86_OP_REG:
   assert r.mnemonic=='jmp'and op.reg==X86_REG_EAX and rows[i-1].mnemonic=='mov'and rows[i-1].operands[1].type==X86_OP_IMM
   assert rows[i-1].operands[1].imm==0x9c48db;entries.append(0x9c48db);continue
  assert op.type==X86_OP_IMM
  if op.imm in owned:continue
  assert r.mnemonic=='call';t=new.read(op.imm,6);assert t[:2]==b'\xff\x25';calls.append((r.address,ni[u32(t,2)]))
 assert entries==[0x9c48db]
 module=next(at for at,n in calls if n=='GetModuleHandleA');main=[r for r in rows if s['HostMain']<=r.address<s['HostFailure']];i=next(i for i,r in enumerate(main)if r.address==module)
 assert main[i+1].mnemonic=='cmp'and main[i+1].operands[1].imm==BASE
 firstread=next(r.address for r in main if r.mnemonic=='call'and r.operands[0].imm==s['ReadExact'])
 assert sum(r.mnemonic=='repe cmpsb'for r in main if module<r.address<firstread)==spans+len(p['data'])
 failure=[r for r in rows if s['HostFailure']<=r.address<s['OwnedEnd']];assert failure[0].mnemonic=='cmp'and failure[0].operands[0].mem.disp==s['DriverStack']
 assert any(r.mnemonic=='mov'and r.op_str==f'esp, dword ptr [0x{s["DriverStack"]:x}]'for r in failure)
 return dict(schema='actual-linked-Z-producer-independent-audit-v1',static_pass=True,native_executed=False,carrier_sha256=sha(a.carrier),owned_sha256=sha(a.owned_exe),map_sha256=sha(a.owned_map),source_bytes=3396,source_spans=3,native_pages=len(pages),gap_sections=len(gaps),adjacent_sections=True,source_and_slack_exact=True,owned_instructions_unchanged=True,source_CFG_closed=True,full_helper='009B6920',partial_caller='009C48DB..009C4992',original_callee_calls=1,owned_exit_bytes=7,source_entry_registers_and_real_stack_require_independent_peer=True,guards_closed=True,owned_control_closed=True,header_bytes=new.headers,limits='Static linked gate only; caller skips original build and stops before diagonal/constraints. No native/numeric/actor admission.')
def main():
 p=argparse.ArgumentParser(description=__doc__)
 for n in('carrier','owned-exe','owned-map','prep','out'):p.add_argument('--'+n,type=Path,required=True)
 p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));a=p.parse_args();r=audit(a)
 with a.out.open('x')as f:json.dump(r,f,indent=2);f.write('\n')
 print(json.dumps(r,indent=2))
if __name__=='__main__':main()

"""Independent static linked audit of the zero-child full-entry stage carrier."""
import argparse,json,hashlib,struct
from pathlib import Path
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms,u32,roundup,IMPORTS
from native_crt_acos_probe import decoder
from cell_schema_probe import SourceImage
ROOT=Path(__file__).resolve().parent.parent
BASE=0x990000;OLD=0x29000000
SIZES={'Input':1256,'Root':700,'Definition':524,'Context':128,'Body':748,'Orientation':16,'DefaultPosition':12,'Reply':736}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def audit(a):
 new,old=Image(a.carrier),Image(a.owned_exe);s=syms(a.owned_map);original=SourceImage(a.source_exe)
 assert new.base==BASE and old.base==OLD
 assert struct.unpack_from('<H',new.b,new.nt+22)[0]&1 and not struct.unpack_from('<H',new.b,new.op+70)[0]&0x40
 assert new.b[:new.nt]==old.b[:old.nt]
 assert all(d==(0,0) for i,d in enumerate(new.dirs()) if i not in(1,12))
 ni,ne=new.imports();oi,oe=old.imports();assert ni==oi and ne==oe and set(ni.values())==IMPORTS and len(ni)==6
 for i in(1,12):rva,n=old.dirs()[i];assert new.dirs()[i]==(rva+OLD-BASE,n)
 assert BASE+u32(new.b,new.op+16)==s['HostMain']
 high=[z for z in new.sections if z['rs'] and z['va']>=OLD];assert len(high)==len(old.sections)
 for before,after in zip(old.sections,high):
  assert all(before[k]==after[k] for k in('name','va','vs','rs','flags'))
  expected=bytearray(old.read(before['va'],before['vs']))
  for address in oe:
   if before['va']<=address<before['va']+before['vs']:
    at=address-before['va'];struct.pack_into('<I',expected,at,u32(expected,at)+OLD-BASE)
  assert new.read(after['va'],after['vs'])==expected
  assert new.b[after['raw']+after['vs']:after['raw']+after['rs']]==old.b[before['raw']+before['vs']:before['raw']+before['rs']]
 assert all(not(s['OwnedStart']<=x<s['OwnedEnd']) for x in oe)
 codebytes=new.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart']);assert codebytes==old.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart'])
 closure=json.loads((ROOT/'target/animation/root-zero-child-staging-source-v2-reviewed/source-closure.json').read_text())
 codepages={int(span['start'],16)&~0xffff for r in closure['source_records'] for span in r['ranges']}
 codepages|={int(x,16)&~0xffff for x in closure['excluded_calls']};datapages={int(r['address'],16)&~0xffff for r in closure['data']};assert not codepages&datapages
 pages={p:bytearray(b'\xcc'*65536 if p in codepages else bytes(65536)) for p in codepages|datapages};source_count=0
 for r in closure['source_records']:
  for span in r['ranges']:
   address=int(span['start'],16);n=span['bytes'];data=original.read(address,n);assert hashlib.sha256(data).hexdigest()==span['sha256'];pages[address&~0xffff][address&0xffff:(address&0xffff)+n]=data;source_count+=n
   assert new.read(s['Original_'+span['start']],n)==data and new.perm(s['Original_'+span['start']],n)==0x40000000
 for r in closure['data']:
  address=int(r['address'],16);n=r['bytes'];data=original.read(address,n);assert hashlib.sha256(data).hexdigest()==r['sha256'];pages[address&~0xffff][address&0xffff:(address&0xffff)+n]=data
  assert new.read(s['Data_'+r['address']],n)==data
 payload=[z for z in new.sections if z['rs'] and z['va']<OLD];assert len(payload)==len(pages)
 for z in payload:
  page=z['va']&~0xffff;skip=4096 if page==BASE else 0
  assert z['va']==page+skip and z['vs']==65536-skip and new.read(z['va'],z['vs'])==pages[page][skip:]
  assert z['flags']==(0x60000020 if page in codepages else 0x40000040)
 gaps=[z for z in new.sections if not z['rs']];expected=[];allpayload=[z for z in new.sections if z['rs']]
 for before,after in zip(allpayload,allpayload[1:]):
  end=before['va']+roundup(before['vs'],4096)
  if end<after['va']:expected.append((end,after['va']-end))
 assert [(z['va'],z['vs']) for z in gaps]==expected and all(z['flags']==0x80 and z['raw']==0 for z in gaps)
 assert u32(new.b,new.op+12)==sum(z['vs'] for z in gaps)
 ext=[]
 for name,n in SIZES.items():
  at=s[name+'Buffer'];assert at-s[name+'Before']==16 and s[name+'After']-at==n
  for lo,count in[(at-16,n+32),(s[name+'Shadow'],n)]:
   assert new.perm(lo,count)==0xc0000000 and all(lo+count<=x or y<=lo for x,y in ext);ext.append((lo,lo+count))
  assert new.read(at-16,16)==new.read(at+n,16)==b'\xa5'*16
 engine=decoder() # initialize the retained Capstone dependency path before imports
 from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM,X86_OP_REG,X86_REG_EAX
 rows=list(engine.disasm(codebytes,s['OwnedStart']));assert sum(r.size for r in rows)==len(codebytes);bounds={r.address for r in rows};entries=[];calls=[]
 for i,r in enumerate(rows):
  if not(r.group(CS_GRP_CALL) or r.group(CS_GRP_JUMP)):continue
  op=r.operands[0]
  if op.type==X86_OP_REG:
   assert r.mnemonic=='call' and op.reg==X86_REG_EAX and rows[i-1].mnemonic=='mov' and rows[i-1].operands[1].type==X86_OP_IMM
   address=rows[i-1].operands[1].imm;assert address in(0x9e82e0,0x9e8930,0x9bd370);entries.append(address);continue
  assert op.type==X86_OP_IMM
  if op.imm in bounds:continue
  assert r.mnemonic=='call';thunk=new.read(op.imm,6);assert thunk[:2]==b'\xff\x25';calls.append((r.address,ni[u32(thunk,2)]))
 assert sorted(entries)==[0x9bd370,0x9e82e0,0x9e8930]
 module=next(x for x,n in calls if n=='GetModuleHandleA');read=next(x for x,n in calls if n=='ReadFile');assert s['HostMain']<module<read
 k=next(i for i,r in enumerate(rows) if r.address==module);assert rows[k+1].mnemonic=='cmp' and rows[k+1].operands[1].imm==BASE and rows[k+2].operands[0].imm==s['HostFailure']
 assert sum(1 for r in rows if module<r.address<read and r.mnemonic=='repe cmpsb')==sum(len(r['ranges']) for r in closure['source_records'])+len(closure['data'])
 # Direct topology guard uses actual supplied definition word, never overwrites it.
 validate=[r for r in rows if s['ValidateStage']<=r.address<s['RunStage']]
 assert any(r.mnemonic=='cmp' and r.op_str==f'dword ptr [0x{s["InputBuffer"]+1252:x}], 0' for r in validate)
 assert any(r.mnemonic=='mov' and r.op_str==f'dword ptr [0x{s["ReplyBuffer"]:x}], 0' for r in rows)
 return dict(schema='independent-zero-child-root-stage-carrier-linked-audit-v2',static_pass=True,native_executed=False,carrier_sha256=sha(a.carrier),owned_exe_sha256=sha(a.owned_exe),map_sha256=sha(a.owned_map),source_bytes=source_count,source_pages=len(pages),virtual_gap_sections=len(gaps),adjacent_sections=True,source_and_slack_exact=True,owned_instructions_unchanged=True,imports_metadata_only=True,guards_closed=True,actual_zero_child_guard=True,full_entries=[f'{x:08X}' for x in entries],header_bytes=new.headers,canonical_reply_pointer_only=True,limits='Static linked source/control/memory closure only; independent peer and native numeric/protocol acceptance required. Explicit supplied zero-child topology, no generated-body/actor admission.')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--carrier',type=Path,required=True);p.add_argument('--owned-exe',type=Path,required=True);p.add_argument('--owned-map',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));a=p.parse_args();r=audit(a)
 with a.out.open('x') as f:json.dump(r,f,indent=2);f.write('\n')
 print(json.dumps(r,indent=2))
if __name__=='__main__':main()

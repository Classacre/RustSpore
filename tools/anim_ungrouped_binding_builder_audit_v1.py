"""Independent linked full9A1E00 prepared-field gate audit. Static only."""
import argparse,json,struct,hashlib
from pathlib import Path
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms,u32,roundup,IMPORTS
from native_crt_acos_probe import decoder
from cell_schema_probe import SourceImage
BASE=0x670000;OLD=0x29000000
SIZES={'Header':32,'Payload':94492,'Definition':1024,'Blocks':287640,'Query':32,'Indices':1020,'Reply':114292,'State':16,'Producer':52,'Clip':348,'Channels':3192,'Descriptors':10752,'ChannelPtrs':56,'Output':40,'Records':114240}
TRAPS={0x67c860,0x99c5e0,0x99ed80,0x9a0980,0x9a1980,0x9a1b80,0xf47290,0xf634b0,0x11e0a70,0x9b03c0,0x9b2ef0}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def audit(a):
 new,old=Image(a.carrier),Image(a.owned_exe);s=syms(a.owned_map);original=SourceImage(a.source_exe);p=json.loads((a.prep/'source-closure.json').read_text());engine=decoder()
 from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM,X86_OP_MEM,X86_OP_REG,X86_REG_EAX
 assert new.base==BASE and old.base==OLD and new.b[:new.nt]==old.b[:old.nt]
 assert struct.unpack_from('<H',new.b,new.nt+22)[0]&1 and not struct.unpack_from('<H',new.b,new.op+70)[0]&0x40
 assert all(d==(0,0)for i,d in enumerate(new.dirs())if i not in(1,12))
 ni,ne=new.imports();oi,oe=old.imports();assert ni==oi and ne==oe and set(ni.values())==IMPORTS and len(ni)==6
 for i in(1,12):r,n=old.dirs()[i];assert new.dirs()[i]==(r+OLD-BASE,n)
 assert BASE+u32(new.b,new.op+16)==s['HostMain']
 high=[z for z in new.sections if z['rs']and z['va']>=OLD];assert len(high)==len(old.sections)
 for before,after in zip(old.sections,high):
  assert all(before[k]==after[k]for k in('name','va','vs','rs','flags'));expected=bytearray(old.read(before['va'],before['vs']))
  for at in oe:
   if before['va']<=at<before['va']+before['vs']:j=at-before['va'];struct.pack_into('<I',expected,j,u32(expected,j)+OLD-BASE)
  assert new.read(after['va'],after['vs'])==expected
  assert new.b[after['raw']+after['vs']:after['raw']+after['rs']]==old.b[before['raw']+before['vs']:before['raw']+before['rs']]
 assert all(not(s['OwnedStart']<=at<s['OwnedEnd'])for at in oe)
 code=new.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart']);assert code==old.read(s['OwnedStart'],len(code))
 pages={BASE:bytearray(b'\xcc'*65536)};codepages={BASE};native=[];sourcebytes=6;spans=0
 for f in p['functions']:
  for r in f['ranges']:
   at=int(r['start'],16);n=r['bytes'];raw=original.read(at,n);assert hashlib.sha256(raw).hexdigest()==r['sha256'];sourcebytes+=n;spans+=1
   assert new.read(s['Original_'+r['start']],n)==raw and new.perm(s['Original_'+r['start']],n)==0x40000000
   page=at&~0xffff;pages.setdefault(page,bytearray(b'\xcc'*65536))[at&0xffff:(at&0xffff)+n]=raw;codepages.add(page);part=list(engine.disasm(raw,at));assert sum(x.size for x in part)==n;native+=part
 assert sourcebytes==13380 and p['source_bytes']==13380
 raw=original.read(0x11e0534,6);assert struct.unpack_from('<I',raw,2)[0]==0x13cc47c and new.read(s['Original_011E0534'],6)==raw
 pages[0x11e0000][0x534:0x53a]=raw
 for at in TRAPS:pages.setdefault(at&~0xffff,bytearray(b'\xcc'*65536));codepages.add(at&~0xffff)
 for d in p['data']:
  at=int(d['address'],16);n=d['bytes'];raw=original.read(at,n);assert hashlib.sha256(raw).hexdigest()==d['sha256']and new.read(s['Data_'+d['address']],n)==raw
  pages.setdefault(at&~0xffff,bytearray(65536))[at&0xffff:(at&0xffff)+n]=raw
 pages.setdefault(0x13c0000,bytearray(65536));struct.pack_into('<I',pages[0x13c0000],0xc478,s['MemsetProvider']);struct.pack_into('<I',pages[0x13c0000],0xc47c,s['MemcpyProvider']);pages[0x9c0000][0x4992:0x4999]=b'\xb8'+struct.pack('<I',s['ProducerDone'])+b'\xff\xe0'
 low=[z for z in new.sections if z['rs']and z['va']<OLD];assert len(low)==len(pages)
 for z in low:
  page=z['va']&~0xffff;skip=4096 if page==BASE else 0;assert z['va']==page+skip and z['vs']==65536-skip and new.read(z['va'],z['vs'])==pages[page][skip:]
  assert z['flags']==(0x60000020 if page in codepages else 0x40000040)
 assert all(new.read(at,1)==b'\xcc'for at in TRAPS)
 bounds={r.address for r in native}|{0x11e0534,0x9c4992}
 for r in native:
  if not(r.group(CS_GRP_CALL)or r.group(CS_GRP_JUMP)):continue
  op=r.operands[0]
  if op.type==X86_OP_IMM:assert op.imm in bounds|TRAPS
  elif r.address==0x99c35d:assert op.type==X86_OP_MEM and op.mem.disp==0x99c3a4 and all(x in bounds for x in struct.unpack('<6I',new.read(0x99c3a4,24)))
  elif r.address==0x99c44c:assert op.type==X86_OP_REG and op.reg==X86_REG_EAX # typed rawmask0 callback unreachable
  else:assert r.address==0x11e052e and op.type==X86_OP_MEM and op.mem.disp==0x13cc478
 assert new.read(0x11e052e,6)==original.read(0x11e052e,6)
 assert p['original_memset_import']==dict(IAT='013CC478',dll='MSVCR90.dll',symbol='memset',targeted_original_import_verified=True)
 for b,z in zip(new.sections,new.sections[1:]):assert b['va']+roundup(b['vs'],4096)==z['va']
 assert new.sections[0]['va']==BASE+4096
 gaps=[z for z in new.sections if not z['rs']];assert all(z['flags']==0x80 and z['raw']==0 for z in gaps)and u32(new.b,new.op+12)==sum(z['vs']for z in gaps)
 ext=[]
 for name,n in SIZES.items():
  at=s[name+'Buffer'];assert at-s[name+'Before']==16 and s[name+'After']-at==n
  for lo,count in[(at-16,n+32),(s[name+'Shadow'],n)]:assert new.perm(lo,count)==0xc0000000 and all(lo+count<=x or y<=lo for x,y in ext);ext.append((lo,lo+count))
  assert new.read(at-16,16)==new.read(at+n,16)==b'\xa5'*16
 rows=list(engine.disasm(code,s['OwnedStart']));assert sum(r.size for r in rows)==len(code);owned={r.address for r in rows};entries=[];calls=[]
 for i,r in enumerate(rows):
  if not(r.group(CS_GRP_CALL)or r.group(CS_GRP_JUMP)):continue
  op=r.operands[0]
  if op.type==X86_OP_REG:
   assert op.reg==X86_REG_EAX and rows[i-1].mnemonic=='mov'and rows[i-1].operands[1].type==X86_OP_IMM;target=rows[i-1].operands[1].imm;assert target in(0x9b2e20,0x9c48db,0x9a1e00);entries.append(target);continue
  assert op.type==X86_OP_IMM
  if op.imm in owned:continue
  assert r.mnemonic=='call';thunk=new.read(op.imm,6);assert thunk[:2]==b'\xff\x25';calls.append((r.address,ni[u32(thunk,2)]))
 assert sorted(entries)==[0x9a1e00,0x9b2e20,0x9c48db]
 main=[r for r in rows if s['HostMain']<=r.address<s['HostFailure']];module=next(at for at,name in calls if name=='GetModuleHandleA');i=next(i for i,r in enumerate(main)if r.address==module);assert main[i+1].mnemonic=='cmp'and main[i+1].operands[1].imm==BASE
 firstread=next(r.address for r in main if r.mnemonic=='call'and r.operands[0].imm==s['ReadExact']);assert sum(r.mnemonic=='repe cmpsb'for r in main if module<r.address<firstread)==spans+len(p['data'])+1
 prodcalls=[r.address for r in main if r.mnemonic=='call'and r.operands[0].imm==s['RunProducer']];wholecalls=[r.address for r in main if r.mnemonic=='call'and r.operands[0].imm==s['RunWhole']];assert len(prodcalls)==len(wholecalls)==2 and max(prodcalls)<min(wholecalls)
 def section(a,b):return[r for r in rows if s[a]<=r.address<s[b]]
 copy=section('MemcpyProvider','RunProducer');assert copy[0].mnemonic=='cmp'and copy[0].operands[1].imm==0x9b221b
 assert any(r.mnemonic=='mov'and r.op_str==f'eax, dword ptr [0x{s["MemcpyDestination"]:x}]'for r in copy)
 assert any(r.mnemonic=='lea'and r.op_str=='eax, [esp + 0x160]'for r in copy)and any(r.mnemonic=='cmp'and r.op_str=='ecx, 0x3fc'for r in copy)
 fill=section('MemsetProvider','RunWhole');assert fill[0].mnemonic=='cmp'and fill[0].operands[1].imm==0x9a1edf
 for mnemonic,operand in [('sub','eax, 0x80'),('cmp','ebp, eax'),('lea','eax, [ebp - 0x1954]'),('cmp','dword ptr [esp + 8], 0xff'),('cmp','dword ptr [esp + 0xc], 0x8f7')]:assert any(r.mnemonic==mnemonic and r.op_str==operand for r in fill),(mnemonic,operand)
 prepared=section('PrepareCase','MemsetProvider')
 for off in(0x154,0x158):assert any(r.mnemonic=='mov'and r.op_str==f'dword ptr [0x{s["ClipBuffer"]+off:x}], 0'for r in prepared)
 for operand in('byte ptr [edi + 0xaf], 0','dword ptr [edi + 0x9c], 2','dword ptr [esi + 0x10], 9'):
  assert any(r.mnemonic=='cmp'and r.op_str==operand for r in prepared),operand
 assert any(r.mnemonic=='cmp'and r.op_str=='eax, 0x18'for r in prepared)
 whole=section('RunWhole','HostMain')
 assert any(r.mnemonic=='sub'and r.op_str=='esp, 0x1000'for r in whole)and any(r.mnemonic=='test'and r.op_str=='dword ptr [esp], eax'for r in whole)
 assert any(r.mnemonic=='sub'and r.op_str=='eax, 0x1de8'for r in whole)
 assert new.read(0x13cc478,4)==struct.pack('<I',s['MemsetProvider'])and new.perm(0x13cc478,8)==0x40000000
 failure=section('HostFailure','OwnedEnd');assert any(r.mnemonic=='mov'and r.op_str==f'esp, dword ptr [0x{s["DriverStack"]:x}]'for r in failure)
 return dict(schema='whole9A1E00-prepared-ungrouped-independent-linked-audit-v1',static_pass=True,native_executed=False,source_bytes=13380,native_pages=len(pages),virtual_gap_sections=len(gaps),header_bytes=new.headers,source_and_slack_exact=True,adjacent_sections=True,owned_instructions_unchanged=True,owned_imports=6,original_source_CFG_closed_except_typed_unreachable_rawmask0_callback=True,source_traps_checked=len(TRAPS),original_memset_thunk_exact=True,original_memcpy_thunk_exact=True,readonly_two_IAT_valuefree_providers=True,real_stack_page_probing_present=True,original_alloca_probe43_retained=True,provider_frame_dest_return_size_guard_instructions_present=True,guarded_buffers_closed=True,owned_control_closed=True,full_entries=['009A1E00','009B2E20'],partial_entry='009C48DB..009C4992 (notfull9C48C0)',producer_passes_before_whole_entry=2,whole_entry_poison_passes=2,carrier_sha256=sha(a.carrier),owned_sha256=sha(a.owned_exe),map_sha256=sha(a.owned_map),limits='Static gate only; independent maxframe/ABI,exactsecondaryFF/AF0/rawmask0/7140capacity branch proofs,sourceoperationorder and numeric/native/protocol gates remain required. Preparedfield snapshots do not establish fullactor/object allocation.')
def main():
 p=argparse.ArgumentParser(description=__doc__)
 for n in('carrier','owned-exe','owned-map','prep','out'):p.add_argument('--'+n,type=Path,required=True)
 p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));a=p.parse_args();r=audit(a)
 with a.out.open('x')as f:json.dump(r,f,indent=2);f.write('\n')
 print(json.dumps(r,indent=2))
if __name__=='__main__':main()


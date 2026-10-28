"""Default-preflight all1644 original-producer/selector runner; no automatic execution."""
import argparse,json,struct,hashlib
from pathlib import Path
from types import SimpleNamespace
import anim_nonroot_selector_z_audit_v1 as audit
from cell_runtime_data_native_run import digest,pinned,saved_file_run,word
ROOT=Path(__file__).resolve().parent.parent
CARRIER='f3fdf07bd8700a3dafbcc4d93010f7b29800f4e5b2efa90f9515873246505698'
OWNED='75bbb23cc381951ade1718b3dc14b21436b83b84552ddd19c8c90f29b20f8807'
MAP='4a2c5dcac421ade7a54e7c901126803fc8d2057f9491fc66ac8de5b335666a4a'
INPUT='3e04e0c1a7766ad80f85d1c4920240f9c904f1e5007b53002b7e611cd9312c63'
EXPECTED='4cdceca9ffb0bb2d28b026cd13919a7bc77333734b487b0f6757eabb00a52259'
PREP=ROOT/'target/animation/nonroot-selector-Z-native-v1-complete'
OLD=ROOT/'target/animation/nonroot-selector-inputs-v3'
ZRUN=ROOT/'target/animation/selector-Z-producer-native-run-v2/native-v1'
FINAL=ROOT/'target/animation/body-finalizer-fields-native-v3-run/native-projection.bin'
CONTROLS=[0x7f|p|r for p in(0,0x200,0x300)for r in(0,0x400,0x800,0xc00)]
FIELDS=[0x104,0x108,0x1f8,0x1fc,0x200,0x21c,0x208,0x10c,0x110,0x114,0x118,0x11c,0x120,0x124,0x128,0x12c,0x130,0x134,0x138,0x13c,0x140,0x144,0x148,0x14c,0x344,0x348,0x34c,0x350,0x354,0x358,0x35c,0x360,0x364,0x368,0x36c,0x378,0x37c,0x204,0x20c]
OFF={x:168+i*4 for i,x in enumerate(FIELDS)}
FILES=['tools/anim_nonroot_selector_z_native_probe_v1.py','tools/anim_nonroot_selector_z_audit_v1.py','tools/anim_nonroot_selector_z_prepare_v1.py','tools/anim_nonroot_selector_native_probe_v2.py','tools/anim_selector_z_bounds_native_probe_v1.py','tools/anim_root_pose_sparse_carrier_v3.py','tools/anim_root_pose_sparse_carrier_audit_v2.py','tools/cell_runtime_data_native_run.py','tools/cell_schema_probe.py','tools/native_crt_acos_probe.py','crates/spore-core/src/anim_nonroot_selector.rs','crates/spore-core/src/anim_nonroot_selector_z.rs','crates/spore-core/src/anim_selector_z_bounds.rs','crates/spore-core/examples/anim_nonroot_selector_z_verify.rs.pending','crates/spore-core/src/native_sse.rs','crates/spore-core/src/native_x87.rs']
def resolve(p):return Path(p)if Path(p).is_absolute()else ROOT/p
def save(p,v):
 with p.open('x')as f:json.dump(v,f,indent=2);f.write('\n')
def peer(path,required,runner=False):
 if path is None:
  assert not required,'Independent actual-linked and supplemental runner peer reports required'
  return dict(admitted=False,reason='runner_peer_pending'if runner else'actual_linked_peer_pending')
 r=json.loads(path.read_text());assert r['no_blocking_static_issue_found']is True and r['native_executed']is False
 for name,value in r['pins'].items():assert digest(resolve(name))==value,name
 assert CARRIER in r['pins'].values()and OWNED in r['pins'].values()
 if runner:assert digest(__file__)in r['pins'].values()
 return dict(admitted=True,path=str(path.resolve()),sha256=digest(path))
def check_header(h):assert len(h)==32 and word(h,0)==0x314c4553 and 1<=word(h,4)<=255 and 1<=word(h,8)<=256 and word(h,12)in CONTROLS and h[20:]==bytes(12)
def check_query(q):
 assert len(q)==32 and word(q,8)==word(q,12)==word(q,24)==0 and word(q,16)<=1 and word(q,20)<=255
 flags=word(q,0)
 if flags&0x40:assert flags in(0x4c,0x1040);return
 assert flags&~0x6f03c==0 and flags&0x2000c in(0,4,8,12,0x20000,0x20004,0x20008)and flags&0x40030 in(0,0x10,0x20,0x30,0x40000,0x40010,0x40020)and flags&0xe000 in(0,0x2000,0x4000,0x6000,0x8000)
 assert flags&0x801c00==0 or flags==0x1000

def review(a):
 m=json.loads((a.inputs/'manifest.json').read_text());assert(m['bodies'],m['blocks'],m['source_aliases'],m['admitted_source_aliases'],m['unique_selector_words'],m['queries'])==(1322,46256,1644,1644,61,161284)
 assert m['retained_Z_exclusions']==0 and m['body_upstream_exclusions']==43 and m['old_155996_exact_slots_preserved']==155996 and m['new_Z_queries']==5288 and m['six_published_fields_identical_across12CWs']is True
 for name,value in m['pins'].items():assert digest(resolve(name))==value,name
 for name,value in m['files'].items():assert digest(a.inputs/name)==value,name
 for name,value in json.loads((PREP/'preparation-review.json').read_text())['pins'].items():assert digest(resolve(name))==value,name
 upstream=json.loads((OLD/'manifest.json').read_text())
 for name,value in upstream['pins'].items():assert digest(resolve(name))==value,name
 assert digest(a.inputs/'input.bin')==INPUT and digest(a.inputs/'expected1032.bin')==EXPECTED
 assert digest(a.host_dir/'selector-Z-carrier-x86.exe')==CARRIER and digest(a.host_dir/'host-x86.exe')==OWNED and digest(a.host_dir/'host-x86.map')==MAP
 assert(a.host_dir/'host-x86.asm').read_bytes()==(PREP/'host-x86.asm').read_bytes()
 actual=audit.audit(SimpleNamespace(carrier=a.host_dir/'selector-Z-carrier-x86.exe',owned_exe=a.host_dir/'host-x86.exe',owned_map=a.host_dir/'host-x86.map',prep=a.host_dir,source_exe=a.source_exe))
 assert actual==json.loads((a.host_dir/'root-linked-audit-v1.json').read_text())and actual['source_bytes']==8089 and actual['native_pages']==10 and actual['original_producer_runs_twice_before_any_query']and actual['upper_stack_guard_offset']==36
 idx=json.loads((a.inputs/'body-index.json').read_text());oldidx=json.loads((OLD/'body-index.json').read_text());selectors=json.loads((a.inputs/'selector-table.json').read_text());aliases=json.loads((a.inputs/'selector-aliases.json').read_text());bounds=json.loads((a.inputs/'native-definition-bounds.json').read_text());assert len(idx)==len(oldidx)==len(bounds)==1322 and len(selectors)==61 and len(aliases)==1644 and all(x['admitted']for x in aliases)
 assert {tuple(x['primary'])for x in aliases}=={tuple(x)for x in selectors}
 # Rejoin raw native publisher output, all12policies, not Rust-derived fields.
 with(ZRUN/'native-projection44.bin').open('rb')as z:
  for i,b in enumerate(bounds):
   assert b['body']==i;chosen=None
   for cw in CONTROLS:
    r=z.read(44);assert len(r)==44 and word(r,0)==i and word(r,4)==cw and list(struct.unpack_from('<6I',r,8))==b['fields']
    if cw==0x27f:chosen=r
   assert chosen.hex()==b['projection44_hex']==idx[i]['native_bounds_projection44_hex']
  assert not z.read(1)
 positives={};count=total=0
 with(a.inputs/'input.bin').open('rb')as inp,(a.inputs/'expected1032.bin').open('rb')as expected,FINAL.open('rb')as final:
  for bi,(b,old)in enumerate(zip(idx,oldidx)):
   offset=inp.tell();h=inp.read(32);check_header(h);n,q=word(h,4),word(h,8);assert word(h,16)==bi and q==122 and word(h,12)==0x27f and n==b['finalized_blocks']and offset==b['offset']
   assert all(b[k]==old[k]for k in('body','native_case','package','key','source_sha256','body_records_sha256'))
   records=inp.read(n*324);final.seek(b['native_case']*82652);native=final.read(82652);assert hashlib.sha256(native).hexdigest()==b['native_projection_sha256']and records==native[32:32+n*324]and hashlib.sha256(records).hexdigest()==b['body_records_sha256']
   for i in range(n):
    block=records[i*324:(i+1)*324];assert word(block,OFF[0x1f8])==i and word(block,OFF[0x21c])==b['source_palette_ids'][i]
    for field in(0x10c,0x110,0x114,0x118,0x11c,0x120,0x124,0x138,0x13c,0x140):
     value=word(block,OFF[field]);assert value&0x7fffffff<=(0x40000000 if 0x118<=field<=0x124 else 0x49800000)
     if field>=0x138:assert 0<value<=0x49800000
   for si,s in enumerate(selectors):
    for mirror in(0,1):
     raw=inp.read(32);check_query(raw);qi=si*2+mirror;assert raw==struct.pack('<8I',*s,mirror,255,0,qi);ref=expected.read(1032);assert len(ref)==1032 and word(ref,0)==bi and word(ref,4)==qi and word(ref,8)<=n
     slots=struct.unpack_from('<255I',ref,12);c=word(ref,8);assert all(x<n for x in slots[:c])and slots[c:]==(0xffffffff,)*(255-c)
     if s[0]in(0x4c,0x1040)and c and s[0]not in positives:positives[s[0]]=dict(body=bi,query=qi,input_offset=offset,record_count=n,expected_offset=count*1032,expected_count=c)
     count+=1
   assert inp.tell()==offset+b['group_bytes'];total+=n
  assert not inp.read(1)and not expected.read(1)
 assert count==161284 and total==46256 and set(positives)=={0x4c,0x1040}
 return m,dict(bodies=1322,blocks=46256,queries=count,source_aliases=1644,source_alias_exclusions=0,body_upstream_exclusions=43,original_producer_before_selector=True,full_helper='009B6920',partial_caller='009C48DB..009C4992 (not full9C48C0)',positive_Z_smokes=positives,actual_finalized_and_original_Z_native_fields_joined=True,source_actor_association=False,peer=peer(a.peer_review,a.run),runner_peer=peer(a.runner_review,a.run,True),native_executed=False)

def inspect(requests,replies,expected=None,projection=None):
 stats=dict(groups=0,queries=0,nonempty=0,selected=0);out=None if projection is None else projection.open('xb')
 try:
  with requests.open('rb')as inp,replies.open('rb')as native:
   gold=None if expected is None else expected.open('rb')
   try:
    while h:=inp.read(32):
     check_header(h);n,q=word(h,4),word(h,8);assert len(inp.read(n*324))==n*324
     for _ in range(q):
      raw=inp.read(32);check_query(raw);r=native.read(1040);assert len(r)==1040 and word(r,0)==word(h,16)and word(r,4)==word(raw,28)and word(r,8)<=min(n,word(raw,20))
      c=word(r,8);slots=struct.unpack_from('<255I',r,12);assert all(x<n for x in slots[:c])and slots[c:]==(0xffffffff,)*(255-c)
      assert struct.unpack_from('<H',r,1032)[0]==word(h,12)and struct.unpack_from('<H',r,1034)[0]&0x3800==0 and word(r,1036)&~0x3f==0x1f80
      if gold is not None:assert r[:1032]==gold.read(1032),(stats['queries'],'native/reference mismatch')
      if out is not None:out.write(r[:1032])
      stats['queries']+=1;stats['nonempty']+=bool(c);stats['selected']+=c
     stats['groups']+=1
    assert not native.read(1)
    if gold is not None:assert not gold.read(1)
   finally:
    if gold is not None:gold.close()
 finally:
  if out is not None:out.close()
 return stats

def execute(a,m,closure):
 out=a.out/'native-v1';out.mkdir(exist_ok=False);paths=[a.source_exe,Path(__file__),a.peer_review,a.runner_review,a.out/'preflight.json',FINAL]+[ROOT/x for x in FILES]
 for v in(m,json.loads((OLD/'manifest.json').read_text()),json.loads(a.peer_review.read_text()),json.loads(a.runner_review.read_text())):paths +=[resolve(x)for x in v['pins']]
 for folder in(a.host_dir,a.inputs,PREP):paths +=[x for x in folder.iterdir()if x.is_file()]
 paths=list(dict.fromkeys(x.resolve()for x in paths));host=(a.host_dir/'selector-Z-carrier-x86.exe').resolve()
 with pinned(paths,out):
  protocol=out/'protocol';protocol.mkdir()
  with(a.inputs/'input.bin').open('rb')as f,(a.inputs/'expected1032.bin').open('rb')as g:
   h=f.read(32);first=h+f.read(word(h,4)*324+word(h,8)*32);gold=g.read(word(h,8)*1032)
  p=protocol/'body0-input.bin';p.write_bytes(first);g=protocol/'body0-expected1032.bin';g.write_bytes(gold);status,r=saved_file_run(host,p,protocol,'body0');assert status['exit']==0 and status['stderr_bytes']==0,status;observed=inspect(p,r,g)
  if a.reference_only:save(out/'reference-only-report.json',dict(native_executed=True,retries=0,observed=observed,host_sha256=CARRIER,scope='Genuine actualbody0 full122querygroup; original producer runs before queries.'));return
  smoke={}
  for flag,row in closure['positive_Z_smokes'].items():
   flag=int(flag);name=f'positive_Z_{flag:04X}'
   with(a.inputs/'input.bin').open('rb')as f,(a.inputs/'expected1032.bin').open('rb')as e:
    f.seek(row['input_offset']);h=bytearray(f.read(32));n=word(h,4);records=f.read(n*324);f.seek(row['input_offset']+32+n*324+row['query']*32);query=f.read(32);struct.pack_into('<I',h,8,1);request=bytes(h)+records+query;e.seek(row['expected_offset']);wanted=e.read(1032)
   p=protocol/(name+'-input.bin');p.write_bytes(request);g=protocol/(name+'-expected1032.bin');g.write_bytes(wanted);status,r=saved_file_run(host,p,protocol,name);assert status['exit']==0 and status['stderr_bytes']==0;inspect(p,r,g);smoke[flag]=(request,wanted,r.read_bytes())
  positive,positivegold,fullpositive=smoke[0x4c];n=word(positive,4);query_at=32+n*324;cases=[('empty',b'',0,b''),('repeat_group',positive*2,0,fullpositive*2)]
  for size in(1,31,32,query_at,len(positive)-1):cases +=[(f'partial{size}',positive[:size],71,b''),(f'complete_then_partial{size}',positive+positive[:size],71,fullpositive)]
  def mutate(name,at,value):
   raw=bytearray(positive);struct.pack_into('<I',raw,at,value);cases.append((name,bytes(raw),71,b''))
  for name,at,value in[('magic',0,0),('count0',4,0),('count256',4,256),('querycount0',8,0),('querycount257',8,257),('cw_reservedPC',12,0x17f),('cw_unmasked',12,0x27e),('cw_highbits',12,0x100027f)]+[(f'reserved{x}',x,1)for x in(20,24,28)]:mutate(name,at,value)
  for name,at,value in[('Z40alone',0,0x40),('Z80',0,0x80),('ZC0',0,0xc0),('ZplusX',0,0x44),('ZplusY',0,0x5c),('classmask',0,0x14c),('hierarchy_other',0,0x2040),('alternate',8,1),('reserved_selector',12,1),('mirror2',16,2),('capacity256',20,256),('expansion',24,1)]:mutate(name,query_at+at,value)
  for name,field,value in[('center_nan',0x10c,0x7fc00000),('quat_inf',0x124,0x7f800000),('quat_bound',0x118,0x40000001),('extent_zero',0x138,0),('extent_negative',0x140,0xbf800000),('ordinal',0x1f8,255),('rootparent',0x1fc,0)]:mutate(name,32+OFF[field],value)
  mutate('capability_mask',32,word(positive,32)^1);results=[]
  for name,raw,exitcode,wanted in cases:
   p=protocol/(name+'-input.bin');p.write_bytes(raw);status,r=saved_file_run(host,p,protocol,name);assert status['exit']==exitcode and status['stderr_bytes']==0,(name,status);assert r.read_bytes()==wanted,name;results.append(dict(case=name,exit=exitcode,exact_reply=True,input_sha256=digest(p),output_sha256=digest(r)))
  for flag,(positive,gold,unused)in smoke.items():
   for capacity in(0,1,255):
    name=f'Z{flag:04X}_capacity{capacity}';raw=bytearray(positive);struct.pack_into('<I',raw,32+word(raw,4)*324+20,capacity);count=min(capacity,word(gold,8));wanted=bytearray(gold);struct.pack_into('<I',wanted,8,count);wanted[12+count*4:]=b'\xff'*(1020-count*4);p=protocol/(name+'-input.bin');p.write_bytes(raw);g=protocol/(name+'-expected1032.bin');g.write_bytes(wanted);status,r=saved_file_run(host,p,protocol,name);assert status['exit']==0 and status['stderr_bytes']==0;inspect(p,r,g);results.append(dict(case=name,exit=0,projection_exact=True))
   for cw in CONTROLS:
    name=f'Z{flag:04X}_policy{cw:04X}';raw=bytearray(positive);struct.pack_into('<I',raw,12,cw);p=protocol/(name+'-input.bin');p.write_bytes(raw);g=protocol/(name+'-expected1032.bin');g.write_bytes(gold);status,r=saved_file_run(host,p,protocol,name);assert status['exit']==0 and status['stderr_bytes']==0;inspect(p,r,g);results.append(dict(case=name,exit=0,projection_exact=True))
  save(protocol/'report.json',dict(checks=len(results),results=results,single_attempt=True,retries=0,body0_smoke_queries=122,positive_Z_smokes=closure['positive_Z_smokes']))
  status,r=saved_file_run(host,a.inputs/'input.bin',out,'corpus',timeout=300);assert status['exit']==0 and status['stderr_bytes']==0,status;observed=inspect(a.inputs/'input.bin',r,a.inputs/'expected1032.bin',out/'native-projection1032.bin');assert observed==dict(groups=1322,queries=161284,nonempty=61624,selected=71395)
  save(out/'report.json',dict(schema='all1644-original-Z-producer-selector-native-run-v1',native_executed=True,retries=0,observed=observed,protocol_checks=len(results),host_sha256=CARRIER,input_sha256=INPUT,output_sha256=digest(r),projection_sha256=digest(out/'native-projection1032.bin'),expected_sha256=EXPECTED,scope='Actual finalized source fields ×all1644aliases/61unique policies. Original fullbboxhelper+partialcaller9C48DB..4992 executes before fullselector. Partialcaller is not full9C48C0. No wholebinding/AEownership/pruning/liveactor association.'))
def main():
 p=argparse.ArgumentParser(description=__doc__)
 for name in('host-dir','inputs','out'):p.add_argument('--'+name,type=Path,required=True)
 p.add_argument('--peer-review',type=Path);p.add_argument('--runner-review',type=Path);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--run',action='store_true');p.add_argument('--reference-only',action='store_true');a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());m,c=review(a);a.out.mkdir(parents=True,exist_ok=False);save(a.out/'preflight.json',dict(schema='all1644-original-Z-producer-selector-preflight-v1',runner_sha256=digest(__file__),closure=c,native_executed=False,execution_requested=a.run))
 if a.run:execute(a,m,c)
 else:print(json.dumps(dict(preflight=True,queries=161284,positive_Z_smokes=c['positive_Z_smokes'],peer=c['peer'],native_executed=False)))
if __name__=='__main__':main()

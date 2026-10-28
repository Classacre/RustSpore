"""Default-preflight, explicit-run full original9A1E00 prepared-field native gate.
Hidden file-backed single attempts only; original bytes never executed by default.
"""
import argparse,json,struct,hashlib,io
from pathlib import Path
from types import SimpleNamespace
import anim_ungrouped_binding_builder_audit_v1 as audit
from cell_runtime_data_native_run import digest,pinned,saved_file_run,word
ROOT=Path(__file__).resolve().parent.parent
CARRIER='9f4f12ab4ddfc2c223d147416a51b2a8ec3fc16572f1ee293f5821ea7fddd30b'
OWNED='e562a314bbbd89c0d2c3c534ca9f4e8356830358aee52093da426c30800d6eb2'
MAP='66f87260178abceedb7422b3f6ffa8e282bb1ad52dc6406cbaab226158ddccbd'
INPUT='d9d009821bd97d0217af832b3c90d6d5012a53d4ac80451f31418a9b61696319'
EXPECTED='1e48f611b34426cb5ae4c351c6354741bec317720fc541f95abeab4a24f5c49a'
PREP=ROOT/'target/animation/ungrouped-binding-builder-native-v1-frozen'
COHORT=ROOT/'target/animation/nonroot-selector-Z-inputs-v1-complete'
NATIVE=ROOT/'target/animation/nonroot-selector-Z-native-run-v1/native-v1'
FINAL=ROOT/'target/animation/body-finalizer-fields-native-v3-run/native-projection.bin'
CAPACITY=7140;MODEL=114284;REPLY=114292
FIELDS=[0x104,0x108,0x1f8,0x1fc,0x200,0x21c,0x208,0x10c,0x110,0x114,0x118,0x11c,0x120,0x124,0x128,0x12c,0x130,0x134,0x138,0x13c,0x140,0x144,0x148,0x14c,0x344,0x348,0x34c,0x350,0x354,0x358,0x35c,0x360,0x364,0x368,0x36c,0x378,0x37c,0x204,0x20c]
OFF={x:168+i*4 for i,x in enumerate(FIELDS)}
FILES=['tools/anim_ungrouped_binding_builder_native_probe_v1.py','tools/anim_ungrouped_binding_builder_audit_v1.py','tools/anim_ungrouped_binding_builder_prepare_v1.py','crates/spore-core/src/anim_ungrouped_binding_builder.rs','crates/spore-core/examples/anim_ungrouped_binding_builder_verify.rs.pending','tools/anim_nonroot_selector_z_native_probe_v1.py','tools/anim_nonroot_selector_native_probe_v2.py','tools/anim_selector_z_bounds_native_probe_v1.py','tools/anim_root_pose_sparse_carrier_v3.py','tools/anim_root_pose_sparse_carrier_audit_v2.py','tools/cell_runtime_data_native_run.py','tools/cell_schema_probe.py','tools/native_crt_acos_probe.py','crates/spore-core/src/anim_nonroot_selector.rs','crates/spore-core/src/anim_nonroot_selector_z.rs','crates/spore-core/src/anim_selector_z_bounds.rs','crates/spore-core/src/anim_body_finalizer_fields.rs','crates/spore-core/src/native_sse.rs','crates/spore-core/src/native_x87.rs']
def resolve(p):return Path(p)if Path(p).is_absolute()else ROOT/p
def save(p,v):
 with p.open('x')as f:json.dump(v,f,indent=2);f.write('\n')
def peer(path,required,runner=False):
 if path is None:
  assert not required,'Actual independent linked peer and subsequent runner peer are required'
  return dict(admitted=False,reason='runner_peer_pending'if runner else'actual_linked_peer_pending')
 r=json.loads(path.read_text());assert r['no_blocking_static_issue_found']is True and r['native_executed']is False
 for name,value in r['pins'].items():assert digest(resolve(name))==value,name
 assert CARRIER in r['pins'].values()and OWNED in r['pins'].values()and MAP in r['pins'].values()
 if runner:assert digest(__file__)in r['pins'].values()
 return dict(admitted=True,path=str(path.resolve()),sha256=digest(path))
def dependencies(start):
 """Verify recursively declared pins/files; retain exact dependency paths."""
 found={};seen=set();queue=[Path(x)for x in start]
 while queue:
  p=queue.pop().resolve()
  if p in seen:continue
  seen.add(p);found[str(p)]=digest(p)
  if p.suffix!='.json':continue
  value=json.loads(p.read_text())
  if not isinstance(value,dict):continue
  for key,relative in(('pins',False),('files',True)):
   if key not in value:continue
   assert isinstance(value[key],dict)
   for name,wanted in value[key].items():
    q=(p.parent/name if relative else resolve(name)).resolve();assert digest(q)==wanted,(str(p),name);found[str(q)]=wanted
    if q.suffix=='.json':queue.append(q)
 return found

def read_case(f):
 offset=f.tell();h=f.read(32)
 if not h:return None
 assert len(h)==32 and word(h,0)==0x31444c42
 n,c=word(h,4),word(h,8);assert 1<=n<=255 and 1<=c<=14 and word(h,12)==0x27f and word(h,28)==0
 records=f.read(n*324);assert len(records)==n*324;channels=[];dc=0
 for _ in range(c):
  at=f.tell()-offset;raw=f.read(80);assert len(raw)==80;words=list(struct.unpack_from('<19I',raw));d=word(raw,76);assert 1<=d<=24
  descriptors=[]
  for i in range(d):
   data=f.read(32);assert len(data)==32;descriptors.append(list(struct.unpack('<8I',data)))
  channels.append(dict(words=words,descriptors=descriptors,wire_offset=at));dc+=d
 assert dc==word(h,24)
 return dict(header=h,records=records,channels=channels,n=n,c=c,body=word(h,16),clip=word(h,20),offset=offset,bytes=f.tell()-offset)
def check_projection(case,p):
 assert len(p)==MODEL and word(p,0)==case['body']and word(p,4)==case['clip']and word(p,8)==1
 count=word(p,12);assert count<=min(CAPACITY,2*case['c']*case['n']);fields=list(struct.unpack_from('<7I',p,16));live=components=zero=fallback=spatial=0;seen=set();nonzero_seen=False
 for i in range(count):
  passmask,high,mask,packed=struct.unpack_from('<4I',p,44+i*16);index=packed&255;ci=(packed>>8)&255;secondary=(packed>>16)&255;flags=packed>>24
  assert passmask in(1,2)and high==0 and mask&0xff000000==0 and index<case['n']and ci<case['c']and secondary==255 and flags&~0x2a==0
  identity=(passmask,index,ci);assert identity not in seen;seen.add(identity)
  if index:nonzero_seen=True
  else:assert not nonzero_seen;zero+=1
  ch=case['channels'][ci];descriptors=ch['descriptors'];assert mask>>len(descriptors)==0
  type1=False
  for di,d in enumerate(descriptors):
   if mask&(1<<di):assert d[0]&0x10;type1|=d[0]&15==1
  assert bool(flags&0x20)==type1
  assert bool(flags&8)==(mask==0)and(mask!=0 or ch['words'][9]&8)
  fallback+=mask==0;ac=ch['words'][9]&0x32
  if ac==0:mirror=passmask==2
  else:
   assert ac==0x10;cx=struct.unpack_from('<f',case['records'],index*324+OFF[0x10c])[0];mirror=cx<=0;spatial+=1
  assert bool(flags&2)==mirror
  live|=1<<ci
  if mask:components|=1<<ci
 assert fields==[live&0xffffffff,live>>32,components&0xffffffff,components>>32,0,2 if count else 1,1]
 assert p[44+count*16:]==b'\xff'*((CAPACITY-count)*16)
 return dict(records=count,selected_zero=zero,empty_fallbacks=fallback,spatial_mirror_records=spatial)

def review(a):
 m=json.loads((a.inputs/'manifest.json').read_text());assert(m['cases'],m['clips'],m['actual_body_ids'],m['source_enabled_channel_instances'],m['source_nonroot_channel_instances'],m['source_descriptors'])==(1085,217,[603,1182,472,833,895],1861,1644,7956)
 assert m['reference_record_count']==27142 and m['reference_empty_fallback_count']==160 and m['record_capacity']==CAPACITY and m['projection_bytes']==MODEL and m['reply_bytes']==REPLY and m['retained_body_upstream_exclusions']==43
 assert digest(a.inputs/'input.bin')==INPUT and digest(a.inputs/'expected114284.bin')==EXPECTED
 assert digest(a.host_dir/'builder-carrier-x86.exe')==CARRIER and digest(a.host_dir/'host-x86.exe')==OWNED and digest(a.host_dir/'host-x86.map')==MAP
 assert(a.host_dir/'host-x86.asm').read_bytes()==(PREP/'host-x86.asm').read_bytes()
 dep=dependencies([a.inputs/'manifest.json',PREP/'preparation-review.json',COHORT/'manifest.json',NATIVE/'rust-comparison-v1.json'])
 proof=json.loads((PREP/'source-closure.json').read_text())
 for fn in proof['functions']:
  for obj in(fn,fn.get('origin',{})):
   for name,hashkey in(('query','query_sha256'),('parent_exact_callee_metadata','query_sha256'),('accepted_allowlist','sha256'),('accepted_case_closed_source','sha256')):
    if name in obj:q=resolve(obj[name]);assert digest(q)==obj[hashkey];dep[str(q.resolve())]=obj[hashkey]
 actual=audit.audit(SimpleNamespace(carrier=a.host_dir/'builder-carrier-x86.exe',owned_exe=a.host_dir/'host-x86.exe',owned_map=a.host_dir/'host-x86.map',prep=a.host_dir,source_exe=a.source_exe))
 assert actual==json.loads((a.host_dir/'root-linked-audit-v1.json').read_text())and actual['source_bytes']==13380 and actual['native_pages']==16 and actual['producer_passes_before_whole_entry']==2 and actual['whole_entry_poison_passes']==2
 idx=json.loads((a.inputs/'cases.json').read_text());clips=json.loads((a.inputs/'clips.json').read_text());bodies={b['body']:b for b in json.loads((a.inputs/'bodies.json').read_text())};cohortidx=json.loads((COHORT/'body-index.json').read_text());sourceclips=json.loads((ROOT/'target/animation/nonroot-builder-construction-plan-v1/actual217-channel-order.json').read_text());assert len(clips)==217 and len(idx)==1085 and set(bodies)=={603,1182,472,833,895}
 for i,c in enumerate(clips):assert c['key']==sourceclips[i]['key']and c['sha256']==sourceclips[i]['source_sha256']and c['channels']==sourceclips[i]['channels']and c['raw_mask154']==c['raw_expected158']==0 and c['raw_version']==25
 bodybytes={}
 with FINAL.open('rb')as f:
  for bi,b in bodies.items():
   original=cohortidx[bi];assert b==original;f.seek(b['native_case']*82652);raw=f.read(82652);assert hashlib.sha256(raw).hexdigest()==b['native_projection_sha256'];bodybytes[bi]=raw[32:32+b['finalized_blocks']*324]
 stats=dict(cases=0,records=0,empty_fallbacks=0,spatial_mirror_records=0);smokes={}
 with(a.inputs/'input.bin').open('rb')as f,(a.inputs/'expected114284.bin').open('rb')as gold:
  for i,r in enumerate(idx):
   case=read_case(f);assert case is not None and case['offset']==r['input_offset']and case['bytes']==r['input_bytes']and case['body']==r['body']and case['clip']==r['clip']and case['n']==r['blocks']and case['c']==r['channels']
   assert case['body']==m['actual_body_ids'][i//217]and case['clip']==i%217 and case['records']==bodybytes[case['body']]
   source=clips[case['clip']];assert r['source_clip_key']==source['key']and r['source_clip_sha256']==source['sha256']and r['source_body_key']==bodies[case['body']]['key']and r['source_body_sha256']==bodies[case['body']]['source_sha256']
   for ci,(ch,sc)in enumerate(zip(case['channels'],source['channels'])):
    assert sc['channel']==ci and sc['enabled']and ch['words']==sc['raw_metadata_words_88_d0']and ch['descriptors']==sc['descriptors']and sc['groupAF']==0 and sc['secondary']==[2,0,0,0]and sc['metadataAC']&4==0
   p=gold.read(MODEL);assert hashlib.sha256(p).hexdigest()==r['projection_sha256'];observed=check_projection(case,p);assert observed['records']==r['record_count']and observed['empty_fallbacks']==r['empty_fallback_count']and observed['selected_zero']==r['selected_zero_count']and list(struct.unpack_from('<7I',p,16))==r['fields']
   for k in('records','empty_fallbacks','spatial_mirror_records'):stats[k]+=observed[k]
   stats['cases']+=1
   if observed['empty_fallbacks']and 'empty_fallback'not in smokes:smokes['empty_fallback']=i
   if observed['spatial_mirror_records']and 'spatial_mirror'not in smokes:smokes['spatial_mirror']=i
  assert read_case(f)is None and not gold.read(1)
 assert stats['cases']==1085 and stats['records']==27142 and stats['empty_fallbacks']==160 and set(smokes)=={'empty_fallback','spatial_mirror'}
 dep[str((ROOT/'target/animation/nonroot-builder-construction-plan-v1/actual217-channel-order.json').resolve())]=digest(ROOT/'target/animation/nonroot-builder-construction-plan-v1/actual217-channel-order.json')
 return m,dict(stats=stats,actual_source_join=True,all1861_source_channels_and1644_nonroot_instances_retained=True,source_7956_descriptors_retained=True,source_body_upstream_exclusions=43,original_actor_association=False,full_entry='009A1E00',partial_preproducer='009C48DB..009C4992 (notfull9C48C0)',source_allocation_claim=False,conditional_source_closure='AF0/AC&0x4=0/secondary2/rawmask0/7140capacity; originalreset/helper/probe/constructor/fallback executed.',genuine_smoke_cases=smokes,dependency_pins=dep,peer=peer(a.peer_review,a.run),runner_peer=peer(a.runner_review,a.run,True),native_executed=False)

def inspect(requests,replies,expected=None,projection=None):
 stats=dict(cases=0,records=0,empty_fallbacks=0,spatial_mirror_records=0);out=None if projection is None else projection.open('xb')
 try:
  with requests.open('rb')as inp,replies.open('rb')as native:
   gold=None if expected is None else expected.open('rb')
   try:
    while(case:=read_case(inp))is not None:
     r=native.read(REPLY);assert len(r)==REPLY
     observed=check_projection(case,r[:MODEL]);assert struct.unpack_from('<H',r,MODEL)[0]==0x27f and struct.unpack_from('<H',r,MODEL+2)[0]&0x3800==0 and word(r,MODEL+4)&~0x3f==0x1f80
     if gold is not None:assert r[:MODEL]==gold.read(MODEL),(stats['cases'],'full original/reference mismatch')
     if out is not None:out.write(r[:MODEL])
     stats['cases']+=1
     for k in('records','empty_fallbacks','spatial_mirror_records'):stats[k]+=observed[k]
    assert not native.read(1)
    if gold is not None:assert not gold.read(1)
   finally:
    if gold is not None:gold.close()
 finally:
  if out is not None:out.close()
 return stats

def execute(a,m,closure):
 out=a.out/'native-v1';out.mkdir(exist_ok=False);paths=[a.source_exe,Path(__file__),a.peer_review,a.runner_review,a.out/'preflight.json',FINAL]+[ROOT/x for x in FILES]+[Path(x)for x in closure['dependency_pins']]
 for p in(a.peer_review,a.runner_review):paths +=[resolve(x)for x in json.loads(p.read_text())['pins']]
 for folder in(a.host_dir,a.inputs,PREP):paths +=[x for x in folder.iterdir()if x.is_file()]
 paths=list(dict.fromkeys(x.resolve()for x in paths));host=(a.host_dir/'builder-carrier-x86.exe').resolve();idx=json.loads((a.inputs/'cases.json').read_text())
 with pinned(paths,out):
  protocol=out/'protocol';protocol.mkdir()
  def source_case(ci):
   row=idx[ci]
   with(a.inputs/'input.bin').open('rb')as f,(a.inputs/'expected114284.bin').open('rb')as g:f.seek(row['input_offset']);raw=f.read(row['input_bytes']);g.seek(ci*MODEL);gold=g.read(MODEL)
   return raw,gold
  first,firstgold=source_case(0);p=protocol/'genuine-first-input.bin';p.write_bytes(first);g=protocol/'genuine-first-expected114284.bin';g.write_bytes(firstgold);status,r=saved_file_run(host,p,protocol,'genuine-first');assert status['exit']==0 and status['stderr_bytes']==0,status;observed=inspect(p,r,g);fullpositive=r.read_bytes()
  if a.reference_only:save(out/'reference-only-report.json',dict(native_executed=True,retries=0,single_attempt=True,host_sha256=CARRIER,observed=observed,scope='Genuine firstcase body603/clip0; full9A1E00 and original preproducer. Numeric Rust comparison remains separate.'));return
  smoke_results=[]
  for name,ci in closure['genuine_smoke_cases'].items():
   raw,gold=source_case(ci);p=protocol/(name+'-input.bin');p.write_bytes(raw);g=protocol/(name+'-expected114284.bin');g.write_bytes(gold);status,r=saved_file_run(host,p,protocol,name);assert status['exit']==0 and status['stderr_bytes']==0;obs=inspect(p,r,g);smoke_results.append(dict(case=name,cohort_case=ci,observed=obs,exact_projection=True))
  parsed=read_case(io.BytesIO(first));n=parsed['n'];ch0=parsed['channels'][0]['wire_offset'];desc0=ch0+80;cases=[('empty',b'',0,b''),('repeat_case',first*2,0,fullpositive*2)]
  for size in sorted({1,31,32,32+n*324,ch0+79,desc0,len(first)-1}):cases +=[(f'partial{size}',first[:size],71,b''),(f'complete_then_partial{size}',first+first[:size],71,fullpositive)]
  def mutate(name,offset,value):
   raw=bytearray(first);struct.pack_into('<I',raw,offset,value);cases.append((name,bytes(raw),71,b''))
  for name,at,value in[('magic',0,0),('body0',4,0),('body256',4,256),('channels0',8,0),('channels15',8,15),('cw24',12,0x7f),('cw64',12,0x37f),('cw_reservedPC',12,0x17f),('cw_unmasked',12,0x27e),('reserved',28,1),('desctotal0',24,0),('desctotal337',24,337)]:mutate(name,at,value)
  for name,field,value in[('center_nan',0x10c,0x7fc00000),('center_inf',0x114,0x7f800000),('center_bound',0x110,0x49800001),('quat_nan',0x118,0x7fc00000),('quat_bound',0x124,0x40000001),('extent0',0x138,0),('extent_negzero',0x13c,0x80000000),('extent_negative',0x140,0xbf800000),('ordinal',0x1f8,255),('rootparent',0x1fc,0)]:mutate(name,32+OFF[field],value)
  mutate('selfparent',32+324+OFF[0x1fc],1);mutate('parent_outside',32+324+OFF[0x1fc],n);mutate('presence_mask',32,word(first,32)^1);mutate('nonzero_mask',36,word(first,36)^1)
  for name,at,value in[('channel_disabled',ch0+9*4,0),('required_mask',ch0+9*4,5),('groupAF1',ch0+9*4,0x01000001),('AC_unsupported',ch0+9*4,0x21),('secondary3',ch0+5*4,3),('secondary_override',ch0+5*4,0x100003),('secondarycap',ch0+6*4,1),('secondaryalternate',ch0+7*4,1),('secondaryreserved',ch0+8*4,1),('primaryforce',ch0+4,0x100001),('primarycap',ch0+8,1),('descriptor_count0',ch0+76,0),('descriptor_count25',ch0+76,25),('slot9',desc0+16,9)]:mutate(name,at,value)
  raw=bytearray(first);struct.pack_into('<II',raw,desc0,0x13,0x70e47545);cases.append(('pseudo_descriptor',bytes(raw),71,b''))
  results=[]
  for name,raw,exitcode,wanted in cases:
   p=protocol/(name+'-input.bin');p.write_bytes(raw);status,r=saved_file_run(host,p,protocol,name);assert status['exit']==exitcode and status['stderr_bytes']==0,(name,status);assert r.read_bytes()==wanted,name;results.append(dict(case=name,exit=exitcode,exact_raw_reply=True,input_sha256=digest(p),output_sha256=digest(r)))
  save(protocol/'report.json',dict(checks=len(results),results=results,genuine_first_case=0,genuine_additional_smokes=smoke_results,single_attempt=True,retries=0,projection_bytes=MODEL,reply_bytes=REPLY))
  status,r=saved_file_run(host,a.inputs/'input.bin',out,'corpus',timeout=300);assert status['exit']==0 and status['stderr_bytes']==0,status;observed=inspect(a.inputs/'input.bin',r,a.inputs/'expected114284.bin',out/'native-projection114284.bin');assert observed==closure['stats']
  save(out/'report.json',dict(schema='full9A1E00-prepared-source-entry-native-run-v1',native_executed=True,retries=0,observed=observed,protocol_checks=len(results),genuine_additional_smokes=smoke_results,host_sha256=CARRIER,input_sha256=INPUT,output_sha256=digest(r),projection_sha256=digest(out/'native-projection114284.bin'),expected_sha256=EXPECTED,source_bytes=13380,source_clips=217,source_enabled_channels=1861,source_nonroot_instances=1644,source_descriptors=7956,actual_body_ids=m['actual_body_ids'],upstream_body_exclusions=43,scope='Fulloriginal9A1E00 underpreparedexactAF0/secondary2/required0/rawmask0/7140capacitydomain, originalreset/helper/probe/constructor/fallback. Separatepreproducerpartial9C48DB..4992; nofull9C48C0, sourceallocation, liveactor/fullobject/outerbuilder claim.'))
def main():
 p=argparse.ArgumentParser(description=__doc__)
 for name in('host-dir','inputs','out'):p.add_argument('--'+name,type=Path,required=True)
 p.add_argument('--peer-review',type=Path);p.add_argument('--runner-review',type=Path);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--run',action='store_true');p.add_argument('--reference-only',action='store_true');a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());m,c=review(a);a.out.mkdir(parents=True,exist_ok=False);save(a.out/'preflight.json',dict(schema='full9A1E00-prepared-source-entry-preflight-v1',runner_sha256=digest(__file__),closure=c,native_executed=False,execution_requested=a.run))
 if a.run:execute(a,m,c)
 else:print(json.dumps(dict(preflight=True,stats=c['stats'],genuine_smokes=c['genuine_smoke_cases'],peer=c['peer'],native_executed=False)))
if __name__=='__main__':main()


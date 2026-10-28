"""Default-preflight, single-attempt actual-body full9B2150 selector runner.
Never runs original code without --run and actual independent linked peer.
"""
import argparse,json,struct,hashlib,math
from pathlib import Path
from types import SimpleNamespace
import anim_nonroot_selector_audit_v2 as audit
from cell_runtime_data_native_run import digest,pinned,saved_file_run,word
ROOT=Path(__file__).resolve().parent.parent
CARRIER='9c7b990c155a12c1ce1db9226c17953226b9d132072f26b906d203a2e18402a3'
OWNED='708df109bb9a5e3ef47a5955652f11ef6841b3d93dd269406afbac5fbccb304b'
MAP='43ff9288ee505d6752d1ae388378de2a05879f14387b451b4b676a5506f9eb0e'
INPUT='5a108caa2e031f168c5306a95d0bf8429963396325175d2a1d1969a56272d9bb'
EXPECTED='b6036545c2061cfcf793ab4431778aa945ad61439ccd03236474306349acd85f'
BODY='e09a5b85cc07bfbe51cd97179b82f2b4630ac746f7af3190a4857d3e7b62367b'
PREP=ROOT/'target/animation/nonroot-selector-native-v2'
FINAL=ROOT/'target/animation/body-finalizer-fields-native-v3-run'
FINPUT=ROOT/'target/animation/body-finalizer-fields-inputs-v2'
PROTOCOL=ROOT/'target/animation/nonroot-selector-protocol-v2'
CONTROLS=[0x7f|p|r for p in(0,0x200,0x300)for r in(0,0x400,0x800,0xc00)]
FIELDS=[0x104,0x108,0x1f8,0x1fc,0x200,0x21c,0x208,0x10c,0x110,0x114,0x118,0x11c,0x120,0x124,0x128,0x12c,0x130,0x134,0x138,0x13c,0x140,0x144,0x148,0x14c,0x344,0x348,0x34c,0x350,0x354,0x358,0x35c,0x360,0x364,0x368,0x36c,0x378,0x37c,0x204,0x20c]
OFF={x:168+i*4 for i,x in enumerate(FIELDS)}
def resolve(p):return Path(p)if Path(p).is_absolute()else ROOT/p
def save(p,x):
 with p.open('x')as f:json.dump(x,f,indent=2);f.write('\n')
def peer(path,required):
 if path is None:
  assert not required,'Actual independent linked peer report required'
  return dict(admitted=False,reason='actual_linked_peer_pending')
 r=json.loads(path.read_text());assert r['no_blocking_static_issue_found']is True and r['native_executed']is False
 for name,value in r['pins'].items():assert digest(resolve(name))==value,name
 assert CARRIER in r['pins'].values()and OWNED in r['pins'].values()
 return dict(admitted=True,path=str(path.resolve()),sha256=digest(path))
def runner_peer(path,required):
 if path is None:
  assert not required,'Actual independent runner supplemental pin review required'
  return dict(admitted=False,reason='runner_peer_pending')
 r=json.loads(path.read_text());assert r['no_blocking_static_issue_found']is True and r['native_executed']is False
 for name,value in r['pins'].items():assert digest(resolve(name))==value,name
 assert digest(__file__)in r['pins'].values()and CARRIER in r['pins'].values()
 return dict(admitted=True,path=str(path.resolve()),sha256=digest(path))

def check_header(h):
 assert len(h)==32 and word(h,0)==0x314c4553 and 1<=word(h,4)<=255 and 1<=word(h,8)<=256 and word(h,12)in CONTROLS and h[20:]==bytes(12)
def check_query(q):
 assert len(q)==32 and word(q,0)&~0x6f03c==0 and word(q,8)==word(q,12)==word(q,24)==0 and word(q,16)<=1 and word(q,20)<=255
 flags=word(q,0);assert flags&0x2000c in(0,4,8,12,0x20000,0x20004,0x20008)and flags&0x40030 in(0,0x10,0x20,0x30,0x40000,0x40010,0x40020)and flags&0xe000 in(0,0x2000,0x4000,0x6000,0x8000)
 assert flags&0x801c00==0 or flags==0x1000

def review(a):
 m=json.loads((a.inputs/'manifest.json').read_text());assert m['bodies']==1322 and m['finalized_blocks']==46256 and m['source_aliases']==1644 and m['admitted_source_aliases']==1641 and m['retained_Z_exclusions']==3 and m['queries']==155996 and m['unique_selector_words']==59
 for name,value in m['pins'].items():assert digest(resolve(name))==value,name
 for name,value in m['files'].items():assert digest(a.inputs/name)==value,name
 assert digest(a.inputs/'input.bin')==INPUT and digest(a.inputs/'expected1032.bin')==EXPECTED and digest(FINAL/'native-projection.bin')==BODY
 assert digest(a.host_dir/'selector-carrier-x86.exe')==CARRIER and digest(a.host_dir/'host-x86.exe')==OWNED and digest(a.host_dir/'host-x86.map')==MAP
 assert(a.host_dir/'host-x86.asm').read_bytes()==(PREP/'host-x86.asm').read_bytes()
 prep=json.loads((PREP/'manifest.json').read_text());assert digest(ROOT/'tools/anim_nonroot_selector_native_probe_v2.py')==prep['tool_sha256'];assert digest(ROOT/'tools/anim_nonroot_selector_audit_v2.py')=='954d9a67735ca65e7424c7e835f4f0bbe39644f173115c8acd81034e98e294b0'
 actual=audit.audit(SimpleNamespace(carrier=a.host_dir/'selector-carrier-x86.exe',owned_exe=a.host_dir/'host-x86.exe',owned_map=a.host_dir/'host-x86.map',prep=a.host_dir,source_exe=a.source_exe))
 assert actual==json.loads((a.host_dir/'root-linked-audit-v2.json').read_text())and actual['source_bytes']==4693 and actual['native_pages']==9
 bodies=json.loads((a.inputs/'body-index.json').read_text());selectors=json.loads((a.inputs/'selector-table.json').read_text());aliases=json.loads((a.inputs/'selector-aliases.json').read_text());assert len(bodies)==1322 and len(selectors)==59 and len(aliases)==1644
 assert len({(x['package'],x['key'])for x in bodies})==1322
 assert sum(x['admitted']for x in aliases)==1641 and all(x['primary'][0]&0x610300==0 and x['primary'][2:]==[0,0]for x in aliases)
 assert {tuple(x['primary'])for x in aliases if x['admitted']}=={tuple(x)for x in selectors}
 excluded=[x for x in aliases if not x['admitted']];assert len(excluded)==3 and all(x['primary'][0]&0x800c0 and x['exclusion']=='OriginalDefinitionZBounds360_36CRequired'for x in excluded)
 cases=[line.split('\t')for line in(FINPUT/'cases.tsv').read_text().splitlines()[1:]];resources=json.loads((ROOT/'target/animation/body-capabilities-inputs-v1/source-resources.json').read_text());resources={x['key']:x for x in resources};assert len(resources)==1365
 count=total=generated=0
 with(a.inputs/'input.bin').open('rb')as f,(a.inputs/'expected1032.bin').open('rb')as expected,(FINAL/'native-projection.bin').open('rb')as native:
  for bi,b in enumerate(bodies):
   h=f.read(32);check_header(h);n,q=word(h,4),word(h,8);assert word(h,16)==bi and q==118 and word(h,12)==0x027f and n==b['finalized_blocks']
   c=cases[b['native_case']];assert c[1]=='source_relabel'and c[-1]==b['key'];resource=resources[b['key']];assert resource['package']==b['package']and resource['source_sha256']==b['source_sha256']and resource['source_rows']==b['source_rows']
   native.seek(b['native_case']*82652);original=native.read(82652);assert word(original,8)==n and hashlib.sha256(original).hexdigest()==b['native_projection_sha256']
   records=f.read(n*324);assert records==original[32:32+n*324]and hashlib.sha256(records).hexdigest()==b['body_records_sha256']
   for i in range(n):
    block=records[i*324:(i+1)*324];assert word(block,OFF[0x1f8])==i and word(block,OFF[0x21c])==b['source_palette_ids'][i]
    generated+=word(block,OFF[0x21c])==0xffffffff
    for field in(0x10c,0x110,0x114,0x138,0x13c,0x140):
     bits=word(block,OFF[field]);assert bits&0x7fffffff<=0x49800000
     if field>=0x138:assert bits&0x80000000==0 and bits&0x7fffffff!=0
    at=i;seen=set()
    while True:
     parent=word(records,at*324+OFF[0x1fc])
     if parent==0xffffffff:assert at==0;break
     assert parent<n and at not in seen;seen.add(at);at=parent
   assert word(records,4)&8
   for si,s in enumerate(selectors):
    for mirrored in(0,1):
     raw=f.read(32);check_query(raw);qi=si*2+mirrored;assert raw==struct.pack('<8I',*s,mirrored,255,0,qi)
     ref=expected.read(1032);assert len(ref)==1032 and word(ref,0)==bi and word(ref,4)==qi and word(ref,8)<=n
     nselected=word(ref,8);slots=struct.unpack_from('<255I',ref,12);assert all(x<n for x in slots[:nselected])and slots[nselected:]==(0xffffffff,)*(255-nselected)
     count+=1
   total+=n
  assert not f.read(1)and not expected.read(1)
 assert count==155996 and total==46256
 protocols=json.loads((PROTOCOL/'report.json').read_text());assert len(protocols['negative_cases'])==17
 for x in protocols['negative_cases']:assert digest(PROTOCOL/(x['name']+'.bin'))==x['input_sha256']
 return m,dict(bodies=1322,finalized_blocks=46256,source_aliases=1644,admitted_aliases=1641,retained_Z_exclusions=3,body_upstream_exclusions=43,queries=count,source_functions_bytes=4687,source_memcpy_thunk_bytes=6,source_total_bytes=4693,actual_genuine_finalizer_body_bytes_exact=True,all_source_query_and_mirror_words_exact=True,full_source_palette_and_parent_preserved=True,generated_blocks=generated,actual_linked_audit_reproduced=True,peer=peer(a.peer_review,a.run),runner_peer=runner_peer(a.runner_review,a.run),native_executed=False)

def inspect(requests,replies,expected=None,projection=None):
 stats=dict(groups=0,queries=0,nonempty=0,selected=0);out=None if projection is None else projection.open('xb')
 try:
  with requests.open('rb')as inp,replies.open('rb')as native:
   gold=None if expected is None else expected.open('rb')
   try:
    while h:=inp.read(32):
     check_header(h);n,q=word(h,4),word(h,8);body=inp.read(n*324);assert len(body)==n*324
     for _ in range(q):
      raw=inp.read(32);check_query(raw);r=native.read(1040);assert len(r)==1040
      assert word(r,0)==word(h,16)and word(r,4)==word(raw,28)and word(r,8)<=min(n,word(raw,20))
      count=word(r,8);slots=struct.unpack_from('<255I',r,12);assert all(x<n for x in slots[:count])and slots[count:]==(0xffffffff,)*(255-count)
      assert struct.unpack_from('<H',r,1032)[0]==word(h,12)and struct.unpack_from('<H',r,1034)[0]&0x3800==0 and word(r,1036)&~0x3f==0x1f80
      if gold is not None:assert r[:1032]==gold.read(1032),(stats['queries'],'original/reference mismatch')
      if out is not None:out.write(r[:1032])
      stats['queries']+=1;stats['nonempty']+=bool(count);stats['selected']+=count
     stats['groups']+=1
    assert not native.read(1)
    if gold is not None:assert not gold.read(1)
   finally:
    if gold is not None:gold.close()
 finally:
  if out is not None:out.close()
 return stats

def execute(a,manifest,closure):
 out=a.out/'native-v1';out.mkdir(exist_ok=False);peerdata=json.loads(a.peer_review.read_text())
 paths=[a.source_exe,Path(__file__),a.peer_review,a.runner_review,a.out/'preflight.json',a.inputs/'manifest.json',ROOT/'tools/cell_runtime_data_native_run.py',ROOT/'tools/cell_schema_probe.py',ROOT/'tools/native_crt_acos_probe.py',ROOT/'tools/anim_nonroot_selector_audit_v2.py',ROOT/'tools/anim_nonroot_selector_native_probe_v2.py',ROOT/'tools/anim_root_pose_sparse_carrier_v3.py',ROOT/'tools/anim_root_pose_sparse_carrier_audit_v2.py',ROOT/'crates/spore-core/src/anim_nonroot_selector.rs',ROOT/'crates/spore-core/examples/anim_nonroot_selector_verify.rs.pending']
 paths += [resolve(x)for x in json.loads(a.runner_review.read_text())['pins']]+[resolve(x)for x in manifest['pins']]+[resolve(x)for x in peerdata['pins']]+[x for x in a.host_dir.iterdir()if x.is_file()]+[x for x in a.inputs.iterdir()if x.is_file()]+[x for x in PROTOCOL.iterdir()if x.is_file()]
 paths=list(dict.fromkeys(x.resolve()for x in paths));host=(a.host_dir/'selector-carrier-x86.exe').resolve()
 with pinned(paths,out):
  protocol=out/'protocol';protocol.mkdir()
  # Root-requested genuine body#0 full118-query group smoke, even if all return0.
  with(a.inputs/'input.bin').open('rb')as f,(a.inputs/'expected1032.bin').open('rb')as g:
   h=f.read(32);first=h+f.read(word(h,4)*324+word(h,8)*32);gold=g.read(word(h,8)*1032)
  request=protocol/'body0-input.bin';request.write_bytes(first);expected=protocol/'body0-expected1032.bin';expected.write_bytes(gold);status,reply=saved_file_run(host,request,protocol,'body0')
  assert status['exit']==0 and status['stderr_bytes']==0,status
  observed=inspect(request,reply,expected)
  if a.reference_only:save(out/'reference-only-report.json',dict(native_executed=True,single_attempt=True,retries=0,scope='Genuine sourcebody0 fullgroup; no fakefallback even ifzeroMatches',host_sha256=CARRIER,observed=observed,output_sha256=digest(reply)));return
  positive=(PROTOCOL/'reference-input.bin').read_bytes();positivegold=(PROTOCOL/'reference-expected1032.bin').read_bytes();source=protocol/'positive-copy-input.bin';source.write_bytes(positive);expected=protocol/'positive-copy-expected1032.bin';expected.write_bytes(positivegold);status,output=saved_file_run(host,source,protocol,'positive-copy');assert status['exit']==0 and status['stderr_bytes']==0;inspect(source,output,expected);fullpositive=output.read_bytes()
  n=word(positive,4);query_start=32+n*324;cases=[('empty',b'',0,b'')]
  for size in(1,31,32,query_start,len(positive)-1):cases += [(f'partial{size}',positive[:size],71,b''),(f'complete_then_partial{size}',positive+positive[:size],71,fullpositive)]
  for row in json.loads((PROTOCOL/'report.json').read_text())['negative_cases']:cases.append((row['name'],(PROTOCOL/(row['name']+'.bin')).read_bytes(),71,b''))
  cases.append(('repeat_group',positive+positive,0,fullpositive*2));results=[]
  for name,raw,exitcode,wanted in cases:
   p=protocol/(name+'-input.bin');p.write_bytes(raw);status,reply=saved_file_run(host,p,protocol,name);assert status['exit']==exitcode and status['stderr_bytes']==0,(name,status);assert reply.read_bytes()==wanted,name;results.append(dict(case=name,exit=exitcode,exact_reply=True,input_sha256=digest(p),output_sha256=digest(reply)))
  for capacity in(0,1,255):
   name=f'capacity{capacity}';p=protocol/(name+'-input.bin');raw=(PROTOCOL/(name+'-input.bin')).read_bytes();p.write_bytes(raw);count=min(capacity,word(positivegold,8));wanted=bytearray(positivegold);struct.pack_into('<I',wanted,8,count);wanted[12+count*4:]=bytes([255])*(1020-count*4);g=protocol/(name+'-expected1032.bin');g.write_bytes(wanted);status,reply=saved_file_run(host,p,protocol,name);assert status['exit']==0 and status['stderr_bytes']==0;inspect(p,reply,g);results.append(dict(case=name,exit=0,projection_exact=True,input_sha256=digest(p),output_sha256=digest(reply)))
  for cw in CONTROLS:
   name=f'policy_{cw:04X}';raw=bytearray(positive);struct.pack_into('<I',raw,12,cw);p=protocol/(name+'-input.bin');p.write_bytes(raw);g=protocol/(name+'-expected1032.bin');g.write_bytes(positivegold);status,reply=saved_file_run(host,p,protocol,name);assert status['exit']==0 and status['stderr_bytes']==0;inspect(p,reply,g);results.append(dict(case=name,exit=0,projection_exact=True,input_sha256=digest(p),output_sha256=digest(reply)))
  save(protocol/'report.json',dict(checks=len(results),results=results,single_attempt=True,retries=0,body0_smoke_queries=118,positive_copy_smoke=True))
  status,reply=saved_file_run(host,a.inputs/'input.bin',out,'corpus',timeout=300);assert status['exit']==0 and status['stderr_bytes']==0,status
  observed=inspect(a.inputs/'input.bin',reply,a.inputs/'expected1032.bin',out/'native-projection1032.bin');assert observed['groups']==1322 and observed['queries']==155996
  save(out/'report.json',dict(schema='actual-body-full-nonroot-selector-native-run-v1',native_executed=True,retries=0,bodies=1322,finalized_blocks=46256,queries=155996,source_aliases=1644,admitted_source_aliases=1641,retained_Z_exclusions=3,body_upstream_exclusions=43,protocol_checks=len(results),observed=observed,host_sha256=CARRIER,input_sha256=INPUT,output_sha256=digest(reply),projection_sha256=digest(out/'native-projection1032.bin'),expected_projection_sha256=EXPECTED,scope='Actual source-derived finalized body fields and two source mirror passes across59policies. No syntheticzerochild;3Zaliases remainexcluded,unknownDefinition360/36Cdeadcalculations/FPstickyflags unestablished; no constructor/AEpruning/liveactor/wholeclip admission.'))

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--inputs',type=Path,required=True);p.add_argument('--peer-review',type=Path);p.add_argument('--runner-review',type=Path);p.add_argument('--out',type=Path,required=True);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--run',action='store_true');p.add_argument('--reference-only',action='store_true');a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());manifest,closure=review(a);a.out.mkdir(parents=True,exist_ok=False);save(a.out/'preflight.json',dict(schema='actual-body-nonroot-selector-preflight-v1',runner_sha256=digest(__file__),closure=closure,native_executed=False,execution_requested=a.run))
 if a.run:execute(a,manifest,closure)
 else:print(json.dumps(dict(preflight=True,bodies=1322,queries=155996,peer=closure['peer'],native_executed=False)))
if __name__=='__main__':main()

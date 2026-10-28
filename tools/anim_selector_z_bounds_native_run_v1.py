"""Default-preflight, explicit-run single-attempt full bbox + partial publisher gate.
Runs only actual sparse carrier after linked and runner peers; never retries.
"""
import argparse,json,struct,hashlib
from pathlib import Path
from types import SimpleNamespace
import anim_selector_z_bounds_audit_v1 as audit
from cell_runtime_data_native_run import digest,pinned,saved_file_run,word
ROOT=Path(__file__).resolve().parent.parent
CARRIER='96381ef7cb644eb1d43b8f2004df77d32be2ed11e3b688d33b7ca1c00e1994ae'
OWNED='5b5aee79c25069c70e6a5f6e1dd2780423539be15e7b5cf0c53002524db7296d'
MAP='8a25a73a7b009737b09552e8a5888f158be275aa0eec6fdbe3938ecc2c080834'
INPUT='998fd91c6fc4d83f50d29fd41a0b341423015082c524f23e57ee85cb5b5f60cc'
PREP=ROOT/'target/animation/selector-Z-producer-native-v1'
UP=ROOT/'target/animation/nonroot-selector-inputs-v3'
FINAL=ROOT/'target/animation/body-finalizer-fields-native-v3-run/native-projection.bin'
CONTROLS=[0x7f|p|r for p in(0,0x200,0x300)for r in(0,0x400,0x800,0xc00)]
FILES=['tools/anim_selector_z_bounds_native_probe_v1.py','tools/anim_selector_z_bounds_prepare_v1.py','tools/anim_selector_z_bounds_audit_v1.py','tools/cell_runtime_data_native_run.py','tools/cell_schema_probe.py','tools/native_crt_acos_probe.py','tools/anim_root_pose_sparse_carrier_v3.py','tools/anim_root_pose_sparse_carrier_audit_v2.py','crates/spore-core/src/anim_selector_z_bounds.rs','crates/spore-core/src/native_sse.rs','crates/spore-core/src/native_x87.rs','crates/spore-core/src/anim_body_finalizer_fields.rs','crates/spore-core/examples/anim_selector_z_bounds_verify.rs.pending']
def resolve(p):return Path(p)if Path(p).is_absolute()else ROOT/p
def save(p,v):
 with p.open('x')as f:json.dump(v,f,indent=2);f.write('\n')
def peer(path,required,runner=False):
 if path is None:
  assert not required,'Independent actual linked/runner peer required before native execution'
  return dict(admitted=False,reason='runner_peer_pending'if runner else'actual_linked_peer_pending')
 r=json.loads(path.read_text());assert r['no_blocking_static_issue_found']is True and r['native_executed']is False
 for name,value in r['pins'].items():assert digest(resolve(name))==value,name
 assert CARRIER in r['pins'].values()and OWNED in r['pins'].values()
 if runner:assert digest(__file__)in r['pins'].values()
 return dict(admitted=True,path=str(path.resolve()),sha256=digest(path))
def header(h):
 assert len(h)==32 and word(h,0)==0x315a4242 and 1<=word(h,4)<=255 and 1<=word(h,8)<=12 and h[16:]==bytes(16)
def blocks(raw,n):
 assert len(raw)==n*324
 for k in range(n):
  f=struct.unpack_from('<39I',raw,k*324+168)
  for j in range(7,10):assert f[j]&0x7fffffff<=0x49800000
  for j in range(10,14):assert f[j]&0x7fffffff<=0x40000000
  for j in range(18,21):assert 0<f[j]<=0x49800000

def review(a):
 m=json.loads((a.inputs/'manifest.json').read_text());assert(m['bodies'],m['blocks'],m['records'])==(1322,46256,15864)and m['policies']==CONTROLS
 for name,value in m['pins'].items():assert digest(resolve(name))==value,name
 assert digest(a.inputs/'input.bin')==INPUT
 upstream=json.loads((UP/'manifest.json').read_text())
 for name,value in upstream['pins'].items():assert digest(resolve(name))==value,name
 for name,value in upstream['files'].items():assert digest(UP/name)==value,name
 assert digest(FINAL)=='e09a5b85cc07bfbe51cd97179b82f2b4630ac746f7af3190a4857d3e7b62367b'
 frozen=json.loads((PREP/'preparation-review.json').read_text())
 for name,value in frozen['pins'].items():assert digest(resolve(name))==value,name
 assert digest(a.host_dir/'Z-carrier-x86.exe')==CARRIER and digest(a.host_dir/'host-x86.exe')==OWNED and digest(a.host_dir/'host-x86.map')==MAP
 assert(a.host_dir/'host-x86.asm').read_bytes()==(PREP/'host-x86.asm').read_bytes()
 actual=audit.audit(SimpleNamespace(carrier=a.host_dir/'Z-carrier-x86.exe',owned_exe=a.host_dir/'host-x86.exe',owned_map=a.host_dir/'host-x86.map',prep=a.host_dir,source_exe=a.source_exe))
 assert actual==json.loads((a.host_dir/'root-linked-audit-v1.json').read_text())and actual['source_bytes']==3396 and actual['native_pages']==7
 idx=json.loads((a.inputs/'body-index.json').read_text());oldidx=json.loads((UP/'body-index.json').read_text());assert len(idx)==len(oldidx)==1322
 total=0
 with(a.inputs/'input.bin').open('rb')as inp,FINAL.open('rb')as original:
  for i,(b,old)in enumerate(zip(idx,oldidx)):
   assert inp.tell()==b['offset'];h=inp.read(32);header(h);n=word(h,4);assert word(h,8)==12 and word(h,12)==i==b['body']and n==b['blocks']==old['finalized_blocks']
   assert all(b[k]==old[k]for k in('native_case','package','key','source_sha256'))
   raw=inp.read(n*324);blocks(raw,n);assert hashlib.sha256(raw).hexdigest()==b['records_sha256']==old['body_records_sha256']
   original.seek(b['native_case']*82652);native=original.read(82652);assert digest_bytes(native)==old['native_projection_sha256']and raw==native[32:32+n*324]
   assert struct.unpack('<12I',inp.read(48))==tuple(CONTROLS)and inp.tell()==b['offset']+b['bytes'];total+=n
  assert not inp.read(1)
 assert total==46256
 return m,dict(bodies=1322,blocks=46256,rows=15864,actual_finalized_body_bytes_exact=True,source_and_actor_provenance='source fields joined; live actor association unsupported',source_bytes=3396,full_helper_bytes=3213,partial_caller_bytes=183,real_stack_argument_creation=True,retained_x87_captured=True,actual_linked_audit_reproduced=True,peer=peer(a.peer_review,a.run),runner_peer=peer(a.runner_review,a.run,True),native_executed=False)
def digest_bytes(b):return hashlib.sha256(b).hexdigest()
def inspect(request,reply,projection=None):
 result=dict(groups=0,rows=0);out=None if projection is None else projection.open('xb')
 try:
  with request.open('rb')as inp,reply.open('rb')as observed:
   while h:=inp.read(32):
    header(h);n,q=word(h,4),word(h,8);blocks(inp.read(n*324),n)
    for _ in range(q):
     cw=word(inp.read(4),0);assert cw in CONTROLS;r=observed.read(52);assert len(r)==52 and word(r,0)==word(h,12)and word(r,4)==cw
     assert r[42:44]==bytes(2)and struct.unpack_from('<H',r,44)[0]==cw and struct.unpack_from('<H',r,46)[0]&0x3800==0 and word(r,48)&~0x3f==0x1f80
     fields=struct.unpack_from('<6I',r,8);assert all(x&0x7f800000!=0x7f800000 for x in fields)
     assert all(x&0x80000000==0 for x in fields[3:])and int.from_bytes(r[40:42],'little')&0x7fff!=0x7fff
     if out is not None:out.write(r[:44])
     result['rows']+=1
    result['groups']+=1
   assert not observed.read(1)
 finally:
  if out is not None:out.close()
 return result

def execute(a,m,closure):
 out=a.out/'native-v1';out.mkdir(exist_ok=False)
 paths=[a.source_exe,Path(__file__),a.peer_review,a.runner_review,a.out/'preflight.json',FINAL]+[ROOT/x for x in FILES]
 for manifest in(m,json.loads((UP/'manifest.json').read_text()),json.loads(a.peer_review.read_text()),json.loads(a.runner_review.read_text())):paths +=[resolve(x)for x in manifest['pins']]
 for folder in(a.host_dir,a.inputs,PREP):paths +=[x for x in folder.iterdir()if x.is_file()]
 paths=list(dict.fromkeys(x.resolve()for x in paths));host=(a.host_dir/'Z-carrier-x86.exe').resolve()
 with pinned(paths,out):
  protocol=out/'protocol';protocol.mkdir()
  raw=(a.inputs/'input.bin').read_bytes();h=bytearray(raw[:32]);n=word(h,4);struct.pack_into('<I',h,8,1);positive=bytes(h)+raw[32:32+n*324]+struct.pack('<I',CONTROLS[0]);assert word(h,12)==0
  request=protocol/'body0-input.bin';request.write_bytes(positive);status,reply=saved_file_run(host,request,protocol,'body0');assert status['exit']==0 and status['stderr_bytes']==0,status;observed=inspect(request,reply);assert observed==dict(groups=1,rows=1);reference=reply.read_bytes()
  if a.reference_only:save(out/'reference-only-report.json',dict(native_executed=True,single_attempt=True,retries=0,host_sha256=CARRIER,observed=observed,output_sha256=digest(reply),scope='Actual body0, first explicit policy, one source helper/caller row; numeric comparison remains separate.'));return
  cases=[('empty',b'',0,b''),('repeat_group',positive*2,0,reference*2)]
  for size in(1,31,32,32+n*324,len(positive)-1):cases +=[(f'partial{size}',positive[:size],71,b''),(f'complete_then_partial{size}',positive+positive[:size],71,reference)]
  def mutate(name,offset,value):
   z=bytearray(positive);struct.pack_into('<I',z,offset,value);cases.append((name,bytes(z),71,b''))
  for name,offset,value in[('magic',0,0),('count0',4,0),('count256',4,256),('policy0',8,0),('policy13',8,13)]+[(f'reserved{i}',i,1)for i in(16,20,24,28)]+[('cw_reservedPC',len(positive)-4,0x17f),('cw_unmasked',len(positive)-4,0x27e),('cw_highbits',len(positive)-4,0x100027f),('cw_infinitybit',len(positive)-4,0x127f)]:mutate(name,offset,value)
  def field(j):return 32+168+j*4
  for name,j,value in[('center_nan',7,0x7fc00000),('center_inf',8,0xff800000),('center_bound',9,0x49800001),('quat_nan',10,0x7fc00000),('quat_bound',13,0x40000001),('extent_zero',18,0),('extent_negativezero',19,0x80000000),('extent_negative',20,0xbf800000),('extent_inf',18,0x7f800000),('extent_bound',19,0x49800001)]:mutate(name,field(j),value)
  results=[]
  for name,data,code,wanted in cases:
   p=protocol/(name+'-input.bin');p.write_bytes(data);status,r=saved_file_run(host,p,protocol,name);assert status['exit']==code and status['stderr_bytes']==0,(name,status);assert r.read_bytes()==wanted,name;results.append(dict(case=name,exit=code,exact_reply=True,input_sha256=digest(p),output_sha256=digest(r)))
  for cw in CONTROLS:
   name=f'policy_{cw:04X}';z=bytearray(positive);struct.pack_into('<I',z,len(z)-4,cw);p=protocol/(name+'-input.bin');p.write_bytes(z);status,r=saved_file_run(host,p,protocol,name);assert status['exit']==0 and status['stderr_bytes']==0;inspect(p,r);assert r.read_bytes()[8:32]==reference[8:32];results.append(dict(case=name,exit=0,published_six_fields_equal=True,input_sha256=digest(p),output_sha256=digest(r)))
  save(protocol/'report.json',dict(checks=len(results),results=results,single_attempt=True,retries=0,body0_smoke_rows=1))
  status,r=saved_file_run(host,a.inputs/'input.bin',out,'corpus',timeout=300);assert status['exit']==0 and status['stderr_bytes']==0,status
  observed=inspect(a.inputs/'input.bin',r,out/'native-projection44.bin');assert observed==dict(groups=1322,rows=15864)
  save(out/'report.json',dict(schema='actual-finalized-body-Z-publisher-native-run-v1',native_executed=True,retries=0,protocol_checks=len(results),observed=observed,host_sha256=CARRIER,input_sha256=INPUT,output_sha256=digest(r),projection_sha256=digest(out/'native-projection44.bin'),scope='Full original9B6920 plus bounded original field-publisher continuation; actual finalized fields and supplied12CWs. No original9C3150/fullwrapper/diagonal/constraints/actor/wholebuilder claim. Numeric Rust equality remains separate.'))
def main():
 p=argparse.ArgumentParser(description=__doc__)
 for name in('host-dir','inputs','out'):p.add_argument('--'+name,type=Path,required=True)
 p.add_argument('--peer-review',type=Path);p.add_argument('--runner-review',type=Path);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--run',action='store_true');p.add_argument('--reference-only',action='store_true');a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());m,c=review(a);a.out.mkdir(parents=True,exist_ok=False);save(a.out/'preflight.json',dict(schema='actual-Z-publisher-preflight-v1',runner_sha256=digest(__file__),closure=c,native_executed=False,execution_requested=a.run))
 if a.run:execute(a,m,c)
 else:print(json.dumps(dict(preflight=True,bodies=1322,rows=15864,peer=c['peer'],native_executed=False)))
if __name__=='__main__':main()

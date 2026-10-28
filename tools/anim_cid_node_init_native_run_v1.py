"""Frozen CID node initializer runner. Default preflight; explicit --run once."""
import argparse,hashlib,json,struct
from pathlib import Path
import anim_cid_node_init_audit_v1 as audit
from cell_runtime_data_native_run import pinned,saved_file_run
ROOT=audit.prep.ROOT
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,v):
 with p.open('x')as f:json.dump(v,f,indent=2);f.write('\n')
def resolve(p):return Path(p)if Path(p).is_absolute()else ROOT/p
def inspect(input_bytes,native,index):
 at=0;projection=bytearray()
 for row in index:
  n=row['count'];size=32+n*700;reply=native[at:at+size+8];assert len(reply)==size+8
  header=input_bytes[row['input_offset']:row['input_offset']+32];assert reply[:32]==header
  assert hashlib.sha256(reply[:size]).hexdigest()==row['projection_sha256'],row['body']
  cw,sw,csr=struct.unpack_from('<HHI',reply,size);assert cw==0x27f and sw&0x3800==0 and csr&~0x3f==0x1f80
  projection+=reply[:size];at+=size+8
 assert at==len(native);return bytes(projection)
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--inputs',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--linked-peer',type=Path,required=True);p.add_argument('--plan',type=Path);p.add_argument('--runner-peer',type=Path);p.add_argument('--run',action='store_true');a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
 assert a.inputs.resolve()==(ROOT/'target/animation/cid-node-init-actual-body-inputs-v1').resolve()
 host=a.host_dir;files=[Path(__file__),Path(audit.__file__),Path(audit.prep.__file__),ROOT/'tools/cell_runtime_data_native_run.py',a.linked_peer];packet=ROOT/'target/animation/cid-node-init-native-packet-v1/pins.json';q=json.loads(packet.read_text());files.append(packet)
 for name,value in q['pins'].items():f=resolve(name);assert sha(f)==value,name;files.append(f)
 linked=audit.audit(host,host/'host-x86.exe',host/'host-x86.map',host/'carrier-x86.exe');assert linked['actual_linked'];peer=json.loads(a.linked_peer.read_text());assert peer['static_pass'] and not peer['native_executed'] and peer['carrier_sha256']==sha(host/'carrier-x86.exe')
 files.extend(f for f in host.iterdir()if f.is_file());files.extend(f for f in a.inputs.iterdir()if f.is_file());raw=(a.inputs/'input.bin').read_bytes();index=json.loads((a.inputs/'index.json').read_text());m=json.loads((a.inputs/'manifest.json').read_text());assert len(index)==1322 and sum(row['count']for row in index)==46256 and sha(a.inputs/'input.bin')==m['input_sha256']
 for name,value in m['protocols'].items():assert sha(a.inputs/(name+'.bin'))==value
 if a.plan:
  plan=json.loads(a.plan.read_text());assert plan['runner_sha256']==sha(__file__)
  for name,value in plan['pins'].items():f=resolve(name);assert sha(f)==value,name;files.append(f)
  files.append(a.plan)
 if a.runner_peer:
  assert a.plan;rp=json.loads(a.runner_peer.read_text());assert rp['no_blocking_static_issue_found']and not rp['native_executed']
  for f in [Path(__file__),a.plan,host/'carrier-x86.exe',host/'host-x86.exe',host/'host-x86.map']:assert sha(f)in rp['pins'].values()
  for name,value in rp['pins'].items():f=resolve(name);assert sha(f)==value,name;files.append(f)
  files.append(a.runner_peer)
 if a.run:assert a.plan and a.runner_peer,'Frozen plan and independent exact-runner peer required'
 a.out.mkdir(parents=True,exist_ok=False)
 if not a.plan:
  save(a.out/'plan.json',dict(schema='CID-node-init-runner-plan-v1',native_executed=False,runner_sha256=sha(__file__),pins={str(f.resolve()):sha(f)for f in dict.fromkeys(files)}))
 save(a.out/'preflight.json',dict(native_executed=False,execution_requested=a.run,cases=1322,nodes=46256,retries=0,actual_linked_audit=True,linked_peer=str(a.linked_peer)))
 if not a.run:print('Preflight PASS; no native execution');return
 try:
  with pinned(list(dict.fromkeys(f.resolve()for f in files)),a.out):
   proto=a.out/'protocol';proto.mkdir();seen={}
   def execute(label,data,code,expected=None):
    req=proto/(label+'-input.bin');req.write_bytes(data);pins=proto/(label+'-pins');pins.mkdir()
    with pinned([req],pins):status,out=saved_file_run(host/'carrier-x86.exe',req,proto,label)
    assert status['exit']==code and not status['timeout']and status['stderr_bytes']==0,(label,status)
    got=out.read_bytes()
    if expected is not None:assert got==expected,label
    return got
   smokes=list(dict.fromkeys([0,max(range(1322),key=lambda i:index[i]['count']),max(range(1322),key=lambda i:index[i]['nonzero_child_nodes'])]))
   for bi in smokes:
    row=index[bi];req=raw[row['input_offset']:row['input_offset']+row['input_bytes']];got=execute(f'genuine_body{bi}',req,0);local=dict(row,input_offset=0);inspect(req,got,[local]);seen[bi]=got
   first=index[0];req=raw[:first['input_bytes']];execute('empty',b'',0,b'');execute('repeat',req*2,0,seen[0]*2);execute('partial_first',req[:16],71,b'');execute('complete_then_partial',req+req[:16],71,seen[0])
   for name in m['protocols']:execute(name,(a.inputs/(name+'.bin')).read_bytes(),71,b'')
   status,out=saved_file_run(host/'carrier-x86.exe',a.inputs/'input.bin',a.out,'corpus',timeout=240);assert status['exit']==0 and not status['timeout']and status['stderr_bytes']==0,status
   native=out.read_bytes();projection=inspect(raw,native,index);assert projection==(a.inputs/'expected.bin').read_bytes();offset=0
   for row in index:
    if row['body']in seen:assert native[offset:offset+row['reply_bytes']]==seen[row['body']]
    offset+=row['reply_bytes']
   pp=a.out/'native-projection.bin';pp.write_bytes(projection);save(a.out/'report.json',dict(native_executed=True,cases=1322,nodes=46256,nonzero_child_nodes=m['nonzero_child_nodes'],upstream_exclusions=43,smokes=len(smokes),protocols=len(m['protocols']),framing=4,retries=0,full700_byte_rows_match_reference=True,raw_sha256=sha(out),projection_sha256=sha(pp),rust_accepted=False,full_9C5130=False,actor_association=False))
 except BaseException as e:save(a.out/'failure.json',dict(error=repr(e),retained=True,retries=0));raise
if __name__=='__main__':main()

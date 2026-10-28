"""Fresh sparse host: default static preflight; --run single attempts, no retry."""
import argparse,json
from pathlib import Path
from types import SimpleNamespace
import cell_runtime_fresh_aid_sparse_carrier_audit_v3 as audit
from cell_runtime_data_native_run import digest,pinned,saved_file_run
from cell_runtime_small_hosts_native_run_v1 import save,inspect
ROOT=Path(__file__).resolve().parent.parent
CARRIER='3e2c593483f63ccc5610b195066f3836153e84b974e712096df93c650f2fd2f9'
OWNED='aa50102a039e3415b57952f9a591cf6cc83287842539f508088ef19ebae062c1'
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--peer-review',type=Path);p.add_argument('--run',action='store_true');a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
 h=ROOT/'target/source-cell/native-runtime-fresh-aid-sparse-carrier-root-v1';inp=ROOT/'target/source-cell/runtime-fresh-aid-inputs-v1';oldpeer=ROOT/'target/source-cell/runtime-fresh-aid-independent-font-review-v1.json';source=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe')
 assert digest(h/'carrier-x86.exe')==CARRIER and digest(h/'host-x86.exe')==OWNED
 actual=audit.audit(SimpleNamespace(carrier=h/'carrier-x86.exe',owned_exe=h/'host-x86.exe',owned_map=h/'host-x86.map',source_exe=source));assert actual==json.loads((h/'independent-linked-audit-v3.json').read_text())
 paths=[Path(__file__),Path(audit.__file__),ROOT/'tools/anim_root_pose_sparse_carrier_audit_v2.py',ROOT/'tools/cell_runtime_fresh_aid_sparse_carrier_v2.py',ROOT/'tools/cell_runtime_fresh_aid_sparse_carrier_v1.py',ROOT/'tools/cell_runtime_small_hosts_native_run_v1.py',ROOT/'tools/cell_runtime_data_native_run.py',source,oldpeer,*[x for x in h.iterdir() if x.is_file()]]
 old=json.loads(oldpeer.read_text());assert old['no_blocking_static_issue_found'] and not old['native_executed']
 for name,value in old['pins'].items():
  q=ROOT/name;assert digest(q)==value,name;paths.append(q)
 report=json.loads((inp/'report.json').read_text());assert report['cases']==288 and digest(inp/'input.bin')==report['input_sha256'];raw=(inp/'input.bin').read_bytes();assert len(raw)==288*64
 for name,value in report['protocols'].items():
  q=inp/(name+'.bin');assert digest(q)==value;paths.append(q)
 if a.peer_review:
  peer=json.loads(a.peer_review.read_text());assert peer['no_blocking_static_issue_found'] is True and peer['native_executed'] is False and CARRIER in peer['pins'].values() and OWNED in peer['pins'].values()
  for name,value in peer['pins'].items():
   q=ROOT/name;assert digest(q)==value,name;paths.append(q)
  paths.append(a.peer_review)
 else:assert not a.run,'Actual sparse linked independent peer required before native execution'
 a.out.mkdir(parents=True,exist_ok=False);save(a.out/'preflight.json',dict(schema='fresh-aid-sparse-preflight-v1',native_executed=False,execution_requested=a.run,actual_linked_audit=True,independent_peer=bool(a.peer_review),cases=288,carrier_sha256=CARRIER,owned_sha256=OWNED,old_failure='Original allocation host v1 exit10 preinput mapping failure retained; no unchanged retry.'))
 if not a.run:print('Sparse preflight PASS; native not executed');return
 try:
  with pinned(list(dict.fromkeys(x.resolve() for x in paths)),a.out):
   exe=h/'carrier-x86.exe';protocol=a.out/'protocol';protocol.mkdir();ref=protocol/'reference-input.bin';ref.write_bytes(raw[:64]);status,out=saved_file_run(exe,ref,protocol,'reference');assert status['exit']==0 and status['stderr_bytes']==0,status;inspect(ref.read_bytes(),out.read_bytes(),'fresh-aid');expected=out.read_bytes()
   checks=[('empty',b'',0,b''),('repeat_two',raw[:64]*2,0,expected*2),('complete_then_partial',raw[:64]+raw[:32],7,expected)]+[(n,(inp/(n+'.bin')).read_bytes(),7,b'') for n in report['protocols']]
   for name,q,code,w in checks:
    request=protocol/(name+'-input.bin');request.write_bytes(q);status,out=saved_file_run(exe,request,protocol,name);assert status['exit']==code and status['stderr_bytes']==0,(name,status);assert out.read_bytes()==w,name
   status,out=saved_file_run(exe,inp/'input.bin',a.out,'corpus');assert status['exit']==0 and status['stderr_bytes']==0,status;projection=inspect(raw,out.read_bytes(),'fresh-aid');pp=a.out/'native-projection56.bin';pp.write_bytes(projection);save(a.out/'report.json',dict(native_executed=True,cases=288,protocols=len(checks),retries=0,carrier_sha256=CARRIER,raw_output_sha256=digest(out),projection_sha256=digest(pp),rust_accepted=False,numeric_expected_from_rust=False))
 except BaseException as e:save(a.out/'failure.json',dict(error=repr(e),retained=True,retries=0));raise
if __name__=='__main__':main()

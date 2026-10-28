"""Prepare endpoint successor with full source slot table, without build or execution."""
import hashlib,json,shutil
from pathlib import Path
import cell_runtime_fresh_sampled_channel_endpoint_runner_v3 as run

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 old=run.prep.OUT;out=run.P
 assert not out.exists()
 previous=json.loads((old/'preparation.json').read_text())
 for path,digest in previous['pins'].items():assert sha(path)==digest,path
 out.mkdir(parents=True);shutil.copytree(old/'inputs',out/'inputs');shutil.copyfile(old/'endpoint_verify_v1.rs.pending',out/'endpoint_verify_v1.rs.pending');(out/'host').mkdir()
 run.setup();images=run.base.arithmetic.Images(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll'));run.base.audit_source(images)
 host=out/'host/host-x86.asm';host.write_text(run.v3.host_source(images))
 pins={}
 for path in previous['pins']:
  p=Path(path)
  try:p=out/p.relative_to(old)
  except ValueError:pass
  pins[str(p.resolve())]=sha(p)
 for p in [Path(__file__),Path(run.__file__),old/'endpoint-slot-table-failure-diagnosis-v1.json',out/'inputs/cases.tsv',out/'endpoint_verify_v1.rs.pending',host]:pins[str(p.resolve())]=sha(p)
 report=dict(schema='endpoint-slot-table-source-preparation-v2',compiled=False,native_executed=False,source_candidate_sha256=run.prep.S,extra_source_data={'address':'014477F8','bytes':4,'expected_value':20},expected_slot_offsets=[0,20,100,200,240,280,320,360,400],expected_kind_lengths=[20,80,100,40],prebuild_review_required=True,linked_and_runner_review_required=True,previous_failure_preserved=True,host_sha256=sha(host),pins=pins,limitations=['Static preparation only; actual linked carrier tables must be checked against original image.','No native endpoint or Rust parity claim.'])
 with (out/'preparation.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps({'host_sha256':sha(host),'out':str(out)}))
if __name__=='__main__':main()

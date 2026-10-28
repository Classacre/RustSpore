"""Single-attempt owned stage arithmetic runner; preflight never executes.
Reuses accepted original host/source contract and immutable promotion aliases.
No Rust projection is generated, no retries or automatic case exclusions.
"""
import argparse,hashlib,json,struct
from pathlib import Path
from types import SimpleNamespace
import anim_root_zero_child_stage_audit_v2 as audit
import anim_root_zero_child_stage_arithmetic_inputs_v1 as fixture_source
import anim_root_zero_child_stage_native_run_v1 as accepted
from cell_runtime_data_native_run import pinned,saved_file_run
ROOT=Path(__file__).resolve().parent.parent
CARRIER_SHA='9279eceacf329d898adc610b53dd19cd7cb91bf35b9908980fe620a1cb63ee67'
OWNED_SHA='6c90439efff9a470ee228146b3e662e6a8b31a2de8f0f40884347a286e6edb00'
PEER_SHA='aef2f4e95681462b13375276ab9a9f11095b3ba7fe316fdddbdd2f7908a1e2d7'
ALIASES={'crates/spore-core/src/anim_root_zero_child_stage.rs':'target/animation/root-zero-child-stage-promotion-v1/anim_root_zero_child_stage.rs','crates/spore-core/examples/anim_root_zero_child_stage_verify.rs.pending':'target/animation/root-zero-child-stage-promotion-v1/anim_root_zero_child_stage_verify.rs.pending'}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def resolve(p):return Path(p) if Path(p).is_absolute() else ROOT/p
def save(p,data):
 with Path(p).open('x') as f:json.dump(data,f,indent=2);f.write('\n')
def uncomment(path):return '\n'.join(line for line in Path(path).read_text().splitlines() if not line.lstrip().startswith('//!'))
def review(a):
 peer_path=resolve(a.peer_review);assert sha(peer_path)==PEER_SHA;peer=json.loads(peer_path.read_text());assert peer['no_blocking_static_issue_found'] is True and peer['native_executed'] is False
 resolved=[];substituted={}
 for name,value in peer['pins'].items():
  key=name.replace('\\','/');path=resolve(ALIASES.get(key,name));assert sha(path)==value,name;resolved.append(path)
  if key in ALIASES:substituted[key]=dict(immutable_archive=ALIASES[key],sha256=value)
 assert len(substituted)==2
 assert uncomment(ROOT/ALIASES['crates/spore-core/src/anim_root_zero_child_stage.rs'])==uncomment(ROOT/'crates/spore-core/src/anim_root_zero_child_stage.rs')
 old=(ROOT/ALIASES['crates/spore-core/examples/anim_root_zero_child_stage_verify.rs.pending']).read_text();current=(ROOT/'crates/spore-core/examples/anim_root_zero_child_stage_verify.rs').read_text();assert old.split('use std',1)[1]==current.split('use std',1)[1]
 assert sha(a.host_dir/'stage-carrier-x86.exe')==CARRIER_SHA and sha(a.host_dir/'host-x86.exe')==OWNED_SHA
 linked=audit.audit(SimpleNamespace(carrier=a.host_dir/'stage-carrier-x86.exe',owned_exe=a.host_dir/'host-x86.exe',owned_map=a.host_dir/'host-x86.map',source_exe=a.source_exe));assert linked==json.loads((a.host_dir/'independent-linked-audit.json').read_text()) and linked['static_pass']
 manifest=json.loads((a.fixture/'manifest.json').read_text());assert manifest['records']==576 and manifest['expected_numeric_projection_generated'] is False and manifest['graph_topology_definition_and_reserved_bytes_unchanged'] is True
 for name,value in manifest['pins'].items():assert sha(resolve(name))==value,name
 for name,value in manifest['files'].items():assert sha(a.fixture/name)==value,name
 seed=(a.fixture/'seed1256.bin').read_bytes()
 with resolve('target/animation/root-zero-child-stage-inputs-v3/input.bin').open('rb') as f:assert seed==f.read(1256)
 index=json.loads((a.fixture/'index.json').read_text());assert len(index)==576;counts={};row=0
 with(a.fixture/'input.bin').open('rb') as inputs:
  for family in fixture_source.FAMILIES:
   for control in fixture_source.CONTEX:
    for label,dt in fixture_source.THRESHOLDS:
     for cw in fixture_source.CONTROLS:
      raw=inputs.read(1256);assert len(raw)==1256;accepted.check(raw)
      # Reconstruct the declared owned inputs, never any output projection.
      expected=fixture_source.record(seed,family,cw,dt,control);assert raw==expected
      item=index[row];assert item['row']==row and item['family']==family['name'] and item['control']==control['name'] and item['dt_relation']==label and item['cw']==f'{cw:04X}' and item['input_sha256']==hashlib.sha256(raw).hexdigest()
      assert all(x==y for i,(x,y) in enumerate(zip(raw,seed)) if i not in fixture_source.allowed())
      key=family['name'];counts[key]=counts.get(key,0)+1;row+=1
  assert inputs.read(1)==b''
 assert row==576 and set(counts.values())=={72}
 old_native=resolve('target/animation/root-zero-child-stage-native-run-v1/native-v1/report.json');old_rust=resolve('target/animation/root-zero-child-stage-native-run-v1/rust-comparison-v1.json');nr=json.loads(old_native.read_text());rr=json.loads(old_rust.read_text());assert nr['records']==rr['records']==66636 and rr['accepted'] and nr['host_sha256']==CARRIER_SHA
 closure=dict(records=576,owned_families=8,explicit_controls=2,threshold_words=['358637BC','358637BD','358637BE'],x87_controls=12,input_magnitudes_le4=True,actual_zero_child_one_root_unchanged=True,accepted_host_peer=str(peer_path.relative_to(ROOT)),accepted_host_peer_sha256=PEER_SHA,immutable_promotion_aliases=substituted,current_numerical_core_equals_accepted_archive=True,current_verifier_body_equals_accepted_archive=True,actual_linked_audit_reproduced=True,no_expected_projection=True,old_original_cohort_unmodified=True,new_native_execution=False,scope='Owned arithmetic inputs only; no new source pose/actor/body observations,sourceFP/normalization policy unchanged.')
 return manifest,closure,list(dict.fromkeys([peer_path,*resolved,old_native,old_rust]))
def execute(a,manifest,closure,reviewed):
 out=a.out/'native-v1';out.mkdir(exist_ok=False)
 paths=[a.source_exe,Path(__file__),Path(fixture_source.__file__),Path(audit.__file__),Path(accepted.__file__),ROOT/'tools/cell_runtime_data_native_run.py',ROOT/'tools/cell_schema_probe.py',ROOT/'tools/native_crt_acos_probe.py',a.out/'preflight.json',*reviewed,*[resolve(p) for p in manifest['pins']],*[p for p in a.fixture.rglob('*') if p.is_file()],*[p for p in a.host_dir.iterdir() if p.is_file()]]
 paths=list(dict.fromkeys(p.resolve() for p in paths))
 with pinned(paths,out):
  try:
   # Exactly one native process call for the complete supplement; no smoke/retry.
   status,output=saved_file_run((a.host_dir/'stage-carrier-x86.exe').resolve(),(a.fixture/'input.bin').resolve(),out,'corpus',timeout=180)
   assert status['exit']==0 and status['stderr_bytes']==0,status
   observed=accepted.inspect(a.fixture/'input.bin',output,out/'native-projection728.bin');assert observed['records']==576 and observed['nonfinite_projection_cases']==0
   save(out/'report.json',dict(schema='zero-child-stage-owned-arithmetic-native-v1',native_executed=True,retries=0,native_process_attempts=1,records=576,host_sha256=CARRIER_SHA,input_sha256=sha(a.fixture/'input.bin'),output_sha256=sha(output),projection_sha256=sha(out/'native-projection728.bin'),observed=observed,expected_numeric_projection_from_rust=False,rust_accepted=False,scope='Owned finite alternate merge/hemisphere/clamp/delta threshold supplement. Original host source/graph contract unchanged. All mismatches/errors remain failures; no actor/body/generalstage admission.'))
  except Exception as ex:
   save(out/'failure.json',dict(native_execution_requested=True,retries=0,completed=False,error_type=type(ex).__name__,error=str(ex),automatic_exclusions=False));raise

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--fixture',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--peer-review',type=Path,default=ROOT/'target/animation/root-zero-child-stage-independent-font-review-v1.json');p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--run',action='store_true');a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
 manifest,closure,reviewed=review(a);a.out.mkdir(parents=True,exist_ok=False);save(a.out/'preflight.json',dict(schema='zero-child-stage-owned-arithmetic-preflight-v1',runner_sha256=sha(__file__),fixture_manifest_sha256=sha(a.fixture/'manifest.json'),closure=closure,native_executed=False,execution_requested=a.run))
 if a.run:execute(a,manifest,closure,reviewed)
 else:print(json.dumps(dict(preflight=True,records=closure['records'],native_executed=False)))
if __name__=='__main__':main()

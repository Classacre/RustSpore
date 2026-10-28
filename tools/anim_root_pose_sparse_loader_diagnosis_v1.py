"""Read-only PE loader structure diagnosis; does not load or execute image."""
import hashlib,json,struct,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
from anim_root_pose_sparse_carrier_audit_v1 import Image

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(out):
 directory=ROOT/'target/animation/root-pose-sparse-carrier-root-v1';rows=[]
 for name in ['host-x86.exe','carrier-x86.exe']:
  p=directory/name;im=Image(p);gaps=[]
  for a,b in zip(im.sections,im.sections[1:]):
   expected=a['va']+((a['vs']+4095)&~4095)
   if expected!=b['va']:gaps.append(dict(previous=a['name'].rstrip(b'\0').decode(),next=b['name'].rstrip(b'\0').decode(),start_va=f'{expected:08X}',end_va=f"{b['va']:08X}",bytes=b['va']-expected))
  rows.append(dict(path=str(p.relative_to(ROOT)),sha256=sha(p),base=f'{im.base:08X}',image_bytes=im.image,physical_bytes=len(im.b),sections=len(im.sections),aligned=True,nonoverlap=True,adjacent=not gaps,virtual_gaps=gaps))
 report=dict(schema='sparse-carrier-loader-structure-diagnosis-v1',native_executed=False,carrier_admitted=False,previous_static_admission_superseded_for_loader_structure=True,blocking_issue='Carrier section virtual ranges contain five holes; prior audit checked non-overlap, not required adjacency.',loader_failure_observed_by_root='CreateProcess WinError193 before process creation; no source/input execution',causal_status='Concrete PE-spec violation; sufficient reason to withhold admission. Relationship to actual loader193 is inferred, not a captured loader internal diagnostic.',primary_source='https://learn.microsoft.com/en-us/windows/win32/debug/pe-format',primary_rule='Section VAs must be ascending and adjacent, and aligned to SectionAlignment.',images=rows,proposed_new_version='Fill each virtual hole with explicit uninitialized/no-RWX zero-raw section. Keep all current original/owned sections at exact VAs, preserve source/owned bytes and image extent. New auditor must require adjacency, accept zero-raw zero-fill sections, and keep them out of physical raw-payload adjacency; all source, import, guard and control checks rerun. Loader acceptance remains root-owned and untested.',pins={str(p.relative_to(ROOT)):sha(p) for p in [Path(__file__),ROOT/'tools/anim_root_pose_sparse_carrier_v2.py',ROOT/'tools/anim_root_pose_sparse_carrier_audit_v1.py',ROOT/'target/animation/root-pose-sparse-carrier-independent-font-review-v2.json',directory/'host-x86.exe',directory/'carrier-x86.exe']})
 p=Path(out);p.resolve().relative_to((ROOT/'target').resolve())
 with p.open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(sha(p));print(json.dumps(rows,indent=2))
if __name__=='__main__':main(sys.argv[1])

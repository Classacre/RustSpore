"""Run the bounded comparison only after two independent review attestations."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

from cell_progress_slice_prepare_v1 import validate

ROOT=Path(__file__).resolve().parent.parent
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser(); p.add_argument('--prep',type=Path,required=True); p.add_argument('--host-dir',type=Path,required=True)
 p.add_argument('--prebuild-review',type=Path,required=True); p.add_argument('--linked-review',type=Path,required=True)
 p.add_argument('--native',type=Path,required=True); p.add_argument('--adapter',type=Path,required=True); p.add_argument('--out',type=Path,required=True); a=p.parse_args()
 a.out.resolve().relative_to((ROOT/'target').resolve())
 m=json.loads((a.host_dir/'manifest.json').read_text()); pre=json.loads(a.prebuild_review.read_text()); linked=json.loads(a.linked_review.read_text())
 if pre.get('decision')!='pass' or pre.get('independent') is not True or pre.get('host_sha256')!=sha(a.host_dir/'host.c') or pre.get('adapter_sha256')!=sha(a.host_dir/'adapter.rs') or pre.get('runner_sha256')!=sha(Path(__file__)) or pre.get('manifest_sha256')!=sha(a.host_dir/'manifest.json'): raise ValueError('prebuild review missing, rejected, or stale')
 if linked.get('decision')!='pass' or linked.get('independent') is not True or linked.get('prebuild_review_sha256')!=sha(a.prebuild_review) or linked.get('native_sha256')!=sha(a.native) or linked.get('adapter_sha256')!=sha(a.adapter): raise ValueError('linked review missing, rejected, or stale')
 if m['host_sha256']!=sha(a.host_dir/'host.c') or m['adapter_sha256']!=sha(a.host_dir/'adapter.rs') or m['runner_sha256']!=sha(Path(__file__)) or m['rust_source_sha256']!=sha(ROOT/'crates/spore-core/src/cell_progression_v1.rs') or m['prep_audit_sha256']!=sha(a.prep/'audit.json') or m['inputs_sha256']!=sha(a.prep/'inputs.json'): raise ValueError('preparation changed')
 cases=json.loads((a.prep/'inputs.json').read_text()); audit=json.loads((a.prep/'audit.json').read_text())
 if len(cases)!=audit['input_count']: raise ValueError('input count mismatch')
 for x in cases: validate(x)
 exe=a.native.resolve(); adapter=a.adapter.resolve()
 a.out.mkdir(parents=True,exist_ok=False)
 journal=a.out/'observations.jsonl'
 def record(item):
  with journal.open('a',encoding='utf-8',newline='\n') as f: f.write(json.dumps(item)+'\n')
 def run(path,args):
  try:
   result=subprocess.run([str(path),*args],capture_output=True,text=True,timeout=10,stdin=subprocess.DEVNULL,creationflags=subprocess.CREATE_NO_WINDOW)
   return result
  except Exception as e:
   record(dict(type='launch_error',executable=str(path),args=args,error=repr(e)))
   raise
 valid=['00000000']*6
 negatives=[[], valid[:-1], valid+['00000000'], ['0000000g']+valid[1:], ['100000000']+valid[1:], valid[:5]+['00000100'], ['FFFFFFFF']+valid[1:]]
 for path in (exe,adapter):
  for args in negatives:
   r=run(path,args)
   record(dict(type='negative',executable=str(path),args=args,returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
   if r.returncode==0 or r.stdout: raise ValueError(f'negative protocol accepted: {path} {args!r}')
 rows=[]; pattern=re.compile(r'^[0-9a-f]{8}(?: [0-9a-f]{8}){3} [01]\n$')
 for case in cases:
  args=[f'{case[k]&0xffffffff:08x}' for k in ('progress','counter20','counter24','counter28','amount','source')]
  n=run(exe,args); r=run(adapter,args)
  record(dict(type='case',index=len(rows),input=case,native_returncode=n.returncode,native_stdout=n.stdout,native_stderr=n.stderr,rust_returncode=r.returncode,rust_stdout=r.stdout,rust_stderr=r.stderr))
  if n.returncode or r.returncode or n.stderr or r.stderr or not pattern.fullmatch(n.stdout) or not pattern.fullmatch(r.stdout): raise ValueError(f'protocol failure case {len(rows)} native={n.returncode} adapter={r.returncode}')
  rows.append(dict(input=case,native=n.stdout.strip(),rust=r.stdout.strip(),match=n.stdout==r.stdout))
 report=dict(schema='cell-progress-differential-v2',case_count=len(rows),match_count=sum(x['match'] for x in rows),native_sha256=sha(exe),adapter_sha256=sha(adapter),inputs_sha256=sha(a.prep/'inputs.json'),prebuild_review_sha256=sha(a.prebuild_review),linked_review_sha256=sha(a.linked_review),native_executed=True,scope=m['limitations'],cases=rows)
 (a.out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
 if report['match_count']!=len(rows): raise SystemExit('differential mismatch; see comparison.json')
 print(f"matched {len(rows)} cases")
if __name__=='__main__': main()

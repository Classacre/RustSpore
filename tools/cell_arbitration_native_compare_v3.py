"""Explicitly run already-reviewed local executables; no build or source extraction."""
import argparse, itertools, subprocess, json, sys
from cell_arbitration_native_build_v3 import sha, verify, verify_cargo
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent

def main():
 p=argparse.ArgumentParser(); p.add_argument('--prep',type=Path,required=True); p.add_argument('--build-manifest-sha256',required=True); p.add_argument('--out',type=Path,required=True); a=p.parse_args()
 prep=a.prep.resolve(); prep.relative_to((ROOT/'target').resolve()); a.native=prep/'host.exe'
 bm_path=prep/'build-manifest.json'; bm_hash=sha(bm_path)
 if bm_hash!=a.build_manifest_sha256.lower(): raise ValueError('build manifest pin mismatch')
 bm=json.loads(bm_path.read_text()); a.rust=Path(bm['adapter_path']); mp=prep/'manifest.json'; m=json.loads(mp.read_text())
 if bm.get('schema')!='cell-arbitration-build-v3' or m.get('schema')!='cell-arbitration-prep-v3': raise ValueError('invalid manifest version')
 def check():
  if sha(bm_path)!=bm_hash or sha(mp)!=bm['prep_manifest_sha256']: raise ValueError('manifest changed')
  verify(prep,m,Path(bm['source_exe_path']))
  verify_cargo(prep,m,bm['prep_manifest_sha256'],Path(bm['cargo_manifest_path']),bm['cargo_manifest_sha256'],a.rust,bm['adapter_sha256'])
  for info in bm['tools'].values():
   if sha(info['path'])!=info['sha256']: raise ValueError('compiler changed')
  if sha(a.native)!=bm['host_sha256'] or sha(a.rust)!=bm['adapter_sha256']: raise ValueError('executable changed')
  if sha(sys.executable)!=bm['python_sha256']: raise ValueError('Python executable changed')
 check()
 for path in (a.native,a.rust,a.out): path.resolve().relative_to((ROOT/'target').resolve())
 if a.out.exists(): raise ValueError('output already exists')
 selectors=[0x80000000,0xffffffff,0,4,7,8,9,0x7fffffff]
 floats=[0,0x80000000,1,0x80000001,0x007fffff,0x00800000,0x3f800000,0xbf800000,0x7f7fffff,0x7f800000,0xff800000,0x7fc01234,0x7f800001,0xff800001]
 cases=[]
 def case(os,cs,od,cd,player):
  old=[0x100+i for i in range(12)]; cand=[0x200+i for i in range(12)]; old[2]=os; cand[2]=cs; old[5]=od; cand[5]=cd
  cases.append(old+cand+[player])
 for os,cs in itertools.product(selectors,repeat=2): case(os,cs,0x3f800000,0x40000000,0x999)
 for od,cd in itertools.product(floats,repeat=2): case(4,4,od,cd,0x999)
 for os,od,cd,player in itertools.product([7,8,9],[0x3f800000,0x7fc01234],[0,0x40000000,0x7f800001],[0x200,0x201,0x999]): case(os,8,od,cd,player)
 failures=[]; fp_flags={}; completed=0
 for i,case_data in enumerate(cases):
  args=[f'{n:08x}' for n in case_data]
  try:
   native=subprocess.run([str(a.native.resolve()),*args],capture_output=True,text=True,timeout=10)
   rust=subprocess.run([str(a.rust.resolve()),*args],capture_output=True,text=True,timeout=10)
  except subprocess.TimeoutExpired as exc:
   failures.append({'case':i,'input':args,'timeout':str(exc),'limitation':'direct child killed; descendants are not contained'}); break
  if native.returncode or rust.returncode or native.stdout!=rust.stdout: failures.append({'case':i,'input':args,'native_exit':native.returncode,'rust_exit':rust.returncode,'native_output':native.stdout,'rust_output':rust.stdout,'native_stderr':native.stderr,'rust_stderr':rust.stderr})
  completed+=1
  flag=native.stderr.strip(); fp_flags[flag]=fp_flags.get(flag,0)+1
 check()
 result={'planned_cases':len(cases),'completed_cases':completed,'build_manifest_sha256':bm_hash,'prep_manifest_sha256':bm['prep_manifest_sha256'],'cargo_manifest_sha256':bm['cargo_manifest_sha256'],'binaries':{'host_sha256':bm['host_sha256'],'adapter_sha256':bm['adapter_sha256']},'tools':bm['tools'],'python_sha256':bm['python_sha256'],'failures':failures,'mxcsr_counts':fp_flags,'scope':'final 12 output words only; independent write-path flags, aliasing and FP exceptions are not compared'}
 a.out.parent.mkdir(parents=True,exist_ok=True); a.out.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8'); print(json.dumps({'cases':len(cases),'failures':len(failures),'result':str(a.out)}))
 if failures: raise SystemExit(1)
if __name__=='__main__': main()

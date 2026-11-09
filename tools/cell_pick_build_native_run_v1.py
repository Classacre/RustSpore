"""Run once only after independent review; no retries or implicit build."""
import argparse, hashlib, json, subprocess
from pathlib import Path
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--host',type=Path,required=True);p.add_argument('--input',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--host-sha256',required=True);p.add_argument('--input-sha256',required=True);p.add_argument('--execute-reviewed-host',action='store_true');a=p.parse_args()
 if not a.execute_reviewed_host: raise SystemExit('Explicit reviewed-host execution flag required; no process launched')
 root=Path(__file__).resolve().parent.parent; out=a.out.resolve();out.relative_to((root/'target').resolve());out.mkdir(parents=True,exist_ok=False)
 assert digest(a.host)==a.host_sha256 and digest(a.input)==a.input_sha256
 raw=a.input.read_bytes();assert len(raw)%232==0
 with (out/'output.bin').open('xb') as f:
  proc=subprocess.run([str(a.host.resolve())],input=raw,stdout=f,stderr=subprocess.PIPE,timeout=120,check=False)
 result={'returncode':proc.returncode,'stderr':proc.stderr.decode('utf8','replace'),'input_sha256':digest(a.input),'host_sha256':digest(a.host),'output_sha256':digest(out/'output.bin'),'bytes':(out/'output.bin').stat().st_size,'expected_bytes':len(raw)//232*792}
 (out/'run.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf8')
 if proc.returncode or result['bytes']!=result['expected_bytes']:raise SystemExit('Invalid run: '+json.dumps(result))
 print(json.dumps(result))
if __name__=='__main__':main()

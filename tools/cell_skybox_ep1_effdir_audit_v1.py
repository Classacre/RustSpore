"""Audit the three installed EP1 effect directories for the Cell skybox export."""
import hashlib,json,sys
from pathlib import Path
from dbpf_extract import payload
from gmdl_skin_probe import indexes
from cell_skybox_effdir_audit_v1 import audit
def sha(b):return hashlib.sha256(b).hexdigest()
def main():
 package=Path(sys.argv[1]);out=Path(sys.argv[2]);root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 before=sha(package.read_bytes()); entries=[e for e in indexes(package) if e['type']==0xea5118b0 and e['instance'] in (0x408a4300c1195706,0x408a4301c1195706,0x408a4302c1195706)]
 if len(entries)!=3 or len({e['instance'] for e in entries})!=3:raise ValueError('expected three unique variant rows')
 rows=[]
 for e in entries:
  data=payload(package,e)
  if len(data)!=e['dec']:raise ValueError('decoded length mismatch')
  parsed=audit(data)
  rows.append({'group':f"{e['group']:08X}",'instance':f"{e['instance']:016X}",'size':len(data),'payload_sha256':sha(data),'matches':parsed['matches']})
 after=sha(package.read_bytes())
 if before!=after:raise ValueError('package endpoint changed')
 report={'package':str(package.resolve()),'package_sha256_before_after':before,'target':'05C7A4D0 group 00000000','rows':rows,'limitations':'Explicit package and three rows; package reopen requires stable input; no live provider or selection.'}
 with out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()

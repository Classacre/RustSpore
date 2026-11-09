"""Bounded search of explicit installed config text for one property key."""
import argparse,hashlib,json,re
from pathlib import Path
def fnv(s):
 h=0x811c9dc5
 for c in s.lower().encode('ascii'):h=((h*0x1000193)^c)&0xffffffff
 return h
def main():
 p=argparse.ArgumentParser();p.add_argument('config',type=Path);p.add_argument('out',type=Path);a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 files=[]
 for name in ('Properties.txt','ConfigManager.txt','VideoCards.txt'):
  path=a.config/name;data=path.read_bytes();text=data.decode('utf-8-sig');tokens=set(re.findall(r'[A-Za-z_][A-Za-z_0-9]+',text)); hits=sorted(t for t in tokens if fnv(t)==0x03ebeecc)
  literal=[i for i,line in enumerate(text.splitlines(),1) if re.search(r'(?i)\b(?:0x)?0*3ebeecc\b',line)]
  files.append({'path':str(path.resolve()),'sha256':hashlib.sha256(data).hexdigest(),'matching_tokens':hits,'literal_lines':literal})
 report={'key':'03EBEECC','method':'ASCII identifier tokens, lowercase FNV-1 multiply then xor and literal hex search','files':files,'limits':'Explicit three config files only; hash matches are candidates, not proof of identity or effective value.'}
 with a.out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()

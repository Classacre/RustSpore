"""Read one explicitly supplied text preference; never copy unrelated entries."""
import argparse,datetime,hashlib,json,re
from pathlib import Path
def main():
 p=argparse.ArgumentParser();p.add_argument('prefs',type=Path);p.add_argument('out',type=Path);a=p.parse_args();root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 before=a.prefs.stat();data=a.prefs.read_bytes();after=a.prefs.stat();text=data.decode('utf-8-sig')
 matches=[{'line':i,'value':int(m.group(1))} for i,l in enumerate(text.splitlines(),1) if (m:=re.fullmatch(r'\s*OptionEffects\s+([0-9]+)\s*',l))]
 if len(matches)!=1 or (before.st_size,before.st_mtime_ns)!=(after.st_size,after.st_mtime_ns):raise ValueError('ambiguous option or file changed')
 report={'path':str(a.prefs.resolve()),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'last_write_utc':datetime.datetime.fromtimestamp(before.st_mtime,datetime.timezone.utc).isoformat(),'matches':matches,'limits':'saved file read once, no live process observed; stat endpoints do not prove continuous stability'}
 with a.out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()

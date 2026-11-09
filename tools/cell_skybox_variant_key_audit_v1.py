"""Bounded static Cell directory-key projection; caller supplies watcher values."""
import argparse,hashlib,json
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
def sha(b):return hashlib.sha256(b).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('out',type=Path);a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 s=SourceImage(a.exe);spans=[(0x7e97b0,12),(0x7e98c4,16),(0x7e9dee,22),(0x7e7403,7)]
 values=[s.read(addr,n) for addr,n in spans]
 expected=['0fb685880000000d00024540','8b8d8800000033c881e1ff00000033c8','8d8d88000000e8e7c5ffff84c074078bcde82cf9ffff','6a0168cceeeb03']
 if [v.hex() for v in values]!=expected:raise ValueError('instruction pin mismatch: '+str([v.hex() for v in values]))
 rows=[]
 for x in (0,1,2,255,256,0x12345678,0xffffffff):
  rows.append({'watcher_u32':f'{x:08X}','explicit_group':f'{0x40450200|(x&255):08X}','template_408A4300':f'{(0x408a4300&0xffffff00)|(x&255):08X}'})
 report={'source_sha256':SOURCE_SHA256,'spans':[{'address':f'{a:08X}','bytes':b.hex(),'sha256':sha(b)} for (a,_),b in zip(spans,values)],'rows':rows,'scope':'static arithmetic projection only; supplied watcher values and template, no provider ordering or active choice'}
 with a.out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()

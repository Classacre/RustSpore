"""Static base-package 030BDEE3 version-10 discriminator census; no source execution."""
import collections,hashlib,json,struct
from pathlib import Path
from dbpf_extract import parse_index
from refpack import decompress
base=Path('D:/Games/SPORE Collection/Data')
out=Path('target/source-cell/manifest-discriminator-census-v1');out.mkdir(parents=True,exist_ok=False)
counts=collections.Counter(); failures=[]; packages=[]; records=[]
for pkg in sorted(base.glob('*.package')):
 try: entries,_=parse_index(pkg)
 except Exception as exc:
  failures.append({'package':pkg.name,'error':str(exc)});continue
 packages.append(pkg.name)
 with pkg.open('rb') as f:
  for e in entries:
   if e['type']!=0x030bdee3:continue
   f.seek(e['offset']);data=f.read(e['stored'])
   if e['dec']>e['stored']:data=decompress(data,out_size=None,header=True)[0]
   if len(data)<64:raise ValueError(f'short {pkg.name} {e}')
   version=struct.unpack_from('>I',data,0)[0];discriminator=struct.unpack_from('>I',data,60)[0]
   counts[f'{version}:'+('zero' if discriminator==0 else 'nonzero')]+=1
   records.append({'package':pkg.name,'group':f"{e['group']:08X}",'instance':f"{e['instance']:016X}",'sha256':hashlib.sha256(data).hexdigest(),'version':version,'discriminator':f'{discriminator:08X}'})
result={'scope':'parseable base Data/*.package only; offsets require ordinary framing; not runtime evidence','source_executed':False,'parsed_packages':packages,'skipped_packages':failures,'counts':dict(counts),'records':records}
(out/'report.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf8');print(json.dumps({'records':len(records),'counts':dict(counts),'skipped_packages':len(failures)}))

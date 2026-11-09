"""Prepare explicit 030BDEE3 prefix samples and independent locale rows."""
import argparse,hashlib,json,struct
from pathlib import Path
from dbpf_extract import parse_index
from refpack import decompress
from locale_table_probe import tokenize
from type_030b_validate import walk
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
p=argparse.ArgumentParser();p.add_argument('--package',type=Path,required=True);p.add_argument('--locale',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--scope',type=lambda x:int(x,16),required=True);p.add_argument('ids',nargs='+',type=lambda x:int(x,16));a=p.parse_args()
out=a.out.resolve();out.relative_to((Path('target').resolve()));out.mkdir(parents=True,exist_ok=False)
package_hash=sha(a.package);loc=a.locale.read_bytes();locale_hash=hashlib.sha256(loc).hexdigest();entries,_,_=tokenize(loc);table={i:t for i,_,_,t in entries};index,_=parse_index(a.package);rows=[]
for low in a.ids:
 matches=[e for e in index if e['type']==0x030bdee3 and e['group']==0x1ffff and e['instance']==(a.scope<<32)|low]
 if len(matches)!=1:raise ValueError('missing or ambiguous key')
 e=matches[0]
 with a.package.open('rb') as f:f.seek(e['offset']);data=f.read(e['stored'])
 if e['dec']>e['stored']:data=decompress(data,out_size=None,header=True)[0]
 if struct.unpack_from('>I',data,0)[0]!=10:raise ValueError('unsupported version')
 values=struct.unpack_from('>5I',data,60)
 if values[0]!=0xffffffff or values[1]!=0x04dc2f94:raise ValueError('unexpected prefix')
 fields=[]
 for label,ident in zip(('first','name','optional'),values[2:]):
  raw=table.get(ident,b'');fields.append({'stage':label,'id':f'{ident:08X}','sentinel':ident==0xffffffff,'nonempty':bool(raw and raw[:2]!=b'\0\0'),'raw_utf16le_hex':raw.hex()})
 (out/f'{low:08X}.bin').write_bytes(data)
 rows.append({'key':f'{a.scope:08X}{low:08X}:030BDEE3:0001FFFF','decoded_sha256':hashlib.sha256(data).hexdigest(),'generic_manifest_walk':walk(data),'fields':fields})
if sha(a.package)!=package_hash or sha(a.locale)!=locale_hash:raise ValueError('source changed')
r={'source_executed':False,'package':str(a.package),'package_sha256':package_hash,'locale':'explicit en-US 04DC2F94','locale_sha256':locale_hash,'rows':rows,'scope':'raw candidate joins only; full native reader, provider selection, output processing and live actor unresolved'}
(out/'report.json').write_text(json.dumps(r,indent=2)+'\n',encoding='utf8');print('rows',len(rows),'report SHA256',sha(out/'report.json'))

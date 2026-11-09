"""Search selected package PROP payloads for a single property identifier."""
import argparse,hashlib,json
from pathlib import Path
from gmdl_skin_probe import indexes
from dbpf_extract import payload
from anim_rig_mapping_probe import properties
def sha(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for b in iter(lambda:f.read(4*1024*1024),b''):h.update(b)
 return h.hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('install',type=Path);p.add_argument('out',type=Path);a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 report={'scope':'raw big-endian identifier scan then bounded parser on hits, selected packages only; no runtime selection','rows':[]}
 for name in ('Data/PatchData.package','Data/Spore_Game.package','DataEP1/Spore_EP1_Data.package'):
  path=a.install/name;before=sha(path);entries=[e for e in indexes(path) if e['type']==0x00b1b104];r={'package':name,'before':before,'count':len(entries),'scanned':0,'errors':[],'hits':[]}
  for e in entries:
   try:
    if e['dec']>4*1024*1024:raise ValueError('decoded size exceeds 4 MiB bound')
    b=payload(path,e)
    if len(b)!=e['dec']:raise ValueError('decoded size mismatch')
    r['scanned']+=1
    if bytes.fromhex('03ebeecc') in b:
     hit={'group':f"{e['group']:08X}",'instance':f"{e['instance']:016X}",'sha256':hashlib.sha256(b).hexdigest(),'raw_offsets':[]}
     i=b.find(bytes.fromhex('03ebeecc'))
     while i!=-1:hit['raw_offsets'].append(i);i=b.find(bytes.fromhex('03ebeecc'),i+1)
     try:hit['parsed_properties']=[{'type':f'{t:04X}','spec':f'{s:04X}','value_hex':v.hex()} for ident,t,s,v in properties(b) if ident==0x03ebeecc]
     except Exception as ex:hit['parse_error']=str(ex)
     r['hits'].append(hit)
   except Exception as ex:r['errors'].append({'instance':f"{e['instance']:016X}",'error':str(ex)})
  r['after']=sha(path);report['rows'].append(r);print(name,'scanned',r['scanned'],'hits',len(r['hits']),'errors',len(r['errors']),flush=True)
 with a.out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
if __name__=='__main__':main()

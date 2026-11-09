"""Audit explicit option catalogs and OptionEffects row in three packages."""
import argparse,hashlib,json,struct
from pathlib import Path
from gmdl_skin_probe import indexes
from dbpf_extract import payload
from anim_rig_mapping_probe import properties
def sha(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for b in iter(lambda:f.read(4*1024*1024),b''):h.update(b)
 return h.hexdigest()
def array(value):
 count,stride=struct.unpack_from('>II',value)
 if stride!=4 or len(value)!=8+count*stride:raise ValueError('array framing')
 return list(struct.unpack_from('>'+str(count)+'I',value,8))
def main():
 p=argparse.ArgumentParser();p.add_argument('install',type=Path);p.add_argument('out',type=Path);a=p.parse_args();root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 rows=[]
 for name in ('Data/PatchData.package','Data/Spore_Game.package','DataEP1/Spore_EP1_Data.package'):
  path=a.install/name;before=sha(path);entries=[e for e in indexes(path) if e['type']==0x00b1b104 and e['instance']==0x4047000000000000]
  if len(entries)!=1:raise ValueError('catalog not unique')
  e=entries[0];data=payload(path,e)
  if len(data)!=e['dec']:raise ValueError('decoded length')
  wanted={i:array(v) for i,t,s,v in properties(data) if i in (0x05daafff,0x05dab000,0x05dab001) and t==0x0a and s==0x9c}
  if len(wanted)!=3 or len({len(x) for x in wanted.values()})!=1:raise ValueError('catalog arrays')
  indexes_found=[i for i,x in enumerate(wanted[0x05daafff]) if x==0x046170a0]
  if len(indexes_found)!=1:raise ValueError('OptionEffects missing or repeated')
  i=indexes_found[0];after=sha(path)
  rows.append({'package':name,'package_sha256_before':before,'package_sha256_after':after,'group':f"{e['group']:08X}",'payload_sha256':hashlib.sha256(data).hexdigest(),'array_count':len(wanted[0x05daafff]),'option_index':i,'default':wanted[0x05dab000][i],'limit':wanted[0x05dab001][i]})
 report={'rows':rows,'scope':'explicit source catalog projection; no provider precedence, user preferences or active selection'}
 with a.out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()

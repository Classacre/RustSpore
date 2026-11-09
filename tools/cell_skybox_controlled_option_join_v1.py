"""Read-only controlled OptionEffects -> source-backed directory/raster join.

Provider priority, index groups, and stable path admission are caller supplied.
This does not observe runtime configuration, provider precedence, material or camera.
"""
import argparse,hashlib,json,re,struct
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from gmdl_skin_probe import indexes
from dbpf_extract import payload
from anim_rig_mapping_probe import properties
from cell_skybox_effdir_audit_v1 import audit
def digest(b):return hashlib.sha256(b).hexdigest()
def sha(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for b in iter(lambda:f.read(4*1024*1024),b''):h.update(b)
 return h.hexdigest()
def select(providers,typ,group,instance):
 for name,rows in providers:
  hits=[x for x in rows if (x['type'],x['group'],x['instance'])==(typ,group,instance)]
  if len(hits)>1:raise ValueError('duplicate exact key within provider '+str(name))
  if hits:return name,hits[0]
 raise ValueError(f'no provider for {typ:08X}:{group:08X}:{instance:016X}')
def pick_option(catalog,option):
 found={}
 for ident,typ,spec,val in properties(catalog):
  if ident in (0x05daafff,0x05dab000,0x05dab001):
   if ident in found or typ!=10 or spec!=0x9c or len(val)<8:raise ValueError('catalog field')
   count,stride=struct.unpack_from('>II',val)
   if stride!=4 or len(val)!=8+4*count or count>4096:raise ValueError('catalog array')
   found[ident]=list(struct.unpack_from('>'+str(count)+'I',val,8))
 if len(found)!=3 or len({len(v) for v in found.values()})!=1:raise ValueError('catalog array count')
 indices=[i for i,v in enumerate(found[0x05daafff]) if v==0x046170a0]
 if len(indices)!=1:raise ValueError('OptionEffects not unique')
 i=indices[0];start=found[0x05dab000][i];limit=found[0x05dab001][i]
 if not start<=option<limit or option>255:raise ValueError('option outside admitted catalog range')
 return i,start,limit
def main():
 p=argparse.ArgumentParser();p.add_argument('--source',required=True,type=Path);p.add_argument('--prefs',required=True,type=Path);p.add_argument('--prefs-sha256',required=True);p.add_argument('--option',required=True,type=int);p.add_argument('--provider',nargs=2,action='append',required=True,metavar=('PACKAGE','SHA256'));p.add_argument('--catalog-group',required=True,type=lambda x:int(x,16));p.add_argument('--option-group',required=True,type=lambda x:int(x,16));p.add_argument('--effdir-group',required=True,type=lambda x:int(x,16));p.add_argument('--raster-group',required=True,type=lambda x:int(x,16));p.add_argument('--out',required=True,type=Path);p.add_argument('--admit-stable-paths',required=True,action='store_true');a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 SourceImage(a.source);prefs=a.prefs.read_bytes()
 if digest(prefs)!=a.prefs_sha256.lower():raise ValueError('preference pin')
 vals=[int(m.group(1)) for line in prefs.decode('utf-8-sig').splitlines() if (m:=re.fullmatch(r'\s*OptionEffects\s+([0-9]+)\s*',line))]
 if vals!=[a.option]:raise ValueError('saved option mismatch or ambiguity')
 paths=[Path(path).resolve(strict=True) for path,_ in a.provider]
 if len(set(paths))!=len(paths):raise ValueError('duplicate provider path')
 pins=[h.lower() for _,h in a.provider]
 if any(len(x)!=64 for x in pins) or [sha(path) for path in paths]!=pins:raise ValueError('provider pin')
 providers=[(path,indexes(path)) for path in paths];used=[]
 def load(label,t,g,i,limit):
  path,e=select(providers,t,g,i)
  if e['dec']>limit:raise ValueError('resource size bound')
  b=payload(path,e)
  if len(b)!=e['dec']:raise ValueError('resource size mismatch')
  used.append({'label':label,'package':str(path),'type':f'{t:08X}','index_group':f'{g:08X}','instance':f'{i:016X}','sha256':digest(b),'bytes':len(b)})
  return b
 cat=load('option_catalog',0xb1b104,a.catalog_group,0x4047000000000000,1024*1024);idx,start,end=pick_option(cat,a.option)
 prop=load('selected_option_list',0xb1b104,a.option_group,((0x40470100|a.option)<<32)|0x046170a0,1024*1024)
 values=[(t,s,v) for ident,t,s,v in properties(prop) if ident==0x03ebeecc]
 if len(values)!=1 or values[0][0:2]!=(9,0) or len(values[0][2])!=4:raise ValueError('watcher property not unique scalar INT32')
 watcher=struct.unpack('>I',values[0][2])[0];directory_group=0x40450200|(watcher&255)
 eff=load('effect_directory',0xea5118b0,a.effdir_group,(directory_group<<32)|0xc1195706,16*1024*1024);parsed=audit(eff)
 matches=[x for x in parsed['matches'] if x['group']=='00000000' and x['kind']=='visual effect']
 if len(matches)!=1:raise ValueError('effect export not unique')
 screens=[x['screen'] for x in matches[0]['visual']['blocks'] if x.get('type')==9 and 'screen' in x]
 if len(screens)!=1 or screens[0]['mode']!=5:raise ValueError('screen component not unique mode 5')
 screen=screens[0];tex_instance=int(screen['texture_instance'],16);tex_group=int(screen['texture_group'],16)
 if tex_group!=0:raise ValueError('texture scope outside fixed candidate')
 raster=load('B_type_candidate',0x2f4e681b,a.raster_group,tex_instance,16*1024*1024)
 if digest(raster)!='5705df1ab03e94c58c8a890383026e68c68dfe64ba488d4d767f2706d8ad8aec':raise ValueError('candidate payload differs')
 if [sha(path) for path in paths]!=pins or sha(a.source)!=SOURCE_SHA256 or sha(a.prefs)!=a.prefs_sha256.lower():raise ValueError('input endpoint changed')
 report={'status':'controlled_join','saved_option':a.option,'catalog_option_position':idx,'catalog_index_bounds':[start,end],'watcher_u32':watcher,'directory_group':f'{directory_group:08X}','screen':screen,'requested_texture':f'{tex_instance:08X}:2F4E681C:{tex_group:08X}','resources':used,'provider_priority':[{'path':str(path),'sha256':pin} for path,pin in zip(paths,pins)],'source_sha256':SOURCE_SHA256,'prefs_sha256':a.prefs_sha256.lower(),'admissions':'caller-selected first-match provider order and index groups; successful catalog load and option merge, no later property change; stable paths; candidate B-type fallback supplied directly; not live provider/selection/material/camera; source alternatives and conversion remain unobserved'}
 with a.out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()

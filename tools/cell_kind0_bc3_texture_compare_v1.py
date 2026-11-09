"""Controlled old/corrected BC3 decoding of one explicitly pinned source texture."""
import argparse,hashlib,json,struct
from pathlib import Path
from PIL import Image
from dbpf_extract import payload
from gmdl_skin_probe import indexes
from raster_extract import decode_dxt1_block,decode_dxt5_block
def sha(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for b in iter(lambda:f.read(4*1024*1024),b''):h.update(b)
 return h.hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('package',type=Path);p.add_argument('out',type=Path);p.add_argument('--admit-stable-path',required=True,action='store_true');a=p.parse_args();root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 before=sha(a.package)
 if before!='e01bb86d8ddf02c56d52394b4e1db7181c1e1ebe1878bf6fb72c4ee523fcedd1':raise ValueError('package pin')
 entries=[e for e in indexes(a.package) if (e['type'],e['instance'])==(0x2f4e681b,0xdae8ce17)]
 if len(entries)!=1 or entries[0]['dec']!=95536:raise ValueError('RW4 key/extent')
 b=payload(a.package,entries[0]);digest=hashlib.sha256(b).hexdigest()
 if digest!='bbd99a741d41a3606a310f242e14e5f85dc7b49af454972ff02124b16e318592':raise ValueError('RW4 payload pin')
 count,table,base=struct.unpack_from('<I',b,36)[0],struct.unpack_from('<I',b,48)[0],struct.unpack_from('<I',b,68)[0]
 if count!=20 or table+24*count>len(b):raise ValueError('section framing')
 def section(i):return struct.unpack_from('<6I',b,table+i*24)
 ras=section(17);data=section(16);link=section(15)
 if ras[5]!=0x20003 or data[5]!=0x10030 or link[5]!=0x2001a:raise ValueError('section type mismatch')
 if struct.unpack_from('<3I',b,link[0])!=(13,1,14):raise ValueError('mesh-state link')
 if struct.unpack_from('<I',b,ras[0])[0]!=0x35545844 or struct.unpack_from('<HH',b,ras[0]+12)!=(128,128) or struct.unpack_from('<I',b,ras[0]+28)[0]!=16:raise ValueError('raster identity')
 blocks=b[base+data[0]:base+data[0]+data[2]]
 if len(blocks)!=21872:raise ValueError('source extent')
 old=bytearray(128*128*4);new=bytearray(len(old));ordered=0
 for j in range(1024):
  block=blocks[16*j:16*j+16];c0,c1=struct.unpack_from('<HH',block,8);ordered+=int(c0<=c1)
  corrected=decode_dxt5_block(block);legacy=decode_dxt1_block(block[8:]);bx=j%32;by=j//32
  for k,rgba in enumerate(corrected):
   at=((by*4+k//4)*128+(bx*4+k%4))*4;new[at:at+4]=bytes(rgba);old[at:at+4]=bytes((*legacy[k][:3],rgba[3]))
 changed=sum(old[i:i+4]!=new[i:i+4] for i in range(0,len(old),4))
 after=sha(a.package)
 if after!=before:raise ValueError('package endpoint changed')
 a.out.mkdir();
 for name,rgba in [('legacy.png',old),('corrected.png',new)]:
  with (a.out/name).open('xb') as f:Image.frombytes('RGBA',(128,128),bytes(rgba)).save(f,format='PNG')
 report={'scope':'source stage-zero texture decode only; legacy simulation vs corrected BC3, no runtime pose/shader/selection','package_sha256':before,'rw4_sha256':digest,'base_block_count':1024,'ordered_endpoint_blocks':ordered,'changed_pixels':changed,'legacy_rgba_sha256':hashlib.sha256(old).hexdigest(),'corrected_rgba_sha256':hashlib.sha256(new).hexdigest(),'artifacts':{n:sha(a.out/n) for n in ('legacy.png','corrected.png')},'stable_path':'operator admitted; endpoint hashes cannot rule out transient replacement'}
 with (a.out/'receipt.json').open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()

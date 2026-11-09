"""Inspect one explicit candidate Raster and source hook branch, without execution."""
import argparse,hashlib,json,struct
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from gmdl_skin_probe import indexes
from dbpf_extract import payload
from rw4_validate import validate
def sha(b):return hashlib.sha256(b).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('package',type=Path);p.add_argument('out',type=Path);a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output parent outside target')
 source=SourceImage(a.exe); before=sha(a.package.read_bytes()); rows=[r for r in indexes(a.package) if r['type']==0x2f4e681b and r['instance']==0xbd08bd8a]
 if len(rows)!=1:raise ValueError('candidate key not unique')
 b=payload(a.package,rows[0]);after=sha(a.package.read_bytes())
 if before!=after or sha(b)!='5705df1ab03e94c58c8a890383026e68c68dfe64ba488d4d767f2706d8ad8aec' or validate(b)!='ok':raise ValueError('candidate mismatch')
 u32=lambda o:struct.unpack_from('<I',b,o)[0]
 if (u32(0x24),u32(0x30),u32(0x38))!=(2,0x144,0):raise ValueError('header mismatch')
 base=struct.unpack_from('<6I',b,0x144); raster=struct.unpack_from('<6I',b,0x15c)
 if base[5]!=0x10030 or raster[5]!=0x20003 or raster[0]!=0x124 or raster[2]!=32:raise ValueError('section mismatch')
 raw=struct.unpack_from('<8I',b,0x124);fmt,flags=raw[:2];w,h=struct.unpack_from('<HH',b,0x130);mips=b[0x135];ref=u32(0x140)
 if (fmt,flags,w,h,mips,ref)!=(0x35545844,8,256,256,9,0):raise ValueError('raster mismatch')
 if int.from_bytes(source.read(0x11efbe0+7*4,4),'little')!=0x11efa82:raise ValueError('source jump table mismatch')
 sizes=[max(1,(w>>i)>>2)*max(1,(h>>i)>>2)*16 for i in range(mips)]
 if sum(sizes)!=base[2] or u32(0x44)+base[0]+base[2]>len(b):raise ValueError('mip extent mismatch')
 report={'scope':'source candidate + conditional source branch; no hook, device or game execution','source_sha256':SOURCE_SHA256,
         'package_sha256_before_after':before,'payload_sha256':sha(b),'raster_words':[f'{x:08X}' for x in raw],
         'format':'DXT5','flags':'00000008','width':w,'height':h,'mip_count':mips,'reference':ref,'compressed_level_sizes':sizes,
         'total_compressed_bytes':sum(sizes),'base_resource_size':base[2],'source_jump_target':'011EFA82',
         'assumptions':'actual object follows admitted type-8 branch; registry, hook order, loader, callbacks, source pointer and device outcomes not observed'}
 with a.out.open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print('candidate mip sizes',sizes,'report sha256',sha(a.out.read_bytes()))
if __name__=='__main__':main()

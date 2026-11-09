"""Bind the material profile's first source key to installed config spelling."""
import argparse,hashlib,json
from pathlib import Path
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from cell_schema_probe import SourceImage,SOURCE_SHA256
from cell_skybox_property_config_audit_v1 import fnv
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('config',type=Path);p.add_argument('out',type=Path);a=p.parse_args();root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 s=SourceImage(a.exe);raw=s.read(0x712110,0x90);d=Cs(CS_ARCH_X86,CS_MODE_32);files=[]
 for name in ('Properties.txt','ConfigManager.txt'):
  path=a.config/name;b=path.read_bytes();files.append({'path':str(path.resolve()),'sha256':hashlib.sha256(b).hexdigest()})
 if fnv('ShaderPath')!=0x1a34e253:raise ValueError('shader hash mismatch')
 report={'source_sha256':SOURCE_SHA256,'instruction_bytes_sha256':hashlib.sha256(raw).hexdigest(),'instructions':[f'{i.address:08X} {i.mnemonic} {i.op_str}' for i in d.disasm(raw,0x712110)],'shader_path_key':f'{fnv("ShaderPath"):08X}','second_key':'E1408308','config_files':files,'scope':'static profile selection and candidate config spelling; active property values, config execution, material selection unobserved'}
 with a.out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()

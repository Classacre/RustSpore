"""Retain bounded source instructions for screen-target storage and dimensions."""
import argparse,hashlib,json
from pathlib import Path
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from cell_schema_probe import SourceImage,SOURCE_SHA256
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('out',type=Path);a=p.parse_args();root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 s=SourceImage(a.exe);d=Cs(CS_ARCH_X86,CS_MODE_32);spans=[(0x11f8430,0x58),(0x11f7ab0,0x38),(0x11f7da7,0x6c),(0x11f7e73,0x47)]
 rows=[]
 for addr,n in spans:
  raw=s.read(addr,n);rows.append({'start':f'{addr:08X}','bytes':raw.hex(),'sha256':hashlib.sha256(raw).hexdigest(),'instructions':[f'{i.address:08X} {i.mnemonic} {i.op_str}' for i in d.disasm(raw,addr)]})
 report={'source_sha256':SOURCE_SHA256,'spans':rows,'scope':'bounded conditional static target storage, no live width/height or source caller asserted'}
 with a.out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print('wrote',a.out)
if __name__=='__main__':main()

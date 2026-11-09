"""Retain bounded source instructions for the screen-draw camera boundary."""
import argparse,hashlib,json
from pathlib import Path
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from cell_schema_probe import SourceImage,SOURCE_SHA256
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('out',type=Path);a=p.parse_args();root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 s=SourceImage(a.exe);d=Cs(CS_ARCH_X86,CS_MODE_32);spans=[(0x776e70,0x5b),(0x7c3910,0x8c),(0x7c4770,0x1da),(0x776ca9,0x4e),(0x7c4950,0xdd),(0x11f3f8b,0x22),(0x11f0ed0,6)]
 rows=[]
 for addr,n in spans:
  raw=s.read(addr,n);rows.append({'start':f'{addr:08X}','sha256':hashlib.sha256(raw).hexdigest(),'bytes':raw.hex(),'instructions':[f'{i.address:08X} {i.mnemonic} {i.op_str}' for i in d.disasm(raw,addr)]})
 report={'source_sha256':SOURCE_SHA256,'spans':rows,'scope':'bounded conditional source draw camera path, no live target, viewport, state or camera observed'}
 with a.out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print('wrote',a.out)
if __name__=='__main__':main()

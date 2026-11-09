"""Bounded static trace of dynamically populated screen mode candidates."""
import argparse,hashlib,json
from pathlib import Path
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from cell_schema_probe import SourceImage,SOURCE_SHA256
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('out',type=Path);a=p.parse_args();root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 s=SourceImage(a.exe);d=Cs(CS_ARCH_X86,CS_MODE_32);spans=[(0x7c7720,0xbf),(0x7cc620,0x3af),(0x11f8670,0x178),(0x7ca130,0x79)]
 rows=[]
 for addr,n in spans:
  raw=s.read(addr,n);rows.append({'start':f'{addr:08X}','bytes':raw.hex(),'sha256':hashlib.sha256(raw).hexdigest(),'instructions':[f'{i.address:08X} {i.mnemonic} {i.op_str}' for i in d.disasm(raw,addr)]})
 report={'source_sha256':SOURCE_SHA256,'spans':rows,'scope':'bounded static display-mode list construction; no live adapter, accepted row or selected screen size'}
 with a.out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print('wrote',a.out)
if __name__=='__main__':main()

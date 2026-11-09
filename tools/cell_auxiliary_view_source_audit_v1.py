"""Retain source scalar and vector inputs to one auxiliary Cell view path."""
import argparse,hashlib,json,struct
from pathlib import Path
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from cell_schema_probe import SourceImage,SOURCE_SHA256
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('out',type=Path);a=p.parse_args();root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 s=SourceImage(a.exe);d=Cs(CS_ARCH_X86,CS_MODE_32);spans=[(0xe54890,0xfa),(0x7c4cd0,0x7f),(0x7c4540,0x39)];rows=[]
 for addr,n in spans:
  raw=s.read(addr,n);rows.append({'start':f'{addr:08X}','bytes':raw.hex(),'sha256':hashlib.sha256(raw).hexdigest(),'instructions':[f'{i.address:08X} {i.mnemonic} {i.op_str}' for i in d.disasm(raw,addr)]})
 constants={f'{addr:08X}':{'bytes':s.read(addr,4).hex(),'f32':struct.unpack('<f',s.read(addr,4))[0]} for addr in (0x148467c,0x145cc2c,0x15a8c34,0x15a8c38,0x15a8c3c,0x15a8c40,0x15a8c44,0x15a8c48)}
 report={'source_sha256':SOURCE_SHA256,'spans':rows,'initial_pe_constants':constants,'scope':'conditional auxiliary view input, not selected primary camera or observed runtime globals'}
 with a.out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print('wrote',a.out)
if __name__=='__main__':main()

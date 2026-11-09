"""Retain source initializer and constants for screen-quad offset inputs."""
import argparse,hashlib,json,struct
from pathlib import Path
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from cell_schema_probe import SourceImage,SOURCE_SHA256
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('out',type=Path);a=p.parse_args();root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 s=SourceImage(a.exe);start=0x6dda60;raw=s.read(start,0x96);d=Cs(CS_ARCH_X86,CS_MODE_32);values={f'{addr:08X}':{'bytes':s.read(addr,4).hex(),'f32':struct.unpack('<f',s.read(addr,4))[0]} for addr in (0x1471b2c,0x13ec480)}
 if values['01471B2C']['bytes']!='0000003f' or values['013EC480']['bytes']!='000000bf':raise ValueError('constant mismatch')
 out={'source_sha256':SOURCE_SHA256,'range':'006DDA60..006DDAF5','bytes_sha256':hashlib.sha256(raw).hexdigest(),'instructions':[f'{i.address:08X} {i.mnemonic} {i.op_str}' for i in d.disasm(raw,start)],'constants':values,'scope':'conditional static initializer; flags, alternate source values, current dimensions and runtime state not observed'}
 with a.out.open('x',encoding='utf-8') as f:json.dump(out,f,indent=2);f.write('\n')
 print(json.dumps(out,indent=2))
if __name__=='__main__':main()

"""Hash-bound static watcher-provider constructor and lookup observations."""
import argparse,hashlib,json,struct
from pathlib import Path
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from cell_schema_probe import SourceImage,SOURCE_SHA256
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('out',type=Path);a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 s=SourceImage(a.exe);d=Cs(CS_ARCH_X86,CS_MODE_32)
 spans=[(0x12f1dd0,23),(0x7e88f4,19),(0x6a1750,80),(0x6a2370,37),(0x6a1fe0,103),(0x7e63e0,113),(0x7e6460,110)]
 rows=[]
 for addr,n in spans:
  raw=s.read(addr,n);ins=list(d.disasm(raw,addr));rows.append({'start':f'{addr:08X}','bytes':raw.hex(),'sha256':hashlib.sha256(raw).hexdigest(),'instructions':[f'{i.address:08X} {i.mnemonic} {i.op_str}' for i in ins]})
 table=list(struct.unpack('<19I',s.read(0x1408818,76)))
 if table[9]!=0x6a2370:raise ValueError('unexpected vtable +24')
 report={'source_sha256':SOURCE_SHA256,'table_01408818':[f'{x:08X}' for x in table],'spans':rows,'scope':'static bounded decode; provider population/order and runtime value unobserved'}
 with a.out.open('x',encoding='utf-8') as f:json.dump(report,f,indent=2);f.write('\n')
 print('wrote',a.out)
if __name__=='__main__':main()

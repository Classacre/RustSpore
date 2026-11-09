"""Static-only call/memory audit for candidate E6B560 comparison host."""
import argparse,hashlib,json,struct
from pathlib import Path
from capstone import CS_GRP_CALL,CS_GRP_JUMP
from capstone.x86 import X86_OP_IMM,X86_OP_MEM
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder
CODE=[(0xe6b560,0xe6b627),(0xe57260,0xe572d8),(0xe4ecc0,0xe4ecfb)]
CALLS={(0xe6b588,0xb71d00),(0xe6b5ae,0xe67a50),(0xe6b5b7,0xe57260),
       (0xe6b5cc,0x7b2350),(0xe6b5dd,0xe4caa0),(0xe6b5fd,0xe82160),
       (0xe6b61b,0xe82160),(0xe57281,0xe4ecc0),(0xe572a0,0xe4ecc0)}
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
 out=a.out.resolve();out.relative_to((Path(__file__).resolve().parent.parent/'target').resolve());out.mkdir(parents=True,exist_ok=False)
 img=SourceImage(a.exe);assert hashlib.sha256(a.exe.read_bytes()).hexdigest()==SOURCE_SHA256
 result=[];calls=set();absolute=set()
 for start,end in CODE:
  raw=img.read(start,end-start);rows=list(decoder().disasm(raw,start));assert sum(i.size for i in rows)==len(raw)
  boundaries={i.address for i in rows};transfers=[];mem=[];ab=[]
  for i in rows:
   if i.group(CS_GRP_CALL) or i.group(CS_GRP_JUMP):
    op=i.operands[0];assert op.type==X86_OP_IMM
    if i.group(CS_GRP_CALL):assert (i.address,op.imm) in CALLS;calls.add((i.address,op.imm))
    else:assert op.imm in boundaries,(hex(i.address),hex(op.imm))
    transfers.append([f'{i.address:08X}',i.mnemonic,f'{op.imm:08X}'])
   for op in i.operands:
    if op.type==X86_OP_MEM:
     if not op.mem.base:
      if op.mem.index:assert i.mnemonic=='lea',(hex(i.address),i.op_str)
      if op.mem.disp:absolute.add(op.mem.disp)
      ab.append([f'{i.address:08X}',i.op_str])
     else:mem.append([f'{i.address:08X}',i.op_str])
  result.append({'start':f'{start:08X}','end_exclusive':f'{end:08X}','sha256':hashlib.sha256(raw).hexdigest(),'transfers':transfers,'absolute_memory':ab,'register_relative_memory':mem,'disassembly':[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in rows]})
 assert calls==CALLS
 table=img.read(0x14846c4,0x254)
 # E4ECC0's loop reads the final threshold at 1484914 for progress 999.
 assert struct.unpack_from('<i',table,0x250)[0]==1000
 (out/'table.bin').write_bytes(table)
 report={'source_sha256':SOURCE_SHA256,'source_executed':False,'ranges':result,'absolute_addresses':[f'{x:08X}' for x in sorted(absolute)],'table_sha256':hashlib.sha256(table).hexdigest(),'table_address':'014846C4','table_length':len(table),'external_stubs':['B71D00','E67A50','7B2350','E4CAA0','E82160']}
 (out/'audit.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k] for k in ['source_executed','absolute_addresses','table_sha256','table_length']}))
if __name__=='__main__':main()

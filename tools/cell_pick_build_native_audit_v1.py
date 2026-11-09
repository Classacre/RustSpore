"""Static-only audit of candidate original spatial-tree operations."""
import argparse,hashlib,json
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder
from capstone import CS_GRP_CALL,CS_GRP_JUMP
from capstone.x86 import X86_OP_IMM,X86_OP_MEM
CODE=[(0xe868f0,0xe86956),(0xe86960,0xe86992),(0xe869a0,0xe869e5),
      (0xe869f0,0xe86ab5),(0xe86c20,0xe86dab),(0xe86db0,0xe86e30),
      (0xe86e30,0xe86ef0),(0xe86ef0,0xe86fd8),(0xe86fe0,0xe87065),
      (0xe870b0,0xe87169)]
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
 root=Path(__file__).resolve().parent.parent;out=a.out.resolve();out.relative_to((root/'target').resolve());out.mkdir(parents=True,exist_ok=False)
 image=SourceImage(a.exe);assert hashlib.sha256(a.exe.read_bytes()).hexdigest()==SOURCE_SHA256
 ranges=[]; starts={x for x,_ in CODE}; constants=set()
 for start,end in CODE:
  raw=image.read(start,end-start);rows=list(decoder().disasm(raw,start));assert sum(i.size for i in rows)==len(raw),(hex(start),hex(end))
  boundaries={i.address for i in rows}; transfers=[];mem=[];absolute=[]
  for i in rows:
   if i.group(CS_GRP_CALL) or i.group(CS_GRP_JUMP):
    op=i.operands[0];assert op.type==X86_OP_IMM,(hex(i.address),i.op_str)
    if i.group(CS_GRP_CALL):assert op.imm in starts,(hex(i.address),hex(op.imm))
    else:assert op.imm in boundaries,(hex(i.address),hex(op.imm))
    transfers.append([f'{i.address:08X}',i.mnemonic,f'{op.imm:08X}'])
   for op in i.operands:
    if op.type==X86_OP_MEM:
     rec=[f'{i.address:08X}',i.op_str]
     if not op.mem.base:
      assert not op.mem.index
      constants.add(op.mem.disp);absolute.append(rec)
     else:mem.append(rec)
  ranges.append({'start':f'{start:08X}','end_exclusive':f'{end:08X}','sha256':hashlib.sha256(raw).hexdigest(),'transfers':transfers,'absolute_memory':absolute,'register_relative_memory':mem,
                 'disassembly':[f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in rows]})
 assert constants=={0x1471b2c,0x14861d0,0x1486bd0,0x1486bd8},[hex(x) for x in constants]
 data={f'{x:08X}':image.read(x,4).hex() for x in sorted(constants)}
 result={'source_sha256':SOURCE_SHA256,'source_executed':False,'ranges':ranges,'absolute_constants':data,
         'scope':'Candidate functions only; no live actors, providers or game initialization.'}
 (out/'audit.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf8');print(json.dumps({'out':str(out),'source_executed':False,'constants':data,'ranges':len(ranges)}))
if __name__=='__main__':main()

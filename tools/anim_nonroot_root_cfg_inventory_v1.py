"""Enumerate source span control flow and memory writes; never execute source."""
import hashlib,json,sys
from pathlib import Path
R=Path(__file__).resolve().parent.parent;sys.path.insert(0,str(R/'tools'))
from cell_schema_probe import SourceImage
from native_crt_acos_probe import decoder
SRC=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe');OUT=R/'target/animation/nonroot-root-cfg-inventory-v1'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 assert sha(SRC)=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
 image=SourceImage(SRC); engine=decoder()
 from capstone import CS_GRP_CALL,CS_GRP_JUMP,CS_GRP_RET,CS_AC_WRITE
 from capstone.x86 import X86_OP_IMM,X86_OP_MEM
 spans=[(0x9b0ca0,0x9b1b82),(0x9b0b90,0x9b0c91),(0x99bd80,0x99be6b),(0x99d74f,0x99dbfe),(0x99c8d0,0x99c993),(0x5992b0,0x5992cf),(0x99cb70,0x99cc74)]
 reports=[]
 for lo,hi in spans:
  raw=image.read(lo,hi-lo);ins=list(engine.disasm(raw,lo));assert sum(i.size for i in ins)==len(raw)
  flow=[];writes=[];stack=[]
  for i in ins:
   fmt={'address':f'{i.address:08X}','bytes':i.bytes.hex(),'instruction':i.mnemonic+' '+i.op_str}
   if i.group(CS_GRP_JUMP) or i.group(CS_GRP_CALL) or i.group(CS_GRP_RET):
    target=i.operands[0].imm if i.operands and i.operands[0].type==X86_OP_IMM else None
    flow.append(dict(fmt,kind='call' if i.group(CS_GRP_CALL) else 'return' if i.group(CS_GRP_RET) else 'branch',target=None if target is None else f'{target:08X}'))
   for op in i.operands:
    if op.type==X86_OP_MEM and op.access&CS_AC_WRITE:writes.append(dict(fmt,size=op.size,base=i.reg_name(op.mem.base),index=i.reg_name(op.mem.index),scale=op.mem.scale,displacement=op.mem.disp))
   if i.mnemonic in ('push','pop','call','ret','enter','leave') or i.op_str.startswith('esp,'):stack.append(fmt)
  reports.append({'start':f'{lo:08X}','end_exclusive':f'{hi:08X}','bytes_sha256':hashlib.sha256(raw).hexdigest(),'instruction_count':len(ins),'control_flow':flow,'explicit_memory_writes':writes,'stack_operations':stack})
 report={'schema':'nonroot-root-cfg-inventory-v1','source_sha256':sha(SRC),'native_executed':False,'compiled':False,'scope':'Exhaustive linear disassembly inventory of seven source spans only; no inferred reachability, indirect aliasing, implicit writes or proof of stack balance. Calls and branches on excluded paths are retained.','limitations':['This is not a reachable CFG, symbolic execution or register provenance.','Implicit stack writes from push/call are listed as stack operations, not explicit memory operands.','Review actual capability flags, all branches, host setup and external call sites before using inventory as a proof.'],'spans':reports,'tool_sha256':sha(Path(__file__))}
 OUT.mkdir(parents=True,exist_ok=False);(OUT/'report.json').write_text(json.dumps(report,indent=2)+'\n')
 print(json.dumps({'native_executed':False,'spans':len(reports),'control_flow':sum(len(x['control_flow']) for x in reports),'explicit_writes':sum(len(x['explicit_memory_writes']) for x in reports),'stack_operations':sum(len(x['stack_operations']) for x in reports),'report_sha256':sha(OUT/'report.json')},indent=2))
if __name__=='__main__':main()

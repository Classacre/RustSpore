"""Bounded local CFG decode of metadata-absent font server vtable entries.
Not Ghidra function-body metadata and not a runtime trace. No native execution.
"""
import hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path[:0]=[str(ROOT/'tools'),str(ROOT/'target/rea/capstone-runtime')]
from cell_schema_probe import SourceImage,SOURCE_SHA256
from capstone import Cs,CS_ARCH_X86,CS_MODE_32,CS_GRP_JUMP,CS_GRP_CALL,CS_GRP_RET
from capstone.x86 import X86_OP_IMM

def main(out):
 out=Path(out);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False);image=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));cs=Cs(CS_ARCH_X86,CS_MODE_32);cs.detail=True;reports=[]
 for start,end,slot in [(0x88d010,0x88d6b0,0x142f65c),(0x88dd90,0x88de50,0x142f658)]:
  assert int.from_bytes(image.read(slot,4),'little')==start;todo=[start];ins={};escapes=[];unresolved=[];rets=[];calls=[]
  while todo:
   pc=todo.pop()
   while pc not in ins:
    if not start<=pc<end:escapes.append(f'{pc:08X}');break
    i=next(cs.disasm(image.read(pc,min(15,end-pc)),pc));ins[pc]=i
    if i.mnemonic=='int3':unresolved.append(dict(pc=f'{pc:08X}',kind='reachable_INT3'));break
    if i.group(CS_GRP_RET):rets.append(f'{pc:08X}');break
    if i.group(CS_GRP_CALL):calls.append(dict(pc=f'{pc:08X}',operand=i.op_str,target=f'{i.operands[0].imm:08X}' if i.operands[0].type==X86_OP_IMM else None))
    if i.group(CS_GRP_JUMP):
     if i.operands[0].type!=X86_OP_IMM:unresolved.append(dict(pc=f'{pc:08X}',kind='indirect_jump',operand=i.op_str));break
     target=i.operands[0].imm;todo.append(target)
     if i.mnemonic=='jmp':break
    pc+=i.size
  rows=sorted(ins.values(),key=lambda i:i.address);ranges=[]
  for i in rows:
   if ranges and ranges[-1][1]==i.address:ranges[-1][1]+=i.size
   else:ranges.append([i.address,i.address+i.size])
  raw=b''.join(image.read(a,b-a) for a,b in ranges);(out/f'{start:08X}.asm').write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in rows)+'\n')
  reports.append(dict(entry=f'{start:08X}',vtable_slot=f'{slot:08X}',bound=f'{end:08X}',ranges=[[f'{a:08X}',f'{b:08X}'] for a,b in ranges],sha256=hashlib.sha256(raw).hexdigest(),bytes=len(raw),returns=rets,calls=calls,escapes=escapes,unresolved=unresolved,local_direct_CFG_closed=not escapes and not unresolved,ghidra_function_metadata_available=False))
 report=dict(schema='font-server-local-cfg-frontier-v1',source_sha256=SOURCE_SHA256,tool_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),native_executed=False,scope='Local bounded entry CFG only; direct callee effects and runtime activeface remain open. Bounds are next observed vtable entries, not inferred Ghidra procedures.',entries=reports);(out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print([(r['entry'],r['bytes'],r['local_direct_CFG_closed'],r['escapes'],r['unresolved']) for r in reports])
if __name__=='__main__':main(sys.argv[1])

"""Source-only disassembly audit of 009E7C80 and its mode-2 helper."""
import argparse,hashlib,json,sys
from pathlib import Path
from cell_schema_probe import SourceImage, SOURCE_SHA256
from native_crt_acos_probe import decoder
ROOT=Path(__file__).resolve().parent.parent
V6=ROOT/"target/rea/20261028-source-runtime-v6"
V4=ROOT/"target/rea/20261027-source-runtime-v4"
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def dossier(p):return json.loads(json.loads(p.read_text())["content"][0]["text"])["raw_result"]["value"]
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument("--out",type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/"target").resolve());sys.path.insert(0,str(ROOT/"target/rea/capstone-runtime"))
 from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM,X86_OP_MEM
 source=SourceImage(Path("D:/Games/SPORE Collection/SporeBin/SporeApp.exe"));paths=[V4/"32-analyze_function.json",V6/"99-analyze_function.json",V6/"100-analyze_function.json",V6/"101-analyze_function.json"];records=[dossier(p) for p in paths];expected=[0x9e7c80,0x99be70,0x9e6890,0x9e8930]
 assert [int(x["procedure"]["address"],16) for x in records]==expected
 all_ins=[];summaries=[]
 for record in records[:2]:
  lines=record["assembly"];actual=[];spans=[]
  for item in record["procedure"]["body"]["ranges"]:
   start=int(item["start"],16);end=int(item["end"],16)+1;raw=source.read(start,end-start);ins=list(decoder().disasm(raw,start));assert sum(i.size for i in ins)==len(raw);actual+=ins;spans.append(dict(start=f"{start:08X}",end_exclusive=f"{end:08X}",sha256=hashlib.sha256(raw).hexdigest()))
  assert [i.address for i in actual]==[int(s.split(":")[0],16) for s in lines]
  all_ins.append(actual);summaries.append(spans)
 main,helper=all_ins
 calls=[(i.address,i.operands[0].imm) for i in main if i.group(CS_GRP_CALL) and i.operands[0].type==X86_OP_IMM]
 assert calls==[(0x9e7fa6,0x99be70),(0x9e7fdd,0x9e6890),(0x9e8249,0x99be70),(0x9e82ae,0x9e6890)]
 assert not any(i.group(CS_GRP_CALL) for i in helper)
 required={0x9e7cde,0x9e7ce8,0x9e7cf7,0x9e7d2b,0x9e7d32,0x9e7d39,0x9e7d40,0x9e7d49,0x9e7fa6}
 assert required <= {i.address for i in main}
 absolutes=set()
 for ins in (main,helper):
  for i in ins:
   for op in i.operands:
    if op.type==X86_OP_MEM and not op.mem.base and not op.mem.index:absolutes.add((op.mem.disp,op.size))
 assert absolutes=={(0x13eb1bc,4),(0x13eb8b0,4),(0x14719e4,4)},absolutes
 data={f"{address:08X}":source.read(address,n).hex() for address,n in sorted(absolutes)}
 report=dict(schema="cid-child-mode2-source-audit-v1",source_sha256=SOURCE_SHA256,source_spans=summaries,absolute_constants=data,all_calls=[dict(at=f"{at:08X}",target=f"{dst:08X}") for at,dst in calls],mode2_call="009E7FA6 -> 0099BE70",mode2_helper_leaf=True,other_mode_calls="009E6890 remains needed for modes 0/1, not admitted in this candidate",native_executed=False,scope="Static whole-function source boundary and explicit mode-2 fixture proposal only; independent linked ABI, control-flow, guard and native/Rust comparison remain required",pins={str(p):sha(p) for p in [Path(__file__),*paths]})
 with a.out.open("x") as f:json.dump(report,f,indent=2);f.write("\n")
 print(sha(a.out))
if __name__=="__main__":main()

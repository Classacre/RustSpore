"""Check retained REA instruction evidence for CID node mode provenance; no native execution."""
import argparse,hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
V6=ROOT/"target/rea/20261028-source-runtime-v6"
V4=ROOT/"target/rea/20261027-source-runtime-v4"
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):
    outer=json.loads(p.read_text());return json.loads(outer["content"][0]["text"])["raw_result"]["value"]
def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument("--out",type=Path,required=True);a=ap.parse_args();a.out.resolve().relative_to((ROOT/"target").resolve())
    paths={i:V6/f"{i:02d}-analyze_function.json" for i in (10,11,12,13,14,21,22,23,24,25,26,27,28,29)}
    records={i:read(p) for i,p in paths.items()};producer=V4/"24-analyze_function.json";consumer=V4/"32-analyze_function.json";records[100]=read(producer);records[101]=read(consumer)
    required={27:["0x9c2f05: CALL 0x009bf4a0","0x9c2f29: CALL 0x009c1810"],21:["0x9c095a: CALL 0x009be870","0x9c0960: ADD ESI, 0x2bc"],22:["0x9bf3ba: CALL 0x009be870"],23:["0x9be9c6: LEA ECX, [EBX + 0x18c]","0x9bea98: CALL 0x009b43c0"],28:["0x9b451e: MOV EDX, dword ptr [ECX + 0xac]","0x9b4524: MOV dword ptr [EAX + 0xac], EDX"],29:["0x9bf258: CALL 0x009b43c0"],100:["0x9f47f9: MOV ESI, dword ptr [EAX + 0x2e4]","0x9f47ff: ADD ESI, dword ptr [ESP + 0x54]","0x9f4ecf: AND CL, 0x7","0x9f4ef9: JNZ 0x009f5724","0x9f4f76: MOV dword ptr [ESI + 0x238], 0x2","0x9f4fac: MOV dword ptr [ESI + 0x238], 0x1","0x9f4fdc: MOV dword ptr [ESI + 0x238], 0x0"],101:["0x9e7cf7: MOV EAX, dword ptr [ESI + 0x238]"]}
    for i,lines in required.items():
        for line in lines:assert line in records[i]["assembly"],(i,line)
    for i,offset in ((10,"0x238]"),(11,"0xac]"),(12,"0x238]"),(13,"0xac]")):
        assert not any(offset in line.lower() for line in records[i]["assembly"]),(i,offset)
    report={"schema":"cid-node-mode-source-audit-v1","native_executed":False,"trace":"9C2ED0 builds stack temporary then 9C1810 uses 9C0940/9BF3A0/9BEC10 copy paths; 9BE870 and 9BF030 delegate subobject to 9B43C0, which copies mode at +AC = node+238","constructor":"traced constructor and reset do not write node+238; incoming stack bytes are not a proven numeric default","later_stores":{"009F4F76":2,"009F4FAC":1,"009F4FDC":0},"branch_qualification":"009F4EF9 bypasses these stores if node+1AC low 3 bits are nonzero","consumer":"009E7CF7 reads node+238; reader skips nodes whose low 3 bits of +1AC equal 4","limits":"retained source evidence only; no all-path initialization, actor association, or execution claim","pins":{str(p):sha(p) for p in [Path(__file__),*paths.values(),producer,consumer]}}
    with a.out.open("x") as f:json.dump(report,f,indent=2);f.write("\n")
    print(sha(a.out))
if __name__=="__main__":main()

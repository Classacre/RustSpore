"""Generate synthetic admitted E6B560 cases; never executes native code."""
import argparse,hashlib,json,struct
from pathlib import Path
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);a=p.parse_args();root=Path(__file__).resolve().parent.parent
 out=a.out.resolve();out.relative_to((root/'target').resolve());out.mkdir(parents=True,exist_ok=False)
 words=[0,1,2,3,5,10,0xfffffffe,0xffffffff,0x7fffffff,0x80000000];progress=[0,49,50,999,1000];flags=[0,1,0x80];kinds=[0,2,3,4]
 data=bytearray();count=0
 for player,candidate in [(0,2),(1,1),(1,2)]:
  for a in words:
   for b in words:
    for q in progress:
     for f0 in flags:
      for f1 in flags:
       for kind in kinds:
        data.extend(struct.pack('<8I',player,candidate,a,b,f0,f1,kind,q));count+=1
 (out/'input.bin').write_bytes(data)
 report={'cases':count,'bytes':len(data),'input_sha256':hashlib.sha256(data).hexdigest(),'source_executed':False,
         'domains':{'player_candidate_lanes':3,'size_words':words,'progress':progress,'flags':flags,'kinds':kinds}}
 (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':main()

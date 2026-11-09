"""Prepare finite synthetic source-tree operation histories without execution."""
import argparse,hashlib,json,random,struct
from pathlib import Path
def w(x):return struct.unpack('<I',struct.pack('<f',x))[0]
def step(op,i=0,x=0,y=0,z=0,r=0,h=0):return (op,i,w(x),w(y),w(z),w(r),h)
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);a=p.parse_args();root=Path(__file__).resolve().parent.parent
 out=a.out.resolve();out.relative_to((root/'target').resolve());out.mkdir(parents=True,exist_ok=False)
 data=bytearray();metadata=[];randomizer=random.Random(0xe86fe0);kinds={}
 def emit(cw,label,steps):
  assert len(steps)<=8; vals=[cw,len(steps)];
  for s in steps:vals.extend(s)
  vals.extend([0]*((8-len(steps))*7));data.extend(struct.pack('<58I',*vals));
  metadata.append(f'{len(metadata)}\t{cw:04X}\t{label}\n');kinds[label]=kinds.get(label,0)+1
 for pc in (0,0x200,0x300):
  for rounding in (0,0x400,0x800,0xc00):
   cw=0x7f|pc|rounding
   for label,steps in [
    ('empty',[]),('single',[step(0,0,1,0,0,1,101),step(1)]),
    ('pending',[step(0,0,1,0,0,1,101),step(0,1,2,0,0,1,102)]),
    ('pair',[step(0,0,1,0,0,1,101),step(0,1,2,0,0,1,102),step(1)]),
    ('triple_tie',[step(0,0,0,0,0,1,101),step(0,1,2,0,0,1,102),step(0,2,1,0,0,1,103),step(1)]),
    ('contained',[step(0,0,0,0,0,5,101),step(0,1,1,0,0,1,102),step(1)]),
    ('coincident',[step(0,0,0,0,0,1,101),step(0,1,0,0,0,1,102),step(1)]),
    ('update_outside',[step(0,0,0,0,0,1,101),step(0,1,2,0,0,1,102),step(1),step(2,0,20,0,0,1),step(1)]),
    ('update_inside',[step(0,0,0,0,0,1,101),step(0,1,2,0,0,1,102),step(1),step(2,0,0.1,0,0,1),step(1)]),
    ('remove_pending',[step(0,0,0,0,0,1,101),step(0,1,2,0,0,1,102),step(3,0),step(1)]),
    ('remove_attached',[step(0,0,0,0,0,1,101),step(0,1,2,0,0,1,102),step(1),step(3,0)]),
    ('reinsert',[step(0,0,0,0,0,1,101),step(1),step(3,0),step(0,0,4,0,0,1,102),step(1)]),
   ]:emit(cw,label,steps)
   for _ in range(150):
    active=set();steps=[]
    for j in range(8):
     choices=[1]
     if len(active)<4:choices.extend([0,0])
     if active:choices.extend([2,2,3])
     op=randomizer.choice(choices)
     if op==1:steps.append(step(1));continue
     if op==0:
      i=randomizer.choice([x for x in range(4) if x not in active]);active.add(i)
      values=[randomizer.randint(-40,40)/4 for _ in range(3)];r=randomizer.randint(0,20)/4
      steps.append(step(0,i,*values,r,100+i))
     else:
      i=randomizer.choice(sorted(active))
      if op==3:steps.append(step(3,i));active.remove(i)
      else:
       values=[randomizer.randint(-40,40)/4 for _ in range(3)];r=randomizer.randint(0,20)/4
       steps.append(step(2,i,*values,r))
    emit(cw,'seeded_history',steps)
 (out/'input.bin').write_bytes(data);(out/'cases.tsv').write_text('case\tcw\tkind\n'+''.join(metadata),encoding='ascii')
 meta={'cases':len(metadata),'bytes':len(data),'input_sha256':hashlib.sha256(data).hexdigest(),'kinds':kinds,'source_executed':False,'lanes':12,'seed':'e86fe0'}
 (out/'manifest.json').write_text(json.dumps(meta,indent=2)+'\n');print(json.dumps(meta))
if __name__=='__main__':main()

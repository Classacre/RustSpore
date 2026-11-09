"""Prepare bounded synthetic pick requests; this script never runs the host."""
import argparse
import hashlib
import json
from pathlib import Path
import random
import struct

MAX=31
def word(x): return struct.unpack('<I',struct.pack('<f',x))[0]
def node(x,y,z,r,kind,a,b=0): return (word(x),word(y),word(z),word(r),kind,a,b)
def packet(cw,capacity,origin,direction,limit,nodes):
    assert 0<=len(nodes)<=MAX and 0<=capacity<=MAX
    header=(cw,len(nodes),capacity,0,*map(word,origin),*map(word,direction),word(limit))
    words=list(header)
    for n in nodes: words.extend(n)
    words.extend([0]*((MAX-len(nodes))*7))
    data=struct.pack('<228I',*words); assert len(data)==912
    return data

def main():
    p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);a=p.parse_args()
    root=Path(__file__).resolve().parent.parent; out=a.out.resolve();out.relative_to((root/'target').resolve());out.mkdir(parents=True,exist_ok=False)
    rng=random.Random(0x5a9c40); lines=[]; data=bytearray(); kinds={}
    def emit(label,cw,capacity,origin,direction,limit,nodes):
        data.extend(packet(cw,capacity,origin,direction,limit,nodes)); kinds[label]=kinds.get(label,0)+1
        lines.append(f'{len(lines)}\t{label}\t{cw:04X}\t{capacity}\t{len(nodes)}\n')
    for pc in (0,0x200,0x300):
      for rounding in (0,0x400,0x800,0xc00):
        cw=0x7f|pc|rounding
        for x,y,r,dx,lim,label in [
            (3,0,1,1,100,'hit'),(-3,0,1,1,100,'behind'),(3,2,1,1,100,'miss'),
            (3,1,1,1,100,'tangent'),(3,0,1,1,2,'limit_equal'),(3,0,1,1,1,'limit_reject'),
            (0,0,1,0,-1,'inside_limit_bypass'),(3,0,1,0,0,'zero_direction'),
            (1,0,1,1,0,'surface'),(3,0,0,1,100,'zero_radius'),
            (0.00000011920928955078125,0,0,1,100,'near_zero'),
            (2**-149,-0.0,0,1,100,'subnormal_signed_zero'),
            (3,1.00000011920928955078125,1,1,100,'outside_tangent'),
            (3,0.999999940395355224609375,1,1,100,'inside_tangent'),
            (3,0,-1,1,3.4028234663852886e38,'negative_radius_max_limit')]:
            emit(label,cw,1,(0,0,0),(dx,0,0),lim,[node(x,y,0,r,0,17)])
        tree=[node(0,0,0,64,1,1,2),node(8,0,0,1,0,8),node(3,0,0,1,0,3)]
        for cap in (0,1,2,3,31):emit('ordered_capacity',cw,cap,(0,0,0),(1,0,0),100,tree)
        emit('empty',cw,4,(0,0,0),(1,0,0),100,[])
        for _ in range(100):
            origin=tuple(rng.randint(-10,10)/4 for _ in range(3))
            direction=tuple(rng.randint(-8,8)/4 for _ in range(3))
            leaves=[node(rng.randint(-40,40)/4,rng.randint(-40,40)/4,rng.randint(-40,40)/4,
                         rng.randint(0,20)/4,0,100+i) for i in range(4)]
            nodes=[node(0,0,0,40,1,1,2),node(0,0,0,30,1,3,4),node(0,0,0,30,1,5,6),*leaves]
            emit('random_tree',cw,rng.randrange(0,6),origin,direction,rng.randint(-4,50)/4,nodes)
    (out/'input.bin').write_bytes(data); (out/'cases.tsv').write_text('case\tkind\tcw\tcapacity\tnodes\n'+''.join(lines),encoding='ascii')
    meta={'cases':len(lines),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),
          'kinds':kinds,'native_executed':False,'seed':'5a9c40','lanes':12}
    (out/'manifest.json').write_text(json.dumps(meta,indent=2)+'\n',encoding='utf8');print(json.dumps(meta))
if __name__=='__main__':main()

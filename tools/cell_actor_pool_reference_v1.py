"""Independent synthetic checked-pool model and bounded comparison protocol.

This uses a Python dictionary plus a free-index stack, not the Rust linked list.
Checked occupancy/high-water validation and generation-boundary refusal are
defensive policies, not claims that the original game performs those checks.
No original executable, actor lifecycle or runtime thread is exercised.

With a fresh directory, from the repository root:
  python -I -S tools/cell_actor_pool_reference_v1.py prepare target/cell-actor-pool-model-repro
  tools\build.bat run -p spore-core --example cell_actor_pool_compare_v1 -- target/cell-actor-pool-model-repro/input.bin target/cell-actor-pool-model-repro/rust.bin
  python -I -S tools/cell_actor_pool_reference_v1.py verify target/cell-actor-pool-model-repro target/cell-actor-pool-model-repro/rust.bin
"""
import argparse
import hashlib
import json
from pathlib import Path
import random
import struct

ROOT=Path(__file__).resolve().parent.parent
FILES={'generator':Path(__file__),'model':ROOT/'crates/spore-core/src/cell_actor_pool_checked_v1.rs',
       'adapter':ROOT/'crates/spore-core/examples/cell_actor_pool_compare_v1.rs'}
def sha(b): return hashlib.sha256(b).hexdigest()
def pins(): return {name:sha(path.read_bytes()) for name,path in FILES.items()}

class Reference:
    def __init__(self,capacity,counter):
        self.capacity=capacity; self.counter=counter; self.entries={}; self.free=[]; self.high=0
    def step(self,op,arg):
        status=0; value=0; extra=[]
        if op==1:
            if len(self.entries)==self.capacity: status=1
            else:
                next_counter=(self.counter+1)&0xffffffff
                if next_counter&0xffff==0: status=2
                else:
                    if self.free: index=self.free.pop()
                    else: index=self.high; self.high+=1
                    value=((next_counter<<16)&0xffffffff)|index
                    self.entries[index]=(value,arg); self.counter=next_counter
        elif op in (2,3):
            index=arg&0xffff; entry=self.entries.get(index)
            if arg==0 or entry is None or entry[0]!=arg: status=3 if op==2 else 4
            else:
                value=entry[1]
                if op==2: del self.entries[index]; self.free.append(index)
        elif op==4:
            for i in sorted(self.entries): extra.extend(self.entries[i])
        elif op==5:
            extra=[self.entries[i][1] for i in sorted(self.entries)]
            self.entries.clear(); self.free.clear(); self.high=0
        else: raise ValueError('invalid operation')
        return [status,value,self.counter,len(self.entries),self.high,len(extra),*extra]

def prepare(out):
    cases=[]
    def add(capacity,seed,operations):
        assert 1<=capacity<=16 and len(operations)<=100
        cases.append((capacity,seed,operations))
    # Fill, free out of order, LIFO reuse, stale/high-water handle and reset.
    add(3,0,[(1,10),(1,20),(1,30),(1,40),(2,0x10000),(2,0x30002),
             (1,40),(1,50),(3,0x10000),(2,0x10000),(3,0x60003),
             (4,0),(5,0),(3,0x40002),(1,60),(4,0)])
    # Allocation reaches generation FFFF, then refuses rollover with a free slot.
    add(3,0xfffd,[(1,10),(1,20),(2,0xfffe0000),(1,30),(4,0),(5,0),(1,40)])
    # Full is reported before generation-boundary when both apply.
    add(1,0xfffe,[(1,7),(1,8),(2,0xffff0000),(1,9),(4,0)])
    sanity=Reference(3,0)
    assert [sanity.step(1,v)[1] for v in (10,20,30)]==[0x10000,0x20001,0x30002]
    assert sanity.step(2,0x10000)[1]==10 and sanity.step(2,0x30002)[1]==30
    assert sanity.step(1,40)[1]==0x40002 and sanity.step(1,50)[1]==0x50000
    assert sanity.step(3,0x10000)[0]==4 and sanity.step(2,0x10000)[0]==3
    assert sanity.step(3,0x60003)[0]==4
    boundary=Reference(1,0xfffe)
    assert boundary.step(1,7)[1]==0xffff0000 and boundary.step(1,8)[0]==1
    assert boundary.step(2,0xffff0000)[1]==7 and boundary.step(1,9)[0]==2
    rng=random.Random(0x41504331)
    for case in range(128):
        capacity=rng.randrange(1,9); seed=rng.choice([0,0x1234,0xfff8,0xfffe])
        pool=Reference(capacity,seed); operations=[]; historical=[]
        for j in range(64):
            op=rng.choices([1,2,3,4,5],[5,3,4,2,1])[0]
            live=[v[0] for _,v in sorted(pool.entries.items())]
            if op==1: arg=rng.getrandbits(32)
            elif op in (2,3):
                candidates=live+historical+[0,((pool.counter&0xffff)<<16)|pool.high,0xffffffff]
                arg=rng.choice(candidates)
            else: arg=0
            result=pool.step(op,arg)
            if op==1 and result[0]==0: historical.append(result[1])
            operations.append((op,arg))
        add(capacity,seed,operations)
    request=bytearray(b'CAPIN001'); expected=bytearray(b'CAPOUT01'); operations_total=0
    for capacity,seed,operations in cases:
        request.extend(struct.pack('<III',capacity,seed,len(operations)))
        pool=Reference(capacity,seed)
        for op,arg in operations:
            request.extend(struct.pack('<II',op,arg))
            row=pool.step(op,arg); expected.extend(struct.pack('<%dI'%len(row),*row))
            operations_total+=1
    request.extend(struct.pack('<III',0,0,0))
    out.resolve().relative_to((ROOT/'target').resolve()); out.mkdir(parents=True,exist_ok=False)
    (out/'input.bin').write_bytes(request); (out/'expected.bin').write_bytes(expected)
    report={'schema':'cell-actor-pool-model-comparison-v1','cases':len(cases),'operations':operations_total,
            'seed':'0x41504331','input_sha256':sha(request),'expected_sha256':sha(expected),'pins':pins(),
            'domain':'synthetic sequential capacity 1..8, bounded handles and values; no mutation during iteration',
            'policies':'checked occupancy and strict high-water access; conservative generation rollover refusal; retained counter on reset',
            'source_execution':False,'real_actor_state':False}
    (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))

def verify(directory,reply):
    m=json.loads((directory/'manifest.json').read_text(encoding='utf-8'))
    if m['pins']!=pins(): raise ValueError('source pins changed')
    request=(directory/'input.bin').read_bytes(); expected=(directory/'expected.bin').read_bytes(); actual=reply.read_bytes()
    if sha(request)!=m['input_sha256'] or sha(expected)!=m['expected_sha256']: raise ValueError('corpus hash mismatch')
    if expected!=actual: raise ValueError(f'reply mismatch or truncation: expected {sha(expected)} actual {sha(actual)}')
    print(json.dumps({'cases':m['cases'],'operations':m['operations'],'reply_sha256':sha(actual),
                      'reply_provenance_authenticated':False,'source_execution':False}))

def main():
    p=argparse.ArgumentParser(description=__doc__); s=p.add_subparsers(dest='action',required=True)
    a=s.add_parser('prepare'); a.add_argument('directory',type=Path)
    a=s.add_parser('verify'); a.add_argument('directory',type=Path); a.add_argument('reply',type=Path)
    args=p.parse_args()
    if args.action=='prepare': prepare(args.directory)
    else: verify(args.directory,args.reply)

if __name__=='__main__': main()

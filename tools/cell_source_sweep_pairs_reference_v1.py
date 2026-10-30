"""Synthetic finite-normal-or-zero sweep cases and independent pair-order model.

No original executable, actor pool, or live Cell state is executed or observed.
The reference constructs endpoint ranks and models the admitted active-list
projection; input identity and pair capacities are explicitly synthetic.

From the repository root, with a fresh output directory:
  python -I -S tools/cell_source_sweep_pairs_reference_v1.py prepare target/cell-sweep-model-repro
  tools\build.bat run -p spore-core --example cell_source_sweep_pairs_compare_v1 -- target/cell-sweep-model-repro/input.bin target/cell-sweep-model-repro/rust.bin
  python -I -S tools/cell_source_sweep_pairs_reference_v1.py verify target/cell-sweep-model-repro target/cell-sweep-model-repro/rust.bin
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import random
import struct

ROOT = Path(__file__).resolve().parent.parent
FILES = {'generator': Path(__file__), 'model': ROOT / 'crates/spore-core/src/cell_source_sweep_pairs_v1.rs',
         'adapter': ROOT / 'crates/spore-core/examples/cell_source_sweep_pairs_compare_v1.rs'}
SLOT = struct.Struct('<iI4f') # ID, tag, x-min, x-max, y-min, y-max

def sha(data): return hashlib.sha256(data).hexdigest()
def pins(): return {name:sha(path.read_bytes()) for name,path in FILES.items()}

def reference(slots, capacity):
    # Independent axis-1 rank test; source BBA9C0 compares endpoint ranks.
    ordered_y = sorted((v, i, end) for i, slot in enumerate(slots)
                       for end, v in enumerate((slot[4], slot[5])))
    rank = {(i,end): j for j, (_,i,end) in enumerate(ordered_y)}
    events = sorted((v, i, end) for i, slot in enumerate(slots)
                    for end, v in enumerate((slot[2], slot[3])))
    active, pairs, suppressed = [], [], 0
    for _, current, end in events:
        if end == 0:
            for prior in active:
                left, right = slots[current], slots[prior]
                if left[1] != 0 and left[1] == right[1]: continue
                if rank[current,1] < rank[prior,0] or rank[prior,1] < rank[current,0]: continue
                if len(pairs) == capacity:
                    suppressed += 1
                else:
                    pairs.append(tuple(sorted((left[0], right[0]))))
            active.append(current)
        else:
            position = active.index(current)
            active[position] = active[-1]
            active.pop()
    assert not active
    return pairs, suppressed

def encode_reply(pairs, suppressed):
    out = bytearray(struct.pack('<II',len(pairs),suppressed))
    for x,y in pairs: out.extend(struct.pack('<ii',x,y))
    for x,y in reversed(pairs): out.extend(struct.pack('<ii',x,y))
    return out

def random_case(rng):
    count = rng.randrange(0, 17)
    xs = list(range(1,2*count+1)); ys = list(range(101,101+2*count))
    rng.shuffle(xs); rng.shuffle(ys)
    ids = rng.sample(range(-200,201), count)
    slots = []
    for i in range(count):
        x0,x1 = sorted(xs[2*i:2*i+2]); y0,y1 = sorted(ys[2*i:2*i+2])
        slots.append((ids[i],rng.choice([0,0,1,2,0xffffffff]),float(x0),float(x1),float(y0),float(y1)))
    rng.shuffle(slots)
    return slots, rng.choice([0,1,2,4,16,128])

def validate(slots, capacity):
    if len(slots)>64 or not 0<=capacity<=128: raise ValueError('fixture bounds')
    for slot in slots:
        if not all(math.isfinite(v) and (v == 0.0 or abs(v)>=2**-126) for v in slot[2:]): raise ValueError('subnormal or nonfinite endpoint')
        if not slot[2]<slot[3] or not slot[4]<slot[5]: raise ValueError('invalid interval')
    for axis in (2,4):
        values = [s[i] for s in slots for i in (axis,axis+1)]
        if len(values)!=len(set(values)): raise ValueError('endpoint tie')

def prepare(out):
    # After slot 5 closes, swap-last changes the order in which slot 2 pairs.
    base=[(5,0,10.,13.,10.,30.),(-1,0,11.,20.,11.,31.),
          (3,0,12.,19.,12.,32.),(2,0,14.,18.,13.,33.)]
    tagged=[(i,7 if i in (5,-1) else t,x0,x1,y0,y1) for i,t,x0,x1,y0,y1 in base]
    cases=[(base,10),(base,2),(base,0),(tagged,10),([],0)]
    assert reference(base,10)[0]==[(-1,5),(3,5),(-1,3),(2,3),(-1,2)]
    assert reference(base,2)==([(-1,5),(3,5)],3)
    assert reference(tagged,10)[0]==[(3,5),(-1,3),(2,3),(-1,2)]
    # Translation preserves ordering while including unique zero and negative
    # endpoints. Do not use -0 or equal endpoints in this corpus.
    signed=[(i,t,x0-10.,x1-10.,y0-20.,y1-20.) for i,t,x0,x1,y0,y1 in base]
    assert reference(signed,10)==reference(base,10)
    duplicate=[(5 if i==-1 else i,t,x0,x1,y0,y1) for i,t,x0,x1,y0,y1 in base]
    assert (5,5) in reference(duplicate,10)[0]
    # 64 simultaneously active intervals yield 2016 pairs; retain the first
    # 128 and account for the remaining 1888 without claiming native safety.
    crowded=[(i,0,float(i-100),float(100+i),float(200+i),float(400+i)) for i in range(64)]
    assert len(reference(crowded,128)[0])==128 and reference(crowded,128)[1]==1888
    cases.extend([(signed,10),(duplicate,10),(crowded,128)])
    rng=random.Random(0x53575031)
    cases.extend(random_case(rng) for _ in range(256))
    request, expected=bytearray(b'SWPIN001'),bytearray(b'SWPOUT01')
    for slots, capacity in cases:
        validate(slots,capacity)
        request.extend(struct.pack('<II',len(slots),capacity))
        for slot in slots: request.extend(SLOT.pack(*slot))
        pairs, suppressed=reference(slots,capacity)
        expected.extend(encode_reply(pairs,suppressed))
    request.extend(struct.pack('<II',0xffffffff,0))
    out.resolve().relative_to((ROOT/'target').resolve())
    out.mkdir(parents=True,exist_ok=False)
    (out/'input.bin').write_bytes(request); (out/'expected.bin').write_bytes(expected)
    report={'schema':'cell-sweep-model-comparison-v1','cases':len(cases),'seed':'0x53575031',
            'input_sha256':sha(request),'expected_sha256':sha(expected),'pins':pins(),
            'coverage':'explicit swap-last, signed/duplicate IDs, equal nonzero/zero tags, 64 active slots, full and zero capacities, reverse order; 256 synthetic generated cases',
            'domain':'unique finite normal-or-zero endpoints, strict intervals, <=64 slots and <=128 pairs',
            'source_execution':False,'real_actor_state':False}
    (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))

def verify(directory, reply):
    manifest=json.loads((directory/'manifest.json').read_text(encoding='utf-8'))
    if manifest['pins']!=pins(): raise ValueError('source pins changed')
    request=(directory/'input.bin').read_bytes(); expected=(directory/'expected.bin').read_bytes(); actual=reply.read_bytes()
    if sha(request)!=manifest['input_sha256'] or sha(expected)!=manifest['expected_sha256']: raise ValueError('corpus hash mismatch')
    if actual!=expected: raise ValueError(f'reply mismatch or truncation: expected {sha(expected)}, actual {sha(actual)}')
    print(json.dumps({'cases':manifest['cases'],'reply_sha256':sha(actual),'mismatches':0,
                      'reply_provenance_authenticated':False,'source_execution':False}))

def main():
    p=argparse.ArgumentParser(description=__doc__); s=p.add_subparsers(dest='action',required=True)
    a=s.add_parser('prepare'); a.add_argument('directory',type=Path)
    a=s.add_parser('verify'); a.add_argument('directory',type=Path); a.add_argument('reply',type=Path)
    args=p.parse_args()
    if args.action=='prepare': prepare(args.directory)
    else: verify(args.directory,args.reply)

if __name__=='__main__': main()

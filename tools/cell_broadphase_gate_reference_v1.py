"""Exact-rational binary32 oracle for synthetic bounded E7F327 predicate.

No native host, runtime actors or original instructions are executed. Finite
errors are checked Rust policy, not native exception or unordered behavior.
Assumes MXCSR 1F80, nearest-even, gradual underflow and masked exceptions.

After production registration:
  python -I -S tools/cell_broadphase_gate_reference_v1.py prepare target/cell-broadphase-gate-model-v1
  tools\build.bat run -p spore-core --example cell_broadphase_gate_compare_v1 -- target/cell-broadphase-gate-model-v1/input.bin target/cell-broadphase-gate-model-v1/rust.bin
  python -I -S tools/cell_broadphase_gate_reference_v1.py verify target/cell-broadphase-gate-model-v1 target/cell-broadphase-gate-model-v1/rust.bin
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import random
import struct
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'tools'))
import cell_start_quaternion_matrix_reference_v1 as f

FILES = {'reference': Path(__file__),
         'binary32': ROOT / 'tools/cell_start_quaternion_matrix_reference_v1.py',
         'model': ROOT / 'crates/spore-core/src/cell_broadphase_gate_v1.rs',
         'geometry_types': ROOT / 'crates/spore-core/src/cell_actor_geometry_projection_v1.rs',
         'sse': ROOT / 'crates/spore-core/src/native_sse.rs',
         'adapter': ROOT / 'crates/spore-core/examples/cell_broadphase_gate_compare_v1.rs'}
def sha(data): return hashlib.sha256(data).hexdigest()
def pins(): return {name: sha(path.read_bytes()) for name, path in FILES.items()}
def b(value): return struct.unpack('<I', struct.pack('<f', value))[0]
def row(x=0, y=0, z=0, r=0, bx=0, by=0, bz=0, br=0):
    return [x, y, z, r, bx, by, bz, br]

def reference(v):
    if not all(map(f.finite, v)): return (1, [0]*6)
    try:
        dz = f.sub(v[2], v[6]); dy = f.sub(v[1], v[5]); dx = f.sub(v[0], v[4])
        total = f.add(v[3], v[7])
        z2 = f.mul(dz, dz); y2 = f.mul(dy, dy); x2 = f.mul(dx, dx)
        distance = f.add(f.add(z2, y2), x2); sum2 = f.mul(total, total)
        return (0, [dx, dy, dz, distance, sum2, int(f.val(sum2) > f.val(distance))])
    except OverflowError: return (2, [0]*6)

def prepare(out):
    cases = [row(b(1.), 0, 0, b(.5), br=b(.5)),
             row(b(1.), 0, 0, b(.5), br=b(.5)+1),
             row(b(1.), 0, 0, b(.5), br=b(.5)+2),
             row(b(1.), b(1.), b(4096.), b(4096.)),
             row(0x80000000, 0x80000000, 0, b(1.), bx=0, by=0, bz=0x80000000, br=b(-1.)),
             row(1, 0x80000001, 0x007fffff, 1),
             row(0x00800000, 0, 0, 0x00800000),
             row(0x5f7fffff), row(0x5f800000),
             row(0x7f7fffff, bx=0xff7fffff),
             row(r=0x7f7fffff, br=0x7f7fffff),
             row(x=0x7fc00000), row(br=0x7f800000),
             row(b(1.), 0, 0, b(-1.)),
             row(b(1.), b(2.), b(3.), b(4.), b(-1.), b(1.), b(2.), b(-2.))]
    assert [reference(cases[i])[1][-1] for i in (0,1,2)] == [0,0,1]
    w = reference(cases[3]); assert w == (0,[b(1.),b(1.),b(4096.),0x4b800000,0x4b800000,0])
    assert f.add(f.mul(b(4096.),b(4096.)), f.add(b(1.),b(1.))) == 0x4b800001
    assert reference(cases[4])[1][:3] == [0x80000000,0x80000000,0]
    assert reference(cases[5])[1][:3] == [1,0x80000001,0x007fffff]
    assert reference(cases[7])[0] == 0 and reference(cases[8])[0] == 2
    assert [reference(cases[i])[0] for i in (9,10,11,12)] == [2,2,1,1]
    rng = random.Random(0x47374633)
    for _ in range(512):
        def number(): return (rng.getrandbits(1)<<31) | (rng.randrange(112,142)<<23) | rng.getrandbits(23)
        cases.append([number() for _ in range(8)])
    request = bytearray(b'GATEIN01' + struct.pack('<I',len(cases)))
    expected = bytearray(b'GATEOUT1' + struct.pack('<I',len(cases)))
    statuses = Counter()
    for values in cases:
        request.extend(struct.pack('<8I', *values))
        status, result = reference(values); statuses[status] += 1
        expected.extend(struct.pack('<7I', status, *result))
    source_pins = pins()  # fail before creating files if registration is missing
    out.resolve().relative_to((ROOT/'target').resolve())
    out.mkdir(parents=True,exist_ok=False)
    (out/'input.bin').write_bytes(request); (out/'expected.bin').write_bytes(expected)
    manifest = {'schema':'cell-broadphase-gate-rational-v1','cases':len(cases),'statuses':dict(statuses),
                'input_sha256':sha(request),'expected_sha256':sha(expected),'pins':source_pins,
                'witnesses':{'strict_threshold':[0,1,2],'nonassociative_order':3,'signed_zero':4,
                             'subnormals':[5,6],'adjacent_square_overflow':[7,8],
                             'subtraction_sum_overflow':[9,10],'nonfinite':[11,12],'negative_scalar':13},
                'scope':'synthetic supplied geometry; finite checked policy; MXCSR1F80; no actor/contact/native execution',
                'original_execution':False,'runtime_identity_observed':False}
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(manifest,indent=2))

def verify(directory,reply):
    m = json.loads((directory/'manifest.json').read_text(encoding='utf-8'))
    if m['pins'] != pins(): raise ValueError('source pins changed')
    request=(directory/'input.bin').read_bytes(); expected=(directory/'expected.bin').read_bytes(); actual=reply.read_bytes()
    if sha(request)!=m['input_sha256'] or sha(expected)!=m['expected_sha256']: raise ValueError('fixture hash mismatch')
    if expected!=actual: raise ValueError(f'reply mismatch expected={sha(expected)} actual={sha(actual)}')
    print(json.dumps({'cases':m['cases'],'reply_sha256':sha(actual),'reply_provenance_authenticated':False,'original_execution':False}))

def main():
    p=argparse.ArgumentParser(description=__doc__); s=p.add_subparsers(dest='action',required=True)
    a=s.add_parser('prepare'); a.add_argument('directory',type=Path)
    a=s.add_parser('verify'); a.add_argument('directory',type=Path); a.add_argument('reply',type=Path)
    args=p.parse_args()
    if args.action=='prepare': prepare(args.directory)
    else: verify(args.directory,args.reply)
if __name__=='__main__': main()

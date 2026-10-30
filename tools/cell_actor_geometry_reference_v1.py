"""Synthetic E6D7F0 geometry reference using exact binary32 arithmetic.

Requires explicit cache snapshots, MXCSR1F80 and the checked model's admitted
domain. No actor, cache lookup, provider, original code or runtime is executed.
The reusable Fraction arithmetic in cell_start_quaternion_matrix_reference_v1
is independent of the Rust implementation and is separately hash-pinned.

  python -I -S tools/cell_actor_geometry_reference_v1.py prepare target/cell-actor-geometry-repro
  tools\build.bat run -p spore-core --example cell_actor_geometry_compare_v1 -- target/cell-actor-geometry-repro/input.bin target/cell-actor-geometry-repro/rust.bin
  python -I -S tools/cell_actor_geometry_reference_v1.py verify target/cell-actor-geometry-repro target/cell-actor-geometry-repro/rust.bin
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import random
import struct
import sys

ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
import cell_start_quaternion_matrix_reference_v1 as f

FILES={'reference':Path(__file__),'binary32':ROOT/'tools/cell_start_quaternion_matrix_reference_v1.py',
       'model':ROOT/'crates/spore-core/src/cell_actor_geometry_projection_v1.rs',
       'sse':ROOT/'crates/spore-core/src/native_sse.rs',
       'adapter':ROOT/'crates/spore-core/examples/cell_actor_geometry_compare_v1.rs'}
def sha(b):return hashlib.sha256(b).hexdigest()
def pins():return {k:sha(p.read_bytes()) for k,p in FILES.items()}
def b(v):return struct.unpack('<I',struct.pack('<f',v))[0]

def reference(row):
    fc,px,py,pz,scale,flags,*tail=row
    matrix=tail[:9]; present,key,cx,cy,cz,cs=tail[9:]
    if not all(map(f.finite,[px,py,pz,scale])): return (3,[0]*4)
    if fc==0:return (0,[px,py,pz,scale])
    if not present:return (1,[0]*4)
    if key!=fc:return (2,[0]*4)
    if not all(map(f.finite,[cx,cy,cz,cs])):return (3,[0]*4)
    rotated=bool(flags&2)
    if rotated and not all(map(f.finite,matrix)):return (3,[0]*4)
    try:
        scalar=f.mul(cs,scale)
        if rotated:
            m=matrix
            a=f.mul(m[6],cz); c=f.mul(m[3],cy); sx=f.add(c,a)
            c=f.mul(m[4],cy); d=f.mul(cx,m[0]); x=f.add(sx,d)
            d=f.mul(m[1],cx); sy=f.add(d,c); d=f.mul(m[7],cz); y=f.add(sy,d)
            c=f.mul(m[2],cx); d=f.mul(m[5],cy); sz=f.add(c,d)
            d=f.mul(m[8],cz); z=f.add(sz,d)
        else:x,y,z=cx,cy,cz
        sy=f.mul(y,scale); sz=f.mul(scale,z); sx=f.mul(scale,x)
        y=f.add(py,sy); z=f.add(pz,sz); x=f.add(px,sx)
        return (0,[x,y,z,scalar])
    except OverflowError:return (4,[0]*4)

def row(fc=7,position=None,scale=None,flags=2,matrix=None,present=1,key=7,center=None,cs=None):
    return [fc,*(position if position is not None else list(map(b,[10.,20.,30.]))),
            b(2.) if scale is None else scale,flags,
            *(matrix if matrix is not None else list(map(b,[1.,0.,0.,0.,1.,0.,0.,0.,1.]))),
            present,key,*(center if center is not None else list(map(b,[1.,2.,3.]))),b(2.) if cs is None else cs]

def prepare(out):
    cases=[row(fc=0,position=[0x80000000,1,b(3.)],scale=0x80000000,matrix=[0x7fc00000]*9,present=0),
           row(flags=0,matrix=[0x7fc00000]*9),
           row(matrix=list(map(b,[1.,2.,3.,4.,5.,6.,7.,8.,9.]))),
           row(position=[0,0,0],scale=b(1.),matrix=[b(1.),0,0,b(1e20),0,0,b(-1e20),0,0],center=[b(1.)]*3,cs=b(1.)),
           row(position=[0,0,0],scale=b(1.),flags=0,center=[1,0x80000001,0x80000000],cs=b(1.)),
           row(present=0), row(key=8), row(position=[0x7fc00000,0,0]),
           row(scale=0x7f7fffff), row(matrix=[0x7fc00000]*9),
           row(fc=0,present=1,key=99,center=[0x7fc00000]*3,cs=0x7f800000),
           row(flags=0,position=[0,0,0],scale=b(0.5),center=[1,3,0x80000001],cs=b(1.)),
           # Rotated identity: all terms and translations preserve -0.
           row(position=[0x80000000]*3,scale=b(1.),center=[0x80000000]*3,cs=b(1.)),
           row(position=[0,0,0],scale=b(1.),center=[1,0x80000001,3],cs=b(1.)),
           row(position=[0,0,0],scale=b(0.5),center=[1,0,0],cs=b(1.)),
           row(position=[0,0,0],scale=b(0.5),center=[3,0,0],cs=b(1.)),
           row(position=[0,0,0],scale=b(1.),matrix=[0x7f7fffff,0,0,0,0,0,0,0,0],center=[b(1.),0,0],cs=b(1.)),
           row(position=[0,0,0],scale=b(1.),matrix=[0x7f7fffff,0,0,0,0,0,0,0,0],center=[0x3f800001,0,0],cs=b(1.))]
    assert reference(cases[0])==(0,[0x80000000,1,b(3.),0x80000000])
    assert reference(cases[2])==(0,list(map(b,[70.,92.,114.,4.])))
    assert reference(cases[3])[1][0]==b(1.)
    assert [reference(cases[i])[0] for i in (5,6,7,8,9)]==[1,2,3,4,3]
    assert reference(cases[12])==(0,[0x80000000,0x80000000,0x80000000,b(1.)])
    assert reference(cases[13])==(0,[1,0x80000001,3,b(1.)])
    assert reference(cases[14])[1][0]==0 and reference(cases[15])[1][0]==2
    assert reference(cases[16])==(0,[0x7f7fffff,0,0,b(1.)]) and reference(cases[17])[0]==4
    rng=random.Random(0x47454F4D)
    for i in range(512):
        def number():return (rng.getrandbits(1)<<31)|(rng.randrange(120,132)<<23)|rng.getrandbits(23)
        cases.append(row(fc=7,position=[number() for _ in range(3)],scale=number(),flags=2 if i%2 else 0,
                         matrix=[number() for _ in range(9)],center=[number() for _ in range(3)],cs=number()))
    request=bytearray(b'GEOMIN01'+struct.pack('<I',len(cases)))
    expected=bytearray(b'GEOMOUT1'+struct.pack('<I',len(cases)))
    statuses=Counter()
    for fields in cases:
        assert len(fields)==21 and fields[5]<=255 and fields[15] in (0,1)
        request.extend(struct.pack('<21I',*fields)); status,values=reference(fields);statuses[status]+=1
        expected.extend(struct.pack('<5I',status,*values))
    out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(parents=True,exist_ok=False)
    (out/'input.bin').write_bytes(request);(out/'expected.bin').write_bytes(expected)
    report={'schema':'cell-actor-geometry-rational-v1','cases':len(cases),'statuses':dict(statuses),
            'input_sha256':sha(request),'expected_sha256':sha(expected),'pins':pins(),
            'witnesses':{'zero_fc':0,'nonrotated_unused_nan':1,'matrix_order':2,'cancellation':3,
                         'signed_zero_subnormal':4,'missing':5,'wrong_key':6,'nonfinite':7,
                         'overflow':8,'rotated_nonfinite_matrix':9,'zero_fc_ignores_cache':10,'half_subnormal':11,
                         'rotated_signed_zero':12,'rotated_subnormal':13,'rotated_half_subnormal':[14,15],
                         'rotated_finite_overflow_boundary':[16,17]},
            'scope':'synthetic actor/cache snapshots, ordered binary32 operations under MXCSR1F80; checked errors are policies',
            'original_execution':False,'runtime_cache_or_actor_observed':False}
    (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(report,indent=2))

def verify(directory,reply):
    m=json.loads((directory/'manifest.json').read_text(encoding='utf-8'))
    if m['pins']!=pins():raise ValueError('source pins changed')
    request=(directory/'input.bin').read_bytes(); expected=(directory/'expected.bin').read_bytes(); actual=reply.read_bytes()
    if sha(request)!=m['input_sha256'] or sha(expected)!=m['expected_sha256']:raise ValueError('fixture hash mismatch')
    if expected!=actual:raise ValueError(f'reply mismatch expected={sha(expected)} actual={sha(actual)}')
    print(json.dumps({'cases':m['cases'],'reply_sha256':sha(actual),'reply_provenance_authenticated':False,'original_execution':False}))

def main():
    p=argparse.ArgumentParser(description=__doc__);s=p.add_subparsers(dest='action',required=True)
    a=s.add_parser('prepare');a.add_argument('directory',type=Path)
    a=s.add_parser('verify');a.add_argument('directory',type=Path);a.add_argument('reply',type=Path)
    a=p.parse_args()
    if a.action=='prepare':prepare(a.directory)
    else:verify(a.directory,a.reply)
if __name__=='__main__':main()

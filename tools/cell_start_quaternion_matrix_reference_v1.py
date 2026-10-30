"""Independent exact-rational binary32 oracle for bounded 59C000 projection.

The input/output protocol is a model comparison, not source-byte execution.
It assumes nearest-even, gradual underflow, DAZ/FTZ off and masked exceptions
(MXCSR 1F80). Exception flags, actual source FP state, pointer aliasing,
normalization, actor lifetime and runtime gameplay are excluded.

  python -I -S tools/cell_start_quaternion_matrix_reference_v1.py prepare target/cell-quaternion-matrix-repro
  tools\build.bat run -p spore-core --example cell_start_quaternion_matrix_compare_v1 -- target/cell-quaternion-matrix-repro/input.bin target/cell-quaternion-matrix-repro/rust.bin
  python -I -S tools/cell_start_quaternion_matrix_reference_v1.py verify target/cell-quaternion-matrix-repro target/cell-quaternion-matrix-repro/rust.bin
"""
import argparse
from collections import Counter
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import random
import struct

ROOT=Path(__file__).resolve().parent.parent
FILES={'oracle':Path(__file__),'model':ROOT/'crates/spore-core/src/cell_start_quaternion_matrix_v1.rs',
       'sse':ROOT/'crates/spore-core/src/native_sse.rs',
       'adapter':ROOT/'crates/spore-core/examples/cell_start_quaternion_matrix_compare_v1.rs'}
def sha(b): return hashlib.sha256(b).hexdigest()
def pins(): return {name:sha(p.read_bytes()) for name,p in FILES.items()}
def p2(n): return Fraction(1<<n) if n>=0 else Fraction(1,1<<-n)
def finite(b): return (b>>23)&255 != 255
def val(b):
    e=(b>>23)&255; m=b&0x7fffff
    n=Fraction(m if e==0 else m|0x800000)*p2(-149 if e==0 else e-150)
    return -n if b>>31 else n
def round_nearest_even(x):
    q,r=divmod(x.numerator,x.denominator)
    return q+int(2*r>x.denominator or (2*r==x.denominator and q&1))
def rounded(x,negative_zero=False):
    if not x: return 0x80000000 if negative_zero else 0
    negative=x<0; v=abs(x)
    exponent=v.numerator.bit_length()-v.denominator.bit_length()
    if v<p2(exponent): exponent-=1
    quantum=p2(max(exponent-23,-149))
    q=round_nearest_even(v/quantum); y=q*quantum
    if not y: return 0x80000000 if negative else 0
    if y>=p2(128): raise OverflowError('nonfinite intermediate')
    e=y.numerator.bit_length()-y.denominator.bit_length()
    if y<p2(e): e-=1
    magnitude=int(y/p2(-149)) if e < -126 else ((e+127)<<23)|(int(y/p2(e-23))&0x7fffff)
    return magnitude | (0x80000000 if negative else 0)
def mul(a,b):
    negative=bool((a^b)&0x80000000)
    return rounded(val(a)*val(b),negative_zero=negative)
def add(a,b):
    av,bv=val(a),val(b); result=av+bv
    # Round-to-nearest exact cancellation is +0; -0 + -0 is -0.
    negative_zero=not av and not bv and bool(a&b&0x80000000)
    return rounded(result,negative_zero=negative_zero)
def sub(a,b): return add(a,b^0x80000000)

def matrix(q):
    if not all(map(finite,q)): return (1,[0]*9)
    x,y,z,w=q; one,two=0x3f800000,0x40000000
    try:
        zz=mul(z,z); zx=mul(z,x); wx=mul(w,x); yx=mul(y,x); xx=mul(x,x)
        zy=mul(z,y); yy=mul(y,y); wz=mul(w,z); wy=mul(w,y)
        m0=sub(one,mul(add(zz,yy),two)); m1=mul(add(wz,yx),two)
        d2=sub(zx,wy); d6=add(wy,zx); m2=mul(d2,two)
        m3=mul(sub(yx,wz),two); m4=sub(one,mul(add(zz,xx),two))
        d5=add(wx,zy); d8=add(yy,xx); d7=sub(zy,wx); d8=mul(d8,two)
        m5=mul(d5,two); m6=mul(d6,two); m7=mul(d7,two); m8=sub(one,d8)
        return (0,[m0,m1,m2,m3,m4,m5,m6,m7,m8])
    except OverflowError: return (2,[0]*9)

def prepare(out):
    cases=[[0,0,0,0x3f800000],[0x80000000,0,0,0x3f800000],
           [1,0,0,0x3f800000],[0x80000001,0,0,0x3f800000],
           [0x3f000001,0x3eaaaaab,0x3f3504f3,0x3f1e3779],
           [0x7f800000,0,0,0],[0x7fc00001,0,0,0],[0x7f7fffff,0,0,0],
           [0x00800000,0x00800001,0x80000001,0xbf800000],
           [0x80000000,0,0,0],                # signed-zero subtraction
           [0x3f800000]*4,                    # exact cancellation
           [0x3f800001,0,0,0x3fc00000],      # product at binary32 midpoint
           [1,0,0,0x3f000000],               # half a minimum subnormal -> zero
           [3,0,0,0x3f000000],               # 1.5 subnormals -> even 2
           [0x00800000,0,0,0x3f000000],      # normal input -> subnormal product
           [0x007fffff,0,0,0x3f800000],      # largest subnormal input
           [0x5f3504f3,0,0,0],               # 2*x*x rounds to max finite
           [0x5f3504f4,0,0,0]]               # adjacent x overflows 2*x*x
    assert matrix(cases[2])[1][5:8:2]==[2,0x80000002]
    assert matrix(cases[4])[1][0]==0xbe638e38
    assert matrix(cases[5])[0]==1 and matrix(cases[7])[0]==2
    assert matrix(cases[9])[1][2:4]==[0x80000000,0x80000000]
    assert matrix(cases[10])[1][2:4]==[0,0]
    assert matrix(cases[11])[1][5]==0x40400002
    assert matrix(cases[12])[1][5]==0 and matrix(cases[13])[1][5]==4
    assert matrix(cases[14])[1][5]==0x00800000
    assert matrix(cases[15])[1][5]==0x00fffffe
    assert matrix(cases[16])[0]==0 and matrix(cases[17])[0]==2
    rng=random.Random(0x514d4154)
    for _ in range(512):
        q=[]
        for _ in range(4):
            # Diverse moderate magnitudes, signs and exact binary32 payloads.
            q.append((rng.getrandbits(1)<<31)|(rng.randrange(120,132)<<23)|rng.getrandbits(23))
        cases.append(q)
    request=bytearray(b'QMATR001'+struct.pack('<I',len(cases)))
    expected=bytearray(b'QMATOUT1'+struct.pack('<I',len(cases)))
    statuses=Counter()
    for q in cases:
        request.extend(struct.pack('<4I',*q)); status,values=matrix(q); statuses[status]+=1
        expected.extend(struct.pack('<10I',status,*values))
    out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(parents=True,exist_ok=False)
    (out/'input.bin').write_bytes(request);(out/'expected.bin').write_bytes(expected)
    report={'schema':'cell-quaternion-matrix-rational-v1','cases':len(cases),'statuses':dict(statuses),
            'input_sha256':sha(request),'expected_sha256':sha(expected),'pins':pins(),
            'rounding_witness_cases':[4,11],'signed_zero_and_cancellation_cases':[9,10],
            'subnormal_witness_cases':[2,3,12,13,14,15], 'overflow_boundary_cases':[16,17],
            'scope':'synthetic finite binary32, signed zero and subnormal; MXCSR1F80; nonfinite input/intermediate rejected',
            'original_execution':False,'runtime_state_observed':False}
    (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))

def verify(directory,reply):
    m=json.loads((directory/'manifest.json').read_text(encoding='utf-8'))
    if m['pins']!=pins(): raise ValueError('source pins changed')
    request=(directory/'input.bin').read_bytes(); expected=(directory/'expected.bin').read_bytes(); actual=reply.read_bytes()
    if sha(request)!=m['input_sha256'] or sha(expected)!=m['expected_sha256']: raise ValueError('corpus hash differs')
    if actual!=expected: raise ValueError(f'reply mismatch expected={sha(expected)} actual={sha(actual)}')
    print(json.dumps({'cases':m['cases'],'reply_sha256':sha(actual),'reply_provenance_authenticated':False,'original_execution':False}))

def main():
    p=argparse.ArgumentParser(description=__doc__); s=p.add_subparsers(dest='action',required=True)
    a=s.add_parser('prepare');a.add_argument('directory',type=Path)
    a=s.add_parser('verify');a.add_argument('directory',type=Path);a.add_argument('reply',type=Path)
    a=p.parse_args()
    if a.action=='prepare':prepare(a.directory)
    else:verify(a.directory,a.reply)
if __name__=='__main__':main()

#!/usr/bin/env python3
"""Independent exact-rational candidate-size fixtures. No Spore execution.

prepare: read a specifically admitted PatchData snapshot, pin decoded bytes,
write cases.tsv and expected.tsv. verify: compare an independently produced
adapter TSV byte-for-byte; verify does not execute Cargo or any native host.
    python -I -S tools/cell_start_equal_size_rational_v1.py prepare "D:/Games/SPORE Collection/Data/PatchData.package" target/cell-start-size-fixture-NEW
    python -I -S tools/cell_start_equal_size_rational_v1.py verify target/cell-start-size-fixture-NEW <rust-reply.tsv>
Outputs are model comparison inputs, not original-instruction observations.
"""
import argparse
from collections import Counter
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import struct
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent))
from native_archive_index_probe import decode
from refpack import decompress

ROOT=Path(__file__).resolve().parent.parent
PKG_SHA='cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff'
G_SHA='e8c4b80348fd8f3a1821533c87110315ba584f66f081d46305ff61ab6cbf8c72'
C_SHA='f12d98c0f59af6d4d58174016f1e04d709647480310103341193df9b0fa76542'
PIN_FILES=[Path(__file__),ROOT/'tools/native_archive_index_probe.py',ROOT/'tools/refpack.py',
           ROOT/'crates/spore-core/src/cell_start_equal_size_v1.rs',ROOT/'crates/spore-core/src/cell_request_rng_v1.rs',
           ROOT/'crates/spore-core/src/native_x87.rs',ROOT/'crates/spore-core/src/cell_definition.rs',ROOT/'crates/spore-core/src/cell_globals.rs',
           ROOT/'crates/spore-core/examples/cell_start_equal_size_compare_v1.rs']
def digest(b): return hashlib.sha256(b).hexdigest()
def filehash(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
    return h.hexdigest()
def pins(): return {str(p.relative_to(ROOT)):filehash(p) for p in PIN_FILES}
def p2(e): return Fraction(1<<e,1) if e>=0 else Fraction(1,1<<(-e))
def normal(bits): return bits>>31==0 and 1<=((bits>>23)&255)<=254
def value(bits):
    exp=(bits>>23)&255; mant=bits&0x7fffff
    if exp==255: raise ValueError('nonfinite')
    v=Fraction(mant if exp==0 else mant|0x800000)*p2(-149 if exp==0 else exp-150)
    return -v if bits>>31 else v
def exponent(x):
    k=x.numerator.bit_length()-x.denominator.bit_length()
    return k-1 if x<p2(k) else k

def rounded_int(x, mode):
    # Positive quantities only; mode: nearest-even, down, up, toward-zero.
    q,r=divmod(x.numerator,x.denominator)
    if mode==0: q+=int(r*2>x.denominator or (r*2==x.denominator and q%2==1))
    elif mode==2: q+=int(r!=0)
    return q

def x87_round(x,prec,mode):
    assert x>0 and prec in (53,64)
    unit=p2(exponent(x)-prec+1)
    return rounded_int(x/unit,mode)*unit

def f32_store(x,mode):
    # Positive x only. Explicit IEEE binary32 normal/subnormal quantum and
    # overflow handling; returns None for +infinity. No host float arithmetic.
    assert x>0
    e=exponent(x); unit=p2(max(e-23,-149)); q=rounded_int(x/unit,mode); y=q*unit
    if y>=p2(128): return 0x7f7fffff if mode in (1,3) else None
    if not y: return 0
    e=exponent(y)
    if e < -126: return int(y/p2(-149))
    return ((e+127)<<23) | (int(y/p2(e-23)) & 0x7fffff)

def oracle(case):
    ident,lo,hi,divisor,arg,seed,cw=case
    if ident!=0xcbcd1287: return ('UnsupportedCandidate','-','-','-')
    if lo!=hi or not normal(lo): return ('UnsupportedRange','-','-','-')
    if not normal(divisor) or not normal(arg): return ('UnsupportedInput','-','-','-')
    pc=cw&0x300
    if cw&0x3f!=0x3f or pc not in (0x200,0x300): return ('UnsupportedControl','-','-','-')
    if seed==0: return ('UnsupportedRng','-','-','-')
    state=(seed*0x278dde6d)&0xffffffff
    if state==0x80000000: return ('UnsupportedRng','-','-','-')
    prec=53 if pc==0x200 else 64; mode=(cw>>10)&3
    inverse=x87_round(Fraction(1)/value(divisor),prec,mode)
    scaled=x87_round(value(lo)*inverse,prec,mode)
    scaled=x87_round(scaled*value(arg),prec,mode)
    out=f32_store(scaled,mode)
    if out is None: return ('Arithmetic.NonFiniteResult','-','-','-')
    if not normal(out): return ('UnsupportedInput','-','-','-')
    return ('OK',f'{lo:08X}',f'{out:08X}',f'{state:08X}')

def resources(path):
    before=filehash(path)
    if before!=PKG_SHA: raise ValueError('package hash mismatch')
    with path.open('rb') as f:
        head=f.read(96)
        if len(head)!=96 or head[:4]!=b'DBPF': raise ValueError('invalid header')
        count=struct.unpack_from('<I',head,0x24)[0]; size=struct.unpack_from('<I',head,0x2c)[0]; at=struct.unpack_from('<I',head,0x40)[0]
        if size>32*1024*1024 or at+size>path.stat().st_size: raise ValueError('index bounds')
        f.seek(at); block=f.read(size)
        if len(block)!=size: raise ValueError('short index')
        _, rows=decode(block,count)
        result={}
        for key,expected in [((0xa426730b,0x2a3ce5b7,0),G_SHA),((0xcbcd1287,0xdfad9f51,0),C_SHA)]:
            matches=[row for row in rows if row[:3]==key]
            if len(matches)!=1: raise ValueError('key absent or duplicate')
            row=matches[0]; _,_,_,off,stored,dec,comp,_,_=row
            if comp!=0xffff or stored>100000 or dec>100000 or off+stored>path.stat().st_size: raise ValueError('payload bounds/tag')
            f.seek(off); blob=f.read(stored)
            if len(blob)!=stored: raise ValueError('short payload')
            raw=decompress(blob,header=True)[0]
            if len(raw)!=dec or digest(raw)!=expected: raise ValueError('decoded payload mismatch')
            result[key]=raw
    if filehash(path)!=before: raise ValueError('package changed during extraction')
    g=result[(0xa426730b,0x2a3ce5b7,0)]; c=result[(0xcbcd1287,0xdfad9f51,0)]
    if len(g)!=276 or len(c)!=796 or struct.unpack_from('<I',g,0x34)[0]!=0xcbcd1287 or struct.unpack_from('<I',c)[0]!=0xcbcd1287: raise ValueError('candidate structure')
    low,high=struct.unpack_from('<II',c,0x304)
    if low!=high or not normal(low): raise ValueError('candidate not equal positive normal')
    return low

def cases(candidate):
    # The first endpoint is source-backed; every other endpoint and each
    # scale/seed/control combination is an explicitly synthetic test input.
    tuples=[(candidate,0x3f800000,0x3f800000),
            (candidate,0x40400000,0x3f800001),
            (0x3f800001,0x40400000,0x3f7fffff),
            (0x3eaaaaab,0x3f800001,0x40400000),
            (0x00800000,0x40000000,0x3f800000),
            (0x7f7fffff,0x3f000000,0x3f800000),
            # Exact halfway between adjacent f32 outputs: (1 + 2^-23) * 1.5.
            (0x3f800001,0x3f800000,0x3fc00000),
            # Just beyond the largest finite f32; directed rounding differs.
            (0x7f7fffff,0x3f800000,0x3f800001),
            (0x00800001,0x40000000,0x40000001),
            (0x3f000001,0x3f000000,0x3f800001),
            (0x41234567,0x3f923456,0x3eabcdef)]
    for ep,div,arg in tuples:
        for pc in (0x200,0x300):
            for rc in range(4):
                for seed in (1,0xffffffff,0x12345678):
                    yield ('candidate' if ep==candidate else 'synthetic',(0xcbcd1287,ep,ep,div,arg,seed,0x7f|pc|(rc<<10)))
    base=(0xcbcd1287,candidate,candidate,0x3f800000,0x3f800000,1,0x027f)
    for index, replacement in [(0,7),(2,0x3f800000),(3,0),(3,1),(4,1),(4,0x7f800000),(5,0),(5,0x80000000),(6,0x007f),(6,0x017f),(6,0x027e)]:
        a=list(base);a[index]=replacement;yield ('rejection',tuple(a))
    a=list(base);a[1]=a[2]=1;yield ('rejection',tuple(a))

def prepare(package,out):
    ep=resources(package); all_cases=list(cases(ep)); rows=[]; answers=[]
    for label,c in all_cases:
        rows.append(label+'\t'+'\t'.join(f'{v:08X}' for v in c))
        answers.append('\t'.join(oracle(c)))
    request=('\n'.join(rows)+'\n').encode(); expected=('\n'.join(answers)+'\n').encode()
    # Witness requires different directed rounding results for identical input.
    witness=any(oracle((ident,lo,hi,div,arg,seed,0x027f))[2] != oracle((ident,lo,hi,div,arg,seed,0x0a7f))[2]
                for _,(ident,lo,hi,div,arg,seed,cw) in all_cases if cw==0x027f and oracle((ident,lo,hi,div,arg,seed,0x027f))[0]=='OK')
    if not witness: raise ValueError('no directed rounding witness in corpus')
    midpoint=(0xcbcd1287,0x3f800001,0x3f800001,0x3f800000,0x3fc00000,1,0x027f)
    if oracle(midpoint)[2]!='3FC00002' or oracle(midpoint[:-1]+(0x067f,))[2]!='3FC00001':
        raise ValueError('midpoint witness failed')
    overflow=(0xcbcd1287,0x7f7fffff,0x7f7fffff,0x3f800000,0x3f800001,1,0x027f)
    if oracle(overflow)[0]!='Arithmetic.NonFiniteResult' or oracle(overflow[:-1]+(0x067f,))[2]!='7F7FFFFF':
        raise ValueError('directed overflow witness failed')
    out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(parents=True,exist_ok=False)
    (out/'cases.tsv').write_bytes(request);(out/'expected.tsv').write_bytes(expected)
    report={'schema':'cell-start-equal-size-rational-v1','records':len(rows),'candidate_endpoint_bits':f'{ep:08X}',
            'input_sha256':digest(request),'expected_sha256':digest(expected),'package_sha256':PKG_SHA,
            'globals_sha256':G_SHA,'cell_sha256':C_SHA,'pins':pins(),'directed_rounding_witness':witness,
            'package_stability_precondition':'no concurrent modification or transient replacement',
            'source_execution':False,'retail_fp_state_known':False,'rust_compared':False,
            'columns':'label,cell-id,low,high,global514c,arg6,seed,cw in hex; output status,sample,scale,state',
            'statuses':dict(sorted(Counter(a.split('\t',1)[0] for a in answers).items())),
            'scope':'single supplied candidate size bytes; synthetic inputs including midpoint/overflow and rejected subnormal/reserved precision; rational model only'}
    (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))

def verify(directory,reply):
    report=json.loads((directory/'manifest.json').read_text(encoding='utf-8'))
    if report['pins']!=pins(): raise ValueError('pinned source changed')
    cases_bytes=(directory/'cases.tsv').read_bytes(); expected=(directory/'expected.tsv').read_bytes()
    if digest(cases_bytes)!=report['input_sha256'] or digest(expected)!=report['expected_sha256']: raise ValueError('fixture changed')
    actual=reply.read_bytes()
    if actual!=expected: raise ValueError(f'comparison mismatch expected={digest(expected)} actual={digest(actual)}')
    print(json.dumps({'rows':report['records'],'matches':True,'reply_sha256':digest(actual),'reply_provenance_authenticated':False,'source_execution':False}))

def main():
    p=argparse.ArgumentParser(description=__doc__);sub=p.add_subparsers(dest='cmd',required=True)
    a=sub.add_parser('prepare');a.add_argument('package',type=Path);a.add_argument('output',type=Path)
    a=sub.add_parser('verify');a.add_argument('directory',type=Path);a.add_argument('reply',type=Path)
    a=p.parse_args()
    if a.cmd=='prepare':prepare(a.package,a.output)
    else:verify(a.directory,a.reply)
if __name__=='__main__':main()

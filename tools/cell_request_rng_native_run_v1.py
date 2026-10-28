"""Run already-built hosts against one bounded deterministic corpus."""
import argparse, hashlib, json, math, random, struct, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
MULT=0x278dde6d

def corpus():
    inv=pow(MULT,-1,1<<32)
    targets=[1,2,0x7ffffffe,0x7fffffff,0x80000000,0x80000001,0x80000002,0xfffffffe,0xffffffff]
    seeds=[1,2,3,0x7fffffff,0x80000000,0x80000001,0xfffffffe,0xffffffff]
    seeds += [(x*inv)&0xffffffff for x in targets]
    randomizer=random.Random(0x935d40)
    seeds += [randomizer.randrange(1,1<<32) for _ in range(64)]
    result=[]
    for cw in [0x7f|precision|rounding for precision in (0x200,0x300) for rounding in (0,0x400,0x800,0xc00)]:
        for seed in seeds:
            state=seed
            for _ in range(16):
                result.append((state,cw)); state=(state*MULT)&0xffffffff
                assert state
    return result

def main():
    p=argparse.ArgumentParser(); p.add_argument('--host',type=Path,required=True); p.add_argument('--adapter',type=Path,required=True); p.add_argument('--out',type=Path,required=True); a=p.parse_args()
    target=(ROOT/'target').resolve()
    for x in (a.host,a.adapter,a.out): x.resolve().relative_to(target)
    rows=corpus(); request=b''.join(struct.pack('<II',*row) for row in rows)
    a.out.mkdir(parents=True,exist_ok=False)
    (a.out/'request.bin').write_bytes(request)
    outputs=[]
    for name,path in [('native',a.host),('model',a.adapter)]:
        proc=subprocess.run([str(path.resolve())],input=request,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW)
        (a.out/(name+'.bin')).write_bytes(proc.stdout)
        assert proc.returncode==0 and not proc.stderr and len(proc.stdout)==12*len(rows), (name,proc.returncode,len(proc.stdout),proc.stderr)
        outputs.append(proc.stdout)
    for index,(state,cw) in enumerate(rows):
        ns,nv=struct.unpack_from('<Id',outputs[0],index*12); ms,mv=struct.unpack_from('<Id',outputs[1],index*12)
        if ns!=ms or not math.isfinite(nv) or nv!=mv:
            raise AssertionError((index,hex(state),hex(cw),hex(ns),hex(ms),nv,mv))
    report=dict(cases=len(rows), controls=8, steps_per_seed=16, compare='state and numeric binary64; zero sign excluded', native_sha256=hashlib.sha256(outputs[0]).hexdigest(),model_sha256=hashlib.sha256(outputs[1]).hexdigest(), request_sha256=hashlib.sha256(request).hexdigest(), retail_control_word_known=False)
    (a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n'); print(json.dumps(report))
if __name__=='__main__': main()

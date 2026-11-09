"""Review-only one-shot controlled submission runner; do not execute pre-audit."""
import argparse,hashlib,json,random,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
HOST=ROOT/'target/cell-tail-submit-review-v1/host-map.exe'
DLL=ROOT/'target/cell-tail-x87-wrapper-link-review-v1/wrapper.dll'
ADAPTER=ROOT/'target/debug/examples/cell_tail_submit_oracle_v1.exe'
HOST_REC=ROOT/'target/cell-tail-submit-review-v1/build-observation.json'
ADAPTER_REC=ROOT/'target/cell-tail-submit-review-v1/adapter-receipt-v2.json'
WRAPPER_REC=ROOT/'target/cell-tail-x87-wrapper-link-review-v1/build-observation.json'
PINS={HOST:'5443a3fa4bbdeba1fc2a2734d66b08769c6634aa3bad8be3f01b86cbb810b702',
 DLL:'9afd99d82133e7e0c973ef0597ddc0d8bf4aef84849af5a5a5fd9c2f9795d236',
 ADAPTER:'60cd9d7855015f270b7f1fa18c10f7f89aae863a22b79b6ada45adf43b68e237',
 HOST_REC:'c5d79632f814c612b40ce461135fd1f41d72184aa3c7c791db7c1c6b31023e50',
 ADAPTER_REC:'2239bcf3368b908b2d0da3fa64661d2f2593f8fd9b391001ceaee6d5b8602d82',
 WRAPPER_REC:'c0470fb3eb712f8e26dae8febf53bce0f4f720a939841bc1c9d4a1f521edd289'}
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def verify():
    for p,h in PINS.items():
        if not p.is_absolute() or sha(p)!=h: raise ValueError('pin mismatch '+str(p))
    for rec in (HOST_REC,ADAPTER_REC,WRAPPER_REC):
        r=json.loads(rec.read_text()); entries=r.get('files',r.get('pinned_files'))
        for e in entries:
            p=Path(e['path'])
            if not p.is_absolute() or sha(p)!=e['sha256']: raise ValueError('receipt mismatch '+str(p))

def cases():
    rng=random.Random(0xe6a2b0)
    depths=[0,0x80000000,1,0x80000001,0x007fffff,0x807fffff,0x00800000,0x80800000,
            0x3f800000,0xbf800000,0x7f7fffff,0xff7fffff]
    for selector in (0x80000000,0xffffffff,0,0x15,0x16,0x7fffffff):
        for depth in depths:
            for word in (0,1,0x80000000):
                row=[rng.getrandbits(32) for _ in range(22)]
                row[2]=selector; row[5]=depths[rng.randrange(len(depths))]; row[20]=depth; row[21]=word
                yield row

def parse(data,counts):
    if len(data)>4096: raise ValueError('oversized output')
    lines=data.decode('ascii').splitlines()
    if len(lines)!=len(counts): raise ValueError('wrong line count')
    result=[]
    for line,n in zip(lines,counts):
        words=line.split()
        if len(words)!=n or any(len(w)!=8 or any(c not in '0123456789abcdef' for c in w) for w in words): raise ValueError('bad word protocol')
        result.append(words)
    return result

def main():
    p=argparse.ArgumentParser(); p.add_argument('--out',type=Path,required=True); a=p.parse_args()
    out=a.out.resolve(); out.relative_to((ROOT/'target').resolve()); out.mkdir(parents=True,exist_ok=False)
    rec={'schema':'cell-tail-submit-comparison-v2','status':'in_progress','limits':'controlled pure stubs, finite nonaliasing relation5 only; output captured before size check; no sandbox or fault recovery; post-build and stable-file limitations'}
    def write(): (out/'receipt.json').write_text(json.dumps(rec,indent=2)+'\n')
    write()
    try:
        runner=Path(__file__).resolve(); runner_hash=sha(runner); verify()
        corpus=list(cases()); payload=''.join(' '.join(f'{x:08x}' for x in row)+'\n' for row in corpus)
        (out/'cases.txt').write_text(payload,encoding='ascii'); rec.update(runner_sha256=runner_hash,cases=len(corpus),cases_sha256=sha(out/'cases.txt'),pins={str(k):v for k,v in PINS.items()}); write()
        with (out/'results.jsonl').open('x',encoding='utf-8') as f:
            for idx,row in enumerate(corpus):
                if idx==0:
                    verify()
                    if sha(runner)!=runner_hash: raise ValueError('runner changed before launch')
                args=[f'{v:08x}' for v in row]; record={'index':idx}
                for name,cmd,counts in [('host',[str(HOST.resolve()),str(DLL.resolve()),*args],[9,12]),('adapter',[str(ADAPTER.resolve()),*args],[12,12])]:
                    try: r=subprocess.run(cmd,capture_output=True,timeout=10)
                    except (OSError,subprocess.TimeoutExpired) as e:
                        record[name]={'error':repr(e)}
                        if isinstance(e,subprocess.TimeoutExpired):
                            record[name]['partial_stdout_hex']=(e.stdout or b'').hex()
                            record[name]['partial_stderr_hex']=(e.stderr or b'').hex()
                        f.write(json.dumps(record)+'\n'); f.flush(); rec['failed_case']=idx; raise
                    record[name]={'exit':r.returncode,'stdout':r.stdout.decode('ascii',errors='replace'),'stderr':r.stderr.decode('ascii',errors='replace')}
                    if r.returncode or r.stderr:
                        f.write(json.dumps(record)+'\n'); f.flush(); rec['failed_case']=idx; raise ValueError(name+' failed')
                    try: parsed=parse(r.stdout,counts)
                    except Exception:
                        f.write(json.dumps(record)+'\n'); f.flush(); rec['failed_case']=idx; raise
                    record[name]['parsed']=parsed
                f.write(json.dumps(record)+'\n'); f.flush()
                # Host's success is conditional on checking three pointers against
                # its trusted buffers. Never compare those process addresses.
                if record['host']['parsed'][0][3:]!=record['adapter']['parsed'][0][:6] or record['adapter']['parsed'][0][6:]!=args[14:20] or record['host']['parsed'][1]!=record['adapter']['parsed'][1]:
                    rec['failed_case']=idx; raise ValueError('candidate/output mismatch')
        verify()
        if sha(runner)!=runner_hash: raise ValueError('runner changed during run')
        rec['results_sha256']=sha(out/'results.jsonl'); rec['status']='passed'
    except Exception as e:
        rec['status']='failed'; rec['error']=repr(e); write(); raise
    write(); print(json.dumps(rec))
if __name__=='__main__': main()

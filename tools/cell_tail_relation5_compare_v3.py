"""Explicitly pinned finite direct-tail host/model comparison; no retries."""
import argparse, hashlib, json, random, subprocess, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def corpus():
    rng=random.Random(0xE6A2B0)
    depths=[0,0x80000000,1,0x80000001,0x007fffff,0x807fffff,0x00800000,0x80800000,
            0x3f800000,0xbf800000,0x7f7fffff,0xff7fffff]
    selectors=[0x80000000,0xffffffff,0,0x15,0x16,0x17,0x7fffffff]
    result=[]
    for selector in selectors:
        for stored in depths:
            for incoming in depths:
                row=[rng.getrandbits(32) for _ in range(21)]
                row[2]=selector; row[5]=stored; row[20]=incoming
                result.append(row)
    return result
def main():
    p=argparse.ArgumentParser()
    for n in ('host','adapter','prep-manifest','build-receipt','adapter-receipt'):
        p.add_argument('--'+n,type=Path,required=True); p.add_argument('--'+n+'-sha256',required=True)
    p.add_argument('--out',type=Path,required=True); a=p.parse_args()
    out=a.out.resolve(); out.relative_to((ROOT/'target').resolve())
    paths={'host':a.host.resolve(),'adapter':a.adapter.resolve(),'prep_manifest':a.prep_manifest.resolve(),
           'build_receipt':a.build_receipt.resolve(),
           'adapter_receipt':a.adapter_receipt.resolve(),
           'model':ROOT/'crates/spore-core/src/cell_feeding_tail_e6a2b0_v1.rs',
           'adapter_source':ROOT/'crates/spore-core/examples/cell_tail_relation5_oracle_v1.rs',
           'runner':Path(__file__).resolve()}
    for name,expected in [('host',a.host_sha256),('adapter',a.adapter_sha256),('prep_manifest',a.prep_manifest_sha256),('build_receipt',a.build_receipt_sha256),('adapter_receipt',a.adapter_receipt_sha256)]:
        if sha(paths[name])!=expected.lower(): raise ValueError(name+' pin mismatch')
    build=json.loads(paths['build_receipt'].read_text())
    if build['host_sha256']!=sha(paths['host']) or build['prep_manifest_sha256']!=sha(paths['prep_manifest']): raise ValueError('build receipt mismatch')
    for entry in build['pinned_files']:
        if sha(entry['path'])!=entry['sha256']: raise ValueError('build input changed: '+entry['path'])
    adapter_build=json.loads(paths['adapter_receipt'].read_text())
    if adapter_build['adapter_sha256']!=sha(paths['adapter']): raise ValueError('adapter receipt mismatch')
    for entry in adapter_build['pinned_files']:
        if sha(entry['path'])!=entry['sha256']: raise ValueError('adapter input changed: '+entry['path'])
    manifest=json.loads(paths['prep_manifest'].read_text())
    if manifest['schema']!='cell-tail-relation5-stub-preparation-v3': raise ValueError('unexpected preparation')
    pins={k:sha(v) for k,v in paths.items()}
    cases=corpus(); out.mkdir(parents=True,exist_ok=False)
    payload=''.join(' '.join(f'{w:08x}' for w in row)+'\n' for row in cases)
    (out/'cases.txt').write_text(payload,encoding='ascii')
    receipt={'schema':'cell-tail-relation5-comparison-v1','paths':{k:str(v) for k,v in paths.items()},'sha256':pins,
             'cases_sha256':sha(out/'cases.txt'),'cases':len(cases),'status':'in_progress',
             'limitations':'finite non-aliasing relation 5 with fixed nonzero stub only; no debugger or sandbox; timeout kills direct child only; build provenance must be reviewed separately'}
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    with (out/'results.jsonl').open('x',encoding='utf-8') as f:
        for index,row in enumerate(cases):
            if index==0:
                if any(sha(v)!=pins[k] for k,v in paths.items()): raise RuntimeError('pinned file changed immediately before execution')
                for entry in build['pinned_files']:
                    if sha(entry['path'])!=entry['sha256']: raise RuntimeError('build input changed immediately before execution')
                for entry in adapter_build['pinned_files']:
                    if sha(entry['path'])!=entry['sha256']: raise RuntimeError('adapter input changed immediately before execution')
            args=[f'{w:08x}' for w in row]
            record={'index':index}
            for name in ('host','adapter'):
                try:
                    r=subprocess.run([str(paths[name]),*args],capture_output=True,text=True,timeout=5)
                    record[name]={'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr}
                except subprocess.TimeoutExpired:
                    record[name]={'timeout':True}
                    f.write(json.dumps(record)+'\n'); f.flush(); receipt['status']='failed'; receipt['failed_case']=index
                    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n'); raise
                except OSError as exc:
                    record[name]={'launch_error':str(exc)}
                    f.write(json.dumps(record)+'\n'); f.flush(); receipt['status']='failed'; receipt['failed_case']=index
                    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n'); raise
                response=record[name]
                if response['exit']!=0 or response['stderr'] or len(response['stdout'].split())!=12 or any(len(w)!=8 or any(c not in '0123456789abcdef' for c in w) for w in response['stdout'].split()):
                    f.write(json.dumps(record)+'\n'); f.flush(); receipt['status']='failed'; receipt['failed_case']=index
                    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n'); raise RuntimeError(f'invalid {name} output at case {index}')
            f.write(json.dumps(record)+'\n'); f.flush()
            if record['host']['stdout']!=record['adapter']['stdout']:
                receipt['status']='failed'; receipt['failed_case']=index; (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
                raise RuntimeError(f'mismatch at case {index}')
    if any(sha(v)!=pins[k] for k,v in paths.items()): raise RuntimeError('pinned file changed during run')
    for entry in build['pinned_files']:
        if sha(entry['path'])!=entry['sha256']: raise RuntimeError('build input changed during run')
    for entry in adapter_build['pinned_files']:
        if sha(entry['path'])!=entry['sha256']: raise RuntimeError('adapter input changed during run')
    receipt['status']='passed'; receipt['results_sha256']=sha(out/'results.jsonl')
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(receipt))
if __name__=='__main__': main()

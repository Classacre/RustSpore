"""REVIEW REQUIRED: run pinned hidden selector host and model adapter once."""
import argparse, hashlib, json, os, re, struct, subprocess, sys
from pathlib import Path

def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def write_new(path, data):
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|getattr(os,'O_BINARY',0),0o600)
    with os.fdopen(fd,'wb') as f: f.write(data); f.flush(); os.fsync(f.fileno())
def main():
    ap=argparse.ArgumentParser()
    for name in ('source','host','adapter','manifest','host-source','model-source'):
        ap.add_argument('--'+name,required=True,type=Path); ap.add_argument('--'+name+'-sha256',required=True)
    ap.add_argument('--out',required=True,type=Path); ap.add_argument('--admit-stable-paths',action='store_true',required=True)
    args=ap.parse_args(); root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
    parent=args.out.parent.resolve(strict=True)
    if not parent.is_relative_to(root): raise ValueError('output parent must be under target')
    inputs={name.replace('-','_'):getattr(args,name.replace('-','_')).resolve(strict=True) for name in ('source','host','adapter','manifest','host-source','model-source')}
    expected={k:getattr(args,k+'_sha256').lower() for k in inputs}
    if any(not re.fullmatch('[0-9a-f]{64}',v) for v in expected.values()) or {k:sha(p) for k,p in inputs.items()}!=expected: raise ValueError('input identity mismatch')
    if expected['source']!='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a': raise ValueError('unapproved source image')
    manifest=json.loads(inputs['manifest'].read_text())
    if manifest['source_pe_sha256']!=expected['source'] or manifest['host_source_sha256']!=expected['host_source']: raise ValueError('manifest identity mismatch')
    host_text=inputs['host_source'].read_text(); m=re.search(r'input_bits\[\]\s*=\s*\{(.*?)\};',host_text,re.S)
    if not m: raise ValueError('missing fixed corpus')
    bits=[int(x.strip(),0) for x in m.group(1).split(',') if x.strip()]
    args.out.mkdir()
    receipt={'scope':'synthetic fixed selector input; no live runtime','status':'failed', 'input_sha256':expected,
             'host':{'status':'not_started','exit_code':None},'adapter':{'status':'not_started','exit_code':None},
             'failure':None,'stable_paths':'operator-admitted; endpoint hashes cannot prove continuous stability',
             'limits':'compares result words only, not x87 exception equivalence or runtime state'}
    native_out=b''; adapter_out=b''; native_err=b''; adapter_err=b''
    try:
        try:
            p=subprocess.run([str(inputs['host'])],stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30,creationflags=subprocess.CREATE_NO_WINDOW,check=False)
            native_out,native_err=p.stdout,p.stderr; receipt['host']={'status':'exited','exit_code':p.returncode}
        except subprocess.TimeoutExpired as e:
            native_out,native_err=e.stdout or b'',e.stderr or b''; receipt['host']={'status':'timeout','exit_code':None}
        except OSError as e:
            receipt['host']={'status':'launch_error','exit_code':None,'error':str(e)}
        # Retain every raw stream before interpreting any result. Even an empty
        # stream is a useful record of a launch, timeout or program failure.
        write_new(args.out/'native.bin',native_out); write_new(args.out/'host-stderr.bin',native_err)
        if receipt['host']['status']=='exited':
            try:
                p=subprocess.run([str(inputs['adapter'])],input=native_out,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30,creationflags=subprocess.CREATE_NO_WINDOW,check=False)
                adapter_out,adapter_err=p.stdout,p.stderr; receipt['adapter']={'status':'exited','exit_code':p.returncode}
            except subprocess.TimeoutExpired as e:
                adapter_out,adapter_err=e.stdout or b'',e.stderr or b''; receipt['adapter']={'status':'timeout','exit_code':None}
            except OSError as e:
                receipt['adapter']={'status':'launch_error','exit_code':None,'error':str(e)}
        write_new(args.out/'adapter-stdout.bin',adapter_out); write_new(args.out/'adapter-stderr.bin',adapter_err)
        if receipt['host']['status']!='exited' or receipt['host']['exit_code']!=0: raise ValueError('host did not complete successfully')
        if len(native_out)!=len(bits)*2*28: raise ValueError('host output extent mismatch')
        expected_rows=[]
        for index in range(len(bits)*2):
            cw,p,seg,bucket,frac,sw,mx=struct.unpack_from('<7I',native_out,index*28)
            if (cw,p)!=((0x027f,0x037f)[index//len(bits)],bits[index%len(bits)]) or seg>=5 or bucket>=2 or sw&0x3840 or mx&~0x3f!=0x1f80: raise ValueError(f'host metadata or corpus mismatch at row {index}')
            expected_rows.append(f'{cw:08X} {p:08X} {seg:08X} {bucket:08X} {frac:08X}')
        if receipt['adapter']['status']!='exited' or receipt['adapter']['exit_code']!=0: raise ValueError('adapter did not complete successfully')
        observed=adapter_out.decode('ascii').splitlines()
        if observed!=expected_rows:
            first=next((i for i,(a,b) in enumerate(zip(observed,expected_rows)) if a!=b),min(len(observed),len(expected_rows)))
            raise ValueError(f'model mismatch at row {first}; expected {len(expected_rows)} rows, got {len(observed)}')
        receipt['status']='matched'; receipt['rows']=len(expected_rows)
    except Exception as e:
        receipt['failure']=f'{type(e).__name__}: {e}'
    finally:
        try:
            after={k:sha(p) for k,p in inputs.items()}; receipt['input_sha256_after']=after
            if after!=expected: receipt['status']='failed'; receipt['failure']=(receipt['failure'] or '')+'; input endpoint identity changed'
        except Exception as e:
            receipt['status']='failed'; receipt['failure']=(receipt['failure'] or '')+f'; endpoint check failed: {e}'
        for filename in ('native.bin','host-stderr.bin','adapter-stdout.bin','adapter-stderr.bin'):
            path=args.out/filename
            if path.is_file(): receipt.setdefault('artifacts',{})[filename]={'sha256':sha(path),'bytes':path.stat().st_size}
        write_new(args.out/'receipt.json',(json.dumps(receipt,indent=2)+'\n').encode())
    if receipt['status']!='matched': raise RuntimeError(receipt['failure'])
    print('matched',receipt['rows'],'records')
if __name__=='__main__':
    try: main()
    except Exception as e: print('error:',e,file=sys.stderr); sys.exit(1)

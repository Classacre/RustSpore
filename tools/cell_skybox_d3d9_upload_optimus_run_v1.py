"""REVIEW REQUIRED: execute fixed D3D9 fixture and retain raw failure evidence."""
import argparse, hashlib, json, os, struct, subprocess, sys
from pathlib import Path
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def write_new(p,b):
    fd=os.open(p,os.O_CREAT|os.O_EXCL|os.O_WRONLY|getattr(os,'O_BINARY',0),0o600)
    with os.fdopen(fd,'wb') as f: f.write(b); f.flush(); os.fsync(f.fileno())
def main():
    p=argparse.ArgumentParser(); names=('source','package','candidate','host','host-source','wrapper-source','manifest','header','disasm','runner','inventory-receipt')
    for n in names:
        p.add_argument('--'+n,required=True,type=Path); p.add_argument('--'+n+'-sha256',required=True)
    p.add_argument('--out',required=True,type=Path);p.add_argument('--admit-stable-paths',action='store_true',required=True)
    a=p.parse_args();root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
    if not a.out.parent.resolve(strict=True).is_relative_to(root): raise ValueError('output outside target')
    paths={n.replace('-','_'):getattr(a,n.replace('-','_')).resolve(strict=True) for n in names};pins={k:getattr(a,k+'_sha256').lower() for k in paths}
    fixed={'source':'17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a','package':'e01bb86d8ddf02c56d52394b4e1db7181c1e1ebe1878bf6fb72c4ee523fcedd1','candidate':'5705df1ab03e94c58c8a890383026e68c68dfe64ba488d4d767f2706d8ad8aec'}
    if paths['runner']!=Path(__file__).resolve() or any(pins[k]!=v for k,v in fixed.items()) or any(len(v)!=64 for v in pins.values()) or {k:sha(v) for k,v in paths.items()}!=pins: raise ValueError('input identity mismatch')
    m=json.loads(paths['manifest'].read_text())
    if m['source_sha256']!=pins['source'] or m['header_sha256']!=pins['header'] or m['host_source_sha256']!=pins['host_source']: raise ValueError('manifest mismatch')
    candidate=paths['candidate'].read_bytes(); a.out.mkdir();out=b'';err=b''
    receipt={'status':'failed','scope':'source size and copy fragments; source-modeled D3D9 managed texture calls; locked CPU readback, no GPU sampling, live provider, material or camera','input_sha256':pins,'host':{'status':'not_started','exit_code':None},'failure':None,'stable_paths':'operator admitted; endpoint hashes do not prove continuous stability'}
    try:
        try:
            r=subprocess.run([str(paths['host'])],input=candidate,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW,check=False);out,err=r.stdout,r.stderr;receipt['host']={'status':'exited','exit_code':r.returncode}
        except subprocess.TimeoutExpired as e:out,err=e.stdout or b'',e.stderr or b'';receipt['host']={'status':'timeout','exit_code':None}
        except OSError as e:receipt['host']={'status':'launch_error','exit_code':None,'error':str(e)}
        write_new(a.out/'native.bin',out);write_new(a.out/'stderr.bin',err)
        if receipt['host']['status']!='exited' or receipt['host']['exit_code']!=0: raise ValueError('host failure')
        lengths=[65536,16384,4096,1024,256,64,16,16,16]; head=24+9*48
        if len(out)!=head+sum(lengths): raise ValueError('unexpected output extent')
        magic,stage,vendor,device,adapter,create=struct.unpack_from('<6I',out)
        if (magic,stage,vendor,create)!=(0x31583944,3,0x10de,0): raise ValueError('adapter or creation mismatch')
        receipt['adapter']={'vendor':vendor,'device':device,'ordinal':adapter}; receipt['levels']=[];offset=0
        for i,n in enumerate(lengths):
            row=struct.unpack_from('<12I',out,24+i*48);w=max(1,256>>i);pitch=((w+3)//4)*16
            if row!=(n,0,w,w,0x35545844,0,pitch,0,0,pitch,0,1): raise ValueError('level metadata mismatch '+str(i)+': '+repr(row))
            src=candidate[0x174+offset:0x174+offset+n];got=out[head+offset:head+offset+n]
            if src!=got: raise ValueError('level bytes mismatch '+str(i))
            receipt['levels'].append({'level':i,'length':n,'pitch':pitch,'sha256':hashlib.sha256(got).hexdigest()});offset+=n
        receipt['status']='matched'
    except Exception as e:receipt['failure']=f'{type(e).__name__}: {e}'
    finally:
        try:
            after={k:sha(v) for k,v in paths.items()};receipt['input_sha256_after']=after
            if after!=pins:receipt['status']='failed';receipt['failure']=(receipt['failure'] or '')+'; input endpoint changed'
        except Exception as e:receipt['status']='failed';receipt['failure']=(receipt['failure'] or '')+'; endpoint error: '+str(e)
        for name in ('native.bin','stderr.bin'):
            f=a.out/name
            if f.is_file():receipt.setdefault('artifacts',{})[name]={'sha256':sha(f),'bytes':f.stat().st_size}
        write_new(a.out/'receipt.json',(json.dumps(receipt,indent=2)+'\n').encode())
    if receipt['status']!='matched':raise RuntimeError(receipt['failure'])
    print('nine managed texture levels matched')
if __name__=='__main__':
    try:main()
    except Exception as e:print('error:',e,file=sys.stderr);sys.exit(1)

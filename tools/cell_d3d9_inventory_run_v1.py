"""Review required: run paired pinned inventories; preserve raw evidence."""
import argparse,hashlib,json,os,subprocess,sys
from pathlib import Path
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write(p,b):
 fd=os.open(p,os.O_CREAT|os.O_EXCL|os.O_WRONLY|getattr(os,'O_BINARY',0),0o600)
 with os.fdopen(fd,'wb') as f:f.write(b);f.flush();os.fsync(f.fileno())
def main():
 p=argparse.ArgumentParser();names=('x86','x64','source','runner','previous-log','previous-receipt')
 for n in names:p.add_argument('--'+n,required=True,type=Path);p.add_argument('--'+n+'-sha256',required=True)
 p.add_argument('--out',required=True,type=Path);p.add_argument('--admit-stable-paths',required=True,action='store_true');a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 paths={n.replace('-','_'):getattr(a,n.replace('-','_')).resolve(strict=True) for n in names};pins={k:getattr(a,k+'_sha256').lower() for k in paths}
 if paths['runner']!=Path(__file__).resolve() or any(len(x)!=64 for x in pins.values()) or {k:sha(v) for k,v in paths.items()}!=pins:raise ValueError('input identity mismatch')
 a.out.mkdir();receipt={'status':'failed','scope':'paired no-window D3D9 enumeration only, no device or live game','pins':pins,'runs':{},'failure':None,'stable_paths':'operator admitted; endpoint checks do not prove continuous stability'}
 try:
  for name in ('x86','x64'):
   stdout=b'';stderr=b'';result={'status':'not_started','exit_code':None}
   try:
    r=subprocess.run([str(paths[name])],stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30,creationflags=subprocess.CREATE_NO_WINDOW,check=False);stdout,stderr=r.stdout,r.stderr;result={'status':'exited','exit_code':r.returncode}
   except subprocess.TimeoutExpired as e:stdout,stderr=e.stdout or b'',e.stderr or b'';result={'status':'timeout','exit_code':None}
   except OSError as e:result={'status':'launch_error','exit_code':None,'error':str(e)}
   receipt['runs'][name]=result;write(a.out/(name+'.stdout'),stdout);write(a.out/(name+'.stderr'),stderr)
  if any(r['status']!='exited' or r['exit_code']!=0 for r in receipt['runs'].values()):raise ValueError('one or more inventory hosts failed')
  receipt['status']='completed'
 except Exception as e:receipt['failure']=f'{type(e).__name__}: {e}'
 finally:
  try:
   after={k:sha(v) for k,v in paths.items()};receipt['pins_after']=after
   if after!=pins:receipt['status']='failed';receipt['failure']=(receipt['failure'] or '')+'; input endpoint changed'
  except Exception as e:receipt['status']='failed';receipt['failure']=(receipt['failure'] or '')+'; endpoint error: '+str(e)
  for f in a.out.iterdir():receipt.setdefault('artifacts',{})[f.name]={'sha256':sha(f),'bytes':f.stat().st_size}
  write(a.out/'receipt.json',(json.dumps(receipt,indent=2)+'\n').encode())
 if receipt['status']!='completed':raise RuntimeError(receipt['failure'])
 print('inventories completed; inspect retained output')
if __name__=='__main__':
 try:main()
 except Exception as e:print('error:',e,file=sys.stderr);sys.exit(1)

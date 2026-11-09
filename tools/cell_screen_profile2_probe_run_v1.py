"""REVIEW REQUIRED: hidden explicit profile-2 exploratory GPU probe."""
import argparse,hashlib,json,os,subprocess,sys
from pathlib import Path
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(4*1024*1024),b''):h.update(b)
 return h.hexdigest()
def write(p,b):
 fd=os.open(p,os.O_CREAT|os.O_EXCL|os.O_WRONLY|getattr(os,'O_BINARY',0),0o600)
 with os.fdopen(fd,'wb') as f:f.write(b);f.flush();os.fsync(f.fileno())
def main():
 p=argparse.ArgumentParser();names=('exe','source','runner','effdir','shader-package','texture-package')
 for n in names:p.add_argument('--'+n,required=True,type=Path);p.add_argument('--'+n+'-sha256',required=True)
 p.add_argument('--out',required=True,type=Path);p.add_argument('--admit-stable-paths',required=True,action='store_true');a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 paths={n.replace('-','_'):getattr(a,n.replace('-','_')).resolve(strict=True) for n in names};pins={k:getattr(a,k+'_sha256').lower() for k in paths}
 if paths['runner']!=Path(__file__).resolve() or any(len(x)!=64 for x in pins.values()) or {k:sha(v) for k,v in paths.items()}!=pins:raise ValueError('input pin mismatch')
 a.out.mkdir();out=b'';err=b'';receipt={'status':'failed','pins':pins,'host':{'status':'not_started','exit_code':None},'scope':'controlled profile 2, source shader and candidate raster; not live state','failure':None,'stable_paths':'operator admitted; endpoint hashes do not prove continuous stability'}
 args=[str(paths['exe']),'--profile2','--inspect',str(paths['effdir']),pins['effdir'],str(paths['shader_package']),'8228198755fee4793a14598b859067a1463d46c5c42d0cf2fda618fd2f6efa60','70000091',str(paths['texture_package']),'5705df1ab03e94c58c8a890383026e68c68dfe64ba488d4d767f2706d8ad8aec','768x768','0','0','1,1,1,1','rtx',str((a.out/'output.png').resolve())]
 receipt['args']=args
 try:
  try:
   r=subprocess.run(args,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=90,creationflags=subprocess.CREATE_NO_WINDOW,check=False);out,err=r.stdout,r.stderr;receipt['host']={'status':'exited','exit_code':r.returncode}
  except subprocess.TimeoutExpired as e:out,err=e.stdout or b'',e.stderr or b'';receipt['host']={'status':'timeout','exit_code':None}
  except OSError as e:receipt['host']={'status':'launch_error','exit_code':None,'error':str(e)}
  write(a.out/'stdout.bin',out);write(a.out/'stderr.bin',err)
  if receipt['host']['status']!='exited' or receipt['host']['exit_code']!=0 or not (a.out/'output.png').is_file():raise ValueError('probe failed or no output')
  if b'profile=2 render_type=0' not in out or b'presented=false' not in out:raise ValueError('missing controlled probe markers')
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
 print('profile-2 controlled probe completed')
if __name__=='__main__':
 try:main()
 except Exception as e:print('error:',e,file=sys.stderr);sys.exit(1)

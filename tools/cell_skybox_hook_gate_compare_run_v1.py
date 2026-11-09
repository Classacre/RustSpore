"""REVIEW REQUIRED: run pinned hook-gate host and retain failure evidence."""
import argparse,hashlib,json,os,struct,subprocess,sys
from pathlib import Path
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write_new(p,b):
 fd=os.open(p,os.O_CREAT|os.O_EXCL|os.O_WRONLY|getattr(os,'O_BINARY',0),0o600)
 with os.fdopen(fd,'wb') as f:f.write(b);f.flush();os.fsync(f.fileno())
def main():
 p=argparse.ArgumentParser();names=('source','host','host-source','manifest','header','disasm','runner')
 for n in names:p.add_argument('--'+n,required=True,type=Path);p.add_argument('--'+n+'-sha256',required=True)
 p.add_argument('--out',required=True,type=Path);p.add_argument('--admit-stable-paths',action='store_true',required=True);a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output parent outside target')
 paths={n.replace('-','_'):getattr(a,n.replace('-','_')).resolve(strict=True) for n in names};pins={k:getattr(a,k+'_sha256').lower() for k in paths}
 if paths['runner']!=Path(__file__).resolve(strict=True) or any(len(v)!=64 for v in pins.values()) or {k:sha(v) for k,v in paths.items()}!=pins:raise ValueError('input identity mismatch')
 m=json.loads(paths['manifest'].read_text())
 if m['source_sha256']!=pins['source'] or m['host_source_sha256']!=pins['host_source'] or m['header_sha256']!=pins['header']:raise ValueError('manifest mismatch')
 a.out.mkdir();receipt={'status':'failed','scope':'guarded synthetic hook gate with explicit recording stub; no original forwarding hook, device or live state','input_sha256':pins,'host':{'status':'not_started','exit_code':None},'failure':None,'stable_paths':'operator admitted; endpoint hashes cannot establish continuous stability'};out=b'';err=b''
 try:
  try:
   r=subprocess.run([str(paths['host'])],stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30,creationflags=subprocess.CREATE_NO_WINDOW,check=False);out,err=r.stdout,r.stderr;receipt['host']={'status':'exited','exit_code':r.returncode}
  except subprocess.TimeoutExpired as e:out,err=e.stdout or b'',e.stderr or b'';receipt['host']={'status':'timeout','exit_code':None}
  except OSError as e:receipt['host']={'status':'launch_error','exit_code':None,'error':str(e)}
  write_new(a.out/'native.bin',out);write_new(a.out/'stderr.bin',err)
  if receipt['host']['status']!='exited' or receipt['host']['exit_code']!=0:raise ValueError('host failure')
  if len(out)!=6*40:raise ValueError('unexpected output extent')
  actual=[struct.unpack_from('<10I',out,i*40) for i in range(6)]
  expected=[(0,1,2,1,1,1,0,0,0,1),(1,1,2,2,1,1,0,0,0,1),(2,1,2,0,1,1,0xfeedcafe,1,1,1),(3,0,2,1,0,0,0xfeedcafe,1,1,1),(4,1,0,0,1,1,0xfeedcafe,1,1,1),(5,1,0xffffffff,0,1,1,0xfeedcafe,1,1,1)]
  if actual!=expected:raise ValueError('projection mismatch: '+repr(actual))
  receipt['status']='matched';receipt['cases']=len(expected);receipt['results']=[list(x) for x in actual]
 except Exception as e:receipt['failure']=f'{type(e).__name__}: {e}'
 finally:
  try:
   after={k:sha(v) for k,v in paths.items()};receipt['input_sha256_after']=after
   if after!=pins:receipt['status']='failed';receipt['failure']=(receipt['failure'] or '')+'; input endpoint changed'
  except Exception as e:receipt['status']='failed';receipt['failure']=(receipt['failure'] or '')+f'; endpoint error: {e}'
  for name in ('native.bin','stderr.bin'):
   f=a.out/name
   if f.is_file():receipt.setdefault('artifacts',{})[name]={'sha256':sha(f),'bytes':f.stat().st_size}
  write_new(a.out/'receipt.json',(json.dumps(receipt,indent=2)+'\n').encode())
 if receipt['status']!='matched':raise RuntimeError(receipt['failure'])
 print('six guarded hook gate cases matched')
if __name__=='__main__':
 try:main()
 except Exception as e:print('error:',e,file=sys.stderr);sys.exit(1)

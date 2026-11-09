"""REVIEW REQUIRED: hidden D3D9 inventory and conditional filter projection."""
import argparse,hashlib,json,os,re,subprocess,sys
from pathlib import Path
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write(p,b):
 fd=os.open(p,os.O_CREAT|os.O_EXCL|os.O_WRONLY|getattr(os,'O_BINARY',0),0o600)
 with os.fdopen(fd,'wb') as f:f.write(b);f.flush();os.fsync(f.fileno())
def main():
 p=argparse.ArgumentParser();names=('host','source','runner','executable','bounds-report')
 for n in names:p.add_argument('--'+n,required=True,type=Path);p.add_argument('--'+n+'-sha256',required=True)
 p.add_argument('--min-width',required=True,type=int);p.add_argument('--min-height',required=True,type=int);p.add_argument('--out',required=True,type=Path);p.add_argument('--admit-stable-paths',required=True,action='store_true');a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 paths={n.replace('-','_'):getattr(a,n.replace('-','_')).resolve(strict=True) for n in names};pins={k:getattr(a,k+'_sha256').lower() for k in paths}
 if paths['runner']!=Path(__file__).resolve() or pins['executable']!='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a' or any(len(x)!=64 for x in pins.values()) or {k:sha(v) for k,v in paths.items()}!=pins:raise ValueError('pin mismatch')
 if not (1<=a.min_width<=65535 and 1<=a.min_height<=65535):raise ValueError('explicit bounds')
 a.out.mkdir();out=b'';err=b'';receipt={'status':'failed','pins':pins,'host':{'status':'not_started','exit_code':None},'failure':None,'scope':'local RTX inventory; source-modeled filtering with supplied bounds, not active game list','stable_paths':'operator admitted; endpoint hashes do not prove continuous stability'}
 try:
  try:
   r=subprocess.run([str(paths['host'])],stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30,creationflags=subprocess.CREATE_NO_WINDOW,check=False);out,err=r.stdout,r.stderr;receipt['host']={'status':'exited','exit_code':r.returncode}
  except subprocess.TimeoutExpired as e:out,err=e.stdout or b'',e.stderr or b'';receipt['host']={'status':'timeout','exit_code':None}
  except OSError as e:receipt['host']={'status':'launch_error','exit_code':None,'error':str(e)}
  write(a.out/'stdout.bin',out);write(a.out/'stderr.bin',err)
  if receipt['host']['status']!='exited' or receipt['host']['exit_code']!=0:raise ValueError('host failure')
  lines=out.decode('ascii').splitlines();header=re.fullmatch(r'adapter=(\d+) vendor=10DE device=2520 desktop=(\d+),(\d+),(\d+),(\d+)',lines[0]);
  if not header:raise ValueError('header')
  adapter,wmax,hmax,refresh,fmt=map(int,header.groups());formats=[35,22,24,23];records=[];pos=1
  for f in formats:
   m=re.fullmatch(r'format=(\d+) count=(\d+)',lines[pos]);pos+=1
   if not m or int(m[1])!=f:raise ValueError('format framing')
   count=int(m[2]);
   if count>65536:raise ValueError('count bound')
   for i in range(count):
    m=re.fullmatch(r'mode=(\d+),(\d+),(\d+),(\d+),(\d+),(\d+)',lines[pos]);pos+=1
    if not m:raise ValueError('mode framing')
    q=list(map(int,m.groups()))
    if q[0]!=f or q[1]!=i or q[5]!=f:raise ValueError('mode identity')
    if f in (21,22,33) and a.min_width<=q[2]<=wmax and a.min_height<=q[3]<=hmax:records.append({'index':len(records),'format':f,'source_ordinal':i,'width':q[2],'height':q[3],'refresh':q[4]})
  if pos!=len(lines):raise ValueError('extra output')
  receipt.update(status='completed',adapter=adapter,desktop={'width':wmax,'height':hmax,'refresh':refresh,'format':fmt},bounds={'width':a.min_width,'height':a.min_height},accepted_modes=records,limitations='Projection assumes this selected adapter, its present desktop mode, source format sequence and bounds; live global property, selected adapter, setup time, device state and subsequent list mutation unobserved; duplicates retained.')
 except Exception as e:receipt['failure']=f'{type(e).__name__}: {e}'
 finally:
  try:
   after={k:sha(v) for k,v in paths.items()};receipt['pins_after']=after
   if after!=pins:receipt['status']='failed';receipt['failure']=(receipt['failure'] or '')+'; input endpoint changed'
  except Exception as e:receipt['status']='failed';receipt['failure']=(receipt['failure'] or '')+'; endpoint error: '+str(e)
  for f in a.out.iterdir():receipt.setdefault('artifacts',{})[f.name]={'sha256':sha(f),'bytes':f.stat().st_size}
  write(a.out/'receipt.json',(json.dumps(receipt,indent=2)+'\n').encode())
 if receipt['status']!='completed':raise RuntimeError(receipt['failure'])
 print('mode inventory projected; inspect receipt')
if __name__=='__main__':
 try:main()
 except Exception as e:print('error:',e,file=sys.stderr);sys.exit(1)

"""REVIEW REQUIRED: compare controlled GPU sampling against CPU BC3 decoding."""
import argparse,hashlib,json,os,struct,subprocess,sys
from pathlib import Path
from raster_extract import decode_dxt, FMT_DXT5
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write(p,b):
 fd=os.open(p,os.O_CREAT|os.O_EXCL|os.O_WRONLY|getattr(os,'O_BINARY',0),0o600)
 with os.fdopen(fd,'wb') as f:f.write(b);f.flush();os.fsync(f.fileno())
def main():
 p=argparse.ArgumentParser();names=('candidate','host','host-source','upload-source','header','disasm','decoder','runner','upload-receipt')
 for n in names:p.add_argument('--'+n,required=True,type=Path);p.add_argument('--'+n+'-sha256',required=True)
 p.add_argument('--out',required=True,type=Path);p.add_argument('--admit-stable-paths',required=True,action='store_true');a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output outside target')
 paths={n.replace('-','_'):getattr(a,n.replace('-','_')).resolve(strict=True) for n in names};pins={k:getattr(a,k+'_sha256').lower() for k in paths}
 if paths['runner']!=Path(__file__).resolve() or paths['decoder']!=Path(__file__).with_name('raster_extract.py').resolve() or pins['candidate']!='5705df1ab03e94c58c8a890383026e68c68dfe64ba488d4d767f2706d8ad8aec' or any(len(x)!=64 for x in pins.values()) or {k:sha(v) for k,v in paths.items()}!=pins:raise ValueError('input identity mismatch')
 candidate=paths['candidate'].read_bytes();a.out.mkdir();out=b'';err=b''
 receipt={'status':'failed','scope':'controlled fixed-function point sampled 9-mip DXT5, CPU reference; no live game material/camera','assumptions':'A8R8G8B8, point min/mag/mip, clamp, max mip level i, viewport 256>>i, zero LOD bias, no sRGB write/read, blend, alpha test, depth, dither or cull; half-pixel vertices','pins':pins,'host':{'status':'not_started','exit_code':None},'failure':None,'stable_paths':'operator admitted; endpoint hashes do not prove continuous stability'}
 try:
  try:
   r=subprocess.run([str(paths['host'])],input=candidate,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW,check=False);out,err=r.stdout,r.stderr;receipt['host']={'status':'exited','exit_code':r.returncode}
  except subprocess.TimeoutExpired as e:out,err=e.stdout or b'',e.stderr or b'';receipt['host']={'status':'timeout','exit_code':None}
  except OSError as e:receipt['host']={'status':'launch_error','exit_code':None,'error':str(e)}
  write(a.out/'native.bin',out);write(a.out/'stderr.bin',err)
  if receipt['host']['status']!='exited' or receipt['host']['exit_code']!=0:raise ValueError('host failure')
  if len(out)!=20+349524 or struct.unpack_from('<5I',out)!=(0x31504d53,3,0,0x252010de,9):raise ValueError('unexpected host output')
  sizes=[65536,16384,4096,1024,256,64,16,16,16];co=0;po=20;receipt['levels']=[]
  for i,n in enumerate(sizes):
   w=max(1,256>>i);ref=decode_dxt(candidate[0x174+co:0x174+co+n],w,w,FMT_DXT5);raw=out[po:po+w*w*4];actual=bytearray(len(raw))
   for k in range(0,len(raw),4):actual[k:k+4]=bytes((raw[k+2],raw[k+1],raw[k],raw[k+3]))
   diffs=[abs(a-b) for a,b in zip(actual,ref)];hist={str(d):diffs.count(d) for d in set(diffs)}
   receipt['levels'].append({'level':i,'width':w,'channels':len(ref),'reference_sha256':hashlib.sha256(ref).hexdigest(),'sampled_rgba_sha256':hashlib.sha256(actual).hexdigest(),'different_channels':sum(d!=0 for d in diffs),'max_abs_difference':max(diffs),'difference_histogram':hist});co+=n;po+=len(ref)
  receipt['status']='compared'
 except Exception as e:receipt['failure']=f'{type(e).__name__}: {e}'
 finally:
  try:
   after={k:sha(v) for k,v in paths.items()};receipt['pins_after']=after
   if after!=pins:receipt['status']='failed';receipt['failure']=(receipt['failure'] or '')+'; input endpoint changed'
  except Exception as e:receipt['status']='failed';receipt['failure']=(receipt['failure'] or '')+'; endpoint error: '+str(e)
  for f in a.out.iterdir():receipt.setdefault('artifacts',{})[f.name]={'sha256':sha(f),'bytes':f.stat().st_size}
  write(a.out/'receipt.json',(json.dumps(receipt,indent=2)+'\n').encode())
 if receipt['status']!='compared':raise RuntimeError(receipt['failure'])
 print('nine levels compared; inspect differences in receipt')
if __name__=='__main__':
 try:main()
 except Exception as e:print('error:',e,file=sys.stderr);sys.exit(1)

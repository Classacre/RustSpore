"""REVIEW REQUIRED: run fixed hidden projection host; preserve failed evidence."""
import argparse,hashlib,json,os,struct,subprocess,sys
from pathlib import Path
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write_new(p,b):
 fd=os.open(p,os.O_CREAT|os.O_EXCL|os.O_WRONLY|getattr(os,'O_BINARY',0),0o600)
 with os.fdopen(fd,'wb') as f:f.write(b);f.flush();os.fsync(f.fileno())
def main():
 p=argparse.ArgumentParser();names=('source','package','host','host-source','candidate','manifest','runner')
 for name in names:p.add_argument('--'+name,required=True,type=Path);p.add_argument('--'+name+'-sha256',required=True)
 p.add_argument('--out',required=True,type=Path);p.add_argument('--admit-stable-paths',action='store_true',required=True);a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output parent outside target')
 paths={name.replace('-','_'):getattr(a,name.replace('-','_')).resolve(strict=True) for name in names};expected={k:getattr(a,k+'_sha256').lower() for k in paths}
 if paths['runner']!=Path(__file__).resolve(strict=True) or any(len(x)!=64 for x in expected.values()) or {k:digest(v) for k,v in paths.items()}!=expected:raise ValueError('input pin mismatch')
 m=json.loads(paths['manifest'].read_text());
 if m['source_sha256']!=expected['source'] or m['package_sha256_before_after']!=expected['package'] or m['payload_sha256']!=expected['candidate'] or m['host_source_sha256']!=expected['host_source']:raise ValueError('manifest mismatch')
 data=paths['candidate'].read_bytes()
 if len(data)!=87780 or struct.unpack_from('<III',data,0x24)[0]!=2 or struct.unpack_from('<I',data,0x30)[0]!=0x144 or struct.unpack_from('<I',data,0x38)[0]!=0:raise ValueError('candidate framing mismatch')
 if struct.unpack_from('<6I',data,0x144+24)!=(0x124,0,0x20,4,5,0x20003):raise ValueError('candidate row mismatch')
 a.out.mkdir();receipt={'status':'failed','scope':'isolated synthetic pointer relocation and explicit registry lookup; no provider, callback, hook, GPU or live state', 'input_sha256':expected,'host':{'status':'not_started','exit_code':None},'failure':None,'stable_paths':'operator-admitted; endpoint hashes do not prove continuous stability'};stdout=b'';stderr=b''
 try:
  try:
   r=subprocess.run([str(paths['host'])],input=data,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30,creationflags=subprocess.CREATE_NO_WINDOW,check=False);stdout,stderr=r.stdout,r.stderr;receipt['host']={'status':'exited','exit_code':r.returncode}
  except subprocess.TimeoutExpired as e:stdout,stderr=e.stdout or b'',e.stderr or b'';receipt['host']={'status':'timeout','exit_code':None}
  except OSError as e:receipt['host']={'status':'launch_error','exit_code':None,'error':str(e)}
  write_new(a.out/'native.bin',stdout);write_new(a.out/'stderr.bin',stderr)
  if receipt['host']['status']!='exited' or receipt['host']['exit_code']!=0:raise ValueError('native host failed')
  if len(stdout)!=32:raise ValueError('native output extent mismatch')
  base,relocated,preserved,typ,offset,tag,missing,guard=struct.unpack('<8I',stdout)
  if not base or base>0xfffff000 or relocated!=base+0x124 or (preserved,typ,offset,tag,missing,guard)!=(0x124,0x20003,28,0x20003,0,1):raise ValueError('native projection mismatch')
  receipt['status']='matched';receipt['projection']={'image_base':f'{base:08X}','object_offset':f'{relocated-base:08X}','preserved':f'{preserved:08X}','type':f'{typ:08X}','registry_offset':offset,'tag':f'{tag:08X}','missing':missing,'guards':guard}
 except Exception as e:receipt['failure']=f'{type(e).__name__}: {e}'
 finally:
  try:
   after={k:digest(v) for k,v in paths.items()};receipt['input_sha256_after']=after
   if after!=expected:receipt['status']='failed';receipt['failure']=(receipt['failure'] or '')+'; input endpoint changed'
  except Exception as e:receipt['status']='failed';receipt['failure']=(receipt['failure'] or '')+f'; endpoint error: {e}'
  for name in ('native.bin','stderr.bin'):
   f=a.out/name
   if f.is_file():receipt.setdefault('artifacts',{})[name]={'sha256':digest(f),'bytes':f.stat().st_size}
  write_new(a.out/'receipt.json',(json.dumps(receipt,indent=2)+'\n').encode())
 if receipt['status']!='matched':raise RuntimeError(receipt['failure'])
 print('fixed synthetic relocation and registry lookup matched')
if __name__=='__main__':
 try:main()
 except Exception as e:print('error:',e,file=sys.stderr);sys.exit(1)

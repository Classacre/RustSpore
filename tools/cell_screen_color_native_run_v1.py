"""Compare prebuilt original-byte and public-model hosts; artifacts stay in target."""
import argparse, hashlib, json, random, struct, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent

def corpus():
 rows=[]
 def add(values,label): rows.append((values,label))
 special=[0,0x80000000,1,0x80000001,0x007fffff,0x807fffff,0x00800000,0x80800000,0x3f800000,0xbf800000,0x7f7fffff,0xff7fffff,0x7f800000,0xff800000,0x7fc00000,0xffc01234,0x7f800001,0xff800123]
 for channel in range(4):
  for v in special:
   a=[0x3f000000]*4; a[channel]=v; add(a,'special-channel-%d'%channel)
 # Values adjacent to the float nearest each half-integer scaled threshold.
 for k in range(255):
  center=struct.unpack('<I',struct.pack('<f',(k+0.5)/255))[0]
  for v in (center-1,center,center+1):
   for channel in range(4):
    a=[0,0,0,0]; a[channel]=v; add(a,'half-threshold-%d-channel-%d'%(k,channel))
 rng=random.Random(0x4584e0)
 for _ in range(4096):
  a=[]
  while len(a)<4:
   v=rng.getrandbits(32)
   if v&0x7f800000 != 0x7f800000: a.append(v)
  add(a,'random-finite')
 return rows

def sha(b): return hashlib.sha256(b).hexdigest()
def main():
 p=argparse.ArgumentParser(); p.add_argument('--host',type=Path,required=True); p.add_argument('--adapter',type=Path,required=True); p.add_argument('--out',type=Path,required=True); p.add_argument('--expected-host-sha256',required=True); p.add_argument('--expected-adapter-sha256',required=True); a=p.parse_args()
 for path in (a.host,a.adapter,a.out): path.resolve().relative_to((ROOT/'target').resolve())
 expected={'native':a.expected_host_sha256.lower(),'adapter':a.expected_adapter_sha256.lower()}
 for name,value in expected.items():
  if len(value)!=64 or any(c not in '0123456789abcdef' for c in value): raise ValueError('invalid expected SHA-256 for '+name)
 for name,path in [('native',a.host),('adapter',a.adapter)]:
  if sha(path.read_bytes())!=expected[name]: raise ValueError(name+' executable digest mismatch before launch')
 rows=corpus(); request=b''.join(struct.pack('<4I',*v) for v,_ in rows)
 a.out.mkdir(parents=True,exist_ok=False); (a.out/'request.bin').write_bytes(request)
 replies=[]
 for name,path,width in [('native',a.host,4),('adapter',a.adapter,8)]:
  r=subprocess.run([str(path.resolve())],input=request,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=120,creationflags=subprocess.CREATE_NO_WINDOW)
  if sha(path.read_bytes())!=expected[name]: raise RuntimeError(name+' executable digest changed during run')
  (a.out/(name+'.bin')).write_bytes(r.stdout); (a.out/(name+'.stderr')).write_bytes(r.stderr)
  if r.returncode or r.stderr or len(r.stdout)!=len(rows)*width: raise RuntimeError((name,r.returncode,len(r.stdout),r.stderr[:500]))
  replies.append(r.stdout)
 errors=[]; mismatches=[]
 for i,(values,label) in enumerate(rows):
  native=struct.unpack_from('<I',replies[0],i*4)[0]; status,model=struct.unpack_from('<2I',replies[1],i*8)
  detail=dict(index=i,label=label,input=[hex(v) for v in values],native=hex(native),status=status,model=hex(model))
  if status: errors.append(detail)
  elif native!=model: mismatches.append(detail)
 if sha(a.host.read_bytes())!=expected['native'] or sha(a.adapter.read_bytes())!=expected['adapter']: raise RuntimeError('executable digest changed before report')
 report=dict(cases=len(rows),compared=len(rows)-len(errors),model_rejections=errors,mismatches=mismatches,request_sha256=sha(request),native_reply_sha256=sha(replies[0]),adapter_reply_sha256=sha(replies[1]),host_exe_sha256=sha(a.host.read_bytes()),adapter_exe_sha256=sha(a.adapter.read_bytes()),expected_host_sha256=expected['native'],expected_adapter_sha256=expected['adapter'],mxcsr='1f80',comparison='packed u32',seed='0x4584e0')
 (a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8'); print(json.dumps({k:v for k,v in report.items() if k not in ('model_rejections','mismatches')},indent=2))
 if errors or mismatches: raise SystemExit(1)
if __name__=='__main__': main()

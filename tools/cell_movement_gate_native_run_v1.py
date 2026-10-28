"""Run prebuilt hosts only; request/replies and reports remain under target."""
import argparse, hashlib, json, random, struct, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent

def corpus():
 one=0x3f800000; three=0x40400000; timer=0x3fc00000
 cases=[]
 def add(p,t,tm,delta,label): cases.append((p,t,tm,delta,label))
 for center in (one,three):
  for x in (center-1,center,center+1):
   for tm in (0,0x80000000,timer-1,timer,timer+1):
    for delta in (0,0x3e800000,0xbe800000): add([0,0,0],[x,0,0],tm,delta,'axis-threshold')
 for x in (three-1,three,three+1):
  for y in (0x3a83126f,0xba83126f): add([0,0,0],[x,y,0],0,0x3f000000,'rounded-length')
 for x in (one-1,one,one+1): add([0,0,0],[x,0x39800000,0],timer+1,0,'near-one')
 for sign in (0,0x80000000):
  for x in (0,1,0x007fffff,0x00800000): add([sign,0,0],[x|sign,0,0],0x80000000,1|sign,'tiny')
 rng=random.Random(0xe56960)
 for _ in range(128):
  def bounded(): return struct.unpack('<I',struct.pack('<f',rng.uniform(-4,4)))[0]
  add([bounded() for _ in range(3)],[bounded() for _ in range(3)],bounded(),bounded(),'seeded')
 return [(p+t+[tm,delta,0x7f|pc|rc,0x1f80],label) for pc in (0x200,0x300) for rc in (0,0x400,0x800,0xc00) for p,t,tm,delta,label in cases]

def sha(b):return hashlib.sha256(b).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--host',type=Path,required=True);p.add_argument('--adapter',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
 for path in (a.host,a.adapter,a.out):path.resolve().relative_to((ROOT/'target').resolve())
 rows=corpus(); request=b''.join(struct.pack('<10I',*r) for r,_ in rows);a.out.mkdir(parents=True,exist_ok=False);(a.out/'request.bin').write_bytes(request)
 replies=[]
 for name,path,width in [('native',a.host,8),('adapter',a.adapter,12)]:
  r=subprocess.run([str(path.resolve())],input=request,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=120,creationflags=subprocess.CREATE_NO_WINDOW)
  (a.out/(name+'.bin')).write_bytes(r.stdout);(a.out/(name+'.stderr')).write_bytes(r.stderr)
  if r.returncode or r.stderr or len(r.stdout)!=len(rows)*width:raise RuntimeError((name,r.returncode,len(r.stdout),r.stderr[:500]))
  replies.append(r.stdout)
 errors=[];mismatches=[]
 for i,((values,label)) in enumerate(rows):
  native=struct.unpack_from('<2I',replies[0],i*8);status,al,timer=struct.unpack_from('<3I',replies[1],i*12)
  detail=dict(index=i,label=label,input=[hex(v) for v in values],native=[hex(v) for v in native],model=[status,hex(al),hex(timer)])
  if status:errors.append(detail)
  elif native!=(al,timer):mismatches.append(detail)
 report=dict(cases=len(rows),compared=len(rows)-len(errors),model_rejections=errors,mismatches=mismatches,request_sha256=sha(request),native_sha256=sha(replies[0]),adapter_sha256=sha(replies[1]),host_exe_sha256=sha(a.host.read_bytes()),adapter_exe_sha256=sha(a.adapter.read_bytes()),comparison='AL and timer binary32 bits',x87_pc=[53,64],rounding_modes=4,mxcsr='1f80')
 (a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:v for k,v in report.items() if k not in ('model_rejections','mismatches')},indent=2))
 if errors or mismatches:raise SystemExit(1)
if __name__=='__main__':main()

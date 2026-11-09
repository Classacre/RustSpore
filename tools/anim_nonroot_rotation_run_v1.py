"""Review-gated one-shot ROT fixture runner. Never retries. Not yet reviewed."""
import argparse, hashlib, json, struct, subprocess, sys
from pathlib import Path

ROOT=Path(__file__).resolve().parent.parent
TOOLS=ROOT/'tools'
HOST=ROOT/'target/animation/rotation-native-v2'
PACKET=ROOT/'target/animation/nonroot-source-fixture-v1'
EXPECTED={
 'tools/anim_rotation_native_probe.py':'f572e2fa7ef5141e4411217d9ac6e2e79e9848b469eae73a5dafc801bee93aba',
 'tools/anim_rotation_native_run.py':'d34d9da54dfd06ab306727fed83558d819dafbb611ac74f695d8fc36b50ce476',
 'tools/anim_quaternion_probe.py':'300af6112c5d28a7b29577a29bb4bb3d50dd1cf60d651e8c16120714465407cf',
 'tools/native_crt_acos_probe.py':'593ba6b27d75faa4d8af6d42310021d8a4bd15c5abe5474c5f93b468d346834b',
 'tools/native_crt_acos_run.py':'44b8bc2f31643f23a5562ec9a4c72cdd7acb3effd95394d1e524f51c7d0d5ef1',
 'tools/cell_schema_probe.py':'05c9e3586e021b9de17668a8cce7ebc8c7ccbee9388dab25e9e4ca9cfc4493fe',
 'target/animation/rotation-native-v2/host-x86.exe':'972bf058d3cf7711b2e9f0ea5ff63c03d5e4e3f1a7a6dafa7fa17653e9d27f18',
 'target/animation/rotation-native-v2/host-x86.asm':'b914ed8a7ff6cd972b60cfc3733628489895ec9a2e538a79988547f27f8e84c6',
 'target/animation/rotation-native-v2/host-x86.map':'51737c770a2fad26457dd917b6bf725dd9e635e02f14080d3eb61b0ee2a541c4',
 'target/animation/rotation-native-v2/manifest.json':'7359156aa96bcc74731cbd26707529bcdfa3c0499cae370502c16c2d35c8077b',
 'target/animation/nonroot-source-fixture-v1/manifest.json':'eeb9516a99f26b4ec167574c25ed811ba206e09151b092c4f90024519785a89a',
 'target/animation/nonroot-source-fixture-v1/pending-rot-request.bin':'ba2b6e3bb6b801dee699d2eec99f14c0031befba00829feb6888c58871f91c55',
 'target/animation/nonroot-source-fixture-v1/source-anim.bin':'9daf9fe5bce32a039ff4a4de2ae70f9090b053b570c17da7d9141d61f83ee10a',
}
def digest(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for block in iter(lambda:f.read(1<<20),b''):h.update(block)
 return h.hexdigest()
def save(path,value):
 with path.open('x',encoding='utf-8',newline='\n') as f:json.dump(value,f,indent=2);f.write('\n')
def main():
 p=argparse.ArgumentParser(description=__doc__)
 p.add_argument('--reviewed-runner-sha256',required=True,help='Exact SHA-256 of independently reviewed runner')
 p.add_argument('--out',type=Path,required=True,help='New directory beneath target/animation')
 a=p.parse_args()
 own=Path(__file__).resolve();runner_hash=digest(own)
 if a.reviewed_runner_sha256 != runner_hash:raise ValueError('reviewed runner hash mismatch')
 for name,want in EXPECTED.items():
  if digest(ROOT/name)!=want:raise ValueError('pinned input mismatch: '+name)
 # Import only after validating workspace helper bytes.
 sys.path.insert(0,str(TOOLS))
 import anim_rotation_native_probe as probe
 import anim_rotation_native_run as rot
 import anim_quaternion_probe as quat
 import native_crt_acos_probe as crt
 from cell_schema_probe import SourceImage
 manifest=json.loads((HOST/'manifest.json').read_text())
 packet=json.loads((PACKET/'manifest.json').read_text())
 for name,want in packet['files'].items():
  if digest(PACKET/name)!=want:raise ValueError('packet file mismatch: '+name)
 for name,want in packet['pins'].items():
  if digest(ROOT/name)!=want:raise ValueError('packet provenance mismatch: '+name)
 source=Path(manifest['source_path']);dllpath=Path(manifest['crt_source_path'])
 if digest(source)!=manifest['source_sha256'] or digest(dllpath)!=manifest['crt_source_sha256']:
  raise ValueError('source/CRT hash mismatch')
 image=SourceImage(source);dll=crt.PE(dllpath);crt.decoder()
 if probe.audit_source(image)!=manifest['source_curve_allowlist']:raise ValueError('source curve audit differs')
 primitives,thunk=quat.audit_source(image)
 if primitives!=manifest['source_primitives'] or thunk!=manifest['source_acos_thunk']:raise ValueError('source primitive audit differs')
 crt_code,bounds=crt.audit_source(dll)
 if crt_code!=manifest['crt_code_allowlist'] or bounds!=manifest['crt_indexed_read_bound_proof']:raise ValueError('CRT audit differs')
 asm,_,_=probe.host_source(image,dll)
 if asm!=(HOST/'host-x86.asm').read_text():raise ValueError('regenerated host source differs')
 if probe.audit_host(HOST/'host-x86.exe',HOST/'host-x86.map')!=manifest['host_code_audit']:raise ValueError('linked host audit differs')
 request=(PACKET/'pending-rot-request.bin').read_bytes()
 if len(request)!=1952 or struct.unpack_from('<8I',request)!=(0,5,0x43310000,8,0x27f,1,1,0):raise ValueError('request header differs')
 frames=[struct.unpack_from('<I',request,32+i*120)[0] for i in range(5)]
 if frames!=[0,72,78,157,176] or request[632:]!=bytes(1320):raise ValueError('request frames/padding differ')
 for i in range(5):
  for offset in (20,24,28,32,36,104,108):
   if struct.unpack_from('<I',request,32+i*120+offset)[0]&0x7f800000==0x7f800000:raise ValueError('nonfinite input')
 detail=packet['rotation_window'];indices=detail['local_source_indices']
 if detail['local_neighbors']!=[3,0,1,2] or [indices[i] for i in detail['local_neighbors']]!=detail['full_neighbors'] or detail['full_neighbors']!=[7,0,1,2]:raise ValueError('source neighbor mapping differs')
 if hashlib.sha256(request[32:]).hexdigest()!=detail['source_row_window_sha256']:raise ValueError('source window differs')
 out=a.out.resolve();out.relative_to((ROOT/'target/animation').resolve())
 out.mkdir(parents=True,exist_ok=False)
 pins={str((ROOT/name).resolve()):want for name,want in EXPECTED.items()}
 pins.update({str((PACKET/name).resolve()):want for name,want in packet['files'].items()})
 pins.update({str((ROOT/name).resolve()):want for name,want in packet['pins'].items()})
 pins.update({str(own):runner_hash,str(source.resolve()):digest(source),str(dllpath.resolve()):digest(dllpath)})
 save(out/'execution-pins.json',pins)
 (out/'request.bin').write_bytes(request)
 result=None;error=None
 try:
  startup=subprocess.STARTUPINFO();startup.dwFlags|=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=0
  result=subprocess.run([str((HOST/'host-x86.exe').resolve())],input=request,capture_output=True,cwd=out,creationflags=subprocess.CREATE_NO_WINDOW,startupinfo=startup,timeout=60)
  (out/'reply.bin').write_bytes(result.stdout);(out/'stderr.bin').write_bytes(result.stderr)
  save(out/'status.json',{'attempts':1,'exit':result.returncode,'output_bytes':len(result.stdout),'stderr_bytes':len(result.stderr),'runner_sha256':runner_hash})
  if result.returncode!=0 or result.stderr or len(result.stdout)!=240:raise ValueError('host execution failed')
  structural=rot.inspect(request,result.stdout)
  save(out/'structural-report.json',{'structural':structural,'native_executed':True,'numeric_comparison':'pending independent Rust comparison','reply_sha256':hashlib.sha256(result.stdout).hexdigest()})
 except BaseException as e:
  error=repr(e);save(out/'failure.json',{'error':error,'attempts_at_most':1});raise
 finally:
  changed=[]
  for name,want in pins.items():
   try:
    if digest(Path(name))!=want:changed.append(name)
   except OSError:changed.append(name)
  save(out/'immutability.json',{'unchanged':not changed,'changed_files':changed,'operation_exception':error})
  if changed:raise RuntimeError('pinned files changed: '+repr(changed))
if __name__=='__main__':main()

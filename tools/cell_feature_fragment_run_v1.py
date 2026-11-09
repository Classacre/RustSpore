"""Pre-reviewed-runner candidate. Do not run until linked-host review completes."""
import hashlib,json,os,platform,struct,subprocess,sys,traceback
from pathlib import Path
import pefile
ROOT=Path(__file__).resolve().parents[1]
PREP=ROOT/'target/cell-feature-fragment-prep-v2'
RUN=ROOT/'target/cell-feature-fragment-run-v1'
SOURCE=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
MODEL=ROOT/'target/debug/examples/cell_feature_fragment_model_v1.exe'
PINS={
 SOURCE:'17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a',
 PREP/'manifest.json':'b299e5ca962221a15bbd620865b179397f47b2639248925531ce1a3bde41dcac',
 PREP/'fragment.bin':'addffaaee26f1051ffeaf1fca15f1fb463449e72c1a14e7c2e2fa50d6dbaefe2',
 PREP/'cases.bin':'a1bfafaae621cd9f168973bb719704d594cd27887f033d350fbafb9728bc67da',
 PREP/'host-x86.asm':'6e25e630d262289432d78c33e9f480fea2e541d734cfddf7e2dcec0a69b09b9a',
 PREP/'host-x86.exe':'86473031923820dffff338f2f8915e7c2cc5f2388c731080e1dc2c4be44b8629',
 ROOT/'crates/spore-bevy/src/cell_source_feature_geometry_v1.rs':'df641fd04862e251266be965c41f288e0e2e18b0151aecc3ffedb6540c6b5b8b',
 ROOT/'crates/spore-core/src/native_sse.rs':'eedac9f108e411f55abe01b1b05d5d690f950a954c16409b2222f15f00298129',
 ROOT/'crates/spore-bevy/examples/cell_feature_fragment_model_v1.rs':'24b4185f2d2657967196049a77d6472c1acc89b05d05a8bae9c6ec7387a1adde',
 MODEL:'9005ae4cb9e06e0ab4f7352c508e9eeeff731c109ecaf58709c6c0a37bb717b4',
}
def digest(data):return hashlib.sha256(data).hexdigest()
def require(ok,msg):
 if not ok:raise RuntimeError(msg)
def stable_pins(reviewed):
 paths={**PINS,Path(__file__).resolve():reviewed}
 for p,h in paths.items():
  for part in (p,*p.parents):
   require(not(part.lstat().st_file_attributes & 0x400),f'reparse point: {part}')
  require(digest(p.read_bytes())==h,f'pin mismatch: {p}')
def run_child(name,args):
 try:
  p=subprocess.run(args,capture_output=True,timeout=8,creationflags=subprocess.CREATE_NO_WINDOW,check=False)
  stdout,stderr=p.stdout,p.stderr
  result={'exit':p.returncode,'timeout':False}
 except subprocess.TimeoutExpired as e:
  stdout,stderr=e.stdout or b'',e.stderr or b''
  result={'exit':None,'timeout':True}
 (RUN/(name+'.stdout.bin')).write_bytes(stdout);(RUN/(name+'.stderr.bin')).write_bytes(stderr)
 result.update(stdout_sha256=digest(stdout),stderr_sha256=digest(stderr),stdout_bytes=len(stdout),stderr_bytes=len(stderr))
 return result,stdout,stderr
def main():
 require(len(sys.argv)==2 and len(sys.argv[1])==64 and all(c in '0123456789abcdef' for c in sys.argv[1]),'supply independently reviewed runner SHA-256')
 reviewed=sys.argv[1]
 RUN.mkdir(exist_ok=False)
 receipt={'status':'failed','host_started':False,'runner_sha256':reviewed,'limits':'Synthetic original fragment and model only; no live game. MXCSR sticky flags recorded but not compared; pinning cannot exclude transient replacement.'}
 try:
  require(sys.platform=='win32' and platform.machine().lower() in ('amd64','x86_64'),'requires 64-bit Windows with SSE2')
  stable_pins(reviewed)
  source=pefile.PE(str(SOURCE));fragment=(PREP/'fragment.bin').read_bytes()
  require(source.get_data(0xe6ae6c-source.OPTIONAL_HEADER.ImageBase,len(fragment))==fragment,'source fragment mismatch')
  host=pefile.PE(str(PREP/'host-x86.exe'))
  require(host.FILE_HEADER.Machine==0x14c,'host is not x86')
  raw=(PREP/'host-x86.exe').read_bytes()
  require(raw.count(fragment)==1 and raw[raw.index(fragment)+len(fragment)]==0xc3,'linked fragment/return mismatch')
  imports={x.dll.lower():{i.name for i in x.imports} for x in host.DIRECTORY_ENTRY_IMPORT}
  require(imports=={b'kernel32.dll':{b'GetStdHandle',b'WriteFile',b'ExitProcess'}},'unexpected host imports')
  model,model_out,model_err=run_child('model',[str(MODEL),str(PREP/'cases.bin')]);receipt['model']=model
  require(model['exit']==0 and not model_err and len(model_out)==96,'model failure')
  receipt['host_started']=True
  native,native_out,native_err=run_child('native',[str(PREP/'host-x86.exe')]);receipt['native']=native
  require(native['exit']==0 and not native_err and len(native_out)==120,'native host failure')
  model_rows=[model_out[i:i+16] for i in range(0,96,16)]
  native_rows=[native_out[i:i+20] for i in range(0,120,20)]
  mxcsr=[struct.unpack_from('<I',row,16)[0] for row in native_rows]
  receipt['mxcsr']=[f'{v:08x}' for v in mxcsr]
  require(all((v & ~0x3f)==0x1f80 for v in mxcsr),'native MXCSR control mismatch')
  receipt['matches']=[a==b[:16] for a,b in zip(model_rows,native_rows)]
  require(all(receipt['matches']),'model/native mismatch')
  stable_pins(reviewed)
  receipt['status']='passed'
 except BaseException as e:
  receipt['error']=str(e);receipt['traceback']=traceback.format_exc()
 finally:
  data=(json.dumps(receipt,indent=2)+'\n').encode()
  with (RUN/'receipt.json').open('xb') as f:f.write(data);f.flush();os.fsync(f.fileno())
 print(json.dumps(receipt,indent=2))
 return 0 if receipt['status']=='passed' else 1
if __name__=='__main__':sys.exit(main())

"""Review candidate: no source-fragment execution until independently reviewed."""
import hashlib,json,os,platform,struct,subprocess,sys,traceback
from pathlib import Path
import pefile
ROOT=Path(__file__).resolve().parents[1];PREP=ROOT/'target/cell-feature-pair-prep-v2';RUN=ROOT/'target/cell-feature-pair-run-v1'
SOURCE=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');MODEL=ROOT/'target/debug/examples/cell_feature_pair_model_v1.exe'
PINS={SOURCE:'17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a',PREP/'manifest.json':'15f94901b3aafea8a58be9bd9a20c3143832bb7843010a3f88a0d487840d1301',PREP/'fragment.bin':'e1c99e797b887d15fe3b2cc6c6d9f93684c85554a062d31bb83318d84a250cde',PREP/'cases.bin':'7022c9c7ca560ac61277259285f0b92cbc28327d4a2fe12c286c988b53e37821',PREP/'host-x86.asm':'490dec072e40a7139fa4a221882d4b96e1760a10962ff05be1adbfe9292557ec',PREP/'host-x86.exe':'e7fdda118e28103ed1950df69d5a50a738e8993af8a4abcc551ce0f4c2da98c5',ROOT/'crates/spore-bevy/src/cell_source_feature_geometry_v1.rs':'ad980714263d2213e9830108fafd61ff995d5ac8281c654651e6e86797dc2280',ROOT/'crates/spore-core/src/native_sse.rs':'eedac9f108e411f55abe01b1b05d5d690f950a954c16409b2222f15f00298129',ROOT/'crates/spore-bevy/examples/cell_feature_pair_model_v1.rs':'d3f4ecc7496c78acbddc335540481ee5327608becd59079bc3da6f13dc7f9230',MODEL:'6770c4e1e65bdedaa4d862fbf7955d4c24d12147f55c8ec9d9e495d4a847af04'}
digest=lambda b:hashlib.sha256(b).hexdigest()
def require(ok,msg):
 if not ok:raise RuntimeError(msg)
def pins(reviewed):
 for p,h in {**PINS,Path(__file__).resolve():reviewed}.items():
  for q in (p,*p.parents):require(not(q.lstat().st_file_attributes & 0x400),f'reparse: {q}')
  require(digest(p.read_bytes())==h,f'pin mismatch: {p}')
def child(name,args):
 try:
  p=subprocess.run(args,capture_output=True,timeout=8,creationflags=subprocess.CREATE_NO_WINDOW,check=False);out,err=p.stdout,p.stderr;rec={'exit':p.returncode,'timeout':False}
 except subprocess.TimeoutExpired as e:out,err=e.stdout or b'',e.stderr or b'';rec={'exit':None,'timeout':True}
 (RUN/(name+'.stdout.bin')).write_bytes(out);(RUN/(name+'.stderr.bin')).write_bytes(err)
 rec.update(stdout_sha256=digest(out),stderr_sha256=digest(err),stdout_bytes=len(out),stderr_bytes=len(err));return rec,out,err
def main():
 require(len(sys.argv)==2 and len(sys.argv[1])==64 and all(c in '0123456789abcdef' for c in sys.argv[1]),'supply reviewed runner SHA-256');reviewed=sys.argv[1];RUN.mkdir(exist_ok=False)
 rec={'status':'failed','host_started':False,'runner_sha256':reviewed,'limits':'Six synthetic cases; no live actor/contact; sticky flags are recorded but not compared; hashes cannot exclude transient replacement.'}
 try:
  require(sys.platform=='win32' and platform.machine().lower() in ('amd64','x86_64'),'64-bit Windows/SSE2 required');pins(reviewed)
  fragment=(PREP/'fragment.bin').read_bytes();source=pefile.PE(str(SOURCE));require(source.get_data(0xe6af72-source.OPTIONAL_HEADER.ImageBase,len(fragment))==fragment,'source bytes differ')
  host=pefile.PE(str(PREP/'host-x86.exe'));require(host.FILE_HEADER.Machine==0x14c,'not x86');raw=(PREP/'host-x86.exe').read_bytes()
  require(raw.count(fragment[:0x101])==1,'fragment prefix not unique');off=raw.index(fragment[:0x101]);linked=raw[off:off+282]
  require(linked[:0x101]==fragment[:0x101] and linked[0x105:]==fragment[0x105:] and raw[off+282]==0xc3,'patched bytes differ')
  target=struct.unpack_from('<I',linked,0x101)[0];require(host.get_data(target-host.OPTIONAL_HEADER.ImageBase,4)==bytes.fromhex('00008037'),'threshold relocation mismatch')
  imports={x.dll.lower():{i.name for i in x.imports} for x in host.DIRECTORY_ENTRY_IMPORT};require(imports=={b'kernel32.dll':{b'GetStdHandle',b'WriteFile',b'ExitProcess'}},'unexpected imports')
  model,mo,me=child('model',[str(MODEL),str(PREP/'cases.bin')]);rec['model']=model;require(model['exit']==0 and not me and len(mo)==240,'model failure')
  rec['host_started']=True;native,no,ne=child('native',[str(PREP/'host-x86.exe')]);rec['native']=native;require(native['exit']==0 and not ne and len(no)==264,'native host failure')
  mr=list(struct.iter_unpack('<10I',mo));nr=list(struct.iter_unpack('<11I',no));rec['mxcsr']=[f'{r[10]:08x}' for r in nr];rec['eflags']=[f'{r[9]:08x}' for r in nr]
  require(all((r[10]&~0x3f)==0x1f80 for r in nr),'MXCSR controls differ')
  rec['matches']=[tuple(a[:9])==tuple(b[:9]) and a[9]==int((b[9]&0x41)==0) for a,b in zip(mr,nr)]
  require(all(rec['matches']),'model/native mismatch');pins(reviewed);rec['status']='passed'
 except BaseException as e:rec['error']=str(e);rec['traceback']=traceback.format_exc()
 finally:
  data=(json.dumps(rec,indent=2)+'\n').encode()
  with (RUN/'receipt.json').open('xb') as f:f.write(data);f.flush();os.fsync(f.fileno())
 print(json.dumps(rec,indent=2));return 0 if rec['status']=='passed' else 1
if __name__=='__main__':sys.exit(main())

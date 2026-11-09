"""Review candidate: no original-fragment execution until independently reviewed."""
import hashlib,json,os,platform,struct,subprocess,sys,traceback
from pathlib import Path
import pefile
ROOT=Path(__file__).resolve().parents[1];PREP=ROOT/'target/cell-contact-geometry-prep-v2';RUN=ROOT/'target/cell-contact-geometry-run-v1'
SOURCE=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');MODEL=ROOT/'target/debug/examples/cell_contact_geometry_model_v1.exe'
PINS={SOURCE:'17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a',PREP/'manifest.json':'d0377b75a98c68f63ccd27d2fa10d435dbfa36f8751ea390d4d75c12c3335e52',PREP/'fragment.bin':'c16003e09ec659ab4f2d729301fa51a4dbd5ce9cfa90756776149069e8f5ef74',PREP/'cases.bin':'6136a3e8dff2a3c218c85b588058e89786cc9f6bef536b6531b45b541ee35bdb',PREP/'host-x86.asm':'4c281cb872a1a11e4241977b952829fc8c71b310c5636a80b970a45ff2e928be',PREP/'host-x86.exe':'a64524d8a01b3ad7b3cdd8fd6db5db34af9ab11f45fb7568e2a0e201e9ba1e14',ROOT/'crates/spore-bevy/src/cell_source_feature_geometry_v1.rs':'04bebc36266ba420f700c5004c2ebcaac070e17c7d7908ca794ada6e5b80f3aa',ROOT/'crates/spore-core/src/native_x87.rs':'08e124429b48ae07a3c9806ff804033ecb20447a3f68a9918ade524209fc1ba0',ROOT/'crates/spore-core/src/native_sse.rs':'eedac9f108e411f55abe01b1b05d5d690f950a954c16409b2222f15f00298129',ROOT/'crates/spore-bevy/examples/cell_contact_geometry_model_v1.rs':'64a5162eca75d39c43c31cad4abca1a276e8e3b8d9f5f0b6f4e29bce7e9e5d35',MODEL:'09b2f933ff6f29f303dce0f68fa7e86125fd5af953eade8329fb6014da4faeca'}
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
 rec={'status':'failed','host_started':False,'runner_sha256':reviewed,'limits':'Six synthetic cases, no live actor/contact; sticky status recorded but not compared; hashes cannot exclude transient replacement.'}
 try:
  require(sys.platform=='win32' and platform.machine().lower() in ('amd64','x86_64'),'64-bit Windows/SSE2 required');pins(reviewed)
  frag=(PREP/'fragment.bin').read_bytes();src=pefile.PE(str(SOURCE));require(src.get_data(0xe6b099-src.OPTIONAL_HEADER.ImageBase,len(frag))==frag,'source fragment differs')
  host=pefile.PE(str(PREP/'host-x86.exe'));require(host.FILE_HEADER.Machine==0x14c,'host not x86');raw=(PREP/'host-x86.exe').read_bytes();require(raw.count(frag[:30])==1,'fragment prefix not unique')
  off=raw.index(frag[:30]);linked=raw[off:off+184];patch=set(range(0x1e,0x22))|set(range(0x26,0x2a))|set(range(0x8e,0x92));require(all(linked[i]==frag[i] for i in range(184) if i not in patch),'nonpatch bytes differ')
  base=host.OPTIONAL_HEADER.ImageBase;rva=host.get_rva_from_offset(off);branch=base+rva+0x22+struct.unpack_from('<i',linked,0x1e)[0]
  require(host.get_data(branch-base,6)==bytes.fromhex('b801000000c3') and raw[off+184:off+187]==bytes.fromhex('33c0c3'),'trampoline mismatch')
  for pos,expected in [(0x26,bytes.fromhex('0000803f')),(0x8e,bytes.fromhex('0000003f'))]:
   va=struct.unpack_from('<I',linked,pos)[0];require(host.get_data(va-base,4)==expected,'constant relocation mismatch')
  imports={x.dll.lower():{i.name for i in x.imports} for x in host.DIRECTORY_ENTRY_IMPORT};require(imports=={b'kernel32.dll':{b'GetStdHandle',b'WriteFile',b'ExitProcess'}},'imports differ')
  model,mo,me=child('model',[str(MODEL),str(PREP/'cases.bin')]);rec['model']=model;require(model['exit']==0 and not me and len(mo)==216,'model failure')
  rec['host_started']=True;native,no,ne=child('native',[str(PREP/'host-x86.exe')]);rec['native']=native;require(native['exit']==0 and not ne and len(no)==264,'native host failure')
  mr=list(struct.iter_unpack('<9I',mo));nr=list(struct.iter_unpack('<11I',no));rec['x87_control_status']=[f'{r[9]:08x}' for r in nr];rec['mxcsr']=[f'{r[10]:08x}' for r in nr]
  require(all((r[9]&0xffff)==0x027f and (r[10]&~0x3f)==0x1f80 for r in nr),'floating control mismatch')
  rec['matches']=[tuple(a)==tuple(b[:9]) for a,b in zip(mr,nr)];require(all(rec['matches']),'model/native mismatch');pins(reviewed);rec['status']='passed'
 except BaseException as e:rec['error']=str(e);rec['traceback']=traceback.format_exc()
 finally:
  data=(json.dumps(rec,indent=2)+'\n').encode()
  with (RUN/'receipt.json').open('xb') as f:f.write(data);f.flush();os.fsync(f.fileno())
 print(json.dumps(rec,indent=2));return 0 if rec['status']=='passed' else 1
if __name__=='__main__':sys.exit(main())

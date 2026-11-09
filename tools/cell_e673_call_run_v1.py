"""Review candidate: no original-fragment execution until independently reviewed."""
import hashlib,json,os,platform,struct,subprocess,sys,traceback
from pathlib import Path
import pefile
ROOT=Path(__file__).resolve().parents[1];PREP=ROOT/'target/cell-e673-call-prep-v1';RUN=ROOT/'target/cell-e673-call-run-v1'
SOURCE=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');MODEL=ROOT/'target/debug/examples/cell_e673_call_model_v1.exe'
PINS={SOURCE:'17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a',PREP/'manifest.json':'6f79656a8536a4222a89c9eb5ca0bb82d1e5eb34508afb3708336f341fe10ccf',PREP/'fragment.bin':'0ce625c52a945160508c11f0c76ebde1678592b8d4a11d63b3b54b313f39d6c6',PREP/'cases.bin':'522713989401fd2d3c068200bc34dd47a1638e6fcab422d94c2fbf925d0d8125',PREP/'host-x86.asm':'96a3530836cdcb8b6fd376b08ec1c39711cf9a1dbe93661f82b68e8b8580c3e7',PREP/'host-x86.exe':'1b94d9bd4e6adf183fbfd19931dc46e35978bda4cbe170c546df72c22cb25a5a',ROOT/'crates/spore-bevy/src/cell_source_contact_call_v1.rs':'64e3bde3fce207c317b1e88d50172ebb4a17c0430f6c8cb068e2f048bfea1fce',ROOT/'crates/spore-core/src/native_x87.rs':'08e124429b48ae07a3c9806ff804033ecb20447a3f68a9918ade524209fc1ba0',ROOT/'crates/spore-bevy/examples/cell_e673_call_model_v1.rs':'473c6eeb1535249ce3155eedc9d753ed1f3f755ea0ea26edb50060b3de93e786',MODEL:'4461f0240675c97d9a7be93dace74fa242d1476d1f71a2439a6b4b2e826d2ecd'}
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
 rec={'status':'failed','host_started':False,'runner_sha256':reviewed,'limits':'Six synthetic cases; no E673A0 or gameplay; sticky status recorded not compared; hashes cannot exclude transient replacement.'}
 try:
  require(sys.platform=='win32' and platform.machine().lower() in ('amd64','x86_64'),'64-bit Windows/SSE2 required');pins(reviewed)
  frag=(PREP/'fragment.bin').read_bytes();src=pefile.PE(str(SOURCE));require(src.get_data(0xe6a548-src.OPTIONAL_HEADER.ImageBase,len(frag))==frag,'source bytes differ')
  host=pefile.PE(str(PREP/'host-x86.exe'));require(host.FILE_HEADER.Machine==0x14c,'not x86');raw=(PREP/'host-x86.exe').read_bytes();require(raw.count(frag[:7])==1,'fragment prefix not unique');off=raw.index(frag[:7]);linked=raw[off:off+75]
  require(linked[:7]==frag[:7] and linked[11:]==frag[11:],'nonpatch source bytes differ')
  ptr=struct.unpack_from('<I',linked,7)[0];base=host.OPTIONAL_HEADER.ImageBase;root=struct.unpack('<I',host.get_data(ptr-base,4))[0];state=struct.unpack('<I',host.get_data(root+0x5190-base,4))[0]
  require(host.get_data(state+0x74-base,4)==struct.pack('<I',0x12345678),'synthetic global relocation mismatch')
  imports={x.dll.lower():{i.name for i in x.imports} for x in host.DIRECTORY_ENTRY_IMPORT};require(imports=={b'kernel32.dll':{b'GetStdHandle',b'WriteFile',b'ExitProcess'}},'imports differ')
  model,mo,me=child('model',[str(MODEL),str(PREP/'cases.bin')]);rec['model']=model;require(model['exit']==0 and not me and len(mo)==24,'model failure')
  rec['host_started']=True;native,no,ne=child('native',[str(PREP/'host-x86.exe')]);rec['native']=native;require(native['exit']==0 and not ne and len(no)==168,'native host failure')
  mr=list(struct.iter_unpack('<I',mo));nr=list(struct.iter_unpack('<7I',no));rec['x87_control_status']=[f'{r[5]:08x}' for r in nr];rec['mxcsr']=[f'{r[6]:08x}' for r in nr]
  require(all((r[5]&0xffff)==0x027f and ((r[5]>>16)&0x3800)==0 and (r[6]&~0x3f)==0x1f80 for r in nr),'floating control or x87 TOP mismatch')
  require(all(r[1:5]==(0x10000,0x20001,0x12345678,1) for r in nr),'handoff identity/current mismatch')
  rec['matches']=[a[0]==b[0] for a,b in zip(mr,nr)];require(all(rec['matches']),'model/native mismatch');pins(reviewed);rec['status']='passed'
 except BaseException as e:rec['error']=str(e);rec['traceback']=traceback.format_exc()
 finally:
  data=(json.dumps(rec,indent=2)+'\n').encode()
  with (RUN/'receipt.json').open('xb') as f:f.write(data);f.flush();os.fsync(f.fileno())
 print(json.dumps(rec,indent=2));return 0 if rec['status']=='passed' else 1
if __name__=='__main__':sys.exit(main())

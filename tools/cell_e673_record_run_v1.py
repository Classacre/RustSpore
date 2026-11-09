"""Review candidate: do not execute isolated original slices before review."""
import hashlib,json,os,platform,struct,subprocess,sys,traceback
from pathlib import Path
import pefile
ROOT=Path(__file__).resolve().parents[1];PREP=ROOT/'target/cell-e673-record-prep-v2';RUN=ROOT/'target/cell-e673-record-run-v1'
SOURCE=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');MODEL=ROOT/'target/debug/examples/cell_e673_record_model_v1.exe'
PINS={SOURCE:'17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a',PREP/'manifest.json':'a4fe5962d04e5eb71d47d2351f60b315f1d81a66b4c2a9ef3a79c721b0f0c6cd',PREP/'cases.bin':'22b989d1b375297a75ad4d5a64dbeb2d0fe5984b536dafe4acc9486a05f56787',PREP/'host-x86.asm':'b201f114fc51c98c875711af305a8e0fb13ee7ee350a125c89d323bd895345d1',PREP/'host-x86.exe':'d3b137c28d243d1edae8b5fde1d876ac5ec502ba7b96f3dfc402931db69a189f',PREP/'fragment0.bin':'233dbf636987d15096f54508905e9569f4bd48990ef6063a40dfebd9c673e1f9',PREP/'fragment1.bin':'eb5237e9a61ef8c9bb4bdfefa8f1396104a56dd75c52cb2fb154888bcb01084a',PREP/'fragment2.bin':'325a2d2cc8273841f710514f209a19baa41f981f2ee87622d94771aeb34d2684',PREP/'fragment3.bin':'fe80d46b69fdf42b835139dfbc5ccfd3120aa9de05cd2944f83ce75ba823199b',ROOT/'crates/spore-bevy/src/cell_source_contact_record_v1.rs':'c1749a45b37883fab6117c5debac027706e2edbee87f458c18f7fe6f6d13d271',ROOT/'crates/spore-core/src/native_sse.rs':'eedac9f108e411f55abe01b1b05d5d690f950a954c16409b2222f15f00298129',ROOT/'crates/spore-bevy/examples/cell_e673_record_model_v1.rs':'51865ba9bf0a74a1337bafd0cf2206874af013c61e6a601a4dec40fb8615c592',MODEL:'5b92f7c34f9b24158e6b39de8e4af613c314d09d89661bf7eb310eec418ea548'}
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
 rec={'status':'failed','host_started':False,'runner_sha256':reviewed,'limits':'Separate fixed synthetic slices; no provider, E65810, callback or live actor/contact. Sticky flags not compared; hashes cannot exclude transient replacement.'}
 try:
  require(sys.platform=='win32' and platform.machine().lower() in ('amd64','x86_64'),'64-bit Windows/SSE2 required');pins(reviewed)
  src=pefile.PE(str(SOURCE));host=pefile.PE(str(PREP/'host-x86.exe'));require(host.FILE_HEADER.Machine==0x14c,'not x86')
  ranges=[(0xe616c0,0x401662),(0xe674dd,0x4016af),(0xe674fe,0x4016b3),(0xe67524,0x4016c3)]
  for i,(source_va,host_va) in enumerate(ranges):
   f=(PREP/f'fragment{i}.bin').read_bytes();require(src.get_data(source_va-src.OPTIONAL_HEADER.ImageBase,len(f))==f,f'source fragment {i} differs');require(host.get_data(host_va-host.OPTIONAL_HEADER.ImageBase,len(f))==f,f'host fragment {i} differs')
   if i:require(host.get_data(host_va-host.OPTIONAL_HEADER.ImageBase+len(f),1)==b'\xc3',f'host fragment {i} return differs')
  imports={x.dll.lower():{i.name for i in x.imports} for x in host.DIRECTORY_ENTRY_IMPORT};require(imports=={b'kernel32.dll':{b'GetStdHandle',b'WriteFile',b'ExitProcess'}},'imports differ')
  model,mo,me=child('model',[str(MODEL),str(PREP/'cases.bin')]);rec['model']=model;require(model['exit']==0 and not me and len(mo)==244,'model failure')
  rec['host_started']=True;native,no,ne=child('native',[str(PREP/'host-x86.exe')]);rec['native']=native;require(native['exit']==0 and not ne and len(no)==260,'native host failure')
  mx=[struct.unpack_from('<I',no,68+i*48+44)[0] for i in range(4)];rec['mxcsr']=[f'{x:08x}' for x in mx];require(all((x&~0x3f)==0x1f80 for x in mx),'MXCSR controls differ')
  projected=no[:68]+b''.join(no[68+i*48:68+i*48+44] for i in range(4));rec['matches']={'lookups':mo[:24]==projected[:24],'existing':mo[24:68]==projected[24:68],'new':[mo[68+i*44:68+(i+1)*44]==projected[68+i*44:68+(i+1)*44] for i in range(4)]}
  require(rec['matches']['lookups'] and rec['matches']['existing'] and all(rec['matches']['new']),'model/native mismatch');pins(reviewed);rec['status']='passed'
 except BaseException as e:rec['error']=str(e);rec['traceback']=traceback.format_exc()
 finally:
  data=(json.dumps(rec,indent=2)+'\n').encode()
  with (RUN/'receipt.json').open('xb') as f:f.write(data);f.flush();os.fsync(f.fileno())
 print(json.dumps(rec,indent=2));return 0 if rec['status']=='passed' else 1
if __name__=='__main__':sys.exit(main())

"""Review gate for E69B00 second-result original-instruction host."""
from pathlib import Path
import hashlib,json,struct,subprocess,sys,platform,re,pefile
R=Path(__file__).resolve().parents[1];P=R/'target/cell-e69b-second-prep-v1';O=R/'target/cell-e69b-second-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');SH='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';MH='26744a9f212b6ed9863c1bcc603ee1e3a92eb1d67211ea95e0a3e918aee9d952';h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True)
for n in ('model','native'):
 for st in ('stdout','stderr'):(O/(n+'.'+st+'.bin')).write_bytes(b'')
receipt={'status':'failed','platform':platform.platform(),'checks':[],'runs':{}};pins=[]
try:
 if sys.platform!='win32' or platform.machine().lower() not in ('amd64','x86_64'):raise ValueError('Windows x64 required')
 mb=(P/'manifest.json').read_bytes()
 if h(mb)!=MH:raise ValueError('manifest mismatch')
 manifest=json.loads(mb);pins=[(Path(k),v) for k,v in manifest['pins'].items()]+[(P/'manifest.json',MH),(S,SH)]
 for path,pin in pins:
  value=h(path.read_bytes());receipt['checks'].append({'path':str(path),'sha256':value,'expected':pin})
  if value!=pin:raise ValueError('pin mismatch '+str(path))
 src=pefile.PE(str(S),fast_load=True);f=src.get_data(0xe69b85-src.OPTIONAL_HEADER.ImageBase,32)
 if f!=(P/'fragment.bin').read_bytes():raise ValueError('source bytes mismatch')
 mt=(P/'host-x86.map').read_text();v=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+SourceFragment\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
 if len(v)!=1 or int(v[0],16)!=0x4013b0:raise ValueError('source symbol mismatch')
 host=pefile.PE(str(P/'host-x86.exe'),fast_load=True)
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host base')
 b=host.get_data(0x13b0,41)
 if len(b)!=41 or any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in range(6,10)):raise ValueError('nonpatch byte mismatch')
 if 0x4013b0+10+struct.unpack_from('<i',b,6)[0]!=0x4013d3:raise ValueError('branch target mismatch')
 if b[32:]!=bytes.fromhex('33c0c3b801000000c3'):raise ValueError('exit mismatch')
 receipt['linked_slice']={'nonpatch_match':True,'target_match':True,'synthetic_exits_match':True}
 corpus=(P/'cases.bin').read_bytes();model=R/'target/debug/examples/cell_e69b_second_oracle_v1.exe'
 for name,cmd,inp in [('model',[str(model)],corpus),('native',[str(P/'host-x86.exe')],None)]:
  try:
   q=subprocess.run(cmd,input=inp,capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW);out,err,code=q.stdout,q.stderr,q.returncode
  except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
  (O/(name+'.stdout.bin')).write_bytes(out);(O/(name+'.stderr.bin')).write_bytes(err);receipt['runs'][name]={'exit':code,'stdout_bytes':len(out),'stderr_bytes':len(err),'stdout_sha256':h(out)}
  if code!=0:raise ValueError(name+' failed')
 model=(O/'model.stdout.bin').read_bytes();native=(O/'native.stdout.bin').read_bytes()
 if len(model)!=64 or len(native)!=96:raise ValueError('output size')
 receipt['rows']=[]
 for i in range(4):
  a=struct.unpack_from('<4I',model,i*16);b=struct.unpack_from('<6I',native,i*24);cw=b[4]&0xffff;sw=b[4]>>16;mx=b[5]
  receipt['rows'].append({'case':i,'model':a,'native':b[:4],'control':hex(cw),'status':hex(sw),'mxcsr':hex(mx)})
  if a!=b[:4] or cw!=0x27f or sw&0x3800 or mx&~0x3f!=0x1f80:raise ValueError('row/control mismatch '+str(i))
 receipt['status']='matched'
except Exception as e:receipt['error']=repr(e)
finally:
 receipt['post_checks']=[]
 for path,pin in pins:
  try:
   value=h(path.read_bytes());receipt['post_checks'].append({'path':str(path),'sha256':value,'expected':pin})
   if value!=pin:receipt['status']='failed';receipt['post_error']='changed pin '+str(path)
  except Exception as e:receipt['status']='failed';receipt['post_error']=repr(e)
 (O/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt,indent=2));sys.exit(receipt['status']!='matched')

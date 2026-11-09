"""Review gate for finite, disjoint 537EE0 fallback comparison."""
from pathlib import Path
import hashlib,json,struct,subprocess,sys,platform,re,pefile,capstone
R=Path(__file__).resolve().parents[1];P=R/'target/cell-e680-fallback-prep-v1';O=R/'target/cell-e680-fallback-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');SH='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';MH='39e4f0cad2d9cd915dfbcf6153b763d75d2c1810204bcb4c66aa98b486caf580';h=lambda b:hashlib.sha256(b).hexdigest()
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
  v=h(path.read_bytes());receipt['checks'].append({'path':str(path),'sha256':v,'expected':pin})
  if v!=pin:raise ValueError('pin mismatch '+str(path))
 spans=[(0x537ee0,0x537fb7,'Fallback',0x40114e),(0x41dbd0,0x41dc28,'VectorCompare',0x401225),(0x41dc30,0x41dc4f,'MatrixCompare',0x40127d),(0x422d90,0x422dfc,'RowsCompare',0x40129c),(0x422e00,0x422e58,'RowCompare',0x401308)]
 patches={0:[(0x537f07,0x401225),(0x537f44,0x40127d)],2:[(0x41dc3b,0x40129c)],3:[(0x422d9c,0x401308),(0x422db9,0x401308),(0x422dd6,0x401308)]}
 src=pefile.PE(str(S),fast_load=True);host=pefile.PE(str(P/'host-x86.exe'),fast_load=True);mt=(P/'host-x86.map').read_text()
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host base')
 dis=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);receipt['linked_functions']=[]
 for j,(a,z,name,va) in enumerate(spans):
  f=src.get_data(a-src.OPTIONAL_HEADER.ImageBase,z-a);b=host.get_data(va-host.OPTIONAL_HEADER.ImageBase,z-a)
  if f!=(P/f'fragment-{j}.bin').read_bytes() or len(b)!=len(f):raise ValueError('source length mismatch')
  v=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+name+r'\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
  if len(v)!=1 or int(v[0],16)!=va:raise ValueError('symbol mismatch '+name)
  ins=list(dis.disasm(f,a));tail=b'\xc2\x04\x00' if j==0 else b'\xc3'
  if not ins or sum(i.size for i in ins)!=len(f) or ins[-1].address+ins[-1].size!=z or bytes(ins[-1].bytes)!=tail:raise ValueError('incomplete return '+name)
  mask=set()
  for address,target in patches.get(j,[]):
   off=address-a;mask.update(range(off+1,off+5))
   if f[off]!=0xe8 or b[off]!=0xe8 or va+off+5+struct.unpack_from('<i',b,off+1)[0]!=target:raise ValueError('call target mismatch')
  if any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in mask):raise ValueError('nonpatch byte mismatch '+name)
  receipt['linked_functions'].append({'name':name,'source_start':hex(a),'length':len(f),'complete_ret':tail.hex(),'matched':True})
 corpus=(P/'cases.bin').read_bytes();model=R/'target/debug/examples/cell_e680_fallback_oracle_v1.exe'
 for name,cmd,inp in [('model',[str(model)],corpus),('native',[str(P/'host-x86.exe')],None)]:
  try:
   q=subprocess.run(cmd,input=inp,capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW);out,err,code=q.stdout,q.stderr,q.returncode
  except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
  (O/(name+'.stdout.bin')).write_bytes(out);(O/(name+'.stderr.bin')).write_bytes(err);receipt['runs'][name]={'exit':code,'stdout_bytes':len(out),'stderr_bytes':len(err),'stdout_sha256':h(out)}
  if code!=0:raise ValueError(name+' failed')
 model=(O/'model.stdout.bin').read_bytes();native=(O/'native.stdout.bin').read_bytes()
 if len(model)!=560 or len(native)!=640:raise ValueError('output size')
 receipt['rows']=[]
 for i in range(10):
  a=model[i*56:(i+1)*56];b=native[i*64:i*64+56];cw,sw,mx=struct.unpack_from('<HHI',native,i*64+56)
  receipt['rows'].append({'case':i,'match':a==b,'control':hex(cw),'status':hex(sw),'mxcsr':hex(mx)})
  if a!=b or cw!=0x27f or sw&0x3800 or mx&~0x3f!=0x1f80:raise ValueError('row/control mismatch '+str(i))
 receipt['status']='matched'
except Exception as e:receipt['error']=repr(e)
finally:
 receipt['post_checks']=[]
 for path,pin in pins:
  try:
   v=h(path.read_bytes());receipt['post_checks'].append({'path':str(path),'sha256':v,'expected':pin})
   if v!=pin:receipt['status']='failed';receipt['post_error']='changed pin '+str(path)
  except Exception as e:receipt['status']='failed';receipt['post_error']=repr(e)
 (O/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt,indent=2));sys.exit(receipt['status']!='matched')

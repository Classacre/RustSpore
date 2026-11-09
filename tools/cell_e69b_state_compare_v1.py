"""Review gate: no native execution until host/ABI/stubs independently reviewed."""
from pathlib import Path
import hashlib,json,struct,subprocess,sys,platform,re,pefile
R=Path(__file__).resolve().parents[1];P=R/'target/cell-e69b-state-prep-v3';O=R/'target/cell-e69b-state-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
SH='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';MH='5c5e5721f1cbbca18f98d5db25e8cae9ad60c5af09b81acd0fc3d9cd4aac2628';h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True)
for n in ('model','native'):
 for st in ('stdout','stderr'):(O/(n+'.'+st+'.bin')).write_bytes(b'')
receipt={'status':'failed','platform':platform.platform(),'checks':[],'runs':{}};pins=[]
try:
 if sys.platform!='win32' or platform.machine().lower() not in ('amd64','x86_64'):raise ValueError('Windows x64 required')
 mb=(P/'manifest.json').read_bytes()
 if h(mb)!=MH:raise ValueError('manifest hash mismatch')
 manifest=json.loads(mb);pins=[(Path(k),v) for k,v in manifest['pins'].items()]+[(P/'manifest.json',MH),(S,SH)]
 for path,pin in pins:
  value=h(path.read_bytes());receipt['checks'].append({'path':str(path),'sha256':value,'expected':pin})
  if value!=pin:raise ValueError('pin mismatch '+str(path))
 src=pefile.PE(str(S),fast_load=True);f=src.get_data(0xe69b2d-src.OPTIONAL_HEADER.ImageBase,68);table=src.get_data(0xe69c4c-src.OPTIONAL_HEADER.ImageBase,52)
 if f!=(P/'fragment.bin').read_bytes() or table!=(P/'table.bin').read_bytes():raise ValueError('source fragment/table mismatch')
 mt=(P/'host-x86.map').read_text();symbols={'OnWrite':0x401156,'SourceFragment':0x401185,'SourceTable':0x4031a2,'Targets':0x4031d6}
 for name,va in symbols.items():
  v=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+name+r'\s+([0-9A-Fa-f]{8})\s+(?:f\s+)?host-x86\.obj\s*$',mt,re.M)
  if len(v)!=1 or int(v[0],16)!=va:raise ValueError('symbol mismatch '+name)
 host=pefile.PE(str(P/'host-x86.exe'),fast_load=True)
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('image base mismatch')
 b=host.get_data(0x1185,70);mask=set(range(0x29,0x2d))|set(range(0x30,0x34))|set(range(0x3d,0x41))
 if len(b)!=70 or any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in mask):raise ValueError('linked source byte mismatch')
 if b[68:]!=b'\x5b\xc3':raise ValueError('synthetic exit mismatch')
 if struct.unpack_from('<I',b,0x29)[0]!=symbols['SourceTable'] or struct.unpack_from('<I',b,0x30)[0]!=symbols['Targets']:raise ValueError('table operand mismatch')
 if 0x401185+0x41+struct.unpack_from('<i',b,0x3d)[0]!=symbols['OnWrite']:raise ValueError('callback target mismatch')
 if host.get_data(0x31a2,52)!=table:raise ValueError('linked table mismatch')
 if struct.unpack('<2I',host.get_data(0x31d6,8))!=(0x4011b9,0x4011c9):raise ValueError('linked table targets mismatch')
 receipt['linked_slice']={'nonpatch_match':True,'table_match':True,'operands_match':True,'exit_match':True}
 corpus=(P/'cases.bin').read_bytes();model=R/'target/debug/examples/cell_e69b_state_oracle_v1.exe'
 for name,cmd,inp in [('model',[str(model)],corpus),('native',[str(P/'host-x86.exe')],None)]:
  try:
   q=subprocess.run(cmd,input=inp,capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW);out,err,code=q.stdout,q.stderr,q.returncode
  except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
  (O/(name+'.stdout.bin')).write_bytes(out);(O/(name+'.stderr.bin')).write_bytes(err);receipt['runs'][name]={'exit':code,'stdout_bytes':len(out),'stderr_bytes':len(err),'stdout_sha256':h(out)}
  if code!=0:raise ValueError(name+' failed')
 model=(O/'model.stdout.bin').read_bytes();native=(O/'native.stdout.bin').read_bytes()
 if len(model)!=440 or len(native)!=880:raise ValueError('output size')
 receipt['rows']=[]
 for i in range(55):
  a=struct.unpack_from('<2I',model,i*8);b=struct.unpack_from('<4I',native,i*16);cw=b[2]&0xffff;sw=b[2]>>16;mx=b[3]
  receipt['rows'].append({'case':i,'match':a==b[:2],'control':hex(cw),'status':hex(sw),'mxcsr':hex(mx)})
  if a!=b[:2] or cw!=0x27f or sw&0x3800 or mx&~0x3f!=0x1f80:raise ValueError('row/control mismatch '+str(i))
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

"""Run only after independent review of host slices and admissions."""
from pathlib import Path
import hashlib,json,struct,subprocess,sys,platform,re,pefile
R=Path(__file__).resolve().parents[1];P=R/'target/cell-case10-event-stores-prep-v3';O=R/'target/cell-case10-event-stores-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
SH='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';MH='73ecef47147874dca4a0511021577eea73dec4378d8a4f3822e821c4bf01a3c6'
h=lambda b:hashlib.sha256(b).hexdigest()
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
 spans=[(0xe51d6f,0xe51dc8),(0xe51dd1,0xe51de2),(0xe51ded,0xe51dfe),(0xe51e09,0xe51e32),(0xe51e33,0xe51e3c)]
 if manifest['spans']!=[[hex(a),hex(z-1)] for a,z in spans]:raise ValueError('source span mismatch')
 src=pefile.PE(str(S),fast_load=True);host=pefile.PE(str(P/'host-x86.exe'),fast_load=True)
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host image base')
 mt=(P/'host-x86.map').read_text();vas=[[0x4010dd,0x40113b,0x401151,0x401167,0x401190],[0x401308,0x401366,0x40137c,0x401392,0x4013bb],[0x401533,0x401591,0x4015a7,0x4015bd,0x4015e6],[0x40175e,0x4017bc,0x4017d2,0x4017e8,0x401811]]
 receipt['linked_slices']=[]
 for j,(a,z) in enumerate(spans):
  b=src.get_data(a-src.OPTIONAL_HEADER.ImageBase,z-a)
  if b!=(P/f'fragment-{j}.bin').read_bytes():raise ValueError('source bytes mismatch')
  for i in range(4):
   name=f'Slice{i}_{j}';v=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+name+r'\s+([0-9A-Fa-f]{8})\s+host-x86\.obj\s*$',mt,re.M)
   if len(v)!=1 or int(v[0],16)!=vas[i][j]:raise ValueError('symbol mismatch '+name)
   if host.get_data(vas[i][j]-host.OPTIONAL_HEADER.ImageBase,len(b))!=b:raise ValueError('linked bytes mismatch '+name)
   receipt['linked_slices'].append({'symbol':name,'va':hex(vas[i][j]),'bytes':len(b),'match':True})
 corpus=(P/'cases.bin').read_bytes();model=R/'target/debug/examples/cell_case10_event_stores_oracle_v1.exe'
 for name,cmd,inp in [('model',[str(model)],corpus),('native',[str(P/'host-x86.exe')],None)]:
  try:
   q=subprocess.run(cmd,input=inp,capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW);out,err,code=q.stdout,q.stderr,q.returncode
  except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
  (O/(name+'.stdout.bin')).write_bytes(out);(O/(name+'.stderr.bin')).write_bytes(err)
  receipt['runs'][name]={'exit':code,'stdout_bytes':len(out),'stderr_bytes':len(err),'stdout_sha256':h(out)}
  if code!=0:raise ValueError(name+' failed')
 model=(O/'model.stdout.bin').read_bytes();native=(O/'native.stdout.bin').read_bytes()
 if len(model)!=496 or len(native)!=528:raise ValueError('output size')
 receipt['rows']=[]
 for i in range(4):
  expected=model[i*124:(i+1)*124];actual=native[i*132:i*132+124];cw,sw,mx=struct.unpack_from('<HHI',native,i*132+124)
  receipt['rows'].append({'case':i,'match':expected==actual,'x87_control':hex(cw),'x87_status':hex(sw),'mxcsr':hex(mx)})
  if expected!=actual or cw!=0x27f or sw&0x3800 or mx&~0x3f!=0x1f80:raise ValueError('row/control mismatch '+str(i))
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

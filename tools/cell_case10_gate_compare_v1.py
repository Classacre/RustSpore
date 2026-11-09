"""Run only after independent review of host bytes, stubs and ABI."""
from pathlib import Path
import hashlib,json,struct,subprocess,sys,platform,re,pefile
R=Path(__file__).resolve().parents[1];P=R/'target/cell-case10-gate-prep-v2';O=R/'target/cell-case10-gate-result-v1'
S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');SH='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
MH='a3de29aa445719b7d9e8f52abdf9a3a46e55041fb805d8fc756b2134c20222b4'
h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True)
for n in ['model','native']:
 for st in ['stdout','stderr']:(O/(n+'.'+st+'.bin')).write_bytes(b'')
receipt={'status':'failed','platform':platform.platform(),'checks':[],'runs':{}}
pins=[]
try:
 if sys.platform!='win32' or platform.machine().lower() not in ('amd64','x86_64'):raise ValueError('Windows x64 required')
 manifest_bytes=(P/'manifest.json').read_bytes()
 if h(manifest_bytes)!=MH:raise ValueError('manifest pin mismatch')
 manifest=json.loads(manifest_bytes)
 pins=[(Path(k),v) for k,v in manifest['pins'].items()]+[(P/'manifest.json',MH),(S,SH)]
 for path,pin in pins:
  value=h(path.read_bytes());receipt['checks'].append({'path':str(path),'sha256':value,'expected':pin})
  if value!=pin:raise ValueError('pin mismatch: '+str(path))
 if manifest['patches']!=[['0xe7e062','GateTrue'],['0xe7e08b','Relation']]:raise ValueError('patch declaration mismatch')
 source=pefile.PE(str(S),fast_load=True);fragment=source.get_data(0xe7e050-source.OPTIONAL_HEADER.ImageBase,0x47)
 if fragment!=(P/'fragment.bin').read_bytes():raise ValueError('source fragment mismatch')
 mt=(P/'host-x86.map').read_text()
 for n,va in [('SourceFragment',0x401969),('Relation',0x401939)]:
  v=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+n+r'\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
  if len(v)!=1 or int(v[0],16)!=va:raise ValueError('map symbol mismatch: '+n)
 host=pefile.PE(str(P/'host-x86.exe'),fast_load=True)
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('image base mismatch')
 linked=host.get_data(0x1969,0x53)
 if len(linked)!=0x53:raise ValueError('short host fragment')
 masks=set(range(0x14,0x18))|set(range(0x3c,0x40))
 if any(a!=b for i,(a,b) in enumerate(zip(fragment,linked)) if i not in masks):raise ValueError('non-patch byte mismatch')
 if linked[0x47:0x53]!=bytes.fromhex('b0025d5b59c3b0015d5b59c3'):raise ValueError('custom exits mismatch')
 targets=[]
 for off,size,operand,expected in [(0x12,6,2,0x4019b6),(0x3b,5,1,0x401939)]:
  target=0x401969+off+size+struct.unpack_from('<i',linked,off+operand)[0];targets.append(hex(target))
  if target!=expected:raise ValueError('patch target mismatch')
 receipt['linked_fragment']={'non_patch_bytes_match':True,'targets':targets,'synthetic_exits_verified':True}
 corpus=(P/'cases.bin').read_bytes();model=R/'target/debug/examples/cell_case10_gate_oracle_v1.exe'
 for name,cmd,inp in [('model',[str(model)],corpus),('native',[str(P/'host-x86.exe')],None)]:
  try:
   q=subprocess.run(cmd,input=inp,capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW);out,err,code=q.stdout,q.stderr,q.returncode
  except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
  (O/(name+'.stdout.bin')).write_bytes(out);(O/(name+'.stderr.bin')).write_bytes(err)
  receipt['runs'][name]={'exit':code,'stdout_bytes':len(out),'stderr_bytes':len(err),'stdout_sha256':h(out)}
  if code!=0:raise ValueError(name+' failed')
 model=(O/'model.stdout.bin').read_bytes();native=(O/'native.stdout.bin').read_bytes()
 if len(model)!=64 or len(native)!=128:raise ValueError('unexpected output size')
 receipt['rows']=[]
 for i in range(8):
  a=struct.unpack_from('<2I',model,i*8);b=struct.unpack_from('<4I',native,i*16);cw=b[2]&0xffff;sw=b[2]>>16;mx=b[3]
  receipt['rows'].append({'case':i,'model':a,'native':b[:2],'x87_control':hex(cw),'x87_status':hex(sw),'mxcsr':hex(mx)})
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

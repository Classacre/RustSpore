"""Review gate for unmodified BBA9C0 filter host."""
from pathlib import Path
import hashlib,json,struct,subprocess,sys,platform,re,pefile,capstone
R=Path(__file__).resolve().parents[1];P=R/'target/cell-sweep-filter-prep-v1';O=R/'target/cell-sweep-filter-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');SH='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';MH='82d475434b4d70fb3ab6828035ef576f1e347a96b94ba2081dc7eb0ad81d6fd5';h=lambda b:hashlib.sha256(b).hexdigest()
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
 src=pefile.PE(str(S),fast_load=True);f=src.get_data(0xbba9c0-src.OPTIONAL_HEADER.ImageBase,94)
 if f!=(P/'fragment.bin').read_bytes():raise ValueError('source bytes mismatch')
 ins=list(capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32).disasm(f,0xbba9c0))
 if sum(i.size for i in ins)!=94 or ins[-1].address+ins[-1].size!=0xbbaa1e or bytes(ins[-1].bytes)!=b'\xc3':raise ValueError('incomplete source or return')
 mt=(P/'host-x86.map').read_text();v=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+SourceFragment\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
 if len(v)!=1 or int(v[0],16)!=0x401850:raise ValueError('source symbol mismatch')
 host=pefile.PE(str(P/'host-x86.exe'),fast_load=True)
 if host.OPTIONAL_HEADER.ImageBase!=0x400000 or host.get_data(0x1850,94)!=f:raise ValueError('linked source mismatch')
 receipt['linked_function']={'all_bytes_match':True,'complete_instructions':True,'return':'c3'}
 corpus=(P/'cases.bin').read_bytes();model=R/'target/debug/examples/cell_sweep_pair_filter_oracle_v1.exe'
 for name,cmd,inp in [('model',[str(model)],corpus),('native',[str(P/'host-x86.exe')],None)]:
  try:
   q=subprocess.run(cmd,input=inp,capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW);out,err,code=q.stdout,q.stderr,q.returncode
  except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
  (O/(name+'.stdout.bin')).write_bytes(out);(O/(name+'.stderr.bin')).write_bytes(err);receipt['runs'][name]={'exit':code,'stdout_bytes':len(out),'stderr_bytes':len(err),'stdout_sha256':h(out)}
  if code!=0:raise ValueError(name+' failed')
 model=(O/'model.stdout.bin').read_bytes();native=(O/'native.stdout.bin').read_bytes()
 if len(model)!=32 or len(native)!=96:raise ValueError('output size')
 receipt['rows']=[]
 for i in range(8):
  a=struct.unpack_from('<I',model,i*4)[0];b,c,mx=struct.unpack_from('<3I',native,i*12);cw=c&0xffff;sw=c>>16
  receipt['rows'].append({'case':i,'model':a,'native':b,'control':hex(cw),'status':hex(sw),'mxcsr':hex(mx)})
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

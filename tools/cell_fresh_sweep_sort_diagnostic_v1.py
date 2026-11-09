"""Reviewed diagnostic-only fresh-sort phase tracing; no parity claim."""
from pathlib import Path
import hashlib,json,struct,subprocess,sys,platform,re,pefile,capstone
R=Path(__file__).resolve().parents[1];P=R/'target/cell-fresh-sort-diagnostic-prep-v2';O=R/'target/cell-fresh-sort-diagnostic-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');D=Path(r'D:\Games\SPORE Collection\SporeBin\msvcr90.dll');OLD=R/'target/cell-fresh-sort-result-v1/receipt.json';SH='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';DH='ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185';OH='734946b244b104d3302ab1246642d30147af85497797e9d987a7ee74d9def1c4';MH='2e487c3b48aa0b7ec730a497cc8599648069344f302ffc634af4c96b091792a8';h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True)
for st in ('stdout','stderr'):(O/('native.'+st+'.bin')).write_bytes(b'')
receipt={'status':'failed','platform':platform.platform(),'checks':[],'runs':{}};pins=[]
try:
 if sys.platform!='win32' or platform.machine().lower() not in ('amd64','x86_64'):raise ValueError('Windows x64 required')
 mb=(P/'manifest.json').read_bytes()
 if h(mb)!=MH:raise ValueError('manifest mismatch')
 manifest=json.loads(mb);pins=[(Path(k),v) for k,v in manifest['pins'].items()]+[(P/'manifest.json',MH),(S,SH),(D,DH),(OLD,OH)]
 for path,pin in pins:
  v=h(path.read_bytes());receipt['checks'].append({'path':str(path),'sha256':v,'expected':pin})
  if v!=pin:raise ValueError('pin mismatch '+str(path))
 src=pefile.PE(str(S),fast_load=True);host=pefile.PE(str(P/'host-x86.exe'),fast_load=True);mt=(P/'host-x86.map').read_text();dis=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host base')
 functions=[(0xbba7c0,511,'sort.bin','SortSource',0x401a66),(0x11e04f0,43,'probe.bin','StackProbe',0x401c65),(0xbba700,29,'compare.bin','Comparator',0x401c90)]
 receipt['linked_functions']=[]
 for a,n,file,name,va in functions:
  f=src.get_data(a-src.OPTIONAL_HEADER.ImageBase,n);b=host.get_data(va-host.OPTIONAL_HEADER.ImageBase,n)
  if f!=(R/'target/cell-fresh-sort-prep-v2'/file).read_bytes() or len(b)!=n:raise ValueError('source mismatch '+name)
  v=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+name+r'\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
  if len(v)!=1 or int(v[0],16)!=va:raise ValueError('symbol mismatch '+name)
  ins=list(dis.disasm(f,a))
  if sum(i.size for i in ins)!=n or ins[-1].address+ins[-1].size!=a+n:raise ValueError('incomplete instructions '+name)
  if name in ('SortSource','Comparator') and bytes(ins[-1].bytes)!=b'\xc3':raise ValueError('missing RET '+name)
  mask=set()
  if name=='SortSource':
   for off in (6,0x1d,0x27):mask.update(range(off,off+4))
   if va+10+struct.unpack_from('<i',b,6)[0]!=0x401c65:raise ValueError('probe target')
   if struct.unpack_from('<I',b,0x1d)[0]!=0x401c90 or struct.unpack_from('<I',b,0x27)[0]!=0x403076:raise ValueError('comparator/tap pointer target')
  if any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in mask):raise ValueError('nonpatch byte mismatch '+name)
  receipt['linked_functions'].append({'name':name,'length':n,'complete_instructions':True,'nonpatch_bytes_match':True})
 if struct.unpack('<I',host.get_data(0x3076,4))[0]!=0x401a30:raise ValueError('tap pointer mismatch')
 tap=bytes.fromhex('9c60b8000300a7e8d3ffffff619dff742410ff742410ff742410ff742410ff157230400083c4109c60b8010300a7e8acffffff619dc3')
 if host.get_data(0x1a30,len(tap))!=tap:raise ValueError('qsort instrumentation mismatch')
 try:
  q=subprocess.run([str(P/'host-x86.exe')],capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW);out,err,code=q.stdout,q.stderr,q.returncode
 except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
 (O/'native.stdout.bin').write_bytes(out);(O/'native.stderr.bin').write_bytes(err)
 allowed={0xa7000000+i for i in range(1,8)}|{0xa7000100+i for i in range(6)}|{0xa7000200+i for i in range(6)}|{0xa7000300,0xa7000301};markers=[]
 for i in range(0,len(out)-3,4):
  v=struct.unpack_from('<I',out,i)[0]
  if v not in allowed:break
  markers.append(hex(v))
 receipt['runs']['native']={'exit':code,'stdout_bytes':len(out),'stderr_bytes':len(err),'stdout_sha256':h(out),'markers':markers}
 receipt['status']='observed';receipt['limits']='Diagnostic instrumentation changes qsort call stack. Stage markers do not prove native parity, module memory integrity or live gameplay.'
except Exception as e:receipt['error']=repr(e)
finally:
 receipt['post_checks']=[]
 for path,pin in pins:
  try:
   v=h(path.read_bytes());receipt['post_checks'].append({'path':str(path),'sha256':v,'expected':pin})
   if v!=pin:receipt['status']='failed';receipt['post_error']='changed pin '+str(path)
  except Exception as e:receipt['status']='failed';receipt['post_error']=repr(e)
 (O/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt,indent=2));sys.exit(receipt['status']!='observed')

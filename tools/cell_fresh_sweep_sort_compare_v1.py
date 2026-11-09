"""Review gate for fresh-only BBA7C0 with installed MSVCR90 qsort."""
from pathlib import Path
import hashlib,json,struct,subprocess,sys,platform,re,pefile,capstone
R=Path(__file__).resolve().parents[1];P=R/'target/cell-fresh-sort-prep-v2';O=R/'target/cell-fresh-sort-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');D=Path(r'D:\Games\SPORE Collection\SporeBin\msvcr90.dll');SH='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';DH='ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185';MH='c0ddf9def312eb4ea4482cbee191393f54dd6e8b155bb9aad7ff8645e02a51da';h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True)
for n in ('model','native'):
 for st in ('stdout','stderr'):(O/(n+'.'+st+'.bin')).write_bytes(b'')
receipt={'status':'failed','platform':platform.platform(),'checks':[],'runs':{}};pins=[]
try:
 if sys.platform!='win32' or platform.machine().lower() not in ('amd64','x86_64'):raise ValueError('Windows x64 required')
 mb=(P/'manifest.json').read_bytes()
 if h(mb)!=MH:raise ValueError('manifest mismatch')
 manifest=json.loads(mb);pins=[(Path(k),v) for k,v in manifest['pins'].items()]+[(P/'manifest.json',MH),(S,SH),(D,DH)]
 for path,pin in pins:
  v=h(path.read_bytes());receipt['checks'].append({'path':str(path),'sha256':v,'expected':pin})
  if v!=pin:raise ValueError('pin mismatch '+str(path))
 src=pefile.PE(str(S),fast_load=True);host=pefile.PE(str(P/'host-x86.exe'),fast_load=True);dll=pefile.PE(str(D),fast_load=True)
 if dll.FILE_HEADER.Machine!=0x14c:raise ValueError('CRT machine mismatch')
 dll.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY['IMAGE_DIRECTORY_ENTRY_EXPORT']]);qs=[x for x in dll.DIRECTORY_ENTRY_EXPORT.symbols if x.name==b'qsort']
 if len(qs)!=1 or qs[0].forwarder is not None:raise ValueError('CRT qsort export mismatch')
 functions=[(0xbba7c0,511,'sort.bin','SortSource',0x401905),(0x11e04f0,43,'probe.bin','StackProbe',0x401b04),(0xbba700,29,'compare.bin','Comparator',0x401b2f)]
 mt=(P/'host-x86.map').read_text();dis=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host base')
 receipt['linked_functions']=[]
 for a,n,file,name,va in functions:
  f=src.get_data(a-src.OPTIONAL_HEADER.ImageBase,n);b=host.get_data(va-host.OPTIONAL_HEADER.ImageBase,n)
  if f!=(P/file).read_bytes() or len(b)!=len(f):raise ValueError('source mismatch '+name)
  v=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+name+r'\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
  if len(v)!=1 or int(v[0],16)!=va:raise ValueError('symbol mismatch '+name)
  ins=list(dis.disasm(f,a))
  if sum(i.size for i in ins)!=n or ins[-1].address+ins[-1].size!=a+n:raise ValueError('incomplete instructions '+name)
  if name in ('SortSource','Comparator') and bytes(ins[-1].bytes)!=b'\xc3':raise ValueError('missing RET '+name)
  mask=set()
  if name=='SortSource':
   for off in (6,0x1d,0x27):mask.update(range(off,off+4))
   if va+10+struct.unpack_from('<i',b,6)[0]!=0x401b04:raise ValueError('probe target')
   if struct.unpack_from('<I',b,0x1d)[0]!=0x401b2f or struct.unpack_from('<I',b,0x27)[0]!=0x403072:raise ValueError('sort callback/IAT target')
  if any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in mask):raise ValueError('nonpatch byte mismatch '+name)
  receipt['linked_functions'].append({'name':name,'length':n,'complete_instructions':True,'nonpatch_bytes_match':True})
 if (P/'host-x86.asm').read_text().count('cmp esp,SavedESP')<6:raise ValueError('missing per-call stack checks')
 receipt['runtime_admissions']='Pinned CRT file before/after; host checks exact module path and qsort address owner. Windows loader and in-memory module integrity admitted.'
 corpus=(P/'cases.bin').read_bytes();model=R/'target/debug/examples/cell_fresh_sweep_sort_oracle_v1.exe'
 for name,cmd,inp in [('model',[str(model)],corpus),('native',[str(P/'host-x86.exe')],None)]:
  try:
   q=subprocess.run(cmd,input=inp,capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW);out,err,code=q.stdout,q.stderr,q.returncode
  except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
  (O/(name+'.stdout.bin')).write_bytes(out);(O/(name+'.stderr.bin')).write_bytes(err);receipt['runs'][name]={'exit':code,'stdout_bytes':len(out),'stderr_bytes':len(err),'stdout_sha256':h(out)}
  if code!=0:raise ValueError(name+' failed')
 model=(O/'model.stdout.bin').read_bytes();native=(O/'native.stdout.bin').read_bytes()
 if len(model)!=312 or len(native)!=336:raise ValueError('output size')
 receipt['rows']=[]
 for i in range(3):
  a=model[i*104:(i+1)*104];b=native[i*112:i*112+104];cw,sw,mx=struct.unpack_from('<HHI',native,i*112+104)
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

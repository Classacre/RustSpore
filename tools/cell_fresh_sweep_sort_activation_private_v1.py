"""Reviewed diagnostic-only activation-context phase tracing; no parity claim."""
from pathlib import Path
import hashlib,json,struct,subprocess,sys,platform,re,pefile,capstone,ctypes,uuid
from ctypes import wintypes
R=Path(__file__).resolve().parents[1];P=R/'target/cell-fresh-sort-activation-prep-v1';O=R/'target/cell-fresh-sort-activation-private-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');D=Path(r'D:\Games\SPORE Collection\SporeBin\msvcr90.dll');OLD=R/'target/cell-fresh-sort-result-v1/receipt.json';SH='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';DH='ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185';OH='734946b244b104d3302ab1246642d30147af85497797e9d987a7ee74d9def1c4';MH='8d429da3d6e0956af2b914c86f38ef28898aa4b6b53bc98e2e5c74b68ba56dd7';h=lambda b:hashlib.sha256(b).hexdigest()
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
 functions=[(0xbba7c0,511,'sort.bin','SortSource',0x401ac9),(0x11e04f0,43,'probe.bin','StackProbe',0x401cc8),(0xbba700,29,'compare.bin','Comparator',0x401cf3)]
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
   if va+10+struct.unpack_from('<i',b,6)[0]!=0x401cc8:raise ValueError('probe target')
   if struct.unpack_from('<I',b,0x1d)[0]!=0x401cf3 or struct.unpack_from('<I',b,0x27)[0]!=0x403144:raise ValueError('comparator/tap pointer target')
  if any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in mask):raise ValueError('nonpatch byte mismatch '+name)
  receipt['linked_functions'].append({'name':name,'length':n,'complete_instructions':True,'nonpatch_bytes_match':True})
 # Validate 32-bit ACTCTXW layout, source executable and assembly directory.
 if struct.unpack('<8I',host.get_data(0x3110,32))!=(32,0x0c,0x40306a,0,0x4030ca,1,0,0):raise ValueError('ACTCTXW layout mismatch')
 expected_source=(str(S)+'\0').encode('utf-16le');expected_dir=(str(S.parent)+'\0').encode('utf-16le')
 if host.get_data(0x306a,len(expected_source))!=expected_source or host.get_data(0x30ca,len(expected_dir))!=expected_dir:raise ValueError('activation paths mismatch')
 src.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY['IMAGE_DIRECTORY_ENTRY_RESOURCE']])
 manifests=[t for t in src.DIRECTORY_ENTRY_RESOURCE.entries if t.id==24]
 if len(manifests)!=1:raise ValueError('source manifest resource missing')
 entries=[e for e in manifests[0].directory.entries if e.id==1]
 if len(entries)!=1:raise ValueError('source manifest ordinal missing')
 langs=entries[0].directory.entries
 if len(langs)!=1:raise ValueError('ambiguous source manifest language')
 data=langs[0].data.struct;embedded=src.get_data(data.OffsetToData,data.Size)
 if b'Microsoft.VC90.CRT' not in embedded or b'9.0.21022.8' not in embedded:raise ValueError('unexpected embedded CRT manifest')
 receipt['activation_input']={'source_resource_ordinal':1,'embedded_sha256':h(embedded),'assembly_dir':str(S.parent),'flags':'0x0c'}
 if struct.unpack('<I',host.get_data(0x3144,4))[0]!=0x401a93:raise ValueError('tap pointer mismatch')
 tap=bytes.fromhex('9c60b8000300a7e8d3ffffff619dff742410ff742410ff742410ff742410ff154031400083c4109c60b8010300a7e8acffffff619dc3')
 if host.get_data(0x1a93,len(tap))!=tap:raise ValueError('qsort instrumentation mismatch')
 # Create a private desktop in the current window station. The parent never
 # attaches to or switches desktops. A child may create windows there, but no
 # desktop switch, user-window enumeration, input or interaction is performed.
 user32=ctypes.WinDLL('user32',use_last_error=True)
 user32.CreateDesktopW.argtypes=[wintypes.LPCWSTR,wintypes.LPCWSTR,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p]
 user32.CreateDesktopW.restype=wintypes.HANDLE
 user32.CloseDesktop.argtypes=[wintypes.HANDLE];user32.CloseDesktop.restype=wintypes.BOOL
 # DESKTOP_READOBJECTS|CREATEWINDOW|ENUMERATE|WRITEOBJECTS: no SWITCHDESKTOP.
 desktop_name='RustSporeDiag-'+uuid.uuid4().hex;desktop=user32.CreateDesktopW(desktop_name,None,None,0,0x00c3,None)
 if not desktop:raise OSError(ctypes.get_last_error(),'CreateDesktopW failed')
 receipt['private_desktop']={'name':desktop_name,'access':'0x00c3','switched':False,'child_reaped_before_close':False}
 out=b'';err=b'';code='not_started'
 try:
  startup=subprocess.STARTUPINFO();startup.lpDesktop=desktop_name;startup.dwFlags|=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=0
  try:
   q=subprocess.run([str(P/'host-x86.exe')],capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW,startupinfo=startup);out,err,code=q.stdout,q.stderr,q.returncode
  except subprocess.TimeoutExpired as e:
   # subprocess.run kills and waits for its child before propagating timeout.
   out,err,code=e.stdout or b'',e.stderr or b'','timeout'
  receipt['private_desktop']['child_reaped_before_close']=True
 finally:
  # Closing this single desktop handle follows subprocess completion/timeout;
  # no switch, thread attachment, window interaction or other handle is made.
  closed=bool(user32.CloseDesktop(desktop));receipt['private_desktop']['close_succeeded']=closed
  if not closed:receipt['private_desktop']['close_error']=ctypes.get_last_error()
  (O/'native.stdout.bin').write_bytes(out);(O/'native.stderr.bin').write_bytes(err)
 if not closed:raise OSError(receipt['private_desktop']['close_error'],'CloseDesktop failed')
 allowed={0xa7000000+i for i in range(1,8)}|{0xa7000010+i for i in range(4)}|{0xa7000100+i for i in range(6)}|{0xa7000200+i for i in range(6)}|{0xa7000300,0xa7000301};markers=[]
 for i in range(0,len(out)-3,4):
  v=struct.unpack_from('<I',out,i)[0]
  if v not in allowed:break
  markers.append(hex(v))
 receipt['runs']['native']={'exit':code,'stdout_bytes':len(out),'stderr_bytes':len(err),'stdout_sha256':h(out),'markers':markers}
 receipt['status']='observed';receipt['limits']='Private desktop is not switched; diagnostic activation context and instrumentation change process state and qsort call stack. Stage markers do not prove native parity, module memory integrity or live gameplay.'
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

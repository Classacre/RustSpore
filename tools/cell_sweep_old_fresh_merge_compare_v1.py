"""Review gate for fresh-only BBA7C0 with installed MSVCR90 qsort."""
from pathlib import Path
import hashlib,json,struct,subprocess,sys,platform,re,pefile,capstone,ctypes,uuid
from ctypes import wintypes
R=Path(__file__).resolve().parents[1];P=R/'target/cell-old-fresh-merge-prep-v1';O=R/'target/cell-old-fresh-merge-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');D=Path(r'D:\Games\SPORE Collection\SporeBin\msvcr90.dll');SH='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';DH='ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185';MH='65afa750d1ef6f397e3e3b3be1041d08805dc078f7531793174226680f7e9bc2';h=lambda b:hashlib.sha256(b).hexdigest()
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
 functions=[(0xbba7c0,511,'sort.bin','SortSource',0x402066),(0x11e04f0,43,'probe.bin','StackProbe',0x402265),(0xbba700,29,'compare.bin','Comparator',0x402290)]
 mt=(P/'host-x86.map').read_text();dis=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host base')
 receipt['linked_functions']=[]
 for a,n,file,name,va in functions:
  f=src.get_data(a-src.OPTIONAL_HEADER.ImageBase,n);b=host.get_data(va-host.OPTIONAL_HEADER.ImageBase,n)
  if f!=(R/'target/cell-fresh-sort-prep-v2'/file).read_bytes() or len(b)!=len(f):raise ValueError('source mismatch '+name)
  v=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+name+r'\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
  if len(v)!=1 or int(v[0],16)!=va:raise ValueError('symbol mismatch '+name)
  ins=list(dis.disasm(f,a))
  if sum(i.size for i in ins)!=n or ins[-1].address+ins[-1].size!=a+n:raise ValueError('incomplete instructions '+name)
  if name in ('SortSource','Comparator') and bytes(ins[-1].bytes)!=b'\xc3':raise ValueError('missing RET '+name)
  mask=set()
  if name=='SortSource':
   for off in (6,0x1d,0x27):mask.update(range(off,off+4))
   if va+10+struct.unpack_from('<i',b,6)[0]!=0x402265:raise ValueError('probe target')
   if struct.unpack_from('<I',b,0x1d)[0]!=0x402290 or struct.unpack_from('<I',b,0x27)[0]!=0x404140:raise ValueError('sort callback/IAT target')
  if any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in mask):raise ValueError('nonpatch byte mismatch '+name)
  receipt['linked_functions'].append({'name':name,'length':n,'complete_instructions':True,'nonpatch_bytes_match':True})
 # Exact 32-bit ACTCTXW points to source EXE resource 1 and SporeBin.
 if struct.unpack('<8I',host.get_data(0x4110,32))!=(32,0x0c,0x40406a,0,0x4040ca,1,0,0):raise ValueError('ACTCTXW layout mismatch')
 if host.get_data(0x406a,len((str(S)+'\0').encode('utf-16le')))!=(str(S)+'\0').encode('utf-16le'):raise ValueError('source EXE path mismatch')
 if host.get_data(0x40ca,len((str(S.parent)+'\0').encode('utf-16le')))!=(str(S.parent)+'\0').encode('utf-16le'):raise ValueError('assembly directory mismatch')
 src.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY['IMAGE_DIRECTORY_ENTRY_RESOURCE']]);r=[x for x in src.DIRECTORY_ENTRY_RESOURCE.entries if x.id==24];entries=[x for x in r[0].directory.entries if x.id==1] if len(r)==1 else []
 if len(entries)!=1 or len(entries[0].directory.entries)!=1:raise ValueError('source manifest ordinal 1 missing or ambiguous')
 data=entries[0].directory.entries[0].data.struct;embedded=src.get_data(data.OffsetToData,data.Size)
 if h(embedded)!='9f2fc067639866642bb1a73fb43006d233e569d25566b16dedec472fe5d3c5c3' or b'Microsoft.VC90.CRT' not in embedded or b'9.0.21022.8' not in embedded:raise ValueError('unexpected CRT manifest')
 receipt['activation_input']={'resource_ordinal':1,'embedded_sha256':h(embedded),'flags':'0x0c'}
 if (P/'host-x86.asm').read_text().count('cmp esp,SavedESP')<5:raise ValueError('missing per-call stack checks')
 receipt['runtime_admissions']='Pinned CRT and manifests before/after; host activates source manifest and checks exact module path and qsort owner. Windows loader and in-memory module integrity admitted; private desktop is not switched.'
 corpus=(P/'cases.bin').read_bytes();model=R/'target/debug/examples/cell_sweep_old_fresh_merge_oracle_v1.exe'
 for name,cmd,inp in [('model',[str(model)],corpus),('native',[str(P/'host-x86.exe')],None)]:
  # Fresh private desktop per child; never switch or attach the parent thread.
  user32=ctypes.WinDLL('user32',use_last_error=True)
  user32.CreateDesktopW.argtypes=[wintypes.LPCWSTR,wintypes.LPCWSTR,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p];user32.CreateDesktopW.restype=wintypes.HANDLE
  user32.CloseDesktop.argtypes=[wintypes.HANDLE];user32.CloseDesktop.restype=wintypes.BOOL
  desktop_name='RustSporeCompare-'+uuid.uuid4().hex;desktop=user32.CreateDesktopW(desktop_name,None,None,0,0x00c3,None)
  if not desktop:raise OSError(ctypes.get_last_error(),'CreateDesktopW failed')
  detail={'name':desktop_name,'access':'0x00c3','switched':False,'child_reaped_before_close':False};receipt.setdefault('private_desktops',{})[name]=detail
  out=b'';err=b'';code='not_started'
  try:
   startup=subprocess.STARTUPINFO();startup.lpDesktop=desktop_name;startup.dwFlags|=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=0
   try:
    q=subprocess.run(cmd,input=inp,capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW,startupinfo=startup);out,err,code=q.stdout,q.stderr,q.returncode
   except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
   detail['child_reaped_before_close']=True
  finally:
   closed=bool(user32.CloseDesktop(desktop));detail['close_succeeded']=closed
   if not closed:detail['close_error']=ctypes.get_last_error()
   (O/(name+'.stdout.bin')).write_bytes(out);(O/(name+'.stderr.bin')).write_bytes(err)
  if not closed:raise OSError(detail['close_error'],'CloseDesktop failed')
  receipt['runs'][name]={'exit':code,'stdout_bytes':len(out),'stderr_bytes':len(err),'stdout_sha256':h(out)}
  if code!=0:raise ValueError(name+' failed')
 model=(O/'model.stdout.bin').read_bytes();native=(O/'native.stdout.bin').read_bytes()
 if len(model)!=660 or len(native)!=700:raise ValueError('output size')
 receipt['rows']=[]
 for i in range(5):
  a=model[i*132:(i+1)*132];b=native[i*140:i*140+132];cw,sw,mx=struct.unpack_from('<HHI',native,i*140+132)
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

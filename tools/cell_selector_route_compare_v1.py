"""Review gate: pending BBB880 branch versus synthetic byte-store model."""
from pathlib import Path
import ctypes,hashlib,json,platform,re,struct,subprocess,sys,uuid,pefile,capstone
from ctypes import wintypes
R=Path(__file__).resolve().parents[1];P=R/'target/cell-selector-route-prep-v1';O=R/'target/cell-selector-route-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');MH='74d50e647b72bb140971b749bdcb00da7b6687c94fb3a9b697229365ea538128';h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True)
for n in ('native.stdout.bin','native.stderr.bin','model.bin'):(O/n).write_bytes(b'')
receipt={'status':'failed','checks':[]};pins=[]
try:
 if sys.platform!='win32' or platform.machine().lower() not in ('amd64','x86_64'):raise ValueError('Windows x64 required')
 mb=(P/'manifest.json').read_bytes()
 if h(mb)!=MH:raise ValueError('manifest mismatch')
 pins=[(Path(k),v) for k,v in json.loads(mb)['pins'].items()]+[(P/'manifest.json',MH)]
 for path,pin in pins:
  v=h(path.read_bytes());receipt['checks'].append({'path':str(path),'sha256':v,'expected':pin})
  if v!=pin:raise ValueError('pin mismatch '+str(path))
 src=pefile.PE(str(S),fast_load=True);host=pefile.PE(str(P/'host-x86.exe'),fast_load=True);va=0x40263c;n=118;a=0xe73e10
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host base')
 f=src.get_data(a-src.OPTIONAL_HEADER.ImageBase,n);b=host.get_data(va-host.OPTIONAL_HEADER.ImageBase,n)
 if f!=(P/'source.bin').read_bytes() or len(b)!=n:raise ValueError('source mismatch')
 mt=(P/'host-x86.map').read_text()
 for name,expected in [('RouteSource',va),('RouteBoth',0x40258f),('RouteSecond',0x402596),('RouteFirst',0x40259d),('RouteNeither',0x4025a4)]:
  matches=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+name+r'\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
  if len(matches)!=1 or int(matches[0],16)!=expected:raise ValueError('symbol mismatch '+name)
 ins=list(capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32).disasm(f,a))
 if sum(i.size for i in ins)!=n or ins[-1].address+ins[-1].size!=a+n or ins[-1].mnemonic!='jmp':raise ValueError('incomplete source')
 mask=set()
 for off,original,linked in [(0x34,0xe6ade0,0x40258f),(0x41,0xe73660,0x402596),(0x64,0xe73930,0x40259d),(0x71,0xe73bf0,0x4025a4)]:
  if f[off]!=0xe9 or a+off+5+struct.unpack_from('<i',f,off+1)[0]!=original or va+off+5+struct.unpack_from('<i',b,off+1)[0]!=linked:raise ValueError('route relocation mismatch')
  mask.update(range(off+1,off+5))
 if any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in mask):raise ValueError('nonpatch mismatch')
 receipt['linked_source']={'range':'E73E10..E73E85','sha256':h(f),'complete':True,'nonpatch_match':True,'four_tail_jumps':'verified'}
 if (P/'host-x86.asm').read_text().count('cmp esp,SavedESP')!=34:raise ValueError('missing stack checks')
 corpus=(P/'cases.bin').read_bytes()
 if len(corpus)!=272:raise ValueError('case size')
 model=bytearray()
 def route(a,b):return 1 if a and b else 2 if b else 3 if a else 4
 for row in range(17):
  a0,a1,b0,b1=struct.unpack_from('<4I',corpus,row*16);present_a=bool(a0 and a1);present_b=bool(b0 and b1)
  model+=struct.pack('<14I',route(present_a,present_b),1,1,1,1,route(present_b,present_a),1,1,1,1,a0,a1,b0,b1)
 (O/'model.bin').write_bytes(model)
 user32=ctypes.WinDLL('user32',use_last_error=True);user32.CreateDesktopW.argtypes=[wintypes.LPCWSTR,wintypes.LPCWSTR,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p];user32.CreateDesktopW.restype=wintypes.HANDLE;user32.CloseDesktop.argtypes=[wintypes.HANDLE];user32.CloseDesktop.restype=wintypes.BOOL
 name='RustSporeSelectorRoute-'+uuid.uuid4().hex;desktop=user32.CreateDesktopW(name,None,None,0,0x00c3,None)
 if not desktop:raise OSError(ctypes.get_last_error(),'CreateDesktopW failed')
 detail={'name':name,'access':'0x00c3','switched':False,'child_reaped_before_close':False};receipt['private_desktop']=detail;out=b'';err=b'';code='not_started'
 try:
  startup=subprocess.STARTUPINFO();startup.lpDesktop=name;startup.dwFlags|=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=0
  try:
   q=subprocess.run([str(P/'host-x86.exe')],capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW,startupinfo=startup);out,err,code=q.stdout,q.stderr,q.returncode
  except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
  detail['child_reaped_before_close']=True
 finally:
  closed=bool(user32.CloseDesktop(desktop));detail['close_succeeded']=closed
  if not closed:detail['close_error']=ctypes.get_last_error()
  (O/'native.stdout.bin').write_bytes(out);(O/'native.stderr.bin').write_bytes(err)
 if not closed:raise OSError(detail['close_error'],'CloseDesktop failed')
 receipt['native']={'exit':code,'stdout_bytes':len(out),'stdout_sha256':h(out),'stderr_bytes':len(err)}
 if code!=0 or len(out)!=1088:raise ValueError('native failure or size')
 receipt['rows']=[]
 for i in range(17):
  match=model[i*56:(i+1)*56]==out[i*64:i*64+56];cw,sw,mx=struct.unpack_from('<HHI',out,i*64+56);receipt['rows'].append({'case':i,'match':match,'control':hex(cw),'status':hex(sw),'mxcsr':hex(mx)})
  if not match or cw!=0x27f or sw&0x3800 or mx&~0x3f!=0x1f80:raise ValueError('row/control mismatch')
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

"""Review gate: pending BBB880 branch versus synthetic byte-store model."""
from pathlib import Path
import ctypes,hashlib,json,platform,re,struct,subprocess,sys,uuid,pefile,capstone
from ctypes import wintypes
R=Path(__file__).resolve().parents[1];P=R/'target/cell-pending-endpoint-update-prep-v2';O=R/'target/cell-pending-endpoint-update-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');MH='5f244f52f0e9278816d2e08174cbbe25a5a9c43d756141830a1738cd3838234f';h=lambda b:hashlib.sha256(b).hexdigest()
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
 src=pefile.PE(str(S),fast_load=True);host=pefile.PE(str(P/'host-x86.exe'),fast_load=True);va=0x401a0e;trap=0x401a07;n=381;a=0xbbb880
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host base')
 f=src.get_data(a-src.OPTIONAL_HEADER.ImageBase,n);b=host.get_data(va-host.OPTIONAL_HEADER.ImageBase,n)
 if f!=(P/'source.bin').read_bytes() or len(b)!=n:raise ValueError('source bytes mismatch')
 mt=(P/'host-x86.map').read_text()
 for name,expected in [('UpdateSource',va),('UnexpectedHelper',trap)]:
  matches=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+name+r'\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
  if len(matches)!=1 or int(matches[0],16)!=expected:raise ValueError('symbol mismatch '+name)
 ins=list(capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32).disasm(f,a))
 if sum(i.size for i in ins)!=n or ins[-1].address+ins[-1].size!=a+n or bytes(ins[-1].bytes)!=b'\xc3':raise ValueError('incomplete source function')
 mask=set()
 for off,target in [(0xd9,0xbbb020),(0xfe,0xbbb130),(0x127,0xbbb240),(0x150,0xbbb320)]:
  if f[off]!=0xe8 or a+off+5+struct.unpack_from('<i',f,off+1)[0]!=target or va+off+5+struct.unpack_from('<i',b,off+1)[0]!=trap:raise ValueError('helper relocation mismatch')
  mask.update(range(off+1,off+5))
 if any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in mask):raise ValueError('nonpatch byte mismatch')
 if (P/'host-x86.asm').read_text().count('cmp esp,SavedESP')!=4:raise ValueError('missing stack checks')
 receipt['linked_source']={'range':'BBB880..BBB9FC','sha256':h(f),'complete':True,'nonpatch_match':True,'four_sorted_helper_calls':'redirected to nonreturning exit 7'}
 corpus=(P/'cases.bin').read_bytes()
 if len(corpus)!=96:raise ValueError('case size')
 model=bytearray()
 for row in range(4):
  l0,l1,l2,l3,v0,v1,v2,v3=struct.unpack_from('<4H4I',corpus,row*24);links=[l0,l1,l2,l3];vals=[v0,v1,v2,v3]
  if any(x>=4 for x in links) or l0==l2 or l1==l3:raise ValueError('bad indices')
  model+=struct.pack('<II4H',0x12345678,0x87654321,*[0x8000|x for x in links])
  for axis in range(2):
   for i in range(4):
    value=0x3e800000+axis*0x10000+i
    if i==links[axis]:value=vals[axis]
    if i==links[axis+2]:value=vals[axis+2]
    model+=struct.pack('<II',axis*16+i,value)
 (O/'model.bin').write_bytes(model)
 user32=ctypes.WinDLL('user32',use_last_error=True);user32.CreateDesktopW.argtypes=[wintypes.LPCWSTR,wintypes.LPCWSTR,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p];user32.CreateDesktopW.restype=wintypes.HANDLE;user32.CloseDesktop.argtypes=[wintypes.HANDLE];user32.CloseDesktop.restype=wintypes.BOOL
 name='RustSporePending-'+uuid.uuid4().hex;desktop=user32.CreateDesktopW(name,None,None,0,0x00c3,None)
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
 if code!=0 or len(out)!=352:raise ValueError('native failure or size')
 receipt['rows']=[]
 for i in range(4):
  match=model[i*80:(i+1)*80]==out[i*88:i*88+80];cw,sw,mx=struct.unpack_from('<HHI',out,i*88+80);receipt['rows'].append({'case':i,'match':match,'control':hex(cw),'status':hex(sw),'mxcsr':hex(mx)})
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

"""Review gate: pending BBB880 branch versus synthetic byte-store model."""
from pathlib import Path
import ctypes,hashlib,json,platform,re,struct,subprocess,sys,uuid,pefile,capstone
from ctypes import wintypes
R=Path(__file__).resolve().parents[1];P=R/'target/cell-sweep-remove-prep-v2';O=R/'target/cell-sweep-remove-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');MH='5023fd3d17b7b3d3d3ea0857b4bcc76c6f33912a2cb7824a683c82dfc02f7037';h=lambda b:hashlib.sha256(b).hexdigest()
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
 src=pefile.PE(str(S),fast_load=True);host=pefile.PE(str(P/'host-x86.exe'),fast_load=True);va=0x401ea0;n=202;a=0xbbac30
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host base')
 f=src.get_data(a-src.OPTIONAL_HEADER.ImageBase,n);b=host.get_data(va-host.OPTIONAL_HEADER.ImageBase,n)
 if f!=(P/'source.bin').read_bytes() or b!=f:raise ValueError('linked source mismatch')
 mt=(P/'host-x86.map').read_text();matches=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+RemoveSource\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
 if len(matches)!=1 or int(matches[0],16)!=va:raise ValueError('symbol mismatch')
 ins=list(capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32).disasm(f,a))
 if sum(i.size for i in ins)!=n or ins[-1].address+ins[-1].size!=a+n or bytes(ins[-1].bytes)!=b'\xc3':raise ValueError('incomplete source')
 receipt['linked_source']={'range':'BBAC30..BBACF9','sha256':h(f),'complete':True,'all_bytes_match':True,'patches':0}
 if (P/'host-x86.asm').read_text().count('cmp esp,SavedESP')!=4:raise ValueError('missing stack checks')
 corpus=(P/'cases.bin').read_bytes()
 if len(corpus)!=80:raise ValueError('case size')
 model=bytearray()
 for row in range(4):
  a0,a1,a2,a3,live,high,head=struct.unpack_from('<4HIII',corpus,row*20);links=[a0,a1,a2,a3]
  if not live or high!=2 or head not in (0xffffffff,1):raise ValueError('invalid fixture state')
  selected=[]
  for j,link in enumerate(links):
   idx=link&0x7fff;bank=(2 if link&0x8000 else 0)+(j%2)
   if idx>=4 or (bank,idx) in selected:raise ValueError('invalid or duplicate endpoint')
   selected.append((bank,idx))
  model+=struct.pack('<III',live-1,high,0)
  model+=struct.pack('<II4H',head,0xffffffff,*links)
  model+=struct.pack('<IIII',0xffffffff if head==1 else 0x55667788,0xffffffff if head==1 else 0x11223344,0x99aabbcc,0xddeeff00)
  for bank in range(4):
   for i in range(4):
    flag=0x11000000+bank*0x10000+i
    if (bank,i) in selected:flag=(flag&0xffff0000)|0xfffe
    model+=struct.pack('<II',flag,0x3f000000+bank*0x10000+i)
 (O/'model.bin').write_bytes(model)
 user32=ctypes.WinDLL('user32',use_last_error=True);user32.CreateDesktopW.argtypes=[wintypes.LPCWSTR,wintypes.LPCWSTR,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p];user32.CreateDesktopW.restype=wintypes.HANDLE;user32.CloseDesktop.argtypes=[wintypes.HANDLE];user32.CloseDesktop.restype=wintypes.BOOL
 name='RustSporeRemove-'+uuid.uuid4().hex;desktop=user32.CreateDesktopW(name,None,None,0,0x00c3,None)
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
 if code!=0 or len(out)!=720:raise ValueError('native failure or size')
 receipt['rows']=[]
 for i in range(4):
  match=model[i*172:(i+1)*172]==out[i*180:i*180+172];cw,sw,mx=struct.unpack_from('<HHI',out,i*180+172);receipt['rows'].append({'case':i,'match':match,'control':hex(cw),'status':hex(sw),'mxcsr':hex(mx)})
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

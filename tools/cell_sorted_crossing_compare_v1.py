"""Review gate: pending BBB880 branch versus synthetic byte-store model."""
from pathlib import Path
import ctypes,hashlib,json,platform,re,struct,subprocess,sys,uuid,pefile,capstone
from ctypes import wintypes
R=Path(__file__).resolve().parents[1];P=R/'target/cell-sorted-crossing-prep-v1';O=R/'target/cell-sorted-crossing-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');MH='42eae53004a68729b4f8b3bf119e8f114096b88960e8b2cebfbfdf1daae02ff5';h=lambda b:hashlib.sha256(b).hexdigest()
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
 src=pefile.PE(str(S),fast_load=True);host=pefile.PE(str(P/'host-x86.exe'),fast_load=True);trap=0x401945
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host base')
 mt=(P/'host-x86.map').read_text();funcs=[('MinLeft',0xbbb020,264,0x40194c,[(0xbbb097,0xbba9c0)]),('MaxRight',0xbbb130,258,0x401a54,[(0xbbb1a5,0xbba9c0)]),('MinRight',0xbbb240,209,0x401b56,[(0xbbb2b3,0xbba9c0),(0xbbb2ca,0xbbaa80)]),('MaxLeft',0xbbb320,207,0x401c27,[(0xbbb391,0xbba9c0),(0xbbb3a8,0xbbaa80)])]
 receipt['linked_source']=[]
 for name,a,n,va,calls in funcs:
  f=src.get_data(a-src.OPTIONAL_HEADER.ImageBase,n);b=host.get_data(va-host.OPTIONAL_HEADER.ImageBase,n)
  if f!=(P/(name+'.bin')).read_bytes() or len(b)!=n:raise ValueError('source mismatch '+name)
  matches=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+name+r'\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
  if len(matches)!=1 or int(matches[0],16)!=va:raise ValueError('symbol mismatch '+name)
  ins=list(capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32).disasm(f,a))
  if sum(i.size for i in ins)!=n or ins[-1].address+ins[-1].size!=a+n or bytes(ins[-1].bytes)!=b'\xc3':raise ValueError('incomplete source '+name)
  mask=set()
  for call,target in calls:
   off=call-a
   if f[off]!=0xe8 or a+off+5+struct.unpack_from('<i',f,off+1)[0]!=target or va+off+5+struct.unpack_from('<i',b,off+1)[0]!=trap:raise ValueError('relocation mismatch '+name)
   mask.update(range(off+1,off+5))
  if any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in mask):raise ValueError('nonpatch mismatch '+name)
  receipt['linked_source'].append({'name':name,'length':n,'complete':True,'nonpatch_match':True,'calls':len(calls)})
 matches=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+UnexpectedHelper\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
 if len(matches)!=1 or int(matches[0],16)!=trap:raise ValueError('trap mismatch')
 if (P/'host-x86.asm').read_text().count('cmp esp,SavedESP')!=4:raise ValueError('missing stack checks')
 corpus=(P/'cases.bin').read_bytes()
 if len(corpus)!=176:raise ValueError('case size')
 model=bytearray()
 for row in range(4):
  offset=row*44;axis,*ranks=struct.unpack_from('<I4H',corpus,offset);records=[struct.unpack_from('<HHI',corpus,offset+12+i*8) for i in range(4)]
  if axis not in (0,1):raise ValueError('invalid axis')
  slot_links=[[0x41,0x42,0x43,0x44],[0x51,0x52,0x53,0x54]]
  for j in range(2):
   for side in range(2):slot_links[j][axis+2*side]=ranks[j*2+side]
  records.sort(key=lambda e:struct.unpack('<f',struct.pack('<I',e[2]))[0])
  for rank,(flag,slot,value) in enumerate(records):slot_links[slot][axis+2*flag]=rank
  for slot in range(2):model+=struct.pack('<II4H',0x10001+slot*0x10001,0,*slot_links[slot])
  for e in records:model+=struct.pack('<HHI',*e)
  model+=b'\xcc'*16+struct.pack('<I',2)
 (O/'model.bin').write_bytes(model)
 user32=ctypes.WinDLL('user32',use_last_error=True);user32.CreateDesktopW.argtypes=[wintypes.LPCWSTR,wintypes.LPCWSTR,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p];user32.CreateDesktopW.restype=wintypes.HANDLE;user32.CloseDesktop.argtypes=[wintypes.HANDLE];user32.CloseDesktop.restype=wintypes.BOOL
 name='RustSporeCrossing-'+uuid.uuid4().hex;desktop=user32.CreateDesktopW(name,None,None,0,0x00c3,None)
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
 if code!=0 or len(out)!=368:raise ValueError('native failure or size')
 receipt['rows']=[]
 for i in range(4):
  match=model[i*84:(i+1)*84]==out[i*92:i*92+84];cw,sw,mx=struct.unpack_from('<HHI',out,i*92+84);receipt['rows'].append({'case':i,'match':match,'control':hex(cw),'status':hex(sw),'mxcsr':hex(mx)})
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

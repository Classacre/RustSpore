"""Review gate: pending BBB880 branch versus synthetic byte-store model."""
from pathlib import Path
import ctypes,hashlib,json,platform,re,struct,subprocess,sys,uuid,pefile,capstone
from ctypes import wintypes
R=Path(__file__).resolve().parents[1];P=R/'target/cell-feature-loop-skeleton-prep-v3';O=R/'target/cell-feature-loop-skeleton-result-v3';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');MH='7315b7e18970ee5c6dc10955d968f750ac9f8deaf8c0a5f5044eecfd0499d1f5';h=lambda b:hashlib.sha256(b).hexdigest()
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
 src=pefile.PE(str(S),fast_load=True);host=pefile.PE(str(P/'host-x86.exe'),fast_load=True);va=0x4017e7;n=1018;a=0xe6ade0
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host base')
 f=src.get_data(a-src.OPTIONAL_HEADER.ImageBase,n);b=host.get_data(va-host.OPTIONAL_HEADER.ImageBase,n)
 if f!=(P/'source.bin').read_bytes() or len(b)!=n:raise ValueError('source mismatch')
 mt=(P/'host-x86.map').read_text()
 for name,expected in [('LoopSource',va),('RelationStub',0x4016d8),('CacheStub',0x4016fd),('OuterCut',0x40178c),('CandidateCut',0x4017b4),('StubFailure',0x4017e0)]:
  matches=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+name+r'\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
  if len(matches)!=1 or int(matches[0],16)!=expected:raise ValueError('symbol mismatch '+name)
 dis=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);ins=list(dis.disasm(f,a));bounds={i.address for i in ins}|{a+n}
 if sum(i.size for i in ins)!=n or ins[-1].address+ins[-1].size!=a+n or bytes(ins[-1].bytes)!=b'\xc3':raise ValueError('incomplete source')
 mask=set()
 for address,original,linked in [(0xe6adf1,0xe57260,0x4016d8),(0xe6ae17,0xe67a50,0x4016fd),(0xe6ae3e,0xe67a50,0x4016fd),(0xe6b19a,0xe6a450,0x4017e0)]:
  off=address-a
  if f[off]!=0xe8 or a+off+5+struct.unpack_from('<i',f,off+1)[0]!=original or va+off+5+struct.unpack_from('<i',b,off+1)[0]!=linked:raise ValueError('call mismatch')
  mask.update(range(off+1,off+5))
 for address,linked in [(0xe6ae6c,0x40178c),(0xe6af72,0x4017b4)]:
  off=address-a
  if address not in bounds or address+9 not in bounds or b[off]!=0xe9 or b[off+5:off+9]!=b'\x90'*4 or va+off+5+struct.unpack_from('<i',b,off+1)[0]!=linked:raise ValueError('cutout mismatch')
  mask.update(range(off,off+9))
 if any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in mask):raise ValueError('nonpatch mismatch')
 if host.get_data(0x178c,8)!=bytes.fromhex("c744243000000000"):raise ValueError("outer stack initialization missing")
 # Verify cutout jumps enter original continuation addresses in the linked body.
 def destinations(start,size):
  return [int(i.op_str,16) for i in dis.disasm(host.get_data(start-host.OPTIONAL_HEADER.ImageBase,size),start) if i.mnemonic=='jmp' and i.op_str.startswith('0x')]
 outer=destinations(0x40178c,0x28);inner=destinations(0x4017b4,0x2c)
 if va+(0xe6af66-a) not in outer or va+(0xe6b1ba-a) not in outer or va+(0xe6b1aa-a) not in inner:raise ValueError('cutout continuation mismatch')
 receipt['linked_source']={'range':'E6ADE0..E6B1D9','sha256':h(f),'complete':True,'nonpatch_match':True,'calls':4,'cutouts':2,'continuations_verified':True}
 if (P/'host-x86.asm').read_text().count('cmp esp,SavedESP')!=5:raise ValueError('missing stack checks')
 corpus=(P/'cases.bin').read_bytes()
 if len(corpus)!=160:raise ValueError('case size')
 model=bytearray()
 for row in range(5):
  na,nb,*classes=struct.unpack_from('<8I',corpus,row*32);ca=classes[:3];cb=classes[3:]
  if na>3 or nb>3:raise ValueError('bad count')
  pairs=[(i,j) for i in range(na) if ca[i]!=1 for j in range(nb) if cb[j]!=1]
  words=[1,2,len(pairs)]+[v for pair in pairs for v in pair]+[0]*(18-2*len(pairs))
  model+=struct.pack('<21I',*words)
 (O/'model.bin').write_bytes(model)
 user32=ctypes.WinDLL('user32',use_last_error=True);user32.CreateDesktopW.argtypes=[wintypes.LPCWSTR,wintypes.LPCWSTR,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p];user32.CreateDesktopW.restype=wintypes.HANDLE;user32.CloseDesktop.argtypes=[wintypes.HANDLE];user32.CloseDesktop.restype=wintypes.BOOL
 name='RustSporeFeatureLoop-'+uuid.uuid4().hex;desktop=user32.CreateDesktopW(name,None,None,0,0x00c3,None)
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
 if code!=0 or err or len(out)!=460:raise ValueError('native failure or size')
 receipt['rows']=[]
 for i in range(5):
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

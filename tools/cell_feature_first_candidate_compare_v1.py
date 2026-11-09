"""Review gate: pending BBB880 branch versus synthetic byte-store model."""
from pathlib import Path
import ctypes,hashlib,json,platform,re,struct,subprocess,sys,uuid,pefile,capstone
from ctypes import wintypes
R=Path(__file__).resolve().parents[1];P=R/'target/cell-feature-first-candidate-prep-v3';O=R/'target/cell-feature-first-candidate-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');MH='cb1173c76a6b37ff589b19baa3dc90420988c16d6e2a0ccde0b28bb64256e066';h=lambda b:hashlib.sha256(b).hexdigest()
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
 src=pefile.PE(str(S),fast_load=True);host=pefile.PE(str(P/'host-x86.exe'),fast_load=True);va=0x401bce;n=1018;a=0xe6ade0
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host base')
 f=src.get_data(a-src.OPTIONAL_HEADER.ImageBase,n);b=host.get_data(va-host.OPTIONAL_HEADER.ImageBase,n)
 if f!=(P/'source.bin').read_bytes() or len(b)!=n:raise ValueError('source mismatch')
 mt=(P/'host-x86.map').read_text()
 for name,expected in [('LoopSource',va),('RelationStub',0x401985),('CacheStub',0x4019aa),('CandidateStub',0x401a49)]:
  matches=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+name+r'\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
  if len(matches)!=1 or int(matches[0],16)!=expected:raise ValueError('symbol mismatch '+name)
 dis=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);ins=list(dis.disasm(f,a))
 if sum(i.size for i in ins)!=n or ins[-1].address+ins[-1].size!=a+n or bytes(ins[-1].bytes)!=b'\xc3':raise ValueError('incomplete source')
 mask=set()
 for address,original,linked in [(0xe6adf1,0xe57260,0x401985),(0xe6ae17,0xe67a50,0x4019aa),(0xe6ae3e,0xe67a50,0x4019aa),(0xe6b19a,0xe6a450,0x401a49)]:
  off=address-a
  if f[off]!=0xe8 or a+off+5+struct.unpack_from('<i',f,off+1)[0]!=original or va+off+5+struct.unpack_from('<i',b,off+1)[0]!=linked:raise ValueError('call mismatch')
  mask.update(range(off+1,off+5))
 for address,original,linked,value in [(0xe6b073,0x1485c9c,0x403006,0x37800000),(0xe6b0bf,0x14861d0,0x40300a,0x3f800000),(0xe6b127,0x1471b2c,0x40300e,0x3f000000)]:
  off=address-a
  if struct.unpack_from('<I',f,off)[0]!=original or struct.unpack_from('<I',b,off)[0]!=linked or struct.unpack('<I',src.get_data(original-src.OPTIONAL_HEADER.ImageBase,4))[0]!=value or struct.unpack('<I',host.get_data(linked-host.OPTIONAL_HEADER.ImageBase,4))[0]!=value:raise ValueError('constant mismatch')
  mask.update(range(off,off+4))
 if any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in mask):raise ValueError('nonpatch mismatch')
 receipt['linked_source']={'range':'E6ADE0..E6B1D9','sha256':h(f),'complete':True,'nonpatch_match':True,'calls':4,'constants':3}
 if (P/'host-x86.asm').read_text().count('cmp esp,SavedESP')!=5:raise ValueError('missing stack checks')
 corpus=(P/'cases.bin').read_bytes()
 if len(corpus)!=20:raise ValueError('case size')
 values=struct.unpack('<5I',corpus)
 if values!=(0x40400000,0x40000000,0x3f800000,0x3b800000,0x3b7fffff):raise ValueError('unexpected corpus')
 model=bytearray()
 for case,bits in enumerate(values):
  words=[0]*22;words[0:2]=[1,2]
  if case:
   distance=struct.unpack('<f',struct.pack('<I',bits))[0];words[2]=1;words[3]=struct.unpack('<I',struct.pack('<f',2.0 if case==4 else 2.0-distance))[0]
   if case!=4:words[4]=struct.unpack('<I',struct.pack('<f',distance*0.5))[0];words[7]=0x3f800000
   else:words[21]=1
   words[10:21]=[2,2,0,3,4,0,3,0,1,1,1]
  model+=struct.pack('<22I',*words)
 (O/'model.bin').write_bytes(model)
 user32=ctypes.WinDLL('user32',use_last_error=True);user32.CreateDesktopW.argtypes=[wintypes.LPCWSTR,wintypes.LPCWSTR,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p];user32.CreateDesktopW.restype=wintypes.HANDLE;user32.CloseDesktop.argtypes=[wintypes.HANDLE];user32.CloseDesktop.restype=wintypes.BOOL
 name='RustSporeFirstCandidate-'+uuid.uuid4().hex;desktop=user32.CreateDesktopW(name,None,None,0,0x00c3,None)
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
 if code!=0 or err or len(out)!=480:raise ValueError('native failure or size')
 receipt['rows']=[]
 for i in range(5):
  match=model[i*88:(i+1)*88]==out[i*96:i*96+88];cw,sw,mx=struct.unpack_from('<HHI',out,i*96+88);receipt['rows'].append({'case':i,'match':match,'control':hex(cw),'status':hex(sw),'mxcsr':hex(mx)})
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

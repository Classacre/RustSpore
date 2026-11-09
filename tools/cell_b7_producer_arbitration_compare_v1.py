"""Review gate: pending BBB880 branch versus synthetic byte-store model."""
from pathlib import Path
import ctypes,hashlib,json,platform,re,struct,subprocess,sys,uuid,pefile,capstone
from ctypes import wintypes
R=Path(__file__).resolve().parents[1];P=R/'target/cell-b7-producer-arbitration-prep-v1';O=R/'target/cell-b7-producer-arbitration-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');MH='5a8254bb4a055d4f6e1aaa63ce44a215370b82f73289419fedad0eb4b33b2ace';h=lambda b:hashlib.sha256(b).hexdigest()
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
 src=pefile.PE(str(S),fast_load=True);host=pefile.PE(str(P/'host-x86.exe'),fast_load=True);base=host.OPTIONAL_HEADER.ImageBase
 if base!=0x400000:raise ValueError('host base')
 mt=(P/'host-x86.map').read_text()
 for name,expected in [('ProducerSource',0x4019c0),('ArbitrationSource',0x402343),('ArbitrateThunk',0x4019a5),('UnexpectedCall',0x4019b9)]:
  matches=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+name+r'\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
  if len(matches)!=1 or int(matches[0],16)!=expected:raise ValueError('symbol mismatch '+name)
 matches=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+CellGlobal\s+([0-9A-Fa-f]{8})\s+host-x86\.obj\s*$',mt,re.M)
 if len(matches)!=1 or int(matches[0],16)!=0x40812e:raise ValueError('global symbol mismatch')
 dis=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
 receipt['linked_source']=[]
 for name,a,n,va,globaloff in [('producer',0xe6a450,0x983,0x4019c0,0x2d),('arbitration',0xe59b40,0xcc,0x402343,0x7a)]:
  f=src.get_data(a-src.OPTIONAL_HEADER.ImageBase,n);b=host.get_data(va-base,n)
  if f!=(P/(name+'.bin')).read_bytes() or len(b)!=n:raise ValueError('source mismatch '+name)
  ins=list(dis.disasm(f,a))
  if sum(i.size for i in ins)!=n or ins[-1].address+ins[-1].size!=a+n or bytes(ins[-1].bytes)!=b'\xc3':raise ValueError('incomplete source '+name)
  mask=set(range(globaloff,globaloff+4))
  if struct.unpack_from('<I',f,globaloff)[0]!=0x16b4be4 or struct.unpack_from('<I',b,globaloff)[0]!=0x40812e:raise ValueError('global relocation '+name)
  calls=[i for i in ins if i.mnemonic=='call']
  if len(calls)!=(42 if name=='producer' else 0):raise ValueError('call census '+name)
  for i in calls:
   off=i.address-a
   if not i.op_str.startswith('0x') or f[off]!=0xe8:raise ValueError('non-direct call')
   target=int(i.op_str,16);linked=0x4019a5 if i.address in (0xe6a4b2,0xe6a4db) else 0x4019b9
   if i.address in (0xe6a4b2,0xe6a4db) and target!=0xe59b40:raise ValueError('unexpected B7 source call')
   if va+off+5+struct.unpack_from('<i',b,off+1)[0]!=linked:raise ValueError('linked call mismatch')
   mask.update(range(off+1,off+5))
  for i in ins:
   if i.mnemonic.startswith('j') and i.op_str.startswith('0x') and not a<=int(i.op_str,16)<a+n:raise ValueError('external source jump')
  if any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in mask):raise ValueError('nonpatch mismatch '+name)
  receipt['linked_source'].append({'name':name,'length':n,'complete':True,'nonpatch_match':True,'calls':len(calls)})
 thunk=list(dis.disasm(host.get_data(0x19a5,0x14),0x4019a5));jumps=[int(i.op_str,16) for i in thunk if i.mnemonic=='jmp' and i.op_str.startswith('0x')]
 if 0x402343 not in jumps:raise ValueError('arbitration thunk target')
 if (P/'host-x86.asm').read_text().count('cmp esp,SavedESP')!=5:raise ValueError('missing stack checks')
 corpus=(P/'cases.bin').read_bytes()
 if len(corpus)!=160:raise ValueError('case size')
 model=bytearray()
 for row in range(5):
  fa,fb,da,db,player,selector,olddepth,depth=struct.unpack_from('<8I',corpus,row*32)
  if db!=7 or fa not in (2,9) or fb!=3 or da not in (2,7) or depth!=0x3f000000:raise ValueError('unsupported fixture gates')
  words=[0xcccccccc]*12;words[2]=selector;words[5]=olddepth;calls=0
  if fa!=9 and fb!=9:
   calls=1;new=19 if player==0x10001 else 0;old=struct.unpack('<i',struct.pack('<I',selector))[0]
   if new>old or (new==old and struct.unpack('<f',struct.pack('<I',depth))[0]>struct.unpack('<f',struct.pack('<I',olddepth))[0]):words=[0x10001,0x20002,new,0,1,depth,0x3f800000,0x40000000,0x40400000,0x40800000,0x40a00000,0x40c00000]
  model+=struct.pack('<13I',*words,calls)
 (O/'model.bin').write_bytes(model)
 user32=ctypes.WinDLL('user32',use_last_error=True);user32.CreateDesktopW.argtypes=[wintypes.LPCWSTR,wintypes.LPCWSTR,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p];user32.CreateDesktopW.restype=wintypes.HANDLE;user32.CloseDesktop.argtypes=[wintypes.HANDLE];user32.CloseDesktop.restype=wintypes.BOOL
 name='RustSporeB7Producer-'+uuid.uuid4().hex;desktop=user32.CreateDesktopW(name,None,None,0,0x00c3,None)
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
 if code!=0 or err or len(out)!=300:raise ValueError('native failure or size')
 receipt['rows']=[]
 for i in range(5):
  match=model[i*52:(i+1)*52]==out[i*60:i*60+52];cw,sw,mx=struct.unpack_from('<HHI',out,i*60+52);receipt['rows'].append({'case':i,'match':match,'control':hex(cw),'status':hex(sw),'mxcsr':hex(mx)})
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

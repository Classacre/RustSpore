"""Review gate: pending BBB880 branch versus synthetic byte-store model."""
from pathlib import Path
import ctypes,hashlib,json,platform,re,struct,subprocess,sys,uuid,pefile,capstone
from ctypes import wintypes
R=Path(__file__).resolve().parents[1];P=R/'target/cell-sorted-update-prep-v1';O=R/'target/cell-sorted-update-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');MH='1707e814c4c269718a32787e20e89aa8fa728965c8e0e2e67c7652e8ce5cb3be';h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True)
for n in ('native.stdout.bin','native.stderr.bin','model.bin','model.stderr.bin'):(O/n).write_bytes(b'')
receipt={'status':'failed','checks':[]};pins=[]
try:
 if sys.platform!='win32' or platform.machine().lower() not in ('amd64','x86_64'):raise ValueError('Windows x64 required')
 mb=(P/'manifest.json').read_bytes()
 if h(mb)!=MH:raise ValueError('manifest mismatch')
 pins=[(Path(k),v) for k,v in json.loads(mb)['pins'].items()]+[(P/'manifest.json',MH)]+[(Path(k),v) for k,v in [('D:\\AI\\RustSpore\\crates\\spore-core\\src\\cell_fresh_sweep_registration_v1.rs', '0497f901974eb1ec82ed8a05519c943455180458971e77a969a295f43fb70d88'), ('D:\\AI\\RustSpore\\crates\\spore-core\\src\\cell_sweep_sorted_crossing_v1.rs', 'f3ee3d5bc00ad9ebb7317bd0cb636efa2faef9a792c8bbece808a4a9614f6edf'), ('D:\\AI\\RustSpore\\crates\\spore-core\\src\\cell_sweep_sorted_update_v1.rs', 'af9d0fd85635e62a86739d70efb1fd8bfa629ffd9fd574717efceed96ebaec76'), ('D:\\AI\\RustSpore\\crates\\spore-core\\examples\\cell_sweep_sorted_update_oracle_v1.rs', 'cc3ffc6e78cff0e9a5ee80d0db6757b10f643fe1ed02d69049881bf35491538a'), ('D:\\AI\\RustSpore\\target\\debug\\examples\\cell_sweep_sorted_update_oracle_v1.exe', '76981376fe68b10253b61ae035abc202eff6bf49075b7d3b90fde68bd24ba0a0')]]
 for path,pin in pins:
  v=h(path.read_bytes());receipt['checks'].append({'path':str(path),'sha256':v,'expected':pin})
  if v!=pin:raise ValueError('pin mismatch '+str(path))
 src=pefile.PE(str(S),fast_load=True);host=pefile.PE(str(P/'host-x86.exe'),fast_load=True);trap=0x401ca5
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host base')
 mt=(P/'host-x86.map').read_text();funcs=[('UpdateSource',0xbbb880,381,0x401cac,[(0xbbb959,0xbbb020,0x401e29),(0xbbb97e,0xbbb130,0x401f31),(0xbbb9a7,0xbbb240,0x402033),(0xbbb9d0,0xbbb320,0x402104)]),('MinLeft',0xbbb020,264,0x401e29,[(0xbbb097,0xbba9c0,trap)]),('MaxRight',0xbbb130,258,0x401f31,[(0xbbb1a5,0xbba9c0,trap)]),('MinRight',0xbbb240,209,0x402033,[(0xbbb2b3,0xbba9c0,trap),(0xbbb2ca,0xbbaa80,trap)]),('MaxLeft',0xbbb320,207,0x402104,[(0xbbb391,0xbba9c0,trap),(0xbbb3a8,0xbbaa80,trap)])]
 receipt['linked_source']=[]
 for name,a,n,va,calls in funcs:
  f=src.get_data(a-src.OPTIONAL_HEADER.ImageBase,n);b=host.get_data(va-host.OPTIONAL_HEADER.ImageBase,n)
  if f!=(P/(name+'.bin')).read_bytes() or len(b)!=n:raise ValueError('source mismatch '+name)
  matches=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+name+r'\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
  if len(matches)!=1 or int(matches[0],16)!=va:raise ValueError('symbol mismatch '+name)
  ins=list(capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32).disasm(f,a))
  if sum(i.size for i in ins)!=n or ins[-1].address+ins[-1].size!=a+n or bytes(ins[-1].bytes)!=b'\xc3':raise ValueError('incomplete source '+name)
  mask=set()
  for call,target,linked in calls:
   off=call-a
   if f[off]!=0xe8 or a+off+5+struct.unpack_from('<i',f,off+1)[0]!=target or va+off+5+struct.unpack_from('<i',b,off+1)[0]!=linked:raise ValueError('relocation mismatch '+name)
   mask.update(range(off+1,off+5))
  if any(x!=y for i,(x,y) in enumerate(zip(f,b)) if i not in mask):raise ValueError('nonpatch mismatch '+name)
  receipt['linked_source'].append({'name':name,'length':n,'complete':True,'nonpatch_match':True,'calls':len(calls)})
 matches=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+UnexpectedHelper\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',mt,re.M)
 if len(matches)!=1 or int(matches[0],16)!=trap:raise ValueError('trap mismatch')
 if (P/'host-x86.asm').read_text().count('cmp esp,SavedESP')!=4:raise ValueError('missing stack checks')
 corpus=(P/'cases.bin').read_bytes()
 if len(corpus)!=128:raise ValueError('case size')
 try:
  q=subprocess.run([str(R/'target/debug/examples/cell_sweep_sorted_update_oracle_v1.exe')],input=corpus,capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW);model,model_err,model_code=q.stdout,q.stderr,q.returncode
 except subprocess.TimeoutExpired as e:model,model_err,model_code=e.stdout or b'',e.stderr or b'','timeout'
 (O/'model.bin').write_bytes(model);(O/'model.stderr.bin').write_bytes(model_err);receipt['model']={'exit':model_code,'stdout_bytes':len(model),'sha256':h(model),'stderr_bytes':len(model_err)}
 if model_code!=0 or len(model)!=464:raise ValueError('Rust model failure')
 user32=ctypes.WinDLL('user32',use_last_error=True);user32.CreateDesktopW.argtypes=[wintypes.LPCWSTR,wintypes.LPCWSTR,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p];user32.CreateDesktopW.restype=wintypes.HANDLE;user32.CloseDesktop.argtypes=[wintypes.HANDLE];user32.CloseDesktop.restype=wintypes.BOOL
 name='RustSporeSortedUpdate-'+uuid.uuid4().hex;desktop=user32.CreateDesktopW(name,None,None,0,0x00c3,None)
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
 if code!=0 or len(out)!=496:raise ValueError('native failure or size')
 receipt['rows']=[]
 for i in range(4):
  match=model[i*116:(i+1)*116]==out[i*124:i*124+116];cw,sw,mx=struct.unpack_from('<HHI',out,i*124+116);receipt['rows'].append({'case':i,'match':match,'control':hex(cw),'status':hex(sw),'mxcsr':hex(mx)})
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

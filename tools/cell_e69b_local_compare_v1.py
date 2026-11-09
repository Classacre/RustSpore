"""Review gate for admitted 4098A0 / 41CAE0 constructor host."""
from pathlib import Path
import hashlib,json,struct,subprocess,sys,platform,re,pefile,capstone
R=Path(__file__).resolve().parents[1];P=R/'target/cell-e69b-local-prep-v2';O=R/'target/cell-e69b-local-result-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');SH='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';MH='566ec7eedbc3dbf1135a659eba8236a5294547d5b949ffd1e7211e988e017f22';h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True)
for n in ('model','native'):
 for st in ('stdout','stderr'):(O/(n+'.'+st+'.bin')).write_bytes(b'')
receipt={'status':'failed','platform':platform.platform(),'checks':[],'runs':{}};pins=[]
try:
 if sys.platform!='win32' or platform.machine().lower() not in ('amd64','x86_64'):raise ValueError('Windows x64 required')
 mb=(P/'manifest.json').read_bytes()
 if h(mb)!=MH:raise ValueError('manifest mismatch')
 manifest=json.loads(mb);pins=[(Path(k),v) for k,v in manifest['pins'].items()]+[(P/'manifest.json',MH),(S,SH)]
 for path,pin in pins:
  value=h(path.read_bytes());receipt['checks'].append({'path':str(path),'sha256':value,'expected':pin})
  if value!=pin:raise ValueError('pin mismatch '+str(path))
 src=pefile.PE(str(S),fast_load=True);ctor=src.get_data(0x4098a0-src.OPTIONAL_HEADER.ImageBase,127);copy=src.get_data(0x41cae0-src.OPTIONAL_HEADER.ImageBase,0xd8)
 if ctor!=(P/'ctor.bin').read_bytes() or copy!=(P/'copy.bin').read_bytes():raise ValueError('source bytes mismatch')
 # Both full functions must decode from first byte through complete RET. The
 # copy helper's RET 4 removes the single pushed matrix argument; the outer
 # constructor returns with a plain RET. Wrapper checks ESP before/after.
 dis=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
 for name,raw,start,tail in [('ctor',ctor,0x4098a0,b'\xc3'),('copy',copy,0x41cae0,b'\xc2\x04\x00')]:
  ins=list(dis.disasm(raw,start))
  if not ins or sum(i.size for i in ins)!=len(raw) or ins[-1].address+ins[-1].size!=start+len(raw) or bytes(ins[-1].bytes)!=tail or ins[-1].mnemonic!='ret':raise ValueError('incomplete function or return ABI: '+name)
 if ctor[0x6b]!=0x68 or ctor[0x73]!=0xe8:raise ValueError('expected push and call ABI absent')
 receipt['abi']={'ctor_return':'c3','copy_return':'c20400','copy_stack_cleanup_bytes':4,'wrapper_checks_esp':True} 
 mt=(P/'host-x86.map').read_text();symbols={'CtorSource':0x401515,'CopySource':0x401594,'Vector':0x403052,'Scalar':0x40305e,'Matrix':0x403062}
 for n,va in symbols.items():
  v=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+n+r'\s+([0-9A-Fa-f]{8})\s+(?:f\s+)?host-x86\.obj\s*$',mt,re.M)
  if len(v)!=1 or int(v[0],16)!=va:raise ValueError('symbol mismatch '+n)
 host=pefile.PE(str(P/'host-x86.exe'),fast_load=True)
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('host base')
 b=host.get_data(0x1515,127);patch=[0x2a,0x39,0x49,0x59,0x6c,0x74];mask={i for x in patch for i in range(x,x+4)}
 if len(b)!=127 or any(x!=y for i,(x,y) in enumerate(zip(ctor,b)) if i not in mask):raise ValueError('nonpatch byte mismatch')
 for off,expected in zip(patch[:5],[0x403052,0x403056,0x40305a,0x40305e,0x403062]):
  if struct.unpack_from('<I',b,off)[0]!=expected:raise ValueError('global operand mismatch')
 if 0x401515+0x78+struct.unpack_from('<i',b,0x74)[0]!=0x401594:raise ValueError('copy call target mismatch')
 if host.get_data(0x1594,len(copy))!=copy:raise ValueError('linked copy mismatch')
 receipt['linked_slices']={'nonpatch_match':True,'globals_match':True,'copy_call_match':True,'copy_bytes_match':True}
 corpus=(P/'cases.bin').read_bytes();model=R/'target/debug/examples/cell_e69b_local_oracle_v1.exe'
 for name,cmd,inp in [('model',[str(model)],corpus),('native',[str(P/'host-x86.exe')],None)]:
  try:
   q=subprocess.run(cmd,input=inp,capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW);out,err,code=q.stdout,q.stderr,q.returncode
  except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
  (O/(name+'.stdout.bin')).write_bytes(out);(O/(name+'.stderr.bin')).write_bytes(err);receipt['runs'][name]={'exit':code,'stdout_bytes':len(out),'stderr_bytes':len(err),'stdout_sha256':h(out)}
  if code!=0:raise ValueError(name+' failed')
 model=(O/'model.stdout.bin').read_bytes();native=(O/'native.stdout.bin').read_bytes()
 if len(model)!=224 or len(native)!=256:raise ValueError('output size')
 receipt['rows']=[]
 for i in range(4):
  a=model[i*56:(i+1)*56];b=native[i*64:i*64+56];cw,sw,mx=struct.unpack_from('<HHI',native,i*64+56)
  receipt['rows'].append({'case':i,'match':a==b,'control':hex(cw),'status':hex(sw),'mxcsr':hex(mx)})
  if a!=b or cw!=0x27f or sw&0x3800 or mx&~0x3f!=0x1f80:raise ValueError('row/control mismatch '+str(i))
 receipt['status']='matched'
except Exception as e:receipt['error']=repr(e)
finally:
 receipt['post_checks']=[]
 for path,pin in pins:
  try:
   value=h(path.read_bytes());receipt['post_checks'].append({'path':str(path),'sha256':value,'expected':pin})
   if value!=pin:receipt['status']='failed';receipt['post_error']='changed pin '+str(path)
  except Exception as e:receipt['status']='failed';receipt['post_error']=repr(e)
 (O/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt,indent=2));sys.exit(receipt['status']!='matched')

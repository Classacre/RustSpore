"""Explicit review gate: only run after independent host/ABI review."""
import hashlib,json,struct,subprocess,sys,platform,re
import pefile
from pathlib import Path
R=Path(__file__).resolve().parents[1]; P=R/'target/cell-pair-coordinator-prep-v2'; O=R/'target/cell-pair-coordinator-result-v1'
H=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
EXPECTED={'host-x86.exe':'b7d07db8199697511bf77b2ebb3ae5ad4aeeb9619d0891eb547b81b8fdb6f5af','host-x86.asm':'3573563e776f3c26a5385429f7cc3210f26a79583d2b9bfbbe1538c5e5071e68','fragment.bin':'f69a63d2cfddb13fc1aa2454e1b6edd6cadc2c12bbacebfead9ce278969642ed','cases.bin':'dddca84e5bcc8d22181b3ee5fa593715e193ada8ab17c5f860bce47fc5d4040e','manifest.json':'73c8de9fc9c0cfd82eee50b5c583710f45df14e6985801114432e285f97b9438','host-x86.map':'b2b8483a7f8bef41b84a017bb51b08569caf1362fedb0e707ea4168ee3b40eb8'}
MODEL=R/'target/debug/examples/cell_pair_coordinator_oracle_v1.exe'; MODEL_HASH='43e6bfe4885c217ccf24eda8952e4189fa004bc51b59e70650df40b67891a62e'
SOURCES={R/'crates/spore-bevy/src/cell_source_pair_coordinator_v1.rs':'16c03de949dddd1f7e0aea70b88d261143e66445601abebe636284a7f4dfb564',R/'crates/spore-bevy/src/cell_source_player_pool_fixture_v1.rs':'b4917b3b96f4aa9079650bf402974bd7f9cb0486a2ff588717bdffa6fbdf13db',R/'crates/spore-bevy/examples/cell_pair_coordinator_oracle_v1.rs':'27b0621ab2c3d1829b4e7d7e83d76992b7fcefbf99938a7e4b8b8f0f4ecf506f'}
SOURCE=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
SOURCE_HASH='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
PINNED=[(P/k,v) for k,v in EXPECTED.items()]+[(MODEL,MODEL_HASH),(SOURCE,SOURCE_HASH)]+list(SOURCES.items())
assert not O.exists();O.mkdir(parents=True)
for name in ('model','native'):
 for stream in ('stdout','stderr'):(O/(name+'.'+stream+'.bin')).write_bytes(b'')
receipt={'status':'failed','platform':platform.platform(),'checks':[],'runs':{}}
try:
 if sys.platform!='win32' or platform.machine().lower() not in ('amd64','x86_64'):raise ValueError('Windows x64 required')
 for path,pin in PINNED:
  actual=H(path);receipt['checks'].append({'path':str(path),'sha256':actual,'expected':pin});assert actual==pin, str(path)
 pe=pefile.PE(str(SOURCE),fast_load=True)
 fragment=pe.get_data(0xe7f170-pe.OPTIONAL_HEADER.ImageBase,284)
 if fragment!=(P/'fragment.bin').read_bytes():raise ValueError('source PE fragment mismatch')
 receipt['source_fragment_match']=True
 # Pinned linker map names and addresses are independently specified here. A
 # manifest/map change cannot silently redefine the approved patch sites.
 symbols={'SourceFragment':0x401be4,'HolderInit':0x401a0f,'Resolve':0x401a2c,'Selector':0x401a9a,'Event':0x401b92,'Cleanup':0x401bc0}
 sites=[(0xe7f18e,'HolderInit'),(0xe7f19f,'Resolve'),(0xe7f1ad,'HolderInit'),(0xe7f1be,'Resolve'),(0xe7f203,'Selector'),(0xe7f211,'Selector'),(0xe7f233,'Event'),(0xe7f23f,'Cleanup'),(0xe7f24f,'Cleanup'),(0xe7f258,'Cleanup'),(0xe7f268,'Cleanup'),(0xe7f271,'Cleanup'),(0xe7f281,'Cleanup')]
 manifest=json.loads((P/'manifest.json').read_text())
 if manifest['patched_call_sites']!=[[hex(a),n] for a,n in sites]:raise ValueError('manifest patch sites mismatch')
 map_text=(P/'host-x86.map').read_text()
 for name,address in symbols.items():
  hits=re.findall(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+'+re.escape(name)+r'\s+([0-9A-Fa-f]{8})\s+f\s+host-x86\.obj\s*$',map_text,re.M)
  if len(hits)!=1 or int(hits[0],16)!=address:raise ValueError('map symbol mismatch: '+name)
 host=pefile.PE(str(P/'host-x86.exe'),fast_load=True)
 if host.OPTIONAL_HEADER.ImageBase!=0x400000:raise ValueError('unexpected host image base')
 linked=host.get_data(symbols['SourceFragment']-host.OPTIONAL_HEADER.ImageBase,len(fragment))
 if len(linked)!=len(fragment):raise ValueError('short linked fragment')
 operands=set();targets=[]
 for address,name in sites:
  off=address-0xe7f170
  if fragment[off] not in (0xe8,0xe9) or linked[off]!=fragment[off]:raise ValueError('patch opcode mismatch')
  operands.update(range(off+1,off+5))
  target=symbols['SourceFragment']+off+5+struct.unpack_from('<i',linked,off+1)[0]
  targets.append({'site':hex(address),'stub':name,'target':hex(target)})
  if target!=symbols[name]:raise ValueError('rel32 target mismatch: '+hex(address))
 if any(a!=b for i,(a,b) in enumerate(zip(fragment,linked)) if i not in operands):raise ValueError('non-patch linked byte mismatch')
 receipt['linked_fragment']={'non_patch_bytes_match':True,'rel32_targets':targets}
 data=(P/'cases.bin').read_bytes()
 for name,cmd,inp in [('model',[str(MODEL)],data),('native',[str(P/'host-x86.exe')],None)]:
  try:
   q=subprocess.run(cmd,input=inp,capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW)
   out,err,code=q.stdout,q.stderr,q.returncode
  except subprocess.TimeoutExpired as e:
   out,err,code=e.stdout or b'',e.stderr or b'','timeout'
  (O/(name+'.stdout.bin')).write_bytes(out);(O/(name+'.stderr.bin')).write_bytes(err)
  receipt['runs'][name]={'exit':code,'stdout_bytes':len(out),'stderr_bytes':len(err),'stdout_sha256':hashlib.sha256(out).hexdigest()}
  if code!=0:raise ValueError(name+' failed')
 model=(O/'model.stdout.bin').read_bytes();native=(O/'native.stdout.bin').read_bytes()
 if len(model)!=7*43*4 or len(native)!=7*45*4:raise ValueError('output length mismatch')
 receipt['rows']=[]
 for i in range(7):
  m=struct.unpack_from('<43I',model,i*172);n=struct.unpack_from('<45I',native,i*180)
  control=n[43]&0xffff;status=n[43]>>16;mx=n[44]
  row={'case':i,'match':m==n[:43],'x87_control':hex(control),'x87_status':hex(status),'mxcsr':hex(mx)};receipt['rows'].append(row)
  if m!=n[:43] or control!=0x27f or status&0x3800 or (mx&~0x3f)!=0x1f80:raise ValueError('row mismatch or control failure '+str(i))
 receipt['status']='matched'
except Exception as e:
 receipt['error']=repr(e)
finally:
 receipt['post_checks']=[]
 for path,pin in PINNED:
  try:
   actual=H(path);receipt['post_checks'].append({'path':str(path),'sha256':actual,'expected':pin})
   if actual!=pin:receipt['status']='failed';receipt['post_check_error']='pin changed: '+str(path)
  except Exception as e:
   receipt['status']='failed';receipt['post_check_error']=repr(e)
 (O/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt,indent=2));sys.exit(receipt['status']!='matched')

from pathlib import Path
import hashlib, json, subprocess, struct, sys
R=Path(__file__).resolve().parents[1]; O=R/'target/cell-actor-frame-update-rust-result-v1'; h=lambda b:hashlib.sha256(b).hexdigest()
pins={'target/cell-actor-frame-update-result-v1/receipt.json': '71fef1ede8d4c0a68c9aa84a82c0f6a8a7240202641f8879b54fe6b5395b0fd5', 'target/cell-actor-frame-update-result-v1/native.stdout.bin': '9e04c4cb08cd166f46dca63e93b87a5ee2744e5701b7ef07cfbd8003403856bb', 'target/cell-actor-frame-update-prep-v2/cases.bin': '87c55150f7f908d05780b25c96a3dd76eb7cb7eab92eb30f469bfe6a4b6899ea', 'crates/spore-core/src/cell_registration_prefix_v1.rs': '3e274119577a50896b40a3a1922f122f43079fde6a75ef38b23f3d7a5fe1636a', 'crates/spore-core/src/cell_actor_frame_update_v1.rs': '5c17f975b036365bb62a11551b79f77938697c9266a8e670469ed487397dd62b', 'crates/spore-core/examples/cell_actor_frame_update_oracle_v1.rs': '6a8df705ece7f67e6b8a7393506b407b69869158921d5d80841cd1029d73fda7', 'target/debug/examples/cell_actor_frame_update_oracle_v1.exe': '6f3ef5ff6222591d2132d0faa4accd3de389b8e55e90620bd686df31108a9619'}
assert not O.exists(); O.mkdir(parents=True)
receipt={'status':'failed','checks':[],'post_checks':[],'scope':'Rust actor caller API compared to preserved four native rows; no native process run'}
for name in ('model.stdout.bin','model.stderr.bin'):(O/name).write_bytes(b'')
try:
 for p,expected in pins.items():
  actual=h((R/p).read_bytes());receipt['checks'].append({'path':p,'sha256':actual,'expected':expected})
  if actual!=expected:raise ValueError('pin mismatch '+p)
 old=json.loads((R/'target/cell-actor-frame-update-result-v1/receipt.json').read_text())
 if old.get('status')!='matched':raise ValueError('old receipt not matched')
 try:
  proc=subprocess.run([str(R/'target/debug/examples/cell_actor_frame_update_oracle_v1.exe')],input=(R/'target/cell-actor-frame-update-prep-v2/cases.bin').read_bytes(),capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW)
  out,err,code=proc.stdout,proc.stderr,proc.returncode
 except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
 (O/'model.stdout.bin').write_bytes(out);(O/'model.stderr.bin').write_bytes(err);receipt['run']={'exit':code,'stdout_sha256':h(out),'bytes':len(out)}
 if code!=0 or len(out)!=144:raise ValueError('model failed or size mismatch')
 native=(R/'target/cell-actor-frame-update-result-v1/native.stdout.bin').read_bytes()
 if len(native)!=176:raise ValueError('native size mismatch')
 receipt['rows']=[]
 for i in range(4):
  match=out[i*36:(i+1)*36]==native[i*44:i*44+36];cw,sw,mx=struct.unpack_from('<HHI',native,i*44+36)
  receipt['rows'].append({'case':i,'match':match,'x87_control':hex(cw),'x87_status':hex(sw),'mxcsr':hex(mx)})
  if not match or cw!=0x27f or sw&0x3800 or mx&~0x3f!=0x1f80:raise ValueError('row mismatch')
 receipt['status']='matched'
except Exception as e:receipt['error']=repr(e)
finally:
 for p,expected in pins.items():
  try:
   actual=h((R/p).read_bytes());receipt['post_checks'].append({'path':p,'sha256':actual,'expected':expected})
   if actual!=expected:receipt['status']='failed';receipt['post_error']='pin changed '+p
  except Exception as e:receipt['status']='failed';receipt['post_error']=repr(e)
 (O/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt,indent=2));sys.exit(receipt['status']!='matched')

from pathlib import Path
import hashlib, json, subprocess, struct, sys
R=Path(__file__).resolve().parents[1]; O=R/'target/cell-sweep-remove-rust-result-v1'; h=lambda b:hashlib.sha256(b).hexdigest()
pins={
 'target/cell-sweep-remove-result-v1/receipt.json':'b31f75f07d6f8fc12cd2075446d022e11c7004286d2e4acde3bb79568bb341bb',
 'target/cell-sweep-remove-result-v1/native.stdout.bin':'874cc173ad3b3e3ae0170ebc757706735a1be36c88fc6b7e0330ff0a16fa4109',
 'target/cell-sweep-remove-prep-v2/cases.bin':'2aa27f4095a0d8769a88727b59de2d410e8308162a2c3b4c791fc49b898e8ab4',
 'crates/spore-core/src/cell_fresh_sweep_registration_v1.rs':'0497f901974eb1ec82ed8a05519c943455180458971e77a969a295f43fb70d88',
 'crates/spore-core/src/cell_sweep_remove_v1.rs':'bf3881c49f14f84f0c7b7ee75772e6a6494da8a21e171647803a003865ba2060',
 'crates/spore-core/examples/cell_sweep_remove_oracle_v1.rs':'48da460a19d9b364dad04711a1f73536ef33233c52f6a81268d599982df997cc',
 'target/debug/examples/cell_sweep_remove_oracle_v1.exe':'6a7bd773a40e7ef94260e7de7b4fc7335969362fdcbb11d7ff6c85a73a6297e9'}
assert not O.exists(); O.mkdir(parents=True)
receipt={'status':'failed','checks':[],'post_checks':[],'scope':'Rust removal API compared to preserved four native rows; no native process run'}
for name in ('model.stdout.bin','model.stderr.bin'):(O/name).write_bytes(b'')
try:
 for p,expected in pins.items():
  actual=h((R/p).read_bytes());receipt['checks'].append({'path':p,'sha256':actual,'expected':expected})
  if actual!=expected:raise ValueError('pin mismatch '+p)
 old=json.loads((R/'target/cell-sweep-remove-result-v1/receipt.json').read_text())
 if old.get('status')!='matched':raise ValueError('old receipt not matched')
 try:
  proc=subprocess.run([str(R/'target/debug/examples/cell_sweep_remove_oracle_v1.exe')],input=(R/'target/cell-sweep-remove-prep-v2/cases.bin').read_bytes(),capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW)
  out,err,code=proc.stdout,proc.stderr,proc.returncode
 except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
 (O/'model.stdout.bin').write_bytes(out);(O/'model.stderr.bin').write_bytes(err);receipt['run']={'exit':code,'stdout_sha256':h(out),'bytes':len(out)}
 if code!=0 or len(out)!=688:raise ValueError('model failed or size mismatch')
 native=(R/'target/cell-sweep-remove-result-v1/native.stdout.bin').read_bytes()
 if len(native)!=720:raise ValueError('native size mismatch')
 receipt['rows']=[]
 for i in range(4):
  match=out[i*172:(i+1)*172]==native[i*180:i*180+172];cw,sw,mx=struct.unpack_from('<HHI',native,i*180+172)
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

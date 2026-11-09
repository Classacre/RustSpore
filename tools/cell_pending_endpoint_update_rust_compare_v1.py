from pathlib import Path
import hashlib, json, subprocess, struct, sys
R=Path(__file__).resolve().parents[1]; O=R/'target/cell-pending-endpoint-update-rust-result-v1'; h=lambda b:hashlib.sha256(b).hexdigest()
pins={
 'target/cell-pending-endpoint-update-result-v1/receipt.json':'c155de4b3a6b2698067d3b05c90b5cdd275ef3635042f8ec003354701cead22d',
 'target/cell-pending-endpoint-update-result-v1/native.stdout.bin':'0d092177456761db59cd45806935bc0a9c24d0ee2df832500b806a3c16f79074',
 'target/cell-pending-endpoint-update-prep-v2/cases.bin':'51265bb66b8abe244cc9a289242b5a31ca40c76720ec749aac2d9838c6fa2db3',
 'crates/spore-core/src/cell_fresh_sweep_registration_v1.rs':'0497f901974eb1ec82ed8a05519c943455180458971e77a969a295f43fb70d88',
 'crates/spore-core/src/cell_sweep_pending_update_v1.rs':'93eff32029f46dcf04b71390515b2586e8ab0ab0c57c086ea668a2b4cca2c8f4',
 'crates/spore-core/examples/cell_sweep_pending_update_oracle_v1.rs':'5657320011fd1e6a77508435743f33f11d197888531898b80e176a99f7bbac6d',
 'target/debug/examples/cell_sweep_pending_update_oracle_v1.exe':'03214aa76df98c6f956900a1b759bfdb7e039a3f5c80ff85a125389fd7f41a24'}
assert not O.exists(); O.mkdir(parents=True)
receipt={'status':'failed','checks':[],'post_checks':[],'scope':'Rust pending update API compared to preserved four native rows; no native process run'}
for name in ('model.stdout.bin','model.stderr.bin'):(O/name).write_bytes(b'')
try:
 for p,expected in pins.items():
  actual=h((R/p).read_bytes());receipt['checks'].append({'path':p,'sha256':actual,'expected':expected})
  if actual!=expected:raise ValueError('pin mismatch '+p)
 old=json.loads((R/'target/cell-pending-endpoint-update-result-v1/receipt.json').read_text())
 if old.get('status')!='matched':raise ValueError('old receipt not matched')
 try:
  proc=subprocess.run([str(R/'target/debug/examples/cell_sweep_pending_update_oracle_v1.exe')],input=(R/'target/cell-pending-endpoint-update-prep-v2/cases.bin').read_bytes(),capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW)
  out,err,code=proc.stdout,proc.stderr,proc.returncode
 except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
 (O/'model.stdout.bin').write_bytes(out);(O/'model.stderr.bin').write_bytes(err);receipt['run']={'exit':code,'stdout_sha256':h(out),'bytes':len(out)}
 if code!=0 or len(out)!=320:raise ValueError('model failed or size mismatch')
 native=(R/'target/cell-pending-endpoint-update-result-v1/native.stdout.bin').read_bytes()
 if len(native)!=352:raise ValueError('native size mismatch')
 receipt['rows']=[]
 for i in range(4):
  match=out[i*80:(i+1)*80]==native[i*88:i*88+80];cw,sw,mx=struct.unpack_from('<HHI',native,i*88+80)
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

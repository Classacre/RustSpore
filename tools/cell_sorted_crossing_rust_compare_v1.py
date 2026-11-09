from pathlib import Path
import hashlib, json, subprocess, struct, sys
R=Path(__file__).resolve().parents[1]; O=R/'target/cell-sorted-crossing-rust-result-v1'; h=lambda b:hashlib.sha256(b).hexdigest()
pins={'target/cell-sorted-crossing-result-v1/receipt.json': 'aac7ff9d17613a647591507eb667f17441c0a6329666f543412460a6922b4f5b', 'target/cell-sorted-crossing-result-v1/native.stdout.bin': '842f0d6a3078f5ad7deeef8ecf7685e4cea47192640ceadab4f65db1a7dea619', 'target/cell-sorted-crossing-prep-v1/cases.bin': '09f82ae7a3e437be05b08a8087779b06915a75e928b83b8a15a254a3f50ab72d', 'crates/spore-core/src/cell_fresh_sweep_registration_v1.rs': '0497f901974eb1ec82ed8a05519c943455180458971e77a969a295f43fb70d88', 'crates/spore-core/src/cell_sweep_sorted_crossing_v1.rs': 'f3ee3d5bc00ad9ebb7317bd0cb636efa2faef9a792c8bbece808a4a9614f6edf', 'crates/spore-core/examples/cell_sweep_sorted_crossing_oracle_v1.rs': '5f472ccafa25b8b5151c28d7257e26eb5d577a5013ae462d620926d2566ea719', 'target/debug/examples/cell_sweep_sorted_crossing_oracle_v1.exe': '3954beb8c47b850e8bdfacf2c9c716ba28484bc4fd731d3ab4530989e5dc10e7'}
assert not O.exists(); O.mkdir(parents=True)
receipt={'status':'failed','checks':[],'post_checks':[],'scope':'Rust crossing API compared to preserved four native rows; no native process run'}
for name in ('model.stdout.bin','model.stderr.bin'):(O/name).write_bytes(b'')
try:
 for p,expected in pins.items():
  actual=h((R/p).read_bytes());receipt['checks'].append({'path':p,'sha256':actual,'expected':expected})
  if actual!=expected:raise ValueError('pin mismatch '+p)
 old=json.loads((R/'target/cell-sorted-crossing-result-v1/receipt.json').read_text())
 if old.get('status')!='matched':raise ValueError('old receipt not matched')
 try:
  proc=subprocess.run([str(R/'target/debug/examples/cell_sweep_sorted_crossing_oracle_v1.exe')],input=(R/'target/cell-sorted-crossing-prep-v1/cases.bin').read_bytes(),capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW)
  out,err,code=proc.stdout,proc.stderr,proc.returncode
 except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
 (O/'model.stdout.bin').write_bytes(out);(O/'model.stderr.bin').write_bytes(err);receipt['run']={'exit':code,'stdout_sha256':h(out),'bytes':len(out)}
 if code!=0 or len(out)!=336:raise ValueError('model failed or size mismatch')
 native=(R/'target/cell-sorted-crossing-result-v1/native.stdout.bin').read_bytes()
 if len(native)!=368:raise ValueError('native size mismatch')
 receipt['rows']=[]
 for i in range(4):
  match=out[i*84:(i+1)*84]==native[i*92:i*92+84];cw,sw,mx=struct.unpack_from('<HHI',native,i*92+84)
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

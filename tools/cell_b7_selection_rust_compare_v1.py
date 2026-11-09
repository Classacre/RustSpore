from pathlib import Path
import hashlib, json, subprocess, struct, sys
R=Path(__file__).resolve().parents[1]; O=R/'target/cell-b7-selection-rust-result-v1'; h=lambda b:hashlib.sha256(b).hexdigest()
pins={'target/cell-b7-producer-arbitration-result-v1/receipt.json': '526e8aaa51d01d49d12bfa977c3ea8e817adf63793f66d3ddf3f250d4bf1f1bf', 'target/cell-b7-producer-arbitration-result-v1/native.stdout.bin': '1b96b5f7d20623c148fe2646a449505fa5c99977993dccd8867cf8c2dccc1462', 'target/cell-b7-producer-arbitration-prep-v1/cases.bin': '0ad5f403bbbf2c582bd1be0935ccf543c619b43b6043ca56e01fb5ee561c92f9', 'crates/spore-core/src/cell_feature_first_candidate_v1.rs': 'd9c7deeba7ce6d9db6bd83b194201d192274b03be39a6446303f4e67e1e566c3', 'crates/spore-core/src/cell_feeding_producer_prefix_v1.rs': 'b945256d9bbee099e666bbfa39143ce49ca66fcd3642220b6e334b170d0d6df5', 'crates/spore-core/src/cell_feeding_contact_arbitration_e59b40_v1.rs': '48a16a5b7a715ab9ffc394b96b2d7f46700daed07632ad1b2ed22f545f8a3df0', 'crates/spore-core/src/cell_feeding_b7_selection_v1.rs': '347207a0067bc3b8ade8831820433b963557d2292e83c37aeddc66984168f4be', 'crates/spore-core/examples/cell_feeding_b7_selection_oracle_v1.rs': '8c9a623081cd751efb5329d7e43222499bb039256411eaef7d235ea918e740de', 'target/debug/examples/cell_feeding_b7_selection_oracle_v1.exe': '35c4fec557170a206900457da7f71d4129bf5a9a3c2372eeb1a44cd68255a01d'}
assert not O.exists(); O.mkdir(parents=True)
receipt={'status':'failed','checks':[],'post_checks':[],'scope':'Rust early-B7 selection compared to 5 retained native rows; supplied nondegenerate geometry, no native rerun; no native process run'}
for name in ('model.stdout.bin','model.stderr.bin'):(O/name).write_bytes(b'')
try:
 for p,expected in pins.items():
  actual=h((R/p).read_bytes());receipt['checks'].append({'path':p,'sha256':actual,'expected':expected})
  if actual!=expected:raise ValueError('pin mismatch '+p)
 old=json.loads((R/'target/cell-b7-producer-arbitration-result-v1/receipt.json').read_text())
 if old.get('status')!='matched':raise ValueError('old receipt not matched')
 try:
  proc=subprocess.run([str(R/'target/debug/examples/cell_feeding_b7_selection_oracle_v1.exe')],input=(R/'target/cell-b7-producer-arbitration-prep-v1/cases.bin').read_bytes(),capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW)
  out,err,code=proc.stdout,proc.stderr,proc.returncode
 except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
 (O/'model.stdout.bin').write_bytes(out);(O/'model.stderr.bin').write_bytes(err);receipt['run']={'exit':code,'stdout_sha256':h(out),'bytes':len(out)}
 if code!=0 or len(out)!=260:raise ValueError('model failed or size mismatch')
 native=(R/'target/cell-b7-producer-arbitration-result-v1/native.stdout.bin').read_bytes()
 if len(native)!=300:raise ValueError('native size mismatch')
 receipt['rows']=[]
 for i in range(5):
  match=out[i*52:(i+1)*52]==native[i*60:i*60+52];cw,sw,mx=struct.unpack_from('<HHI',native,i*60+52)
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

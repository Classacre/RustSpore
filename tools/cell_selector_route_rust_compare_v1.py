from pathlib import Path
import hashlib, json, subprocess, struct, sys
R=Path(__file__).resolve().parents[1]; O=R/'target/cell-selector-route-rust-result-v1'; h=lambda b:hashlib.sha256(b).hexdigest()
pins={'target/cell-selector-route-result-v1/receipt.json': '5fb1474c53f2474ff4e78d3de9f89956262030131b57a212a4db2081d1a35431', 'target/cell-selector-route-result-v1/native.stdout.bin': '6e40c5fb694dc6a6cb31ec42e0b8b1a085b9f7da9319fbae863c82dd0f921ce9', 'target/cell-selector-route-prep-v1/cases.bin': '2aca99c27d66242c41739883b7c4084c4610a27e8073fa92b59d238148413b17', 'crates/spore-core/src/cell_selector_route_v1.rs': '55099ecb63df8798fadf1978d6b6740cf9119eb56df3bdf3e0ba915d25f885fd', 'crates/spore-core/examples/cell_selector_route_oracle_v1.rs': '185e9a0fe26d67c999dc6dc7100e5f6f3e022f387de7e7e09a5a60e39afc659e', 'target/debug/examples/cell_selector_route_oracle_v1.exe': 'be4178330d2021fa0246a7dd015497bc5f28277c11094ea4ee96c098df550ea1'}
assert not O.exists(); O.mkdir(parents=True)
receipt={'status':'failed','checks':[],'post_checks':[],'scope':'Rust selector route predicate compared to 17 retained native rows; no native process run'}
for name in ('model.stdout.bin','model.stderr.bin'):(O/name).write_bytes(b'')
try:
 for p,expected in pins.items():
  actual=h((R/p).read_bytes());receipt['checks'].append({'path':p,'sha256':actual,'expected':expected})
  if actual!=expected:raise ValueError('pin mismatch '+p)
 old=json.loads((R/'target/cell-selector-route-result-v1/receipt.json').read_text())
 if old.get('status')!='matched':raise ValueError('old receipt not matched')
 try:
  proc=subprocess.run([str(R/'target/debug/examples/cell_selector_route_oracle_v1.exe')],input=(R/'target/cell-selector-route-prep-v1/cases.bin').read_bytes(),capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW)
  out,err,code=proc.stdout,proc.stderr,proc.returncode
 except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
 (O/'model.stdout.bin').write_bytes(out);(O/'model.stderr.bin').write_bytes(err);receipt['run']={'exit':code,'stdout_sha256':h(out),'bytes':len(out)}
 if code!=0 or len(out)!=952:raise ValueError('model failed or size mismatch')
 native=(R/'target/cell-selector-route-result-v1/native.stdout.bin').read_bytes()
 if len(native)!=1088:raise ValueError('native size mismatch')
 receipt['rows']=[]
 for i in range(17):
  match=out[i*56:(i+1)*56]==native[i*64:i*64+56];cw,sw,mx=struct.unpack_from('<HHI',native,i*64+56)
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

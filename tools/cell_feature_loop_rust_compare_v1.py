from pathlib import Path
import hashlib, json, subprocess, struct, sys
R=Path(__file__).resolve().parents[1]; O=R/'target/cell-feature-loop-rust-result-v1'; h=lambda b:hashlib.sha256(b).hexdigest()
pins={'target/cell-feature-loop-skeleton-result-v3/receipt.json': '37b8f6f7b6be5e0370ea453bc212c4b2dc030ef7b9f96bac373df42d5de19a79', 'target/cell-feature-loop-skeleton-result-v3/native.stdout.bin': 'f74f5f59d70796c2d130c65212c1fe457cfab7ee23032fb1bfaa633ffc182387', 'target/cell-feature-loop-skeleton-prep-v3/cases.bin': '4ef07a790f14abbe596ce06ce3fb8e8423ff147dcb4e76e68c846774c1730e64', 'crates/spore-core/src/cell_feature_loop_v1.rs': '1fad78914513d8d0655bc658a3f9d9c416e24c153ddd3460c55c882d2409b76b', 'crates/spore-core/examples/cell_feature_loop_oracle_v1.rs': '00b9066428694ce225f659f564b54874cf596bc42b5d1f1a9e3bbef3cc271f20', 'target/debug/examples/cell_feature_loop_oracle_v1.exe': '08068fc9a121a8142faef6ec095603f338f10a7bc17b7469e5676223aaa7cd5b'}
assert not O.exists(); O.mkdir(parents=True)
receipt={'status':'failed','checks':[],'post_checks':[],'scope':'Rust feature traversal compared to 5 retained instrumented native rows; no native process run'}
for name in ('model.stdout.bin','model.stderr.bin'):(O/name).write_bytes(b'')
try:
 for p,expected in pins.items():
  actual=h((R/p).read_bytes());receipt['checks'].append({'path':p,'sha256':actual,'expected':expected})
  if actual!=expected:raise ValueError('pin mismatch '+p)
 old=json.loads((R/'target/cell-feature-loop-skeleton-result-v3/receipt.json').read_text())
 if old.get('status')!='matched':raise ValueError('old receipt not matched')
 try:
  proc=subprocess.run([str(R/'target/debug/examples/cell_feature_loop_oracle_v1.exe')],input=(R/'target/cell-feature-loop-skeleton-prep-v3/cases.bin').read_bytes(),capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW)
  out,err,code=proc.stdout,proc.stderr,proc.returncode
 except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
 (O/'model.stdout.bin').write_bytes(out);(O/'model.stderr.bin').write_bytes(err);receipt['run']={'exit':code,'stdout_sha256':h(out),'bytes':len(out)}
 if code!=0 or len(out)!=420:raise ValueError('model failed or size mismatch')
 native=(R/'target/cell-feature-loop-skeleton-result-v3/native.stdout.bin').read_bytes()
 if len(native)!=460:raise ValueError('native size mismatch')
 receipt['rows']=[]
 for i in range(5):
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

from pathlib import Path
import hashlib, json, subprocess, struct, sys
R=Path(__file__).resolve().parents[1]; O=R/'target/cell-feature-first-candidate-rust-result-v1'; h=lambda b:hashlib.sha256(b).hexdigest()
pins={'target/cell-feature-first-candidate-result-v1/receipt.json': '4b90a8998e1c122c6bf6c078c5b7964b37c61d47ebb71fbe62eeefcc3f26db96', 'target/cell-feature-first-candidate-result-v1/native.stdout.bin': 'bc4c06052a0c87dfd9666bc4ff9e4d4c832a07bffa0a0dc25dcfb5555ce2e80f', 'target/cell-feature-first-candidate-prep-v3/cases.bin': '08126914cca6cc090c64d534834a0e55f1341f328c375c001e2cc8f542007d13', 'crates/spore-core/src/native_sse.rs': 'eedac9f108e411f55abe01b1b05d5d690f950a954c16409b2222f15f00298129', 'crates/spore-core/src/native_x87.rs': '08e124429b48ae07a3c9806ff804033ecb20447a3f68a9918ade524209fc1ba0', 'crates/spore-core/src/cell_feature_projection_v1.rs': '06e7b586fe4a0abb71ca6d9f4ec672d485e2b69ef1d3a4c653bfe94163ff11ec', 'crates/spore-core/src/cell_feature_first_candidate_v1.rs': 'd9c7deeba7ce6d9db6bd83b194201d192274b03be39a6446303f4e67e1e566c3', 'crates/spore-core/examples/cell_feature_first_candidate_oracle_v1.rs': '58ddde46d3d1365b6b13a689c2148877869b6b7e5740b3e7a4ec83725682924f', 'target/debug/examples/cell_feature_first_candidate_oracle_v1.exe': '2ae8606d5f13cc28543d613f18d9a64840dcae2a247d74a79fc40fdb740ce1c7'}
assert not O.exists(); O.mkdir(parents=True)
receipt={'status':'failed','checks':[],'post_checks':[],'scope':'Rust first-candidate composition compared to 5 retained native rows; E6A450 trapped and small locals opaque; no native process run'}
for name in ('model.stdout.bin','model.stderr.bin'):(O/name).write_bytes(b'')
try:
 for p,expected in pins.items():
  actual=h((R/p).read_bytes());receipt['checks'].append({'path':p,'sha256':actual,'expected':expected})
  if actual!=expected:raise ValueError('pin mismatch '+p)
 old=json.loads((R/'target/cell-feature-first-candidate-result-v1/receipt.json').read_text())
 if old.get('status')!='matched':raise ValueError('old receipt not matched')
 try:
  proc=subprocess.run([str(R/'target/debug/examples/cell_feature_first_candidate_oracle_v1.exe')],input=(R/'target/cell-feature-first-candidate-prep-v3/cases.bin').read_bytes(),capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW)
  out,err,code=proc.stdout,proc.stderr,proc.returncode
 except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
 (O/'model.stdout.bin').write_bytes(out);(O/'model.stderr.bin').write_bytes(err);receipt['run']={'exit':code,'stdout_sha256':h(out),'bytes':len(out)}
 if code!=0 or len(out)!=440:raise ValueError('model failed or size mismatch')
 native=(R/'target/cell-feature-first-candidate-result-v1/native.stdout.bin').read_bytes()
 if len(native)!=480:raise ValueError('native size mismatch')
 receipt['rows']=[]
 for i in range(5):
  match=out[i*88:(i+1)*88]==native[i*96:i*96+88];cw,sw,mx=struct.unpack_from('<HHI',native,i*96+88)
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

"""Compare reusable projection to retained source-fragment output; no native run."""
from pathlib import Path
import hashlib,json,subprocess,sys
R=Path(__file__).resolve().parents[1];O=R/'target/cell-feature-projection-core-result-v1';h=lambda b:hashlib.sha256(b).hexdigest()
pins={'target/cell-feature-fragment-prep-v2/cases.bin': 'a1bfafaae621cd9f168973bb719704d594cd27887f033d350fbafb9728bc67da', 'target/cell-feature-pair-prep-v2/cases.bin': '7022c9c7ca560ac61277259285f0b92cbc28327d4a2fe12c286c988b53e37821', 'target/cell-feature-fragment-run-v1/receipt.json': '50f91155eca2a1bb9634b36b8563ad0059d098bebd7747f3628014713807b664', 'target/cell-feature-pair-run-v1/receipt.json': '34bcbcd68eee414a9cbb14a8012cedc2dea7feea7e82cad6ad551689d7162953', 'target/cell-feature-fragment-run-v1/native.stdout.bin': '9fd8b182dbd46ea71dc948db7a3c86763b9a08d32e483bc29c1912ac8e2c8ba9', 'target/cell-feature-pair-run-v1/native.stdout.bin': 'd8db598cfc6250e8f978abd534202f4f482c0bd6abf175afe1907b85b8eb55ef', 'crates/spore-core/src/native_sse.rs': 'eedac9f108e411f55abe01b1b05d5d690f950a954c16409b2222f15f00298129', 'crates/spore-core/src/cell_part_geometry.rs': 'eceacabb53be8878f083027864ef8ae4a8f5aca91529a96597cd0d11236bcc45', 'crates/spore-core/src/cell_feature_projection_v1.rs': '06e7b586fe4a0abb71ca6d9f4ec672d485e2b69ef1d3a4c653bfe94163ff11ec', 'crates/spore-core/examples/cell_feature_projection_oracle_v1.rs': '6bdf25a5036f3cc33d99e504bbc4e365e7bfc3664701096d46c79415c462e9a1', 'target/debug/examples/cell_feature_projection_oracle_v1.exe': '0e07e66fa490b30deb75a3f37f1a2dba9f9506c03c2b6cc8b57ce7d6c23f4c95'}
assert not O.exists();O.mkdir(parents=True)
rec={'status':'failed','checks':[],'post_checks':[],'scope':'projection words only; first and second retained native outputs from separately pinned original revisions; no native rerun'}
for name in ['first.stdout.bin','first.stderr.bin','second.stdout.bin','second.stderr.bin']:(O/name).write_bytes(b'')
try:
 for p,pin in pins.items():
  v=h((R/p).read_bytes());rec['checks'].append({'path':p,'sha256':v,'expected':pin})
  if v!=pin:raise ValueError('pin mismatch '+p)
 if any(json.loads((R/p).read_text()).get('status')!='passed' for p in ['target/cell-feature-fragment-run-v1/receipt.json','target/cell-feature-pair-run-v1/receipt.json']):raise ValueError('native receipt not passed')
 rec['runs']={};rec['rows']={}
 for name,corpus,native,stride in [('first','target/cell-feature-fragment-prep-v2/cases.bin','target/cell-feature-fragment-run-v1/native.stdout.bin',20),('second','target/cell-feature-pair-prep-v2/cases.bin','target/cell-feature-pair-run-v1/native.stdout.bin',44)]:
  try:
   q=subprocess.run([str(R/'target/debug/examples/cell_feature_projection_oracle_v1.exe'),name],input=(R/corpus).read_bytes(),capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW);out,err,code=q.stdout,q.stderr,q.returncode
  except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
  (O/(name+'.stdout.bin')).write_bytes(out);(O/(name+'.stderr.bin')).write_bytes(err);rec['runs'][name]={'exit':code,'bytes':len(out),'sha256':h(out),'stderr_bytes':len(err)}
  if code!=0 or err or len(out)!=96:raise ValueError(name+' model failed')
  old=(R/native).read_bytes()
  if len(old)!=6*stride:raise ValueError(name+' native size')
  matches=[out[i*16:(i+1)*16]==old[i*stride:i*stride+16] for i in range(6)];rec['rows'][name]=matches
  if not all(matches):raise ValueError(name+' mismatch')
 rec['status']='matched'
except Exception as e:rec['error']=repr(e)
finally:
 for p,pin in pins.items():
  try:
   v=h((R/p).read_bytes());rec['post_checks'].append({'path':p,'sha256':v,'expected':pin})
   if v!=pin:rec['status']='failed';rec['post_error']='changed pin '+p
  except Exception as e:rec['status']='failed';rec['post_error']=repr(e)
 (O/'receipt.json').write_text(json.dumps(rec,indent=2)+'\n')
print(json.dumps(rec,indent=2));sys.exit(rec['status']!='matched')

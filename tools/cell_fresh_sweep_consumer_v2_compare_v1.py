from pathlib import Path
import hashlib, json, subprocess, struct, sys
R=Path(__file__).resolve().parents[1]; O=R/'target/cell-fresh-consumer-v2-result-v1'; h=lambda b:hashlib.sha256(b).hexdigest()
pins={
 'target/cell-fresh-consumer-result-v1/receipt.json':'371a2f0bfc665f7a56e0be7d58af824aa7950015f166dbad32cacb36af6a3912',
 'target/cell-fresh-consumer-result-v1/native.stdout.bin':'d09e41b2a94b63bb0094b3c9a8092e5f7e4db7550f336a8a8ae2e5b0caa3400f',
 'target/cell-fresh-consumer-prep-v1/cases.bin':'eefb3b05fa514bb9738715e6cffeee0e98db08761439850921d7d2f60c835fb1',
 'crates/spore-core/src/cell_fresh_sweep_registration_v1.rs':'0497f901974eb1ec82ed8a05519c943455180458971e77a969a295f43fb70d88',
 'crates/spore-core/src/cell_source_sweep_pairs_v1.rs':'2d51cb48f97dbba4ed0283f12c1dd81e159761e538a6f60baa5a3c2280fe5279',
 'crates/spore-core/examples/cell_fresh_sweep_consumer_oracle_v2.rs':'1af759f86c2664484be00f72e92a5a6595f93597c707ff3d7453462964d08a7e',
 'target/debug/examples/cell_fresh_sweep_consumer_oracle_v2.exe':'0cf36894d3bcc0644ec06dc8b70e3940b0bbb2dbfaf990e1cb60d7ffdec06628'}
assert not O.exists(); O.mkdir(parents=True)
receipt={'status':'failed','checks':[],'post_checks':[],'scope':'Rust v2 API compared to preserved four native rows; no native process run; sorting is repeated within rebuild'}
for name in ('model.stdout.bin','model.stderr.bin'):(O/name).write_bytes(b'')
try:
 for p,expected in pins.items():
  actual=h((R/p).read_bytes());receipt['checks'].append({'path':p,'sha256':actual,'expected':expected})
  if actual!=expected:raise ValueError('pin mismatch '+p)
 old=json.loads((R/'target/cell-fresh-consumer-result-v1/receipt.json').read_text())
 if old.get('status')!='matched':raise ValueError('old receipt not matched')
 try:
  proc=subprocess.run([str(R/'target/debug/examples/cell_fresh_sweep_consumer_oracle_v2.exe')],input=(R/'target/cell-fresh-consumer-prep-v1/cases.bin').read_bytes(),capture_output=True,timeout=10,creationflags=subprocess.CREATE_NO_WINDOW)
  out,err,code=proc.stdout,proc.stderr,proc.returncode
 except subprocess.TimeoutExpired as e:out,err,code=e.stdout or b'',e.stderr or b'','timeout'
 (O/'model.stdout.bin').write_bytes(out);(O/'model.stderr.bin').write_bytes(err);receipt['run']={'exit':code,'stdout_sha256':h(out),'bytes':len(out)}
 if code!=0 or len(out)!=1008:raise ValueError('model failed or size mismatch')
 native=(R/'target/cell-fresh-consumer-result-v1/native.stdout.bin').read_bytes()
 if len(native)!=1040:raise ValueError('native size mismatch')
 receipt['rows']=[]
 for i in range(4):
  match=out[i*252:(i+1)*252]==native[i*260:i*260+252];cw,sw,mx=struct.unpack_from('<HHI',native,i*260+252)
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

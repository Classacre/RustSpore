"""One-shot pinned locale raw source host; no retry and no automatic comparison."""
import hashlib, json, subprocess, sys
from pathlib import Path
root=Path(__file__).resolve().parent.parent
host=root/'target/source-cell/locale-raw-host-v3/host.exe'
expected='8812847375a3a361b0f605f7cca67aec5365afe38cde5fa11b03a7f29cc5b11a'
assert hashlib.sha256(host.read_bytes()).hexdigest()==expected
out=root/'target/source-cell/locale-raw-native-run-v1'
out.mkdir(parents=True,exist_ok=False)
record={'host_sha256':expected,'input':'six fixed admitted cases embedded in host; no stdin','attempts':1,'source_executed':'attempted'}
try:
 p=subprocess.run([str(host)],input=b'',stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=10,check=False)
 (out/'output.bin').write_bytes(p.stdout);(out/'stderr.bin').write_bytes(p.stderr)
 record.update(exit_code=p.returncode,output_sha256=hashlib.sha256(p.stdout).hexdigest(),output_bytes=len(p.stdout),stderr_sha256=hashlib.sha256(p.stderr).hexdigest())
except subprocess.TimeoutExpired as e:
 (out/'output.bin').write_bytes(e.stdout or b'');(out/'stderr.bin').write_bytes(e.stderr or b'')
 record.update(timeout=True,output_bytes=len(e.stdout or b''))
finally:
 (out/'run.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf8')
print(json.dumps(record));sys.exit(0 if record.get('exit_code')==0 else 1)

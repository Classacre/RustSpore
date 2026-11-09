"""Create self-debugger-checking relation-5 host; no build or execution."""
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent.parent
src=root/'target/cell-tail-relation5-review-v2'; out=root/'target/cell-tail-relation5-review-v3'
def sha(b): return hashlib.sha256(b).hexdigest()
host=(src/'host.c').read_bytes(); code=(src/'relocated.bin').read_bytes()
if sha(host)!='184b8447692afc83fa15f06fdd54ad6f3e45d0dfe96fe332959ad2bf3b222859' or sha(code)!='c84b8bef852b87e4d7953c632dc60d2ce5e91498d347f01b468a49f5d6530513': raise ValueError('v2 source mismatch')
needle=b' void *code=VirtualAlloc(NULL,0x1000,MEM_COMMIT|MEM_RESERVE,PAGE_READWRITE); if(!code) return 12;'
replacement=(b' BOOL debugged=FALSE;\r\n'
 b' if(IsDebuggerPresent() || !CheckRemoteDebuggerPresent(GetCurrentProcess(),&debugged) || debugged) return 17;\r\n'+needle)
if host.count(needle)!=1: raise ValueError('host insertion point mismatch')
host=host.replace(needle,replacement)
out.mkdir(parents=True,exist_ok=False)
(out/'host.c').write_bytes(host); (out/'relocated.bin').write_bytes(code)
r={'schema':'cell-tail-relation5-stub-preparation-v3','source_v2_host_sha256':sha((src/'host.c').read_bytes()),
 'host_sha256':sha(host),'relocated_sha256':sha(code),'generator_sha256':sha(Path(__file__).read_bytes()),
 'relation':5,'debugger_check':'IsDebuggerPresent and CheckRemoteDebuggerPresent before allocation/source execution; fail closed',
 'built':False,'executed':False,'limitations':'debugger can attach after check; host canaries are not guard pages; INT3/UD2 requires unhandled exceptions'}
(out/'manifest.json').write_text(json.dumps(r,indent=2)+'\n',encoding='ascii'); print(json.dumps(r,indent=2))

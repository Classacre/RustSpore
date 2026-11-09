"""Record relation-5 host file observations after build, without executing it."""
import hashlib,json,sys
from pathlib import Path
import capstone,pefile
root=Path(__file__).resolve().parent.parent; prep=root/'target/cell-tail-relation5-review-v3'
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
exe=prep/'host.exe'; manifest=prep/'manifest.json'
if sha(exe)!='dec665dd2711a1cdaf04cab0faf295c7bf426418627bc9384e9ab5ef893ed90f': raise ValueError('unexpected host')
msvc=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86')
files=[prep/'host.c',prep/'host.obj',prep/'relocated.bin',prep/'linked-call-listing.txt',
 root/'target/cell-tail-direct-review-v3/source.bin',root/'tools/cell_tail_relation5_prepare_v3.py',
 msvc/'cl.exe',msvc/'link.exe',Path(sys.executable),Path(capstone.__file__),Path(pefile.__file__),Path(__file__).resolve()]
r={'schema':'cell-tail-relation5-postbuild-observation-v3','host_sha256':sha(exe),'prep_manifest_sha256':sha(manifest),
 'pinned_files':[{'path':str(p.resolve()),'sha256':sha(p)} for p in files],
 'command':'cl /nologo /W4 /O2 /arch:SSE2 /Fohost.obj /Fehost.exe host.c /link /DYNAMICBASE:NO',
 'runtime_prerequisites':'Windows x86, SSE2, Windows VM APIs, x86 CRT; INT3/UD2 fail-closed only if exceptions remain unhandled; no attached debugger swallowing or modifying exceptions',
 'limitations':'Post-build hashes do not prove compiler invocation; compiler helpers, SDK, CRT and other runtime dependencies are unpinned; no sandbox or Job Object; before/after hashing cannot rule out transient replacement.'}
out=prep/'build-receipt.json'
with out.open('x',encoding='utf-8') as f: json.dump(r,f,indent=2); f.write('\n')
print(sha(out))

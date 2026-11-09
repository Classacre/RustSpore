"""Record observed post-build file hashes; does not prove build process provenance."""
import hashlib,json,sys
from pathlib import Path
import capstone,pefile
root=Path(__file__).resolve().parent.parent
prep=root/'target/cell-tail-direct-review-v3'
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
exe=prep/'host.exe'; manifest=prep/'manifest.json'
if sha(exe)!='45c16c338b33115c767fd400f325498430e1b9678d8af3e189c42c719f7bd504': raise ValueError('unexpected host')
if sha(prep/'linked-call-listing.txt')!='b0b2b373695925566fff66d86acfa8ff0661f7d25de481f52d64094b0b5fc15f': raise ValueError('unexpected listing')
msvc=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86')
paths=[prep/'host.c',prep/'host.obj',prep/'source.bin',prep/'reachable-listing.txt',prep/'linked-call-listing.txt',
       msvc/'cl.exe',msvc/'link.exe',Path(sys.executable),Path(capstone.__file__),Path(pefile.__file__),
       root/'tools/cell_tail_direct_native_prepare_v1.py',Path(__file__).resolve()]
files=[{'path':str(p.resolve()),'sha256':sha(p)} for p in paths]
r={'schema':'cell-tail-direct-postbuild-observation-v1','host_sha256':sha(exe),'prep_manifest_sha256':sha(manifest),
   'pinned_files':files,'command':'cl /nologo /W4 /O2 /arch:SSE2 /Fohost.obj /Fehost.exe host.c /link /DYNAMICBASE:NO',
   'runtime_prerequisites':'Windows x86 process, SSE2, VirtualAlloc/VirtualProtect/FlushInstructionCache, x86 CRT; runtime DLLs, SDK, compiler helpers and Python module dependencies are not pinned',
   'limitations':'Hashes observed after the build; not a cryptographic binding to compiler invocation. No process sandbox or Job Object; before/after hashes cannot detect transient replacement.'}
with (prep/'build-receipt.json').open('x',encoding='utf-8') as f: json.dump(r,f,indent=2); f.write('\n')
print(sha(prep/'build-receipt.json'))

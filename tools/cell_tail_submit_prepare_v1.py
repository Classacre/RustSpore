"""Prepare pinned source-byte include and reviewed source copies only."""
import hashlib,json
from pathlib import Path
import pefile
root=Path(__file__).resolve().parent.parent
exe=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
out=root/'target/cell-tail-submit-review-v1'
def sha(b): return hashlib.sha256(b).hexdigest()
data=exe.read_bytes()
if sha(data)!='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a': raise ValueError('unexpected PE')
pe=pefile.PE(data=data,fast_load=True)
if pe.FILE_HEADER.Machine!=0x14c or pe.OPTIONAL_HEADER.ImageBase!=0x400000: raise ValueError('unexpected PE layout')
code=pe.get_data(0xe6a2b0-pe.OPTIONAL_HEADER.ImageBase,0x192)
if sha(code)!='7629f782f801bae85724e65d12835160badf5648eb96c6109dacbef0e01b664e': raise ValueError('unexpected source range')
offsets=[0x12,0x32,0x43,0x7a,0xa4]
if any(code[i]!=0xe8 for i in offsets): raise ValueError('unexpected call opcodes')
host=(root/'re/cell-tail-submit-host-review-v1.c').read_bytes()
stubs=(root/'re/cell-tail-submit-stubs-review-v1.c').read_bytes()
inc=(','.join('0x%02x'%b for b in code)+'\n').encode('ascii')
out.mkdir(parents=True,exist_ok=False)
for name,content in [('host.c',host),('cell-tail-submit-stubs-review-v1.c',stubs),('tail_source_bytes.inc',inc),('source.bin',code)]: (out/name).write_bytes(content)
r={'schema':'cell-tail-submit-preparation-v1','source_path':str(exe),'source_sha256':sha(data),'source_span_sha256':sha(code),
 'files':{n:sha((out/n).read_bytes()) for n in ('host.c','cell-tail-submit-stubs-review-v1.c','tail_source_bytes.inc','source.bin')},
 'generator_sha256':sha(Path(__file__).read_bytes()),'call_offsets':[hex(i) for i in offsets],
 'scope':'preparation only; independent source/linked/runner audits required','built':False,'executed':False}
(out/'manifest.json').write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8'); print(json.dumps(r,indent=2))

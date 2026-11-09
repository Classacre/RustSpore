"""Prepare non-executable E6A450 source slices and linear call census only."""
import hashlib,json
from pathlib import Path
import capstone,pefile
ROOT=Path(__file__).resolve().parent.parent
EXE=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
OUT=ROOT/'target/cell-feeding-prefix-source-review-v1'
def sha(b): return hashlib.sha256(b).hexdigest()
data=EXE.read_bytes()
if sha(data)!='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a': raise ValueError('source digest mismatch')
p=pefile.PE(data=data,fast_load=True)
if p.FILE_HEADER.Machine!=0x14c or p.OPTIONAL_HEADER.ImageBase!=0x400000: raise ValueError('unexpected PE')
start,end=0xe6a450,0xe6add3
code=p.get_data(start-p.OPTIONAL_HEADER.ImageBase,end-start)
if len(code)!=end-start or code[-2:]!=bytes.fromhex('59c3'): raise ValueError('missing expected final pop/ret')
offset=0xe6a548-start
if code[offset:offset+6]!=bytes.fromhex('d98790000000'): raise ValueError('unexpected continuation bytes')
patched=bytearray(code); patched[offset:offset+6]=bytes.fromhex('e98008000090')
m=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
ins=list(m.disasm(code,start)); covered=sum(i.size for i in ins)
if covered!=len(code): raise ValueError('linear disassembly did not cover entire slice')
calls=[{'address':hex(i.address),'bytes':i.bytes.hex(),'operand':i.op_str} for i in ins if i.mnemonic=='call']
OUT.mkdir(parents=True,exist_ok=False)
(OUT/'source.bin').write_bytes(code); (OUT/'b6-stop-patch.bin').write_bytes(patched)
(OUT/'linear-calls.json').write_text(json.dumps(calls,indent=2)+'\n')
r={'schema':'cell-feeding-prefix-source-preparation-v1','source_sha256':sha(data),'start':hex(start),'end_exclusive':hex(end),
 'original_sha256':sha(code),'b6_stop_sha256':sha(patched),'b6_patch_address':hex(start+offset),
 'old_bytes':code[offset:offset+6].hex(),'new_bytes':patched[offset:offset+6].hex(),
 'linear_calls_sha256':sha((OUT/'linear-calls.json').read_bytes()),'linear_call_count':len(calls),
 'generator_sha256':sha(Path(__file__).read_bytes()),'built':False,'executed':False,
 'limitations':'linear call census is not CFG; no host, no patched call targets, no original execution'}
(OUT/'manifest.json').write_text(json.dumps(r,indent=2)+'\n'); print(json.dumps(r,indent=2))

"""Review-only relation-5 E6A250-nonzero controlled-stub host; no build or execution."""
import hashlib,json,struct
from pathlib import Path

ROOT=Path(__file__).resolve().parent.parent
BASE=ROOT/'target/cell-tail-direct-review-v3'
OUT=ROOT/'target/cell-tail-relation5-review-v1'
def sha(b): return hashlib.sha256(b).hexdigest()
code=(BASE/'source.bin').read_bytes()
if sha(code)!='7629f782f801bae85724e65d12835160badf5648eb96c6109dacbef0e01b664e': raise ValueError('source mismatch')
host=(BASE/'host.c').read_bytes().decode('ascii')
if sha(host.encode('ascii'))!='d35eea090c61e414a198ab065a804e3ba781a4d629bd16dc86dad2ef885880d4': raise ValueError('host mismatch')
if len(code)!=0x192 or code[0x12]!=0xe8: raise ValueError('unexpected call location')
# Only change the rel32 operand at E6A2C2. A six-byte in-buffer stub always
# returns 1 in EAX, so source JNE enters the direct-write block. Other external
# call sites remain invalid after relocation and MUST remain unreachable.
patched=bytearray(code); stub_offset=len(patched); displacement=stub_offset-(0x12+5)
struct.pack_into('<i',patched,0x13,displacement); patched.extend(b'\xb8\x01\x00\x00\x00\xc3')
old='static const unsigned char source[] = {'+','.join('0x%02x'%x for x in code)+'};'
new='static const unsigned char source[] = {'+','.join('0x%02x'%x for x in patched)+'};'
if host.count(old)!=1: raise ValueError('source array missing')
host=host.replace(old,new).replace('Fixed relation=0: no execution can enter external-call branch.',
    'Fixed relation=5: stub returns 1 and only the direct priority-22 path is admitted.')
old_call='((tail)code)(out+1,point,normal,a[20],actor_a+1,actor_b+1,0,0);'
new_call='((tail)code)(out+1,point,normal,a[20],actor_a+1,actor_b+1,5,0);'
if host.count(old_call)!=1: raise ValueError('host call mismatch')
host=host.replace(old_call,new_call)
OUT.mkdir(parents=True,exist_ok=False)
(OUT/'relocated.bin').write_bytes(patched)
(OUT/'host.c').write_bytes(host.encode('ascii'))
r={'schema':'cell-tail-relation5-stub-preparation-v1','source_sha256':sha(code),'relocated_sha256':sha(patched),
   'host_sha256':sha(host.encode('ascii')),'generator_sha256':sha(Path(__file__).read_bytes()),
   'stub_offset':hex(stub_offset),'stub_hex':'b801000000c3','modified_source_offsets':['0x13..0x16'],
   'patched_call_offset':'0x12','rel32':displacement,'relation':5,'stub_return':1,
   'built':False,'executed':False,'scope':'explicit deterministic E6A250 substitute; other calls unsafe; independent CFG and linked-host review mandatory'}
(OUT/'manifest.json').write_text(json.dumps(r,indent=2)+'\n',encoding='ascii'); print(json.dumps(r,indent=2))

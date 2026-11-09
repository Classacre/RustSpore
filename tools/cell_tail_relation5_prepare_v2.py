"""Prepare guarded relation-5 code and host only; never build or execute it."""
import hashlib,json,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
BASE=ROOT/'target/cell-tail-direct-review-v3'
OUT=ROOT/'target/cell-tail-relation5-review-v2'
def sha(b): return hashlib.sha256(b).hexdigest()
code=(BASE/'source.bin').read_bytes(); host=(BASE/'host.c').read_bytes().decode('ascii')
if sha(code)!='7629f782f801bae85724e65d12835160badf5648eb96c6109dacbef0e01b664e': raise ValueError('source mismatch')
if sha(host.encode('ascii'))!='d35eea090c61e414a198ab065a804e3ba781a4d629bd16dc86dad2ef885880d4': raise ValueError('host mismatch')
buf=bytearray(code); trace_offset=len(buf)
# mov eax,[esp+4]; mov [absolute argument slot],eax; inc [absolute count];
# mov eax,1; ret. Host patches the two absolute addresses before RX protection.
trace=bytes.fromhex('8b442404a300000000ff0500000000b801000000c3'); buf.extend(trace)
fail_offset=len(buf); buf.extend(bytes.fromhex('cc0f0b')) # INT3; UD2
calls={'e6a250':0x12,'e4cb40':0x32,'e571c0':0x43,'e59b40_a':0x7a,'e59b40_b':0xa4}
for name,pos in calls.items():
    if buf[pos]!=0xe8: raise ValueError('unexpected call '+name)
    target=trace_offset if name=='e6a250' else fail_offset
    struct.pack_into('<i',buf,pos+1,target-(pos+5))
original='static const unsigned char source[] = {'+','.join('0x%02x'%x for x in code)+'};'
replacement='static const unsigned char source[] = {'+','.join('0x%02x'%x for x in buf)+'};'
if host.count(original)!=1: raise ValueError('source array missing')
host=host.replace(original,replacement)
old='uint32_t actor_a[3], actor_b[3], aa[3], bb[3]; DWORD old; unsigned int mx;'
new='uint32_t actor_a[3], actor_b[3], aa[3], bb[3]; uint32_t log[4]={0x9c9c9c9c,0,0,0x6d6d6d6d}; DWORD old; unsigned int mx;'
if host.count(old)!=1: raise ValueError('declaration mismatch')
host=host.replace(old,new)
old='memcpy(code,source,sizeof source); if(!VirtualProtect(code,0x1000,PAGE_EXECUTE_READ,&old) ||'
new=('memcpy(code,source,sizeof source); { uint32_t arg=(uint32_t)(uintptr_t)&log[2], count=(uint32_t)(uintptr_t)&log[1]; '
     f'memcpy((unsigned char*)code+{trace_offset+5},&arg,4); memcpy((unsigned char*)code+{trace_offset+11},&count,4); '
     '} if(!VirtualProtect(code,0x1000,PAGE_EXECUTE_READ,&old) ||')
if host.count(old)!=1: raise ValueError('patch boundary mismatch')
host=host.replace(old,new)
host=host.replace('Fixed relation=0: no execution can enter external-call branch.',
  'Fixed relation=5: one instrumented E6A250 stub; all other calls trap.')
old='((tail)code)(out+1,point,normal,a[20],actor_a+1,actor_b+1,0,0);'
if host.count(old)!=1: raise ValueError('call mismatch')
host=host.replace(old,'((tail)code)(out+1,point,normal,a[20],actor_a+1,actor_b+1,5,0);')
old='if(out[0]!=before[0] || out[13]!=before[13] || memcmp(point,p0,12) || memcmp(normal,n0,12) ||'
new=('if(log[0]!=0x9c9c9c9c || log[3]!=0x6d6d6d6d || log[1]!=1 || log[2]!=(uint32_t)(uintptr_t)(actor_a+1)) return 16;\r\n '
     'if(out[0]!=before[0] || out[13]!=before[13] || memcmp(point,p0,12) || memcmp(normal,n0,12) ||')
if host.count(old)!=1: raise ValueError('guard mismatch')
host=host.replace(old,new)
OUT.mkdir(parents=True,exist_ok=False)
(OUT/'relocated.bin').write_bytes(buf); (OUT/'host.c').write_bytes(host.encode('ascii'))
metadata={'schema':'cell-tail-relation5-stub-preparation-v2','source_sha256':sha(code),'relocated_sha256':sha(buf),
 'host_sha256':sha(host.encode('ascii')),'generator_sha256':sha(Path(__file__).read_bytes()),
 'source_call_offsets':{k:hex(v) for k,v in calls.items()},'trace_offset':hex(trace_offset),'trace_hex':trace.hex(),
 'runtime_absolute_pointer_offsets':[hex(trace_offset+5),hex(trace_offset+11)],'fail_offset':hex(fail_offset),'fail_hex':'cc0f0b',
 'relation':5,'built':False,'executed':False,'scope':'requires independent CFG, argument/stack, runtime patch, trap and linked-binary review'}
(OUT/'manifest.json').write_text(json.dumps(metadata,indent=2)+'\n',encoding='ascii'); print(json.dumps(metadata,indent=2))

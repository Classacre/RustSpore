"""Prepare a bounded 76F930 x86 host from the pinned executable; does not build or run it."""
import argparse, hashlib, json, struct
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
PIN='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
FUNCTION_PIN='81fc0c1de942b4c378cf5b24662f2b5b7104e577ca6c98108e8ba46a403bc0f6'
START=0x76f930
SIZE=0x6c
C=r'''#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
static const unsigned char code[]={SOURCE_BYTES};
typedef unsigned char (__cdecl *fn)(void*,void*,void*);
typedef struct { uint32_t reserved; int32_t *parents; uint32_t ids; int32_t count; } object;
typedef struct { int32_t *map; } mapping;
typedef struct { int n, bn; int32_t a[3], b[3], m[3]; } testcase;
static const testcase cases[]={
 {0,0,{0},{0},{0}}, {1,1,{-1},{-1},{-1}}, {1,1,{-1},{-1},{0}},
 {3,2,{-1,0,1},{-1,0},{0,-1,1}}, {3,2,{-1,0,1},{-1,-1},{0,-1,1}},
 {2,2,{-1,0},{-1,0},{-1,1}}, {2,2,{-1,0},{-1,0},{1,0}},
 {3,3,{-1,0,1},{-1,0,1},{0,1,2}} };
int main(void) {
 if (sizeof(void*)!=4) return 10;
 void *p=VirtualAlloc(NULL,4096,MEM_COMMIT|MEM_RESERVE,PAGE_READWRITE); DWORD old;
 if(!p) return 11; memcpy(p,code,sizeof(code));
 if(!VirtualProtect(p,4096,PAGE_EXECUTE_READ,&old)||!FlushInstructionCache(GetCurrentProcess(),p,sizeof(code))) return 12;
 for(unsigned i=0;i<sizeof(cases)/sizeof(cases[0]);i++) {
  testcase t=cases[i],before=t; object a={0,t.a,0,t.n},b={0,t.b,0,t.bn},aa=a,bb=b; mapping m={t.m},mm=m;
  unsigned char result=((fn)p)(&a,&b,&m);
  if(memcmp(&t,&before,sizeof(t))||memcmp(&a,&aa,sizeof(a))||memcmp(&b,&bb,sizeof(b))||memcmp(&m,&mm,sizeof(m))||result>1) return 13;
  printf("%u\n",(unsigned)result);
 }
 return 0;
}'''
def sha(b): return hashlib.sha256(b).hexdigest()
def main():
 p=argparse.ArgumentParser(); p.add_argument('exe',type=Path); p.add_argument('--out',type=Path,required=True); a=p.parse_args()
 out=a.out.resolve(); out.relative_to((ROOT/'target').resolve()); data=a.exe.read_bytes()
 if sha(data)!=PIN: raise ValueError('source executable hash mismatch')
 pe=struct.unpack_from('<I',data,0x3c)[0]
 if data[:2]!=b'MZ' or data[pe:pe+4]!=b'PE\0\0': raise ValueError('invalid PE')
 machine,nsects=struct.unpack_from('<HH',data,pe+4); opts=struct.unpack_from('<H',data,pe+20)[0]; opt=pe+24
 if machine!=0x14c or struct.unpack_from('<H',data,opt)[0]!=0x10b: raise ValueError('expected x86 PE32')
 rva=START-struct.unpack_from('<I',data,opt+28)[0]; sections=opt+opts; offsets=[]
 for i in range(nsects):
  vs,va,rs,raw=struct.unpack_from('<IIII',data,sections+40*i+8)
  if va<=rva and rva+SIZE<=va+rs and raw+rs<=len(data): offsets.append(raw+rva-va)
 if len(offsets)!=1: raise ValueError('source function not uniquely file-backed')
 code=data[offsets[0]:offsets[0]+SIZE]
 if sha(code)!=FUNCTION_PIN: raise ValueError('function hash mismatch')
 out.mkdir(parents=True,exist_ok=False)
 host=C.replace('SOURCE_BYTES',','.join('0x%02x'%x for x in code)).encode('utf-8'); (out/'host.c').write_bytes(host)
 manifest=dict(source_sha256=PIN,function_sha256=sha(code),function_address=hex(START),function_size=SIZE,file_offset=offsets[0],host_sha256=sha((out/'host.c').read_bytes()),prepared=True,built=False,executed=False,scope='eight bounded controlled fixtures; stack allocations lack guard pages or read guards; no live association or pose claim; independent linked-code audit required')
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8'); print(json.dumps(manifest))
if __name__=='__main__': main()

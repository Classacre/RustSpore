"""Prepare a guarded x86 76F930 host under target; never builds or executes it."""
import argparse, hashlib, json, struct
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
PIN='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
FUNCTION_PIN='81fc0c1de942b4c378cf5b24662f2b5b7104e577ca6c98108e8ba46a403bc0f6'
START,SIZE=0x76f930,0x6c
C=r'''#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <fcntl.h>
#include <io.h>
static const unsigned char code[]={SOURCE_BYTES};
typedef unsigned char (__cdecl *fn)(void*,void*,void*);
typedef struct { uint32_t reserved; const int32_t *parents; uint32_t ids; int32_t count; } object;
typedef struct { const int32_t *map; } mapping;
/* Every object resides in its own readable page between inaccessible pages.
   Placement selects which end of the readable page touches the object. */
static void *guarded(const void *src, size_t n, int placement, size_t page) {
 unsigned char *base=(unsigned char*)VirtualAlloc(NULL,page*3,MEM_RESERVE,PAGE_NOACCESS);
 unsigned char *mid, *at; DWORD old;
 if(!base || n>page) return NULL;
 mid=(unsigned char*)VirtualAlloc(base+page,page,MEM_COMMIT,PAGE_READWRITE);
 if(mid!=base+page) return NULL;
 at=placement ? mid+page-n : mid;
 if(n) memcpy(at,src,n);
 if(!VirtualProtect(mid,page,PAGE_READONLY,&old)) return NULL;
 return at;
}
static uint32_t le32(const unsigned char *b) {
 return (uint32_t)b[0] | (uint32_t)b[1]<<8 | (uint32_t)b[2]<<16 | (uint32_t)b[3]<<24;
}
int main(void) {
 unsigned char head[16], raw[256*12]; int32_t a[256],b[256],m[256];
 uint32_t an,bn,side; SYSTEM_INFO sys; DWORD old; size_t page,bytes; void *f;
 if(sizeof(void*)!=4 || _setmode(_fileno(stdin),_O_BINARY)==-1 || _setmode(_fileno(stdout),_O_BINARY)==-1) return 10;
 if(fread(head,1,sizeof(head),stdin)!=sizeof(head) || memcmp(head,"H930",4)) return 11;
 an=le32(head+4); bn=le32(head+8); side=le32(head+12);
 if(an>256 || bn>256 || side>1) return 12;
 bytes=((size_t)an*2+bn)*4;
 if(fread(raw,1,bytes,stdin)!=bytes || getchar()!=EOF || ferror(stdin)) return 13;
 for(uint32_t i=0;i<an;i++) a[i]=(int32_t)le32(raw+4*i);
 for(uint32_t i=0;i<bn;i++) b[i]=(int32_t)le32(raw+4*(an+i));
 for(uint32_t i=0;i<an;i++) m[i]=(int32_t)le32(raw+4*(an+bn+i));
 for(uint32_t i=0;i<an;i++) {
  if(a[i]<-1 || (a[i]>=0 && (uint32_t)a[i]>=i)) return 14;
  if(m[i]<-1 || (m[i]>=0 && (uint32_t)m[i]>=bn)) return 15;
 }
 for(uint32_t i=0;i<bn;i++) if(b[i]<-1 || (b[i]>=0 && (uint32_t)b[i]>=bn)) return 16;
 for(uint32_t i=0;i<bn;i++) {
  int32_t p=(int32_t)i; uint32_t steps=0;
  while(p!=-1) { if(++steps>bn) return 17; p=b[p]; }
 }
 GetSystemInfo(&sys); page=sys.dwPageSize; if(page<4096 || page>65536) return 18;
 int32_t *ga=(int32_t*)guarded(a,an*4,side,page), *gb=(int32_t*)guarded(b,bn*4,side,page), *gm=(int32_t*)guarded(m,an*4,side,page);
 if(!ga || !gb || !gm) return 19;
 object aa={0,ga,0,(int32_t)an},bb={0,gb,0,(int32_t)bn}; mapping mm={gm};
 object *oa=(object*)guarded(&aa,sizeof(aa),side,page), *ob=(object*)guarded(&bb,sizeof(bb),side,page);
 mapping *om=(mapping*)guarded(&mm,sizeof(mm),side,page);
 if(!oa||!ob||!om) return 20;
 f=VirtualAlloc(NULL,page,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE); if(!f) return 21;
 memcpy(f,code,sizeof(code));
 if(!VirtualProtect(f,page,PAGE_EXECUTE_READ,&old)||!FlushInstructionCache(GetCurrentProcess(),f,sizeof(code))) return 22;
 unsigned char result=((fn)f)(oa,ob,om);
 if(result>1) return 23;
 if(printf("%u\n",(unsigned)result)<0 || fflush(stdout)) return 24;
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
 manifest=dict(schema='cell-hierarchy-76f930-prep-v2',source_sha256=PIN,function_sha256=sha(code),function_address=hex(START),function_size=SIZE,file_offset=offsets[0],host_sha256=sha((out/'host.c').read_bytes()),prepared=True,built=False,executed=False,scope='explicit validated inputs; separate read-only guard-paged buffers; one case per process; linked audit and execution required; no runtime association')
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8'); print(json.dumps(manifest))
if __name__=='__main__': main()

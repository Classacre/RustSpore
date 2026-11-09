"""Prepare static audit and x86 C hosts; never execute original instructions."""
import argparse, hashlib, json
from pathlib import Path
from cell_schema_probe import SourceImage, SOURCE_SHA256
from native_crt_acos_probe import decoder

SRC=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe')
RANGES=((0x6b4ff0,0x6b501f),(0x6b5031,0x6b5044),(0x6b50ec,0x6b50fc))
PREFLIGHT=r'''
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>
int main(void) {
  void *a=(void*)0x006b0000, *b=(void*)0x01600000; MEMORY_BASIC_INFORMATION x,y;
  if (!VirtualQuery(a,&x,sizeof(x)) || !VirtualQuery(b,&y,sizeof(y))) return 10;
  printf("code state=%08lx base=%p allocation=%p size=%lu; global state=%08lx base=%p allocation=%p size=%lu\n",x.State,x.BaseAddress,x.AllocationBase,(unsigned long)x.RegionSize,y.State,y.BaseAddress,y.AllocationBase,(unsigned long)y.RegionSize);
  if (x.State != MEM_FREE || y.State != MEM_FREE) return 11;
  if (VirtualAlloc(a,65536,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE)!=a) return 12;
  if (VirtualAlloc(b,65536,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE)!=b) return 13;
  if (!VirtualFree(a,0,MEM_RELEASE) || !VirtualFree(b,0,MEM_RELEASE)) return 14;
  puts("fixed allocations and release succeeded; no source instructions present"); return 0;
}
'''
HOST=r'''
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>
#include <string.h>
typedef unsigned int U;
/* This fixed corpus admits only raw-return cases; no request input exists. */
static const unsigned char cases[6][4]={{0,0,0,0},{0,1,0,0},{1,0,0,0},{1,0,0,1},{1,0,1,0},{1,0,1,1}};
static const unsigned short raw[]={77,97,120,105,115,0};
static unsigned char state[0x12];
static U invoke(void *obj) { U result; __asm {
    mov ecx,obj
    mov eax,006B4FF0h
    call eax
    mov result,eax
  } return result; }
int main(void) {
  U i,old; unsigned char *code, *global;
  code=(unsigned char*)VirtualAlloc((void*)0x006b0000,65536,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE);
  if(code!=(unsigned char*)0x006b0000) return 21;
  global=(unsigned char*)VirtualAlloc((void*)0x01600000,65536,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE);
  if(global!=(unsigned char*)0x01600000) return 22;
  memset(code,0xcc,65536);
  memcpy(code+0x4ff0,code_a,sizeof(code_a)); memcpy(code+0x5031,code_b,sizeof(code_b)); memcpy(code+0x50ec,code_c,sizeof(code_c));
  if(!VirtualProtect(code,65536,PAGE_EXECUTE_READ,(DWORD*)&old) || !FlushInstructionCache(GetCurrentProcess(),code,65536)) return 23;
  for(i=0;i<6;i++) {
    unsigned char *c=(unsigned char*)cases[i]; U value;
    /* ordered fields: cached flag, cached present, classifier, global */
    if (!((c[0] && !c[1]) || (!c[0] && !c[2] && !c[3]))) return 24;
    memset(state,0,sizeof(state)); *(U*)(state+8)=(U)raw;
    *(U*)(state+4)=c[1] ? (U)raw : 0; state[0x10]=c[2]; state[0x11]=c[0]; global[0x6354]=c[3];
    value=invoke(state);
    if(value!=(U)raw || state[0x10]!=c[2] || state[0x11]!=c[0] || global[0x6354]!=c[3]) return 25;
    printf("%u %u %u %u raw\n",c[0],c[1],c[2],c[3]);
  }
  return 0;
}
'''
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);a=p.parse_args()
 out=a.out.resolve();out.relative_to((Path(__file__).resolve().parent.parent/'target').resolve());out.mkdir(parents=True,exist_ok=False)
 assert hashlib.sha256(SRC.read_bytes()).hexdigest()==SOURCE_SHA256
 image=SourceImage(SRC); report=[]; chunks=[]
 for start,end in RANGES:
  b=image.read(start,end-start); ins=list(decoder().disasm(b,start));assert sum(i.size for i in ins)==len(b)
  chunks.append(b);report.append({'start':hex(start),'end_exclusive':hex(end),'sha256':hashlib.sha256(b).hexdigest(),'instructions':[f'{i.address:08X} {i.mnemonic} {i.op_str}' for i in ins]})
 def arr(n,b):return 'static const unsigned char '+n+'[]={'+','.join(str(x) for x in b)+'};\n'
 (out/'preflight.c').write_text(PREFLIGHT,encoding='ascii',newline='\n')
 (out/'host.c').write_text(arr('code_a',chunks[0])+arr('code_b',chunks[1])+arr('code_c',chunks[2])+HOST,encoding='ascii',newline='\n')
 (out/'audit.json').write_text(json.dumps({'source_sha256':SOURCE_SHA256,'source_executed':False,'ranges':report,'fill':'CC','fixed_code':'006B0000','fixed_global':'01600000','global_byte':'01606354','cases':6,'limits':'6B5044 and cached output path 6B501F are INT3; SEH handler 0120C588 is not mapped. Any exception is fatal and not a comparison result. No retry.'},indent=2)+'\n')
 print('Prepared static host and preflight; no source executed:',out)
if __name__=='__main__':main()

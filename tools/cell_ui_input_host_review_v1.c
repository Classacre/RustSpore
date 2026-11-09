/* REVIEW ONLY. Fixed synthetic source save object; no game process.
 * E5BE20 and helper absolute operands and calls are explicitly relocated.
 * Three E283A0 FADD absolute sites are unreachable for immutable [2;5]. */
#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <fcntl.h>
#include <io.h>
#include "source_bytes.h"
typedef float (__cdecl *mapper)(void);
typedef void (__cdecl *selector)(float,const uint32_t*,uint32_t*,uint32_t*,float*);
struct guarded { uint32_t pre,value,post; };
struct row { uint32_t cw,progress,value,segment,bucket,fraction,sw,mxcsr; };
static const int32_t corpus[]={0,1,2,198,199,200,201,202,398,399,400,401,402,598,599,600,601,602,798,799,800,801,802,998,999,1000};
static unsigned char root[0x5194], save[0x20];
static unsigned char *root_pointer=root;
int main(void) {
 __declspec(align(16)) unsigned char saved[512];
 const uint32_t widths[5]={2,2,2,2,2}; const uint16_t controls[2]={0x027f,0x037f};
 unsigned char *code,*part[3],*save_pointer=save; unsigned int i,j; DWORD old;
 if(sizeof(void*)!=4 || sizeof(struct row)!=32 || IsDebuggerPresent())return 10;
 {BOOL remote=0;if(!CheckRemoteDebuggerPresent(GetCurrentProcess(),&remote)||remote)return 11;}
 memcpy(root+0x5190,&save_pointer,4);
 code=(unsigned char*)VirtualAlloc(0,sizeof(code0)+sizeof(code1)+sizeof(code2),MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE);if(!code)return 12;
 part[0]=code;part[1]=part[0]+sizeof(code0);part[2]=part[1]+sizeof(code1);
 memcpy(part[0],code0,sizeof(code0));memcpy(part[1],code1,sizeof(code1));memcpy(part[2],code2,sizeof(code2));
 for(i=0;i<sizeof(fixups)/sizeof(fixups[0]);i++) {uint32_t x=fixups[i][2]?(uint32_t)&constant_words[fixups[i][2]-1]:(uint32_t)&root_pointer;memcpy(part[fixups[i][0]]+fixups[i][1],&x,4);}
 for(i=0;i<sizeof(call_offsets)/sizeof(call_offsets[0]);i++) {uint32_t x=(uint32_t)(part[1]-(part[0]+call_offsets[i]+4));memcpy(part[0]+call_offsets[i],&x,4);}
 if(!VirtualProtect(code,sizeof(code0)+sizeof(code1)+sizeof(code2),PAGE_EXECUTE_READ,&old)||!FlushInstructionCache(GetCurrentProcess(),code,sizeof(code0)+sizeof(code1)+sizeof(code2)))return 13;
 if(_setmode(_fileno(stdout),_O_BINARY)==-1)return 14;
 for(j=0;j<2;j++)for(i=0;i<sizeof(corpus)/sizeof(corpus[0]);i++) {
  struct guarded seg={0x13579bdf,0xaaaaaaaa,0x2468ace0},bucket={0x13579bdf,0xaaaaaaaa,0x2468ace0},frac={0x13579bdf,0xaaaaaaaa,0x2468ace0};
  struct row r;uint16_t cw=controls[j],sw;uint32_t mx=0x1f80;float value;
  memcpy(save+0x1c,&corpus[i],4);
  __asm { fxsave saved } __asm { fninit } __asm { fldcw cw } __asm { ldmxcsr mx }
  value=((mapper)part[0])();
  ((selector)part[2])(value,widths,&seg.value,&bucket.value,(float*)&frac.value);
  __asm { fnstsw sw } __asm { stmxcsr mx } __asm { fxrstor saved }
  if(seg.pre!=0x13579bdf||seg.post!=0x2468ace0||bucket.pre!=0x13579bdf||bucket.post!=0x2468ace0||frac.pre!=0x13579bdf||frac.post!=0x2468ace0||seg.value>=5||bucket.value>=2||(sw&0x3840)||(mx&~0x3f)!=0x1f80)return 15;
  r.cw=cw;r.progress=corpus[i];memcpy(&r.value,&value,4);r.segment=seg.value;r.bucket=bucket.value;r.fraction=frac.value;r.sw=sw;r.mxcsr=mx;
  if(fwrite(&r,1,sizeof(r),stdout)!=sizeof(r))return 16;
 }
 if(fflush(stdout))return 17;VirtualFree(code,0,MEM_RELEASE);return 0;
}

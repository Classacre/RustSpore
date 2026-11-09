/* REVIEW ONLY: 763BC0 with a relocated global pointer and one explicit
 * recording forwarding stub. No provider, Raster hook, game or device runs. */
#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <io.h>
#include <fcntl.h>
#include "source_bytes.h"
struct block_g {uint32_t pre;unsigned char bytes[0x40];uint32_t post;};
struct block_i {uint32_t pre;unsigned char bytes[0x134];uint32_t post;};
struct block_c {uint32_t pre;unsigned char bytes[0x140];uint32_t post;};
struct block_r {uint32_t pre;unsigned char bytes[0x20];uint32_t post;};
struct row {uint32_t id,flag,count,matched,forward_value,result,after,calls,args_ok,guards_ok;};
typedef int (__cdecl *hook_fn)(void*,void*);
static unsigned char *global_ptr; static uint32_t stub_calls,stub_args_ok,stub_return;static void *expected_raster,*expected_context;
static uint32_t __cdecl forward_stub(void *raster,void *context){stub_calls++;stub_args_ok=(raster==expected_raster&&context==expected_context);return stub_return;}
static void put(unsigned char *p,uint32_t v){memcpy(p,&v,4);}
static uint32_t get(const unsigned char *p){uint32_t v;memcpy(&v,p,4);return v;}
int main(void){
 struct block_g g;struct block_i in;struct block_c c;struct block_r r;unsigned char g0[0x40],i0[0x134],c0[0x140],r0[0x20];
 const uint32_t flags[]={1,1,1,0,1,1},counts[]={2,2,2,2,0,0xffffffff},matches[]={1,2,0,1,0,0},returns[]={1,1,1,0,1,1};
 unsigned int i;unsigned char *code;DWORD old;uint32_t addr,delta;
 if(sizeof(void*)!=4||IsDebuggerPresent())return 10;
 {BOOL remote=0;if(!CheckRemoteDebuggerPresent(GetCurrentProcess(),&remote)||remote)return 11;}
 if(_setmode(_fileno(stdout),_O_BINARY)==-1)return 12;
 code=(unsigned char*)VirtualAlloc(0,sizeof(source_code),MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE);if(!code)return 13;
 memcpy(code,source_code,sizeof(source_code));addr=(uint32_t)&global_ptr;memcpy(code+GLOBAL_DISPLACEMENT,&addr,4);
 delta=(uint32_t)forward_stub-(uint32_t)(code+CALL_DISPLACEMENT+4);memcpy(code+CALL_DISPLACEMENT,&delta,4);
 if(!VirtualProtect(code,sizeof(source_code),PAGE_EXECUTE_READ,&old)||!FlushInstructionCache(GetCurrentProcess(),code,sizeof(source_code)))return 14;
 for(i=0;i<6;i++){
  struct row out;uint32_t flag=flags[i],count=counts[i],match=matches[i],rv;
  memset(&g,0,sizeof(g));memset(&in,0,sizeof(in));memset(&c,0,sizeof(c));memset(&r,0,sizeof(r));
  g.pre=in.pre=c.pre=r.pre=0x13579bdf;g.post=in.post=c.post=r.post=0x2468ace0;
  put(g.bytes+0x3c,(uint32_t)in.bytes);put(in.bytes+0x130,flag);put(c.bytes+0x134,count);
  put(c.bytes+0x138,match==1?(uint32_t)r.bytes:(uint32_t)(r.bytes+0x10));
  put(c.bytes+0x13c,match==2?(uint32_t)r.bytes:(uint32_t)(r.bytes+0x10));put(r.bytes+8,0xfeedcafe);
  memcpy(g0,g.bytes,sizeof(g0));memcpy(i0,in.bytes,sizeof(i0));memcpy(c0,c.bytes,sizeof(c0));memcpy(r0,r.bytes,sizeof(r0));
  global_ptr=g.bytes;expected_raster=r.bytes;expected_context=c.bytes;stub_calls=stub_args_ok=0;stub_return=returns[i];
  rv=((hook_fn)code)(r.bytes,c.bytes);
  if(flag&&count==2&&match)put(r0+8,0);
  if(memcmp(g.bytes,g0,sizeof(g0))||memcmp(in.bytes,i0,sizeof(i0))||memcmp(c.bytes,c0,sizeof(c0))||memcmp(r.bytes,r0,sizeof(r0)))return 15;
  if(g.pre!=0x13579bdf||in.pre!=g.pre||c.pre!=g.pre||r.pre!=g.pre||g.post!=0x2468ace0||in.post!=g.post||c.post!=g.post||r.post!=g.post)return 16;
  out.id=i;out.flag=flag;out.count=count;out.matched=match;out.forward_value=returns[i];out.result=rv;out.after=get(r.bytes+8);out.calls=stub_calls;out.args_ok=stub_args_ok;out.guards_ok=1;
  if(fwrite(&out,1,sizeof(out),stdout)!=sizeof(out))return 17;
 }
 if(fflush(stdout))return 18;VirtualFree(code,0,MEM_RELEASE);return 0;
}

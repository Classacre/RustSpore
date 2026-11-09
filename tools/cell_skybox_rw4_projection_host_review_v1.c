/* REVIEW ONLY: isolated non-base-resource relocation fragment and registry
 * lookup, never the live game. Fixed type 20003 avoids the truncated branch.
 * No provider, callback, Raster hook, D3D device, or texture creation runs. */
#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <io.h>
#include <fcntl.h>
#include "source_bytes.h"
static unsigned char *reloc_code,*lookup_code;
__declspec(naked) static void __cdecl relocate(uint32_t *row, unsigned char *image) {
 __asm {
  push ebx
  push esi
  mov eax,[esp+12]
  mov esi,[esp+16]
  xor ebx,ebx
  mov edx,20003h
  call dword ptr [reloc_code]
  pop esi
  pop ebx
  ret
 }
}
__declspec(naked) static uint32_t* __cdecl lookup(void *registry,uint32_t key) {
 __asm {
  mov ecx,[esp+4]
  push dword ptr [esp+8]
  call dword ptr [lookup_code]
  ret
 }
}
struct registry {uint32_t unused,count,entries[10];};
struct result {uint32_t image_base,relocated,preserved,type,found_offset,found_tag,missing,guard_ok;};
int main(void) {
 unsigned char *code,*storage,*image,*original;uint32_t *row,*found,*missing;DWORD old;struct registry r={0,2,{0x10030,0,0,0,0,0x20003,0,0,0,0}};struct result out;
 if(sizeof(void*)!=4 || IsDebuggerPresent())return 10;
 {BOOL remote=0;if(!CheckRemoteDebuggerPresent(GetCurrentProcess(),&remote)||remote)return 11;}
 if(_setmode(_fileno(stdin),_O_BINARY)==-1||_setmode(_fileno(stdout),_O_BINARY)==-1)return 12;
 storage=(unsigned char*)malloc(87780+32); original=(unsigned char*)malloc(87780);if(!storage||!original)return 13;
 memset(storage,0xa5,87780+32);image=storage+16;
 if(fread(image,1,87780,stdin)!=87780||fgetc(stdin)!=EOF)return 14;
 memcpy(original,image,87780);
 if(*(uint32_t*)(image+0x24)!=2||*(uint32_t*)(image+0x30)!=0x144||*(uint32_t*)(image+0x38)!=0)return 15;
 row=(uint32_t*)(image+0x144+24);
 if(row[0]!=0x124||row[1]!=0||row[2]!=0x20||row[3]!=4||row[4]!=5||row[5]!=0x20003)return 16;
 code=(unsigned char*)VirtualAlloc(0,sizeof(relocation)+4+sizeof(registry_lookup),MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE);if(!code)return 17;
 memcpy(code,relocation,sizeof(relocation));code[sizeof(relocation)]=0xc3;code[sizeof(relocation)+1]=0xcc;code[sizeof(relocation)+2]=0x0f;code[sizeof(relocation)+3]=0x0b;
 memcpy(code+sizeof(relocation)+4,registry_lookup,sizeof(registry_lookup));reloc_code=code;lookup_code=code+sizeof(relocation)+4;
 if(!VirtualProtect(code,sizeof(relocation)+4+sizeof(registry_lookup),PAGE_EXECUTE_READ,&old)||!FlushInstructionCache(GetCurrentProcess(),code,sizeof(relocation)+4+sizeof(registry_lookup)))return 18;
 relocate(row,image);
 found=lookup(&r,0x20003);missing=lookup(&r,0x7000c);
 if(found!=&r.entries[5]||missing||row[0]!=(uint32_t)(image+0x124)||row[1]!=0x124)return 19;
 if(memcmp(storage,"\245\245\245\245\245\245\245\245\245\245\245\245\245\245\245\245",16)||memcmp(storage+16+87780,"\245\245\245\245\245\245\245\245\245\245\245\245\245\245\245\245",16))return 20;
 memcpy(original+0x144+24,row,8);if(memcmp(image,original,87780))return 21;
 out.image_base=(uint32_t)image;out.relocated=row[0];out.preserved=row[1];out.type=row[5];out.found_offset=(uint32_t)((unsigned char*)found-(unsigned char*)&r);out.found_tag=*found;out.missing=missing?1:0;out.guard_ok=1;
 if(fwrite(&out,1,sizeof(out),stdout)!=sizeof(out)||fflush(stdout))return 22;
 VirtualFree(code,0,MEM_RELEASE);free(storage);free(original);return 0;
}

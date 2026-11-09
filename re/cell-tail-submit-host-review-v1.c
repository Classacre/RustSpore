/* REVIEW ONLY. Incomplete build inputs by design; do not compile or execute.
 * Requires generated `tail_source_bytes.inc` from the pinned PE and a copied
 * reviewed cell-tail-submit-stubs-review-v1.c. All code, patch sites, DLL
 * binding, ABI and the fully linked host need independent review. */
#include <windows.h>
#include <intrin.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "cell-tail-submit-stubs-review-v1.c"

static const unsigned char source[] = {
#include "tail_source_bytes.inc"
};
typedef void (__cdecl *tail_fn)(void*,void*,void*,uint32_t,void*,void*,int32_t,uint32_t);
typedef int (__cdecl *envelope_fn)(void*,void*,void*,void*,void*,uint32_t*);
struct Actor { uint32_t low, words[68], high; };
static struct Actor aa,bb,aa_before,bb_before;
static uint32_t output[14], output_before[14], point[3], normal[3], point_before[3], normal_before[3];
static uint32_t input[22];
static void *source_page;

/* Wrapper calls this cdecl thunk with no args. Its actual linked instructions
 * must be checked for no FP/SIMD instructions and balanced nonvolatile state. */
__declspec(noinline) static uint32_t __cdecl source_thunk(void) {
    ((tail_fn)source_page)(output+1,point,normal,input[20],aa.words,bb.words,5,input[21]);
    return 0x72a94b16u;
}
static uint16_t u16(const unsigned char *p) { uint16_t v; memcpy(&v,p,2); return v; }
static uint32_t u32(const unsigned char *p) { uint32_t v; memcpy(&v,p,4); return v; }

int main(int argc,char **argv) {
    unsigned char *fp[4]={NULL,NULL,NULL,NULL}; HMODULE dll=NULL; SYSTEM_INFO si;
    BOOL remote=FALSE; int regs[4], status=1; DWORD old; uint32_t thunk_result=0xa9a9a9a9u;
    envelope_fn envelope; char fullpath[MAX_PATH]; DWORD length;
    if(sizeof(void*)!=4 || argc!=24 || sizeof(source)!=0x192) return 10;
    length=GetFullPathNameA(argv[1],MAX_PATH,fullpath,NULL);
    if(!length || length>=MAX_PATH || strcmp(argv[1],fullpath) || fullpath[1]!=':' || fullpath[2]!='\\') return 11;
    if(IsDebuggerPresent() || !CheckRemoteDebuggerPresent(GetCurrentProcess(),&remote) || remote) return 12;
    __cpuid(regs,1);
    if(!(regs[3]&(1<<24)) || !(regs[3]&(1<<26)) || !IsProcessorFeaturePresent(PF_XMMI64_INSTRUCTIONS_AVAILABLE)) return 13;
    for(int i=0;i<22;i++) {
        char *end; if(strlen(argv[i+2])!=8) return 14;
        for(int j=0;j<8;j++) { char c=argv[i+2][j]; if(!((c>='0'&&c<='9')||(c>='a'&&c<='f')||(c>='A'&&c<='F'))) return 14; }
        input[i]=(uint32_t)strtoul(argv[i+2],&end,16); if(*end) return 14;
    }
    if((input[5]&0x7f800000u)==0x7f800000u || (input[20]&0x7f800000u)==0x7f800000u) return 15;
    memset(&aa,0,sizeof aa); memset(&bb,0,sizeof bb);
    aa.low=0xa1a1a1a1; aa.high=0xa2a2a2a2; bb.low=0xb1b1b1b1; bb.high=0xb2b2b2b2;
    aa.words[0]=input[12]; aa.words[0x108/4]=0x2468ace0u;
    bb.words[0]=input[13]; bb.words[0x10c/4]=input[12]^1u;
    memcpy(&aa_before,&aa,sizeof aa); memcpy(&bb_before,&bb,sizeof bb);
    output[0]=0x9a9a9a9a; output[13]=0x6b6b6b6b; memcpy(output+1,input,48); memcpy(output_before,output,sizeof output);
    memcpy(point,input+14,12); memcpy(normal,input+17,12); memcpy(point_before,point,12); memcpy(normal_before,normal,12);
    trace_count=0; memset((void*)trace_kind,0,sizeof trace_kind); memset((void*)trace_return,0,sizeof trace_return);
    trace_actor=trace_definition_arg=0; memset((void*)trace_e59_args,0,sizeof trace_e59_args);
    GetSystemInfo(&si); if(si.dwPageSize<4096) return 16;
    for(int i=0;i<4;i++) {
        MEMORY_BASIC_INFORMATION info;
        fp[i]=(unsigned char*)VirtualAlloc(NULL,si.dwPageSize,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE);
        if(!fp[i] || ((uintptr_t)fp[i]&15u) || VirtualQuery(fp[i],&info,sizeof info)!=sizeof info ||
           info.State!=MEM_COMMIT || info.Protect!=PAGE_READWRITE || info.RegionSize<512 || info.BaseAddress!=fp[i]) goto done;
        for(int j=0;j<i;j++) if(fp[i]==fp[j]) goto done;
        memset(fp[i],0xa5,512);
    }
    source_page=VirtualAlloc(NULL,0x1000,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE); if(!source_page) goto done;
    memcpy(source_page,source,sizeof source);
    /* The five rel32 fields are patched modulo 2^32 on x86; verify all E8
       opcodes first. Source encoding and all resulting targets need audit. */
    { const unsigned offsets[5]={0x12,0x32,0x43,0x7a,0xa4};
      void *targets[5]={(void*)stub_e6a250,(void*)stub_e4cb40,(void*)stub_forbidden,(void*)stub_e59b40,(void*)stub_e59b40};
      for(int i=0;i<5;i++) { unsigned char *call=(unsigned char*)source_page+offsets[i]; uint32_t rel;
          if(*call!=0xe8) goto done;
          rel=(uint32_t)(uintptr_t)targets[i]-(uint32_t)(uintptr_t)(call+5); memcpy(call+1,&rel,4);
      }
    }
    if(!VirtualProtect(source_page,0x1000,PAGE_EXECUTE_READ,&old) || !FlushInstructionCache(GetCurrentProcess(),source_page,sizeof source)) goto done;
    dll=LoadLibraryExA(fullpath,NULL,LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR|LOAD_LIBRARY_SEARCH_SYSTEM32); if(!dll) goto done;
    envelope=(envelope_fn)(uintptr_t)GetProcAddress(dll,"envelope"); if(!envelope) goto done;
    if(envelope(fp[0],fp[1],fp[2],fp[3],(void*)source_thunk,&thunk_result)!=0 || thunk_result!=0x72a94b16u) goto done;
    /* Pointer words are compared to trusted buffers before any interpretation. */
    if(trace_count!=3 || trace_kind[0]!=1 || trace_kind[1]!=2 || trace_kind[2]!=3 ||
       trace_return[0]!=0 || trace_return[1]!=0x13579bdfu || trace_return[2]!=0 ||
       trace_actor!=(uint32_t)(uintptr_t)aa.words || trace_definition_arg!=0x17 ||
       trace_e59_args[0]!=(uint32_t)(uintptr_t)(output+1) || trace_e59_args[1]!=(uint32_t)(uintptr_t)point ||
       trace_e59_args[2]!=(uint32_t)(uintptr_t)normal || trace_e59_args[3]!=input[20] ||
       trace_e59_args[4]!=input[12] || trace_e59_args[5]!=input[13] ||
       trace_e59_args[6]!=0xffffffffu || trace_e59_args[7]!=0xffffffffu ||
       trace_e59_args[8]!=(input[21]?0x11u:0x16u)) goto done;
    if(memcmp(&aa,&aa_before,sizeof aa) || memcmp(&bb,&bb_before,sizeof bb) || memcmp(output,output_before,sizeof output) ||
       memcmp(point,point_before,12) || memcmp(normal,normal_before,12)) goto done;
    if(u16(fp[1])!=0x027f || u16(fp[1]+2)!=0 || fp[1][4]!=0 || u32(fp[1]+24)!=0x1f80) goto done;
    /* Permit DE sticky for subnormal FLD; reject all other status/stack bits. */
    if(u16(fp[2])!=0x027f || (u16(fp[2]+2)&0xb8fdu)!=0 || fp[2][4]!=0 || (u32(fp[2]+24)&~0x3fu)!=0x1f80) goto done;
    if(u16(fp[0])!=u16(fp[3]) || u16(fp[0]+2)!=u16(fp[3]+2) || fp[0][4]!=fp[3][4] ||
       u32(fp[0]+24)!=u32(fp[3]+24) || memcmp(fp[0]+160,fp[3]+160,128)) goto done;
    /* Fixed-width protocol: raw trace words and unchanged output. */
    for(int i=0;i<9;i++) printf("%08x%c",(unsigned)trace_e59_args[i],i==8?'\n':' ');
    for(int i=1;i<=12;i++) printf("%08x%c",(unsigned)output[i],i==12?'\n':' ');
    status=0;
done:
    if(dll) FreeLibrary(dll);
    if(source_page) VirtualFree(source_page,0,MEM_RELEASE);
    for(int i=0;i<4;i++) if(fp[i]) VirtualFree(fp[i],0,MEM_RELEASE);
    return status;
}

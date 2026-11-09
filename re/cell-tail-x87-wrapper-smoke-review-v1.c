/* REVIEW ONLY: unbuilt and unexecuted standalone wrapper smoke host.
 * It does not execute source instructions. Only normal returns are covered.
 * Before build/run: inspect compiler output, DLL loader binding and the exact
 * non-source thunk, independently pin binaries and fixture inputs, and review
 * all Win32 feature, allocation, debugger, and cleanup assumptions.
 */
#include <windows.h>
#include <intrin.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

typedef int (__cdecl *envelope_fn)(void *, void *, void *, void *, void *, uint32_t *);
__declspec(noinline) static uint32_t __cdecl fixture_thunk(void) { return 0x72a94b16u; }
static uint16_t u16(const unsigned char *p) { uint16_t x; memcpy(&x,p,2); return x; }
static uint32_t u32(const unsigned char *p) { uint32_t x; memcpy(&x,p,4); return x; }

int main(int argc, char **argv) {
    int regs[4], status=1; BOOL remote=FALSE; HMODULE dll=NULL;
    unsigned char *area[4]={NULL,NULL,NULL,NULL}; uint32_t result=0xa9a9a9a9u;
    envelope_fn envelope; SYSTEM_INFO si; char fullpath[MAX_PATH]; DWORD pathlen;
    if (sizeof(void*)!=4 || argc!=2) return 10;
    /* Require a fully expanded absolute path. The external runner must also
       independently hash and bind this exact file before/after execution. */
    pathlen=GetFullPathNameA(argv[1],MAX_PATH,fullpath,NULL);
    if(pathlen==0 || pathlen>=MAX_PATH || strcmp(argv[1],fullpath)!=0 ||
       !(fullpath[0] && fullpath[1]==':' && (fullpath[2]=='\\' || fullpath[2]=='/'))) return 14;
    if (IsDebuggerPresent() || !CheckRemoteDebuggerPresent(GetCurrentProcess(),&remote) || remote) return 11;
    __cpuid(regs,1);
    if ((regs[3] & (1<<24))==0 || (regs[3] & (1<<26))==0 ||
        !IsProcessorFeaturePresent(PF_XMMI64_INSTRUCTIONS_AVAILABLE)) return 12;
    GetSystemInfo(&si); if(si.dwPageSize<4096) return 13;
    for(int i=0;i<4;i++) {
        MEMORY_BASIC_INFORMATION info;
        area[i]=(unsigned char*)VirtualAlloc(NULL,si.dwPageSize,MEM_COMMIT|MEM_RESERVE,PAGE_READWRITE);
        if(!area[i] || ((uintptr_t)area[i]&15u)!=0) goto cleanup;
        if(VirtualQuery(area[i],&info,sizeof(info))!=sizeof(info) || info.State!=MEM_COMMIT ||
           info.Protect!=PAGE_READWRITE || info.RegionSize<512 || info.BaseAddress!=area[i]) goto cleanup;
        for(int j=0;j<i;j++) if(area[i]==area[j]) goto cleanup;
        memset(area[i],0xa5,512);
    }
    /* The caller must pass an absolute, hash-pinned DLL path and ensure it
       remains stable throughout; this code does not cryptographically bind it. */
    dll=LoadLibraryExA(argv[1],NULL,LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR|LOAD_LIBRARY_SEARCH_SYSTEM32);
    if(!dll) goto cleanup;
    envelope=(envelope_fn)(uintptr_t)GetProcAddress(dll,"envelope");
    if(!envelope) goto cleanup;
    int ret=envelope(area[0],area[1],area[2],area[3],(void*)(uintptr_t)fixture_thunk,&result);
    if(ret!=0 || result!=0x72a94b16u) goto cleanup;
    /* Exact fields only. FXSAVE FTW is abridged; empty register payload,
       reserved bytes and software bytes are intentionally not compared. */
    if(u16(area[1])!=0x027f || u16(area[1]+2)!=0 || area[1][4]!=0 || u32(area[1]+24)!=0x1f80) goto cleanup;
    if(u16(area[2])!=0x027f || (u16(area[2]+2)&0xb8ff)!=0 || area[2][4]!=0 ||
       (u32(area[2]+24)&~0x3fu)!=0x1f80) goto cleanup;
    if(u16(area[0])!=u16(area[3]) || u16(area[0]+2)!=u16(area[3]+2) ||
       area[0][4]!=area[3][4] || u32(area[0]+24)!=u32(area[3]+24) ||
       memcmp(area[0]+160,area[3]+160,128)!=0) goto cleanup;
    for(int i=0;i<4;i++) {
        printf("image%d fcw=%04x fsw=%04x ftw=%02x mxcsr=%08x xmm=",i,
               (unsigned)u16(area[i]),(unsigned)u16(area[i]+2),area[i][4],(unsigned)u32(area[i]+24));
        for(int j=0;j<128;j++) printf("%02x",area[i][160+j]);
        putchar('\n');
    }
    status=0;
cleanup:
    if(dll) FreeLibrary(dll);
    for(int i=0;i<4;i++) if(area[i]) VirtualFree(area[i],0,MEM_RELEASE);
    if(status==0) puts("x87_wrapper_smoke=pass");
    return status;
}

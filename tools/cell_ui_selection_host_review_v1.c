/* REVIEW ONLY: fixed E283A0 host. Generated source_bytes.h required.
 * x86 MSVC, no game process. The three absolute FADD sites in the source slice
 * are unreachable ONLY because total=10 and each of the five widths=2.
 * Keep input and code immutable; inspect the linked binary before any run. */
#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <fcntl.h>
#include <io.h>
#include "source_bytes.h"
typedef void (__cdecl *selector)(float, const uint32_t *, uint32_t *, uint32_t *, float *);
struct guarded { uint32_t pre, value, post; };
struct row { uint32_t cw, input, segment, bucket, fraction, sw, mxcsr; };
static const uint32_t input_bits[] = {
    0,1,0x3dcccccc,0x3dcccccd,0x3dccccce,
    0x3e4ccccc,0x3e4ccccd,0x3e4cccce,
    0x3e999999,0x3e99999a,0x3e99999b,
    0x3ecccccc,0x3ecccccd,0x3eccccce,
    0x3effffff,0x3f000000,0x3f000001,
    0x3f199999,0x3f19999a,0x3f19999b,
    0x3f333332,0x3f333333,0x3f333334,
    0x3f4ccccc,0x3f4ccccd,0x3f4cccce,
    0x3f666665,0x3f666666,0x3f666667,
    0x3f7fffff,0x3f800000
};
int main(void) {
    __declspec(align(16)) unsigned char saved[512];
    const uint32_t widths[5] = {2,2,2,2,2};
    const uint16_t words[2] = {0x027f,0x037f};
    DWORD old; unsigned int i,j;
    unsigned char *code;
    if (sizeof(void*) != 4 || sizeof(float) != 4 || sizeof(struct row) != 28) return 10;
    if (IsDebuggerPresent()) return 11;
    { BOOL remote = 0; if (!CheckRemoteDebuggerPresent(GetCurrentProcess(), &remote) || remote) return 12; }
    code = (unsigned char*)VirtualAlloc(0,sizeof(source_bytes),MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE);
    if (!code) return 13;
    memcpy(code,source_bytes,sizeof(source_bytes));
    if (!VirtualProtect(code,sizeof(source_bytes),PAGE_EXECUTE_READ,&old) || !FlushInstructionCache(GetCurrentProcess(),code,sizeof(source_bytes))) return 14;
    if (_setmode(_fileno(stdout),_O_BINARY) == -1) return 15;
    for (j=0;j<2;j++) for (i=0;i<sizeof(input_bits)/sizeof(input_bits[0]);i++) {
        struct guarded seg={0x13579bdf,0xaaaaaaaa,0x2468ace0};
        struct guarded bucket={0x13579bdf,0xaaaaaaaa,0x2468ace0};
        struct guarded fraction={0x13579bdf,0xaaaaaaaa,0x2468ace0};
        struct row r; float p; uint32_t mx=0x1f80; uint16_t sw, cw=words[j];
        memcpy(&p,&input_bits[i],4);
        __asm { fxsave saved }
        __asm { fninit }
        __asm { fldcw cw }
        __asm { ldmxcsr mx }
        ((selector)code)(p,widths,&seg.value,&bucket.value,(float*)&fraction.value);
        __asm { fnstsw sw }
        __asm { stmxcsr mx }
        __asm { fxrstor saved }
        if (seg.pre!=0x13579bdf || seg.post!=0x2468ace0 || bucket.pre!=0x13579bdf || bucket.post!=0x2468ace0 || fraction.pre!=0x13579bdf || fraction.post!=0x2468ace0) return 16;
        if (seg.value>=5 || bucket.value>=2 || (sw & 0x3840) || (mx & ~0x3f)!=0x1f80) return 17;
        r.cw=words[j];r.input=input_bits[i];r.segment=seg.value;r.bucket=bucket.value;r.fraction=fraction.value;r.sw=sw;r.mxcsr=mx;
        if (fwrite(&r,1,sizeof(r),stdout)!=sizeof(r)) return 18;
    }
    if (fflush(stdout)) return 19;
    VirtualFree(code,0,MEM_RELEASE);
    return 0;
}

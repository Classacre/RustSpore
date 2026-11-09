/* REVIEW ONLY: stub and trace design for E6A2B0 relation-5 submission.
 * This is not a complete host, has not been built, and must not be executed.
 * MSVC x86 naked procedures shown so an auditor can inspect exact ABI effects.
 * All three stubs have integer-only instructions and use no x87, MMX or XMM.
 */
#include <stdint.h>

/* Only one single-threaded child invocation is admitted. These globals must be
 * disjoint from code, actors, geometry, output, FXSAVE images and result slot.
 * All slots are fixed length. A trap means failed child, never a retry. */
volatile uint32_t trace_count;
volatile uint32_t trace_kind[3];
volatile uint32_t trace_return[3];
volatile uint32_t trace_actor;
volatile uint32_t trace_definition_arg;
volatile uint32_t trace_e59_args[9];

/* Source calls E6A250(A) cdecl, then removes its argument. */
__declspec(naked) void stub_e6a250(void) {
    __asm {
        cmp dword ptr [trace_count], 0
        jne fail
        mov dword ptr [trace_kind], 1
        mov eax, [esp+4]
        mov [trace_actor], eax
        mov dword ptr [trace_return], 0
        mov dword ptr [trace_count], 1
        xor eax, eax                 /* controlled low-byte return zero */
        ret
    fail:
        int 3
        ud2
    }
}

/* Source calls E4CB40(0x17), then removes its argument. */
__declspec(naked) void stub_e4cb40(void) {
    __asm {
        cmp dword ptr [trace_count], 1
        jne fail
        mov dword ptr [trace_kind+4], 2
        mov eax, [esp+4]
        mov [trace_definition_arg], eax
        mov dword ptr [trace_return+4], 13579BDFh
        mov dword ptr [trace_count], 2
        mov eax, 13579BDFh            /* distinct from admitted A+108 */
        ret
    fail:
        int 3
        ud2
    }
}

/* Source calls E59B40 with nine stack words, then removes all 36 bytes.
 * Record raw words only: no pointer is dereferenced within the envelope. */
__declspec(naked) void stub_e59b40(void) {
    __asm {
        cmp dword ptr [trace_count], 2
        jne fail
        mov dword ptr [trace_kind+8], 3
        xor edx, edx
    copy_args:
        mov eax, [esp+edx*4+4]
        mov [trace_e59_args+edx*4], eax
        inc edx
        cmp edx, 9
        jb copy_args
        mov dword ptr [trace_return+8], 0
        mov dword ptr [trace_count], 3
        xor eax, eax                  /* caller ignores controlled return */
        ret
    fail:
        int 3
        ud2
    }
}

/* Any source call outside this controlled sequence must target this trap. */
__declspec(naked) void stub_forbidden(void) { __asm { int 3 } __asm { ud2 } }

/* The eventual host must: bind the reviewed x87 wrapper DLL; set a known empty
 * x87 stack through that wrapper; construct guarded A>=0x10c, B>=0x110 and
 * other disjoint objects; verify A.id != B+10c and A+108 != 13579bdf after
 * each admitted pure stub; patch exactly the five source call sites; use a
 * bounded no-FP thunk; and validate all trace words and exact pointer equality
 * before dereferencing geometry. Stub globals are bounded only by this fixed
 * design, not by guard pages. All compiled bytes and control flow require
 * independent review before execution. */

; DESIGN ONLY. Not assembled or executed. This is a review skeleton, not a
; callable host. The checked source/stub thunk and its ABI are NOT supplied.
; Preconditions for an eventual isolated x86 test process: CPU FXSR+SSE2 and
; OS support, three distinct 16-byte-aligned writable 512-byte areas, valid
; bounded pointers, no concurrent signal/exception handler changing FP state.
;
; FXSAVE format in 32-bit mode: FCW +0 (word), FSW +2 (word), abridged FTW
; +4 (byte), MXCSR +24 (dword). The prior complete architectural x87/MMX/XMM
; and MXCSR state is the saved area. It does not cover AVX upper registers or
; arbitrary extended state. Exact C/assembler ABI and register handling must
; be reviewed in the assembled linked host.
;
; Intended straight-line region (all branches on validation go to restore):
;
;   fxsave [saved]                 ; no waiting x87 instruction, no reset
;   ; reject if saved.FSW has any exception flag bits 0..5, stack fault bit 6,
;   ; exception summary bit 7, or busy bit 15. Do not call FNINIT on rejection.
;   ; Reject nonempty saved FTW or nonzero saved TOP; FTW is abridged.
;   ; On rejection return without changing FP environment; no source call.
;   fninit                         ; explicit fixture reset, safe only after above
;   fldcw [fixture_cw_027f]        ; masked, nearest, 53-bit precision
;   ldmxcsr [fixture_mxcsr_1f80]  ; masked, nearest, no DAZ or FTZ
;   fxsave [before]
;   ; require before.FCW=027f, before.FSW=0, before.FTW=0, MXCSR=1f80.
;   ; Call reviewed native thunk here, with bounded stack and prevalidated
;   ; pointers. Thunk must preserve nonvolatile registers and its ESP.
;   fxsave [after]                 ; capture before any other FP operation
;   fxrstor [saved]                ; restore original admitted legacy FP state
;   ; Compare after.FCW=027f, after.FTW=0, after TOP=(FSW>>11)&7=0;
;   ; require stack fault and exception-summary clear, SSE control unchanged.
;   ; Retain status flags as observations: finite subnormal FLD may set DE.
;   ; Compare saved and post-restore architectural state where safely possible.
;
; A fault, debugger interception, corrupted saved area, incorrect ESP, or lost
; control before FXRSTOR has no verified recovery here. The isolated child must
; be marked failed and terminated; no claim of in-process recovery is made.
; FXSAVE/FXRSTOR and FNINIT do not provide arbitrary asynchronous fault safety.

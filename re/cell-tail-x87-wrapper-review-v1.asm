; REVIEW ONLY -- not assembled or run. x86 MASM, cdecl wrapper.
; int envelope(saved,before,after,restored,thunk,thunk_result)
; Four caller-owned disjoint 512-byte, 16-byte-aligned read/write areas; all
; accessible for the full call. thunk_result is a distinct writable 4-byte
; slot, disjoint from all four areas, code, source inputs, output and log.
; Caller admits FXSR/SSE2 and OS support, and a
; synchronous thunk with cdecl ABI, no args, balanced ESP, preserved EBP and
; no unhandled exceptions. It may clobber EAX/ECX/EDX, x87 and XMM state.
; No asynchronous recovery is supplied. A fault requires isolated child exit.
;
; Wrapper EAX=0 says the capture sequence completed; thunk EAX is stored in
; thunk_result immediately after the call. Nonzero wrapper EAX is wrapper
; failure; thunk_result is untouched on pre-call failure. Caller MUST
; inspect before/after/restored fields and the thunk result separately; this
; wrapper does not establish source semantics or compare reserved/payload bytes.
.386
.xmm
.model flat, c

.const
fixture_cw WORD 027Fh
fixture_mxcsr DWORD 00001F80h

.code
envelope PROC C saved:DWORD, before_state:DWORD, after_state:DWORD, restored:DWORD, thunk:DWORD, thunk_result:DWORD
    push ebx
    push esi
    push edi
    ; MASM-generated EBP frame is required; verify actual prologue in listing.
    mov ebx, saved
    mov esi, before_state
    mov edi, after_state
    mov eax, restored
    mov ecx, thunk
    test ebx, ebx
    jz bad_input
    test esi, esi
    jz bad_input
    test edi, edi
    jz bad_input
    test eax, eax
    jz bad_input
    test ecx, ecx
    jz bad_input
    cmp thunk_result, 0
    je bad_input
    mov edx, ebx
    or edx, esi
    or edx, edi
    or edx, eax
    test edx, 15
    jnz bad_input
    ; Range/disjointness and mapping are caller admissions, not validated here.
    fxsave [ebx]
    ; Reject pending x87 exception/status, nonempty tag or nonzero TOP.
    ; FSW mask = B | TOP | ES | SF | six exception flags.
    test WORD PTR [ebx+2], 0B8FFh
    jnz bad_saved
    cmp BYTE PTR [ebx+4], 0
    jne bad_saved
    fninit
    fldcw fixture_cw
    ldmxcsr fixture_mxcsr
    fxsave [esi]
    cmp WORD PTR [esi], 027Fh
    jne fixture_error
    cmp WORD PTR [esi+2], 0
    jne fixture_error
    cmp BYTE PTR [esi+4], 0
    jne fixture_error
    cmp DWORD PTR [esi+24], 01F80h
    jne fixture_error
    ; The supplied thunk MUST adhere to the reviewed ABI, including EBP/ESP.
    call thunk
    mov edx, thunk_result
    mov [edx], eax
    ; Re-fetch buffer pointers from the stable EBP frame after the thunk.
    mov edi, after_state
    fxsave [edi]
    mov ebx, saved
    fxrstor [ebx]
    mov eax, restored
    fxsave [eax]
    xor eax, eax
    jmp done
fixture_error:
    mov ebx, saved
    fxrstor [ebx]
    mov eax, restored
    fxsave [eax]
    mov eax, 3
    jmp done
bad_saved:
    ; FXSAVE has not changed the architectural FP state.
    mov eax, 2
    jmp done
bad_input:
    mov eax, 1
done:
    pop edi
    pop esi
    pop ebx
    ret
envelope ENDP
END

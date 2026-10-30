# Cell quaternion-to-matrix source instruction boundary

Static x86 Capstone trace of installed `SporeApp.exe` SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. Addresses use preferred image base. This extends [the null-orientation trace](cell-start-null-orientation-static-v1.md); no game, original snippet, Cargo or GPU code was run.

## Caller, storage and layout

On the admitted constructor path, `E74C10..E74C31` copies actor +08/+0C/+10 to +4C/+50/+54, ORs 4 into 16-bit +48 and increments 16-bit +4A. It pushes the address of actor +14 and a temporary output address, then calls `59C000`. `E74C36..E74C44` copies nine consecutive dwords from the returned pointer to actor +5C through +7C using `rep movsd`; `E74C46..E74C50` ORs 2 into +48 and increments +4A again. Thus the source stores nine sequential binary32 words in three contiguous groups of three. The calling convention does not alone establish whether those groups are semantic rows or columns, a world/view basis, or how later matrix consumers multiply them; preserve the nine-word physical order until a separate consumer trace establishes that convention.

`59C000` reads four input f32 words at offsets 0,4,8,12, denoted `(x,y,z,w)` below. It does not normalize them. `41CAE0` copies the resulting nine temporary values in order into the supplied destination; its staged copies matter if caller buffers alias. A safe bounded fixture should require non-overlapping live input/output and valid memory rather than reproduce unproved alias behavior. This use follows the null-orientation branch, but `59C000` itself can be called with other inputs.

## Exact scalar operation dependency order

Use `M(a,b)`, `A(a,b)` and `S(a,b)` to mean one source `MULSS`, `ADDSS` and `SUBSS`, respectively, each committed to binary32 under the admitted MXCSR. `1` is the installed f32 bit pattern `3F800000` at `014861D0`; `2` is `40000000` at `014719E4`. Products are made in source order (`59C021..59C076`):

```text
zz=M(z,z); zx=M(z,x); wx=M(w,x); yx=M(y,x); xx=M(x,x)
zy=M(z,y); yy=M(y,y); wz=M(w,z); wy=M(w,y)
```

The output slots are written in this dependency/order (source can interleave independent operations):

```text
0 = S(1, M(A(zz,yy),2))       // temporary +00 -> actor +5C
1 = M(A(wz,yx),2)             // +04 -> +60
2 = M(S(zx,wy),2)             // +08 -> +64
3 = M(S(yx,wz),2)             // +0C -> +68
4 = S(1, M(A(zz,xx),2))       // +10 -> +6C
5 = M(A(wx,zy),2)             // +14 -> +70
6 = M(A(wy,zx),2)             // +18 -> +74
7 = M(S(zy,wx),2)             // +1C -> +78
8 = S(1, M(A(yy,xx),2))       // +20 -> +7C
```

In particular, the source computes `A(wy,zx)` (not the reverse) for slot 6 and `A(wx,zy)` for slot 5. For finite operands those additions often agree numerically, but operand order matters for exceptional values and exact source representation. There is no fused operation, deferred extra precision, algebraic reassociation or implicit quaternion normalization in the inspected body. The source loads 1.0 twice, and stores and reloads intermediate binary32 words; those moves do not add an arithmetic rounding step.

## FP state and admissible model

The helper does not set MXCSR. Its result depends on the incoming scalar SSE state, including rounding, FTZ/DAZ and exception masks/flags; no original live MXCSR has been established. A bounded model can accept explicitly admitted finite normal or signed-zero quaternion inputs, use a scoped explicitly named MXCSR policy (for example the existing `native_sse::Policy::Mxcsr1f80`), issue the listed scalar operations in order, and reject nonfinite intermediate/results and unsupported environments. This would model the admitted numeric path, not the original exception flags, memory aliasing, data race, live pose or actual runtime policy. Subnormal intermediate results under gradual underflow must either be preserved or excluded explicitly; rejecting subnormal **inputs** does not rule them out.

The prior null-orientation result is a valid explicit fixture input, but its synthetic RNG/global/axis/FP state and conditional actor allocation are not evidence of an actual Cell actor. Compare model output against an independent integer/binary32 SSE oracle over exact bits, including signed zeros, non-unit quaternions, cancellation, normal/subnormal boundaries and deliberate nonfinite rejection. A separate guarded original-byte host with exact code/constants, source ABI, MXCSR, nonaliasing, memory guards and independent review is needed for source-instruction validation; a rational model, host SSE approximation or equivalent matrix library alone is not such evidence. `E74C` field mutations and counter updates can be checked separately under explicit initial words, including wrapping 16-bit increment semantics.

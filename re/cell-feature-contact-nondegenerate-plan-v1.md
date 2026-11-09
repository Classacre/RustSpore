# Nondegenerate Cell feature contact: implementation and comparison plan

This is a plan only. No Rust code or original fragment was executed for it.
The static derivation, branches and argument offsets are in
[cell-feature-contact-geometry-continuation-v1.md](cell-feature-contact-geometry-continuation-v1.md).
The 184 bytes `E6B099..E6B150` of installed `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`
have SHA-256 `c16003e09ec659ab4f2d729301fa51a4dbd5ce9cfa90756776149069e8f5ef74`.
These hashes identify an inspection target, not a runtime observation.

## Model boundary

1. Accept explicitly supplied first/second projected centers and radii and a
   checked pair prefix, or recompute that prefix under the existing finite-input
   policy. Recheck fresh, stable, disjoint snapshot identity at the boundary;
   do not infer an actor, cache or feature pair from a matching word.
2. Keep `d2 < 37800000` separate. Return a pending small-distance result with
   the radius-sum depth and caller-admitted six opaque local words, or stop
   before reading them. Do not initialize, sanitize, interpret or claim they
   are valid normal/point words. A further E6A450 call requires fresh values,
   alias/lifetime admission and callback protocol; it is outside this plan.
3. For finite `d2 >= 37800000`, admit x87 control word 027F, an empty x87
   stack and masked exceptions, plus MXCSR 1F80. Use the existing
   `native_x87::Context` load/sqrt/store sequence only after checking its
   semantics and error mapping against `FLD m32; FSQRT; FSTP m32`; store the
   binary32 result before SSE subtraction. Return a pending separated result
   when ordered zero is greater than depth; equality proceeds. Reject unsafe
   nonfinite inputs/intermediates and unsupported FP/alias conditions as
   checked policy, not source behavior.
4. For accepted finite inputs implement the operand ordering and per-operation
   rounding in the linked trace note, returning raw distance, depth, normal and
   point. Preserve source operand order, especially Bx+Ax versus Ay+By/Az+Bz,
   and dz*inv versus inv*dx/dy. Stop before E6A450: no candidate arbitration,
   contact event, actor mutation or gameplay claim. Negative radii and other
   unsupported source cases must be either explicitly admitted and tested or
   rejected as model policy.

Meaningful unit cases: exact tangent at distance 1 and radii 0.5+0.5; a
separated pair with radii 0.25+0.25; an overlapping unequal-radius pair with
known normal and point; asymmetric 3-4-5 direction; finite cancellation and
rounding cases that distinguish operation grouping; the exact threshold,
just-below and just-above cases; and explicit prefilled small-branch locals
that must be returned unchanged or left unread. Test rejected nonfinite,
overflow and unsupported control states separately as Rust policy, not native
behavior. Ensure x87/MXCSR state restoration on every model exit.

## Independently reviewed host proposal

The source slice begins after threshold admission. Under a caller-provided
frame, initialize XMM0 to the first-plus-second radius sum in source operand
order; XMM1/XMM6/XMM4 to dx/dy/dz; XMM5/XMM3/XMM7 to Bx/By/Bz; and source
stack +20 to d2, +30/+34/+38 to Ax/Ay/Az, +5C/+60 to rb/ra. Other registers
must be seeded or shown unused. Reserve and guard adequate stack scratch,
accounting for CALL's four-byte return address, and save and restore host
nonvolatiles. Use separate, stable, canaried input/output buffers and known
initial local sentinels.

The contiguous fragment has one external conditional branch at `E6B0B5`
(`0F 87 EF 00 00 00`, fragment offset 0x1C), and two absolute data operands:
constant 1 address bytes at offset 0x26 and constant 0.5 address bytes at
0x8E. A prepared host must preserve every other byte, relocate those two
four-byte operands to guarded host constants, and redirect the branch's rel32
operand to an explicit in-host separated trampoline. The fallthrough returns
through an explicit nonseparated trampoline. Independently decode the linked
PE to verify patched locations, destination boundaries, preserved branch
meaning, embedded source bytes, entry, imports, memory bounds and ABI before
any execution. No original executable, game process, network or GUI is needed.

Initialize x87 with a known empty stack and control 027F and MXCSR 1F80 for
each fixed finite case; record control/status and MXCSR before/after, but do
not claim sticky-status equivalence unless separately compared. Capture the
source-written binary32 distance and depth for both paths; capture normal and
point only on fallthrough. For separated cases prove that guarded sentinel
normal/point bytes were not written. Compare raw words and branch outcome with
a separately built, hash-pinned model adapter and corpus. Threshold neighbors
belong in a separately admitted gate fixture or in the already compared pair
prefix; this fragment itself assumes its gate was passed.

Retain source image, extracted bytes, patched-byte map, source/linked assembly,
compiler/linker and library identities, corpus, model adapter, host, runner,
independent review, raw stdout/stderr and a success or failure receipt. Use
exclusive fresh directories, timeout and no-window creation; preserve every
failure. Check pins immediately before and after and reject reparse points;
these checks do not prove atomic stable-file admission or exclude transient
replacement. Do not run the host until its exact files, pins, inputs and command
have been independently reviewed. A match would cover only the declared finite
cases and this fragment, not source actors, cache lifetime, the small branch's
locals, E6A450, callbacks, actual contacts or gameplay.

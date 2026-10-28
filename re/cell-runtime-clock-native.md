# Animation clock arithmetic gates

[MEASURED] Original9A2BB0 advances by FLD32 delta, FADD64 stored time and
FSTP32 the next argument. The admitted draft9A1CDB path stores that binary32
time as64 bits, divides by first duration under the explicit x87 policy,
commits a binary32 normalized value, then computes SUBSS(resourceFrames,1)
and MULSS(normalized). Strict time<duration and no-loop branches lead to the
exit immediately before sampling callback9A1590.

The original four partial spans total146 bytes. Original calls are absent;
event reset, crossing counters and fraction/loop helper paths remain unexecuted.
All twelve masked x87 PC/RC contexts and MXCSR1F80 are explicit, rather than
inferred retail thread state. Loops, nonfinite input/result, nonpositive
duration, negative committed time, crossing and invalid frame count are typed
policy exclusions in the accepted first-duration Rust API. The distinct loop
arithmetic API below requires an explicit state boundary.

[MEASURED] Preparation contains1,560 valid requests, including literal frame
words from eight source resources with supplied times/durations. Sixteen
negative protocol cases remain separate. Root independently reproduced and
compiled host v1 and passed source/import/section/guard/PE-entry audits.
An independent reviewer additionally checked all input domains with exact
rational arithmetic. This is input/host preparation, not arithmetic acceptance.

## Retained v1 failure

[MEASURED] Root run `target/source-cell/runtime-clock-native-run-v1/` passed
standalone absolute/delta references and earlier protocol checks, then
`crossing_equal` exited C0000005 with zero output/stderr. The1,560-case corpus
never ran. All initial before/after execution pins are retained. The failed
host/input is not retried. V1 EXE SHA256:
`9b78caa91da34bcdbc26007c3f14f58fc2e9e9c8ba3c14d98eab62d4fd1fb72b`.

[INFERRED] The source rejection falls through9A1CFF to the owned HostFailure
exit while ESP still points into the fixture FrameBuffer. That exit calls
Windows ExitProcess without restoring NativeStack. This is a concrete owned
error-path defect; no faulting instruction pointer was captured, so the exact
fault location remains unknown. A separately versioned host will restore the
real stack before its process error exit, preserving all146 original bytes and
the frozen input cohort. No v2 execution or Rust acceptance is claimed here.

## Accepted v2 arithmetic

[MEASURED] A new source-pinned v2 changes only the owned HostFailure exit:
restore the saved real stack when nonzero before calling Windows; before the
first RunOnce its zero value leaves the current real stack in place. Root
independently reproduced, compiled and audited v2. A peer independently reviewed
both source and linked failure exits, all source/guard/section/import checks
and1,560 rational input domains. All146 original bytes and inputs are unchanged.
V2 EXE SHA256:
`1b50b88351a7b5eca1edc236decf8b0ddc773d5ea30451feed93ef09ec7503e9`.

[MEASURED] Native v2 passes19 protocols, two standalone references and all1,560
corpus cases on its first corpus attempt with58 unchanged execution pins.
Rust matches all20 modeled bytes per case under12 x87 policies, with three
hostile incoming CSR states and27 positive policy errors. Its public module
and verifier are promoted; the prepromotion packet remains archived.
Projection SHA256:
`5bed86653abd77395f3e5c22d64c0bacb940d3c0ba3d1b21ecd7708dfa51d91d`.
Acceptance: `target/source-cell/runtime-clock-native-run-v2/rust-comparison-v1.json`.
V1 failure remains unclassified and unchanged; v2 success is not a fault-IP
observation or a retry of the failed executable.

[MEASURED] A separate public composed adapter now passes40 native/Rust cases:
actual clock frames drive96 fresh full-channel source samples, then40 ordered
contribution/consumer calls. All252 bytes per case agree, with120 typed errors,
three CSR states,95 unchanged global pins and nine unchanged stage pin checks.
See `target/source-cell/runtime-clock-root-data-native-run-v1/rust-comparison-v1.json`.
Duration1/root binding/actor route remain explicit; the older reserved variant
still rejects. Loop/crossing behavior, events, actual duration setting and live
clip admission remain open.

## Accepted loop arithmetic

[MEASURED] A separate arithmetic-only gate copies157 original game bytes and
217 bytes from the installed CRT's finite SSE2 `modf` implementation. Original
IAT013CC4C0 resolves to MSVCR90.dll export785A3350, ordinal1324. The guarded
owned forwarding provider preserves its original double/out-pointer arguments
and returned80-bit value. This is original CRT execution, not a replacement
numeric helper.

All2,256 native/Rust cases match56 projected bytes, including the retained80-bit
quotient, both binary32 commits, integral64 and returned fraction80. Twelve x87
policies, three incoming MXCSR states,15 protocols and21 positive typed errors
pass. In204 cases the retained quotient differs from extending its first32-bit
commit. All42 execution pins remain unchanged; native runs use single attempts.
Projection SHA256:
`59cc9360356023bbc2a1d13510e8a08a0945eb5f45a7714591c0527fd7c6b9f4`.
Acceptance:
`target/source-cell/runtime-loop-clock-native-run-v1/rust-comparison-v1.json`.

The public API requires `CallerSuppliedArithmeticSnapshot` and
`ExplicitFiniteSse2`; both unestablished variants reject. Loop counter,
generation, event reset, actual dispatch, selected clip and duration lifetime
remain distinct gates. A source follow-up identifies constructor9A3140's
duration as the rounded32-bit product `(resource+120 - 1) * resource+11C`,
then widened64; loop+AD comes from resource+118 bit3. Serialized-resource mapping
and selected-slot integration are being traced before admitting that path.

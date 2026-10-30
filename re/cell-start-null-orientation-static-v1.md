# Cell start actor: null-orientation constructor boundary

Static Capstone x86 trace of installed `SporeApp.exe` SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. Addresses are preferred-base virtual addresses. This is conditional source-instruction evidence, not an observed spawn, original runtime execution, or a complete actor initializer. It follows [the equal-size boundary plan](cell-start-equal-size-independent-validation-plan-v1.md) and [the actor constructor trace](cell-start-actor-constructor-static-v1.md).

## Order and branch conditions

On the previously admitted `E751C0` route, `E74A50` first resolves the supplied definition and takes the equal-size sample path when argument 7 is +0. That sampling calls `935D40` once through `4DF9A0 -> 4BA080`; its state advance occurs before actor allocation. At `E74AF8..E74B14` the function then calls actor-pool allocation `B71C50` and projects its returned word through unchecked `B71D00`. It writes actor +04 byte 1, copies the supplied position to +08/+0C/+10, and zeros float +1D0. Allocation failure and pointer validity must be admitted or checked separately; a safe model must not convert zero/failure to an unchecked address. See [pool trace](cell-actor-pool-definition-handle-static-v1.md).

At `E74B41`, definition byte +AC is compared to exactly 1. The previously extracted explicit PatchData candidate has byte 0, so its supplied copy falls through. If +AC is 1 the function copies four words from globals `015A8C4C..015A8C58` to actor +14..+20 and skips the random-orientation path. Otherwise it reads the ninth argument at `E74B75`; a nonnull pointer copies four words from that pointer. The conditional player call passes null. Only when both the byte is not 1 and the orientation pointer is null does `E74B98` call `935D40` **once more**, after allocation and before matrix formation. Thus, on this specific size-sampling/null-orientation path, the orientation call consumes the next RNG state after the size draw; intervening allocation/other failures or side effects cannot be skipped when claiming full native behavior. The `935D40` zero-state RDTSC branch and live initial state remain unresolved.

## Random orientation instruction order

For the admitted null-pointer branch, `E74B9D` obtains the x87 RNG result; `E74BA2` multiplies it by binary32 `[016B4DB0]`, then `E74BAB` multiplies by binary32 `0.5` at `01471B2C`. Let the retained x87 value be `a`, preserving the actual x87 control word and operation order. `E74BB1..E74BB5` duplicates `a`, executes `FSIN`, and stores/pops the sine result to a binary32 stack slot. It loads that stored binary32 into SSE. `E74BC2` executes `FCOS` on the still-retained `a`, and `E74C05` stores/pops cosine to binary32. Each of the first three actor +14/+18/+1C words is the stored sine multiplied with the corresponding binary32 word at `015A8C40/+44/+48` using separate `MULSS` operations; +20 is the stored cosine. The installed initial `.data` words at those addresses are `00000000`, `00000000`, `3F800000` (0, 0, 1), but their runtime mutation/initialization history is not proven, and IEEE signed-zero and MXCSR handling must not be silently simplified.

`016B4DB0` resides in zero-fill storage; initializer `137E950..137E958` copies the binary32 word at `015A8BB8`, whose installed bytes are `DB0FC940` (approximately 6.28318548). This is an initializer observation, not proof the global is unchanged at the draw. RNG constants and semantics remain as in `cell_request_rng_v1`; in its admitted nonzero PC53/PC64 domain one state transition is exact, but no retail control word or actual state is supplied by the static trace.

`FSIN`/`FCOS` are original x87 transcendental instructions with hardware-dependent approximation, range-reduction/C2 and exception behavior; a generic host `sin`/`cos` or algebraic unit-quaternion simplification is not proven equivalent. For a finite, masked-exception fixture, explicitly bound `a` so each instruction performs its operation, capture C2 and the binary32 stores, specify x87 precision/rounding and MXCSR rounding, denormal and exception settings, and preserve the x87 stack. The `FSIN` result is committed to f32 **before** the three SSE multiplies, while `FCOS` receives the original retained extended-precision angle; it is not the cosine of the rounded sine or a rounded f32 angle. Exception flags and original environment remain separate evidence.

## Matrix and actor writes

`E74C10..E74C31` copies the position from actor +08..+10 to +4C..+54, ORs 4 into word +48 and increments word +4A, then passes the actor's four words at +14 to `59C000`, with a temporary 36-byte destination. `59C000..59C156` reads four f32 words `(x,y,z,w)` and computes nine f32 components using separate ordered `MULSS`, `ADDSS`, and `SUBSS`; it calls `41CAE0` to copy those components to the supplied destination. The caller copies the nine returned words to actor +5C..+7C, ORs 2 into +48 and increments +4A again. These are direct field operations; no untraced normalization or actor orientation equivalence is implied.

For understanding the array only, absent reassociation the nine values correspond to the following formulas, with each intermediate rounded by the actual sequence in `59C000`:

```text
1-2*(z*z+y*y), 2*(w*z+x*y), 2*(x*z-w*y)
2*(x*y-w*z), 1-2*(z*z+x*x), 2*(w*x+y*z)
2*(w*y+x*z), 2*(y*z-w*x), 1-2*(y*y+x*x)
```

The formulas are not a bit-exact implementation: operand order, all intermediate SSE stores, signed zeros, nonfinite cases and MXCSR are material. `014719E4` is the binary32 literal 2.0 and `014861D0` is 1.0 in this installed executable. This trace establishes a nine-word ordered output, not a world/view convention or a live pose.

## Smallest validation gate

Keep the real candidate definition and current globals source identities/hash admissions separate from supplied runtime values. A bounded fixture may start after the prior size draw with an explicitly supplied post-size RNG word, known successful allocation, +AC byte, orientation pointer case, global-angle and axis-vector bits, x87 control and MXCSR. It should report which branch ran, pre/post RNG words, sine/cosine store bits, four orientation words, nine matrix words, and the specified +48/+4A deltas. Branch tests must cover +AC exactly 1 versus other values, nonnull orientation versus null, zero-seed unsupported policy, signed-zero/masked exceptional input exclusions, and whether the random draw is skipped. Compare original-byte instructions only in a separately reviewed guarded host that preserves code/constant identity, precise stack/register/FP ABI, bounds, callback absence and process timeout. A rational or reimplemented-trigonometric oracle is a separate model comparison; it cannot authenticate `FSIN`/`FCOS` bits. The fixture does not establish native actor allocation, package winner, complete actor initialization, callback effects or gameplay.

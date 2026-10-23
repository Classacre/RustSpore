# Bounded native quaternion primitives

## Evidence and scope

[MEASURED] This dossier concerns the pinned base-game `SporeBin/SporeApp.exe`,
SHA256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
REA v3 query486 (`99C810`, Evidence
`ev_99d9bfbef1ddf8edddb2e81ac48bcbe028800cc5fac5c4b7cdf49e77f23b0005`),
query487 (`99C8D0`, Evidence
`ev_86787427c58f9c2b2f29a62c229a7c809b4104cda6c223a8edae284d5af22761`),
and query488 (`99C9A0`, Evidence
`ev_0df07164d53cfa4cfc5782a1cd00d69977592ef29e5aeb59a4d910c671b29f4e`)
contain their complete native bodies. A fresh source-byte Capstone audit gives:

| Operation | Start | End exclusive | Bytes | SHA256 |
| --- | --- | --- | ---: | --- |
| Normalize | `0099C810` | `0099C8D0` | 192 | `847bc56754311f2a49b098f41d196d1d81086858764189666ec5a31ed41ded44` |
| Hemisphere add | `0099C8D0` | `0099C993` | 195 | `394774fa15aa030c971f89fb733d6e697ea998f804c8454e7b9415a796c106b2` |
| Slerp | `0099C9A0` | `0099CB6C` | 460 | `a484b933e7953918157d8c37e3f349062edc50bb2d236c18e24bdd75ecb275d5` |

The source primitives use explicit binary32 SSE operations and, for normalize
and spherical interpolation, retained x87 intermediates. These operations do
not establish channel selection, neighbor selection, rotation-channel controls,
generated-body hierarchy, or the original process's arithmetic environment.
They do not complete an assembled source animation milestone.

## Normalize

[MEASURED] `99C810(out, input, optional_length)` loads xyzw as binary32,
preserving each component locally. In retained x87 source order it calculates
`sqrt(z*z + (y*y + (x*x + w*w)))`, then commits the length to binary32. PC and RC
apply to the retained operations; the final store obeys RC. A supplied length
pointer receives that committed value before the branch.

A zero committed length copies all four original component bits, including
signed zeros. Otherwise SSE computes `reciprocal = 1.0f / length`, then
`reciprocal*x`, `reciprocal*y`, `reciprocal*z`, and `w*reciprocal`.

The Rust bounded API returns the length and quaternion. Nonfinite inputs,
intermediates, or outputs are defensive errors. For example, a tiny finite
nonzero committed length can overflow the native SSE reciprocal and produce
nonfinite output components; `NonFiniteOutput` records that boundary rather
than claiming that the native function itself rejects or repairs the input.

## Hemisphere add

[MEASURED] `99C8D0(out, left, right)` forms the SSE dot as
`((right.x*left.x + right.y*left.y) + right.z*left.z) + right.w*left.w`.
If the dot is negative, each output is `left-right`. Otherwise each is
`right+left`. It does not normalize either input or its output.

## Slerp and finite CRT binding

[MEASURED] `99C9A0(out, left, right, amount)` uses the same ordered SSE dot,
clamps it into `[-1,+1]`, and extracts an absolute dot and hemisphere sign.
Negative zero does not take the negative branch. The source constants are
`+1` at `014861D0`, `-1` at `013EB1BC`, the four-lane sign mask at `013EB8B0`,
and the exact near-dot threshold `0x38D1B717` at `013EBCA8`.

If SSE `1-abs_dot <= threshold`, weights are SSE `1-amount` and `amount`.
Otherwise the only external call is `0099CA88 -> 011E06B8`. The unchanged
six-byte thunk jumps through source IAT slot `013CC48C` to the CRT acos entry.
The bounded fixture binds that slot explicitly to the previously reviewed
finite SSE2 intrinsic wrapper `785A35E0`. It does not emulate or claim the
original CRT dispatch decision. The wrapper commits its argument and result
to binary64, consuming and returning one x87 stack entry while preserving
the other entries.

After that wrapper, the native x87 sequence retains `theta = acos(abs_dot)`
and calculates the following source-order expressions:

1. `reciprocal = 1 / FSIN(theta)`.
2. Retained `complement = 1 - amount`.
3. Retained `first_angle = complement * theta`, then
   `FSIN(first_angle) * reciprocal`, committed to binary32 `first`.
4. Retained `second_angle = theta * amount`, then
   `reciprocal * FSIN(second_angle)`, committed to binary32 `second`.

Finally SSE multiplies the **first input's weight** by the hemisphere sign,
and computes `left[i]*first + right[i]*second`. In particular, opposite input
quaternion signs can produce the sign of the second quaternion. A generic
slerp implementation that flips the second input instead changes output bits.
The original does not normalize or require unit quaternion inputs.

[MEASURED] `native_x87::Extended::from_f64` preserves the finite binary64
operand exactly in canonical 80-bit storage, including subnormals and signed
zero. The bounded retained sine helper uses actual `FSIN` under the explicit
CW, saves/restores the calling x87 environment, admits only magnitude at most
two, and checks C2 immediately after the instruction. Its result follows the
processor's x87 sine approximation; it is not a software mathematical sine
with a claim of platform-independent transcendental bits.

## Prepared independent fixture

[MEASURED] `tools/anim_quaternion_probe.py prepare` reads the immutable source
files, audits every instruction/branch/memory operand in the bounded source
spans, and assembles/links a no-CRT x86 host. Preparation does not execute it.
The prepared fixture is under `target/animation/quaternion-native-v2/`.
The earlier unexecuted v1 fixture is retained; v2 adds an immediate guard
against writes into the twelve bytes later used for result context. Its
manifest records the complete disassembly and access/branch/import audit.
Unlisted code bytes are INT3; original constants are readonly. Seven fresh
64KiB regions change from RW initialization to RX code or R data before a
primitive can execute. The only PE imports are eight enumerated Kernel32 I/O,
memory, cache-flush, error and exit functions. No original module is loaded.

The fixed finite CRT code spans and constant window are exactly those from
the independent 62,624-row CRT conformance fixture, described in
`re/crt-ciacos-native.md`. Its source DLL SHA256 is
`ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185` and
the readonly window SHA256 is
`76210f1fef0a723d62dbcf92c1a7ce9ce1697b99f6a14530692fa8e3871f07df`.

The input stream has 86,124 deterministic rows: 18,144 normalize, 6,180
hemisphere-add and 61,800 slerp. Each case covers the twelve masked x87
PC24/53/64 by RC nearest/down/up/toward-zero combinations. SSE is explicitly
MXCSR1F80 with gradual underflow and no DAZ. Inputs include zero/sign variants,
subnormals, nonunit magnitudes, clamps, near-dot boundaries, endpoints and
bounded deterministic bit patterns. The host admits finite component magnitude
at most four and fractions in `[-0,+1]`. This proves the SSE dot finite and
the clamped absolute dot admissible for the finite CRT wrapper. Its theta
and all three subsequent sine arguments have magnitude less than two.

Each row executes twice with zero/all-one XMM scratch, comparing all output
and optional-length bits. The host checks cdecl ESP restoration, empty x87
TOP, unchanged requested CW and MXCSR control bits, slerp C2 cleared and both
reply canaries. The twelve bytes following the result/length region are also
sentinels checked immediately after each call before any context write can
overwrite them. Optional NULL length output is covered separately. Finite tiny
normalizations with nonfinite native results remain raw in the native output;
the comparison records them as explicit typed-error rows, separate from exact
finite-result conformance.

The separate pinned runner `tools/anim_quaternion_run.py` checks the reviewed
host/source/data hashes and can run hidden binary pipes plus malformed-input
protocol checks. `anim_quaternion_verify.rs` evaluates only Rust against the
same external inputs. `anim_quaternion_probe.py compare` compares already
produced outputs and never executes native code.

[MEASURED] Root independently reviewed both complete v1 host/branch/access
audits and the v2 immediate-sentinel correction before authorizing execution.
Only v2 executed. Its assembly SHA256 is
`54f0fea6efbb77c3a1c778857ebd6737c2dd52f16dbb36315f4cd15db4aedca7`,
host PE SHA256 is
`7621203e6f41906c35febd00b9b24ad508417d8299561621b11c4c59d9317909`,
and input SHA256 is
`af3b78c5de8c0e1444bf5bb0e7c71b731293ec6de91fd6646afbbb4fa2e83a2d`.

The hidden original-byte run passed all 86,124 rows, each with two XMM scratch
patterns, plus 114 EOF/partial-record/domain/protocol checks. Each of the
twelve CW contexts has 7,177 rows. All ESP/TOP/CW/MXCSR/C2, immediate context
sentinel, final canary, hash and source-immutability checks passed. There are
85,932 finite-result rows and 192 nonfinite native normalization rows. The
raw replies SHA256 is
`0f7002fc6f191c948aa9e2ef757f8631d9887b8897bd3d72d01c6178c9eb1dc9`.
`target/animation/quaternion-native-v2-run/run.json` and the corresponding
raw binary/protocol files retain the complete run, including state counts.

[MEASURED] `anim_quaternion_probe.py audit-rational` provides a third,
independent integer/Fraction oracle. Source-order PC/RC square accumulation,
integer square-root rounding and binary32 commit, followed by exact integer
SSE reciprocal/products, match all 17,952 finite normalize rows. It also
identifies the same 192 reciprocal-overflow boundaries. Exact integer SSE
matches all 6,180 hemisphere-add and 36,240 near-dot slerp rows. Thus 60,372
finite rows and 192 boundary classifications are independently checked, with
zero problems. The 25,560 spherical rows are explicitly classified without
evaluating their FSIN-dependent outputs in this rational oracle. The report
`target/animation/quaternion-native-v2-run/rational-crosscheck-v2.json`
separates those counts. Spherical inputs cover the finite CRT tiny, small,
table, sqrt-table and near-one reductions. The exact-one CRT case is bypassed
by the source near-dot branch.

[MEASURED] The exported Rust module passed all five focused tests, including
restoration of the caller's complete MXCSR after both success and error. The
fresh Rust verifier and binary comparison match all 85,932 finite-result rows
and the 192 explicit nonfinite normalization boundaries, with zero problems.
Both normalized result streams have SHA256
`36a6e81ba5d1d9c4ae1f207d5aeacf10719a046082c6ddf457d8036d606b761e`;
`target/animation/quaternion-native-v2-run/comparison.json` retains the result.
The exported-cohort regression passed 511 tests (447 core, 44 binary, 20
archive), with two deliberate corpus ignores and no failures.

[UNKNOWN] The full retail floating-point policy remains distinct from the
explicit fixture policy; see
`re/native-float-render-context.md` for measured source precision/rounding,
FTZ/DAZ writes and unresolved thread/callback ordering.

## Separate source-curve extrapolation domain

[MEASURED] Native type2 rotation curves call `99C9A0` with tangent fraction
`-1`, and interpolation fractions `t+1` and `t-1`. These calls require a
separate `[-1,2]` entry when the selected interpolation fraction is in `[0,1]`.
`slerp_extrapolated` therefore preserves the unit-interval API and uses the
same original arithmetic with an explicit wider admission. Its retained sine
arguments are bounded by pi, strictly below four; the new bounded-four x87
sine entry uses the same FSIN/status/environment block as the accepted
bounded-two entry. The runtime still selects an explicit CW and MXCSR policy.

The separately reviewed host under
`target/animation/quaternion-extrapolate-native-v1/` differs from accepted
quaternion-v2 only in its owned signed-integer fraction admission. Its unchanged
source/CRT bytes, thunk, constants, permissions and immediate guards were
reviewed again before execution. The fresh run2 passed all 86,520 finite rows
(14 fractions by 515 quaternion pairs by 12 CW contexts), two XMM scratch
patterns each, and 108 protocol checks. No retail process, original DLL load
or GUI was used; all original and fixture hashes remained unchanged.

Fresh Rust verification matches every wider-domain row exactly, with result
SHA256 `896c5de5168759ee7d017aa78d29b8e4f58eb26c61f0e1d85bc32bf0075f5a0a`.
`target/animation/quaternion-extrapolate-native-v1-run2/comparison.json`
retains the comparison. The exported helper regression passed 512 tests with
two deliberate corpus ignores.

[UNKNOWN] The first wider-domain smoke attempt captured an empty reply and
failed its wrapper assertion. That initial wrapper did not retain the exit
code, so its cause remains unknown. Identical pinned host/input bytes passed
a separately retained diagnostic and the complete fresh run2. The failed
attempt remains under `quaternion-extrapolate-native-v1-run/`; the corrected
runner records exit code, stdout and stderr before every assertion. No
fixed-address allocation explanation is claimed without evidence.

## Finite source inputs with overflow/unordered dots

[MEASURED] Finite source components can exceed the earlier primitive fixture's
magnitude-four domain. Their scalar SSE products may overflow, and mixed
positive/negative overflow can make the ordered dot sum NaN. This changes the
comparison flags without introducing a nonfinite source input.

Source `99C8D0` compares zero,dot using COMISS at `99C936`, then JBE at
`99C939`. An unordered dot takes the same right+left path as a nonnegative
dot. Only an ordered negative dot, including negative infinity, takes
left-right. Source `99C9A0` compares raw,-1 at `99CA1B`; JA at `99CA41`
admits only an ordered raw dot above -1. Unordered therefore falls through
to -1 at `99CA47`. The ordered path compares raw,+1 at `99CAE1`, keeping
raw at most +1 and clamping positive infinity to +1. Its exact predicate is:

```text
if raw > -1:
    if raw <= +1: keep raw
    else:        use +1
else:            use -1
```

Consequently NaN and negative infinity become -1, positive infinity becomes
+1, and signed zero remains unchanged. All overflow/unordered dots select
the native near-dot branch before any CRT call. The first input's weight
still receives the resulting hemisphere sign.

Separate `hemisphere_add_source`, `slerp_source` and
`slerp_extrapolated_source` preserve these branches for finite vector inputs.
The earlier bounded entries retain `NonFiniteDot` errors. The source entries
retain input, output, fraction, x87 and explicit CRT/state bounds; they do not
accept arbitrary nonfinite vector components or repair output bits.

[MEASURED] The fresh `rotation-source-dot-inputs-v1` cohort has 3,456 contexts,
including 1,152 each with primary positive-infinity, negative-infinity and
unordered dots. Exact finite components are `2^93`, within the measured
source magnitude, and each product's magnitude is `2^186`. Mixed signs prove
the unordered sum independently of Python floating-point arithmetic. The
unchanged reviewed ROT host exercises both blend policies, all 16 control
mode pairs, three interior frames and all twelve CWs. After separate root
input/runner review and independent source/host audit, its hidden execution
passed all 3,456 contexts and eleven protocol checks. Every native result was
finite. Rust matches all 3,456 projections exactly, with zero exclusions or
problems and projection SHA256
`aae34f7c2057c95d9595c211216c8bcf33c66ec17488799ff5f59a8764b9c5c2`;
`target/animation/rotation-source-dot-native-v1-run/comparison-v1.json`
retains the result. The complete authored ROT baseline also now matches all
30,240 contexts with zero exclusions, including the earlier 144 finite native
outcomes excluded by the old bounded API. See
`re/anim-rotation-curve-native.md` for its complete scope and separate source
row census. Original bounded entry errors/behavior remain preserved.

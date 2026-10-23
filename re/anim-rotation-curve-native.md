# Authored native type2 rotation curves

## Original source and row layout

[MEASURED] REA v3 query424 analyzes complete `99F6A0..9A0973`, Evidence
`ev_c81bba6281ecb6cd79aff2c54efd3699e1d74b4e1bf199b2da511543ff10560d`.
Source executable SHA256 is
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The type2 branch enters `99FAF7`, writes quaternion at output+100..115 and
weight at output+116, and leaves through the common component loop at
`9A0890`. It does not call a generic quaternion spline library.

Native endpoint copy helper `99F630` uses the type-size table at `15514CC`:
type0 INFO is 20 bytes, type1 POS is 80, type2 ROT is 100. The native slot2
destination table at `1447818` is output offset100. The independently retained
ROT component row therefore has this measured subset of semantics:

| ROT-relative bytes | Interpretation in this evaluator |
| --- | --- |
| `+00..0F` | quaternion xyzw, preserved as supplied |
| `+10..13` | weight binary32 |
| `+1C` low two bits | quaternion incoming/previous control mode |
| `+1D` low two bits | quaternion outgoing/next control mode |
| `+54..57` | weight incoming/previous slope binary32 |
| `+58..5B` | weight outgoing/next slope binary32 |
| `+5C` low two bits | weight incoming/previous tangent mode |
| `+5D` low two bits | weight outgoing/next tangent mode |
| Other bytes through `+63` | retained raw; not assigned extra quaternion slope semantics |

In particular, the floats at `+14/+18` are not consumed as quaternion slopes
by this branch. The native 100-byte endpoint copy preserves them even if their
bit patterns are nonfinite; the evaluator does not validate unused floats.

## Selection, enable and endpoints

[MEASURED] The common `99C6C0` neighbor helper is unchanged from the accepted
position curve path. Header word280 mask8 controls previous/following wrapping,
separately from any caller playback-loop policy. The caller upper key is the
first source key strictly greater than the frame. Thus duplicate integer
frames select their final equal key; before-first and after-last cases become
endpoint copies. Neighbors are previous, left, right, following.

When left equals right, the original helper copies recognized slot2 data even
when the descriptor's interpolation enable mask10 is clear. For distinct left
and right, only enabled type2 descriptors enter the rotation branch. This
preserves the source difference between disabling interpolation and removing
endpoint data. Endpoints copy raw quaternion and weight without normalization
or weight clamping. Interpolated weight is clamped into `[0,1]`.

The common frame fraction is retained x87 `(frame - unsigned(left_frame)) /
unsigned(right_frame-left_frame)`, committed once to binary32. Unsigned FILD
conversion and wrapped spans use the original conditional add2^32. Cubic basis
and weight arithmetic reuse the measured position implementation's literal
SSE/x87 order; neither a generic Hermite shortcut nor new time clamping is added.

## Quaternion controls and runtime policy

[MEASURED] Let P,L,R,F be the four source quaternions. The left outgoing mode
selects the first control: mode1 uses source `slerp(L,R,-1)`, mode2 uses R,
and modes0/3 use P. The right incoming mode selects the second control:
mode1 uses `slerp(R,L,-1)`, mode2 uses L, and modes0/3 use F. Mode3 does not
consume an authored quaternion scalar slope here.

At `99FD12` global `15514F8` selects two complete native branches. Its pinned
image word is one. This establishes initial bytes, not the current runtime
producer/history; `RotationBlendPolicy` supplies the branch explicitly.

The normalized-hemisphere branch forms `u=t+1` using x87 FADD and a binary32
store at `99FD26..99FD46`. The spherical branch instead uses scalar SSE ADDSS
at `9A0173..9A0181`. This preserves their different rounding policies when
x87 RC differs from MXCSR RC. `t-1`, `u/2` and `t/2` are separately rounded
scalar SSE operations. With the selected controls A and B:

| Stage | Spherical policy, global zero | Normalized-hemisphere policy, global nonzero |
| --- | --- | --- |
| Q0 | `slerp(A,L,u)` | same source slerp |
| Q1 | `slerp(L,R,t)` | normalized hemisphere blend of L,R at t |
| Q2 | `slerp(R,B,t-1)` | same source slerp |
| Q3 | `slerp(Q0,Q1,u/2)` | normalized hemisphere blend at u/2 |
| Q4 | `slerp(Q1,Q2,t/2)` | normalized hemisphere blend at t/2 |
| Result | `slerp(Q3,Q4,t)` | normalized hemisphere blend at t |

Every normalized hemisphere blend first scales its left vector by SSE
`1-amount` and right vector by amount, calls source `99C8D0` on those scaled
vectors, then calls source `99C810`. The source branch has four such sums and
four normalizations. The spherical branch has six slerps after control
construction. Source inputs are not normalized or quality-filtered first.

The `[-1,2]` source extrapolation primitive and its explicit four-radian FSIN
bound are independently accepted over 86,520 original-byte/Rust cases; see
`re/anim-quaternion-native.md`. CW, MXCSR and CRT dispatch remain explicit.

## Fresh installed source denominator

[MEASURED] `anim_rotation_curve_census.py` freshly opened all 82 package
indexes and examined all 5,022 ANIM resource copies and 48,700 channels. There
are 39,915 ROT descriptors/channels, 232 empty rotation channels, and 731,383
complete 100-byte source rotation rows. Every row's quaternion, weight and
both weight slopes is finite; no quaternion is all zero, and all weights are
in `[0,1]`. Source control bytes include all four quaternion modes and all
four weight modes. The raw fields and unsupported bytes remain retained.

Source inputs include 1,050 quaternions outside the historical norm tolerance
0.01 and maximum component magnitude approximately `1.9832258e28`. These are
retained literal source facts, not a reason to normalize or delete records.
Finite large inputs can overflow an ordered SSE dot. The earlier bounded
primitive entries retain their typed `NonFiniteDot` error, while the authored
rotation evaluator now uses separate source-compatible entries. Those preserve
the original ordered/unordered comparisons: hemisphere addition sums an
unordered dot, and slerp clamps unordered to -1 before its near-dot branch.
Positive and negative infinity clamp to +1 and -1 respectively. This admits
the literal large source inputs without normalization or deletion; nonfinite
vector inputs/outputs remain explicit API boundaries.

The 5,022 fresh resource input digests have aggregate SHA256
`e79060199eddbb367b1e429411e38a74fa4cafc08caa2ac87f75eeedb6f408e0`.
`target/animation/rotation-source-census-v1/census.json` retains all counts,
raw mode distributions, per-resource digests and zero framing problems.

The subsequent canonical package-index correction independently reread all
158,720 payloads and preserved every ANIM type, scoped ID and payload span.
The same 5,022 ANIM copies remain in BoosterPack_01 (25), PatchData (2,223),
Spore_Game (2,091), and Spore_EP1_Data (683). The retained correlation is
`target/native-archive-index-census-v1/compatibility-correlation-v1.json`;
the compact/common-header correction therefore does not change this census.

## Prepared whole-branch host

[MEASURED] `target/animation/rotation-native-v2/` contains the newly assembled,
unexecuted no-CRT host. It copies unchanged common/type2/tail/neighbor/copy
helper instructions, unchanged accepted quaternion primitives and the unchanged
finite CRT spans/window. Unlisted POS/deformation/CRT-dispatch paths are INT3.
Two immutable descriptor arrays choose INFO0 plus ROT12 or ROT02, with fixed
stride120, offsets0/20 and slots0/2. This excludes POS/deformation dispatch.

The only owned source-address shim is checked memcpy at `11E0534`: exact
INFO20 to output0 or ROT100 to output100 from a checked active aligned row.
Its relative transfers are internal; failure jumps absolutely to the owned
exit. The original acos thunk remains unchanged and its fixture IAT binds to
the accepted finite SSE2 intrinsic wrapper.

Inputs admit 0..16 sorted integer keys at most2^24, finite frame magnitude at
most2^24, duration1..2^24 with duration-1 covering the final key, twelve masked
CW combinations, explicit blend0/1 and enabled0/1, and header flags0/8. Used
quaternion/weight/slope float fields must be finite; unused controls are not
filtered. Exact integer keys make the owned upper-key check exact across PCs
and RCs. Interpolation t is in `[-0,1]`; all source slerp fractions are in
`[-1,2]`. Native dot overflow/unordered is clamped by the original branch to
an admissible absolute dot before any finite CRT call. Thus its sine arguments
remain bounded by pi, below four, even when native output becomes nonfinite.

Every code/data region begins RW and becomes RX/R before source execution.
The first valid request fixes one blend policy, then the global15514F8 page
becomes R; subsequent requests cannot change it. Each policy runs in a fresh
hidden host process. The guarded native 200-byte output is separate from
reply/context storage, avoiding masked overruns. Every row runs twice with
zero/all-one XMM scratch, comparing the entire native output and source
index metadata. Canary, ESP, TOP, CW and MXCSR control checks are explicit.
Fixed-address allocation failure has its own exit10.

The host assembly SHA256 is
`b914ed8a7ff6cd972b60cfc3733628489895ec9a2e538a79988547f27f8e84c6`;
host PE SHA256 is
`972bf058d3cf7711b2e9f0ea5ff63c03d5e4e3f1a7a6dafa7fa17653e9d27f18`.
Its complete owned/source disassembly and branch/import/memory audit are in
the manifest. Initial v1 failed assembly and never executed.

The separate `rotation-inputs-v1` cohort has 30,240 cases: 25,632 synthetic
contexts and 4,608 contexts containing literal retained source ROT100 windows.
It covers both policies, wrapping, enable states, twelve CWs, mode pairs,
empty/single/duplicate/16-key channels, endpoints, nonunit/zero/large finite
inputs and nonfinite unused controls. Its input SHA256 is
`4c4c90b6745f7dcc30d598324671ee067afb563311b5d193b4940a5b4ff450b4`.
The source windows and their explicit contexts are not an entire runtime
animation corpus or a substitute for the 731,383-row source denominator.

An independent exact-rational audit of all 30,240 prepared requests covers
12,096 enabled interpolations, 18,048 copy/disabled contexts and 96 empty
contexts. Every source slerp fraction is in `[-1,2]` across all 24 blend/CW
contexts. In 492 interpolated contexts, x87 `t+1` differs from SSE `t+1`, so
the fixture exercises the policy-specific rounding above. This audit is
retained at `target/animation/rotation-fraction-domain-audit-v1.json` and
executes no original instructions.

[MEASURED] After independent peer and root reviews, the pinned v2 host ran
all 30,240 requests hidden with binary pipes. All 25 malformed-protocol/domain
checks, zero/all-one XMM comparisons, guards, ESP, FP state and source/fixture
immutability checks passed. It produced 96 empty cases, 5,952 raw endpoint
copies, 12,096 disabled interpolations and 12,096 finite interpolations.
Large finite native outcomes remain retained even where the bounded Rust
primitive may exclude them. Raw native output SHA256 is
`5405b02ee87683a02257fe2b5a63c2c9b775ed8d453d3e8de4fcf40cd12a6125`.
Run evidence is under `target/animation/rotation-native-v2-run/`. The game,
GUI and original modules were not started or loaded.

[MEASURED] Fresh Rust verification now matches all 30,240 native projections
exactly, with zero arithmetic exclusions or other problems. Native and Rust
projection SHA256 is
`e991e7b2c2f0246a8753fbe5ef152ea014fda6569dea4543b38ac6ea31dde373`.
The strict result is `rotation-native-v2-run/comparison-source-v2.json`.
The earlier `comparison-bounded-api-v1.json` remains unchanged: its 30,096
exact rows and 144 finite native outcomes excluded by the old API document
the superseded evaluator limitation rather than source failure.

### Separate overflow and unordered-dot acceptance

[MEASURED] The additional `rotation-source-dot-inputs-v1` cohort contains
3,456 contexts: 1,152 each with primary positive-infinity, negative-infinity
and unordered dot products. Every supplied component is finite with magnitude
exactly `2^93`, below the measured source maximum; every scalar product has
magnitude `2^186`, proving overflow by integer exponent arithmetic. Mixed
product signs independently prove the unordered sum. Both runtime policies,
all sixteen quaternion control-mode pairs, three interior frames and twelve
CWs are represented, with wrap0 and enabled1 supplied explicitly.

Root reviewed the fresh input manifest/runner, reproduced its exact inputs,
and independently reaudited the unchanged original/source/CRT/host fixture.
The hidden run passed all 3,456 contexts and eleven protocol checks. Every
native interpolation was finite, including the unordered-dot cases. Raw
native output SHA256 is
`63a7bfb5b571e5699df671a3faff5b7db6d6d3962928e440f49e3e8bee4cfabe`.
Fresh Rust strict comparison matches every row with zero exclusions/problems;
both projection streams have SHA256
`aae34f7c2057c95d9595c211216c8bcf33c66ec17488799ff5f59a8764b9c5c2`.
Evidence is under `target/animation/rotation-source-dot-native-v1-run/`,
including `comparison-v1.json`. The old baseline input/output files remain
unchanged.

The comparison utility forbids arithmetic exclusions by default. Reproducing
the historical bounded-API result requires the explicit
`--allow-large-finite-exclusions` option. Raw 100-byte endpoints, source INFO,
neighbor/index metadata and untouched output regions are checked separately
from the Rust projection. An independent exact-rational PC/RC oracle checks
each reported interpolation fraction.

[UNKNOWN] These 33,696 exact contexts establish the bounded authored ROT100
operation under explicit descriptor, blend, CW, MXCSR and CRT policies.
They do not recover selector mapping, generated hierarchy, channel-to-runtime
pose conversion, assembled source creature animation or a full-roadmap
milestone. Runtime policy producers/history remain explicit caller inputs.

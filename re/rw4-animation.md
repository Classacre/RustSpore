# RW4 skeletons, skinning and keyed transforms

The implementation is `crates/spore-core/src/rw4_animation.rs`. It reads user-supplied
RW4 data, exposes typed joints and key values, and supplies an explicit RustSpore
interpolation policy. No executable pseudocode or mod-tool implementation was used.

## Verification scope

`[MEASURED]` On 2026-10-22 the Rust verifier opened all 82 packages under
`D:\Games\SPORE Collection` and parsed all 13,498 RW4 resources. Their animation
objects contain 9,966 skeletons, 26,061 bone records, 3,053 skin bindings, 19,145
inverse-bind matrices and 7,803 keyed clips. All 19,145 inverse-bind matrices are
invertible. Every corresponding 12-word `SkinMatrixBuffer` slot is identity.

`[MEASURED]` All 51,102 channels have one of the three supported layouts:

| Kind | Stride | Channels | Exposed values |
| --- | ---: | ---: | --- |
| `0x100` | 8 | 4,136 | scalar, time |
| `0x101` | 32 | 20,405 | quaternion XYZW, translation XYZ, time |
| `0x601` | 48 | 26,561 | quaternion XYZW, translation XYZ, scale XYZ, unused word, time |

`[MEASURED]` The parser exposes 13,107 scalar keys and 571,070 transform keys.
Sampling every clip at zero, half duration and duration produced 23,409 successful
sample calls with finite outputs. There were no parse or sample errors and no
unsupported channel kinds in this installed corpus. This result does not establish
the interpolation used by the original executable.

`[MEASURED]` Per-vertex skinning succeeds for 4,849 of 4,853 binding/mesh pairs,
covering 5,007,037 decoded vertices in 2,458 resources containing both meshes and
bindings. Four pairs lack declared joint indices. Those cases return explicit
errors. All four use a position/UV/normal/tangent vertex declaration in resource
`4060600001DCAF31` (group `00010000`); their attachment behavior remains unknown.
The `SHORT2` case in `406060009343E658` is supported: all 98 vertices have one
unit weight and first index values from 0 through 21 in increments of three, with
a zero second index. These are counts of
binding/mesh pairs, rather than a claim that every game model has a usable skeleton.

## Object layouts

All offsets are little-endian. Header pointers below are absolute file offsets;
the per-channel offsets in a keyed clip are relative to that object's start.

`[MEASURED]` `Skeleton` (`0x70002`) has a 24-byte header followed by three arrays of
N 32-bit words. Header +0 points to flags, +4 to signed parent indices and +8 to
bone IDs; +12 is N, +16 is the skeleton ID and +20 is a retained flags word. Parent
`-1` denotes a root. All observed parents precede their children; the Rust parser
also accepts valid forward references and rejects cycles and out-of-range parents.
No skeleton in the corpus has duplicate bone IDs. The meanings of individual flags
and the source strings behind the IDs remain `[UNKNOWN]`.

`[MEASURED]` `AnimationSkin` (`0x70003`) has a 16-byte header: data pointer at +0,
N at +4, then two zero words. N records contain 16 floats each. The basis and
translation occupy four padded columns. For raw words `v`, the exposed affine rows
are `[v0,v4,v8,v12]`, `[v1,v5,v9,v13]`, `[v2,v6,v10,v14]`. Padding is retained as raw
columns, and is not used as a homogeneous fourth row.

`[INFERRED]` These matrices convert model-space points into joint bind space. This
interpretation is supported by the hierarchy cancellation experiment below,
including 2,096 cases where local composition yields identity and treating the same
keys as global poses does not.

`[MEASURED]` `SkinMatrixBuffer` (`0x7000F`) has the same 16-byte header shape and N
12-float slots. All serialized slots in this install are identity. `[INFERRED]` The
12 words represent three affine rows for runtime skinning. Identity matrices alone
cannot distinguish alternative row/column storage conventions; callers should use
`AnimationSkin` for source inverse-bind data.

`[MEASURED]` `SkinsInK` (`0x7000C`) is 20 bytes. Its section references at +8, +12 and
+16 target `SkinMatrixBuffer`, `Skeleton` and `AnimationSkin`, respectively. The
parser verifies reference types and equal joint counts. The first two words remain
opaque.

`[MEASURED]` `KeyframeAnim` (`0x70001`) has a 48-byte header. +0 points to channel
IDs; +4 is their count; +8 is clip ID; +16/+20 delimit the allocated value region;
+24 repeats channel count; +32 is duration seconds; +44 points to descriptors.
Each descriptor is 12 bytes: object-relative value offset, record stride and kind.
Header +36 is 12 in all observed clips; +40 is retained flags (1 or 3 observed).

`[MEASURED]` A critical allocation detail prevents extra zero records from being
mistaken for animation keys. For homogeneous `0x100` clips the value region contains
only key data. For `0x101` it contains key data followed by an equal-sized zero
region; for `0x601` the zero region is twice the key-data size. The Rust parser
derives the final channel's extent with these factors, validates the full reserve
as zero and reports `reserved_bytes`. The full install contains 37,210,240 such
zero bytes. The reserve's original runtime purpose remains `[UNKNOWN]`.

`[MEASURED]` Unknown channel kinds remain opaque and cannot be sampled as transforms.
Known keys preserve source times. The explicit RustSpore policy clamps outside the
first/last key, uses linear scalar/translation/scale interpolation and shortest-arc
quaternion slerp. Looping is caller-selected. `[UNKNOWN]` Original runtime curve
evaluation, tangent generation, event timing and blend policies are not recovered.

## Local versus global pose experiment

`[MEASURED]` The verifier matches transform channels to all bone IDs of each bound
skeleton. It compares the maximum absolute component error from identity after
multiplying the first stored pose by the inverse-bind matrix. One interpretation
uses each track directly as a global pose; the other first composes parent and
child local transforms. There are 6,644 complete matches, of which 6,562 have a
nontrivial parent hierarchy. Scalar and unrelated/partial clips are excluded from
this comparison.

| Result over 6,562 comparisons | Count |
| --- | ---: |
| Direct pose times inverse bind is identity within `1e-3` | 232 |
| Parent-composed pose times inverse bind is identity within `1e-3` | 2,328 |
| Composition alone reaches identity | 2,096 |
| Direct interpretation alone reaches identity | 0 |
| Composition has lower maximum error (margin `1e-4`) | 5,508 |
| Direct interpretation has lower maximum error | 315 |
| Tied within the margin | 739 |

`[INFERRED]` Keyed transforms are local to the stored parent hierarchy. A clip's
first pose need not equal its bind pose, so the 4,234 composition results that are
not identity do not establish an error. Median maximum error is 0.41986787 after
composition versus 1.15599966 directly. These measurements establish the working
convention, not full animation fidelity.

`[MEASURED]` All 6,644 clips whose transform-channel IDs belong uniquely to the
bound skeleton also cover every bone exactly once and have the same clip ID as
the skeleton ID. There are no different-ID compatible pairs in this corpus. The
viewer can therefore select clips using ID equality plus exact unique bone-ID
coverage, rather than accepting a clip that merely shares one joint.

## Joint indices and weights

`[MEASURED]` `BlendShapeBuffer` arrays 9 and 10 carry joint-register indices and
weights. Header +60 gives influence count, observed from 1 through 4. Joint indices
are unsigned 16-bit values, packed per vertex with their stride rounded up to four
bytes. Weights are influence-count consecutive float32 values per vertex. Active
indices are divisible by three; dividing by three gives a valid joint index in
every supported binding/mesh pair. Zero-weight index slots are ignored.

`[INFERRED]` The factor of three refers to three shader registers per affine joint
matrix. The decoder retains source weights, validates finite normalized sums and
bounds, and does not silently normalize or invent a residual weight. Conventional
vertex-buffer declarations read raw `UBYTE4`, `SHORT2` or four-short joint storage directly,
avoiding the general mesh helper's byte normalization.

## Reproducible candidate

`[MEASURED]` RW4 `406060007A070007` in `Data\Spore_Graphics.package` has one binding
at section 14, targeting skeleton section 5 and inverse-bind section 12. Its two
bones have IDs `D0BE09E0` and `D0BE09E3`, with parents `[-1,0]`. Mesh section 17
contains 716 vertices and 676 triangles; 536 vertices use joint 0 and 180 use joint
1, with all weights equal to one. Source geometry is in BlendShapeBuffer section
20. This model's scalar morph channels are separate from its skeletal channels.

`[MEASURED]` Transform clip section 1 (`FDB981D0`) has five keys per joint at
0, 0.33333334, 0.625, 0.95833337 and 1.25 seconds. At its first pose, maximum
identity error is 1.00000060 when tracks are treated as global, and 0.00000060
after parent composition. `[INFERRED]` This is a useful matrix-convention test,
but a poor visible deformation test: the child is a flat internal rotor turning
about Y inside an opaque outer disk. A large point displacement can remain fully
occluded. The initial identical screenshots were therefore not evidence of a GPU
skinning failure.

`[MEASURED]` The exposed deformation candidate is `40606100AD73C528` in the same
package: 632 vertices, 1,064 triangles and three bones, with clip `CB96BD72`.
At 0.25 seconds, the application reports bind-pose error `1.5e-8` and maximum point
displacement `0.073260702`. Bind and posed renders differ in 69,023 pixels above a
two-level RGB threshold. A GPU-skinned pose and independently CPU-deformed pose
differ above that threshold in only one of 1,958,400 region pixels (maximum channel
difference 18, at a raster edge). Local evidence is
`target/animation/rtx-render-agreement.json`. These checks establish usable pose
playback and GPU deformation; source materials, morph evaluation and original
runtime interpolation remain outside this acceptance result.

## Commands and local artifacts

```powershell
tools/build.bat test -p spore-core rw4_animation
tools/build.bat run -p spore-bevy --example rw4_animation_verify -- --bone-inventory target/animation/rw4-bones.tsv --digests target/animation/rw4-animation-digests.tsv "D:\Games\SPORE Collection"
C:/Python311/python.exe tools/rw4_animation_probe.py --compare-digests target/animation/rw4-animation-digests.tsv --out target/animation/rw4-animation-digest-comparison.json
```

`[MEASURED]` Ten focused synthetic tests pass, including malformed object bounds,
cyclic parents, exact transform-track times, allocation-reserve corruption,
quaternion interpolation, hierarchy composition and raw register-index decoding.
Raw resource identifiers, per-object canonical digests and bone inventories stay
under ignored `target/animation/`; no EA assets are added to the repository.

`[MEASURED]` The final independent Python comparison reconstructs canonical typed
word streams for every resource containing these objects: 3,294 resources and
28,724 object/influence records, with zero FNV-1a-64 mismatches. This includes both
raw inverse-bind words and converted affine rows, skeleton IDs/flags/parents, every
exposed key value/time, matrix slots and all 4,849 supported influence arrays.
The Rust corpus run covers the other RW4 resources as well, including resources
with no animation objects. Comparison output is
`target/animation/rw4-animation-digest-comparison.json`.

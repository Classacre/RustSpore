# Conditional grid to surface-position join

Static trace of pinned `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
This does not assert that retained 0473 rows were live-selected.

On the admitted `414270` route, successful `4F9D60` is followed by `5402E0`
with intermediate +38 query, intermediate +34 geometry and the current step.
`5402E0` retains these as surface worker +90 and +94, and stores the step at
+14. `53EEE0` obtains the query's aligned lower bound through `4F9CA0` and
copies it to worker +08. That lower bound is separate from the padded query
origin at query +50. `53ED50` reads a packed word's bytes 2, 1 and 0 as x, y,
z, respectively; it converts each unsigned byte to binary32, multiplies by
worker +14, then adds worker +08 using the separately called vector helpers.
The packed high byte is not read by this transform.

`creature_skin_lattice_world_v1` models this finite coordinate transform for a
caller-supplied aligned origin, positive accepted step and scoped MXCSR 1F80.
It bounds invalid/nonfinite behavior more strictly and does not select a packed
point, trace callbacks or establish an isosurface. Its two focused tests pass.
A reviewed isolated host executed the original `53ED50`, `41DCE0` and
`41DAB0` spans for four fixed synthetic cases. All 12 coordinate words matched
the Rust model. The packed words/origin/step inputs were: `00000000`,
`[0,0,0]`, 1; `FF030201`, `[-1,2,3]`, 0.5; `00FFFFFF`, `[-20,0.125,2]`,
0.035; and `7F102030`, `[-0.25,1.25,-2.5]`, 0.125. The receipt
`target/creature-skin-lattice-native-20261108/result.json` has SHA-256
`3ea3c93b89823fbb2521fab34f694e8ac1c23475e853ebaaad0338111678dcc26`.
The host had output canaries but no guard pages or sticky-flag comparison;
endpoint hashes do not establish continuous stability or complete build
provenance. These synthetic coordinates do not establish isosurface positions,
accepted source step or runtime identity.

The surface route uses a **signed scalar field**, distinct from the existing
positive influence-request kernel. `53E8D0` seeds work by converting packed
lattice points and calling the function pointer at query +08; `53EFE0` walks a
mutable work queue and `540070` uses classification/edge tables to request
crossing vertices. `53F540` caches field evaluations by packed lattice point.
The query constructor selects field implementations at +08 from `4F9440`,
`4F9760` or `4F9840` according to an earlier capability check. The scalar
`4F9440` begins at negative query threshold and walks addressed packed records,
calling `4F94E0` for contributions; the SIMD variants have their own address
and accumulation instructions. Do not substitute a sum of the existing
positive-only skin requests without proving exact FP and address behavior.

`53FBA0` caches an ordered edge identity, obtains endpoint field samples,
evaluates a midpoint and invokes `4FC7C0` to bisect a signed-field boundary.
The resulting crossing position is appended to the geometry object's +08
12-byte position vector or accumulated into an existing position. On a
successful completion path, `53F720` averages accumulated positions and can
perform further correction/normal work. Thus raw lattice positions, initial
crossings and final geometry positions are different stages. Table contents,
queue/cache ownership, signed-field controls, edge bisection and finalization
must be traced and validated before generating actual 0473 vertices.

Separately, `4CC060` uses the intermediate's +34 geometry and +38 query and
passes them with its remap to `52B700`, which reads geometry +08 positions and
queries influences per position. A connection to the surface output requires
the same +34 object and +38 query, successful callbacks, stable data and
lifetimes, and no intervening replacement. The export selects influence records
by geometry position index; mapping their bytes to a canonical animation
palette additionally requires the identity and constructor conditions in
`creature-generated-influence-palette-identity-static-v1.md`. Numeric equality
or a pinned source payload alone is insufficient.

A practical next component after this coordinate slice is the selected
signed-field evaluator for explicit packed-grid snapshots, with separate
native comparisons for its scalar and SIMD dispatch/FP policies. Only then can
an independently tested edge/bisection and queue model supply positions to the
existing one-point skin fixture. Reproduce with PE-mapped instructions in
`5402E0..54044A`, `53EEE0..53EF1F`, `53ED50..53EDD8`,
`53E8D0..53ECDD`, `53EFE0..53F539`, `53F540..53F6F7`,
`53FBA0..54006D`, `4FC7C0..4FC9D1`, `53F720..53FB95`,
`4F9440..4F9624`, `4CC060..4CC4B9` and `52B700..52C3DF`. Supporting
read-only Ghidra logs are `target/ghidra-creature-entry-v1/skin-surface-position-trace-v1.log`,
`skin-surface-field-trace-v1.log`, and `skin-surface-origin-interp-v1.log`;
instruction decoding controls the bounded claims.

## Scalar signed field component

`creature_skin_scalar_field_v1` implements the scalar `4F9440/4F94E0`
field for a supplied stable grid snapshot, point, threshold and explicit SSE
and x87 controls. It starts with the threshold's sign bit flipped. Within each
selected 0x80 record it visits four lanes in order, skips zero cutoff,
computes squared distance in x/y/z order, applies the strict cutoff, and adds
`coefficient * q * q` to the record's binary32 SSE accumulator, where
`q = (distance * scale - 1)^2`. After each record the original uses x87 to
add that record sum to the running binary32 field and stores it before the
next record. Negative coefficients therefore affect the field even though the
positive-only influence query would not append them. Checked nonfinite and
work errors, restored FP state and transactional behavior are stricter than
native. The model does not represent SIMD dispatch variants, queue progression,
isosurface intersections or actual source object identity.

The focused tests passed for signed contributions, strict cutoff and null-cell
signed zero. A reviewed isolated host subsequently executed the original scalar field and
addressing spans for four fixed synthetic cases. All four field words matched:
null cell at point `[0,0,0]` and threshold 0 yielded `80000000`; one record
at `[0,0,0]`, threshold 0.5, with positive, negative and strict-cutoff lanes
yielded `3E800000`; two records at `[0.25,0,0]`, threshold 0.01 yielded
`3F7001FE`; and four 0.25-weight lanes at `[0,0,0]`, threshold 1 yielded
`00000000`. The fixture bytes define the complete lane inputs and are pinned
by the runner. The receipt
`target/creature-skin-scalar-field-native-20261108/result.json` has SHA-256
`260877a9477bd4ffba19990ca07fcecc6b8eb3aa1e3d45346b63a97bb82afa9a`.
The host used explicit 027F/1F80 controls, output canaries and stable supplied
records; it had no guard pages or sticky-flag comparison. This establishes
only the four scalar-dispatch cases, not SIMD behavior, source-selected
runtime identity, queue, isosurface, or generated mesh. Endpoint hashes do not
prove continuous stability or complete build provenance.

## SSE field variant

`4F8C70` chooses the query's field function using cached `4F87E0` feature
selection: scalar `4F9440` for selector 0, `4F9840` for selector 1, and
`4F9760` for selector at least 2. The selection depends on the source's cached
CPUID path; it is not established for any particular live creature.
`creature_skin_sse_field_v1` models only `4F9840`: scalar `4F9630` cell
addressing, four independent SSE lane sums seeded by the actual query +10
bias, and final `(lane0 + lane2) + (lane1 + lane3)` reduction. It rejects a
nonnull zero-record cell, which the original SIMD do-loop cannot safely walk,
and rejects nonfinite/overflow inputs more strictly than native. The caller
admits stable aligned records, valid addressing and FP conditions. It does
not model `4F9760`'s separate SSE2 packed addressing or reduction.

A reviewed isolated host matched five `4F9840` field words: the four scalar
fixture cases documented above (null signed zero, mixed signs/strict cutoff,
two records, and four equal lanes), plus a cancellation-sensitive record with
coefficients `[1e20,1,-1e20,1]` and threshold 0.25, which yielded 2.0 in the
native pairwise order. Complete inputs are pinned in
`target/creature-skin-sse-field-native-20261108/fixtures.bin`; the receipt
`target/creature-skin-sse-field-native-20261108/result.json` has SHA-256
`dda4512c301e90990c7a7f8bd8a3aea51ab5eb1dff6b0362302f3e01d1a309dd`.
The host used 027F/1F80 controls and output canaries but no guard pages or
sticky-flag comparison. These five synthetic SSE results do not establish SSE2,
queue, isosurface, selected runtime object or full mesh behavior; endpoint
hashes do not prove continuous stability or complete build provenance.

## SSE2 field and bounded packed addressing

`creature_skin_sse2_field_v1` models `4F9760` with explicit four-float point,
origin and scale storage and the actual query +10 bias. The source converts
all four scaled values with `CVTTPS2DQ`, saturates them to signed 16-bit
words, applies `PMADDWD` coefficients `[256,32,4,0]`, and forms the byte offset
from the signed x/y dword plus the **unsigned low 16 bits** of `4*z`. The
native routine dereferences that offset without an independent table bound;
the checked model preserves wrapping/aliasing but rejects any offset outside
its 512-cell snapshot, exceptional conversions, nonfinite arithmetic and a
nonnull zero-record cell. The fourth coordinate is read and converted even
though its packed coefficient is zero. With records, the native reduction is
`(lane0 + lane3) + (lane1 + lane2)`, not the scalar or `4F9840` order.

A reviewed native host matched seven fixed synthetic field words. With origin
zero and scale one in all four components, the cases were: `[0,0,0,0]`
null cell 0; `[1,2,3,0]` cell 83 with cancellation-sensitive coefficients
`[1e20,1,1,-1e20]`; `[-1,8,0,0]` aliasing cell 0; `[-255,0,-1,0]`
aliasing cell 63 via low-word extraction; `[4096,-40000,0,0]` saturating the
second coordinate to -32768 and aliasing cell 0; `[7,7,7,0]` selecting last
cell 511; and `[0,0,0,12345]` selecting cell 0 despite a nonzero fourth
coordinate. The latter five nonnull alias/boundary cases used coefficients
`[1,0.5,0.25,0.125]` and bias -0.5. The fixture, including all lane words, is
`target/creature-skin-sse2-field-native-20261108/fixtures.bin`. The receipt
`target/creature-skin-sse2-field-native-20261108/result.json` has SHA-256
`0a5589d14ff94d1b518c47c4de72915a7626812a0cb55e84d11c01ed9638773a`.

The host prechecked each fixed native offset as inside its allocated table and
checked SSE2 capability, with 027F/1F80 controls and output canaries; it had
no guard pages or sticky-flag comparison. This tests seven safe offsets and
field results, not native out-of-table or invalid-conversion behavior, live
CPU dispatch, source identity, queue, isosurface or generated mesh. Hashes do
not prove continuous file stability or complete build provenance.

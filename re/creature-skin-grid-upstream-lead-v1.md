# Upstream generated-skin lane and bounds lead

Static inspection of `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
This is a conditional path, not a live runtime observation or a complete source
creature reconstruction.

On the `413D70` path, after finding a row with +08 flag 1, the leading
contiguous class-5 rows each produce a builder node. At `413F90..413FDB`,
the row's float +2C and vectors at +54 and +48 are passed to `4A7550`; that
helper computes each component of `row[+54] + row[+48] * (C * row[+2C])`,
where C has binary32 bits `BE23C528`. The same +2C word enters `4A74E0`,
which multiplies by binary32 constants `3EC18CD8` then `3F333333`, and selects
at least `3D23D70A` for ordered finite inputs. The calls and 4FD2E0 append
make this computed vector the node's +00..+08, constant `1.0` its +0C,
the helper's result its +10, and sign-extended row +00 its key +14. Scalar
operation order and x87 return/store matter; decimal approximations are not a
substitute for the pinned bits. Runtime post-mutation row identity and the
meaning of those vector fields remain admissions. The path also emits segment
records between nodes and has a separate class-3 path; the node prefix alone
is not the full lane set.

`4FDA40` walks node records in order and invokes `4F9100` with center, node
+0C, node +10 and key. At `4F9100` it replaces each nonpositive scalar with
zero under its ordered finite branch. `4F8930..4F8A2B` stores center, the
first scalar as coefficient at +50, and key at +70. For the second scalar r,
it computes `r*r` at +30, `1/(r*r)` at +40, and coefficient times that
reciprocal at +60, with ordered binary32 SSE operations. A zero r can generate
an infinite reciprocal; the existing finite checked filter cannot accept it.
The bound accumulator at `4F92F0..4F9434` combines componentwise minima of
`center - r` and maxima of `center + r` into receiver +20 and +2C; its helper
comparisons and initial sentinels require explicit treatment of nonfinite and
signed-zero cases. `4F9100` append order includes all nodes before the sampled
interior segment lanes from `4FD3C0`; omitting latter changes both candidates
and bounds.

`4F9D60` consumes these receiver bounds and a positive caller step. Its
successful path divides the bounds by the step using a single reciprocal and
multiplications, performs the `4FAB30/4FABE0` coordinate decomposition,
rounds the span upward, checks axis sizes against the pinned constants 1 and
256, and computes the grid-origin/scale and 512 filter boxes. Constants in the
partition path include binary32 `0.125`, `0.5`, `0.0625`, `0.0001`, `2`,
and `8`. In particular, `4FABE0` subtracts one before `modf` on negative
inputs, including negative integers; replacing it casually with floor or
fraction can change boundaries. `414270` calls the builder and can retry with
a modified step after later work, so the step is not derived from the rows by
this trace. Exact helper order, x87 modes, all acceptance conditions, and the
caller-selected step must be modeled before claiming a source-built grid.

A specific next testable join is the **leading class-5 node slice**: inspect
frozen 0x8C row snapshots, calculate the two helpers' outputs with exact bits
and rounding, construct the direct node lanes and bound updates, and compare
against an isolated host executing the original helper/append slices with
admitted successful storage. A retained decoded CreatureData file such as
`0473.creature` (SHA-256
`dbbb55830f7f75eb5c45afd470a8bd0716cb08134e26f62a82674e28cd697137`)
can supply real row bytes under the already documented frozen-post-mutation
assumption, but the result must be labeled a partial prefix: no `4FD3C0`
segment samples, class-3 lanes, complete bounds, grid partition, runtime
object identity, or generated mesh are established by that experiment.
After native validation, add the independently traced segment and class-3
paths before feeding these lanes to the filter as a complete grid.

Reproduction: PE-map and decode `413D70..4141FC`, `4A74E0..4A751F`,
`4A7550..4A75E2`, `4FDA40..4FDBDF`, `4F9100..4F943C`,
`4F8930..4F8A2B`, `4F9D60..4FAB2B`, and `4FABE0..4FAC59`. The read-only
Ghidra logs `target/ghidra-creature-entry-v1/skin-query-owner-v1.log`,
`skin-lane-upstream-v1.log`, and `skin-grid-partition-builder-v1.log` provide
supporting pseudocode; instruction decoding governs these claims.

The checked `creature_skin_class5_prefix_v1` model now implements this direct
node prefix, including the row flag gate, ordered finite center/radius
arithmetic, lane fields and cumulative prefix bounds. Its caller admits stable
post-mutation rows and successful native preparation/storage. It rejects
nonfinite arithmetic and imposes a row cap; it does not model exception flags,
segment samples, class-3 lanes, full bounds or grid construction. Two focused
unit tests passed via `tools\build.bat test -p spore-core
creature_skin_class5_prefix_v1 --lib`.

The source inspector `creature_skin_class5_source_inspect_v1` verifies a bounded
CreatureData payload's caller-provided SHA-256, exact parse and explicit
`--assume-frozen-post-mutation` flag. The retained 0473 payload produced six
direct prefix lanes from ten rows. Its report is
`target/creature-skin-class5-native-20261108/source-inspection.json`, SHA-256
`4e9e9237119031f952076c49b91b75b4c1bc43c03f1be7a85d5aad9ebda56eb0`.
This is an inspection of the checked model on real row bytes, not an original
runtime observation. A reviewed isolated host subsequently executed the ten pinned original helper
and append/bounds spans on those six row records. All center, cutoff, scale,
coefficient, key and cumulative bound words matched the pinned Rust inspector.
The receipt `target/creature-skin-class5-native-20261108/result.json` has
SHA-256 `d01cce4ff2d10945bf0ba599ffd770177533c4f9d8a9ad200cedb216fa669a92`;
raw model and host stdout/stderr are retained alongside it. Both processes
exited zero. This comparison admits that the supplied file rows are the frozen
post-mutation rows and uses a synthetic preallocated receiver with capacity for
eight lanes so allocation-growth branches are not taken. It does not establish
runtime row identity, source preparation, segments, class-3, complete bounds,
grid construction, palette or mesh. The host has canaries but no guard pages;
MXCSR control is checked while sticky flags are not captured. Pinned endpoint
hashes do not prove continuous file stability or complete build provenance.

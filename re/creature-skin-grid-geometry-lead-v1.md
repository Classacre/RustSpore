# Conditional generated-skin grid geometry

Static trace of pinned `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
`4F9D60` consumes existing query-object bounds and a caller-supplied positive
step. Its checked-success path uses a reciprocal of the step and componentwise
multiplications, `4FAB30/4FABE0`'s integer/fraction decomposition, and `ceil`
to align the lower bound and calculate step dimensions. Each dimension is
checked against 1 and 256 before forming world extents. The negative branch of
`4FABE0` subtracts 1 before `modf`, even at negative integers; the checked
model retains this behavior for its bounded finite domain.

After the aligned lower bound and step-scaled dimensions, the source computes
`delta = (aligned + world_size) - aligned`, `margin = 0.0001 * delta`,
query `origin = aligned - margin`, and `expanded = delta + 2*margin`,
preserving the instruction order and binary32 constants. Query `scale` is
`8/expanded`. For each axis, filter-box spacing is `expanded * 0.125`, the
first center is **aligned lower bound** plus half that spacing, and box half
extent is `expanded * 0.0625 + 0.0001`. Subsequent centers add the
zero-based cell coordinate times spacing; cell iteration is x-major/z-minor.
The filter threshold is the query object's +0C, distinct from the step.
`4FA800` passes the saved aligned lower vector, not the padded query origin,
to the first-center calculation; the distinction affects candidate inclusion.

`414270` calls the grid builder with a supplied step. When grid construction
and later operations succeed but a separate downstream condition asks for a
retry, it multiplies the step by binary32 `3F99999A` (1.2), subject to its
other exit conditions. A builder failure exits this loop rather than being an
automatic step retry. Upstream path `418A80` initializes a candidate step from
`13EBA90` (`3D0F5C29`, about 0.035), but property/provider choices can
replace values, and this does not establish the step used for retained 0473.
See `skin-grid-lifecycle-math-v1.log` and `skin-identity-lifecycle-v1.log`.

`creature_skin_grid_geometry_v1` is a bounded checked arithmetic model with
explicit bounds, step and threshold. Its `derive_and_pack` composes derived
boxes with `4FAC60` filtering and owned packing. It admits x87 027F,
MXCSR 1F80, compatible CRT `modf`/`ceil` results and successful preparation;
it restricts decompositions to a bounded finite range, rejects inverted bounds
and invalid/intermediate values, and does not implement retries or establish a
caller step. The retained 11-lane construction requires the prior frozen-row,
fresh-query threshold and experimental CRT-provider assumptions; even if it
produces 512 boxes it is not a demonstrated original grid or live creature.
The query-origin/scale arithmetic and resulting per-cell candidate counts
and key order have the bounded native comparisons below; they do not validate
all copied lane words or original allocation behavior.

Reproduce the static claims by PE-mapping `4F9D60..4FAB2B`,
`4FAB30..4FAC59`, `4FAC60..4FAD7C` and `414270..414365`; the pinned source
and supporting read-only Ghidra logs are cited in the grid pack and threshold
notes. Further validation should compare origin/scale and all 512 box inputs
and packed cell records against a controlled native host at explicit steps,
including negative integer boundary and retry-independent cases, before using
the model as a native-conformance claim.


A reviewed isolated original-instruction host compared the native initial
origin and scale with the Rust model for three bounded cases: positive bounds,
negative-integer lower boundaries, and retained partial 0473 bounds with an
explicit 0.035 step. All three return values and origin/scale words matched.
The receipt `target/creature-skin-grid-geometry-native-20261108/result.json`
has SHA-256 `b03e1fc710259b29b95eb8471775fd72d9afa310393e3d2cb0d5e9165c556a2b`.
The host deliberately supplied a zero-lane receiver, so original `4F9D60`
returned before constructing any boxes, filtering or packing. It used pinned
CRT path/file checks, but did not independently verify the mapped image; its
output canaries did not provide guard pages or sticky-flag comparison. The
source-bound bounds and explicit step retain the admissions described above.

The separate partial-source model composition supplied the eleven admitted
0473-derived lanes, their computed bounds, 0.01 threshold and explicit 0.035
step. It produced 230 nonempty cells and 1,090 candidate entries. The report
`target/creature-skin-grid-geometry-native-20261108/partial-source-model.json`
has SHA-256 `358f61b35369cc90491bdf4d5d9455f4b0a06c90755f0ed754d78d902eec7859`.
Those counts were subsequently compared with the bounded native host below;
the step was supplied for the experiment, not established as the actual
source-selected step, and no live creature was observed.


A second reviewed host executed the original `4F9D60` builder, cell-box
arithmetic, `4FAC60` candidate filter and `4F8A30` lane-copy instructions on
two fixed inputs: five synthetic lanes and the eleven conditional partial
0473 reference lanes. It matched Rust origin/scale plus all 512 cell candidate
counts and key orders for both cases. The synthetic case had 511 nonempty
cells and 2,406 entries; the partial-source case had 230 and 1,090. Receipt
`target/creature-skin-grid-cells-native-20261108/result.json` SHA-256 is
`a47a51387964721b62079d8a8357ce50cbb364e07cef32353132c255d7c17a89`.

This host substituted a bounded scratch allocator, a preallocated aligned
cell pool, and no-op cleanup for the initially empty table; successful native
memory management is an admission, not a comparison. It checked per-cell
counts and key order, not every copied lane word, and used fixed valid inputs,
output canaries without guard pages, and no sticky-flag comparison. The loaded
CRT module path and file hash were checked but mapped bytes were not. The
0473 case retains the frozen-row, query-threshold and supplied-step assumptions
and does not establish live object identity or a complete source grid. Endpoint
hashes do not prove continuous stability or complete build provenance.

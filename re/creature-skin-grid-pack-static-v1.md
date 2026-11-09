
# Supplied candidate grid packing

Static instruction evidence from installed `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The `4F9D60` successful construction path loops 8 x 8 x 8 cells. For each,
`4FAC60` supplies candidate lane indices; zero candidates produce a null cell.
Otherwise it allocates `(count + 3) >> 2` records of 0x80 bytes and copies
selected lanes with `4F8A30`. The loop counts down, but writes each source
candidate into the destination slot with the same candidate-array index, so
the resulting lane sequence preserves the supplied candidate-array order.
The unused trailing slots are initialized by `4F8860`; its finite lane words
are zero, including cutoff, and its key word is zero. The `4F8A30` span copies
all eight words per lane; this projection models only those consumed by the
existing query kernel. This is conditional static evidence, not a native run.

`creature_generated_skin_grid_pack_v1` takes explicitly supplied active lanes
and 512 candidate lists and builds owned records for the existing query and
vertex fixture. It rejects invalid candidate indices, allocation failure and
more than 4,096 records per cell. The filter's geometry, ordering and threshold,
source lane creation, bounds, floating-point details, live object identity and
selected rows are **not** inferred by this bridge. In particular, these
synthetic candidate lists cannot demonstrate an original generated creature.
The existing query's finite-input checks and exception limitations also apply.

Reproduce static evidence by PE-mapping `4F9D60..4FAB2B`,
`4FAC60..4FAD7C`, `4F8860..4F8921`, and `4F8A30..4F8ADC`. Read-only Ghidra
pseudocode is retained at `target/ghidra-creature-entry-v1/skin-grid-partition-builder-v1.log`
and `skin-grid-candidate-filter-v1.log`; instruction decoding controls the
claims where the decompiler's calling convention is incomplete.

The source inspector `creature_skin_source_vertex_inspect_v1` accepts a decoded
CreatureData file and expected SHA-256, an explicit JSON snapshot, and the
`--assume-frozen-and-supplied-snapshot` flag. The snapshot contains
`endpoint_positions` (one XYZ array per topology endpoint), `point`,
`grid_origin`, `grid_scale`, `lanes` (objects containing `center`, `cutoff`,
`scale`, `coefficient`, `key`), and `candidates` (exactly 512 arrays of lane
indices). Its result records both input hashes, addresses, request count,
weights and initial record. Candidate lists must already be in the order
supplied by the source filter, and all coordinates and lane words are explicit
inputs, not derived from the source file. The supplied SHA is checked over the
bytes read; this is not a continuous stability guarantee. The Rust
`evaluate_packed` API also accepts those components directly and routes them
through the existing vertex fixture.

The packing model validates all candidate indices and record counts before
allocating cells. It limits the whole grid to 65,536 records as well as 4,096
per cell; these are model resource limits, not observed native limits.

One installed decoded payload (`0473.creature`, SHA-256
`dbbb55830f7f75eb5c45afd470a8bd0716cb08134e26f62a82674e28cd697137`)
was inspected with an explicitly synthetic snapshot (SHA-256
`684b2dd73685354bd5a963e9fe0ca27b8a5343539707eac3eb356d2ada51f8ae`).
It produced 10 rows, 11 endpoints, 10 edges, and one query request. All
positions, lanes and candidates were artificial; this only exercises the join,
not native geometry or grid construction.

## Prepared-cell candidate filter

The checked `filter_and_pack` bridge now models the finite 4FAC60 filter on
explicit prepared cell boxes and lane records, then passes its selected indices
to the owned packer. `evaluate_filtered` composes that result with the existing
row/topology and one-point accumulator. The caller must supply consistent
boxes, grid origin/scale, lanes, endpoint coordinates, and threshold. This is
not the 4F9D60 grid-bounds builder and does not derive any of these from rows.

On the pinned executable, instructions 4FACA9..4FAD14 subtract the cell
center, clear the sign, subtract half extents, clamp against zero, square and
sum x/y/z in order, then calculate `((distance * scale - 1)^2)^2 * coefficient`.
The cutoff comparison is strict; a masked weight participates in one of four
lane sums. 4FAD1D..4FAD51 appends positive lanes in descending lane-index
order, and 4FAD57..4FAD76 reduces `(sum0 + sum2) + (sum1 + sum3)` and discards
the entire cell if this is below the supplied threshold. The model assumes the
source's padded lanes are inactive; it rejects nonfinite inputs and arithmetic,
uses scoped MXCSR 1F80, and imposes the packer's allocation limits. These are
checked-model restrictions, not native exception or allocation behavior.

The retained 0473 CreatureData snapshot supplies real row bytes, but the trace
in the row-edge note explicitly leaves endpoint-coordinate preparation and the
identity of the native prepared geometry unproved. Its translation fields do
not establish the 4FAC60 lane centers, cutoffs, coefficients, cell boxes, or
threshold. Therefore no source-geometry claim is made and no synthetic values
are presented as original geometry. A native filter comparison and the upstream
4F9D60/lane-production and source-geometry joins remain open.

A separately reviewed, hidden 32-bit host executed the unmodified pinned
`4FAC60..4FAD7C` span on six fixed synthetic eight-lane cases and compared its
ordered candidate indices directly with the pinned Rust comparator. All six
matched: `[0]`, `[0]`, `[]`, `[7,6,5,4,3,2,1,0]`, `[7,6,5,4,3,2,1]`, and
`[7,6,5,4,3,2,1]`. These cases exercise reverse ordering, strict cutoff equality,
threshold equality versus the next larger binary32 value, box extents, and
nonzero scales. They compare candidate decisions, not intermediate score bits
or all possible floating-point inputs. The host installs MXCSR `1F80` and
checks its control bits after each call; sticky flags are not recorded.

The checked receipt is
`target/creature-skin-filter-native-20261108/result.json`, SHA-256
`b4e2536cfccfc81f33cec331c39559026d1ca809578ad1affc0f8da8f0adcce6`.
The exact source span SHA-256 is
`98eb2ef129fe6ba87ed5e608622c191b8f1609799d40b2fe871ca5e0a3a86da0`.
The successful host SHA-256 is
`ee5ecd7b2dd5ba13f871120e7dc71e2de83a18f1335c1b3410428e5526d63da1`;
the pinned comparator executable is
`83bc7eed24f8a2c96043a1fe5b6cd104dbf04c22e2976beea1723ce14b6ff869`.
Source, host, generator, assembly and comparator pins were checked before and
after the run. The first host attempt exited 7 with no output or receipt;
its requested fixed allocation was not aligned to Windows reservation
granularity. That is a static explanation of the failure, not an observed
failure-site diagnostic. The corrected host requested an aligned region.
The host had output canaries but no guard pages; it used explicitly supplied
synthetic lanes and boxes and did not execute grid construction, creature
regeneration, or the game. Hashes do not prove continuous file stability or
complete build provenance.

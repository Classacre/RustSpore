# Supplied generated-skin spatial grid query

Static x86 evidence is from installed `SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
This joins the previously modeled four-lane kernel to a caller-supplied grid;
it does not build the original grid, produce geometry or observe the game.

`4F9630` passes the point and receiver +50 vector to `41D9B0`, which subtracts
the latter from the former one scalar SSE component at a time. It then passes
that result and receiver +4C vector to `4FCBD0`, which multiplies corresponding
components one scalar SSE operation at a time. The active fixed branch uses
`CVTTSS2SI` to truncate each component to signed 32-bit integer. At
`4F9720..4F9747`, 32-bit shift/LEA arithmetic computes `z + 8*y + 64*x`; only
that result is checked **unsigned** against 512 before receiver +54 is indexed.
There is no individual axis clamp in the traced slice. Negative coordinates and
integer overflow can therefore alias a different valid linear word, subject to
the source floating-point and conversion conditions.

`4F9C30` emits nothing when that lookup returns null. Otherwise it reads a count
at the supplied cell pointer minus eight, then calls `4F8AE0` on each 0x80-byte
record in order. That kernel processes four lanes in order and appends positive
weights under its separately documented admissions. The actual native vector
allocation, cell construction and resource/object identity are not established.

`creature_generated_skin_grid_query_v1` takes explicit finite origin, scale,
point and a caller-supplied table of optional record slices. It models
the same ordered address arithmetic with scoped MXCSR and wrapping 32-bit
indexing, preserves record/lane request order, and limits each selected cell to
4,096 records. It rejects nonfinite inputs or intermediates and conversions
outside the representable signed range instead of reproducing native exceptions
or partial writes. The original scoped floating-point status and append-helper
side effects are not returned. Four focused tests passed for normal, negative,
wrapping, outside-table and conversion cases, plus ordered dispatch of two
explicit records and rejection of excessive or invalid work. These tests use
synthetic data.

An independently reviewed x86 host executed the unmodified `4F9630`,
`41D9B0` and `4FCBD0` instruction spans with their original relative calls and
a synthetic receiver/table. The Rust comparator matched all six returned cell
markers, covering truncation, negative aliasing, 32-bit wrapping, index 511,
the out-of-range result, and nonzero origin/scale. Markers were returned only,
never dereferenced. This did not execute `4F9C30`, build a native grid or use a
live actor; canaries were used without guard pages. The checked receipt is
`target/creature-skin-grid-query-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `fd64ee4c01493ca9afa5358650606392ec79fb76e16905a3606ba0badaa84ee7`.
Hashes do not prove continuous file stability or complete build provenance.

Reproduce the static inspection by hashing the PE and decoding instruction-aligned
`4F9630..4F9756`, `41D9B0..41DA3E`, `4FCBD0..4FCC5E` and
`4F9C30..4F9C95`, mapping virtual addresses using image base `00400000` and the
PE sections. Further work must trace `4FDA40`'s construction into those cells,
the actual selected source rows and point geometry, and the resulting palette
identity. A supplied table is not evidence of any of those runtime values.

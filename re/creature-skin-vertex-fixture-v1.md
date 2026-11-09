# Supplied-input generated skin vertex fixture

`creature_generated_skin_vertex_fixture_v1` composes the checked row-to-edge,
endpoint-membership, row-remap, spatial query, four-lane kernel, ordered
accumulation, ranking and initial 20-byte record models for one supplied point.
The initial accumulator is +0 and has one word per supplied constructed edge,
as on the traced producer path. The fixture keeps query/record order, forwards
the explicit remap, and returns the topology, query requests and unsmoothed
record together. It rejects invalid row/endpoint relationships, nonfinite
endpoint coordinates, excess queries and downstream checked errors without
returning partial output.

Its admission requires the caller to provide stable **post-mutation** rows,
corresponding endpoint coordinates and the already constructed source-style
grid; native preparation, allocations and append operations are assumed
successful, and positive correction and smoothing are excluded. None of those
objects is derived from a live creature by this fixture. It does not construct
the source surface, infer ownership or prove the row-to-animation-palette
association. Zero-weight placeholders do not indicate an actual influence.

Three focused synthetic tests passed. One walks a supplied high-bit grid lane
through endpoint membership and remapping into a normalized record, then
changes the second row to class 1 and observes the remap select its parent.
Another composes high-bit and clear-bit queries in source order, and the third
rejects mismatched or nonfinite endpoint snapshots. These are Rust
composition checks: the combined sequence itself was not executed in the
original program. Individual grid, kernel, accumulation, ranking, record,
edge and remap slices have separate fixed native comparisons documented in the
linked notes; matching separate slices is not evidence of a live end-to-end
editor or saved model.

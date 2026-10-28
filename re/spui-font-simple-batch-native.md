# Source font0 simple-glyph Rust coverage

[MEASURED] The batch attempts all578 glyphs in the actual font0 resource at
fresh15ppem.81 of389 simple glyphs complete the bounded Rust hint/raster path
and match34,520 canonical numeric rows. Those glyphs have324 retained native
fresh observations.1,165 pinned inputs/references/models remain unchanged.
Report: `target/source-ui/font-simple-batch-rust-run-v2/acceptance-v1.json`.
Actual TSV SHA256 is
`d7b7baca262bb7c820f3b89c10344d8c5712d8345f694995588fa71cc03c173e`.

Each completed group compares all design/original/hinted points, touched flags,
CVT/storage, graphics state, final stack/axis/reference points/call count, advance,
both dropout-profile raw raster crossings and alpha, and final origins after
the source phantom subtraction/alignment. Expected rows come from the pinned
original-instruction/native references; root independently regenerates them.
No glyph coordinates or expected pixels are embedded in Rust.

308 simple glyphs stop at explicit unsupported operations, function or shape
guards.183 composites and six empty glyphs are excluded. The largest initial
opcode gaps are MDRP variants, followed by loop/state operations. Passing glyphs
are complete numeric groups; partial output from excluded glyphs is discarded.
The frozen census's historical stream/PC labels after100 program bytes remain
excluded from this comparison, while whole numeric state and pixels are retained.

The first batch verifier failed compilation because its parser-enum match omitted
ZeroContours. The preserved v3 verifier adds that typed exclusion and renames an
unused binding. No hint/raster numerical code or expected values changed.

[UNKNOWN] The remaining simple glyphs, composites, other ppem/faces, persistent
CVT/storage and cache-miss ordering, real atlas placement, text layout and actual
SPUI screens remain open. Fresh glyph comparisons do not establish those paths.

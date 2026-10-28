# Whole font programs with original intersection

[MEASURED] The public VM now completes 360 of 389 simple font0 glyphs at fresh
15 ppem. All 164,948 numeric rows agree with original instruction references:
points, touch flags, six vector words, dot and callback identities, CVT/storage,
graphics, scan events, raw alpha and phantom-relative origins. The cohort retains
1,440 earlier native observations. All 353 previous completions remain.

The seven newly completed glyphs execute 28 intersection dispatches in both
the original source interpreter and Rust. The VM pops target,b0,b1,a0,a1 in
literal source order, snapshots all endpoint coordinates before writing, and
preserves existing touch bits through the accepted intersection helper.

700 separate owned cases cover operand binding, including 525 cases where the
output point aliases an endpoint. All 6,300 rows and four typed guards pass.
The source generator's raw TSV uses CRLF; Rust uses LF. The original raw source
and first hash assertion remain preserved. An explicit source-only LF projection
matches Rust bytes exactly; this changes no numeric data or implementation.
All 1,186 public check pins remain unchanged.

Whole-glyph SHA256:
`0bbeb4a0031fd96d841c53aaf89358adb573c78083781a25c05f8d59514c503f`.
Acceptance: `target/source-ui/font-general-intersection-rust-run-v2/acceptance-v1.json`.

[UNKNOWN] Full-program87 and F20/its called functions remain unaccepted. There
are29 remaining simple-glyph exclusions,183 composites and six empty glyphs.
Actual empty-program bypass, larger maxp shapes, other zones, persistent cache
order and the complete UI text pipeline remain separate work. No new native
execution or complete font/UI claim is made by this gate.

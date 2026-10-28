# Original font intersection arithmetic

[MEASURED] The public bounded intersection helper matches 2,172 original
instruction-reference cases in private and public Rust. The source oracle
interprets the full 641-byte `008AC3C0` handler and its 198 bytes of fixed-point
helpers. Root independently reproduces the expected stream. No new native
execution is claimed.

The original uses separate horizontal/vertical paths, fixed slope division,
multiply or reciprocal division selected by magnitudes, and a parallel fallback
that shifts deltas separately before adding bases. Line order and signed
truncation remain literal. This is not an ideal geometric intersection formula.
All nine reachable branch blocks in the owned coordinate domain are exercised;
the zero-numerator alternate at `008AC5ED` is unreachable after the preceding
nonzero-denominator and magnitude tests in this nonoverflow domain.

Coordinates are explicitly bounded to absolute value 8,192. Two typed guard
assertions reject outside values and signed minimum. Existing touch bits are
ORed with 3. Source snapshots also prove the four line points remain unchanged.
The initial 12 candidate pins and 532 public reproduction pins stay unchanged.
SHA256 `5636f98ec2e07c54b2ec83c23fe6073bf79a784601fd4b1a7f81dc297c001edd`.
Acceptance: `target/source-ui/font-intersection-rust-run-v1/acceptance-v1.json`.

[UNKNOWN] This arithmetic gate alone admits no VM opcode or glyph. A separate
candidate integrates literal five-operand stack order, endpoint aliases, point
writes and touches, and requires complete glyph state/raster comparisons.
Other zones, persistent caches and complete UI text remain outside this gate.

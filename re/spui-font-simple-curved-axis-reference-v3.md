# Curved reference v3: source-derived final axis correction

[CORRECTED REFERENCE ONLY] The frozen v2 reference inherited a formatter that
emitted normalized final axis0 unconditionally. That happened to match all
previous positive glyphs. Actual P(gid49) and U(gid54) end with source opcode00,
so their final projection/freedom are Y and normalized axis must be1.

This field is an owned normalized diagnostic derived from original frame
state,not a literal axis byte in the original structure. It is retained and
now derived from allfour frame words14/16/18/1A,dotword40 and allthree callbacks
44/48/4C. Unsupported vector/callback combinations reject. Final stack length
is read from remaining_stack; RP0/RP1/RP2 still come from frame30..38.

Independent proof interprets the exact original dispatch table entries and
both original setters on owned A5-filled frames. Slot1432F30(op00) points to
8ABEE0: frame14/18 become0,16/1A become4000,40 becomes4000,44 becomes8AB9A0,
48/4C become8ABA40. Op01 at8ABF20 gives the X tuple4000/0/4000/0,callbacks
8AB970/8ABA30/8ABA30. Every other frame byte remains unchanged. Fresh actual
D/P/U final frames and last source opcodes independently match these tuples.
The original handler bytes/ranges are digest-checked against immutable source.
No Rust or native executable is used to generate this proof/reference.

Exactly two of2291 rows differ between v2 and v3: zero-based1044(gid49 final)
and1753(gid54 final),only axis0->1. Every point,GS,CVT/storage,reference-point,
stack,advance,crossing,bitmap metric and alpha field stays identical.
V2 canonical SHAefe9a23d1ac78e17b29c1a4616457f2debc4d14677f1174b81d01a0ab74e90e5
and the initial Rust output are retained. New v3 source canonical SHA is
25b98fee70d0a92dfa618ad342e2ad88f12263bf8ae053004ae4de03a426cfc4.
It equals the previously produced first Rust TSV; the generator does not read
Rust output. The separate review script compares it only after the source
reference and source setter proof exist. No numerical Rust change is made.

P's105-byte program already uses the corrected full-owned-program trace
window in the new curved hint probe. The frozen389-glyph census's historical
post-byte100 stream-label limitation remains documented independently; its
final-state/pixel comparisons are unchanged. Native full-frame observation,
allfonts/scales/composites,UI lifetime/bindings and fulltext remain separate.
Root independently regenerates the new source fixture and compares the
retained first Rust output before deciding acceptance.

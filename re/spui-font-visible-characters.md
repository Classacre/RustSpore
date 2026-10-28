# Four actual visible font0 characters

[MEASURED: SOURCE INSTRUCTIONS] Separate read-only source replay now closes
actual underscoreU+005F(gid64), barU+007C(gid93), hyphenU+002D(gid14) and
capitalIU+0049(gid42) at fresh15x15ppem. All are four-real-point,on-curve,
one-contour axis rectangles. Original font0 package provenance remains the
caller contract; lengths/topology alone do not identify a font.

| Character | Hint bytes | Dispatches | New point semantics | Bitmap |
|---|---:|---:|---|---|
| underscore |19|12|MDRPD6/CD/C6|10x3,left0,top0|
| bar |24|21|No new point arithmetic; F75false branch|2x15,left256,top768|
| hyphen |29|12|MDRPD6/CD|5x3,left64,top384|
| capitalI |31|12|MIAP3F andMDRPD6|3x12,left64,top768|

The source instruction counts after fresh PREP are1125,1472,1189,1193
respectively. Bar's ordinary CALL enters original FPGM definition75 at
1347..1370. It executes PUSHB/RS/EQ/IFfalse/POPfour/EIF, preserving the shared
operand stack/cursor contract. The unvalidated true branch is not admitted.

[MEASURED] MDRP8AE410 projects original signed DESIGN words then invokes the
source15/32 units callback. Subtracting already-rounded26.6 original point
coordinates is a different operation. Source flagsD6/CD/C6 then apply grid
rounding/minimum distance/reference-point updates. MIAP3F8ACB90 uses the
ordinary glyph zone, source CVT cut-in and grid round callback. No twilight
initialization, diagonal vector or floating-point branch is added.

[MEASURED: RETAINED NATIVE] Full source hint plus original one-contour scan
conversion agrees byte-for-byte with allfour fresh native observations per
character:16 distinct observations total. Alpha, dimensions,pitch,left/top,
advance,CVT and storage agree. Both source dropout settings agree. Advances
are589824,589824,393216,262144 respectively. Source input points and complete
GS/frame projections remain source-instruction-only; the native host did not
emit their complete structures. Sequence contexts are excluded here.

[MEASURED: ACCEPTED RUST] New spui_font_hint_axis.rs independently models only this
cohort's axis opcodes, variable/fixed PUSH, SRP0, MDRP flags, MIAP3F, existing
MDAP/MIRPED/IUP/DELTAP semantics and borrowed F75false-body processing.
CALL depth32 and dispatch fuel4096 complement checked operand/index accesses.
Changing storage2 to reach F75's true branch produces a typed rejection.
The source-FPGM table is constructed by accepted bootstrap code; bodies are
actually entered, not replaced by constant results.

New spui_font_raster_axis_rectangle.rs uses the recovered source
crossing/gray arithmetic behind a separate one-contour rectangle guard. It
reuses accepted public raster parameters/result types and does not edit the
accepted glyph0 implementation. Actual hinted points produce all pixels.
Negative underscoreY coordinates use signed bounds arithmetic.

The pending verifier reads originalfont0 from the installed Graphics package,
uses its Microsoft Unicode format4 cmap, accepted parsing/scaling/PREP,
then the new axis hint/raster modules. The independent generator replays
original source instructions for allfour characters and both dropout settings.
Its1,357 canonical rows include mappings, every dispatched opcode/stack depth,
original/design/hinted points,touched flags,full GS,CVT/storage,reference points,
advances,normalized crossing lists,metrics and every alpha byte. SHA256 is
653de66a2113a1a0154800d7a02aa79a325936f6eaaba836df259354f0b36690.

[MEASURED: ACCEPTED RUST] Root independently reproduced all1,357 canonical
rows and compiled Rust matched the source/root streams exactly at the SHA above.
Both positive branch/depth guards pass, as do the prior913/249/1389-row
regressions. Acceptance report: target/source-ui/font-visible-rust-comparison-v1.json.
Accepted glyph0/PREP/raster arithmetic has not been edited.
Other glyphs, curves,fractional points,other ppem/aspects,prior-glyph lifetime,
font fallback,kerning/shaping,complete text layout and UI screens remain open.

The initial pending verifier used storage16 for its positive F75 test. Root
identified the actual borrowed PUSH2/RS operand; only the test setup changes
to storage2=6. The VM already read the correct source-selected slot. Root
preserved the failed/partial artifacts before its verifier-only correction.

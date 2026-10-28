# Font0 simple-glyph source/native census

[MEASURED: SOURCE INSTRUCTIONS + RETAINED NATIVE] The full578-glyph font0
inventory contains389 simple glyphs,183 composites and6 empty glyphs. All389
simple glyphs complete original saved x86 hint and quadratic/line raster
instruction replay at fresh square15ppem. No unsupported source instruction,
VM opcode or helper is reached in this census. No native process was run.

The strict hint census checks every reached JG consumes defined Z/S/O flags;
other predicates retain the integer model's defined-flag checks. Each glyph
receives an independent copy of the same complete source PREP snapshot, with
original coordinates copied before phantom grid locking. Source image, font,
frontier and tool hashes accompany the reports. This is original-instruction
model evidence; it does not make arbitrary same-shaped font data original.

[MEASURED: RETAINED NATIVE] All389 alpha buffers match all four retained fresh
mode0 native observations per glyph,1556 observations total. Full width,height,
pitch,final origin,advance,CVT and storage also match after replaying the
original driver's final origin policy. Both source dropout0/1 settings produce
equal alpha and raw raster metrics for the same hinted input.

The final origin is not the internal scan-converter's raw origin. Source
8C0EA5..8C0EB1 copies raw raster18/1C into glyphE0/E4. Source8C0F7D..8C0FA8
subtracts the first current phantom X/Y and appliesFFFFFFC0 to both glyph
origins. The independent final comparison replays that exact source slice with
owned buffers and its explicit EDX=0 entry condition from8C0F60. Negative
native origin DWORDs are signed-normalized for comparison. Native intermediate
hinted points and full graphics/frame structures were not observed.

Historical first-pass reports remain retained. Raster censusv1/v2 compared the
internal raw origins directly:158 full matches and231 apparent metric
mismatches; all389 alpha buffers already matched. Final-origin censusv3 still
compared signed source origins to unsigned wire DWORDs:370 full matches and19
negative-origin differences. V4 corrects signed comparison. V5 additionally
pins the cleanly scoped strict hint reports and both raster dropout replays.
These exploration reports do not authorize a changed raster metric formula.

[BOUNDARY: RUST] Existing accepted Rust gates remain913 PREP rows,249 glyph0
hint rows,1389 glyph0 raster rows and1357 four-visible rows. This census does
not expand Rust acceptance. No accepted implementation/tool was edited.
Composites,other ppem/aspects,sequence lifetime,shaping/kerning,text layout and
complete UI screens remain outside this gate.

The smallest useful next cohort is capital L/T:6/8 on-curve points forming
rectilinear polygons. L adds MIRP FD; T adds MIRP E6 and uses the already
source-closed F75 false body. A separate bounded polygon raster path and
variable point/contour axis VM avoid per-glyph bitmap constants. E/F/H then
share SRP1/2,IP39,ALIGNRP3C and more MIRP flags. The broader original source
frontier also executes curves for ordinary digits and letters, including D/P/U;
no new REA query is required for the measured simple-glyph domain. A generic
quadratic Rust port still requires independent implementation and root review,
build and canonical/native comparison before acceptance.

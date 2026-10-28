# Generic bounded simple curves: D/P/U candidate

[PREPARED, NOT RUST ACCEPTED] New unexported spui_font_hint_simple_curved.rs
and spui_font_raster_simple_curved.rs form one shared axis-hint/quadratic raster
path. Accepted glyph0,axis,rectilinear and PREP files are unchanged. Actual font0
D(gid37,19points/2contours),P(gid49,22points/2contours),U(gid54,18points/1contour)
are the proposed positive cohort. No glyph coordinates or bitmap bytes are
embedded. Caller original package provenance remains required; bounds/shape
admission do not identify font content.

[MEASURED: SOURCE REFERENCE] Fresh original x86 hint and raster replay produces
2291canonical rows,SHAefe9a23d1ac78e17b29c1a4616457f2debc4d14677f1174b81d01a0ab74e90e5.
Allthree actual glyph alpha/metrics/advance/CVT/storage targets agree with all
four retained fresh native observations each:12 distinct native observations.
The source generator also replays each hinted outline translated by(+64,-128)
under the same converter parameters. These3 owned geometry variants are not
native font/ppem claims. Actual+translated outlines under profiles(dropout0,
providerA5) and(dropout1,provider0) yield12 raster cases. All outputs are from
original instruction replay,not the derived Rust/Python algorithm.

The derived high-level integer algorithm development check independently
matches every crossing list and alpha byte for D/P/U against the frozen x86
oracle. The original constructor makes27quadratic calls and84flattened line
calls total: D9/33,P10/29,U8/22. This development check is not Rust execution.
Source ranges,digests and original disassembly remain only in ignored target.

New hinting is SHP32/33 through8AD0A0,8ACF30,8AD030. Op32 selects RP2,op33 RP1;
source projects current-minus-original reference displacement and adds it to
the popped point. Ordinary coincident axis projection/freedom vectors make
source2.14 multiply8D0D50 exact identity. Only the active axis is moved/touched.
The existing variable point/contour SRP,IP,ALIGNRP,MIAP,MDRP/MIRP,IUP,DELTAP
policies remain duplicated behind the new module. SLOOP/zone/vector changes
and other unimplemented states reject. Depth32 and dispatchfuel4096 remain.

Raster follows8C8310,8C72C0,8C6EB0,8C8000,8C7900,8C7320,8C70A0,8C86E0.
Quadratic8C7900 chooses one uniform depth from
max(abs((P0-2P1+P2)>>2)) and repeatedly quarters error until<=4. It uses literal
+1 midpoint and+2 quarterpoint biases before arithmetic shifts,then processes
left half before right half. Implicit adjacent-off-curve endpoints use+1
midpoints. This is the original fixed integer flattening rule,not a tolerance
chosen to fit the glyph pixels.

Small-span8C7320 X-scan interpolation uses signed truncating IDIV; Y-scan uses
its literal direction/sign-dependent denominator-minus-one bias. Half-grid
endpoints use the previous nonconstant segment start; half-grid horizontal
segments insert both original crossings. Event lists retain source encoding
coordinate*2|direction and nonzero winding.8C70A0 derives bitmap bounds by
combining populated half-grid scan rows with event extrema,not control-point
bounds.8C86E0 preserves same-pixel horizontal accumulation and carries vertical
endpoint coverage across adjacent intervals before its3970-based integer gray
blend. Output remains original0..126 coverage,without normalization.

Admission is4..64 real points,up to8 strictly increasing inclusive contour
ends,on-curve contour starts whose coordinates are not half-grid ties,coordinate
magnitude<=8192,bitmap extent1..64. Bounds prove constructor special-span mode0
(<B504),safe32bit interpolation products and bounded subdivision. Off-curve or
half-grid contour starts reach explicitly rejected constructor-mutation paths.
The previous-axis value remains optional until source semantics define it;
undefined prior-endpoint use rejects rather than assuming provider-zero state.
Segmentfuel4096,eventlimit512perrow and depth8 are owned resource guards.
Gray3/quadratic2,minimumprecision0,equal dropout flags,no scan upper bits,
no transform/LUT/special/postprocess fields are required. Allocation/provider
and memset remain bounded owned storage,not native heap/cache admission.

Bitmap left/top are internal raw xmin/ymax+64,preceding the source driver's
phantom subtraction/alignment. Actual D/P/U and the owned translations have
aligned origins,so their values equal retained native cache origins. This API
does not silently invent final metrics for other fractional-origin outlines.

The pending verifier reads actual installed font0 through accepted parsing,
scaling and PREP; new hint output feeds the new raster. Positive guards request
unsupported0F,off-curve start,half-grid start,coordinate bound and wrong gray
control. These guards and all Rust numeric results remain unexecuted until
root compile/comparison. Existing913/249/1389/1357/2573 regressions remain separate
root gates. Full389-glyph Rust coverage,composites,other scale/font,allseven
sequence cases,actual UI cache/lifetime,texture/shader bindings,layout and full
screens remain open; the source UI/lifetime ledger is separate evidence.


Root acceptance (2026-10-27): target/source-ui/font-curved-rust-comparison-v3.json records all2291 source/root/Rust rows and15 positive guards passing. The v3 diagnostic correction derives final axis from original projection/freedom words and three source callbacks; it changes only P/U final-axis diagnostics and no numerical Rust implementation. Earlier prepared/pending paragraphs and frozen v1/v2 evidence remain historical. This accepts the bounded D/P/U cohort only; broader glyphs, live UI state/cache/lifetime and rendering remain separate open gates.

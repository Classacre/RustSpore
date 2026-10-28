# Original font0 glyph0 hinted outline

[MEASURED: SOURCE INSTRUCTIONS] Newsource instruction reference executes the
complete38-byte .notdef glyph0 program after the accepted freshfont0 PREP at
15x15ppem. It performs15VM dispatches and1,988 originalx86 instructions, with
empty final operand stack, restored incoming cursor/CPU stack/nonvolatile
registers. It does not execute native machine code or a retail process.

The source real points finish at(64,192),(64,576),(448,576),(448,192),
(128,256),(384,256),(384,512),(128,512). All coordinates are source26.6 integers.
The untouched IUP results are included; four phantom points remain separate.
Final reference points are3,3,5. Original program SHA256 is
932c6b5a550851f79c2bd884bb6ec0b248ddd2d622ed3067c69a86fd34d4e9a7.

[MEASURED] Source initialization8CC5C0..8CC7E5 constructs the glyph descriptor
at twilight base+2C, sets fourDWORD/twoWORD/twoBYTE array pointers and contour
start/end pointers, copies current scaled points into original scaled arrays,
and zeros touched flags. It then grid-locks phantom origin and advance.
Consequently the original phantom advance is480 while current advance is512;
original top is635 while current top is640. The first exploratory glyph dump
copied original arrays after grid-lock; that report is preserved and rejected
for the original-phantom fields. Correctedv2 retains exact copy ordering.

The new handlers reached are MDAP1 at8ACB10, MIRPFD/ED at8AE590,
SDS at8AEA90, DELTAP1 at8AEB80 and IUPx/y at8ACC70. Source templates choose
ordinary axis callbacks; no floating-point or diagonal branch is admitted.
The scalar interpreter closes all source reads/transfers used by this path;
source opcode/switch/dispatch bytes are pinned to the immutable executable.

[MEASURED: RETAINED NATIVE STATE] All8 retained glyph0 observations agree on
173CVT values,47storage values,round/CVT callbacks and flags after hinting.
Native controls areCW037F/MXCSR1F80/DF0. The native reference reports a7x7 alpha
image,left64,top640,alphaSHA256
9743b9addf12cd626b675657fa32b9814517f0f1260ba89caed0904ccac7d265.
These pixel/metric records are references, not equality claims for modeled
pixels. The native host did not emit hinted point coordinates or complete
frame/graphics structures. Those remain source-instruction-only evidence.

The separate unexportedspui_font_hint_glyph0.rs implements this real glyph path
using the accepted PREP/scaling/unscaled input. It fails closed on other
opcodes, shape/state mismatches and unsupported large IUP interpolation
branches. Shape guards do not establish font content identity. Fresh PREP
state is cloned rather than allowing ambiguous prior-glyph lifetime.
The verifier remains.pending until root compiles and compares all249 canonical
rows against independent source instruction replay. No build was run by this
worker, and no Rust hinted-outline acceptance is asserted yet.

[UNKNOWN] Source hinted coordinates have no independent native-point snapshot.
Rasterization, native pixel equality, other glyph programs/composites, varied
ppem/aspect, source diagonal/FP branches and whole UI screens remain open.
The accepted PREP, bootstrap and control arithmetic were not modified.

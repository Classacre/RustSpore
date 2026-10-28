# Source UI font selection, alpha upload and persistent hint lifetime

[MEASURED: SHIPPED CONFIGURATION, NOT LIVE SELECTION] The base copyright
Window17 selects GuiDataSmallCentered7237635F. Its retained inheritance chain
in base/PatchData/EP1 declares Trebuchet EA,bold,font-smooth:always and11px.
The matching installed font0 resource is027C5CEF/00010000/0248E873511B49CA,
117628bytes,SHAf6ad6f1e12e2d2a7c1e28b0ea3cb93a653482840b2b6340fb8682f3eb4c75f26.
Existing spui-title-native.md and spui-font-native.md retain the loader/style
and native ABI evidence. Missing style,temporary WinText size override,
recursive fitting254==2,fallback and live registration/order remain conditions;
this declaration does not by itself prove the active font pointer on a screen.

[MEASURED: SOURCE SIZE/FLAGS] CSS896370/8962F0 uses unit multiplier1.0 for both
px and pt,so11px supplies float11.0. Font891390 calls891150; fixed matrix inputs
use65536 and original resolution96/72. Source8BF6E0/8BF430 produces rounded
15x15ppem from fixed961191. This is the source derivation behind the explicit
native fixture; arbitrary UI sizes are not all15ppem.

The newly retained exact Ghidra891D50 dossier(q43) closes745bytes at
891D50..892038 inclusive. Caller951D40 passes zero fractional positions and
flag argument0. Bitmap891D50 chooses flags8B when that argument's bit0 is clear
and83 otherwise. A0==1 or nonzero effectsA4 selects mode3. With no effects,
outputbits=(-(A0!=1)&FFFFFFF9)+8:8bits whenA0==1,otherwise1bit. Effects instead
select32bits. Normal source style smoothing214 is sent through890D60 toA0.
Thus mode3/flags8B/8-bit byte-alpha is a closed conditional source chain,
not a choice inferred from the alpha histogram.

[MEASURED: LOCAL CFG, METADATA ABSENT] Exact Ghidra analyses for88D010,
88DD90 and88B8E0 report no observed procedure; the containing queries for the
firsttwo also report not_in_procedure. No neighboring body is substituted.
Observed vtable142F640 entries point to them. New bounded Capstone local CFGs
from those entries close1668,186 and315 reachablebytes respectively,with no
unresolved jumps or escapes to the next observed vtable entry bounds. These
are local entry CFGs,not Ghidra function metadata or closed direct-callee effects.
890FC0's138-byte local CFG is also retained as the per-font character coverage
check. Exact88E0C0(q41) instead enumerates0x74-byte description records; it does
not establish style matching or an activeface.

Within88D010,caller reuseflag gates the existing-font list at+80. Source
88D5C1..88D606 calls existingfontv2C for its description,requires absolute size
difference strictly below binary32epsilon3D4CCCCD (approximately0.05),and equal
smoothing/style214 and effect/style218. Success88D69A calls AddRef on the same
font pointer. Otherwise88D649 calls serverv14=88D860 to create a new live font.
This provides an explicit persistent-object path; actual reuseflag/caller
values and runtime selectedface remain open. Source88B8E0 gives exact-family
score1000 versus900 for a nonmatching UTF16 family and applies source size,
style,weight and other-description penalties. Complete selection/fallback
requires caller/catalog inputs and the other callees; it is not inferred from
that initial score alone.

[MEASURED: CACHE AND RELEASE] Source951D40 requests cachev14 before fontv50.
A hit skips raster/hint execution. A miss calls fontv50=891D50,then cachev1C
upload and fontv54=891480 release. Cache88ADF0 calls88AB90; its bucket index
uses((font_pointer<<16)+token16) modulo bucketcount,but equality compares the
full font pointer and16-bit token. Hash truncation is not identity truncation.
Source891480 calls8BFDF0 with purgelevel1. That enters8BFD30,which disposes
current glyph/bitmap buffers104/118/11C and does not invoke PREP or directly
restore the separate CVT/storage arrays. Font close890DB0 is separate and
calls8C1600 before clearing font15C. Complete font-cache invalidation/eviction
and object-destruction ordering remain open; cachepage clear is not a reason
to reset hint state.

The existing source8CC2DB..8CC2E3 same-ppem early return skips PREP.
8CC7B3..8CC7D2 clears per-glyph flags and copies default graphics words without
restoring the separately allocated CVT/storage. Thus glyph programs execute in
font-object state order. Combining the source paths implies that UI hint
writer order follows actual cache misses,not every text character occurrence.
A cache hit retains its earlier bitmap even if another miss has since changed
CVT. This is a source consequence; no retail cache-miss trace was captured.

[MEASURED: NEW SOURCE SEQUENCE/NATIVE COMPARISON] A fresh source PREP executes
once per context. New original-x86 replay compares[354] with[342,354],preserving
CVT/storage/twilight across the latter while source resets ordinary glyph frame
and graphics. Glyph342 changes CVT14 from-192 to-256. Gamma354 preserves the
incoming value and yields12 versus13pixel height. Complete original raster and
final-origin replay matches all12 retained native order-intervention
observations for alpha,width/height/pitch,origin,advance,CVT/storage.
This extends source-model lifetime evidence to that exact sequence. It does
not claim allseven context-dependent glyphs,live cache order,native fullGS/
intermediate points,or a Rust sequence implementation. Comparison manifest:
target/source-ui/font-ui-sequence-comparison-v1/manifest.json.

[MEASURED: ALPHA TO TEXTURE BYTES] The source cache default is512-square format3,
created asDXT3. Source88A400 byte-alpha conversion stores each input alpha's
high nibble;88A8C0 clears alpha to0 and RGB565 endpoints towhite. No normalization
or multiplication by2 occurs in this branch. Existing accepted spui_glyph_atlas
has5201 independent source-loop/Rust comparisons. New composition feeds the
389 retained actual simple-glyph alpha images through that source primitive at
owned block-aligned0,0 in a freshly cleared default512page. Every stored nibble
matches alpha>>4; every white RGB block is preserved. Maximum126 becomes7.
Report: target/source-ui/font-ui-alpha-upload-v1/manifest.json. This is byte
composition,not observed runtime atlas placement,eviction,texture sampling,
shader/blend bindings,GPU execution or full visible text. Existing controlled
original UI shader PNG runs are separate evidence and do not close these font
bindings. No reconstructed alpha is silently rescaled to255.

[REPORTING LIMITATION] Frozen simple census reports inherit a100-byte glyph
trace-label window from the earlier visible scout. Long programs still execute
correctly,with exact final state/pixels,but trace stream/relativePC labels past
byte100 are wrong. The389/1556 census is a final-state/pixel comparison,not a
complete389-program canonical trace gate. The five E/F/H/L/T programs are all
shorter than100 and unaffected. New sequence tools correct the full owned
program address window without changing VM execution or frozen accepted tools.

[OPEN] Active retail face/size overrides/fitting,fallback catalog/coverage,
font reuse caller arguments,cache invalidation/eviction/paint order and allseven
sequence variants;actual font atlas placements and shader/sampler/blend binding;
composites and general curved Rust text,kerning/shaping/layout and fullUI.
Smallest follow-up source entries for cache epochs are cachev10=889F80 and page
releasev24=7A8BC0 plus their actual caller edges. No additional query is needed
to repeat the measured gamma sequence or alpha composition above.

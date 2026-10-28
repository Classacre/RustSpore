# Original glyph0 scan conversion reference

[MEASURED: SOURCE INSTRUCTIONS] The original8C8310 constructor and8C86E0
bitmap writer now execute completely in the bounded integer instruction model,
using the actual source-hinted glyph0 points. Each run performs5,029 source
instructions for dropout0 or5,027 for dropout1. String instructions are modeled
with their exact count and DF0; each REP counts as one instruction here.
No retail/native host, compiler or GPU is executed.

[MEASURED: EXACT RETAINED PIXELS] All49 output alpha bytes agree byte-for-byte
with all8 retained nativeglyph0 observations. Width7,height7,pitch7,left64,
top640 and advance524288 also agree. AlphaSHA256 is
9743b9addf12cd626b675657fa32b9814517f0f1260ba89caed0904ccac7d265.
The native/source maximum alpha is126(7E), not255. No guessed conventional
antialiasing engine or replacement font rasterizer was used.

## Actual source chain

All bodies/ranges are already pinned byfont-fixture-frontier-v7.json:

-8C8310,630bytes: allocate/initialize converter, set gray/curve/control fields,
 padded edge-list domains and invoke quadratic contour processing.
-8C72C0,95bytes: extrema over real points only, through final contour endpoint.
-8C6EB0,389bytes: initialize internal event-node pool. No pool growth was needed.
-8C8000,769bytes: original quadratic/on-curve contour traversal. Glyph0 uses only
 on-curve straight edges; eight calls reach8C7320.
-8C7320,1,486bytes: sample contour edges on half-grid scanlines, encode crossing
 position ascoordinate*2 plus winding directionbit, insert sorted events into
 horizontal/vertical lists. Large-span and curve branches are excluded.
-8C70A0,542bytes: derive populated event extents/bounds before bitmap writing.
-8C86E0,2,662bytes: allocate/zero49-byte image, apply original winding intervals,
 gray edge coverage and second-axis weighted blend. No LUT/effect callback.

The reference records every visited source instruction, normalized sorted
crossings, source function entry counts and raw object snapshot under target.
No missing function query or new Ghidra session was needed.

## Bounds, sampling and gray coverage

[MEASURED] Real point extrema arex64..448,y192..576. Source writer uses
left=xmin>>6=1,right=(xmax+64)>>6=8,bottom=ymin>>6=3,
top=(ymax+64)>>6=10. Thus width/height are7 and output top26.6 is640.
The extra top row and right column follow source bounds arithmetic.

Half-grid events lie on samplepixel*64+32. The horizontal lists are:
row3:[129,896]; rows4..7:[129,256,769,896]; row8:[129,896].
Vertical lists arecolumn1:[384,1153]; columns2..5:[384,513,1024,1153];
column6:[384,1153]. Remaining padded rows/columns are empty.
The low event bit preserves winding, rather than treating every adjacent
crossing as an independent fill interval.

[MEASURED] In the gray branch, source8C86E0 uses event>>7 for pixel coordinates
and(event>>1)&63 for edge fractions. Interior bytes are literal7E; endpoint
coverage uses twice the fractional span and63-minus-fraction. A vertical pass
blends horizontal/vertical edge contributions with sourceweight3970-(63-a)^2,
using signed integer division. This is the recovered source algorithm, not
an assumed standard area-coverage formula. The49-byte output contains only
0and126 for this axis-aligned glyph.

## Caller contract and owned provider boundaries

[MEASURED] Native source driver8BFE40 passes mode3, curve type2 from ordinary
glyph parser8CD77E, and the same glyph+74 boolean to both dropout arguments.
Glyph+74 is assigned1, a scan-controlbit, or0 in the original driver, so its
entire admitted domain is0/1. Both settings produce the identical actualglyph0
crossings, metrics and alpha. Exact choice need not be inferred from equality.
Accepted PREP scan-control upper bits17/18 arezero, matching converter flags.
Constructor8BF25C/25F clears optional transform/LUT fields48/4C;
8BF2B9/2BF/2C5 clears18C/190/194. Source8BF98B computes minimum precision
(min15*64)/(2048*2)=0 for field168. Thus converter precision/control tail
arguments arezero. The post-filter branches excluded by nativeflags8B remain
excluded. Curve/point flags are read through the original on-curve-bit tests.

The model substitutes only explicit bounded owned buffers for allocation calls
at8C8320/8C87DC and the one-byte memset call at8C8819. It models the allocator
storage provider, not native heap/cache lifecycle. Both0andA5 initial buffer
poisons independently pass for both dropout settings. Incoming nonvolatile
registers and CPU stack restore after constructor and writer. Allocator/cache
ownership equivalence is not asserted. The reference assumes DF0, matching
retained native context controls.

[PREPARED] The later unexportedspui_font_raster_glyph0.rs draft consumes
hinted input coordinates and explicit source parameters. It independently
builds sorted winding crossings, initializes the whole pitched image tozero,
and applies the recovered horizontal gray coverage/vertical weighted blend.
It embeds no bitmap. Its positive scope is two strictly nested on-curve axis
rectangles in the original order, integerpixel coordinates, output sizes1..30,
graymode3, curvekind2 and the source-supported equal dropout pair. Other
geometry/control states return typed errors. The size cap keeps both original
event domains within the exercised inline-row-buffer branch. The verifier
reconstructs originalfont0 through accepted parsing/scaling/PREP/hinting before
passing its actual eight real points to the rasterizer. It also checks seven
owned geometry variants, including negative coordinates and asymmetric holes.

[MEASURED: SOURCE FIXTURES] The new generator executes16 original-instruction
cases: actualglyph0 plus seven owned grid geometries, each withdropout0/poisonA5
anddropout1/poison0. Its1,389 canonical rows cover complete crossing lists,
all pixels, metrics and actual source advance. SHA256 is
0959de501335b5d91ea9f947d57d4851cb18452882225be5f482a4403eb42c36.
Only the two actualglyph0 cases have retained native pixel equality; the14
owned geometry cases are independent source-instruction references.
The first generator version incorrectly subtracted unsigned negativeY bounds
when extracting its output. Those reports are preserved and rejected for
negativeY alpha fields;v2/v3 use signed bounds and retain all output bytes.

[MEASURED] Root independently reproduced the16-case source fixture, reviewed
the Rust geometry/control bounds and promoted the public module/verifier.
All1,389 source/root/Rust rows match SHA256
`0959de501335b5d91ea9f947d57d4851cb18452882225be5f482a4403eb42c36`;
both positive API guards pass. The actual installed font0 goes through parsing,
scaling, accepted PREP and glyph hinting into this Rust rasterizer; its49 alpha
bytes and metrics match all eight retained native observations.14 owned geometry
cases are source-instruction comparisons only. Acceptance report:
`target/source-ui/font-raster-glyph0-rust-comparison-v1.json`.
Frozen prepromotion source/verifier remain in the review archive.

[UNKNOWN] The reference directly invokes closed scan-converter
entries; whole8BFE40 execution/lifetime is not modeled. Other glyphs, curves,
non-axis edges, other ppem/aspects, dropout differences, LUT/style/effect
callbacks and full UI screens remain open. Native hinted point coordinates
were not emitted; exact final pixels do not prove unique intermediate points.
The frozen glyph0 hinting files and all accepted VM arithmetic are unchanged.

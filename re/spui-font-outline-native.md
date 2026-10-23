# Original ordinary glyph input boundary

The original native glyph corpus and its seven context-dependent results are
recorded in [spui-font-fixture-native.md](spui-font-fixture-native.md). This
note separates unscaled packet data from the phantom-point, scale and hint
inputs that follow. All addresses refer to the pinned base `SporeApp.exe`,
SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Native code, fonts and derived packet rows remain under ignored `target/`.

## Packet reader

`[MEASURED]` Native `8CD700` reads ordinary glyf headers. A negative contour
marker selects raw component records; a positive marker selects endpoint,
hint, flag and delta streams; zero returns at `8CE236` with point count zero,
without interpreting any putative instruction-length tail. Coordinate
accumulation and stores use 16-bit words, including `8CE1A1/8CE1B3` for Y.
The last component's instruction bit is retained at `8CD7E4..8CD7F5`, rather
than an inferred OR over every component. Native malformed repeat runs are
clamped; the Rust API rejects an overlong run explicitly.

`[VERIFIED]` `spui_font_outline` retains original design coordinates, expanded
raw flags, endpoints, borrowed hint instructions, component argument widths
and signed F2DOT14 transform coefficients. Empty loca intervals and nonempty
zero-contour packets remain distinct. It neither synthesizes phantom flags
nor flattens or hints component outlines. Root's five focused packet tests
pass. Fresh canonical Rust export matches the full independent packet
reference byte for byte: 64,705,863 bytes with the SHA-256 below, covering
all 51,680 glyphs, 2,888,616 points, 28,483 components and 13,084,181 hint
instruction bytes. The retained result is
`target/source-ui/font-outline-rust-comparison-v1.json`. This verifies packet
decoding only, not native scaled/hinted coordinates or rasterization.

`[MEASURED]` `tools/spui_font_outline_probe.py` has independently checked all
five installed faces against FontTools 4.60.1. This covers 51,680 packets,
2,888,616 simple points, 28,483 raw components and 13,084,181 original glyph
instruction bytes. Header boxes, endpoints, coordinates/on-curve flags,
program bytes and component descriptors agree with FontTools. Compression
flags and trailing bytes are preserved separately in the canonical rows.

Fresh canonical package intake uses the flag-driven index and actual
`type=027C5CEF`, `scope=0248E873`, 32-bit instance fields. The old extracted
file names contain historical compression-metadata labels; `00010000` and
`0001FFFF` in those names are not font scopes. Every cached font was matched
byte for byte with its freshly read canonical package payload, selecting
decompression from the source compression tag. The canonical packet TSV is
`target/source-ui/font-outline-expected-v3.tsv`, SHA-256
`d3e152068cc845caba0a62bc0a43af65b9e7049916507c90d1d52d53156d2c8e`.
The report is `font-outline-fonttools-v3.json` in that directory. The retained
v2 comparison's non-key row bodies are identical; the rekey proof is
`font-outline-canonical-rekey-proof-v1.json`. No native scaled-outline,
hint-engine or raster equivalence follows from this design-data check.

## Four simple-glyph phantom coordinates

`[MEASURED]` For a positive-contour ordinary packet, `8CD700` appends four
coordinate pairs at `8CE1C4..8CE223`. The inputs are the header's xMin/yMax,
horizontal bearing/advance and vertical bearing/advance. All additions and
subtractions below narrow to a 16-bit word, preserving native wrapping:

```text
x0 = i16(xMin - horizontal_left_bearing)
x1 = i16(x0 + horizontal_advance)
x2 = x3 = i16((i32(x0) + i32(x1)) arithmetic_shift_right 1)
y0 = y1 = 0
y2 = i16(yMax + vertical_top_bearing)
y3 = i16(y2 - vertical_advance)
```

The center uses the sign-extended, already narrowed x0 and x1; a negative
odd sum rounds toward negative infinity through SAR. It is not a float
average or a truncating signed division. Native X/Y arrays are separate
16-bit arrays, and the original real-point count excludes the four points.
This source fact does not establish the phantom flag values. The empty,
zero-contour and composite branches construct their own objects and remain
separate from this positive-contour interface.

`[MEASURED]` Ordinary glyph loader `8CE6C0` supplies those fields. With hmtx,
the native `font+68` object has bearing array at `+0C` and advance array at
`+10` (`8CE88D..8CE8A1`). With vmtx, `font+6C` supplies the corresponding
vertical fields at `8CE8BA..8CE8C7`. Without vmtx, the original fallback is:

```text
vertical_advance = u16(hhea.lineGap - hhea.descender + hhea.ascender)
vertical_top_bearing = vertical_advance / 10  // unsigned integer quotient
```

The fallback uses wrapping word arithmetic before zero extension and the
division (`8CE8D2..8CE8FC`). Hhea loader `8CD020` consumes the version, then
places the first three source words at native offsets `+08/+0A/+0C`;
`8CF6B0` selects table tag `68686561` and stores that object at `font+60`.
It therefore uses those explicit hhea fields, rather than a guessed OS/2
line metric. The ordinary call at `8CE972..8CE981` passes five stack words:
retain-hints, horizontal bearing, horizontal advance, vertical top bearing
and vertical advance, with native memory/stream receivers in ECX/EAX.

## Native scale parameters before glyph hints

`[MEASURED]` Native transform `8BF6E0` calls coefficient builder `8CC8C0`
for X and Y, storing the resulting twenty-byte structures at
`scaler+140/+154` (`8BF905..8BF91F`). The structure consumed by `8BF350` is:

| Offset | Consumed field |
|---:|---|
| +00 | multiplier, signed 16-bit |
| +02 | shift count, 16-bit; SAR uses x86 CL masking |
| +04 | denominator, signed 32-bit |
| +08 | rounding addend, signed 32-bit |
| +0C | fixed multiplier for selector 2 |
| +10 | selector, signed 16-bit |

For positive ppem/UPE inputs, the builder first removes common powers of two
from `ppem << 6` and UPE (`8CC8CB..8CC8E2`). When the reduced multiplier fits
its signed-word guard and the remaining denominator is a power of two,
it selects mode 0 and stores the denominator's trailing-zero count, plus
half the denominator as rounding addend (`8CC90F..8CC947`).

`[MEASURED]` The tested font-0 fixture has 15 ppem and 2,048 UPE. That path
produces multiplier 15, shift 5, denominator 32, addend 16, fixed multiplier
`00007800` and selector 0 in both axes. Native `8BF350` mode 0 sign-extends
each design coordinate, multiplies, adds and arithmetic-shifts
(`8BF3F3..8BF41D`). Thus this fixture's pre-hint coordinate conversion is:

```text
coordinate_26_6 = (i32(design_coordinate_i16) * 15 + 16) >> 5
```

Negative half ties are asymmetric: design coordinate -16 yields -7,
whereas +16 yields +8. The maximum signed-word product in this fixture fits
i32. The original caller supplies real-point-count plus four and separately
scales X/Y before hint setup at `8C0589..8C05BF`. This coefficient/point
input is not a final glyph outline: native hinting mutates it afterward.

`[VERIFIED]` Exported `spui_font_scaling` supplies the simple-packet phantom
coordinate formulas and ordinary no-vmtx fallback above. Its selector-0
builder admits positive 16-bit ppem/UPE inputs only when the reduced
multiplier fits the native signed-word guard and the denominator is a power
of two. All products and sums then fit i32. The remaining denominator is
at most 32,768 and divides 65,536 exactly, so the fixed-multiplier field has
an exact integer quotient without inventing a division-rounding policy.
Other selectors return named unsupported errors. Zero-input guards are an
explicit API policy, not inferred native recovery behavior. Root's five
focused tests pass. The module is not connected to a font renderer.

`[MEASURED]` The actual builder loads ppem with a full-width MOV at
`8CC8C7`, then SHL 6. Positive u16 values at or above 32,768 are not
sign-extended before that shift. The largest shifted input is 4,194,240;
the largest multiplier-fit guard intermediate is 4,227,007, both positive
i32 values. Native default selector 2 is stored before the fixed-division
call; the oversized-multiplier guard precedes the denominator's power-of-two
classification. Thus `(32768,2048)` selects mode 0 with multiplier 1024
and denominator 1, while `(32768,1000)` keeps selector 2 with reduced
numerator 262144 and denominator 125. The separate source-instruction
domain audit evaluates 196 boundary cases and retains the whole-domain
bounds in `target/source-ui/font-scaling-positive-u16-domain-audit-v1.json`.
This is an explicit callee-input contract, not proof that upstream retail
style/matrix conversion admits every such ppem. Unsupported-selector errors
do not reproduce native writes to unsupported coefficient buffers.

`[MEASURED]` `tools/spui_font_scaling_probe.py` supplies a separate integer
instruction oracle. It interprets the pinned coefficient builder and its
fixed-division callee, the hhea fallback, and the simple phantom suffix with
strict model memory bounds. It evaluates the actual four selector-0 data
instructions over all 65,536 signed words and checks an eleven-point full
selector-0 call model. Undefined flags remain unknown and any consuming
branch fails. No native machine code executes in this oracle.

Fresh canonical package intake then supplies 37,691 simple phantom sets.
Selector-0 scaling covers 37,254 simple glyphs on the three 2,048-UPE faces:
2,858,480 real points and 149,016 phantom points. The two 1,000-UPE faces
retain 437 explicit unsupported simple-scale records. The 13,939 composite
and 50 empty glyphs remain excluded. Its 51,685-row canonical reference is
`target/source-ui/font-scaling-expected-v2.tsv`, 27,347,887 bytes, SHA-256
`1937c6281123e39b3aaa79c2f3952b7ad4c9af45ca2b452ee16cd03a20b1d39b`.
The v2 oracle report, original port version and Rust verifier are pinned by
`target/source-ui/font-scaling-reference-packet-v1.json`. Root freshly
reproduced the complete instruction reference and Rust output byte for
byte; see `font-scaling-root-reproduction-comparison-v1.json` and
`font-scaling-rust-comparison-v1.json`. These comparisons preserve the
selector-1 exclusions of that original gate. Source-instruction emulation
and final native raster output remain distinct evidence.

`[RECOVERED]` Selector 1 is the sign-aware integer quotient path
`8BF3A5..8BF3F2`. It loads the denominator at coefficient+4, sign-extends
the multiplier word, and for each sign-extended design word computes the
signed product. The nonnegative branch applies LEA(product+round), CDQ and
IDIV. The negative branch applies SUB(round-product), CDQ and IDIV, then
NEG. Both divisions therefore receive a positive magnitude in the admitted
positive-u16 coefficient domain. Exact half ties round away from zero.
Stored shift and fixed-multiplier fields are not consumed by this branch.
At 15 ppem the two 1,000-UPE faces select multiplier120, denominator125,
round62, shift0 and fixed62914, with selector1.

The same native builder first strips common powers of two and preserves
selector2 for a reduced multiplier above32767. Classification as mode0/1
occurs only after that guard. For admitted selector1, multiplier is1..32767,
denominator is positive3..65535 and round is at most32767. The largest
negative-product magnitude plus round is1,073,741,823; the largest
nonnegative-product sum is1,073,709,056. Both fit i32, division cannot use
zero, and the final optional NEG cannot overflow. The fixed field equals
floor(multiplier*65536/denominator), whose numerator is at most2,147,418,112.
These bounds cover the declared callee-input domain; they do not infer
upstream retail style acceptance or selector2 arithmetic.

`[VERIFIED]` Separate exported `spui_font_scaling_quotient` implements
only this admitted selector1 path, with named unsupported results for
selectors0/2 and explicit API zero guards. Its new source-instruction
oracle adds checked signed CDQ/IDIV semantics to the unchanged selector0
interpreter. Nine signed quotient/remainder fixtures and two modeled
divide traps check the interpreter's signed behavior. CDQ preserves flags;
IDIV leaves them unknown. It then evaluates every signed word for five
coefficient classes: actual15/2048 and15/1000, half-tie1/384, largest-word
multiplier32767/192, and largest-denominator1/65535. Each quotient class
takes32,768 nonnegative and32,768 negative-product branches; an eleven-point
complete original-call model also checks stores, providers and stack.

The separate reference additionally records224 native-builder boundary
combinations, observing68 selector0,73 selector1 and83 explicitly unsupported
selector2 cases. Fresh canonical package reads cover all five faces at15ppem:
37,691 simple glyphs,2,888,616 real points and150,764 phantom points, with
13,939 composite and50 empty packets still excluded. The reference contains
379,589 rows and35,876,188 bytes, SHA-256
`cfb9b1846f2ab23ce84ae6528b4aaaddcb442a69041f490bf190c9d05fd13eb1`.
Its packet is `target/source-ui/font-scaling-quotient-reference-packet-v1.json`.
Root's fresh source-instruction oracle and Rust output agree byte for byte
across every row; the five focused tests also pass. The accepted comparison
is `target/source-ui/font-scaling-quotient-rust-comparison-v1.json`.
The original selector0 packet and expected file remain retained.

`[UNKNOWN]` Selector2 calls the native fixed multiply helper. Composite
assembly, empty-glyph phantom construction, native phantom flags and
per-point pre/post-hint traces remain separate work. Neither arithmetic
gate establishes Rust FPGM/PREP/glyph hints, raster output or native scaled
point equality.

Evidence: bounded pinned-byte disassembly in
`target/source-ui/font-native-008CD700-capstone.asm`,
`font-native-008CE6C0-phantom-caller.asm`,
`font-native-008CD020-hhea-reader.asm`,
`font-native-008CC8C0-capstone.asm`, and
`font-native-008BF350-coordinate-scale.asm`; source caller facts from the
admitted `8BF6E0` and `8BFE40` ordinary-path ranges. Pseudocode alone is not
used to infer the private register/stack ABI.

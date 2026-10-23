# Native UI glyph cache and raster prerequisites, 2026-10-23

`[MEASURED]` This note uses installed base `SporeBin/SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Original code excerpts/payloads remain in ignored `target/`. The shipped
style/font chain and the verified source SFNT reader are documented in
[spui-title-native.md](spui-title-native.md). These prerequisites do not
establish native text shaping, bitmap rasterization or a complete screen.

## Concrete command and cache path

`[MEASURED]` WinText paint `988960` builds native layout through `89D380`,
then sends 16-bit tokens and 32-byte placements through `96E6E0` to
Graphics2D `955E50`. The latter groups placements by their shared font
pointer, floors each bounding box, clips locally, and emits four 20-byte
vertices per token with unit UV corners. A separate 12-byte record stores
token and float clipping offsets. The original atlas coordinates are
resolved later; unit corners cannot be treated as a glyph atlas.

`[MEASURED]` `953250` packages command, vertex, font-run and token arrays.
Its object constructor `952790` sets vtable `1441124`. Slot `C` resolves
to no-op `DF5080`, so that slot is not a glyph renderer. SDK `95BE80`
passes manager and command to the active context's virtual `14`, while a
null context uses the fallback slot. Context `82EDC0` initializes iterator
`951CF0` and reads it through `952800`; the iterator calls `951D40` when
crossing an active glyph range.

`[MEASURED]` `951D40` uses cache pointer `166BA98`. Cache virtual `14`
looks up `(font pointer, 16-bit token)` and returns a page/UV record. A
miss can invoke font virtual `50` to produce bitmap data, cache virtual
`1C` to insert it, then font virtual `54` to release bitmap data. The
iterator rewrites the four quad UVs using cache origin, clipping offsets,
quad dimensions and the page multiplier at `40`. Page `4C` is compared
with the run's cached value before regeneration.

`[MEASURED]` Bootstrap `7A82A0` allocates a `1A8`-byte cache, calls base
constructor `88AD10(0)`, assigns vtable `140F950`, configures option
`2 = 1`, initializes it and registers it with `67DB00`. Getter `67D930`
returns that registry pointer at `15FE8E0`. UI initialization `82F8B0`
passes it to `9568F0`, which stores it in `166BA98` and obtains a page
through virtual `30(0,0)`. Its image wrapper `166BAAC` references the
page texture through global `154FCD0`. Initialization writes wrapper
field `24` (global `166BAD0`) to zero. Iterator `952800` consequently sets
batch mask 1 (LINEAR) and leaves mask 2 clear (the measured ordinary
SRCALPHA blend path). Transparent font texels retain white RGB.

| Cache slot | Native target | Measured role |
|---|---|---|
| `14` | `88ADF0` | Font/token lookup |
| `1C` | `88B000` | Allocate/write/cache glyph and normalized UV record |
| `20` | `7A8D10` | GPU page texture factory |
| `30` | `88B150` | Page creation/registration |
| `40` | `7A80F0` | Lock page texture |
| `44` | `7A8190` | Unlock page texture |
| `48` | `88A400` | Bitmap conversion into locked page |
| `4C` | `88A390` | Locked-page write wrapper |

`[MEASURED]` Constructor defaults are maximum pages 2, page size `200`
(512), format 3 and eight allocation columns. `88B150` uses the default
size for a new zero-sized page and writes `page[40] = float(1 / size)`.
The texture factory explicitly sets page format 3 and requests FourCC
`33545844` (DXT3), square dimensions from page `48`. These are bootstrap
and constructor facts; later runtime option mutations remain separate.

## DXT3 clear and byte-alpha write

`[MEASURED, STATIC]` Ghidra queries 467/468 explicitly report undefined
procedures. The complete SDK bodies are instead bounded as follows:

| Operation | Inclusive range | Bytes | Code SHA-256 |
|---|---|---|---|
| Bitmap conversion | `88A400..88A8BB` | 1,212 | `431c3cdc4b577304aea943b80d6e7bb4dcef507f173d892e1819537416b3f964` |
| Page clear | `88A8C0..88A9B8` | 249 | `7bc562a717f94937aa425e25ca3e5145abba793b095628788d83c630f7f45da9` |

These start at valid entries and end at return instructions before INT3
padding. Ignored `ui-native-glyph-atlas-functions-v1.json` retains the
image/code digests and SDK paths. No unrelated Ghidra body was substituted.

`[MEASURED]` Conversion is thiscall with ten stack arguments and `ret 28`:
destination pointer, X, Y, destination pitch, destination format, source
pointer, width, height, source pitch, source bits. For destination format
3 and an 8-bit source, the write base is:

```text
destination + (Y >> 2) * destination_pitch + X * 4
```

For each row, add `(current_Y & 3) * 2` within the DXT3 block. Four source
alpha bytes form a little-endian word with low-to-high nibbles
`a0>>4, a1>>4, a2>>4, a3>>4`, written at 16-byte horizontal intervals.
Each 1..3-byte remainder writes the same low nibbles and zeroes the unused
high nibbles of that row word. The source advances by its supplied pitch;
the destination block row advances only when the new Y is divisible by 4.
The byte-alpha branch preserves each block's RGB endpoints/indices.

`[MEASURED]` The clear method is thiscall with pointer, square size, pitch
and format, ending in `ret 10`. Format 3 loops over
`ceil(size/4)` block rows and columns, using the supplied row pitch. Every
16-byte block receives four little-endian dwords:

```text
00000000 00000000 FFFFFFFF 00000000
```

That initializes transparent alpha and white RGB565 endpoints, with zero
color indices. Byte-alpha writes therefore quantize each input alpha by
discarding its low four bits, without changing the block's white RGB.
The method also contains packed 1-bit and 32-bit source branches; those
are separate from the bounded byte-alpha operation described here.

`[UNKNOWN]` This does not prove the title's eventual bitmap source format
or native raster output. The X formula assumes block-aligned placement;
a bounded helper should reject unaligned X rather than inventing a
general-purpose sub-block insertion rule. Caller allocation, active page
eviction, complete locking lifetime and runtime atlas contents remain
separate verification requirements.

## Concrete mode-2 font lead

`[MEASURED]` Font server factory `8859D0` constructs `88EE60(0)` in a
`2FD0`-byte allocation; its vtable is `142F640`. The source `font`
directive callback invokes slot `2C`, resolving to `88C5C0`, with stream
and mode 2. That branch constructs a preview font with `891550`, opens
stream/face index through `891BC0`, and obtains its description with
`890E60`. The constructor assigns vtable `142F690`; its bitmap/release
slots `50`/`54` resolve to `891D50`/`891480`. Constructor `891550` has one
other direct caller, `88D8A2` inside server slot `14 = 88D860`. The live
mode-2 factory allocates `2D0` bytes, opens the saved stream/face through
slot `60 = 891BC0`, sets the style size through slot `48 = 891390`, and
passes style `214` through slot `64 = 890D60`, which directly sets font
field `A0`.

`[MEASURED]` Face opener `8916A0` selects platform 3 and encoding 1 in
the scaler's font-data subobject before requesting name IDs 4 and 1 with
language `0409`. It also resolves the space code `20` through `8C1750`.
This matches the verified reader's explicit Windows BMP cmap and English
family/full-name records for the ordinary initial mode-2 path. Later
encoding or face changes remain separate.

`[MEASURED]` CSS parser `896500` dispatches property enum 9 (`font-size`)
to `896370`, which writes style `200` through number/unit parser `8962F0`.
Unit table `154A808` assigns multiplier 1.0 to both `pt` and `px`.
Therefore the source title declaration `11px` supplies float 11.0; a
CSS-standard pixel-to-point conversion would alter the original behavior.
Property enum 10 (`font-smooth`) writes style `214` through the table at
`154A6D8`, where `always` is 1 and `auto`/`never` are zero. The source
copyright style consequently supplies `A0 = 1` on the ordinary live
mode-2 font path.

`[MEASURED]` Size wrapper `891390` sends diagonal size and zero skew to
`891150`. That method multiplies four matrix components by float 65,536,
truncates each to integer with SSE, and calls:

```text
8BF6E0(face, 1, 96, 96, &fixed_matrix, 1, &error)
```

Its returned metrics are converted from fixed 16.16 using float
`1/65536`. This establishes the transform input, not a final raster height.

`[MEASURED]` Raster method `891D50` locks the font, handles special token
`FFFE`, and otherwise calls:

```text
8C1310(face, glyph & FFFF, fractional_X_byte, fractional_Y_byte,
       render_mode_byte, flags, &error)
```

The normal cache caller supplies zero fractional positions and flags
argument zero. The resulting renderer flags are `8B`; argument mask 1
instead selects `83`. Render mode is 3 when font `A0 == 1` or effects
field `A4` is nonzero, otherwise zero. Without effects, output bits are
`(-(A0 != 1) & FFFFFFF9) + 8`: exactly 8 when `A0 == 1`, otherwise 1.
Effects select 32-bit output. Thus the source style-to-mode chain selects
the byte-alpha branch, subject to the actual face/style remaining active.

`[INFERRED]` These transform/render argument patterns resemble a T2K
scaler. An [IBM native T2K trace](https://www.ibm.com/support/pages/apar/IV45517)
independently documents the same render-argument structure. This is an
ABI comparison, not proof of the installed engine's vendor/version or
permission to substitute another rasterizer.

`[MEASURED]` Native `8C1310` uses MSVCR90 `setjmp3` on the font memory
object and delegates ordinary outlines to `8BFE40`; it is not a standalone
raster leaf. Allocation adapters `8A6E30`, `8A6E40` and `8A6E50` jump
through globals `1668A2C`, `1668A30` and `1668A34`. Setter `8A6E60`
assigns those callbacks. UI-owner constructor `813FF0` installs native
callbacks `926380`, `6B4970` and `613EA0`.

`[MEASURED]` The font's stream callback `890AF0` takes stream, destination,
offset and length. It calls stream virtual `28(offset,0)`, then virtual
`30(destination,length)`, returning zero only when the seek succeeds and
the read returns exactly the requested length; otherwise it returns -1.
These are concrete requirements for an isolated source-bitmap fixture.
The separately reviewed fixture below now exercises those callbacks
successfully; this source boundary alone does not establish bitmap output.

`[MEASURED]` The bounded original-font fixture now produces native bitmap
output without presenting a game window. All 578 glyphs of the pinned
TrebuchetEA Bold face complete at 15 ppem in 4,624 observations, across
fresh/ordered contexts and four heap policies. Seven context-dependent
results are preserved; identical ordered inputs match across heap policies.
The native coverage range for this mode-3 fixture is 0..126, retained
without normalization. The source admission, host/debug startup policy,
raw outputs and independent state audit are documented in
[spui-font-fixture-native.md](spui-font-fixture-native.md).

`[UNKNOWN]` The selected retail face and cache lifetime, full UI shaping
and layout, and independently reconstructed Rust hint/raster output remain
unverified. Reading SFNT metadata, or using another font engine, does not
establish native pixel equivalence.

Evidence: native v3 queries 205, 215, 230, 285, 288, 400, 422, 426, 427,
432, 435, 436, 437, 465, 466, 469, 470, 472, 473, 479, 480, 481 and 482;
complete bounded SDK
`target/source-ui/ui-native-glyph-*.asm`,
`ui-native-command-context-dispatch-sdk.asm`,
`ui-native-font-server-{factory,ctor,add-face,live-factory}-sdk.asm`,
`ui-native-outline-font-ctor-sdk.asm` and
`ui-native-font-{css-fields,css-size,css-number,size,smoothing,transform,
open-face,stream-read,memory-init,allocator-adapter,raster-init}-sdk.asm`.
Query 481 does not include its switch bodies in the inferred function
body set; the field cases above use the explicit SDK entries/jump table.

## Synthetic atlas comparison

`[VERIFIED]` `spui_glyph_atlas` supplies bounded
positive-pitch byte-alpha insertion and format-3 clear. The independent
Python source-instruction model in `tools/spui_glyph_atlas_probe.py`
generates input-only TSV for example `spui_glyph_atlas_verify`. It covers
5,201 cases: all alpha bytes, partial groups, Y offsets/block crossings,
block-aligned X, odd/padded pitches, clear plus write, entire destination
preservation, zero extents and explicit API guard failures. The native
instruction model supplies valid byte expectations; invalid-input statuses
are the Rust API's explicit contract rather than inferred native guards.
Expected LF SHA-256 is
`e47937d66bc9939e907910b3c1f9ab5649af24acf18650dfda924996b34221dd`.
All 5,201 Rust results match the independent model byte for byte; the
retained comparison is `target/source-ui/glyph-atlas-comparison-v1.json`.
The three focused primitive tests and the subsequent full workspace run
pass (484 tests in the latter). This synthetic corpus does not contain or
approximate a source glyph bitmap. Native SFNT design data and native
atlas-format primitives are separate verified steps. Native glyph output
has subsequently been measured in the explicitly scoped fixture; Rust
raster equivalence, live retail font selection, shaping/layout and the
complete screen remain unverified.

The native bitmap fixture's source inputs, exact scaler call order,
15-ppem transform derivation, allocator/stream ownership, retained failures
and successful output corpus are recorded in
[spui-font-fixture-native.md](spui-font-fixture-native.md).

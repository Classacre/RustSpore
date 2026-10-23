# Source UI property streams, 2026-10-22

## Coverage

`[MEASURED]` The Rust verifier opens 82/82 installed package indexes and finds 748
SPUI resources. All 748 parse into 93,726 elements and 1,183,787 properties, including
nested structures. Re-encoding the typed body reproduces every original byte.

`[MEASURED]` An independent Python walk agrees on all 748 resource records, object
counts, recursive property counts and body digests. The SHA-256 of sorted digest lines
with LF endings is `1718cc35c575392f8fa7949ab3cc4d3a034cf1f0bc3c9593a01afc740dc29252`.
The Python probe scans top-level Data/DataEP1/bp1content packages and explicitly records
three unsupported compact indexes. The Rust verifier recursively opens all 82 indexes;
those additional packages contain no SPUI resources, so both measured populations agree.

`[MEASURED]` Focused Ghidra analysis of base SporeApp.exe independently corroborates
the container/body framing: reader `0x82CE10`, writer `0x82DCE0`, and recursive property
reader `0x82B2D0`. The native reader accepts versions 1 through 3, limits the total
resource/proxy count to 6,000, and limits each object/structure's property count to 200.
It resolves tag 19 through the global object array, tag 18 through literal UTF-16 or
localization lookup, and tag 20 through a shared structure identifier followed by
per-item property counts. Root flags cause a separate registration call. Runtime class
defaults and property conversions still depend on the designer metadata.

The executable digest is
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Local native Evidence stays in ignored
`target/rea/20261022-anim-rig-finalize-v2/{40,41,53}-analyze_function.json`.

Commands:

```powershell
python tools/spui_layout_probe.py '<install>' --out target/source-ui/spui
tools/build.bat run -p spore-bevy --example spui_layout_verify -- '<install>' target/source-ui/rust-spui-digests-v1.tsv
tools/build.bat test -p spore-core spui::
```

`[MEASURED]` Eight focused Rust checks pass, including typed nested arrays, literal
UTF-16 and localized references, every truncated prefix, invalid property tags,
duplicate/missing/out-of-range elements and trailing bytes. Raw game resources, original
artwork and per-resource analysis JSON remain under ignored `target/source-ui/`.

## Body framing

`[MEASURED]` The old parser retained the body as an opaque designer-dependent blob.
The byte framing is self-typed and can be decoded without a designer schema:

```text
per element:
  LE u16 0x5FF5
  LE u16 global reference index
  LE u16 property count, high bit retained as root flag
  properties
end:
  LE u16 0x5FF5, LE u16 0xFFFF, LE u32 0x1C01C047

per property:
  LE u32 property identifier
  LE u16 type code
  LE u16 array count
  typed values
```

`[MEASURED]` There is exactly one element for each proxy ID. Global indices begin
after all image, atlas and hit-mask resources; the matching proxy supplies the element's
class identifier. Scalar bit patterns are retained without guessing signedness or units.

| Code | Measured representation |
|---|---|
| 2, 3, 7 | one byte per value |
| 5, 9, 11 | four bytes per value |
| 15, 17 | eight bytes per value |
| 16 | sixteen bytes per value |
| 19 | two-byte reference value |
| 18 | UTF-16 literal preceded by u16 length, or FFFF followed by two u32 localization keys |
| 20 | one shared u32 structure type, then per array item a u16 property count and recursive properties |

`[UNKNOWN]` Class-specific defaults, layout/procedure execution, hit-test semantics,
anchors, text style resolution, slicing, animation and event handling are not established
by an exact body parse. The typed data is a prerequisite for reproducing the source UI,
not evidence that the Rust UI already matches it.

## Resource identity leads

`[MEASURED]` FNV-1 hashes of literal names in the base executable match these source
resource instance low halves:

| Literal | SPUI instance |
|---|---|
| TitleScreen | 404642000910F2B9 |
| SporeTitleSeq | 404642005C5AC8CB |
| GlobalUICell-3 | 4046420026B068D2 |

The TitleScreen resource references PNG `31A448931264E677`. The exact PNG is present
in Spore_Graphics and decodes as a title-logo atlas. These hash matches establish naming
candidates and data links; they do not prove the original application's active screen
selection or the visual role of every field.

## Native local SimpleLayout rules, 2026-10-23

`[MEASURED]` Native class ID `2F418D72` is SimpleLayout. Bounded SDK getters
`97E840`/`97E890`, the name at `14441E8`, and property metadata at `1444144`
identify byte property `0F3D0000`, stored at object offset `10`. Native
`97E920` applies parent spans to local child edges; `97E990` subtracts them.
The property byte retains all bits. Only the low four affect these helpers:

- Bit 8 adds parent width to the right edge, and to the left when bit 4 is clear.
- Bit 2 adds parent height to the bottom edge, and to the top when bit 1 is clear.

Parent origin is not added by these local helpers. Float32 operation order is
retained. The separate wrapper at `97E8A0` clamps inverted right/bottom edges
after its virtual adjustment call. Procedure dispatch/order and subsequent
global transforms are separate semantics; these helpers do not establish them.

`[MEASURED]` `spui_layout::SimpleLayout` decodes the explicit source flag byte
and implements the two local operations and separate clamp. All 7,919 instances
in all 748 SPUI resources have that byte, with all sixteen low-bit combinations
observed. Independent Python/Rust records agree on the flags and fixed local
rectangle results. Sorted LF SHA-256 is
`cf1936e4e7497e17c98b78a036fff4127706863e2bf9b955e857dc450a09f5c2`.
The original 748 typed body digests remain unchanged.

Evidence: `target/source-ui/simple-layout-comparison-v1.json`, native v3 queries
173, 179, 184, 185 and `ui-{layout-identity,simple-layout}-sdk.asm`. The extended
`spui_layout_verify` accepts a third output path, checked by
`tools/spui_simple_layout_probe.py`. A local anchor operation passing every
source flag is not completed source UI rendering or event handling.

## ImageDrawable sizing and local clipping

`[MEASURED]` ImageDrawable class `CF3C4EAE` has a native descriptor at
`1443C34`, property table at `1550688`, constructor `97D850` and draw path
`97D1F0`. Property `4F3C4A27` supplies sizing flags, `4F3C4A29` horizontal
alignment, `4F3C4A28` vertical alignment and `4F3C4A2B` scale. Constructor
defaults are flags 0, scale 1 and both alignments 3.

Native flag bit 1 uses intrinsic image width times scale, and bit 2 uses
intrinsic height times scale. Other axes retain the input area. Alignment
2 moves the first edge to `last-size`; 3 centers by sequential float32
subtraction, multiplication by 0.5 and addition. Native 1 and other values
retain the first edge. This is a sizing pass before effects, tiling/slicing
and painting, not a generic CSS layout rule or an inferred stretch mode.

`[MEASURED]` `ImageSizing` preserves those defaults, property bits and operation
order. Independent Python/Rust fixture calculations agree for all 6,299
ImageDrawable instances in the 748 source SPUIs. The fixture supplies a fixed
area and intrinsic image size; this does not establish all resource resolutions
or final positions. Sorted LF SHA-256 is
`2eee97cb71807e3c865be3a05c142cf05c22378c421b0aba68b26ea247c4fb87`.
Evidence is `target/source-ui/local-ui-layout-comparison-v2.json`.

`[MEASURED]` Local image clipping in `95C1F0` preserves fully contained draws,
even below one unit. Partial draws require original width and height at least
one; the strict intersection helper `634520` rejects touching/disjoint edges.
Relative UVs use rounded float32 reciprocals followed by multiplication, before
the image's saved atlas UV mapping. `clip_image` implements these branches.
The focused checks cover contained subpixels, rejected partial subpixels,
cropped UVs, inverted containment, touching edges and nonfinite rejection.

Native v3 queries 187, 193, 194, 196 and 198, plus
`ui-image-{dispatch,clip}-sdk.asm`, retain the source evidence. Only metadata
and independently implemented mathematical semantics are in the repository.
Full WinProc dispatch/order, image effects, texture conversion/sampler policy,
complete UI rendering and event handling remain open.

## Serialized Image regions and atlas UV mapping

`[MEASURED]` Image proxy `01BE6B15` supplies resource reference `01BE0001`,
float rectangle `01BE0002`, signed integer dimensions `01BE0003` and raw
flag word `01BE0004`. Native metadata initializer `1300320`, UV setter
`957850`, getter `957810` and draw builder `955410` establish the property
and consumer chain. The setter stores origin and rounded float32 spans;
relative UVs are multiplied by those spans and added to the origin.
Coordinates are not clamped to the unit rectangle by this operation.

`[MEASURED]` `ImageRegion` retains all source values and implements that
mapping. Independent Rust/Python comparison covers **15,489 Images** in
all **748 SPUI resources / 82 indexes**. Every reference identifies a
container image/atlas resource. All explicit Image flags are zero, including
after serialized property application; **1,347 UV rectangles extend outside
0..1**. Source flags therefore support linear/source-alpha drawing in this
path without asserting every later dynamic mutation or texture upload.

The canonical records include source reference, raw flags, dimensions,
saved UV bits and an independent out-of-range relative-UV fixture. Sorted
LF SHA-256 is
`25fa25bccf3c88d500a451e6259822987fb87b176dae0f0170a57810d9cbe34a`.
Evidence: `target/source-ui/image-region-comparison-v1.json`; reproduction
uses the fifth output argument of `spui_layout_verify` followed by
`tools/spui_image_region_probe.py`. Existing typed-body and local sizing
digests remain separate gates. No complete screen is claimed by this data
or the controlled native draw fixtures in `re/spui-draw-state.md`.

# Payload format notes

Working notes on the resource payload types, as of 2026-10-16. Grades: `[MEASURED]`,
`[INFERRED]`, `[UNKNOWN]`.

The container and the compression codec are solved and validated
(`docs/dbpf-format.md`). This file is about what is *inside* the payloads, which is
mostly still open. It is deliberately blunt about that.

## Summary

| Type | Name | Status |
|---|---|---|
| `0x2F7D0004` | PNG | **decoded.** Standard PNG. 13,174 extracted, all CRCs valid. |
| `0x00B1B104` | prop | **Primitive numeric correction validated on 30,885/30,889 resources across 82 indexes.** 847,835 INT32/UINT32/FLOAT values independently digest-checked; four named-framing exclusions remain. Earlier size-only completeness claim withdrawn. See `re/prop-numeric.md`. |
| `0x2F4E681B` | RW4 | **container and object payloads 100% (10,797 of 10,797).** Magic, header, section-info table, per-object type/data ranges, full file tiling, and every object's payload size validated. See below. |
| `0x2F4E681C` | RASTER | **decoded and validated.** DXT1, DXT5, and raw RGBA sub-formats. Extracted to PNG with zero errors. See below. |
| `0x01C135DA` | gmsh mesh | **decoded. 100% (2,482 of 2,482).** Every sample fully parsed with valid indices. Index magic accepts `15 00 06 08` and the variant `14 00 06 08`. See below. |
| `0xEE17C6AD` | anim | **Typed key parsing: 5,022 of 5,022 resources across 82 packages.** 48,700 channels and 878,791 rows independently digest-checked; generic safe sampling implemented, native rig binding/curves open. See below. |
| `0x5CAA5E28` | blockdata | **Partial source layout recovered.** 36-byte header, dimensions and three-byte samples; 296,176 samples across 3,098 selected records agree with an independent decoder. Interpolation, caller semantics and terrain role remain open. See `re/blockdata-native-sample-static-v1.md`. |
| `0xDFAD9F51` | Cell definition | **partially interpreted.** Fixed 796 bytes; structure reference and developer label identified. Earlier string-table/null-padding identification was incorrect. |
| `0x02B9F662` | audio event | **decoded. 100% (16,841 of 16,841 across `PatchData` and `Spore_Audio2`).** Variable-length entries, prop-style array markers, length-prefixed string arrays, FNV-1 hashed keys. See below. |
| `0x00E6BCE5` | gmdl model | **Typed geometry/tails independently checked on 11,041/11,041 across 82 indexes.** Versions 8/9; material payloads, part palettes and 46,576 external bone records preserved. Body hierarchy and ambiguous influence mappings remain open. See `re/gmdl-skin.md`. |
| `0x03F51892` | audio event (variant) | **decoded. 100% (90 of 90).** Same FNV-1 hash scheme and entry rules as 0x02B9F662; covered by the same parser. |
| `0x02D5C9AF` | hash-pair table | **decoded. 100% (1,631 of 1,631).** version 4, then two `(count, count×12-byte records)` sections. See below. |
| `0x030BDEE3` | content manifest | **decoded. 100% (1,641 of 1,641).** version 10, ASCII tag, UTF-16 string list, hash lists. See below. |
| `0x0F43029A` | creaturedata | **decoded. 100% (1,022 of 1,022).** magic `0xABB455B7`, version 9/10, RLE chunk buffer, capability/morph tables. See below. |
| `0x250FE9A2` | spui | **typed framing exact on 748/748 across 82 indexes.** 93,726 elements, 1,183,787 recursive properties; independent Python/Rust digest agreement. Native runtime semantics remain open. See `re/spui-layout.md`. |
| `0x0469A3F7` | shader/material libraries | **Typed framing independently checked on all 12 header-1 fragment libraries and all 12 header-7 shader packs.** Header-0 material links also account for all companion RW4 states. Native material selection, fragment predicates and shader execution remain open. See `re/shader-resources.md`. |
| `0x4B9EF6DC` | Cell structure | **Measured fixed header and 40-byte entries.** Baked-creation references resolve original Cell meshes/skins; effect/simple-model kinds retain raw words. See `re/cell-source-assets.md`. |
| `0x1A99B06B` | bem | **decoded. 100% (1,733 of 1,733).** magic `0x48657ED3`, version 5, two `0xFE` RLE chunk buffers (128 and `unk1*472` bytes). See below. |
| `0x01A527DB` | snr (audio) | **container 100% (2,898 of 2,898).** 4-byte tag (version, flags, BE u16 rate), BE byte rate, BE declared length, then a non-self-delimiting codec payload. Codec decode out of scope. See below. |

## prop, type 0x00B1B104

The most common type in `Spore_Game.package`: 7,030 of 11,529 resources.

Status as of 2026-10-22: **primitive numeric interpretation independently checked
on 30,885 complete named lists of 30,889 PROP resources across all 82 installed
package indexes.** Four framing exclusions remain. The earlier six-package,
23,762-resource size-only result did not prove correct values or identifiers.
The retained compatibility framing rules below include hypotheses; exact EOF
alone does not establish them. Current corpus, native evidence and exclusions:
`re/prop-numeric.md`.

### Layout

`[MEASURED]` Confirmed against the public documentation and refined where the
documentation is a statement of intent rather than a byte layout:

```
u32 item count, BIG-ENDIAN
per item:
    u32 identifier, big-endian
    u16 type, big-endian
    u16 specifier, big-endian
    if specifier is an array marker:
        u32 element count, big-endian
        u32 element stride in bytes, big-endian
        count * stride bytes of elements
    else:
        one value, sized by type
```

### The finding that mattered: there are two array markers

`[MEASURED]` The public notes mention only `0x009C`. **`0x001C` is also an array
marker.** Recognising it took the exact-consumption rate from **52.82% to 94.99%** over
three packages.

It was found by dumping bytes at a failure rather than by guessing. A record with
`type = 0x39` (bbox) carried `specifier = 0x001C`, and the bytes immediately after were
`00 00 00 01 | 00 00 00 18`, that is `count = 1, stride = 0x18 = 24`. A bbox is 24 bytes,
so the record was an array of one bbox and `0x001C` was flagging it.

### The array header is count then stride

`[MEASURED]` The two words after an array marker are the element **count** and the
element **stride in bytes**. Reading the second as the count is wrong. Advancing by
`count * stride` raised the rate from 22.93% to 68.17% on its own.

### Mixed endianness

`[MEASURED]` Count, identifier, type, specifier, array header and string lengths are
big-endian. UTF-16 string **characters are little-endian**. `00 00 00 07` followed by
`44 00 65 00 74 00 ...` is `"Details"`. Reading characters as big-endian yields `一愀洀攀`
for `"Name"`, which is how this was caught.

`[CORRECTION]` INT32 (`0x0009`), UINT32 (`0x000A`), and FLOAT (`0x000D`) use
**four-byte big-endian values**, for singles and array elements. The old Rust
little-endian interpretation preserved framing but changed numbers. The native
Variant reader/writer and an independent 847,835-value bit digest confirm the
correction. Compound vector components retain raw little-endian bytes.

`[MEASURED]` The three words of a `key` value are **little-endian**. Read
big-endian, a key to a gmdl comes out with type `0xE5BCE600`
instead of `0x00E6BCE5`, which is the same four bytes byte-reversed. The cause is
plausible: the game stores a key as a raw struct copy, and the struct's members are in
native order while the surrounding fields are written by an explicit big-endian writer.

### A key's three words are not group, type, instance

`[MEASURED]` The fields are `(instance low 32 bits, type id, instance high 32 bits)`, and
the 64-bit instance is `high << 32 | low`. Reading them as `(group, type, instance)`
resolves nothing: across `Spore_Content`, all 4,403 model references and 306 texture
references point at instances that do not exist. Reassembled, all of them do.

One example, from the prop at `0x40666200D1B4BB56`. Its model slot holds the words
`D1B4BB56`, `00E6BCE5`, `40667100`. The instance is `0x40667100D1B4BB56`, a gmdl in the
same package. The prop's own instance shares the low half `D1B4BB56`, which is why the
first word looks like a group id.

### What a model prop contains, measured

`[MEASURED]` Prop idents are hashes and the strings behind them live in the executable,
so the names are not recoverable from the data. What each ident *does* is measurable, and
that is enough to use them. In `Spore_Content`, 1,162 props describe a model:

| Ident | Role | Props holding it |
|---|---|---|
| `0x00F9EFBB` | model, highest detail | 1,162 |
| `0x00F9EFBC` | model | 1,095 |
| `0x00F9EFBD` | model | 1,095 |
| `0x00F9EFBE` | model, lowest detail | 1,051 |
| `0x00F9EFB9` | float beside the chain | 1,162 |
| `0x00F9EFBA` | bounding box | 1,162 |
| `0xD124FCE4` | colour map | 153 |
| `0xD124FCE7` | colour map | 153 |
| `0x00B2CCCB` | parent prop | 1,162 |

The four model keys in one prop point at four gmdl resources whose instances differ only
in the low byte (`...7100`, `...6201`, `...6202`, `...6203`), which is a level-of-detail
chain rather than four parts. Both texture idents decode to colour images of the same
object, so neither is a bump map.

`[MEASURED]` The parent prop is always in a **different package**. All 1,009 model props
in `Spore_Content` that carry no texture name a parent that is absent from that package
and present in `PatchData` or `Spore_Graphics`.

`[MEASURED]` **Following the chain does not find a texture, and the reason is not the
chain.** With all 82 packages indexed, none of those 1,009 gains a texture from an
ancestor, and none has one in its `PatchData` copy either. Every one of the 1,162 props in
`Spore_Content` has a copy in `PatchData` (the patch overlay covers all of them), and the
copy carries no texture where the original has none. So those parts are not textured by
the prop graph at all. They are almost certainly painted by the creature layer, which
assigns a skin texture per creature rather than per part. That is Phase 3 work, and it is
the honest reason the count stays at 153.

### A type of 0 means unspecified, not absent

`[MEASURED]` Keys whose type field is `0x00000000` still carry a real instance. One
measured example is `0x00000000BE37F695`, which is an RW4 in `Spore_Graphics`. Matching on
the instance alone resolves it; matching `(0, instance)` does not. Doing this raised the
resolved model count in `Spore_Graphics` from 2,072 of 2,613 to 2,443, and the LOD slots
from 2,589 to 3,400.

`[MEASURED]` The remaining 170 references in `Spore_Graphics` are **dangling**: the
instances exist in no package in the install. `rustspore --locate <instance> [type]`
answers this question directly rather than leaving it to guesswork.

### Type sizes, measured

`[MEASURED]` A single `key` is **16** bytes, not the 12 that three u32 suggests; its
array stride is 12. Grid test: 12 bytes gives 6.51% exact, 16 gives 22.93% at that
stage. Array elements carry no single-value padding.

`[MEASURED]` The four-float types are **16 bytes with no single-value padding**:
`vector4` (`0x33`) and `colorRGBA` (`0x34`). An earlier revision padded `vector4` to
20 bytes; that was a drift artifact. The correction was forced by the last residual
class: when a `0x33` single was read as 20 bytes the walk overran the blob by exactly
4 bytes, and read as 16 bytes it lands exactly on the prop end after four valid floats.
`0x34` was already known to be 16; `0x33` is the same.

`[PARTIAL]` For other compound singles (`vector2` `0x30`, `vector3` `0x31`,
`colorRGB` `0x32`, `transform` `0x38`, `bbox` `0x39`), the compatibility walk
retains tight and four-extra-byte candidates. The independent named/count-bounded
numeric walk preserves these alternatives, without changing compound components.
The old 99.40% versus 100% size-only comparison does not prove a universal padding
rule; native per-type framing and value checks are still needed.

`[HISTORICAL]` Type frequencies from the older three-package size-only walk are
retained below. They are not the current typed numeric validation totals:

| Type | Name | Occurrences |
|---|---|---|
| `0x0020` | key | 37,622 |
| `0x000d` | float | 25,765 |
| `0x0009` | int32 | 9,485 |
| `0x0001` | bool | 7,470 |
| `0x000a` | uint32 | 7,286 |

### A superseded derivation, kept as a warning

An earlier revision gave `texts` (type `0x22`) a bespoke layout, derived from a prop
holding exactly one record of that type: `count`, then per entry a buffer size, two more
u32, and that many bytes. It appeared to work because it consumed the same number of
bytes as the array reading for that particular case.

`[INFERRED]` It was wrong. `texts` is an ordinary array with a stride of 520 bytes. This
is the clearest instance in the project of a derivation that fits the evidence available
at the time and is still not the real rule. Fitting one case is not confirmation.

### Legacy separator heuristic and a measured collision

`[MEASURED]` The variant layout inserts 4-byte separators between logical groups of
entries. The early decoder knew two of them by full byte match (`3c c4 12 00`,
`e4 0c a0 00`) plus the constant `01 00 00 00`. That was incomplete: the real set shares
a **low 16-bit word** of `12 00` or `a0 00`, with a varying high word
(`01 93 12 00`, `34 ab 12 00`, `bc a1 12 00`, and others). A separator is therefore any
4-byte candidate whose bytes at offset +2..+4 are `12 00` or `a0 00`, or exactly
`01 00 00 00` in the compatibility decoder. This is **a heuristic, not a sufficient
wire rule**: real FLOAT identifier `0195A000` in resource `02AE0C7E521D0174`
collides with it in both PatchData and Spore_Game. These records are explicitly
excluded from the independent numeric corpus, pending count-aware framing work.

### String arrays are length-prefixed, not fixed stride

`[MEASURED]` Arrays of `string8` (`0x12`) and `string` (`0x13`) are **length-prefixed
per element**, not a fixed `count * stride` block. Each element is `BE u32 length` then
`length` bytes (1 byte/char for `0x12`, 2 for `0x13`). This matters because the strings
are variable length (the UTF-16 string `"difficulty"` is 10 characters = 20 bytes, not a
fixed 16-byte stride). Reading them as fixed stride was a second residual cluster. The
array header (`count`, `stride`) is still present in the bytes, but the walk must skip
`count` individually length-prefixed strings rather than multiply.

### Named and anonymous compatibility recovery

`[HISTORICAL HYPOTHESIS]` The compatibility walk permits two entry shapes: **named**
`(id u32, type u16, spec u16, value)` and **anonymous** `(type u16, spec u16, value)`
with no id. A naive "anonymous if the first two bytes look like a type" rule fails: some
real named ids have a high word that is a type code (for example `0x0030xxxx`), so
forcing anonymous there drifts the walk by 4 bytes and cascades. The walk instead tries
**named first** and falls back to anonymous only when the named branch cannot consume the
rest of the blob exactly. This is a backtracking walk with memoisation on position; it is
deterministic because the named branch is always preferred. The `type 0x0000` quirk
(value size = spec bytes) is what makes a misread named entry look locally valid, so the
fallback can reach EOF, but does not by itself prove that an anonymous record
exists. In particular the old primitive eight-byte sizing could discard the next
real identifier and manufacture an anonymous path. The independent numeric walk
requires named entries, the declared count, four-byte primitives and exact EOF.

### Validation

`[VALIDATED]` `tools/prop_numeric_probe.py` and the Rust installed corpus test agree
on all named primitive identifiers, type/specifier/count fields and decoded bits
for **30,885 resources and 847,835 values**. All 82 package indexes were read;
30,889 PROP records were inventoried, including expansion/BoosterPack content.
Four explicit named-framing exclusions remain. The focused Rust suite passes
18/18 including that corpus. See `re/prop-numeric.md` for reproducible commands.

`[CORRECTION]` The old `prop_v5.py` 23,762-resource EOF figure is retained only as
historical size evidence. Its primitive singles incorrectly consumed eight bytes;
anonymous fallback could hide the lost next identifier. The tool is corrected to
four-byte primitive singles, but its old aggregate rate has not been recertified.

## RW4, type `0x2F4E681B`

`[MEASURED]` Every resource of this type, across all packages where it appears
(`Spore_Graphics`, `Spore_Content`), begins with the same 12-byte signature:

```
89 52 57 34 77 33 32 00 0d 0a 1a 0a
```

In ASCII: `0x89` (non-printable, same as PNG), `"RW4w32"`, `0x00`, then `0x0D 0x0A 0x1A 0x0A`.
The trailing four bytes are **identical** to the PNG signature trailer. This is a deliberate
magic number, not coincidence.

`[MEASURED]` The resources appear in `Spore_Graphics.package` (9,368) and
`Spore_Content.package` (1,131). `Spore_Game.package` carries none.

`[MEASURED]` Header structure, derived from statistical analysis of 500 samples:

```
offset   0: magic signature (12 bytes)
offset  12: 00 20 04 00   (u32, invariant)
offset  16: "454\0"       (platform code, invariant)
offset  20: "000\0"       (version, invariant)
offset  24: 00 00 00 00   (u32, invariant)
offset  28: subtype       (u32, two values: 0x00000001 or 0x00000004)
offset  32: u32           (34 distinct values, always equals offset 36)
offset  36: u32           (same as offset 32)
offset  40: u32           (0x04 when subtype=0x04, 0x10 when subtype=0x01)
offset  44: 00 00 00 00   (u32, invariant)
offset  48: off48         (u32, 203 distinct values)
offset  52: 0x00000098    (u32, invariant = 152)
offset  56: 00 00 00 00   (u32, invariant)
offset  60: 00 00 00 00   (u32, invariant)
offset  64: 00 00 00 00   (u32, invariant)
offset  68: off68         (u32, 202 distinct values)
offset  72: u72           (0x04 when subtype=0x04, 0x10 when subtype=0x01)
offset  76: off76         (u32, 228 distinct values)
offset  80: 0x00000004    (u32, invariant)
offset  84: 00 00 00 00   (u32, invariant)
offset  88: 0x00000001    (u32, invariant)
offset  92: 00 00 00 00   (u32, invariant)
offset  96: 0x00000001    (u32, invariant)
offset 100: off100        (u32, 32 distinct values)
offset 104: 0x00000004    (u32, invariant)
offset 108: 00 00 00 00   (u32, invariant)
offset 112: 0x00000001    (u32, invariant)
offset 116: 00 00 00 00   (u32, invariant)
offset 120: 0x00000001    (u32, invariant)
offset 124: 00 00 00 00   (u32, invariant)
```

`[MEASURED]` **Footer size invariant**: for every sample, `len(payload) - off76` equals
`off68` when `subtype = 0x04`, and the relationship `off68 = len(payload) - off76` holds
for every resource regardless of subtype. So `off76` is an offset to a footer section whose
size is `off68`.

`[MEASURED]` **Subtype 0x04 is uniform**: all 291 samples with `subtype = 0x04` have
`off48 = 324`, `off68 = 372`, and `off72 = 4`. The footer is exactly 372 bytes. This
suggests subtype 0x04 is a single well-defined object type (possibly a mesh or shader),
while subtype 0x01 is a family of related types with variable sizes.

`[INFERRED]` This is a **container format** for RenderWare 4 objects, not a single asset
type. The payload after the header is expected to carry an object type id that selects the
decoder (shader, mesh, etc.). The string literals `RWGOBJECTTYPE_*` and `RWCOBJECTTYPE_*`
extracted from the binary are the dispatch table. See `docs/engine-architecture.md`.

### Container structure and object dispatch

`[MEASURED]` The header fields that matter for dispatch are four little-endian
integers (cross-checked against the public SporeModder `rw4_base.py`, read for the
layout only):

```
offset 28: i32 rw_type_code      (1 = model, 0x04000000 = texture, 0xCAFED00D = special)
offset 36: i32 section_count
offset 48: i32 p_section_infos   (offset of the section-info table)
offset 68: i32 p_buffer_data     (offset of the BaseResource data region)
```

`[MEASURED]` The container is an array of **sections**, one per RenderWare object.
At `p_section_infos` there are `section_count` records of 24 bytes each:

```
p_data i32, field_04 i32, data_size i32, alignment i32, type_code_index i32, type_code i32
```

The object's payload is at `p_data` for ordinary objects, or at
`p_buffer_data + p_data` for `BaseResource` (`type_code = 0x10030`), and is
`data_size` bytes long. Between the section-info table and `p_buffer_data` sit the
sub-reference offset table and reserved runtime space.

`[MEASURED]` The object type codes and their decoders (from the reference):
`0x10030` BaseResource, `0x10004` SectionManifest, `0x10005` SectionTypes,
`0x10006` SectionExternalArenas, `0x10007` SectionSubReferences, `0x10008`
SectionAtoms, `0x20003` Raster, `0x20004` VertexDescription, `0x20005`
VertexBuffer, `0x20007` IndexBuffer, `0x20009` Mesh, `0x2000B` CompiledState,
`0x2001A` MeshCompiledStateLink, `0x200AF` BlendShapeBuffer, `0x70001`
KeyframeAnim, `0x70002` Skeleton, `0x70003` AnimationSkin, `0x7000C` SkinsInK,
`0x7000F` SkinMatrixBuffer, `0x80003` TriangleKDTreeProcedural, `0x80005`
BoundingBox, `0xFF0001` Animations, `0xFF0002` BlendShape, `0xFF0000` MorphHandle.

`[VALIDATED]` **Container 100% validated: 10,797 of 10,797.** `tools/rw4_validate.py`
checks the magic, the four header fields, every section-info record, that every
object's `[start, start+data_size)` range lies inside the file, that sections do
not overlap, and that the header, object data, section-info table, sub-reference
space and BaseResource buffer together tile the whole file with no gap larger than
16 bytes (alignment). Every resource passes.

`[VALIDATED]` **Object payloads 100% validated: 10,797 of 10,797.**
`tools/rw4_objects.py` reimplements the per-object reader for every type code in
the catalogue and requires each object to consume exactly its `data_size`. Each
object has a deterministic size rule: fixed sizes for Raster, VertexDescription,
CompiledState, SkinnedInK, KeyframeAnim, MorphHandle and BoundingBox; count-scaled
for VertexBuffer (`count * stride`), IndexBuffer (`count * 2`), Mesh,
Skeleton, Animations, BlendShape and the KD-tree; and a 32-byte fixed layout for
the one type code the public add-on does not cover, `0x7000B` (measured by
inspection). Zero objects overrun or underrun.

`[INFERRED]` The container carries geometry, materials, skinning and animation;
per-object semantics beyond extent (for example the meaning of each Raster flag)
are a modelling concern, not a parsing gap. Raster pixel data is already handled
by the separate `0x2F4E681C` decoder.

`[MEASURED]` **RW4 containers carry their own textures.** 8,915 of the 9,368 RW4 files in
`Spore_Graphics` hold a `0x20003` Raster object whose payload is a `BaseResource`. The
object is 32 bytes: format at 0, width (u16) at 12, height at 14, mipmap count (u8) at 17,
and the data object index at 28. Format codes are the same `DDS`/`D3DFORMAT` values the
RASTER type uses, measured across the install:

| Format | Files |
|---|---|
| DXT5 | 4,769 |
| DXT1 | 2,366 |
| `A8R8G8B8` | 1,555 |
| `L8` | 220 |
| DXT3 | 5 |

Every payload is large enough for mip level 0, and all 8,915 decode. This is what makes a
model shadeable without following a prop, and it is why the viewer can bind a texture to
an RW4 mesh with no material graph at all.

`[MEASURED]` **A `BaseResource` offset is relative to `p_buffer_data`; every other object's
offset is absolute.** The distinction cost a false negative: a comparison script that used
the offset as absolute decoded 8,915 textures without error but produced different pixels
on every one, because the wrong offset happened to land on other valid block data. Both
readings produce a plausible image, so only a digest comparison catches it.

`[CORRECTION]` Earlier black-box notes read offset 28 as a "subtype" with values 1,
0x04000000, 0xCAFEBABE. It is the RenderWare type code (model / texture / special),
which is why the payload shapes differ. The "type code table at offset 196" is the
`SectionTypes` object reached through `p_section_infos`, not a fixed offset.

## RASTER, type `0x2F4E681C`

`[MEASURED]` This type appears **only** in `Spore_Content.package` (2,954 resources). It
never appears alongside `0x2F4E681B` in the same package. The two types are **different
formats**, not variants of the same container.

`[MEASURED]` Header structure (28 bytes, little-endian):

```
offset  0: version    (u32, always 0x00000001)
offset  4: width      (u32)
offset  8: height     (u32)
offset 12: mipmaps    (u32)
offset 16: unknown    (u32, always 0x00000008)
offset 20: format     (u32: 0x31545844 = "DXT1", 0x35545844 = "DXT5", 0x15 = raw RGBA)
offset 24: data_size  (u32, size of the first/largest mip)
offset 28: texture data
```

`[MEASURED]` Three sub-formats exist:

| Format code | Name | Count (sampled) |
|---|---|---|
| `0x35545844` | DXT5 | 123 |
| `0x00000015` | Raw RGBA | 59 |
| `0x31545844` | DXT1 | 18 |

`[MEASURED]` Correction from full mip-level validation on 2026-10-23: each
level has a little-endian byte-length word. The first is at offset 24, with
pixels at 28; every later length immediately follows the previous pixels.
A 512 x 512 DXT5 texture with ten mips has 349,552 pixel/block bytes and
349,616 total bytes: 24 header bytes plus ten four-byte lengths. The extra
36 bytes are the nine later length prefixes, not a footer. Treating them as
pixels visibly corrupts minified texture sampling.

`[MEASURED]` `Raster::mip_levels` validates format-specific extents, partial
DXT blocks, level counts, bounds and exact consumption. Rust and independent
Python agree on all 5,931 standalone RASTER resources / 56,936 levels in
all 82 installed package indexes, totaling 2,804,636,276 pixel/block bytes.
Counts by format are DXT5 3,508, raw BGRA 1,789 and DXT1 634. The comparison
SHA-256 is `e6776d8fd5770179cecf2f98803902e8e2bab7f7b8ac7140c45e996baf8a5b9b`.
Evidence: `target/source-shaders/raster-mip-comparison-v1.json`;
reproduce with `raster_mip_verify` and `tools/raster_mip_probe.py`.

`[MEASURED]` The raw RGBA format uses BGRA byte ordering; channels must be swapped to
produce correct colours.

`[VALIDATED]` `tools/raster_extract.py` decodes DXT5, DXT1 and raw RGBA to PNG. Twenty
resources extracted from `Spore_Content.package` with zero errors; output PNGs are valid
512x512 RGBA images.

`[VALIDATED]` The Rust decoder in `crates/spore-core/src/raster.rs` matches the Python one
byte for byte on all 2,954 RASTER resources in `Spore_Content`, and on all 8,915 textures
embedded in RW4 containers in `Spore_Graphics` (`--raster-digest` and `--rw4-textures`
against `tools/raster_digest.py` and `tools/rw4_texture_digest.py`). It covers five
sub-formats: DXT1, DXT3, DXT5, `A8R8G8B8` and `L8`.

`[MEASURED]` DXT3 appears only inside RW4 containers (5 resources), never as a RASTER
resource. It carries 4-bit explicit alpha and a colour block that is **always** decoded in
four-colour mode, even when `c0 <= c1`. Swapping the endpoints to satisfy the
`c0 > c1` test, which is the obvious way to force that mode, also swaps the two blend
weights and shifts every colour: it produced a digest mismatch on all 5 resources.
`L8` is one byte per pixel replicated across RGB (220 resources, all inside RW4).

`[INFERRED]` This is a texture/image raster format, separate from the RW4 object
container. It was previously listed as an unexplained "adjacent type" to RW4; the
explanation is that they are independent formats with adjacent type ids in the DBPF
index.

## gmsh mesh, type `0x01C135DA`

`[MEASURED]` Appears in `Spore_Graphics.package` (2,000+), `PatchData.package` (14),
and `Spore_Pack_03.package` (46). Total sampled: 560 across three packages.

### Header (24 bytes)

`[MEASURED]` All fields confirmed across 560 samples:

```
offset  0: BE u32 = 1        (version)
offset  4: BE u32 = 1        (flags)
offset  8: LE u32 = 1024     (constant)
offset 12: BE u16 = N        (section count: position + attributes + index)
offset 14: BE u16 = 256      (constant)
offset 16: LE u32 = 3        (floats per position vertex)
offset 20: BE u16 = vcount   (vertex count, 348 distinct values)
offset 22: BE u16 = 12       (position stride in bytes = 3 floats)
```

`[MEASURED]` `stride_BE@22 = 12` is invariant across all 560 samples. `vcount_BE@20`
varies with 348 distinct values. All vertex floats are valid (small coordinates).

### Position data

`[MEASURED]` Immediately follows the header. `vcount * 12` bytes of little-endian
float3 XYZ coordinates. No section header.

### Attribute sections

`[MEASURED]` One or more attribute sections follow the position data. Each has a
10-byte header:

```
4 bytes: magic (varies by attribute type)
4 bytes: BE u32 = vcount
2 bytes: BE u16 = stride
vcount * stride bytes of data
```

`[MEASURED]` Observed magic values and their strides:

| Magic | Stride | Count (560 samples) | Likely content |
|---|---|---|---|
| `02 00 07 01` | 4 | 1543 | packed normals/colors |
| `03 00 07 02` | 4 | 1404 | packed normals/colors |
| `08 00 02 02` | 8 | 1873 | float2 UVs |
| `09 00 07 00` | 4 | 1308 | packed attribute |
| `0a 00 0a 00` | 4 | 1308 | packed attribute |
| `02 00 03 01` | 12 | 509 | float3 (tangent/binormal?) |
| `03 00 03 01` | 12 | 507 | float3 |
| `0c 00 03 00` | 12 | 507 | float3 |
| `0d 00 03 00` | 12 | 507 | float3 |

`[MEASURED]` The first byte of the magic appears to be a section index (02, 03, 08,
09, 0a, 0c, 0d, 0e). The third byte correlates with stride: 07 = stride 4, 03 = stride 12,
02 = stride 8, 0a = stride 4.

### Index section

`[MEASURED]` Magic: `15 00 06 08`. The BE u32 at +4 is a version discriminator.
The index section has two sub-types based on the stride field:

**Direct indices** (stride=2, 382 of 2000 samples):

The count/stride sub-header is followed directly by u16 LE index data.
For version 1, the sub-header is at +20/+22 within the 24-byte header.
For version 2+, the header is `16 + version*4` bytes, then the sub-header
(BE u16 count + BE u16 stride=2), then `count * 2` bytes of u16 LE indices.

**Matrices + indices** (stride=48, 1617 of 2000 samples):

The count/stride sub-header specifies `count` 48-byte 3x4 transformation
matrices. After the matrices, a v1-style index header follows:

```
+0:  BE u32 = 1 (version)
+4:  BE u32 = index_count
+8:  BE u16 = 2 (index stride)
+10: BE u16 = 0 (flags)
+12: index_count * 2 bytes of LE u16 indices
```

The 48-byte matrices contain rotation/transformation data (values like
-1.0, 0.0, 1.0 for 90-degree rotations). Matrix count ranges from 2 to 16.

**Version 1 header** (24 bytes):
```
4 bytes:  magic (15 00 06 08)
4 bytes:  BE u32 = 1 (version)
4 bytes:  LE u32 = 1024 (constant)
4 bytes:  zeros
2 bytes:  BE u16 = 1
2 bytes:  BE u16 = 0
2 bytes:  BE u16 = count (matrix count or index count)
2 bytes:  BE u16 = stride (2 or 48)
```

**Version 2+ header** (16 + version*4 bytes, then 4-byte sub-header):
```
4 bytes:  magic (15 00 06 08)
4 bytes:  BE u32 = version (2-11)
4 bytes:  constant (00 04 00 00)
(version-1) * 4 bytes: entries (00 00 0X 00 for X=1..version-1)
2 bytes:  BE u16 = count
2 bytes:  BE u16 = stride (2 or 48)
```

### File trailer (after index data)

`[MEASURED]` After the index data, a file trailer follows. The last 8 bytes are
always `ff ff ff ff ff ff ff ff`. Before that, a BE u16 value of unknown meaning.
The trailer size varies (56-206 bytes observed).

### Parse rate

`[VALIDATED]` **100% overall (2,482 of 2,482).** The full population spans
`Spore_Graphics.package` (2,422), `Spore_Pack_03.package` (46) and
`PatchData.package` (14). Every sample parses with all indices valid (all
`< vcount`) and the `0xff` x 8 footer present:
- Direct indices (stride=2): 382 samples, 100% valid
- Matrices + indices (stride=48): 1,617 samples, 100% valid
- Plus the version-1 direct-index samples; total 2,482.

`[MEASURED]` The last previously failing sample was not a different structure: its
index section magic is `14 00 06 08` (first byte `0x14`) instead of the usual
`15 00 06 08` (`0x15`), with an otherwise identical v1 header (version 1, count 8,
stride 48 matrices, then a 492-index v1 footer). Accepting both magics took the
rate from 1,999/2,000 to 2,482/2,482. The Rust port `crates/spore-core/src/gmsh.rs`
accepts both and has a regression test that flips the magic byte to `0x14`.

## anim, type `0xEE17C6AD`

[MEASURED] The corrected Rust parser and an independent Python decoder agree on
all 5,022 resources, 48,700 channels and 878,791 keyframe rows. They expose
731,075 positions and 731,383 rotations. All 82 package indexes open; ANIM
resources occur in PatchData (2,223), Spore_Game (2,091), Spore_EP1_Data (683)
and BoosterPack_01 (25).

The earlier sampled interpretation in this section was incorrect: descriptor
stride was called a keyframe count, +304 was called a channel count, and +296
was treated as a universal last-frame bound. Those interpretations are superseded
by the full-corpus measurements in [anim-channels.md](anim-channels.md).

[MEASURED] Header channel count and table pointer are at +324 and +328. CHAN
records store row count/data pointer at +212/+216 and descriptor count/pointer at
+220/+224. Each 32-byte descriptor contains flags, tag, component offset, row
stride, slot and three reserved words. Components occupy interleaved rows;
INFO gives the integer frame, POS gives float3, and ROT gives xyzw float4.
The parser follows the descriptors and preserves unsupported components and
raw rows instead of imposing one fixed layout.

[MEASURED] All 5,022 per-resource FNV digests match between independent readers,
including source float bits, selectors, descriptors and complete raw rows.
Four source position keys are nonfinite and produce explicit sampling errors;
1,050 rotation keys require normalization. Step and linear/slerp policies are
available with validated sample times and quaternion norms.

[UNKNOWN] Faithful authored curves, selector-to-rig binding and procedural gait
retargeting remain unresolved. Successful parsing and generic sampling do not
establish native animation behaviour. [anim-rig-mapping.md](anim-rig-mapping.md)
records the measured join constraints and negatives.

```powershell
tools\build.bat run -p spore-bevy --example anim_verify -- --digest target/anim-rust-digest.txt "D:/Games/SPORE Collection"
python tools/anim_channel_probe.py --out target/anim-channel-probe.json --digest-out target/anim-python-digest.txt
```

## blockdata, type `0x5CAA5E28`

> The notes below are historical measurements and hypotheses. Source instructions
> supersede the proposed 12-byte records and quadtree interpretation: see
> [the native sample trace](blockdata-native-sample-static-v1.md).

`[MEASURED]` Appears in `PatchData.package` (1,772) and `Spore_Game.package`
(1,326). Full population **3,098**. No magic string. Size distribution: 36
(424), 48 (660), 84 (930), 228 (738), 804 (256), 3108 (74), 12324 (14), 49188
(2).

### Bounding box

`[MEASURED]` All samples begin with 6 little-endian floats forming a bounding box:

```
+0:  LE float: min X
+4:  LE float: min Y
+8:  LE float: min Z
+12: LE float: max X
+16: LE float: max Y
+20: LE float: max Z
```

`[MEASURED]` For the 36-byte samples, the remaining 12 bytes are 3 u32 zeros.
Example: min = (-0.1, -0.1, -0.1), max = (0.1, 0.1, 0.1), then 3 zeros.

### Flags and additional data

`[MEASURED]` For larger samples (48+ bytes), after the 6 bounding box floats:

```
+24: LE float (unknown, varies)
+28: 4 bytes of flags (low byte pattern 0x04; full 8-byte context spans +28..+35)
+32: 4 more bytes of flag/level context (NOT always zero)
+36: variable data
```

`[MEASURED]` The data size is `36 + 12 * k` where `k` is always a power of 4:
`k in {1, 4, 16, 64, 256, 1024, 4096}` (sizes 48, 84, 228, 804, 3108, 12324,
49188). `k = 4^L` for a level `L`, which is the signature of a quadtree (4
children per node). Across the full population of **3,098** samples the distinct
flag values at +28 are exactly `{0, 0x04, 0x404, 0x40404, 0x4040404}`, and the
level `L` correlates with the number of `0x04` bytes present in the +28..+35
context for the regular-grid cases (48/84/228: `L = 0/1/2`, field@32 = 0).

`[MEASURED]` **The flag field alone does NOT determine `k`.** The flag context
`0x04040404` at +28 is shared by the four largest sizes (804, 3108, 12324,
49188) which have `k = 64, 256, 1024, 4096` respectively. Their discriminator in the measured corpus is
the field at +32: `0`, `0x04`, `0x404`, `0x40404`. The relationship between
the complete 8-byte context and `k` is described by the empirical closed form
below. This correlation alone does not establish the meaning of the context,
a quadtree, or the behavior of unmeasured inputs.

`[MEASURED]` The post-header region is **not** uniform 3-u32 groups. Dumped
representatives show:
- 48-byte (`k=1`): one 12-byte record of 3 LE floats (e.g. `512.0, ~1.25e-38,
  -0.002`), plausibly one leaf cell position.
- 84-byte (`k=4`): four 12-byte records of 3 LE floats (positions around 512..850).
- 228-byte and up: the 12-byte records are **not** clean floats; they read as u16
  sequences (e.g. `0x2111, 0x7121, 0x5122, ...`), suggesting quadtree node codes
  or leaf index lists rather than XYZ positions. The earlier "high byte 0x44"
  observation was a coincidence on one specific 84-byte sample.

`[MEASURED]` **The size is a closed form of the 8-byte flag context.** Let `c`
be the number of bytes equal to `0x04` in bytes 28..36 (the context spans both
the +28 and +32 dwords). Then:

```
total_size == 36 + 12 * 4^(c - 1)     (c >= 1)
total_size == 36                      (c == 0)
```

The eight observed size classes map one-to-one onto `c = 0..7`:
`36, 48, 84, 228, 804, 3108, 12324, 49188`. The `+32` dword is what
discriminates the four largest sizes that share `flags@28 = 0x04040404`.

`[VALIDATED]` `tools/blockdata_validate.py` checks the closed form against the
full population: **3,098 of 3,098** samples match exactly, zero mismatches. The
context distribution is `c=0..7` -> `424, 660, 930, 738, 256, 74, 14, 2`. The
Rust port `crates/spore-core/src/blockdata.rs` reproduces the rule and passes 8
unit tests, including a context that spans both dwords.

`[INFERRED]` This is a **terrain / LOD quadtree block**. The bounding box bounds
a region; `L = c - 1` is the subdivision depth; the 12-byte records are the
`4^L` quadtree cells. Each subdivision level quadruples the cell count.

`[UNKNOWN]` The exact 12-byte record semantics (node child-mask vs. terminal leaf
position/attributes) are not decoded. The structure is fully consumed (every byte
accounted for), but the meaning of the 3 u32 per record remains open.

## Cell definition, type `0xDFAD9F51`

`[MEASURED]` All 446 samples are exactly 796 bytes. Structure:

```
offset  0: LE u32 referenced Cell structure ID
offset  4: UTF-16LE string (null-terminated)
remainder: Cell data with unresolved fields, not verified null padding
```

`[MEASURED]` The string is encoded as UTF-16LE (little-endian 16-bit characters).
Maximum string length: (796 - 4) / 2 = 396 characters. Observed strings include
"PLACEHOLDER_Maa", "PLACEHOLDER_Jawhead", "PLACEHOLDER_Ducky". Many samples
have empty strings (all zeros after the hash).

`[MEASURED]` The previous string-table identification was incorrect. In PatchData,
222/223 Cell records resolve the first word to a `0x4B9EF6DC` structure resource.
90 definition rows name baked creations: 65 rows resolve 28 unique available creations
with model props, GMDL geometry, own skin atlases and thumbnails. The other 25 rows
reference 17 low IDs with no matching model/art under any prefix in all 82 indexes.
One additional definition names structure zero. These unresolved records are explicit
results. For example Cell `7D00E384` links structure `285A339B`, which names
creation `40616200780A9EAF`. The old `string_table` module reads only the embedded
label; it is not a decoder of the complete Cell definition or a localization table.
Remaining behaviour fields are `[UNKNOWN]` pending independent/native validation.

## audio event, type `0x02B9F662`

`[MEASURED]` 16,841 resources across `PatchData.package` and `Spore_Audio2.package`.
No magic string. The format uses **FNV-1 32-bit hashes** as string keys and is now
**decoded to 100%**: every one of the 16,841 resources parses to exactly the last
byte, with zero overruns. The Python reference validator is `tools/audio_event_v4.py`;
the Rust port is `crates/spore-core/src/audio_event.rs`.

### Layout

`[MEASURED]` All multi-byte fields are big-endian (header, hash, type, flags, array
count/stride, and string lengths). String *characters* are little-endian (UTF-16LE
for type `0x13`), matching the prop convention. Value bytes (int/float) are decoded
little-endian, matching prop; this is `[INFERRED]` pending a dedicated measurement
of a known scalar value.

```
offset  0: BE u32 entry_count  (ADVISORY; the walk proceeds to the buffer end)
offset  4: entries, each:
              BE u32 hash        (FNV-1 of the property name)
              BE u16 type
              BE u16 flags
              payload, sized by flags/type:
                  if flags in {0x009C, 0x001C}:   # array marker
                      BE u32 count
                      BE u32 stride
                      if type in {0x12, 0x13}:  count length-prefixed strings
                          (each: BE u32 length then length * width bytes)
                      else:                     count * stride bytes of elements
                  elif type in {0x12, 0x13}:       # single string
                      BE u32 length then length * width bytes
                  elif type == 1:                  # bool
                      1 byte value
                  else:                            # scalar
                      4 byte value
              4-byte separators may appear between entries (see below)
```

### The finding that mattered: entries are variable length

`[MEASURED]` The early decoder assumed every entry was a fixed 12 bytes
(hash + type + flags + 4-byte value). That capped the exact rate at **50.6%**
(8,515 of 16,841). Three rules, all found by dumping bytes at failures rather than
guessing, took it to 100%:

1. **A bool value is 1 byte, not 4.** Type `1` (`is3d` etc.) carries a single
   trailing byte. Reading 4 bytes here overruns the blob by exactly 3 bytes. This
   was the single largest residual cluster.
2. **The `flags` field carries the same two array markers as prop** (`0x009C`,
   `0x001C`). An entry with one of these flags is an array: `BE u32 count` then
   `BE u32 stride` then `count * stride` bytes, or (for string types) `count`
   length-prefixed strings. The old "samples section / codec section" notes were
   really just array-flagged entries: `flags = 156` is `0x009C` in decimal.
3. **String arrays are length-prefixed per element**, not a fixed
   `count * stride` block. Each element is `BE u32 length` then `length * width`
   bytes (width 1 for `0x12`, 2 for `0x13`). The array header is still present in
   the bytes, but the walk skips `count` individually length-prefixed strings.

### Separators

`[MEASURED]` Between logical groups, 4-byte separators appear, detected by the same
low-word rule as prop: bytes at offset +2..+4 are `12 00` or `a0 00`, or the value
is exactly `01 00 00 00`. They are skipped as 4-byte no-ops during the walk.

### Known FNV-1 hash-to-string mappings

`[MEASURED]` Confirmed against the engine string literals:

| Hash | String | Meaning |
|---|---|---|
| `0x5F6317D5` | `parent` | Parent reference (appears in most entries) |
| `0x25DF0108` | `gain` | Audio gain/volume |
| `0x12D2A4D4` | `is3d` | 3D positional audio flag |
| `0x701ED91E` | `samples` | Audio sample references |
| `0xBA03D0CD` | `codec` | Audio codec identifier |

### Validation

`[VALIDATED]` `tools/audio_event_v4.py` (exact-consumption forward walk) parses
**16,841 of 16,841** audio event resources exactly across `PatchData.package` and
`Spore_Audio2.package`. The walk is deterministic (no backtracking, no
named/anonymous ambiguity), so the authoritative signal is simply "did the scan
reach the buffer end". The Rust port `crates/spore-core/src/audio_event.rs` mirrors
the same rules and passes 9 unit tests covering bool, int, string, fixed-stride
array, length-prefixed string array, separator, and the second array marker
`0x001C`.

`[MEASURED]` This is an audio event/sound configuration table. The property names
(parent, gain, is3d, samples, codec) are all audio-related, defining sound events
with their properties and sample references. Type `0x03F51892` is a related variant
using the same FNV-1 hash scheme and the same entry rules; it was validated
separately by `tools/audio_event_variant.py` and reaches **100% (90 of 90)** under
the identical exact-consumption walk. The Rust `audio_event::parse` is type-agnostic
and covers both `0x02B9F662` and `0x03F51892`.

Tools: `tools/audio_event_v4.py` (validator), `tools/audio_event_parse.py`,
`tools/audio_event_v2.py` (superseded), `tools/type_02b9_deep.py` (superseded).

## gmdl model, type `0x00E6BCE5`

`[MEASURED]` 4,209 resources, all in `Spore_Content.package`. All RefPack compressed.

`[MEASURED]` **Corrected header layout.** The field at offset 4 is a
**big-endian reference count**, not a flags field. When it is non-zero, it is
followed by `ref_count` 12-byte resource references (4-byte type, 8-byte key; the
type is `0x2F4E681B`, a RenderWare 4 object). This shifts every later field. The
header size is therefore `64 + ref_count*12`:

```
offset  0: LE u32 version = 8
offset  4: BE u32 ref_count            (0..11 observed)
offset  8: refs: ref_count * 12 bytes  (type 0x2F4E681B + 8-byte key)
offset  8+ref*12: LE u32 mesh_count    (1, 2 or 3 observed)
offset 12+ref*12: 7 LE floats bbox
offset 40+ref*12: LE u32 triangle_buffer_count
offset 44+ref*12: LE u32 primitive_type = 4
offset 48+ref*12: LE u32 indices_count
offset 52+ref*12: LE u32 indices_bits = 16
offset 56+ref*12: LE u32 buffer_size   (= indices_count * 2 for mesh_count=1)
offset 60+ref*12: LE u32 pad
offset 64+ref*12: index data (u16 LE), buffer_size bytes
```

`[MEASURED]` The old "flags = 0" reading was correct only because `ref_count` is
usually 0. Full-population `ref_count` distribution: 0 (2,699), 1..11 (1,510
total). `mesh_count` distribution: 1 (2,423), 2 (1,715), 3 (70).

`[MEASURED]` **Full structure** (cross-checked against the public SporeModder
Blender add-on's `gmdl_importer.py`, read for the byte layout only; no code
copied). After the header and the referenced-file keys, the layout is:

```
mesh_count                     i32
bbox                           6 f32, then radius f32
triangle_buffer_count          i32
  per buffer:  primitive_type i32, indices_count i32, indices_bits i32, buffer_size i32
               indices: indices_count * (2 if bits==16 else 4) bytes
vertex_format_count            i32
  per format:  element_count i32, then element_count * VertexElement
               VertexElement = <hhbbbbi (12 bytes):
                   stream i16, offset i16, type i8, method i8, usage i8, usage_index i8, rw_decl i32
vertex_buffer_count            i32
  per buffer:  format_index i32, vertex_count i32, buffer_size i32
               vertices: vertex_count * stride bytes,
                   stride = sum of D3DDECLTYPE sizes for the referenced format
mesh_count times:  t_index i32, v_index i32
mesh_count times:  material_id i32
trailing section (skinned models):
  u32 zero, material_count i32, per material: shdata_count i32 then shader-data blobs
      (0x20D textures: count then count * 24 bytes; 0x210: 20 bytes; 0x218: 0 bytes;
       0x238: 1 byte)
  skeleton: count1 i32, count1*8 bytes, part_count i32, per part two 0x38 transforms,
      field_70 i32, key (3 i32), var i32, and if var!=0 a bone_count i32 + bone_count*0xA0 bytes
  12-byte footer (00 00 00 00 | ff ff ff ff | 00 00 00 00)
```

`[MEASURED]` The vertex stride is **not fixed**: it is the sum of the D3DDECLTYPE
sizes of the format's elements (24 for position+normal, 36, 28, etc.). This is why
the earlier single-stream formula only fit a subset.

`[VALIDATED]` **Exact rate: 4,209 of 4,209 (100%).** `tools/gmdl_full.py`
implements the structure above and consumes every byte for every sample. Zero
core-structure errors, zero size mismatches. The 12-byte footer is fixed.

`[CORRECTION]` Shader data `0x238` is a **single byte**, not a length-prefixed
blob. Reading it as `u32 size` then `size` bytes is what held 132 skinned models
at 96.86%. The byte abuts the following `count1` field, so the two read together
as a plausible-looking size: the first sample I examined had `0x11` followed by
`count1 = 1`, which reads as `0x111 = 273`. The tell was arithmetic rather than
semantics: the part records align on a fixed 132-byte stride (`0x38` + `0x38` +
4 + 12 + 4) backwards from the footer, which put `part_count` three bytes past
where the bogus size had left the reader.

`[MEASURED]` Part record size, verified by walking backwards from the footer in a
skinned sample: 132 bytes when `var == 0`, with the 12-byte key of all `0xFF`
sitting at record offset 116 and the footer immediately after the last record.

`[INFERRED]` This is the higher-level model format referenced by the SporeModder
Blender Addon documentation. It contains mesh geometry (indices + vertices) with
a bounding box and external resource references. The format is related to but
distinct from the gmsh mesh format.

Tool: `tools/gmdl_full.py` (validator), `tools/gmdl_validate.py`,
`tools/gmdl_trailer.py`, `tools/gmdl_sweep.py`, `tools/gmdl_quick.py`,
`tools/gmdl_parse.py`, `tools/gmdl_deep.py`.

## Other unknown types (structural notes)

`[MEASURED]` Type census identified 80 distinct types across 91,267 resources.
Beyond the formats documented above, several more have identifiable structure:

**Type `0x030BDEE3`** (1,641 resources, Spore_Content): **100% decoded.**

- `[MEASURED]` All big-endian. Layout:

```
offset  0: u32 version = 10
offset 84: u32 ascii_length
offset 88: ascii tag (ascii_length bytes), e.g. "tag:spore.com,2006:AssembledContent"
then:
    u32 count_a, then count_a * (u32 utf16_length + utf16_length*2 bytes)
    u32 count_b, then count_b * 4-byte hash
    one 4-byte hash
    one optional extra 4-byte hash (the 139-byte variant)
```

- `[MEASURED]` The `0x23` byte at +87 is the ASCII string length (35) and also
  reads as `#`, which is where the earlier `#tag:` note came from; the tag itself
  is `tag:spore.com,2006:AssembledContent`. The body carries resource hashes with
  `0xFFFFFFFF` "no resource" sentinels.
- `[VALIDATED]` `tools/type_030b_validate.py` walks the whole structure to EOF:
  **1,641 of 1,641 (100%)**. Size classes 135/139/159/163/... differ only in the
  ASCII tag length and the UTF-16 string list.
- `[INFERRED]` Content assembly manifest.

**Type `0x250FE9A2`** (.spui, Spore user-interface layout): **typed framing decoded on
748/748 resources across all 82 package indexes.** Earlier work used 601 resources
from PatchData/Spore_Graphics.

- `[MEASURED]` Little-endian. Magic `0xE3FE3FB8` at +0 (bytes `b8 3f fe e3`; the
  earlier "BE 0xB83FFEE3" reading had the endianness backwards), version u16 at +4
  (all samples version 3), then four u16 counts: image, atlas, hit-mask, proxy.
- `[MEASURED]` Then: `imageCount` resource keys (u32 instance, type, group),
  `atlasCount` resource keys, `hitMaskCount` hit masks, and `proxyCount` i32 class
  ids. A hit mask is a u8 `useRle` flag (version 3+); if set, `i32 width`,
  `i32 height`, `i32 wordCount`, then `wordCount*2` bytes; otherwise a resource key.
- `[MEASURED]` The remaining bytes are a self-typed **element section**. The current
  Rust/Python decoders read 93,726 elements and 1,183,787 properties, including nested
  structure arrays, literal text and localization references. Typed re-encoding matches
  every source byte. Designer class defaults and UI runtime semantics remain open.
- `[VALIDATED]` `tools/type_250f_validate.py` parses the container for
  **601 of 601 (100%)**. The earlier "hash table with open addressing" reading was
  a misreading of the hit-mask/key/proxy arrays as one record table.
- `[MEASURED]` `re/spui-layout.md` records the full-population typed-body checks,
  independent digest agreement and native reader/writer corroboration.

**Type `0x1A99B06B`** (.bem, creature body editor model, 1,733 resources):
**100% decoded.**

- `[MEASURED]` Little-endian. Magic `0x48657ED3` (bytes `d3 7e 65 48`; the earlier
  "BE 0xD37E6548" reading was the same bytes with the endianness reversed), then
  `version` (3 < version <= 5), `unk1` (<= 1024, the part count), `unk2` (<= 128).
- `[MEASURED]` Then two RLE chunk buffers, in order: one with decompressed size
  **128**, then one with decompressed size `unk1 * 472`.
- `[MEASURED]` RLE: repeat `u16 chunk_size` (1..2048) then `chunk_size` bytes.
  Inside a chunk, byte `0xFE` is an escape; the next byte is a count (0 = a literal
  `0xFE`, else repeat the previous byte count times). Chunks are read until the
  requested output length is reached.
- `[VALIDATED]` `tools/type_1a99_validate.py` walks both sections to exact EOF for
  **1,733 of 1,733 (100%)**. Every sample is version 5.
- `[CORRECTION]` My first transcription used 68 bytes for the first section on
  version 5 and validated only 65.1%. Decompressing with 68 stops short, and the
  second section then resynchronises on a chunk boundary, which hid the error.
  Measuring both sizes across the population showed 128 succeeds for every sample
  and 68 fails for 605, so 128 is the version 5 size.
- `[MEASURED]` This is the body-editor sibling of `0x0F43029A` (.creaturedata).
  Both are editor saves built from the same `0xFE` RLE chunk scheme, with
  different following sections (128 + `unk1*472` here, then typed row/array
  sections in CreatureData). Both current first headers decode to 128 bytes.

**Type `0x0F43029A`** (.creaturedata, 1,365 resources across base and expansion):
**Full native header and typed rows independently checked.**

- `[MEASURED]` Little-endian. Magic `0xABB455B7` at +0 (bytes `b7 55 b4 ab`; the
  earlier "BE 0xB755B4AB" reading had the endianness backwards), version at +4
  (9 or 10), then 3 counts (version 9) or 4 counts (version 10) as i32.
- `[MEASURED]` Then an **RLE chunk buffer** with decompressed length `0x80`
  for versions 9/10: repeat `u16 chunk_size` then `chunk_size` bytes. Byte `FE`
  escapes a count: zero emits FE; nonzero repeats the last decoded byte. State
  persists across chunks; dangling escapes and oversized repeats fail. The
  earlier 68-byte retained prefix and chunk-first-byte assumption were wrong.
  Native reader/writer and independent corpus evidence are in
  [creature-header-native.md](creature-header-native.md).
- `[MEASURED]` Then, in order: `counts[0] * 0x8C` bytes (data0), `counts[1] * 4`
  bytes (capability ids), `counts[1] * 1` (capability numbers), `counts[2] * 4`
  (morph ids), `counts[2] * 4` (value1 f32), `counts[2] * 4` (value2 f32), 4 bytes
  (field110), and for version 10 `counts[3] * 0x38` bytes (data3).
- `[MEASURED]` The independent full-header probe completes **1,365/1,365**:
  1,022 version 9 and 343 version 10. The old 68-byte framing probe still found
  correct row offsets because every source header occupies one encoded chunk;
  it did not validate the complete decoded header.
- `[MEASURED]` This is the creature editor save. It shares its editor-save family
  with `0x1A99B06B` (bem) but has a different body layout.

#### The rig table: `data0` rows

`[MEASURED]` Each `data0` row is **0x8C bytes** and describes one part: where it sits and
which prop it is. The layout, by offset:

| Offset | Contents |
|---|---|
| 0 | `u16` part index |
| 2 | `u16` generated joint/node reference; complete mapping remains open |
| 4 | `u16` parent row, `FFFF` for root |
| 6 | `u16` reciprocal symmetric peer, `FFFF` for none |
| 8, 10 | two `u16`; byte +0B is the observed source class |
| 12, 14 | `u16` starts in the capability and morph arrays |
| 16, 17 | `u8` capability and morph counts for this row |
| 18 | retained raw `u16`, zero in the measured source population |
| 20..44 | six floats, a bounding box (min then max) |
| 44 | one float, `1.0` in most rows |
| 48..84 | 3x3 rotation, row-major |
| 84..96 | translation |
| 96..108 | three floats, zero in every sample |
| 108..128 | five floats, meaning not established |
| 128 | `u32`, zero in every sample |
| 132 | `u32` instance **high** half of the part prop |
| 136 | `u32` instance **low** half of the part prop |

`[MEASURED]` The 3x3 at 48 is a rotation and not coincidence: it is orthonormal (row norms
1.0, pairwise dots 0.0, tolerance 0.01) on **31,704 of 31,704 rows** across the 1,022
creature files in `Spore_Content`. The translation is model scale, with a median largest
component of 0.90 and a maximum of 17.25.

`[MEASURED]` The words at 132 and 136 are the part's prop instance, high half first. Every
one of the 31,704 rows resolves to a prop in the install, all of them in `PatchData`.

`[MEASURED]` Each part prop names one model, and 27,959 of the 31,704 resolve to a model
that exists (**88.2%**). By creature, **773 of 1,022** files have every part's model
present. The other 3,745 references point at instances that are in no package in this
install.

`[MEASURED]` Transforms are **absolute, not hierarchical**. For the 4-part creature
`0x406B62000660EACA`, the assembled bounds are x 0.156..0.627, y -0.643..-0.039, z
0.173..1.117 (radius 0.472), and the model the game ships pre-merged for the same instance
has x 0.119..0.615, y -0.654..-0.008, z 0.126..0.934 (radius 0.404). Composing a parent
chain is not needed to get parts into the right volume.

`[MEASURED]` The assembly reproduces the merged model closely. Comparing vertices by
nearest-neighbour distance, over the **93 creatures in `Spore_Content` that have both a
complete rig and a merged model**:

| Statistic | Mean distance as a share of the merged model's diagonal |
|---|---|
| minimum | 0.9% |
| 10th percentile | 1.3% |
| median | 2.1% |
| 90th percentile | 5.6% |
| maximum | 10.4% |

Two independently tessellated versions of the same surface sit a couple of percent apart,
so the rig is not merely placing parts in the right volume, it is reproducing the shape.

`[MEASURED]` **A creature's skin is keyed by its instance low half, and no prop names
it.** For creature `0x40666200D1B4BB56` the install holds six RASTERs that share the low
half `D1B4BB56` and differ only in the high half:

| Instance | Size | Format | Reading |
|---|---|---|---|
| `0x40662900D1B4BB56` | 512x512 | DXT5 | skin atlas: eyes, mouths, body patches |
| `0x40662901D1B4BB56` | 256x256 | `A8R8G8B8` | smaller variant |
| `0x40662800D1B4BB56` | 256x256 | DXT5 | colour map |
| `0x40662801D1B4BB56` | 256x256 | DXT5 | colour map |
| `0x40662C00D1B4BB56` | 256x256 | DXT1 | detail or specular |
| `0x40662C01D1B4BB56` | 256x256 | DXT1 | detail or specular |

`[MEASURED]` **366 of the 1,022 creatures in `Spore_Content` have such a set**; the other
656 have none, and their skin must be generated from the creature's paint parameters rather
than loaded. Among the 366, the largest raster is always the atlas, so "largest wins" is a
measured rule, not a guessed ident. Decoding it gives an image of eyes, mouths and body
patches laid out for the part UVs, which is what makes a part render with a face.

`[MEASURED]` A creature's parts are untextured in their own containers: the RW4 for part
`0x4060600006B00D54` holds no Raster object. So the skin atlas is the only texture a
creature has, and every part's UVs point into it.

`[MEASURED]` **The paint palette is nine floats at offset 32 of the decompressed header
buffer**, three RGB triples in 0..1. Established by checking the palette against the shipped
skin of creatures that have one: across 40 such creatures, all three palette colours appear
(within 0.08 RGB) somewhere in the skin for 30, and at least one for 37. So the floats are
colours the creature was authored with, not some other quantity that happens to be in 0..1.

`[MEASURED]` The palette is a colour set, not a flat palette for the shipped skin. Over the
same 40 creatures, only a median 14 percent of a skin's opaque pixels lie within 0.08 RGB of
a palette colour (p10 1 percent, p90 30 percent, best 59 percent). The atlases are painted
with shading and intermediate tones on top of the three colours, and hold 169 to 2,312
distinct colours each.

`[MEASURED]` For a creature with no shipped skin, the palette is all there is, and
`spore_core::skin` paints it over the mesh's own UVs. Measured on five creatures that have
both a shipped skin and a resolvable mesh, the shipped alpha mask lies inside the generated
one: IoU equals the shipped mask's area fraction to two decimals (0.098/0.092, 0.193/0.186,
0.268/0.269, 0.271/0.275, 0.348/0.349). The rasteriser therefore lands on the artist's
islands, including the negative `v` range, which is shifted by a whole tile. The generated
atlas covers more of the square than the artist painted, which is intended: the rest of the
body still needs a colour.

`[MEASURED]` A creature's UVs sit in one tile with `v` negative: measured ranges are
`u 0.005..0.998`, `v -0.998..-0.006` across sampled creatures, and 0 triangles straddle a
seam after the tile shift. 74 to 96 percent of triangles have a usable UV, the rest being
degenerate in UV space.

`[CORRECTION]` The RLE decompressor in `crates/spore-core/src/creature_data.rs` was wrong:
it emitted `current[0]` for every literal byte and never advanced its index, so the
"decompressed" buffer was one byte repeated. The parse still consumed exactly, because the
decompressed buffer was kept only for information and nothing read it. It surfaced the
moment the palette was read out of it and came back as -8e18. The buffer is now decoded
properly, and a unit test covers a chunk with a repeat escape.

`[CORRECTION]` An earlier revision of this document said the silhouette did not match,
citing an intersection over union of 0.178 between two screenshots. That comparison was
worthless: the viewer normalises each mesh by its own radius and centres it on its own
bounds, so two identical shapes render at different sizes. The pixel metric measured the
framing, not the geometry. Comparing vertices needs no camera and no normalisation, and it
gives the answer above.

**Type `0x02D5C9AF`** (1,631 resources, Spore_Content): **100% decoded.**

- `[MEASURED]` All big-endian. The format is a two-section hash-pair table:

```
u32 version = 4
section 1: u32 count_a, then count_a * 12-byte records
section 2: u32 count_b, then count_b * 12-byte records
```

Each 12-byte record is `(hash, 0x02E1A75D, value)`; the middle word `0x02E1A75D`
is a constant record-type marker.

- `[VALIDATED]` `tools/type_02d5_validate.py` walks `version` then repeated
  `(count, count*12 bytes)` sections to EOF. **1,631 of 1,631 (100%)** consume
  exactly, and every sample has exactly **2 sections**.
- `[INFERRED]` Reference/dependency table linking resource hashes.

**Type `0x01A527DB`** (.snr, audio; 2,898 resources, PatchData/Spore_Audio2):
**container decoded, codec payload out of scope.**

- `[MEASURED]` The first four bytes are a **tag**, not a u32: byte 0 is a version
  (1, 2, 4, 5 observed), byte 1 is flags (`00` or `04`), and bytes 2-3 are the
  sample rate as a big-endian u16 (`ac44` = 44100, `5622` = 22050, `bb80` =
  48000). Reading the tag as a little-endian u32 gives `0x44AC0404` for the most
  common form, which is where the earlier `0x0400AC44`-style notes came from.
- `[MEASURED]` Two header shapes:
  - **8 bytes**: tag then one BE float (2.6-ish). No payload. 11 resources.
  - **12 bytes or more**: tag, then BE byte rate at +4 (88200, 176400, 470400,
    705600, and 15435/17640 for the low-rate tags), then BE declared length at
    +8, then the payload.
- `[MEASURED]` The payload is **not self-delimiting**: it carries no internal
  length, so its extent is the resource boundary. The declared length at +8 is a
  decoded-PCM byte count, not a stored-byte count. It equals `size - 8` for
  **2,776** resources (the uncompressed-PCM cases) and does not for the rest.
- `[MEASURED]` 83 resources set bit `0x20000000` in the byte-rate field and use a
  variant header whose extra fields are not decoded; 28 more match neither rule.
- `[VALIDATED]` `tools/type_01a5_validate.py` consumes the container for
  **2,898 of 2,898 (100%)**.
- `[UNKNOWN]` The payload codec. The high-entropy bodies are an EA audio codec
  stream. Decoding them is audio work, outside this project's geometry and UI
  scope, so it is not attempted. 10 distinct tags appear, so there is more than
  one codec/rate combination in the install.

**Identifiable text formats** (no binary decoding needed):
- XML: types `0x2B978C46` (94), `0x3D97A8E4` (26), `0x3C7E0F63` (86),
  `0x0330A65E` (10), `0x24682294` (8), `0xCF6C21B8` (5), `0x476A98C7` (4),
  `0x2399BE55` (2), `0x438F6347` (2)
- Lua scripts: type `0x024A0E52` (101), starts with `DeclareState CIS...`
- PureData patches: type `0x617715D9` (289), starts with `#N canvas`
- Plain text: types `0x497767B9` (123), `0x03055F61` (50), `0x0248F226` (4)
- WAV audio: type `0x3F9C28B5` (1), RIFF magic
- OLE documents: type `0x1F639D98` (2), `D0CF11E0` magic
- Nested DBPF: type `0x06EFC6AA` (23), `DBPF` magic

Tool: `tools/type_census.py`, `tools/type_unknowns.py`.

## Note on method

This file exists because the alternative was to ship `tools/prop_names.py` as though it
worked. The tool is kept, because its failure mode is the evidence that the hypothesis is
wrong, and because it will be the validation harness once the structure is known. But its
output is not a result.

That is the same pattern as the corrections already recorded elsewhere in this project: the
bit-31 compression flag in `docs/dbpf-format.md` section 4.2, and the shared-code claim in
`re/binary-analysis.md` section 6. Measuring beat guessing in both cases, and it did here
too.

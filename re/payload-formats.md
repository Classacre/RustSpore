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
| `0x00B1B104` | prop | **decoded.** 11,640 of 11,953 (97.38%) parse exactly. Zero overruns. Remaining 2.6% are clustered in groups `0x0001FFFF` and `0x00010000`, likely variant layouts. |
| `0x2F4E681B` | RW4 | **header decoded.** Magic signature found, two subtypes (0x01 and 0x04), footer invariant measured. Object dispatch not yet implemented. See below. |
| `0x2F4E681C` | RASTER | **decoded and validated.** DXT1, DXT5, and raw RGBA sub-formats. Extracted to PNG with zero errors. See below. |
| `0x01C135DA` | gmsh mesh | **96.6% decoded and validated.** Header, attributes, index v1 (100%), index v2+ (91.3%). See below. |
| `0xEE17C6AD` | anim | **decoded and validated.** 1000/1000 samples (100%) parse correctly. CHAN sections, INFO/POS/ROT headers, keyframe structure all decoded. See below. |
| `0x5CAA5E28` | blockdata | **header partially decoded.** Starts with bounding box floats. Full structure not yet solved. |
| `0xDFAD9F51` | string table | **decoded.** Fixed 796-byte records: 4-byte hash + UTF-16LE string + null padding. See below. |
| `0x02B9F662` | audio event | **header partially decoded.** FNV-1 hashed string keys. Known keys: parent, gain, is3d, samples, codec. 50.6% exact parse. See below. |
| `0x00E6BCE5` | gmdl model | **header decoded.** Version 8, bounding box, triangle buffer header, index data. Vertex format partially decoded. See below. |
| `0x03F51892` | audio event (variant) | **header partially decoded.** Same FNV-1 hash scheme as 0x02B9F662. |
| `0xDFAD9F51` | string table | **decoded.** Fixed 796-byte records: 4-byte hash + UTF-16LE string + null padding. |

## prop, type 0x00B1B104

The most common type in `Spore_Game.package`: 7,030 of 11,529 resources.

Status: **decoded.** 11,640 of 11,953 resources across three packages parse to exactly
the last byte, which is **97.38%**. Zero overruns.

The remaining 313 resources (2.6%) fail with garbage type words, which means the walk
went out of alignment. They are heavily clustered in groups `0x0001FFFF` (236 failures)
and `0x00010000` (77 failures), so they are probably a variant prop layout used by
specific subsystems rather than random corruption.

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

### Type sizes, measured

`[MEASURED]` A single `key` is **16** bytes, not the 12 that three u32 suggests; its
array stride is 12. Grid test: 12 bytes gives 6.51% exact, 16 gives 22.93% at that
stage. Array elements carry no single-value padding.

`[MEASURED]` Compound types (vector2/3/4, colour, transform, bbox) carry 4 extra bytes
as single values **except** `colorRGBA` (`0x34`), which is exactly 16 bytes with no
padding. Treating it as 20 bytes produced 40 overruns; removing the padding fixed them
and raised the rate from 94.99% to 97.38%.

`[MEASURED]` Observed type frequencies over the three packages are dominated by real
types, which is itself a check that the walk is aligned:

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

### Remaining failures

`[MEASURED]` 313 resources still fail, about 2.6%. They are heavily clustered by group:

| Group | Failures |
|---|---|
| `0x0001FFFF` | 236 |
| `0x00010000` | 77 |

`[INFERRED]` These are not random corruption or a structural bug in the general layout.
They are probably a variant prop format used by specific subsystems (effects, particles,
or animation). The exact-consumption rate inside those groups is near zero, while outside
them it is effectively 100%.

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

### Payload structure and object dispatch

`[MEASURED]` All payloads begin with LE u32 = 1 at offset 96 (right after the 96-byte
header). At offset 100, a u32 value varies (possibly a payload size). At offset 104,
LE u32 = 4 (invariant).

`[MEASURED]` Object type codes appear as LE u32 values throughout the payload, starting
around offset 152. They are not at fixed offsets but appear in a table-like sequence.
Type code categories by high 16 bits:

| High 16 bits | Category | Examples |
|---|---|---|
| `0x0001` | BaseResource family | 0x00010030 (BaseResource), 0x00010031, 0x00010032 |
| `0x0002` | Graphics objects | 0x00020003 (Raster), 0x00020004 (VertexDescription), 0x00020005 (VertexBuffer), 0x00020007 (IndexBuffer), 0x00020009 (Mesh), 0x000200AF (BlendShapeBuffer) |
| `0x0007` | Animation objects | 0x00070001 (KeyframeAnim), 0x00070002 (Skeleton) |

`[MEASURED]` Type code occurrence frequency across 80 samples: Raster (255),
BaseResource (207), IndexBuffer (206), Skeleton (196), KeyframeAnim (185),
VertexDescription (89), VertexBuffer (87), Mesh (83), BlendShapeBuffer (15).

`[MEASURED]` Subtype distribution (offset 28): 1 (22 samples), 0x04000000 (38),
0xCAFEBABE (20). The subtype determines the payload structure. Subtype 1 payloads
contain mesh-related types (VertexDescription, VertexBuffer). Subtype 0x04000000
payloads contain fewer types and different data after the type code table.

### Payload structure

`[MEASURED]` The payload after the 96-byte header has a consistent structure:

```
offset  96: LE u32 = 1 (version/count)
offset 100: LE u32 = varies (hash for subtype 1, offset 440 for subtype 0x04000000)
offset 104: LE u32 = 4 (invariant)
offset 108-148: zeros and repeating (1, 0) pattern (3 pairs)
offset 152: section header: type 0x00010004 + count + offset/size pairs
offset 180: section header: type 0x00010005 + count + values
offset 196: type code table (flat list of LE u32 type codes)
offset ???: object data (type code + instance data)
offset off76: footer (off68 bytes)
```

`[MEASURED]` The two section headers at 152 and 180 (types 0x00010004 and 0x00010005)
contain metadata: count followed by offset/size pairs. For subtype 1, the first
section header has 3 offset/size pairs (12/28, 128/164, 192/???). The second has
count=22 and values (12, 0).

`[MEASURED]` The type code table at offset 196 is a flat sequence of LE u32 type codes.
For subtype 1 (Sample 1, 33 type codes total), the table contains:
0x00010030, 0x00010031, 0x00010032, 0x00010010, 0x00020003, 0x00020004,
0x00020005, 0x00020007, 0x00020009, 0x0002000B, 0x0002001A, 0x00070001,
0x00070002, 0x00070003, 0x0007000B, 0x0007000C, 0x0007000F.

`[MEASURED]` For subtype 0x04000000, the type code table is shorter (ends at Raster
0x00020003), and the data after the table diverges. The value at offset 100 is
440 (0x01B8) for this subtype, consistently across samples.

`[UNKNOWN]` The exact mapping between type code table entries and object data
instances is not yet decoded. Object instances appear later in the file with
type codes followed by data, but the table-to-instance relationship is unclear.

`[CORRECTION]` An earlier revision claimed `0xCAFED00D` at offset 28 as a universal magic
trailer. It is not universal: it appeared in one sample and was misreported. The actual
field at offset 28 is a subtype discriminator with values 1, 0x04000000, and 0xCAFEBABE.

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

`[MEASURED]` The texture data includes all mipmaps, packed consecutively. A 512x512
DXT5 texture with 10 mips occupies 349,552 bytes; the file is 349,616 bytes, so the
overhead is 64 bytes (28-byte header + 36 bytes of metadata/footer).

`[MEASURED]` The raw RGBA format uses BGRA byte ordering; channels must be swapped to
produce correct colours.

`[VALIDATED]` `tools/raster_extract.py` decodes DXT5, DXT1 and raw RGBA to PNG. Twenty
resources extracted from `Spore_Content.package` with zero errors; output PNGs are valid
512x512 RGBA images.

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

**Version 1** (BE u32 @+4 = 1, 352 of 560 samples, 100% parse success):

```
4 bytes:  magic (15 00 06 08)
4 bytes:  BE u32 = 1 (version)
4 bytes:  LE u32 = 1024 (constant)
4 bytes:  zeros
2 bytes:  BE u16 = 1
2 bytes:  BE u16 = 0
2 bytes:  BE u16 = index_count
2 bytes:  BE u16 = 2 (index stride, u16)
index_count * 2 bytes of LE u16 indices
```

`[VALIDATED]` All 352 version 1 samples parse to exactly the last byte. All indices
are less than vcount. Example: 410-byte sample with vcount=14 has 36 indices forming
a triangle list.

**Version 2+** (BE u32 @+4 = 2 through 10, 208 of 560 samples):

`[MEASURED]` The header layout differs. The actual index data starts at a variable
offset after the magic, preceded by sub-section headers and 3x4 transformation
matrix data. For the 39722-byte PatchData sample (vcount=942, version=2), 2702
valid u16 indices were found starting at offset 34214. The first index is 512,
followed by 0, 1, 2, 3, 2, 1, 4, 5, 6, 7, 6, 5, 8... suggesting a triangle strip
with degenerate triangles.

`[MEASURED]` The gap structure follows the formula:
`gap = 16 + version*4 + 4 + N*matrix_size + 8`

Where:
- 16 = common header (magic + version + 1024 + 256)
- version*4 = per-version attribute fields (v2=24, v3=28, v4=32, v5=36)
- 4 = count (BE u16) + stride (BE u16)
- N*matrix_size = N transformation matrices
- 8 = footer

`[MEASURED]` Matrix size is 48 bytes (3x4 transform) for 49.8% of v2+ samples
(103/207). Alternative matrix sizes: 32 bytes (44 samples), 36 bytes (23),
40 bytes (12). The 48-byte matrices contain rotation/transformation data with
values like -1.0, 0.0, 1.0 (90-degree rotations).

`[MEASURED]` The header size = 16 + version*4. Version 2 has a 24-byte header,
version 5 has a 36-byte header. The count/stride sub-header appears right
after the version-specific fields.

`[MEASURED]` Header-based decoding (reading count and stride directly from the
header at position `16 + version*4`) validates 125 of 207 v2+ samples (60.4%).
All valid samples have stride=48. The count field gives the number of 48-byte
matrices, and the index data starts at `header + 4 + count*48 + 8` (footer).

`[MEASURED]` Invalid samples fall into two categories:
1. stride=2 with large count (e.g. 1116, 2874): the `15 00 06 08` at this
   position is not the index section header but a different structure
2. stride=48 with reasonable count but 0 valid indices at the predicted
   position: the footer may be larger than 8 bytes, or there may be additional
   data between the matrices and the index data

### Footer (between matrices and index data)

`[MEASURED]` The footer between the transformation matrices and the index data
is either 8 or 10 bytes:

**8-byte footer** (125 samples): padding only.

**10-byte footer** (64 samples): v1-style index header:
```
+0:  BE u32 = 1 (version)
+4:  BE u16 = 0 (padding)
+6:  BE u16 = index_count - 1
+8:  BE u16 = 2 (index stride)
```

### File trailer (after index data)

`[MEASURED]` After the index data, a file trailer follows. The last 8 bytes are
always `ff ff ff ff ff ff ff ff`. Before that, a BE u16 value (8, 14, 15, 26
observed) of unknown meaning. The trailer also repeats the index count as a
LE u32.

### Parse rate

`[VALIDATED]` **96.6% overall** (541/560 samples):
- Version 1 index sections: 352/352 (100%) parse to exactly the last byte
- Version 2+ index sections: 189/207 (91.3%) with variable footer (8 or 10 bytes)
- Remaining 19 samples have stride=2 at the count/stride position, indicating
  a different structure not yet decoded

## anim, type `0xEE17C6AD`

`[MEASURED]` Appears in `PatchData.package` (500+) and `Spore_Game.package` (500+).
Total sampled: 1000. All 1000 start with `"ANIM"` magic.

### Header (12 bytes)

`[MEASURED]` Confirmed across 1000 samples:

```
offset  0: "ANIM" (4 bytes magic, 100% invariant)
offset  4: LE u32 = file size (100% match)
offset  8: LE u32 = 25 or 21 (two groups of 500 each)
```

`[MEASURED]` The constant at offset 8 has two values: 25 (500 samples) and 21
(500 samples). The split correlates with the source package: PatchData uses 25,
Spore_Game uses 21. This may be a format version or package identifier.

### String field (260 bytes)

`[MEASURED]` Starts at offset 12. Null-terminated ASCII Windows file path. The
path is padded with null bytes to exactly 260 bytes total. Animation data always
starts at offset 272.

Example paths:
- `c:\spore\sporepatch\data\creatures\animations\_final\tribe\com_battle_roar_3.animation`
- `C:\Spore_Executables_Latest\Data\Creatures\Animations\_final\cell\Cell_death_eaten.animation`

### Animation data header (40 bytes)

`[MEASURED]` Starts at offset 272. Structure:

```
+0:  LE u32 (hash/ID, varies per animation)
+4:  LE u32 = 0 (100% invariant)
+8:  LE u32 = 0 or small value (0: 294, 8: 67, 1: 27, 9: 12)
+12: LE float = frame duration (0.0333 = 1/30 FPS in 384/400 samples;
     other values: 0.0278 = 1/36, 0.025 = 1/40, 0.0286 = 1/35, 0.0222 = 1/45)
+16: LE float = duration in frames (range: 1.0 to 1988.0)
+20: LE u32 = 0 (398/400) or 2 or 90
+24: LE u32 = duration - 1 (last frame index, 0-indexed)
+28: LE u32 (varies widely: 17 to 4299, purpose unknown)
+32: LE u32 (varies: 11 to 25, possibly bone/track count)
+36: LE u32 = 0 (100% invariant)
```

`[MEASURED]` The frame duration at +12 confirms 30 FPS as the standard. The
duration at +16 is in frames (e.g. 77.0 frames at 30 FPS = 2.57 seconds).
Field at +24 is always duration - 1.

### Track data

`[MEASURED]` The track data is organized as per-bone "CHAN" sections. An offset
table at file offset 512 contains u32 values, each pointing to a bone's CHAN
section. The table is followed by the "CHAN" magic (0x4348414E).

**Offset table** (at file offset 512):
```
N u32 values (file offsets to each bone's CHAN section)
"CHAN" magic (4 bytes)
```

**Per-bone CHAN section**:
```
+0:   "CHAN" (4 bytes magic)
+4:   LE u32 = 0
+8:   bone name (null-terminated ASCII, e.g. "root", "head", "MouthMorphs")
+232: "INFO" header: magic(4) + LE u32(0) + LE u32(kf_count) + LE u32(0)
+264: "POS\0" header: magic(4) + LE u32(20) + LE u32(kf_count) + LE u32(1)
+296: "ROT\0" header: magic(4) + LE u32(100) + LE u32(kf_count) + LE u32(2)
+312: keyframe data
```

`[MEASURED]` The v2 field in INFO/POS/ROT headers is the keyframe count (e.g. 200,
280, 240). The v3 field is the channel type: 0=INFO, 1=POS, 2=ROT. The v1 field
varies: 0 for INFO, 20 for POS, 100 for ROT (possibly data stride or offset).

`[MEASURED]` ROT keyframes are 20 bytes each: LE float time + 4 LE floats
(x, y, z, w quaternion). POS keyframes are likely 16 bytes: time + 3 floats.

`[MEASURED]` Morph bones ("MouthMorphs", "eyes", "ears") have no POS/ROT headers.
Instead, the INFO section contains sub-entries with channel types 3 and 4:
```
LE u32 = 19 (0x13, constant across morph entries)
LE u32 = hash
LE u32 = data size (20 or 60)
LE u32 = keyframe count
LE u32 = channel type (3 or 4)
```

`[MEASURED]` Bone names observed: root, head, MouthMorphs, eyes, ears, R hand,
R hand rel, L hand, R foot, L foot. The "rel" suffix indicates relative
transforms.

### Trailer

`[MEASURED]` The file ends with:
```
LE u32 = 0x60 (96, constant)
LE u32 = 2 (repeated several times)
LE u32 = 0 (padding)
4 bytes: hash (varies)
LE u32: file size related value
ASCII animation name (null-terminated, e.g. "vidle_get_angry03")
```

`[MEASURED]` Animation names are ASCII (not UTF-16LE like the path in the header).
Examples: "vidle_get_angry03", "vidle_herbi_rollindust", "vwalk_neutral",
"footstep".

### Validation

`[VALIDATED]` `tools/anim_validate.py` parses all 1000 anim resources across
PatchData and Spore_Game with 100% success. Every bone's INFO header is at
section+232, POS at +264, ROT at +296. POS and ROT keyframe counts always match
the INFO keyframe count.

`[MEASURED]` Bone count per animation: 5 to 14 (median 10). Most common bones:
root (892), ears (857), R foot (773), L foot (773), eyes (691), head (674).
Keyframe counts: 440 (most common), 280, 240, 200, 140, 60, 180, 360, 100, 320.

## blockdata, type `0x5CAA5E28`

`[MEASURED]` Appears in `PatchData.package` (500+) and `Spore_Game.package` (500+).
Total sampled: 1000. No magic string. Size distribution: 36 (121), 48 (241),
84 (256), 228 (279), 804 (81), 3108 (18), 12324 (3), 49188 (1).

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
+28: 4 bytes of flags (0x04 = set, 0x00 = not set)
+32: variable data
```

`[MEASURED]` The 4 flag bytes at +28 show patterns like `04 04 00 00`,
`04 00 00 00`, `04 04 04 00`. The value 0x04 appears to be a boolean true.

`[MEASURED]` After the flags, 4 bytes of zeros, then variable data:
- 36-byte samples: 3 u32 zeros (total padding after bounding box)
- 48-byte samples: 4 zeros + 4-byte value + 8-byte hash/ID
- 84-byte samples: 4 zeros + 48 bytes of structured data

`[MEASURED]` The 48-byte structured data in 84-byte samples consists of 4 groups
of 12 bytes. Each group contains 6 BE u16 values. The values show a consistent
increment pattern: the step between groups is 0x22 (34 decimal) for most fields.
This suggests 4 quantized reference points or vertices with multiple attributes.

`[MEASURED]` Example (Sample 3): groups have bytes like `01 00 | 00 30 | 30 00 |
70 90 | 00 d0 | f0 00` with the second byte of fields 2, 3, 5, 6 incrementing by
0x22 across groups (00, 22, 44, 66). The first field (`01 00` = 256) is constant.

`[UNKNOWN]` The exact meaning of the 6 u16 fields per group is not yet decoded.
They could be quantized positions, normals, or other geometric attributes
relative to the bounding box.

## string table, type `0xDFAD9F51`

`[MEASURED]` All 446 samples are exactly 796 bytes. Structure:

```
offset  0: LE u32 (hash/ID, varies per string)
offset  4: UTF-16LE string (null-terminated)
offset X: null padding to 796 bytes
```

`[MEASURED]` The string is encoded as UTF-16LE (little-endian 16-bit characters).
Maximum string length: (796 - 4) / 2 = 396 characters. Observed strings include
"PLACEHOLDER_Maa", "PLACEHOLDER_Jawhead", "PLACEHOLDER_Ducky". Many samples
have empty strings (all zeros after the hash).

`[INFERRED]` The 4-byte hash at offset 0 is likely a lookup key for the string.
The fixed 796-byte size suggests this is a pre-allocated string buffer, possibly
for locale text or creature part names.

## audio event, type `0x02B9F662`

`[MEASURED]` 16,841 resources across `PatchData.package` and `Spore_Audio2.package`.
No magic string. The format uses **FNV-1 32-bit hashes** as string keys.

`[MEASURED]` Known FNV-1 hash-to-string mappings (confirmed against binary strings):

| Hash | String | Meaning |
|---|---|---|
| `0x5F6317D5` | `parent` | Parent reference (appears in 85% of entries) |
| `0x25DF0108` | `gain` | Audio gain/volume |
| `0x12D2A4D4` | `is3d` | 3D positional audio flag |
| `0x701ED91E` | `samples` | Audio sample references |
| `0xBA03D0CD` | `codec` | Audio codec identifier |

`[MEASURED]` Header structure (all big-endian):

```
offset   0: BE u32 entry_count (2-11, typically 2-3)
offset   4: N entries of 12 bytes each:
              BE u32 hash (FNV-1 of property name)
              BE u16 type (32=value, 13=float, 1=bool, 0=null)
              BE u16 flags
              BE u32 value (or float if type=13)
```

`[MEASURED]` After the entries, optional sections follow:

- **samples section**: hash=`0x701ED91E`, type=32, flags=156, then BE u32 pair_count,
  BE u32 stride=12, then pair_count pairs of 12 bytes (hash + 8 bytes data).
  The pairs contain consecutive hash values (e.g., 0xA844269D, 0xAB442269D).
- **codec section**: hash=`0xBA03D0CD`, type=10, flags=0, then BE u32 count.

`[MEASURED]` Parse rate: 50.6% exact fit (8,515 of 16,841 samples). The remaining
49.4% have additional sections or data not yet fully decoded. Gap analysis shows
28-byte and 25-byte trailing data in many inexact samples.

`[INFERRED]` This is an audio event/sound configuration table. The property names
(parent, gain, is3d, samples, codec) are all audio-related. The format defines
sound events with their properties and sample references. Type `0x03F51892` is a
related variant using the same FNV-1 hash scheme.

Tool: `tools/audio_event_parse.py`, `tools/audio_event_v2.py`, `tools/type_02b9_deep.py`.

## gmdl model, type `0x00E6BCE5`

`[MEASURED]` 4,209 resources, all in `Spore_Content.package`. All RefPack compressed.

`[MEASURED]` Header structure (little-endian):

```
offset   0: LE u32 version = 8 (100% invariant across 50 samples)
offset   4: LE u32 flags = 0 (92% invariant)
offset   8: LE u32 count = 1 (84%) or 2 (8%) - possibly mesh count or LOD level
offset  12: 7 LE floats (bounding box + extra):
              min X, min Y, min Z, max X, max Y, max Z, extra (radius?)
offset  40: LE u32 triangle_buffer_count = 1 (84%)
offset  44: LE u32 primitive_type = 4 (D3DPT_TRIANGLELIST, 92%)
offset  48: LE u32 indices_count
offset  52: LE u32 indices_bits = 16 (92%)
offset  56: LE u32 buffer_size (usually = indices_count * 2)
offset  60: LE u32 = 0 (padding)
offset  64: index data (u16 LE values, triangle indices)
```

`[MEASURED]` Validation across 50 samples: version=8 (100%), primitive_type=4 (92%),
indices_bits=16 (92%), buffer_size = indices_count * 2 (92%). The 8% failures are
from variant layouts or insufficient decompressed data.

`[MEASURED]` Index data starts at offset 64 as u16 LE values. First indices observed:
2, 0, 3, 1, 4, 5, 6, 6, 5, 7, 8, 4, 6, 6, 7, 9... These form triangles in
groups of 3, with some degenerate triangles (repeated indices).

`[MEASURED]` After the index data, a vertex format description section follows,
containing vertex element descriptors. Vertex data appears as LE floats
(position XYZ + normal XYZ, stride approximately 24 bytes). The exact vertex
format layout is not yet fully decoded.

`[INFERRED]` This is the higher-level model format referenced by the SporeModder
Blender Addon documentation. It contains mesh geometry (indices + vertices) with
a bounding box. The format is related to but distinct from the gmsh mesh format.

Tool: `tools/gmdl_quick.py`, `tools/gmdl_parse.py`.

## Other unknown types (structural notes)

`[MEASURED]` Type census identified 80 distinct types across 91,267 resources.
Beyond the formats documented above, several more have identifiable structure:

**Type `0x030BDEE3`** (1,641 resources, Spore_Content):
- BE u32=10 (version), BE u32=0, BE u16 count, then hash/float pairs
- Contains ASCII string `#tag:spore.com,2006:AssembledContent`
- Two `0xFFFFFFFF` sentinels at +24 and +28
- `[INFERRED]` Content assembly manifest or RSS-like index format

**Type `0x250FE9A2`** (601 resources, PatchData/Spore_Graphics):
- Magic `0xB83FFEE3` (BE u32, 100% invariant)
- BE u32=3 (version), BE u16 count
- Repeating 12-byte records: BE u32 hash + BE u32 + BE u32
- Same hash+value pairs repeat many times (lookup table)
- `[INFERRED]` Fixed-stride lookup table with magic header

**Type `0x1A99B06B`** (1,733 resources, Spore_Content/Spore_Pack_03):
- Magic `0xD37E6548` (BE u32, 100% invariant)
- BE u32=5 (version), BE u32 count, BE u32=0
- Float data at +48 onwards (small values, transform-like)
- `[INFERRED]` Animation or transform data with magic header

**Type `0x0F43029A`** (1,022 resources, Spore_Content):
- Magic `0xB755B4AB` (BE u32, 100% invariant)
- BE u32=9 (version), then BE u32 counts
- Float data at +48 onwards (similar to 0x1A99B06B)
- `[INFERRED]` Related to 0x1A99B06B, possibly a variant

**Type `0x02D5C9AF`** (1,631 resources, Spore_Content):
- BE u32=4 (count), BE u32=10 or variable
- Pairs of hashes: `0x02DD90AF` + `0x02E1A75D` (both 100% invariant)
- Followed by more hash pairs (e.g., `0x438F6347`, `0x05FB4759`)
- `[INFERRED]` Reference/dependency table linking resource hashes

**Type `0x01A527DB`** (2,898 resources, PatchData/Spore_Audio2):
- BE u32 ~0x05005622 or 0x0400AC44 (variable, similar range)
- Data after first 4 bytes appears compressed or encoded
- `[UNKNOWN]` Format not yet identified

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

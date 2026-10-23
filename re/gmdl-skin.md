# GMDL assembled geometry and retained skin data

The installed assets are the evidence. The new companion module
`crates/spore-core/src/gmdl_skin.rs` reads the tail that the older geometry parser
only skipped. It does not copy a third-party implementation or retail code.
Raw assets and resource-level diagnostics remain local under ignored `target/`.

## Observed layout

The core geometry boundary is `Gmdl.consumed`. After that boundary:

| Field | Storage |
|---|---|
| zero, material count | two little-endian words |
| material prefix | **version 9 only**, one big-endian presence word; zero is an empty slot with no following count |
| shader-data count and records | little-endian identifiers and measured payload extents |
| palette descriptor count | little-endian word |
| palette descriptors | two little-endian words each |
| part count | little-endian word |
| part | two 56-byte transforms, one word, three resource-key words, presence word |
| optional part records | count followed by count × 160 bytes |
| footer | three words |

Shader-data payloads are retained exactly: `0x20D` has a count and count × 24
bytes; `0x210` has 20 bytes; `0x218` has no payload; `0x21C` has 32 bytes;
`0x238` has one byte. The new `0x21C` extent was measured from exact source-tail
consumption and independently corroborated by the installed source shader
declaration `float4 bakedPaint[2]` with metadata identifier 540 (`0x21C`). This
establishes storage width; it does not implement the source shader's behavior.
Their texture roles and shader behavior are not inferred from record order.

The 56-byte transform contains two unsigned shorts at +0/+2, translation XYZ at
+4, uniform scale at +16, and nine rotation floats at +20. This matches the
published [community SDK Transform declaration](https://github.com/Spore-Community/Spore-ModAPI/blob/master/Spore%20ModAPI/Spore/Transform.h).
Only this data layout was consulted; its implementation was not copied.
The decoder retains the flags, count and floats without applying an unverified
composition policy to a creature.

Version 9 matters: the old parser's skipped-tail reader treats the new material
prefix as a shader-data count. Its `exact` field can therefore be false even when
the new companion decoder consumes the complete tail. The geometry boundary is
still measured separately. A version-9 synthetic regression checks the prefix's
byte order and full consumption.

## Concrete creature correspondence

In `Data/Spore_Content.package`, assembled model
`00E6BCE5:0001FFFF:40627100067C5390` is 323,828 decompressed bytes. Its geometry
ends at 317,460. It has 12 tail part slots. Five populated external model slots
contain 29 records of 160 bytes:

| Tail slot | Referenced RW4 | Records | Authored creature row |
|---|---|---:|---:|
| 1 | `41606100FB60AF37` | 9 | 10, mouth |
| 2 | `41606100BDA70E3B` | 4 | 11, eye |
| 3 | `416061003F31394D` | 4 | 12, eye |
| 8 | `416061003A25C5A2` | 6 | 17, grasper |
| 9 | `41606100910867CC` | 6 | 18, grasper |

These row correspondences are measured by unique matching low instance halves
and equal RW4 types. The authored part props name `40606000…` resources while the
tail names `41606100…` generated variants. The complete identifiers differ;
replacing one prefix with another is not a generally established resolver rule.

The first descriptor is `(0, 48)`. The first part's separate count is 19, and
19 + 29 = 48. The mesh has 6,967 vertices with UBYTE4 indices at byte +28 and
UBYTE4N weights at +32 of its 36-byte vertex records. Its first influence record
contains raw indices `[0, 30, 33, 36]` and weights `[66, 63, 63, 63] / 255`.
The checked decoder divides active joint-register offsets by three, preserves
the source weights and rejects active indices outside the selected palette.

Part 1 begins at byte 317,704 and its first 160-byte record at 317,840. Lower-detail
same-low-half models `40626202…` and `40626203…` retain the 12 default/identity part
slots and a 19-count descriptor but have no external bone records. They cannot
be assumed to contain the same animation data as the high-detail models.

## Unresolved animation semantics

The 160-byte records contain pose-like floats and additional words that resemble
saved process addresses. The decoder preserves all 40 words, including values
that are non-finite if interpreted as floats. It does not reinterpret them as
parent indices, joint hashes or inverse-bind matrices.

There are no obvious 19 body bind transforms in the example tail. A complete
assembled runtime skeleton has therefore **not** been recovered merely by
decoding these records. Native hierarchy and pose-consumer evidence is still
required.

The native trailing-part reader independently confirms the two transforms, count
field, resource-key words and optional 160-byte record array. A key is absent
when its **type word** is `FFFFFFFF`; the instance words need not be interpreted
in that case. The companion accessor follows that rule and retains all raw words.
The native record copier skips word indices 3, 18, 19 and 39 after zeroing its
destination. Those four fields are padding candidates. Word 14 is copied as byte
fields, so interpreting it as a float is inappropriate. Native generation and
consumption establish translation at +0, quaternion at +16, and scale at +32.
All 30,538 base-compatible records have finite values, positive scales and unit
quaternions within 0.001. The byte at +56 is 0 or 1. A zero byte avoids reading the
matrix storage at +80 and uses the multiplier triple at +44; a nonzero byte uses
that matrix storage, the triple at +144, and translation at +60. All 7,449 first
external records have zero matrix storage, which is consistent with that gate.
The matrix's inverse-bind role and complete hierarchy policy remain unresolved.
The parser does not convert the sixteen storage words into a homogeneous Mat4 or
copy the native runtime's implementation.

Independent source joins strengthen the external-part association: all 7,449
non-empty external record arrays in the base-compatible population resolve their
exact referenced RW4 model and match at least one contained skeleton's bone
count. 7,435 have a unique matching skeleton section; 14 are ambiguous by count.
Count agreement does not prove bone ordering. The retained evidence is
`target/animation/gmdl-rw4-skeleton-count-join.json`; native reader/copy/generator
findings are retained in the focused local REA evidence, rather than copied into
repository code.

Multiple-mesh examples establish that the descriptor's first word is not a mesh
index or vertex-buffer index. `4061710006444D55` has two meshes sharing VB 0 and
descriptors `(0, 62), (62, 22)`; `4062710006725707` has `(0, 60), (60, 36)`.
Those values fit cumulative palette offsets. The public structure retains this
word without claiming a selector meaning. Automatic influence decoding currently
supports only the unambiguous one-mesh/one-palette case; ambiguous mappings return
an explicit error.

The independent base-compatible population contains 934 models with non-empty
descriptors. All 934 have exact cumulative first words, one descriptor per mesh,
and a descriptor total equal to non-sentinel per-part count fields plus external
160-byte record counts. That establishes the partition equation; it does not
recover the generated body's bone hierarchy. A tempting `2 × body source rows -
1` count rule failed in the wider creature join population and is not used.

## Verification

`gmdl_skin_verify` scans every supplied package using the production DBPF reader
and emits resource-level canonical FNV digests. The independent Python probe
reads the source fields directly. `gmdl_skin_compare.py` compares every retained
tail and, by default, every successful influence row independently from source
vertex declarations. An explicit influence limit is labeled partial.

```powershell
tools/build.bat test -p spore-core gmdl_skin::tests
tools/build.bat run -p spore-bevy --example gmdl_skin_verify -- `
  'D:/Games/SPORE Collection' --digests target/animation/gmdl-skin-digests.tsv
python tools/gmdl_skin_probe.py --out target/animation/gmdl-skin-probe.json
python tools/gmdl_skin_compare.py target/animation/gmdl-skin-probe.json `
  target/animation/gmdl-skin-digests.tsv --out target/animation/gmdl-skin-comparison.json
```

Final Rust and independent Python coverage is **11,041/11,041 GMDL resources in
82/82 packages, with zero parse problems**: 4,209 version 8 and 6,832 version 9.
The source tails retain 128,242 part slots and 46,576 external 160-byte records.
All 2,489 models with nonempty palettes satisfy all three partition constraints.

Thirteen focused tests pass. The conservative one-mesh/one-palette influence
path decodes 1,053 meshes and 5,105,226 vertices. It explicitly reports unsupported
declarations and ambiguous/no-palette mappings; this count is not a claim of full
assembled creature animation support.

**Complete independent digest agreement:** all 11,041 tails and all 1,053
supported influence records match, with zero mismatches across 12,094 canonical
records. The comparison independently walks source vertex declarations and
retained source-tail fields. Evidence is
`target/animation/gmdl-skin-comparison.json`, with resource rows in
`gmdl-skin-digests.tsv` and full Python source measurements in
`gmdl-skin-probe-complete.json`. The first base-compatible pass also measured all
7,134,298 UBYTE4/UBYTE4N vertices summing to weight-byte 255 and all 10,575,658
active register indices divisible by three; source shader `SkinBones[192]`
independently corroborates the three-register, 64-bone palette convention.

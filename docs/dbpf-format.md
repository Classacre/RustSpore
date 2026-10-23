# The DBPF container format, as measured in SPORE

**2026-10-23 correction:** The original index reader now establishes common
header fields and current-record compression metadata. The historical group,
64-bit instance and wide prefix/trailer labels below are superseded by
`../re/archive-index-native.md`. Payload offsets and sizes remain verified.
Use `dbpf::native_index` for canonical native keys; the older shape API remains
a compatibility view. Fresh agreement covers all 82 packages / 158,720 records
and complete payload decoding, rather than the initial 13-package investigation.

Status: verified against the shipped game data on 2026-10-16.
Evidence grades: `[MEASURED]` observed directly in bytes, `[INFERRED]` reasoned from
measurements, `[UNKNOWN]` not established.

Every claim below was produced by scripts in `tools/`. Reproduction commands are at the
end. Nothing here is copied from a third party; where public documentation agrees or
disagrees, that is noted separately in `prior-art.md`.

## 1. What DBPF is

`[MEASURED]` Every asset in SPORE lives inside a `.package` file. Each such file begins
with the four bytes `DBPF` (0x46 0x42 0x50 0x44, read as a little-endian u32 it is
0x46504244). The 13 packages in this install total roughly 4.9 GB and hold **136,081
resource records across the 10 that parse cleanly** (see section 6 for the 3 that do
not).

The container is an archive: a header, an index describing every resource, and then the
resource payloads. The payloads are individually compressed or stored raw.

## 2. Header

`[MEASURED]` The header is 96 bytes (0x60). Only a few fields carry information in these
files; everything else is zero.

| Offset | Type | Meaning | Evidence |
|---|---|---|---|
| 0x00 | char[4] | `DBPF` magic | `[MEASURED]` all 13 packages |
| 0x04 | u32 | major version, always `3` | `[MEASURED]` all 13 |
| 0x08 | u32 | minor version, always `0` | `[MEASURED]` all 13 |
| 0x24 | u32 | resource record count | `[MEASURED]` see section 4 |
| 0x2C | u32 | size of the index block in bytes | `[MEASURED]` see section 3 |
| 0x40 | u32 | absolute file offset of the index block | `[MEASURED]` see section 3 |
| 0x0C to 0x23, 0x28, 0x30 to 0x3C, 0x44 to 0x5F | u32 | all zero in every package inspected | `[MEASURED]` |

The decisive test for 0x2C and 0x40 is that they are jointly self-consistent:

```
Spore_Game.package        19887288 + 322820 = 20210108  == file size
Spore_Pack_03.package      9572727 +   7456 =  9580183  == file size
Spore_EP1_Locale_02.package 838193 +     96 =   838289  == file size
```

Three independent packages, exact to the byte. This is why 0x40 is the index offset and
0x2C is the index size rather than, say, a count or a version.

### 2.1 A conflict with public documentation about the version field

`[MEASURED]` The u32 at offset 0x04 is `3` in all 13 packages, and the u32 at 0x3C is
also `3` in all 13.

`[UNKNOWN]` **Whether the field at 0x04 is properly called the major version.** The name
is inherited from the conventional DBPF layout, not derived from measurement. Public
documentation researched in `docs/prior-art.md` section 1.4 says the opposite: three
independent public sources describe SPORE's archive as DBPF version **2.0**, and one of
them annotates the field at 0x3C as the index minor version.

So there is a direct conflict: this measurement says the value at 0x04 is 3, and public
sources say SPORE is version 2.0. Both can be true at once if the field at 0x04 is not
the archive version, or if the public sources are describing a different revision of the
format than the one in this install. This install is a 2016 GOG repack of the 2009
patched build, so a later format revision is plausible, but that is a hypothesis, not a
finding.

`[INFERRED]` A reader should not branch on this field. Nothing measured here depends on
its interpretation, and the index layout was established structurally without reference
to it.

`[UNKNOWN]` Fields 0x0C to 0x23 are zero here. Public documentation for sibling EA
formats describes date-created and date-modified fields in that region; in this install
they are zero, so the question is open.

## 3. Index block

`[MEASURED]` The index block sits at the offset in header field 0x40 and runs for the
number of bytes in field 0x2C. Its internal layout is:

```
u32 prefix            (4 bytes)
N x 28-byte records   (N = header field 0x24)
u32 trailer           (4 bytes)
```

The arithmetic closes exactly: `4 + N*28 + 4 == index size`. For `Spore_Game.package`,
`4 + 11529*28 + 4 = 322820`, which equals header field 0x2C.

`[MEASURED]` The prefix is `4` in all 10 packages that parse cleanly.
`[MEASURED]` The trailer is `0x0001FFFF` or `0x00010000`, both of which are values that
also occur in the group field of records in the same file.
`[UNKNOWN]` The meaning of the prefix and trailer is not established. They are
constant-shaped and can be skipped by a reader.

### 3.1 How the layout was established

It was not assumed. A record is only accepted as valid if its `(offset, offset+size)`
span lies inside the file. `tools/dbpf_find_index.py` marks every 4-byte-aligned start
position as valid or not for a candidate record shape, then looks for the longest run of
consecutive valid records at a fixed stride. A run of 11,529 valid records at a 28-byte
stride cannot occur by chance, so finding one both locates the index and fixes the record
size. That search returned exactly 11,529 records for `Spore_Game.package`, matching the
header count at a ratio of 1.0000.

`tools/dbpf_index_search.py` records an earlier, wrong hypothesis (24-byte records at
`file size - 96`) that this search disproved. It is kept in the repository so the
reasoning is auditable.

## 4. Resource record (28 bytes)

`[MEASURED]` Seven little-endian u32 fields in this order:

| Field | Bytes | Meaning |
|---|---|---|
| group_id | 0 to 3 | resource group |
| type_id | 4 to 7 | resource type |
| instance_hi | 8 to 11 | high word of the instance id |
| instance_lo | 12 to 15 | low word of the instance id |
| offset | 16 to 19 | absolute file offset of the payload |
| size | 20 to 23 | stored payload size in bytes |
| decompressed_size | 24 to 27 | size after decompression |

The resource key is therefore `(type_id, group_id, instance_id)` with
`instance_id = (instance_hi << 32) | instance_lo`.

### 4.1 How the field roles were established

Not by guessing the order. `tools/dbpf_fields.py` measures how many distinct values each
of the seven fields takes across a whole package. A resource key has a low-cardinality
part (type, group) and a high-cardinality part (instance), so cardinality separates them
without assuming anything. In `PatchData.package` (25,535 records):

| Field | Distinct values | Reading |
|---|---|---|
| group_id | 3 | group: a handful of system groups |
| type_id | 58 | type: a small closed set |
| instance_hi | 442 | part of the instance id |
| instance_lo | 22,301 | part of the instance id |
| offset | 25,535 | unique per resource, so it is the file offset |
| size | 4,082 | repeated because many resources share a size |
| decompressed_size | 4,403 | repeated likewise |

### 4.2 Correction: bit 31 is not a compression flag

`[MEASURED]` Bit 31 of the `size` field is set on **100% of records** in every package
that parses: 25,535/25,535 in `PatchData.package`, 11,529/11,529 in
`Spore_Game.package`, 266/266 in `Spore_Pack_03.package`. A flag that is always set
carries no per-resource information.

An earlier hypothesis in this project treated bit 31 as "this resource is compressed".
That was wrong, and the measurement above is what disproved it. The corrected rule is in
section 5.

`[INFERRED]` Because the flag is constant, a reader should mask it off (`size &
0x7FFFFFFF`) and decide raw versus coded from the sizes instead.

## 5. Payload encoding: raw or RefPack

`[MEASURED]` Comparing `size` against `decompressed_size` per record splits the resources
into two clean groups. In `Spore_Game.package` (11,529 records):

- `size == decompressed_size`: **2,215** records. The payload is stored raw.
- `size < decompressed_size`: **9,314** records. The payload is compressed.

`[INFERRED]` Equality of stored and decompressed size is the raw marker. This is
consistent with the smallest examples: many resources of type `0x00B1B104` are 4 bytes
stored and 4 bytes declared, which cannot be a compressed stream.

### 5.1 RefPack

`[MEASURED]` The compressed payloads are EA's RefPack (LZ77 with a variable-length
command byte). The port in `tools/refpack.py` implements the four documented command
encodings. Validation is described in section 5.2.

`[MEASURED]` **The compressed stream carries its own size header.** Decompressing with
that header alone, with no size hint from the index, reproduces the index's declared
decompressed size exactly.

### 5.2 Validation result

`tools/validate_refpack.py` and a sweep over both packages give:

| Package | Coded resources | Sampled | Decompressed | Output length matched index |
|---|---|---|---|---|
| `Spore_Game.package` | 9,314 | 400 | 400 / 400 | **400 / 400** |
| `Spore_Graphics.package` | 15,211 | 400 | 400 / 400 | **400 / 400** |

Two independent sources of truth agree: the size declared inside the RefPack stream, and
the `decompressed_size` field in the DBPF index. The decompressor's output matched both
for all 800 sampled resources. Zero failures.

`[MEASURED]` Content identification after decompression confirms the codec is producing
real data, not plausible-looking noise. Type `0x2F7D0004` decompressed to bytes beginning
`89 50 4E 47 0D 0A 1A 0A`, which is the PNG signature. Public sources independently name
`0x2F7D0004` as PNG, so the measurement and the documentation agree on that ID.

### 5.3 End-to-end pipeline validation

`[MEASURED]` The strongest available test of the whole format is to decode every PNG in
the install and verify its internal checksums. A PNG carries a CRC32 over each chunk's
type and data, so any error anywhere in the chain, in the header parse, the index
location, the record field order, the raw-versus-coded rule or the decompressor, produces
a failing CRC.

`tools/dbpf_extract.py --validate-png` was run against three packages:

| Package | PNG resources | Valid PNGs | Bytes decoded |
|---|---|---|---|
| `Data/Spore_Graphics.package` | 8,360 | **8,360 (100.0%)** | 107,582,886 |
| `Data/Spore_Content.package` | 1,642 | **1,642 (100.0%)** | 44,445,374 |
| `DataEP1/Spore_EP1_Data.package` | 3,172 | **3,172 (100.0%)** | 62,025,611 |
| **Total** | **13,174** | **13,174 (100.0%)** | **214,053,871** |

Every chunk of every image passed. Zero CRC failures across 13,174 resources and about
214 MB of decoded output.

`[INFERRED]` This is a much stronger result than the length checks in section 5.2, because
a length match can be satisfied by a decompressor that produces the right number of wrong
bytes. A CRC match cannot. The container format and the codec are therefore validated
byte-exactly, not merely plausibly.

`[INFERRED]` It also settles the field-order question in section 4.1 independently of the
cardinality argument: if `offset` and `size` were swapped, or if `decompressed_size` were
misplaced, no PNG would decode correctly.

## 6. Index layout variants, and the rule that unifies them

Earlier in this project three packages were recorded as unsolved: `Spore_Audio1.package`,
`Spore_EP1_Locale_01.package` and `Spore_EP1_Locale_02.package` did not yield an index
under the 28-byte record shape. **That gap is now closed.**

### 6.1 Every package fits exactly

`[MEASURED]` `tools/dbpf_variants.py` searches for a `(prefix P, record size R)` pair that
consumes the index block exactly: `P + N*R == index_size`. **All 13 of 13 top-level packages
have such a pair**, and the Rust reader opens **all 82 packages in the install** (the extra
ones are locale and booster-pack archives) using the same three shapes. There is no package
left without an exact fit.

| Package group | P | R | Trailer T | P + R + T |
|---|---|---|---|---|
| 10 packages (`Spore_Game`, `Spore_Graphics`, `PatchData`, `Spore_Audio2`, both EP1 content, EP1 data, `BoosterPack_01`, `Spore_Content`, `Spore_Pack_03`) | 4 | 28 | 4 | **36** |
| `Spore_Audio1.package` | 12 | 24 | 0 | **36** |
| `Spore_EP1_Locale_01.package`, `Spore_EP1_Locale_02.package` | 16 | 20 | 0 | **36** |

`[MEASURED]` The invariant `P + R + T = 36` holds for every one of the 13. Equivalently,
`index_size = 36 + (N-1)*R`.

`[INFERRED, UNCONFIRMED]` That the total is always 36 suggests the block begins with a
fixed 36-byte region which happens to contain the first record plus its surrounding
metadata. This is a plausible reading of the arithmetic, not an established fact. The
number 36 has no confirmed meaning.

`[MEASURED]` **The exact fit is necessary but not sufficient to pick a shape, and it is
also not always sufficient on its own to accept one.** The arbiter is storage contiguity
(`offset_i + stored_size_i == offset_{i+1}`), because payloads are packed. But contiguity
needs at least two records: a package holding a single resource scores 0 by construction.
`Locale/en-us/UI.package` in a retail install is exactly that, and the Rust reader refused
it until the fit itself was allowed to decide when `N == 1`. The layout was never in doubt;
the test was.

### 6.2 How each variant was confirmed

Arithmetic alone is not sufficient, and this is worth stating because it produced a false
positive. For `Spore_EP1_Locale_02.package`, both `(P=16, R=20)` and `(P=32, R=16)` satisfy
the exact-fit equation. So exact fit narrows the candidates but does not pick between them.

`[MEASURED]` The arbiter is storage contiguity. When records are sorted by offset, each
resource should end exactly where the next begins, because the payloads are packed. The
test is `offset_i + size_i == offset_{i+1}`.

| Package | Layout tested | Exact chains | Gaps |
|---|---|---|---|
| `Spore_EP1_Locale_01.package` | P=16, R=20 | **16 / 16** | 0 |
| `Spore_EP1_Locale_02.package` | P=16, R=20 | **3 / 3** | 0 |
| `Spore_Audio1.package` | P=12, R=24 | **17,650 / 17,650** | 0 |
| `Spore_Pack_03.package` (control, known layout) | P=4, R=28 | 259 / 265 | 6 |

`Spore_Audio1.package` is the decisive case: 17,650 of 17,650 consecutive pairs are exactly
contiguous, with zero gaps. No wrong layout produces that. The next best candidate for the
same package scored 15,730 chains with 1,920 gaps.

The control row is included on purpose. `Spore_Pack_03.package` is a package whose layout
was already established independently, and the method rediscovers it, with the small
residual gaps expected from payload alignment padding.

### 6.3 Record fields per variant

`[MEASURED]` Field roles were separated by cardinality and confirmed by contiguity, as in
section 4.1.

**28-byte records** (10 packages), offset at byte 16, size at byte 20:
`group, type, instance_hi, instance_lo, offset, stored_size, decompressed_size`

**24-byte records** (`Spore_Audio1.package`), offset at byte 8, size at byte 12:

| Byte | Field | Evidence |
|---|---|---|
| 0 | type_id | 12 distinct values across 17,651 records; `0x01A527DB` alone covers 17,538 |
| 4 | instance_id | 17,634 distinct, with visibly consecutive runs such as `0x2B50C7CA`, `0x2B50C7CB` |
| 8 | offset | all 17,651 distinct, and the contiguity test passes |
| 12 | stored_size, bit 31 set | 14,137 distinct |
| 16 | decompressed_size | greater than or equal to the stored size |
| 20 | group_id | only 2 distinct values, `0x00010000` (15,730) and `0x0001FFFF` (1,921) |

Note that this variant orders the fields differently from the 28-byte record: `group_id`
comes last, and the instance id is a single u32 rather than a split u64.

**20-byte records** (both locale packages), offset at byte 4, size at byte 8:

| Byte | Field | Evidence |
|---|---|---|
| 0 | instance or hash | all four values in `Spore_EP1_Locale_02.package` differ |
| 4 | offset | 96 for the first record, which is the data start |
| 8 | stored_size, bit 31 set | contiguity holds |
| 12 | decompressed_size | greater than the stored size |
| 16 | group_id | `0x0001FFFF` throughout |

`[UNKNOWN]` This variant appears to carry no `type_id`. That is plausible, since a locale
package holds only locale data, but it is unconfirmed.

### 6.4 Consequence for a reader

`[INFERRED]` The record shape varies per file, so a reader must determine `R` from the
package rather than assuming 28. The recommended approach, and the one that works on all
13 packages here, is: try each candidate `R` from the exact-fit equation, then keep the one
whose records chain contiguously when sorted by offset. That is what
`tools/dbpf_variants.py` does.

Note also that `Spore_Audio1.package` and `Spore_Audio2.package`, two audio archives in the
same install, use different record shapes. Never assume one global layout.

## 7. Resource types observed

`[MEASURED]` Type distribution by package, most common first:

`Spore_Game.package` (11,529 records):
`0x00B1B104` 7,030, `0xEE17C6AD` 2,091, `0x5CAA5E28` 1,549, `0xDFAD9F51` 223,
`0x4B9EF6DC` 169, `0x055ADA24` 56.

`Spore_Graphics.package` (24,707 records):
`0x2F4E681B` (RW4) dominant, then `0x01C135DA` (gmsh mesh), `0x00B1B104`, `0x250FE9A2`,
`0x2F7D0004` (PNG).

Cross-referenced against the community type list compiled in `docs/resource-types.md`:

| ID | Name | Agreement |
|---|---|---|
| 0x00B1B104 | prop (property file) | consistent with decompressed shape |
| 0x2F4E681B | RW4 (RenderWare 4) | dominates the graphics package, as expected |
| 0x2F7D0004 | PNG | **confirmed by PNG magic in decompressed bytes** |
| 0x01C135DA | gmsh (mesh) | graphics package, consistent |
| 0xEE17C6AD | anim | game package, consistent |
| 0xDFAD9F51 | cell | game package, consistent |
| 0x5CAA5E28 | blockdata | low confidence in public sources |

`[INFERRED]` The distribution is architecturally coherent, which is a second kind of
check on the format work: if the index parse were wrong, the type IDs would not sort into
"graphics package is nearly all RenderWare and meshes" and "game package is nearly all
property and animation". They do.

## 8. Reproduction

```bash
PY=C:/Users/nieuw/.workbuddy-ai/binaries/python/envs/default/Scripts/python.exe

# header field dump for a single package
$PY tools/dbpf_probe.py "<package>"

# locate the index by structure alone
$PY tools/dbpf_find_index.py "<package>"

# parse every package and report validity plus field cardinality
$PY tools/dbpf_fields.py

# RefPack validation against real payloads
$PY tools/validate_refpack.py "<package>" 20
```

`tools/dbpf_verify.py` checks the whole header-plus-index hypothesis against a named set
of packages.

## 9. Summary of what is settled and what is not

Settled by measurement: the header fields at 0x24, 0x2C and 0x40; the index block layout;
the 28-byte record layout and all seven field roles; the raw-versus-coded rule; the
RefPack codec, validated on **101,312 coded resources across all 82 packages in one retail
install** by the Rust reader (`rustspore --verify`), every one decompressing to the size
the index declares.

Open: the meaning of the 4-byte index prefix and trailer; the internal structure of the
proprietary payload types; whether `0x00B1B104` is exactly "prop" as the community says.

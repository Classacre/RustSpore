# The DBPF container format, as measured in SPORE

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

## 6. Packages where the index was not located

`[MEASURED]` Three of the 13 packages do not yield an index under the 28-byte record
shape. The structural search in `tools/dbpf_find_index.py` was run against all three and
**found no valid index run in any of them**:

| Package | Entries at 0x24 | Index size at 0x2C | Index at 0x40 | Result of the search |
|---|---|---|---|---|
| `Spore_Audio1.package` | 17,651 | 423,636 | 1,018,588,194 | no run found for any tested layout |
| `Spore_EP1_Locale_01.package` | 17 | 356 | 3,794,024 | best run 66 records, 3.88x the claimed count, 1,528,200 bytes short of EOF: spurious |
| `Spore_EP1_Locale_02.package` | 4 | 96 | 838,193 | best run 2 records, 0.5x the claimed count: spurious |

Header fields 0x2C and 0x40 stay consistent in all three:
`1018588194 + 423636 = 1019011830`, `3794024 + 356 = 3794380`, `838193 + 96 = 838289`,
each equal to the file size. So the header reading holds; it is the record shape inside
the block that differs.

`[INFERRED, NOT CONFIRMED]` Dividing the block size by the entry count suggests a smaller
record for these three: about 24.0 bytes for `Spore_Audio1.package`, about 22.0 for
`Spore_EP1_Locale_02.package`, about 20.5 for `Spore_EP1_Locale_01.package`. These numbers
are suggestive, but the structural search did not confirm any of them, so they are
recorded as an unverified hypothesis rather than a finding. That the 20-byte layout scored
best on `Spore_EP1_Locale_02.package` (a spurious 2-record run) is a hint in that
direction and nothing more.

`[UNKNOWN]` The index layout for these three packages. A reader should fail loudly on them
rather than silently mis-parse. All gameplay content parses cleanly; the affected files
are the two expansion locale packages and the first audio archive.

`[MEASURED]` For contrast, `Spore_Audio2.package` (12,111 entries) **does** fit the
28-byte model exactly: `4 + 12111*28 + 4 = 339116`, equal to its index size. So the
variation is per-file, not per-content-kind. Two audio archives in the same install use
different record shapes, which is worth remembering before assuming one global record
size.

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
RefPack codec, validated on 800 resources against two independent size sources.

Open: the meaning of the 4-byte index prefix and trailer; the index layout of the three
packages in section 6; the internal structure of the proprietary payload types (prop,
RW4, gmsh, anim, cell); whether `0x00B1B104` is exactly "prop" as the community says.

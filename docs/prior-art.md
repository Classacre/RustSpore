# Prior public art: SPORE (2008) file formats and engine

Research date: 2026-10-16. Compiled by: general-purpose-2 (team `_auto_09c1c1dd`).

Purpose: catalogue what is ALREADY publicly documented about the SPORE (EA / Maxis, 2008) DBPF
container, its compression, its resource types, its SPORE-specific formats, its engine, the
existing tooling, and the legal position, so that a clean-room Rust + Bevy reimplementation can
(a) cross-check independently measured findings and (b) keep a clear separation between
"measured by us" and "asserted by a third party".

Every factual claim below carries a source URL. Where a source could not be found, the text says
"not found publicly" rather than guessing. Where sources disagree, both are shown and the
disagreement is flagged explicitly. No version numbers, dates, or IDs have been invented.

## How to read this document

Each section is split into:

- Documented publicly: a source states this.
- Sources conflict: two or more sources state different things.
- Not found publicly: no source located.

Nothing in this document is presented as legal advice.

---

## 1. DBPF container format

### 1.1 What DBPF is, documented publicly

- DBPF ("Database Packed File") is a proprietary archive format used in several Maxis-developed
  games including The Sims 2, SimCity 4, and Spore. Files are similar to ZIP archives and hold
  other files, usually compressed. The extension is irrelevant; the first four bytes are the ASCII
  magic `DBPF`.
  Source: https://wiki.sc4devotion.com/index.php?title=DBPF
  Source: https://simswiki.info/wiki.php?title=DBPF (titled "DatabasePackedFile")
- The general structure is: 96-byte header, then the file bodies, then the index entries.
  Source: https://wiki.sc4devotion.com/index.php?title=DBPF
- A community-maintained, more recent specification exists ("The DBPF File Format Community
  Specification", part of the DBPF.js project) that explicitly says it is compiled without official
  EA/Maxis feedback and is therefore partly speculative.
  Source: https://anonhostpi.github.io/DBPF.js/docs/other/spec
  Source: https://anonhostpi.github.io/DBPF.js/docs/other/spec/DBPF

### 1.2 Header field layout, source by source

Three public sources give a header layout. They are shown side by side in section 1.3.

Source A, SC4D Encyclopaedia (general DBPF, covers Sims 2 / SimCity 4 / Spore), gives explicit
offsets:
https://wiki.sc4devotion.com/index.php?title=DBPF

Source B, SimsWiki "DatabasePackedFile", gives the same field list but without offsets (and is
effectively the same text):
https://simswiki.info/wiki.php?title=DBPF

Source C, the "Spore DBPF" page (SC4D, copied from CustomSims3, original forum post now dead),
gives a C-style comment block without numeric offsets:
https://wiki.sc4devotion.com/index.php?title=Spore_DBPF
Mirror: https://simswiki.info/wiki.php?title=Spore:DBPF

Source D, the DBPF.js community spec, gives a field/byte/version table without numeric offsets:
https://anonhostpi.github.io/DBPF.js/docs/other/spec/DBPF

### 1.3 Side-by-side header layout

Source A (SC4D) offsets, quoted verbatim from the page:

| Offset | Name | Notes (Source A) |
|---|---|---|
| 0x00 | Identifier char[4] | `DBPF` |
| 0x04 | Major Version u32 | 1.0 SimCity 4 / Sims 2; 1.1 Sims 2; 2.0 Spore / Sims 3; 3.0 SimCity |
| 0x08 | Minor Version u32 | |
| 0x0C | User Major Version u32 | additional version field |
| 0x10 | User Minor Version u32 | |
| 0x14 | unknown (2.0) u32 | "Should always be zero in DBPF 2.0." |
| 0x18 | Date Created u32 | unix timestamp (1.0) |
| 0x1C | Date Modified u32 | unix timestamp (1.0) |
| 0x20 | Index major version u32 | always 7 in Sims 2 / SimCity 4; 0 for SPORE in 2.0 |
| 0x24 | Index entry count u32 | number of entries in the Index Table |
| 0x28 | Offset of first Index entry u32 | (< 2.0) |
| 0x2C | Index size u32 | size of the Index table in bytes |
| 0x30 | Hole entry count u32 | (< 2.0) |
| 0x34 | Hole offset u32 | (< 2.0) |
| 0x38 | Hole size u32 | (< 2.0) |
| 0x3C | Index minor version u32 | 01 = 7.0, 02 = 7.1; in DBPF 2.0 it has value 03 |
| 0x40 | Index offset u32 | offset to Index table in DBPF 2.0 |
| 0x44 | unknown (2.0) u32 | |
| 0x48 | reserved char[24] | ends at 0x60 (96 bytes total) |

Source C (Spore DBPF page) layout, in the source's own order (no numeric offsets given):

| Position (1-based DWord) | Source C field |
|---|---|
| 1 | `DWord type_name [0x46504244 "DBPF"]` |
| 2 | `DWord version_maj [2]` |
| 3 | `DWord version_min [0]` |
| 4 | `DWord ???` |
| 5 | `DWord ???` |
| 6 | `DWord ???` |
| 7 | `DWord ??? [created?]` |
| 8 | `DWord ??? [modified?]` |
| 9 | `DWord ??? [index_maj?]` |
| 10 | `DWord entry_count` |
| 11 | `Dword old_version_field [formerly the entry point]` |
| 12 | `DWord index_size [index size]` |
| 13 | `Dword ??? [num_holes?]` |
| 14 | `Dword ??? [hole_loc?]` |
| 15 | `Dword ??? [hole_size?]` |
| 16 | `Dword ??? [index_min?] [3]` |
| 17 | `DWord index_start_V2 [file offset to seek to for index start]` |
| 18 | `DWord padding[7]` |

Note: Source C has 17 DWords (68 bytes) plus `padding[7]` (28 bytes), which sums to 96 bytes. It
also keeps an unnamed DWord in positions 4, 5, and 6, i.e. it agrees with Source A that there is a
field at 0x14 (Source A's "unknown 2.0"), and it annotates the field at position 16 (Source A's
0x3C, "index minor version") as `[3]`.

Source D (DBPF.js) field list, in order, with stated byte sizes and applicable versions:

| Order | Field | Bytes | Versions |
|---|---|---|---|
| 1 | Magic Number | 4 | All |
| 2 | Major Version | 4 | All |
| 3 | Minor Version | 4 | All |
| 4 | User Major | 4 | 2.0 |
| 5 | User Minor | 4 | 2.0 |
| 6 | Date Created | 4 | 1.0 |
| 7 | Date Modified | 4 | 1.0 |
| 8 | Index Major | 4 | < 2.0 |
| 9 | Index Count | 4 | All |
| 10 | Index First | 4 | < 2.0 |
| 11 | Index Size | 4 | All |
| 12 | Trash Count | 4 | < 2.0 |
| 13 | Trash Offset | 4 | < 2.0 |
| 14 | Trash Size | 4 | < 2.0 |
| 15 | Index Minor | 4 | < 2.0 |
| 16 | Index Offset | 4 | 2.0 |
| 17 | Unknown | 4 | 2.0 |
| 18 | Reserved | 32 | All |

### 1.4 Where the header sources disagree

This is the most important part of the header research. The disagreements are real and must not be
silently resolved.

1. The field at offset 0x14 (Source A "unknown 2.0", Source C unnamed DWord position 6).
   Source D omits any field between "User Minor" and "Date Created". Consequences: if Source D's
   list is laid out contiguously from 0x00, everything from "Date Created" onward lands 4 bytes
   earlier than in Source A, and Source D's 17 four-byte fields plus a 32-byte Reserved region sum
   to 100 bytes, not 96. Source A and Source C both sum to 96 bytes. So Source D is internally
   inconsistent with a 96-byte header, OR it intends an omitted field. This is a genuine
   disagreement about the middle of the header.
   Sources: https://wiki.sc4devotion.com/index.php?title=DBPF ,
   https://wiki.sc4devotion.com/index.php?title=Spore_DBPF ,
   https://anonhostpi.github.io/DBPF.js/docs/other/spec/DBPF

2. Which major version Spore uses.
   Sources A, C, and D all say Spore is DBPF archive version 2.0:
   - Source A: "2.0 seen in Spore, The Sims 3" and "3.0 seen in SimCity".
   - Source C: `version_maj [2]`.
   - Source D known-versions table: Spore -> archive 2.0, index n/a.
   Our own measurement reports major version 3 in `Spore_Game.package` (team-lead's finding, not a
   public source). This directly conflicts with all three public sources. Resolution is required
   before relying on either. Note also that Source C annotates the header field at 0x3C
   (index minor version) as `[3]`, which may be the origin of a "3" if the wrong offset was read.
   Sources: as in item 1.

3. The exact meaning of the field at 0x40.
   Source A calls it "Index offset" and restricts it to DBPF 2.0. Source C calls it
   `index_start_V2` ("file offset to seek to for index start"). Source D calls it "Index Offset"
   for 2.0 ("also known as Index Position"). The three agree in intent. However team-lead measured
   that this field equals exactly (file size - 96) for `Spore_EP1_Locale_02.package`. No public
   source explains that relationship, so it is an open item (see section 1.6).

4. The "index size" field versus the Spore index prefix lengths.
   Source A and Source D call offset 0x2C "Index size" / "Index Size" (size of the index table in
   bytes). Source C (Spore) additionally constrains it arithmetically per index type:
   type 4 -> `8 + (entry_count * 28)`, type 5 -> `12 + (entry_count * 24)`,
   type 6 -> `12 + (entry_count * 24)`, type 7 -> `16 + (entry_count * 20)`.
   These are compatible (Source C just adds formulas), so this is a refinement, not a conflict.
   Sources: https://wiki.sc4devotion.com/index.php?title=Spore_DBPF

### 1.5 Index table formats, documented publicly

Source A and Source B (general DBPF) describe two legacy index-entry formats:

- DBPF 1.x, index table 7.0, 20 bytes per entry, field order:
  Type ID (u32), Group ID (u32), Instance ID (u32), location of file in archive (u32),
  size of file (u32).
- DBPF 1.x, index table 7.1, 24 bytes per entry, field order:
  Type ID (u32), Group ID (u32), Instance ID (u32), Second Instance ID (u32),
  location of file in archive (u32), size of file (u32).
  Sources: https://wiki.sc4devotion.com/index.php?title=DBPF , https://simswiki.info/wiki.php?title=DBPF

Source B additionally warns of a third Sims 2 index format whose structure is unknown (an "index
minor version 0", informally "7.-1"), seen only in shipped files.
Source: https://simswiki.info/wiki.php?title=DBPF

Source C (Spore DBPF page) describes a different, four-way scheme keyed by an `index_type` DWord
whose known values are 4, 5, 6, and 7. This is the layout that matters most for Spore:

- `index_type == 4`: prefix 8 bytes (`DWord index_type`, `DWord index_null`), then per entry
  28 bytes: type, group, instance, chunk_offset, disk_size (OR'd with 0x80000000),
  mem_size (decompressed size), compressed (Word: 0 = no, 0xFFFF = yes), unknown (Word: 1).
  Constraint: `8 + (entry_count * 28) == index_size`.
- `index_type == 5`: prefix 12 bytes (`DWord index_type`, then a shared `DWord type`,
  `DWord index_null`), then per entry 24 bytes: group, instance, chunk_offset, disk_size,
  mem_size, compressed (Word), unknown (Word).
  Constraint: `12 + (entry_count * 24) == index_size`.
- `index_type == 6`: same sizes and constraint as type 5 (Source C lists it identically).
- `index_type == 7`: prefix 16 bytes (`DWord index_type`, `DWord type`, `DWord group`,
  `DWord index_null`), then per entry 20 bytes: instance, chunk_offset, disk_size, mem_size,
  compressed (Word), unknown (Word).
  Constraint: `16 + (entry_count * 20) == index_size`.

Source C states the design rationale: type 4 has a unique type, group, and instance per entry;
types 5 and 6 share a single type across all entries; type 7 shares a single type and group across
all entries, so entries are shorter.
Source: https://wiki.sc4devotion.com/index.php?title=Spore_DBPF
Mirror: https://simswiki.info/wiki.php?title=Spore:DBPF

Source D describes the DBPF v2.0 index entry as a "header entry" plus per-entry data totalling 32
bytes, where the mode flag (also called "Index Type") is a bitmask that selects which fields come
from the shared header entry (bit 0 = Type ID, bit 1 = Group ID, bit 2 = lower instance, and so on,
read right to left). Per-entry fields listed: Type ID (4), Group ID (4), Instance ID High (4),
Instance ID Low (4), Offset (4), Size-File (4), Size-Memory (4), Compression Flag (2), Unknown (2).
Source: https://anonhostpi.github.io/DBPF.js/docs/other/spec/DBPF

Conflict note: Source C's Spore entry field order is type, group, instance, chunk_offset,
disk_size, mem_size, compressed(Word), unknown(Word), i.e. one 32-bit instance field and the size
fields before the compressed flag. Source D's v2.0 entry splits the instance into a high and low
32-bit pair and places the compression flag after both size fields, with the whole entry nominally
32 bytes. These are related but not stated identically, and Source C's entries are 20/24/28 bytes
depending on index type while Source D says "32 bytes total". This is a genuine conflict in the
level of detail and in the field ordering/naming.

### 1.6 Mapping of team-lead's measured header fields to public sources

These are OUR measurements; the public-source interpretation is offered as a cross-check only.

- magic `DBPF` at 0x00: matches all sources.
- major version 3 at 0x04: conflicts with Sources A, C, D (all say Spore is 2.0). See 1.4 item 2.
- 32-bit field that is 11529 in `Spore_Game.package` and 25535 in `PatchData.package`, suspected
  index entry count: this matches the field at offset 0x24, which Sources A, C, and D all call
  "Index entry count" / "entry_count" / "Index Count". So the public interpretation supports the
  "index entry count" hypothesis.
- 32-bit field at 0x2C: Sources A and D call offset 0x2C "Index size" / "Index Size". Source C
  calls it `index_size`. Consistent across sources.
- 32-bit file offset at 0x40 that equals (file size - 96) for `Spore_EP1_Locale_02.package`:
  Sources A, C, and D call offset 0x40 the index offset / `index_start_V2` / "Index Offset" for
  DBPF 2.0. If the value truly equals file size minus 96, then the index table would begin 96
  bytes before end of file, which is plausible only for a very small index; no public source
  documents the "file size - 96" relationship, so this is flagged as not found publicly and needs
  our own verification (for example, confirm whether a locale package genuinely has a tiny index,
  or whether 0x40 holds a different quantity in this package).

### 1.7 Not found publicly (header)

- No public source explains the exact "file size - 96" relationship for the 0x40 field in a Spore
  locale package.
- No public source provides a fully offset-annotated header layout specifically measured from a
  shipped Spore 1.x or Galactic Adventures package. The only Spore-specific header layout (Source
  C) is a comment block copied from a now-dead CustomSims3 forum thread
  (http://www.customsims3.com/forum1/YaBB.pl?num=1214408105), which both SC4D and SimsWiki note is
  404 as of 2012.

---

## 2. Compression (RefPack / QFS)

### 2.1 Documented publicly

- Spore and its relatives compress individual package entries with a scheme called RefPack (also
  written QFS). The SC4D DBPF page links a "DBPF Compression" page and a "QFS compression" page
  (the latter does not exist as a standalone article).
  Source: https://wiki.sc4devotion.com/index.php?title=DBPF
- The SimsWiki DBPF page likewise says entries may be compressed with RefPack.
  Source: https://simswiki.info/wiki.php?title=DBPF
- The SC4D "DBPF Compression" page contains a full description of the algorithm plus a PHP
  reference implementation (credited to Delphy, based on dmchess's Perl) and a canonical RefPack
  description credited to FrANK G. Barchard (EAC).
  Source: https://wiki.sc4devotion.com/index.php?title=DBPF_Compression

### 2.2 The RefPack control-byte encoding (documented publicly)

From https://wiki.sc4devotion.com/index.php?title=DBPF_Compression :

- The stream is a sliding-window LZSS method with a 131k window and byte-oriented coding.
- The first control byte is simply the first byte of the compressed payload; there is no separate
  magic marker. The parser reads one byte and branches on its high bits:
  - `0xFC..0xFF`: literal (short) / EOF
  - `0xE0..0xFB`: literal (long)
  - `0xC0..0xDF`: very long reference
  - `0x80..0xBF`: long reference
  - `0x00..0x7F`: short reference
- Command encodings (the page uses `p` = number of plain-text bytes, `c` = number of bytes to copy,
  `o` = copy offset, `d` = plain-text count in the codex):

| Range | Type | Control length | Plain bytes | Copy length | Max offset |
|---|---|---|---|---|---|
| `0x00-0x7F` | Short reference | 2 | 0-3 | 3-10 | 1024 |
| `0x80-0xBF` | Long reference | 3 | 0-3 | 4-67 | 16384 |
| `0xC0-0xDF` | Very long reference (SC4 variant) | 4 | 0-3 | 5-2047 | 65535 |
| `0xC0-0xDF` | Very long reference (Sims 2 variant) | 4 | 0-3 | 5-1028 | 131072 |
| `0xE0-0xFB` | Literal (long) | 1 | 4-112 | 0 | n/a |
| `0xFC-0xFF` | Literal / EOF (short) | 1 | 0-3 | 0 | n/a |

- Short reference bit layout: `0oocccpp oooooooo`.
  Plain = `byte0 & 0x03`; copy = `((byte0 & 0x1C) >> 2) + 3`;
  offset = `((byte0 & 0x60) << 3) + byte1 + 1`.
- Long reference bit layout: `10cccccc ppoooooo oooooooo`.
  Plain = `((byte1 & 0xC0) >> 6) & 0x03`; copy = `(byte0 & 0x3F) + 4`;
  offset = `((byte1 & 0x3F) << 8) + byte2 + 1`.
- Sims 2 very-long reference bit layout: `110occpp oooooooo oooooooo cccccccc`.
  Plain = `byte0 & 0x03`; copy = `((byte0 & 0x0C) << 6) + byte3 + 5`;
  offset = `((byte0 & 0x10) << 12) + (byte1 << 8) + byte2 + 1`.
- Literal long: `111ppppp`, plain = `((byte0 & 0x1F) << 2) + 4` (the page also gives the shorthand
  `(C - 0x7F) * 4`, so `0xE0` reads 4 plain bytes).
- Literal short / EOF: `111111pp`, plain = `byte0 & 0x03`.
- The page stresses that multi-byte bit groups are concatenated (shifted), not added, and that
  copying must be done one character at a time to support overlapping self-referential copies
  (offset 0 means "copy the last output character").

The canonical RefPack codex reproduced on that page (credited to FrANK G. Barchard, EAC) reads:

```
refpack is a sliding window (131k) lzss method, with byte oriented coding.
huff fb5 style header:
     *10fb  fb5      refpack 1.0  reference pack
header:
[10fb] [unpacksize] [totalunpacksize]
  2         3
format is:
0ffnnndd_ffffffff          short ref, f=0..1023,n=3..10,d=0..3
10nnnnnn_ddffffff_ffffffff long ref, f=0..16384,n=4..67,d=0..3
110fnndd_f.._f.._nnnnnnnn  very long,f=0..131071,n=5..1028,d=0..3
111ddddd                   literal, d=4..112
111111dd                   eof, d=0..3
```

The page also recommends a C implementation by Wouangaine as more reliable than its PHP sample:
https://github.com/wouanagaine/SC4Mapper-2013/blob/master/Modules/qfs.c

### 2.3 The "0x10 0xFB" convention and the 5-byte vs 9-byte compressed header

This is a place where sources conflict on the wrapper header (not on the payload algorithm).

- General DBPF (Sources A and B) say the compressed file header is 9 bytes:
  `DWORD` compressed size of file, `WORD` compression ID (they write `0x10FB`, QFS), then 3 bytes
  uncompressed size of the file. They also mention a 4-byte preceding length field that is absent in
  older DBPF files.
  Sources: https://wiki.sc4devotion.com/index.php?title=DBPF ,
  https://wiki.sc4devotion.com/index.php?title=DBPF_Compression , https://simswiki.info/wiki.php?title=DBPF
- Source C (Spore DBPF) says that in Spore the compressed chunk header is 5 bytes, not 9, and that
  these 5 bytes are counted inside the compressed size given by the index. The first two bytes are
  the compression type `0x10 0xFB` or `0x50 0xFB` (Source C also writes the alternative reading
  `[0xFB10 or 0xFB50]`), and the next three bytes are the decompressed size in Big-Endian.
  Source: https://wiki.sc4devotion.com/index.php?title=Spore_DBPF

Conflict: 9-byte header (general DBPF, Sources A/B) versus 5-byte header (Spore, Source C). Also
the byte-order of the compression ID is written inconsistently across sources (`0x10FB` as a WORD
versus the byte pair `0x10 0xFB` versus `0xFB10`).

Team-lead observes `0x10 0xFB` at the start of compressed payloads. That is consistent with Source
C's byte pair for Spore. It is NOT the same as Source A's "WORD 0x10FB" unless read as a
little-endian WORD `0xFB10`.

### 2.4 Compression IDs and the zlib question

- Source C documents two Spore compression type byte pairs: `0x10 0xFB` and `0x50 0xFB`, and says
  both appear to decompress the same way.
  Source: https://wiki.sc4devotion.com/index.php?title=Spore_DBPF
- Source D (DBPF.js, v2.0 table entry) lists compression flag values: `0xFFFF` = RefPack,
  `0xFFFE` = streamable RefPack, `0x5A42` = zlib, and a supposed `0xFFE0` for deleted entries.
  Source: https://anonhostpi.github.io/DBPF.js/docs/other/spec/DBPF
  However, Source D states the community spec is biased toward The Sims 4. So the zlib flag
  (`0x5A42`) is documented for the DBPF family but not confirmed for Spore.
- The SC4D "DBPF Compression" page does NOT mention `0x50FB` or zlib anywhere in the extracted
  content.

Conclusion: RefPack/QFS is the documented compression for Spore packages. zlib is documented as a
possible DBPF v2.0 compression flag in a Sims 4-biased spec, but whether Spore ever uses zlib is
"not found publicly" and should be treated as unverified.

### 2.5 Not found publicly (compression)

- No source was found that documents a Spore-specific compressed-entry size accounting beyond
  Source C's statement that the 5-byte header is included in the index's compressed size.
- No source was found confirming zlib usage inside Spore packages specifically.

---

## 3. Resource type IDs

The full numeric table is in `docs/resource-types.md`. Summary of the sourcing:

- The SimsWiki "Spore:Chunk Types" page lists about 60 Spore chunk type IDs (hex) with short names,
  several marked as added in 2008.
  Source: https://simswiki.info/wiki.php?title=Spore:Chunk_Types
- SporeModder FX ships `reg_type.txt`, a large name-to-hex mapping of resource types (and, further
  down, of Simulator class hashes and group/palette names).
  Source: https://raw.githubusercontent.com/Spore-Community/SporeModder-FX/master/reg_type.txt
- The two sources agree on many IDs (for example `0x00B1B104` = prop, `0x00E6BCE5` = gmdl,
  `0x2F7D0004` = png, `0xEE17C6AD` = anim, `0xEFBDA3FF` = layout, `0xCF6C21B8` = xml).
- They disagree on a few names (for example `0x022D2C83` is "EAPD" in SimsWiki but "pdr" in
  reg_type.txt; `0x02FAC0B6` is "textual" in SimsWiki but "locale" in reg_type.txt;
  `0x03055F61` is "textual" in SimsWiki but "instrument" in reg_type.txt). See `resource-types.md`.

---

## 4. SPORE-specific formats

### 4.1 `.prop` property files (type `0x00B1B104`)

Documented publicly in detail on SimsWiki (copied from the same CustomSims3 thread):

- A `.prop` file is a list of "variable = value" items.
- The file begins with a `uint32` count of items, stored BIG-ENDIAN. If the count is 0, the file ends.
- Each item is: `identifier` (u32, BE), `type` (u16, BE), `specifier` (u16, BE). If the specifier is
  `0x009C` an array follows, preceded by `ArrayNumber` (u32, BE) and `ArraySize` (u32, BE). Any other
  specifier value means a single value.
- Endianness is mixed: most numeric types are big-endian, but `key`, `vector2/3/4`, and the color
  types are little-endian.
- Property type IDs documented: `0x01` bool, `0x09` int32, `0x0a` uint32, `0x0d` float,
  `0x12` string8, `0x13` string (UTF-16), `0x20` key (three u32 little-endian, commonly File/Type/
  Group), `0x22` texts, `0x30` vector2, `0x31` vector3, `0x32` colorRGB, `0x33` vector4,
  `0x34` colorRGBA, `0x38` transform, `0x39` bbox.
- Property identifiers are hashed names or Maxis-assigned numbers, recovered from a lookup list
  (`properties.txt` referenced as living in `main.package`, coded `0x00000000-0x658B0AB2-0x024A0E52`).
- Padding rules vary per type and per single-versus-array context (for example a single `key` is
  followed by an extra u32; a single `vector3` is followed by one u32; arrays have no such padding).
  Source: https://simswiki.info/wiki.php?title=Spore:00B1B104

Note on `.prop` to XML: the SimsWiki page does NOT describe an XML mapping. The existence of
`.prop` <-> `.xml` conversion tools (Gibbed's `Gibbed.Spore.PropToXML.exe` and
`Gibbed.Spore.XMLToProp.exe`) is documented only in a 2008 forum post, not in a format spec.
Source: https://game.ali213.net/thread-2302357-1-1.html

### 4.2 Name / property hashing

Documented publicly: the Spore name hash is the FNV-1 style hash with offset basis `0x811C9DC5` and
prime `0x1000193`, computed over the lowercased ASCII name:

```
hash = 0x811C9DC5;
while (*p) { hash *= 0x1000193; hash ^= (*p); p++; }
```

Source: https://simswiki.info/wiki.php?title=Spore:HashVal (originally written by Jongware, copied
from the same CustomSims3 thread). Corroborated by the existence of the
`SporeHash-VSCode` tool ("Convert names <-> FNV Hash IDs used in Spore"):
https://github.com/Spore-Community/SporeHash-VSCode

### 4.3 `.spore` / creation files and the Spore PNG creation format

Documented publicly:

- Creations (creatures, buildings, vehicles, adventures, and the like) are shared as PNG images
  whose ancillary chunks carry the encoded creation data, so a single PNG file contains both a
  thumbnail and the full creation.
  Source: https://github.com/Spore-Community/PNG-Decoder-Python
- The 2008 decoders by ymgve and rick decode the PNG into a metadata header (name, asset ID,
  description, tags, author, parent creation) plus an XML document containing the 3D model and
  paints, including parts (rigblocks), their IDs, locations, and connections.
  Source: https://github.com/Spore-Community/PNG-Decoder-Python
- Adventures use a different, newer format that the old Python decoder cannot handle; a newer .NET
  Core decoder exists.
  Source: https://github.com/Spore-Community/PNG-Decoder-Python
- The Spore Community Hub states it has "fully decoded the format used to store creations in PNG
  files" and can extract name, author, description, and tags from a creation PNG alone.
  Source: https://spore-community.github.io/
- The original illustrated explanation of the PNG format was published in 2008 by rouli.net.
  Source (referenced by the decoder repo): http://www.rouli.net/2008/08/spores-png-format-illustrated.html
  (the live page could not be fetched during this research; treat the exact URL as reported by the
  decoder repo rather than independently verified)

### 4.4 Locale packages

Documented publicly (partially): locale text is stored in packages such as
`Spore_EP1_Locale_02.package`, and the Spore community describes locale data as being edited by
mods that create localization files. The `Server-IDs` repository and the `Spore-AdventureLocalizer`
mod ("creates localization files from adventures") exist.
Sources: https://github.com/Spore-Community (repository list) , https://github.com/Spore-Community/Server-IDs
A dedicated specification of the locale package internal structure was NOT found publicly. The
team-lead's measurement that the 0x40 field equals (file size - 96) in
`Spore_EP1_Locale_02.package` is therefore an original finding, not a documented one.

### 4.5 `Config/*.txt` config format

Not found publicly. A 2026 Chinese installation guide references a `Data/Config` folder inside the
game directory but gives no format specification:
https://tieba.baidu.com/p/5211514921
Some Spore configuration values are known to live in `.prop` resources (the property dump on
SimsWiki includes settings such as `frameLimitMS`, `memoryLimit`, `showFPS`, `renderUI`,
`renderEffects`, `showProfiler`).
Source: https://simswiki.info/wiki.php?title=Spore:00B1B104
No standalone `Config/*.txt` grammar was located.

### 4.6 Models, meshes, animation, shaders

Documented publicly (names and tooling, limited format detail):

- Resource type IDs exist for `gmdl` (`0x00E6BCE5`), `gmsh` (`0x01C135DA`), `rw4` (`0x2F4E681B`),
  `anim` (`0xEE17C6AD`), `animation` (`0xEE17C6AD`), `mcl` (`0x3C7E0F63`), `cnv`
  (`0x055ADA24`), `bmdl` (`0x72047DE2`), `physx_bin` (`0x76E1259D`), `physx_xml` (`0x7EAB18FD`),
  and shader/effect types such as `cfx` (`0x278CF8F2`).
  Source: https://raw.githubusercontent.com/Spore-Community/SporeModder-FX/master/reg_type.txt
- SporeModder FX can visualize most Spore-format models and has Blender add-ons that import
  `raster-dds`, `gmdl`, `gmsh`, and `rw4`, with support for animation and morph animation.
  Sources: https://github.com/Spore-Community/SporeModder-FX ,
  https://dl.3dmgame.com/patch/106978.html
- SporeModder FX also ships an Animation Editor and an Effect Editor.
  Source: https://github.com/Spore-Community/SporeModder-FX
- A precise binary layout for gmdl/gmsh/rw4 was NOT found as a standalone public spec; the
  knowledge appears to live in the tool source code and in the Blender add-ons.

### 4.7 Pollinator / Sporepedia / online backend

- Wikipedia does NOT use the term "Pollinator". It describes the equivalent systems: creations are
  uploaded to the Sporepedia online server; content is "asynchronous sharing" (a "massively
  single-player online game"); user content is automatically uploaded, catalogued, quality-rated,
  and redistributed to populate other players' games. It also describes a public RESTful Spore API
  returning XML, and the Sporecast RSS feed.
  Source: https://en.wikipedia.org/wiki/Spore_(2008_video_game)
- The internal Spore term "pollen" does appear in the resource data: `pollen_metadata`
  (`0x030BDEE3`), `pollen_metadata.bak` (`0xE1E7F839`), and palette/group names `UserPollinated`
  (`0x89CA8A12`) and `UserCreated` (`0x37216F9C`). This is consistent with "pollination" being the
  internal name for creation sharing, but no public prose document states that mapping.
  Source: https://raw.githubusercontent.com/Spore-Community/SporeModder-FX/master/reg_type.txt
- The Spore Community Hub is building a replacement Spore server and states its intent to retain
  compatibility with the existing Spore API and the game's built-in online features.
  Source: https://spore-community.github.io/
- A community reimplementation of the Spore server exists: https://github.com/Rosalie241/SporeServer
  Source (link): https://spore-community.github.io/

---

## 5. The engine

Documented publicly:

- Developer Maxis, publisher Electronic Arts, 2008, Windows and Mac OS X. Lead programmer Andrew
  Willmott is the only programming credit listed.
  Source: https://en.wikipedia.org/wiki/Spore_(2008_video_game)
- No engine name is given in the Wikipedia article. The article does not name a scripting language
  or describe internal subsystems (rendering, physics, AI middleware) beyond audio.
  Source: https://en.wikipedia.org/wiki/Spore_(2008_video_game)
- Procedural generation is used extensively: Will Wright stated a creature's generating information
  is "only a couple of kilobytes", and content is small because of procedural generation, which is
  what makes asynchronous upload/download practical. Creature painting uses many textures,
  overlays, colours, and patterns applied procedurally based on creature topology.
  Source: https://en.wikipedia.org/wiki/Spore_(2008_video_game)
- Audio was implemented using a modified version of Pure Data (Miller Puckette). Generative music
  credits include Brian Eno, Peter Chilvers, Kent Jolly, Aaron McLeran, Cliff Martinez, Jerry
  Martin, Saul Stokes, and Marc Russo.
  Source: https://en.wikipedia.org/wiki/Spore_(2008_video_game)
- There are eighteen editors sharing the same general UI and controls.
  Source: https://en.wikipedia.org/wiki/Spore_(2008_video_game)

Engine subsystem names as they appear inside the game's own data (evidence, not prose docs):

- SporeModder FX ships `reg_simulator.txt` and `reg_simulator_stub.txt` alongside `reg_file.txt`,
  `reg_property.txt`, `reg_type.txt`, and `reg_cnv.txt`, which indicates the game has a distinct
  "simulator" subsystem with its own name registry.
  Source: https://github.com/Spore-Community/SporeModder-FX (repository file list)
- The `reg_type.txt` class-hash section contains many `c...Simulator` class names, including
  `cSimulatorSpaceGame` (`0x21F15DD`), `cSimulatorUniverse` (`0x2A8BFBE`), `cSimulatorPlayerUFO`
  (`0x2F011ED`), `cSimulatorWalkAround` (`0x2F404EA`), `cCitySimulator` (`0x21F1187`),
  `cCombatSimulator` (`0x23BA44D`), `cPosseSimulator` (`0x21F11AE`), plus managers such as
  `cMissionManager`, `cDramaManager`, `cEmpireGrowthSim`, `cRelationshipManager`, and record types
  `cPlanetRecord`, `cStarRecord`. This is direct evidence of a class/object simulator architecture.
  Source: https://raw.githubusercontent.com/Spore-Community/SporeModder-FX/master/reg_type.txt

Relationship to the Sims engines: Spore, The Sims 2, The Sims 3, and SimCity 4 share the DBPF
container and (per the SC4D/SimsWiki pages) the RefPack compression, which shows a shared Maxis
toolchain lineage. No public source was found that states Spore's runtime engine is the same as any
Sims engine; the class names above are Spore-specific. Treat "related lineage, distinct runtime" as
the defensible position.

Not found publicly: an official engine name, an official scripting language, and an official
description of the renderer or physics stack for Spore.

---

## 6. Existing open-source tooling, licenses, and reuse

This section is also reproduced in condensed form in the "licenses and reuse" part of
`resource-types.md` (see section 8 there). Reuse here means code reuse; documentation findings from
any tool can be learned from regardless of license, but copying code is a different matter.

### 6.1 Tools and libraries

| Tool / project | URL | License (as stated on the page) | What it does |
|---|---|---|---|
| SporeModder FX | https://github.com/Spore-Community/SporeModder-FX | GPL-3.0 | The main modern Spore data modding tool: text format for Spore files, UI editor, image/texture viewer, model viewer, animation editor, effect editor, DBPF handling |
| SporeModder FX Blender Addons | https://github.com/Spore-Community/SporeModder-Blender-Addons | GPL-3.0 | Blender add-ons to import Spore models (raster-dds, gmdl, gmsh, rw4) |
| Spore ModAPI SDK | https://github.com/Spore-Community/Spore-ModAPI | no license shown | C++ SDK to write Spore mods (DLL injection); includes a Ghidra import script and name/data files |
| ModAPI Launcher Kit | https://github.com/Spore-Community/ModAPI-Launcher-Kit | MIT | Installs, manages, and loads ModAPI mods |
| SporeTools | https://github.com/Spore-Community/SporeTools | MIT | Browser-based Spore tools (tagged `dbpf`) |
| SporeHash-VSCode | https://github.com/Spore-Community/SporeHash-VSCode | MIT | Convert names to and from FNV hash IDs used in Spore |
| Spore-CreationZIP-Porter | https://github.com/Spore-Community/Spore-CreationZIP-Porter | MIT | Import/export creations via ZIP / .sporemod files |
| SporeGlobals-Python | https://github.com/Spore-Community/SporeGlobals-Python | MIT | Python primitives for Spore data |
| Spore-AdventureLocalizer | https://github.com/Spore-Community/Spore-AdventureLocalizer | MIT | Proof-of-concept mod that creates localization files from adventures |
| PNG-Decoder-Python | https://github.com/Spore-Community/PNG-Decoder-Python | no license shown | 2008 Spore PNG creation decoder by ymgve and rick |
| PNG-Decoder-NetCore | https://github.com/Spore-Community/PNG-Decoder-NetCore | no license shown | Newer .NET Core Spore PNG decoder (handles adventures) |
| Adventure-Decoder | https://github.com/Spore-Community/Adventure-Decoder | no license shown | Decodes Spore Adventure files (tagged `dbpf`) |
| Spore_EP1Stegano-Python | https://github.com/Spore-Community/Spore_EP1Stegano-Python | Unlicense | EP1 model-in-picture steganography |
| sporeget | https://github.com/Spore-Community/sporeget | Unlicense | Retrieves API links for published creations |
| Spore-Community organization listing (all 22 repos and licenses) | https://github.com/orgs/Spore-Community/repositories | mixed | Reference for the above |
| pyDbpf | (name encountered in community references) | not verified | Python DBPF library. The URL `https://github.com/kmarquette/pyDbpf` returned HTTP 404 during this research and `https://pypi.org/project/pyDbpf/` also returned 404, so its current location and license are "not found publicly" |
| S3PI (Sims 3 Package Interface), by Peter L Jones | https://github.com/marcos4503/sims3-package-interface , original at https://sourceforge.net/projects/s3pi/ | GPL-3.0 | .NET core library that understands Sims 3 (and partly Sims 4 / SimCity) package files; supports DBPF version 2 (Sims 3) and limited version 3 (SimCity 5) |
| DBPF.js | https://github.com/anonhostpi/DBPF.js | license stated to exist in repo but not displayed in the extracted page content; verify before reuse | TypeScript DBPF reader, supports DBPF v1.0-v2.x with notes for v3; also publishes the community DBPF spec |
| chieltbest/dbpf | https://github.com/chieltbest/dbpf | multi-license (repo uses a `LICENSES` directory and `REUSE.toml`; exact per-file license not confirmed here) | Rust library and tools for DBPF archives; README states it works for any DBPF game including Spore, with best resource support for The Sims 2 |
| kitlith/sims3-rs | https://github.com/kitlith/sims3-rs | not verified | Rust crate for Sims 3 package handling (referenced from the DBPF.js spec) |
| Gibbed.Spore | https://github.com/gibbed/Gibbed.Spore | not verified | Gibbed's Spore tools (referenced from the DBPF.js spec; the PropToXML / XMLToProp tools are the historically known ones) |
| DBPFSharp | https://github.com/0xC0000054/DBPFSharp | not verified | C# DBPF library (referenced from the DBPF.js spec) |
| jDBPFX | https://github.com/memo33/jDBPFX | not verified | Java DBPF library (referenced from the DBPF.js spec) |
| sc4-dbpf-loading | https://github.com/0xC0000054/sc4-dbpf-loading | not verified | SimCity 4 DLL plugin that optimizes DBPF loading |
| qfs.c (Wouangaine) | https://github.com/wouanagaine/SC4Mapper-2013/blob/master/Modules/qfs.c | not verified | C implementation of QFS/RefPack decompression |

The DBPF.js community spec contains a long "Background and Acknowledgements" list naming many more
DBPF implementations across C#, Python, Java, Rust, TypeScript, C, and Tcl. That list is the single
best index of prior art for the container format:
https://anonhostpi.github.io/DBPF.js/docs/other/spec

### 6.2 License red flags for a clean-room Rust implementation

- GPL-3.0 projects (SporeModder FX, SporeModder FX Blender Addons, S3PI): their source code cannot
  be copied into a permissively licensed Rust project without forcing that project under GPL-3.0.
  Their documented findings and behaviour can still be studied. If any code is to be reused, the
  whole Rust project would have to be GPL-3.0 compatible.
- No-license projects (Spore ModAPI SDK, PNG decoders, Adventure-Decoder): "no license" means
  all rights reserved by default; do not copy code. Documentation and observable behaviour are still
  usable as research input.
- MIT / Unlicense projects (ModAPI Launcher Kit, SporeTools, SporeHash, CreationZIP-Porter,
  SporeGlobals, AdventureLocalizer, sporeget, Spore_EP1Stegano): permissive, code reuse is
  generally allowed with attribution, but confirm the actual LICENSE file because the listing was
  read from GitHub summaries.
- chieltbest/dbpf: it is a Rust DBPF implementation, which makes it the most tempting to reuse, but
  its license is a multi-license `REUSE` setup and was not resolved here. Resolve it before any
  copy. Its README also warns the tools are EXPERIMENTAL.
  Source: https://github.com/chieltbest/dbpf

---

## 7. Legal position

This section is factual only. It is not legal advice.

- EA's End User License Agreement for its software states, in the "Reservation of Rights" section:
  "Except to the extent permitted under applicable law, you may not decompile, disassemble, or
  reverse engineer the Software, or any component thereof, by any means whatsoever." It also
  states, in "Further Restrictions", that the user "may not otherwise copy, display, seek to
  disable, distribute, perform, publish, modify, create works from, or use the Software or any
  component of it, except as expressly authorized by EA", and prohibits distributing the software
  over a network.
  Source: https://store.steampowered.com/eula/24960_eula_1
  Caveat: the EULA page itself does not name the product. It is a generic EA EULA template. The
  technical signatures on that page (SecuROM, a five-machine activation limit, EA Download Manager)
  are consistent with the Spore-era EA EULA, but the page cannot be tied to Spore with certainty
  from its text alone. The clause is representative of EA's EULA wording for the period.
- The phrase "except to the extent permitted under applicable law" is the standard carve-out that
  leaves room for statutory interoperability rights in some jurisdictions. No source located here
  interprets that carve-out for Spore specifically.
- The Spore Community Hub states plainly that it is "Not associated with or endorsed by EA or
  Maxis", and its stated goals are preservation and keeping online features working.
  Source: https://spore-community.github.io/
- Community consensus (as expressed by the existence and longevity of tools such as SporeMaster,
  SporeModder, SporeModder FX, Spore ModAPI, and the archive projects) is that reverse engineering
  and modding Spore for interoperability and preservation is widely practised, and no public record
  was found of EA enforcing the reverse-engineering clause against Spore modders.
  Sources: https://github.com/Spore-Community , https://spore-community.github.io/
- Game assets (models, textures, audio, locale text, and the shipped `.package` files) remain the
  copyright of Electronic Arts. The community projects treat extracted assets as not
  redistributable; the Spore Community Hub frames its work as archiving and preservation, and the
  no-license status of several decoders means even the decoders themselves are not freely
  redistributable.
  Sources: https://spore-community.github.io/ , https://github.com/Spore-Community

Not found publicly: an official EA or Maxis statement specifically authorizing or prohibiting Spore
modding. The only located authoritative text is the general EA EULA restriction above.

---

## 8. Implications for a clean-room Rust implementation

1. Treat the header as a measured artifact first. Three public sources say Spore is DBPF 2.0, while
   our own measurement says major version 3. Do not hard-code a version check until this is
   resolved. Implement a tolerant reader that keys off the index offset field and the index type
   rather than the major version.

2. Use Source C's Spore index scheme (index_type 4/5/6/7 with 28/24/24/20-byte entries and the
   `8/12/12/16 + n * entry` size constraint) as the primary hypothesis, because it is the only
   Spore-specific index description located. Cross-check each real package by asserting the size
   constraint. If a package fails the constraint, fall back to Source A's 20/24-byte legacy entry
   layout.

3. The index entry carries both a compressed size (with the high bit OR'd in) and an uncompressed
   size, plus a Word "compressed" flag (0 or 0xFFFF). Implement the reader around that pair of
   sizes; do not infer compression from the payload alone.

4. Compression: implement RefPack from the SC4D "DBPF Compression" description, including the
   overlapping-copy rule (copy one byte at a time). Support both the 5-byte Spore chunk header
   (`0x10 0xFB` or `0x50 0xFB`, then 3-byte big-endian uncompressed size) and the 9-byte legacy
   header (compressed size, Word compression ID, 3-byte uncompressed size) behind a detection step.
   Do NOT assume zlib for Spore; the zlib flag is documented only in a Sims 4-biased spec.

5. `.prop` parsing must handle mixed endianness and per-type, per-context padding. Encode the type
   table (0x01, 0x09, 0x0a, 0x0d, 0x12, 0x13, 0x20, 0x22, 0x30-0x34, 0x38, 0x39) as data, and treat
   the specifier 0x9C as "array with ArrayNumber and ArraySize".

6. Names are FNV-1 hashes with basis 0x811C9DC5 and prime 0x1000193 over lowercased ASCII. Build a
   reverse map from the SporeModder `reg_type.txt` and `reg_property.txt` dictionaries (these are
   data files, not code, so they can be consulted for the name table; but confirm their license
   terms before shipping them).

7. Do not copy GPL-3.0 (SporeModder FX, S3PI, Blender add-ons) or unlicensed (Spore ModAPI, PNG
   decoders, Adventure-Decoder) source. Use them only to confirm behaviour and names. If any Rust
   DBPF crate is used, resolve its license first.

8. Assets stay EA's copyright. Plan for a user-supplied game-data directory at runtime rather than
   bundling extracted assets.

---

## 9. Source list

Primary format documentation:

- SC4D Encyclopaedia, DBPF: https://wiki.sc4devotion.com/index.php?title=DBPF
- SC4D Encyclopaedia, Spore DBPF: https://wiki.sc4devotion.com/index.php?title=Spore_DBPF
- SC4D Encyclopaedia, DBPF Compression: https://wiki.sc4devotion.com/index.php?title=DBPF_Compression
- SimsWiki, DatabasePackedFile: https://simswiki.info/wiki.php?title=DBPF
- SimsWiki, Spore:DBPF: https://simswiki.info/wiki.php?title=Spore:DBPF
- SimsWiki, Spore:Chunk Types: https://simswiki.info/wiki.php?title=Spore:Chunk_Types
- SimsWiki, Spore:00B1B104 (.prop): https://simswiki.info/wiki.php?title=Spore:00B1B104
- SimsWiki, Spore:HashVal: https://simswiki.info/wiki.php?title=Spore:HashVal
- DBPF.js community spec, overview: https://anonhostpi.github.io/DBPF.js/docs/other/spec
- DBPF.js community spec, DBPF: https://anonhostpi.github.io/DBPF.js/docs/other/spec/DBPF

Tooling and data:

- SporeModder FX: https://github.com/Spore-Community/SporeModder-FX
- SporeModder FX reg_type.txt: https://raw.githubusercontent.com/Spore-Community/SporeModder-FX/master/reg_type.txt
- Spore-Community repositories: https://github.com/orgs/Spore-Community/repositories
- Spore ModAPI SDK: https://github.com/Spore-Community/Spore-ModAPI
- PNG-Decoder-Python: https://github.com/Spore-Community/PNG-Decoder-Python
- S3PI: https://github.com/marcos4503/sims3-package-interface and https://sourceforge.net/projects/s3pi/
- DBPF.js: https://github.com/anonhostpi/DBPF.js
- chieltbest/dbpf (Rust): https://github.com/chieltbest/dbpf

Background and legal:

- Wikipedia, Spore (2008 video game): https://en.wikipedia.org/wiki/Spore_(2008_video_game)
- EA End User License Agreement: https://store.steampowered.com/eula/24960_eula_1
- Spore Community Hub: https://spore-community.github.io/
- Original PNG format write-up (as cited by the decoder repo): http://www.rouli.net/2008/08/spores-png-format-illustrated.html

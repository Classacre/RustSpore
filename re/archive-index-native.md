# Native archive index and resource identity

`[MEASURED]` The installed base executable has SHA256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Its actual packed-index reader `8DAFE0` corrects the historical field labels
in `docs/dbpf-format.md`. Earlier payload offsets and size measurements remain
valid; the historical group and combined-instance interpretation is superseded.
No original executable or game payload is embedded in the new Rust reader.

## Native reader chain

`[MEASURED]` Constructor `8D9600` installs database vtable `14373D8`.
Open `8D8040` reaches `8D79E0`, which reads the header through `8D8310`
and index through `8D9DD0`. Index factory `8D7A20` constructs
PFIndexModifiable through `8DAD60`, installing vtable `14374A0`.
Its slot34 is `8DAFE0`. Native v3 queries793..803 retain exact-address
dossiers for this chain. Original bytes and virtual-table observations stay
under ignored `target/rea/`.

`[MEASURED]` The first index word supplies common-field flags. Bit0 reads
one common type word; bit1 reads one common scope/group word. This reader
requires bit2 and reads the next word as a common instance-high value, which
must equal zero. Other flag bits are ignored by this body. Each resource then
reads its non-common type/scope words followed by instance-low, file offset,
stored-size word and decoded-size word. Its native key has three words in
memory: `(instance-low, type, scope)`. The 16-digit inspection ID is scope
concatenated with instance-low, not a native 64-bit instance field.

`[MEASURED]` Stored-size bit31 means an additional metadata word follows
this same record. The actual size is its low31 bits. Metadata low16 bits give
the compression tag; bit16 is the persisted committed flag. The loader has
a separate argument that can force that flag after reading. Without metadata,
the source derives tag0 when stored and decoded sizes agree and tagFFFF
otherwise. The Rust parser retains the raw metadata word and unused flag bits.

## Full installed census and implementation

`[MEASURED]` All 82 packages / 158,720 records parse exactly. All records
have explicit metadata. There are ten flag4 packages, 46 flag6 packages and
26 flag7 packages, with these physical layouts in this install:

| Shared flags | Common prefix bytes | Per-record bytes | Trailer bytes |
|---|---|---|---|
| 4 | 8 | 28 | 0 |
| 6 | 12 | 24 | 0 |
| 7 | 16 | 20 | 0 |

`[MEASURED]` The old wide view's apparent group is the previous record's
compression word, except its first row, which reads the common zero word.
The old compact views put the current metadata in that field and omit common
key fields. A one-record flag7 package also satisfies the old wide shape's
size equation, demonstrating that exact fit and payload contiguity alone do
not establish key semantics. `RecordShape`, `ResourceKey` and `PackageIndex`
remain historical compatibility interfaces; new native consumers use
`dbpf::native_index::{NativeResourceKey, NativeIndex, NativePackageIndex}`.

`[MEASURED]` The independent Python and Rust canonical records are identical:
15,531,772 TSV bytes, SHA256
`3de1933446c61546f8afcf44e22c1010a5405baf353d0d5c9259ea6071f4e7c6`.
Every package has unique native keys. All 101,312 tagFFFF resources decompress
to their exact declared size; all 57,408 tag0 resources read at their declared
raw size. Total decoded bytes are 10,447,076,586. No tag/size-rule mismatch or
unsupported tag occurs in the install. Evidence is
`target/native-archive-index-census-v1/{report,rust-comparison}.json` and
`rust-payload-verification-v1.log`.

```powershell
python tools/native_archive_index_probe.py 'D:/Games/SPORE Collection' --out target/new-native-index-census
tools/build.bat run -p dbpf --example native_index_verify -- 'D:/Games/SPORE Collection' target/new-native-index-census/rust-records.tsv --verify-payloads
```

`[MEASURED]` Five focused tests cover common fields, the one-record ambiguity,
metadata framing, implicit compression tags, truncated input/count bounds,
the source's unsupported high-word branch and explicit duplicate-key errors.
The bounded Rust API checks every read and complete index consumption rather
than reproducing unsafe pointer reads on malformed input. Other compression
tags remain typed unsupported errors when reading payloads.

## Package priority remains a separate question

`[MEASURED]` Global getter `67D7F0` returns `15FE894`; native initialization
at `7E8C13` installs an `8E1AB0` object with vtable `1437740`.
Slot50 `8E0720` registers a database through `8DE010`. That list insertion
orders signed priority values descending and inserts after equal priorities.
The previously recovered manifest values are therefore meaningful inputs to
registration. Queries767,777..785 and the bounded SDK excerpt retain this
source chain. Undefined-function output was not replaced with fallback code.

`[MEASURED]` Exact database query `8DD930` walks that list from its head,
calls each database's slot34 with the supplied native key and no output handle,
and returns the first successful database. `8DF1A0` uses this query when there
is no type mapping and no supplied database. Its other branch can substitute
type values from an ordered map; it also walks database-list order. Original
SDK excerpts retain the exact undefined bodies, and query804 establishes the
caller `8DE8A0`. No unrelated decompiler fallback was admitted.

`[MEASURED]` The original-program renderer now uses the canonical index and
typed96-bit lookup. Fresh Cell, base Creature, expansion and flora captures
are PNG-byte-identical to their earlier baselines over 2,359,296 pixels.
`target/native-archive-index-census-v1/render-regression-v1/comparison.json`
retains all four comparisons and executable SHA256
`e46c4a8c6dc0769bff7c5d6e9a2050c7f6089e44b3fcb36e1926c02fc6d413f5`.
The subsequent AssetSource, PackageSet and SourceCellLibrary migration uses
native indexes/typed keys throughout the actual viewer/source-art intake. All
158,720 archive-order keys,27,295 sorted mesh keys and20,625 texture keys agree
with the independent Python census. The catalog SHA256 is
`9a0f3d5a3c04f1d7de92e5e9adb20607424d324d823ea729023480ac244b82d2`.
The primary package also retains only its index instead of its whole payload.
Type-unspecified inspection references reject multiple matches within an
archive. Explicit overlay order remains unchanged.

`[MEASURED]` All704 linked creations pass the actual CPU saved-mesh/map loader,
including strict v8/v9 material-tail framing and their own source sampler maps.
The expansion loader's obsolete v8-only skipped-tail prerequisite was replaced
by the already strict complete typed-tail decoder. The four fresh main RTX
regressions remain PNG-byte-identical over2,359,296 pixels with executable SHA256
`817d55ecea231cbfe4c5c7ccb0ef42ea732a36b72bcafac8f2723828ee8363f2`.
Evidence is `viewer-catalog-comparison-v1.json`,
`complete-authored-intake-v1/report.json` and
`render-regression-v2/comparison.json` in the index census directory.
Some older standalone CLI/corpus tools still explicitly use compatibility APIs.

`[MEASURED]` A fresh compatibility correlation confirms identical payload
spans for all 158,720 records. Canonical interpretation corrects 4,519 type
values and 22,618 scope/low spellings in compact/common-header data. All
5,022 ANIM resources retain their type, scope/low identity and byte spans in
the same four packages; all 1,365 CreatureData resources remain present.
Earlier ANIM/body comparisons therefore retain their source inputs. See
`compatibility-correlation-v1.json` in the same census directory.

`[MEASURED]` The extended row-by-row correlation also confirms unchanged type,
scope/low identity and payload spans for every GMDL, RW4, RASTER, GMSH, SPUI,
Cell definition and CreatureData record.26 PROP records retain their payload/type
but require corrected scope spellings. Other compact data includes4,497 actual
locale tables of type02FAC0B6 previously labeled typezero. Full per-type counts
and changed package identities are retained in
`all-corpus-identity-correlation-v1.json`; old payload checks transfer their
input bytes, not incorrect historic native key labels.

`[UNKNOWN]` Active product selection, equal-priority registration order and
runtime alias-map contents still need closure before claiming retail overlay
precedence. The inspection renderer retains its explicit deterministic order.
The native index correction alone does not settle the selected retail package,
full resource-manager caching or gameplay.

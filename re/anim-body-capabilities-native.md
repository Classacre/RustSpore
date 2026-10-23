# Native body capability preparation

## Original operation and constructor boundary

[MEASURED] Canonical CreatureData loader `9CBFA0` copies raw per-part capability
IDs and number bytes into the zero-initialized block's fixed 32-slot array,
then calls `9FC2F0` before counting nonzero `foot` capabilities. Consequently
raw source capability masks cannot establish native root or fake-foot inputs.
The dictionary expansion, duplicate handling and slot ordering are required.

Retained native Evidence:

| Operation | Query / Evidence |
| --- | --- |
| Canonical loader `9CBFA0` | v1 query55, `ev_0789da66e7c2b2f79cb1e09c7665a7ab598558b8968857cb0bc80cdee48647ab` |
| Aggregate expansion `9FC210` | v1 query64, `ev_729252a6b054bd0958708da4f20301fa4c632ffdb4a4bacb1d3951fd34d4eb6a` |
| Duplicate/compaction `9FC2F0` | v1 query61, `ev_7d936e0ea9b7b9c0c19b86584ebfb35c1836f9db8310402be848b22a6b0c535e` |

Source executable SHA256 is
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Source-pinned decoding also establishes mask builder `9B2E20..9B2E5B` and
its eight-category lookup `9B2DA0..9B2E10`. Every direct transfer closes within
these four exact function spans; there are no native indirect calls, imports,
floating instructions or original initialization calls in this operation.

## Exact 32-slot semantics

[MEASURED] The native capability object contains a presence mask at +0, a
nonzero-number mask at +4, 32 IDs at +8, and 32 number bytes at +88 (hexadecimal
offsets). Its complete extent is 168 bytes.

`9FC210` finds the first zero ID, using that slot as the append position. It
then scans the changing array in order, stopping at zero or after 32 slots.
For each ID, it looks up a member vector through the unsigned ordered map
pointed to by global `166D060`. Every member is appended in source order and
inherits the current slot's number. Newly appended IDs participate in the
same scan; it does not repeatedly merge a mathematical set. A zero member can
stop further expansion before later slots are deduplicated and compacted.

`9FC2F0` keeps each ID's first occurrence and sets its number to the maximum
among duplicate occurrences. It clears duplicate IDs but leaves their number
bytes untouched. It then fills holes from the last occupied slot, copying
that ID and number forward and clearing only the old ID. The result therefore
retains meaningful native slot order and stale bytes in unused number slots.

`9B2E20` stops at the first zero or 32 entries and rebuilds both masks. The
known categories are root1, limb2, foot4, spin8, psft10, mout20, grsp40 and
nstr80, with mask values in hexadecimal. Unknown IDs contribute no mask bit.
Presence and nonzero-number masks must remain distinct, including `spin`
present with number zero after later finalizer relabeling.

The new `anim_body_capabilities` operation preserves all 32 IDs and number
bytes. `prepare_source_capabilities` copies a checked CreatureData part view
into the initially zeroed array and uses an explicit PartCapabilities context.
The old approximate `SourceRig` diagnostic remains separate.

[MEASURED] Native instruction `9FC230` loads an uninitialized local append
position when all 32 initial IDs are nonzero. The bounded API/host reject this
input before execution. They also reject an expansion that would write past
32 slots and an ambiguous duplicate aggregate key. A complete 32-slot result
created by bounded expansion is supported. These are interface bounds rather
than claims about native error handling.

## Fresh source population and prepared native fixture

[MEASURED] Fresh reads through the canonical flag-driven archive index cover
all 82 package indexes, 1,365 CreatureData creation resources and 46,418 source
rows, including flora. The source contains 81,052 capability records. Raw
capability counts are at most eight, so every actual initial array has a zero
before the unsafe full-array branch. No source row is discarded for its
geometry, class or capability contents.

All three installed PCTP resources independently decode to the same eight
aggregate relationships, with one member each. Their canonical table SHA256
is `489955a96aa6750c88739832681257918e3da3efb552e9ed24c10ef377812c23`.
Version and dictionary payload hashes remain separately retained under
`target/animation/body-capabilities-inputs-v1/`. This equivalence removes
dictionary overlay choice from this particular operation; it does not settle
retail archive precedence or unrelated dictionary fields.

The prepared cohort has 58,802 rows: all 46,418 source rows and 12,384 synthetic
contexts. Synthetic inputs cover twelve CWs, duplicates/maximum numbers,
tail compaction, retained number bytes, zero members, recursive expansion,
full expanded capacity, unsigned map keys, and distinct masks. The Python
reference uses an independent two-index compaction specification. A fresh
Rust source verifier separately rereads all source rows and dictionaries and
checks a 1,365-resource digest stream; it does not merely trust prepared inputs.

[MEASURED] The original-byte host is in
`target/animation/body-capabilities-native-v1/`. Its assembly SHA256 is
`40ce06078b813d8e6f4e0cd02556e089f6865d5cab5cf54b374bfe7e58f322aa`;
host PE SHA256 is
`187db8dea46679a0cb25dd0596828a604eb77bd23d66e8e620e7d23d5f39f253`.
The input SHA256 is
`47d1bd4aee83d7c11f4bb9a7b5b042dd04071e1b38bedc5ae0f73e2f17d7edb6`.

The host supplies an owned unsigned ordered tree with at most eight acyclic
nodes and 32 members per node. Entire tree/global storage becomes read-only
before each original call. Admission simulates the changing ID array and bounds
every append before native execution. Separate guarded input, capability,
validation and reply buffers, full input shadows, two scratch patterns, exact
ESP and unchanged CW/SW/MXCSR checks bound the operation. Unlisted code bytes
remain INT3 and original code pages become RX. No proprietary code/data is
stored outside ignored target evidence.

[MEASURED] Root and an independent peer reviewed the exact prepared host and
cohort before execution. Root reproduced every input and reaudited source,
linked code, tree/capacity bounds and guards. The hidden native run passed all
58,802 rows and 21 protocol checks, matching the independent Python projection
exactly. Every original call ran under both XMM scratch patterns; guards,
input shadows, ESP and CW/SW/MXCSR checks passed, and original/fixture sources
remained unchanged. No game, original module, initializer or GUI was used.

The exported Rust module passes all four focused tests and matches all 58,802
complete 168-byte native outputs exactly, with zero exclusions or problems.
Native, Python and Rust projection SHA256 is
`582aca8c6e189c9d8385749d68798bb03efdd9bb8da0446f7ac3c01032c6b337`.
Fresh canonical Rust reads independently match all 1,365 resource digests and
46,418 source capability rows. The completed three-way result is
`target/animation/body-capabilities-native-v1-run/comparison-v1.json`.

[UNKNOWN] This closes the constructor's capability preparation operation,
including exact slot order and masks. Generated hierarchy/root selection,
optional foot insertion, selector mapping, runtime poses and an assembled
animation milestone remain separate gates. The provisional raw-mask counts in
`body-hierarchy-source-census-v1.json` were explicitly rejected before native
eligibility use; the corrected v2 input census labels them raw pre-expansion.

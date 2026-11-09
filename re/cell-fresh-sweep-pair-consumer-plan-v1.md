# Fresh source sweep: next pair-emission boundary

`BBAD00` invokes `BBA7C0` for each axis and then consumes the sorted x-axis
records at manager +24. For each opening record it walks a stack-owned active
list, calls `BBA9C0` with excluded axis zero on the current and active slot,
and, when accepted and pair count is below capacity, stores the two actor
words in signed order. A closing record removes its slot by swapping in the
last active entry. `BBAD00..BBAE18` is 281 original bytes, SHA-256
`e51e8ec01ff952f605280bac28cabfc64a8af471cbc9cd389449c71281cdcf66`;
its three calls are stack probe at `BBAD05`, `BBA7C0` at `BBAD31` and
`BBA9C0` at `BBAD91`. It sets manager +20 to the sorted count, +2C to zero,
and +08 to zero before emitting fresh pairs. The source local active array is
0x1000 bytes; more than 1024 active entries is outside the bounded model.

A practical next independent test is a guarded *fresh-only whole consumer*
host containing the complete BBAD00, BBA7C0, BBA700, BBA9C0 and original
stack probe. Use the same reviewed source-manifest activation and explicit
installed pinned MSVCR90 qsort on a private non-switched desktop. Relocate
only the source's direct stack-probe/sort/filter calls, BBA7C0's probe call,
comparator pointer and qsort IAT; verify every remaining byte, all target
addresses, complete instruction/return extents and no unintended imports.
Prebuild a synthetic manager with old count zero, checked slot records, two
fresh endpoint arrays, guarded sorted arrays and a guarded pair buffer. No
native allocator, actor pool or live collection is needed. Ensure source
qsort's CRT/activation/loader identity and x87/MXCSR controls are checked just
as in the fresh-sort comparison.

Use distinct finite endpoint inputs and explicit valid buffer sizes. Compare
both sorted endpoint arrays and slot ranks, manager counts, full written pair
records and their source order to the existing bounded fresh append→sort→
`cell_source_sweep_pairs_v1::rebuild` model. Include same nonzero tags,
other-axis disjoint/overlap, signed actor ordering, capacity zero and partial
saturation, and three-or-more overlapping actors that exercise active-list
swap-last removal. The source does not expose the Rust diagnostic suppressed
count, so do not assert a native suppressed counter. Before running any host,
review exact stack/probe behavior, cdecl cleanup, qsort ownership, nonaliasing,
all guard ranges and fresh collection initialization. Preserve outputs and a
failure receipt; load/error cases may require private-desktop isolation.

A successful test would cover the original fresh sort plus pair pass for those
fixed controlled inputs. It would not validate incremental old/new merge,
tombstones, ties, source heap allocation, larger capacities, arbitrary FP
states, concurrent mutation, `BBB840` reverse-iterator lifecycle, source actor
resolution or contact/gameplay. The currently running original comparisons
validate only their individually pinned fragments and source hashes.

## Reviewed whole fresh-consumer comparison

The independently reviewed host was run with source PE SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
All four controlled rows matched the pinned Rust adapter, including full
manager counts, slot ranks, both sorted endpoint arrays and the bounded pair
buffer. Cases cover four actors with swap-last removal and signed handle
ordering, capacity two, equal nonzero tags with other-axis separation, and
capacity zero. Both children exited zero; x87 control was `027f`, MXCSR
control `1f80`, and both private desktops closed after child reaping. Receipt:
`target/cell-fresh-consumer-result-v1/receipt.json`, SHA-256
`371a2f0bfc665f7a56e0be7d58af824aa7950015f166dbad32cacb36af6a3912`.
The runner is `tools/cell_fresh_sweep_consumer_compare_v1.py` SHA-256
`2f639b5080a616b232317890b83ab7166c688133c722c55b9b5f1d8a572b67e7`;
its manifest SHA-256 is
`48b7538271a488c3e3bbcbf3d7417a33ebb267d99df36f9f653075bec18cbc7f`.
It verifies the complete source instructions and patched targets, pinned source,
CRT, activation inputs and model before and after execution. The Windows loader
and in-memory module integrity are admitted. Source calls run against guarded,
synthetic fresh collections, not live state. The exclusions above remain in
force; in particular this is not evidence for old/new merging, ties, source
allocation, persistent collection state, actors, callbacks or gameplay.

The later `FreshCollection::consume_fresh` API returns the sorted snapshot and
pair result to the installed-frame fixture. It calls `sorted_fresh` and then
`rebuild`; the latter independently sorts the same checked inputs again. The
v2 Rust-only adapter was compared byte-for-byte against the retained native
stdout for the same four rows, without rerunning native code. Its receipt is
`target/cell-fresh-consumer-v2-result-v1/receipt.json`, SHA-256
`ab2d2bba9eea49bb3d79279d917e406b56531459a1f2b4029966c1cadaf9c0c5`;
the comparison runner SHA-256 is
`c11cbf67bda82040cc03182bc158a278170efc5a7ef8644a5ceb4d6bb70bd5ea`.
The receipt pins the newer source and executable, corpus, old receipt and raw
native output before and after comparison. This extends evidence only to those
four fixed rows of the new API. Actor provenance, stable registry ownership and
source-registration assumptions remain explicit fixture admissions.

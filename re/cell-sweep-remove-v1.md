# BBAC30 checked removal trace

The pinned source PE (`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`)
contains complete 202-byte `BBAC30..BBACF9` SHA-256
`5a5ab6971ec9b2363a5d75937810093321839498a986bbb05087c62dd0fdd43c`.
It addresses a 16-byte slot by supplied index, processes links +0C, +08,
+0E, +0A in that order (max X, min X, max Y, min Y), selects old or pending
axis arrays by bit 15, masks pending indices and writes only the endpoint's
first u16 to FFFE. It then writes the previous manager +1C free head to slot
+00, FFFFFFFF to slot +04, decrements manager +10 and stores the removed slot
index as free head. Slot link words, count/high-water +18, endpoint slot words
and float values are untouched. The source makes no local validity or
lifetime checks; a caller must admit valid slot ownership, positive count,
coherent free-list state, valid nonaliasing arrays and bounded indices.

The build-only v2 host and runner isolate this function with disjoint guarded
synthetic storage. Four cases cover old-only, pending-only, mixed and a prior
one-slot free chain. Full arrays, slot records, selected manager words, stack
and FP controls are compared to a Python byte model. Source bytes, complete
instructions, no patches, pre/post pins, hidden private desktop and preserved
failure streams are checked. This host has not been executed as of
preparation; the v1 build was preserved but not run, and v2 fixes the prior
free slot's next and tag words to FFFFFFFF. Even a match only supports these
controlled state transitions and does not prove actual actor lifetime,
free-list ownership, allocation, concurrent mutation or live gameplay.

The independently reviewed host was subsequently executed. All four rows
matched the synthetic byte model; native exit was zero, x87/MXCSR controls were
`027f`/`1f80`, and the private desktop closed after reaping. Receipt:
`target/cell-sweep-remove-result-v1/receipt.json`, SHA-256
`b31f75f07d6f8fc12cd2075446d022e11c7004286d2e4acde3bb79568bb341bb`.
The exact bounded scope and admissions above still apply.

The separate checked Rust removal API was compared directly with the retained
native output for these four cases; all matched, without a native rerun. The
receipt `target/cell-sweep-remove-rust-result-v1/receipt.json` has SHA-256
`08f34f82d6a28c10aca44262e37ad9056d4b826e6d9086736ac20d229d25767d`;
runner SHA-256 is
`1e74cc3a7f46f1527d6a9c67b90f8ef4c1c8fb52955fe081b00193058fafc58e`.
The API checks bounded slot and head, positive live count, distinct in-range
links and mutation-free error behavior; these are added fixture safeguards.
The caller still admits target liveness, endpoint ownership, free-chain
coherence, disjointness and absence of concurrent mutation.

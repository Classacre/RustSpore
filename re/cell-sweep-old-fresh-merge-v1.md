# Controlled old/fresh endpoint merge trace

Source BBA7C0..BBA9BE is the 511-byte function pinned in the fresh-sort
comparison (`46084f94dc1d8fb34cedf4f8f79b8912300bbf25e7ef8c033ed8b0f6e178a908`).
It qsorts the fresh array, then merges into the old array in place. Its
0x8000-byte local FIFO buffers old records that would otherwise be overwritten.
The first u16 value FFFE marks a skipped record in either input. For old
records that already remain at their position, BBA87C..BBA888 advances without
rewriting the slot link. BBA916..BBA948 and BBA99F..BBA9AF write the new
rank for moved records, indexed by sign-extended slot word and selected axis.
It returns surviving count. This establishes neither how old values are
updated nor which actor registrations survive into the next frame.

The separate checked model admits finite normal/zero distinct keys, old live
entries already sorted with rank words matching their physical old positions,
valid unique live slot/endpoint links, signed-positive slot indices, at most
4096 total input entries and disjoint caller-owned snapshots. It preserves
nonselected rank words and omits tombstones. These are fixture restrictions,
not native validation or evidence of native checks. Its complete sort is a
mathematical model under these admissions; original in-place FIFO, qsort,
source allocation, endpoint update producer, ties, FP exceptions and live
lifetimes have not been verified by this model.

A bounded independent comparison should reuse the exact pinned complete source
BBA7C0/probe/comparator slices and reviewed direct MSVCR90 qsort activation,
with guarded disjoint old, fresh and slot buffers. At minimum include an
interleaving that forces FIFO preservation, old and fresh tombstones, all-old,
all-new and empty, and both selected axes with untouched other-axis words.
Record full initial arrays, output buffer including tail, slot words, returned
count, stack and FP controls. Explicitly pin corpus, host, model and original
source; compare only initialized defined outputs. Non-switched private desktop,
loader/in-memory admissions and raw failure receipts apply. Do not execute a
new host before independent review.

## Reviewed original-instruction comparison

The reviewed host executed the complete `BBA7C0`, original stack probe and
`BBA700` comparator with the pinned installed MSVCR90 qsort. All five fixed
synthetic cases matched the checked model: unchanged old entries, four old
plus two fresh endpoints forcing scratch preservation, old and fresh
`tombstone` markers on axis 1, all-fresh entries, and empty inputs. Each row
compared all four slot records, the complete eight-entry in-place old buffer
(including its tail) and the returned live count. Stack and x87/MXCSR controls
passed; both private desktops closed after the children were reaped. Receipt:
`target/cell-old-fresh-merge-result-v1/receipt.json`, SHA-256
`5611bf037260f201472790cd79c0d06cfa1d2786438710fc013b6a93ee054bce`.
Runner SHA-256
`4c71084a00709d57cff31f5a05e08f7173f82136c3c29f2df2a8457c6f882b39`;
manifest SHA-256
`65afa750d1ef6f397e3e3b3be1041d08805dc078f7531793174226680f7e9bc2`.
Source and linked instructions, relocated targets, original EXE, CRT, manifest,
model, corpus and wrapper were hash checked. Windows loader and in-memory module
integrity were admitted. These five cases do not validate arbitrary input,
producer updates, persistent registration, source allocation, concurrency,
actor identity or live gameplay.

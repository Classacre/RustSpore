# Source-bound start-reference fixture plan

This is a test plan, not an executed Rust fixture or native reference run. It
binds a caller-selected installed PatchData example to the symbolic draft
`cell-start-reference-fixup-draft-v1.rs`; it does not establish the retail
resource-manager winner, a live descriptor pointer, or player creation.

## Rechecked candidate

On this machine, a read-only SHA-256 of `Data/PatchData.package` is
`cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff`.
A fresh flag-driven `native_archive_index_probe.decode` read of that package
found one record for each key below. The order is native `(instance, type,
scope)`, not the historical wide-index group spelling. Both entries report
compression `FFFF` and committed bit 1.

| Selected key | Ordinal | Stored offset / size | Decoded size | Decoded SHA-256 |
| --- | ---: | --- | ---: | --- |
| `A426730B:2A3CE5B7:00000000` | 12117 | 10330892 / 203 | 276 | `e8c4b80348fd8f3a1821533c87110315ba584f66f081d46305ff61ab6cbf8c72` |
| `CBCD1287:DFAD9F51:00000000` | 5236 | 12183763 / 119 | 796 | `f12d98c0f59af6d4d58174016f1e04d709647480310103341193df9b0fa76542` |

The decoded globals bytes `+34..+37` are `87 12 CD CB`, or little-endian
`CBCD1287`. The source instruction and descriptor trace for those observations
is in `cell-player-start-definition-static-v1.md`. The package and payload
hashes, ordinals and offsets are snapshot-specific; fail closed if they change.
The extra stored metadata word is not part of the native resource key.

## Proposed independent comparison

1. A fresh harness should use the flag-driven index reader, check header/index
   bounds and uniqueness for both full keys, then independently decode the
   selected payloads with their declared `FFFF` compression and require exact
   decoded lengths and the two digests above. Record the full package SHA-256
   before and after extraction, index/entry data, tool/source hashes and the
   two decoded hashes. Require the package to remain unchanged throughout the
   read; before/after hashes alone cannot rule out a concurrent transient
   mutation. Prefer an immutable snapshot or appropriate read-sharing policy.
   Do not use legacy `dbpf_extract.parse_index` group labels as canonical keys.
2. Independently derive the expected raw field and scope-zero CELL key from
   the bytes and static descriptor trace: `CBCD1287`, `DFAD9F51`, scope zero.
   Supply the two verified byte slices and exact keys to `prepare_start`, with
   `Resolution::Supplied` and an explicitly symbolic descriptor token whose
   type is `DFAD9F51`. The token is a fixture identity, not the source pointer
   `016AE490`. The supplied package label is descriptive; package and payload
   authentication must occur in the harness, because the draft validates only
   keys and lengths.
3. Require a successful result, the expected key and provenance labels, and
   `table.get(handle)` equal to `(CBCD1287, supplied descriptor)` with exactly
   one table entry. Repeat the same pair to check interning reuse, and use a
   second symbolic descriptor identity to check that an equal raw ID does not
   imply a shared handle. A different table must not resolve the first table's
   handle. Compare normalized facts, not synthetic table IDs or pointer values.
4. Use separate copies of the verified inputs for negative tests: a changed
   same-length byte must fail the **harness hash gate**; wrong scope/type/ID,
   264-byte old globals, short Cell bytes, zero ID and `FFFFFFFF` must hit
   the appropriate bounded fixture errors without mutating the table. Exercise
   unavailable/ambiguous/fallback states as explicitly supplied fixture
   conditions. Do not interpret those safe errors as native failure behavior.
5. Pin the exact current draft, adapter, canonical index/decompression tools,
   fixtures and reference calculation. Retain input/output manifests and
   hashes; run focused tests and compare the actual Rust output after independent
   review. A separately reviewed native host with original instructions and
   real or explicitly stubbed dependencies would be a different, stronger gate.

`E4AAD0` performs the nonzero field replacement before the later resource
acquisition; the draft intentionally validates the complete supplied fixture
first and stores a symbolic result instead of writing a pointer into the loaded
buffer. In particular `FFFFFFFF` is interned by the native nonzero fixup before
the ordinary `E82200` lookup rejects it and fallback may run. The proposed
atomic sentinel rejection is a safety policy, not reproduced native order.

The selected PatchData input is an **inspection admission**. The active product
mounts, priority/tie registration, provider and cache state, type aliases,
fallback and runtime descriptor lifetime remain open. A matching source-bound
fixture cannot establish which resources the original manager or player used.

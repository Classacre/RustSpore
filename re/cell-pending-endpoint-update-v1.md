# Pending endpoint update source trace

`BBBD10` is a direct jump to `BBB880`. Contrary to an initial incomplete
inspection, `BBB880` is the same two-axis 16-byte-slot collection: its sorted
branch iterates twice and uses manager +24/+28 for old endpoint arrays and
+30/+34 for pending arrays. Arguments are manager, slot index, pointer to
minimum XY, pointer to maximum XY. Source `BBB89B` tests bit 15 of the slot's
+0C link. On the pending branch `BBB8A1..BBB8FC`, it masks bit 15 from the four
slot links, then performs x87 binary32 loads/stores in order min X, max X,
min Y, max Y to the selected pending arrays. It does not itself establish that
all four links are valid pending indices, vectors are finite, nonaliasing,
intervals valid, or that the slot belongs to a live actor. Those must be
explicit fixture admissions. The alternate sorted branch updates endpoint
values and calls crossing helpers `BBB020`, `BBB130`, `BBB240`, `BBB320`;
their reorder and pair mutations remain unmodeled.

The prepared original-instruction host and runner, not executed, are
`target/cell-pending-endpoint-update-prep-v2` and
`tools/cell_pending_endpoint_update_compare_v1.py`. Four guarded cases use
various distinct pending indices and finite normal/zero bit patterns; some
vectors are not valid geometric bounds because the source branch simply
copies. The full 381-byte function is copied with four alternate-branch helper
calls redirected to a nonreturning failure trap. The runner verifies the full
instruction extent, source bytes and every relocation, checks x87/MXCSR and
stack, records whole synthetic slot/endpoint arrays and uses a private
non-switched desktop. Model expectation is generated from the pinned corpus;
this is a raw-store comparison only. The first build-only prep failed assembly
and was preserved as `target/cell-pending-endpoint-update-prep-v1/build-failure.json`;
neither version has been executed as of preparation. No source allocation,
source actor identity, sorted update or live gameplay is admitted.

The independently reviewed v2 host was subsequently run. All four raw-store
rows matched the Python byte model; x87/MXCSR controls were `027f`/`1f80`,
the native child exited zero and the private desktop was closed after reaping.
The receipt is `target/cell-pending-endpoint-update-result-v1/receipt.json`,
SHA-256 `c155de4b3a6b2698067d3b05c90b5cdd275ef3635042f8ec003354701cead22d`.
It does not validate a Rust model, the sorted branch, crossing helpers, actor
ownership, valid intervals or source collection lifetime.

The separate checked Rust pending-update API was subsequently compared against
the preserved native stdout for all four cases, with no native rerun. All
matched. Receipt `target/cell-pending-endpoint-update-rust-result-v1/receipt.json`
has SHA-256 `8aea1e6cce42f8752131bb184c5f0b182bd9912602d059fe11f5842b68765c95`;
runner SHA-256 is
`2f3b61f72840cf789b52a38a20cc029be515bffba1a6a07f7a79e54b70f27be7`.
The API requires all four pending links in range and distinct within an axis,
normal/zero values and an explicit x87 027f admission, with immutable slot and
safe disjoint mutable arrays. These are additional fixture checks. Endpoint
flag and slot words are opaque, so the caller admits entry ownership. It does
not validate actor identity, interval geometry, sorted updates or lifecycle.

An opt-in two-frame composition now applies the checked pending update to
owned copies of newly appended synthetic endpoint arrays before reconstructing
and validating their bounds and calling the existing old/fresh diagnostic.
It rejects duplicate updates and invalid intervals without mutating either
input snapshot. This composition has focused model tests; the four original
instruction matches above apply to the pending update slice, not independently
to the complete two-frame composition. Sorted updates, actor identity and
persistent registration remain excluded.

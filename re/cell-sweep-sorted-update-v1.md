# Controlled BBB880 sorted update

The sorted branch performs four conditional operations in each axis, in this
order: lower minimum then `BBB020` (left), raise maximum then `BBB130`
(right), raise minimum then `BBB240` (right), lower maximum then `BBB320`
(left). Each helper receives pair-effect flag zero. `cell_sweep_sorted_update_v1`
composes the separately checked crossing model in this order on immutable
snapshots. It requires finite normal/zero values, strictly valid input bounds,
coherent slot ranks and unique endpoint values at each intermediate step;
these are additional checks, not source checks. Ties, tombstones, aliasing,
nonzero pair effects, live actor identity and persistent ownership are excluded.

A build-only comparison in `target/cell-sorted-update-prep-v1` contains the
complete original BBB880 and four original helper bodies. Its four source
helper calls target the copied helpers; their six conditional pair-related
calls target a nonreturning failure trap. Four synthetic cases cover crossing
while shrinking, crossing while expanding, mixed direction, and equality of
signed zeros with no store. The pinned runner
`tools/cell_sorted_update_compare_v1.py` checks all source instructions and
relocations, guarded synthetic buffers, stack, floating controls and compares
full result records to the directly invoked Rust adapter. As of preparation it
has not been executed; no claim of full native behavior or live gameplay is
made. The previous independent helper receipts remain bound to their original
source hashes and scopes.

The independently reviewed host was subsequently run: all four fixed cases
matched the Rust adapter, both children exited zero, x87/MXCSR controls were
`027f`/`1f80`, and the private desktop closed after reaping. Receipt
`target/cell-sorted-update-result-v1/receipt.json` SHA-256
`58df0d9d0af849b1e6ff2a9f20bdd30b3d8377d39cb6772995ecf47e872b142f`.
Runner SHA-256
`3d132f9d11d627fa557cb965f034cff4c43d35634f1cf5475c48984ed038c340`.
The compared scope is those four synthetic rows, under the exclusions above;
source actor ownership and full collection lifecycle remain unverified.

The opt-in `cell_sweep_multiframe_diagnostic_v1` composes a retained first
snapshot, a selected sorted update and separately supplied pending updates,
then merges old and new endpoints on each axis and independently rebuilds
pairs from the combined bounds. It returns both the original and updated old
snapshots, rejects duplicate synthetic actor words/updates, and retains
mutation-free errors. Its focused tests exercise both update paths and the
shared merge. The complete composition has not been independently compared
with source; the 4-row sorted update and earlier pending/merge receipts cover
only their respective pinned component slices. No source actor ownership,
lifecycle, ties, tombstones or pair-effect behavior is inferred.

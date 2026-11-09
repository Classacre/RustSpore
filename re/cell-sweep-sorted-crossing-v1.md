# Pair-disabled sorted endpoint crossing trace

In the sorted branch of BBB880, the four helper calls pass zero as their
pair-effect argument. `BBB020..BBB127` moves a minimum endpoint left;
`BBB130..BBB231` moves a maximum right; `BBB240..BBB310` moves a minimum
right; and `BBB320..BBB3EE` moves a maximum left. Each compares neighboring
binary32 values, swaps eight-byte endpoint records while out of order, adjusts
the selected slot's rank and the crossed live slot's corresponding rank, and
preserves other axes. Tombstones move but receive no crossed rank update.
Conditional pair insertion/removal logic exists when the flag is nonzero;
that branch and tombstones are excluded from the initial model. Source does
not locally enforce unique finite values, coherent ranks, actor identity or
nonaliasing; the checked fixture does.

A build-only four-helper comparison is prepared in
`target/cell-sorted-crossing-prep-v1`, with runner
`tools/cell_sorted_crossing_compare_v1.py`. The complete helper instructions
are retained with six conditional pair-related calls redirected to a
nonreturning failure trap. Four fixed two-slot cases cross two live records in
each direction, axes 0 and 1. Full slots, sorted endpoint array, unchanged pair
buffer and count, stack and FP controls are checked against a Python model.
The source and linked bytes, instruction boundaries, relocation targets,
fixture and EXE are pinned; a non-switched private desktop and preserved raw
failure output are required. As of preparation the host is not executed. The
Rust crossing model remains statically inferred and has focused unit tests;
ties, nonfinite values, tombstones, nonzero pair effects, full BBB880 update,
source ownership and live lifecycle are not established.

The reviewed host was subsequently run: all four fixed cases matched the
Python byte model; native exit was zero, x87/MXCSR controls `027f`/`1f80`, and
the private desktop closed after child reaping. Receipt
`target/cell-sorted-crossing-result-v1/receipt.json` SHA-256
`aac7ff9d17613a647591507eb667f17441c0a6329666f543412460a6922b4f5b`.
This result is limited to the pair-disabled, finite unique, nontombstone
fixtures above; a Rust comparison and full update are separate boundaries.

The separate Rust crossing adapter was compared to the retained original
native stdout, without rerunning the host. All four fixed rows matched. Receipt
`target/cell-sorted-crossing-rust-result-v1/receipt.json` SHA-256
`fee54756e7228457fdf5c74e8fe3e0baf3461bea228294d83ecf347d30a6a970`;
runner SHA-256
`c38e9c5ba5cefdc220756d72065028dc309f9b5d1319be890a0d1899f987b1a0`.
This directly tests the checked Rust helper composition only for these four
rows; all pair, tombstone, source actor and full update limits above persist.

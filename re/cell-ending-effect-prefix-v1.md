# E6E920 completion-effect admission prefix

Original `E6E920..E6EA15` first constructs and resolves a reference. If the
resolved global +E0 is zero and the supplied flag byte is zero, it reaches the
E534C0 effect path. Otherwise it resolves a player through root +411C and the
root +1C pool. Null or a nonzero player byte at +112, +113 or +188 goes to
reference cleanup; an all-zero valid player reaches E6EA10, before later
mutation, effect and record operations.

The independently reviewed source-instruction host
`target/cell-e6e920-prefix-prep-v3` traps both effect and valid-player entries
and returns through the original cleanup. The scripted resolver, pool lookup,
root, handle, object and reference lifetime are synthetic. All 20 combinations
of +E0 0/1, flag 0/1, and null/all-zero/one blocked player byte matched the
inert-stub counts. Native receipt
`target/cell-e6e920-prefix-result-v3/receipt.json` SHA-256
`436e7053fa2ee92b137d680ca0fd5266283ef8616613fffad62b777524a136ef`;
raw output SHA-256
`4f771efb2441a47327d0c03e0c007f8032343f69c3fbe8ea319f86efe9e9df8a`.
Runner SHA-256 `5cbf31ab9c976f8a7e34cb644fa4ef41772ba65817effb47521027c72d39c54e`.
FP controls and private-desktop closure passed. This does not establish live
pool identity, callback behavior, successful allocation, effect execution or
record mutation. Canaries are not guard pages; v1 build failure and earlier
v2 host are preserved and were not executed.

Checked Rust composition requires a pending prior gate, stable/disjoint
synthetic snapshots and an explicit lookup result when the source would read
it. It distinguishes a missing lookup from an admitted null result and stops
at the effect or eligible-player boundary. Two focused tests passed. The Rust
comparator matched all 20 retained native rows and FP controls without native
rerun; `target/cell-e6e920-prefix-rust-result-v1/receipt.json` SHA-256
`5107260178af5568d5910a477b2c1330cfc307925aac718da56aac9efa0619c8`.

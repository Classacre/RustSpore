# E780D0 player-word prefix

The bounded projection models the complete B71CC0 pool word check and E780D0's
first conditional write. An invalid actor returns without changing the supplied
player word. A valid actor clears the word only when the two words are equal,
then stops before the fresh B71D00 lookup and all manager or actor teardown.

The source uses signed `JG` on the low-16 index and the pool +10 bound. A read
when index equals that bound is possible; the checked model requires a separate
equal-bound admission and an actually supplied readable slot. Stable, disjoint,
non-null slot storage is a caller admission. Negative bounds and inaccessible
slots are rejected as additional safety policies. The input snapshots are not
native actors, ownership, or a live pool.

The reviewed source-instruction host matched six controlled cases, including an
explicitly allocated equal-bound slot. Its native receipt
`target/cell-e780-prefix-result-v2/receipt.json` has SHA-256
`7501a8bd821254a09f0ca0fa5331da1be88895da57c50baafeb49871bb4ae408`.
The checked Rust projection matched those six retained rows in a separate
comparison, receipt `target/cell-e780-prefix-rust-result-v1/receipt.json`
SHA-256 `ca08e20b0c3a9c515bb6dcda8b6221ea945313c8166f5b3de5c1c3ad1518e6b0`.
The native continuation was cut before B71D00 and no live teardown ran.

## Next fresh lookup and strict root branch

After the first write, E7810B..E78129 invokes B71D00 with the same actor word,
reads the resulting actor's +35C and +360 words, and passes them as container
and node to E869F0. B71D00 itself computes `base + (actor_word & 0xffff) *
stride` and performs no generation or bounds check; a fresh stable, readable
slot and its identity must be admitted separately from the earlier B71CC0
check. The next source instruction at E7812A reads actor +364.

On E869F0's strict branch where container +08 equals the supplied non-null
node, the source clears container +08, takes the manager at container +04,
reads its free head at +08, decrements its count at +00, writes the old free
head to node +00, and sets manager +08 to that node. Other branches traverse
or modify other links and remain outside this boundary. Source operations do
not validate pointer lifetime, ownership or aliasing here; a bounded fixture
must admit disjoint stable structures and a valid manager count.

The independently reviewed original-instruction host under
`target/cell-e780-root-unlink-prep-v1` ran this caller slice, complete B71D00
and the strict root branch for two synthetic free-head cases; both matched.
Its receipt `target/cell-e780-root-unlink-result-v1/receipt.json` has SHA-256
`8049f31c92c59a8aaf472e6a80c60d4a861d897406b2252623fe3d156cde6d27`.
The nonroot branch traps and the continuation stops before the +364 read. It
does not establish real actor ownership, lifecycle, or a live removal.
The checked Rust projection matched both retained native rows; its receipt
`target/cell-e780-root-unlink-rust-result-v1/receipt.json` has SHA-256
`24ea09b31208ca7bdbbe2aaea8b07f2f8634370d88df495d8a1ff0f9e225d746`.
It requires a matching fresh slot word, nonzero owned synthetic identities,
disjoint stable structures, a strict root match and a positive manager count.
The count and identity checks add safety beyond the original instructions.

## Following manager removal boundary

At E7812A the source reads the actor's +364 word; FFFFFFFF skips the following
call. Other values are passed, with the manager at global root +4104, through
BBB800's tail jump to BBAC30. That complete removal operation has a separate
bounded comparison in [cell-sweep-remove-v1.md](cell-sweep-remove-v1.md).
Composing it after the root unlink requires fresh actor +364 and manager
snapshots, valid slot ownership and a disjoint stable pool and endpoint arrays;
the earlier root unlink alone establishes none of these. Later +248/+370
branches and actor release are separate pending operations.

The reviewed handoff host under `target/cell-e780-manager-handoff-prep-v1`
matched the four previous BBAC30 input states and retained output rows, plus a
sentinel case with a null manager pointer to check that the source skips manager
access. It stops before +248. Native receipt
`target/cell-e780-manager-handoff-result-v1/receipt.json` SHA-256 is
`ff7e844ed05c7adeaf8d1f6f09ce71019b4989c2222c8abb74ffbbd56d7116f4`.
The checked Rust join matched all five retained rows, with receipt
`target/cell-e780-manager-handoff-rust-result-v1/receipt.json` SHA-256
`44d1930d2eed44a51cbf6113b6d26f43551a8274d363d6d0363aac91c3942023`.
It needs no manager for the sentinel; other cases require fresh stable slot and
manager ownership and apply the checked removal's additional validity rules.

## Later component boundaries

At E7814E, nonzero actor +248 calls E66090; that function freshly resolves the
actor and component through separate unchecked pools, may invoke E4F4E0, then
clears actor +248 before calling E65B50 and releasing the component. A nonzero
E780D0 argument byte also permits a later E67900 call. These calls and their
intervening mutations require separate admissions.

At E78178, nonzero actor +370 calls E59080. It first validates the supplied
record handle in the global +8C pool. If valid, it validates record +0C in the
global +1C actor pool; a valid second lookup clears that actor's +370 and +36C.
The first pool record is released even if the second actor lookup fails. An
invalid first handle returns without release. The final E78242 path separately
calls B71D50 on the original actor handle without a nearby validity recheck.
These are source-order observations, not evidence of live actor ownership.

After independent review, the guarded original-function comparison for E59080
and the two pool helpers matched all three controlled validation/release cases.
Both equal-bound synthetic slots were explicitly allocated and readable, and
the two pools disjoint and stable. Native receipt
`target/cell-e590-component-release-result-v2/receipt.json` has SHA-256
`41eb94931f99796892a56479c9640ea60011862254bc706b44996643e747b150`.
It does not establish actual pool ownership or live component lifetime.
The checked Rust projection matched all three retained rows, receipt
`target/cell-e590-component-release-rust-result-v1/receipt.json` SHA-256
`b27ed901cfc12aa231ce6eec1a3d82beff430f1767943e0f85d09ad6a7782761`.
It requires stable disjoint synthetic pools and explicit equal-bound readable
slots; invalid first record needs no second pool, while a valid record requires
the second pool snapshot before projecting any release.

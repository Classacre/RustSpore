# Cell cursor picking: spatial tree and source inputs

Static SDK disassembly and PE data reads of installed base `SporeApp.exe`,
SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
No original instructions or process were executed. This complements
`cell-player-target-input-static-v1.md`; it is not live selection acceptance.

## Leaf population and maintained tree

`E74F55..E74F85` stores the chosen tree owner at actor `+35C`, calls
`E6D7F0` for center and scalar, and passes tree, center, scalar and actor
handle to `E86960`; the returned leaf pointer is stored at actor `+360`.
`E868F0` allocates a node and writes center `+00/+04/+08`, scalar `+0C`,
zero `+14`, and supplied handle `+18`. `E86960` sets parent `+10` to zero
and pushes the new leaf onto the owner's pending linked list at `+0C`.
`E7C3B3..E7C3D9` calls the same geometry producer and `E870B0` with owner
and leaf after the observed actor-position update. `E870B0` overwrites leaf
center and scalar; if a parent exists it tests containment with an x87
distance and can remove/requeue the leaf. Its mutation sequence, the pool,
deletion, update order, frame policy and capacity behavior need faithful
modeling for a generated tree. The source does not sort by handle or distance.

`E86FE0` drains pending entries from head: it removes head via node `+14`,
zeros that link, and when root `owner+08` is null installs that leaf as root.
With an existing root leaf it makes a parent by `E86DB0(owner,new,old)`.
For an internal root, `E86E30` descends by squared center distance from the
new node to the two children, choosing `+14` only when child `+18`'s squared
distance is strictly greater; ties take `+18`. Radius is not included in
this branch. It replaces the chosen leaf with `E86DB0(owner,new,chosen)`,
sets the new parent's parent pointer, then calls `E86EF0` starting at the
old parent. Each new parent stores new child at `+14`, old child at `+18`,
and computes its enclosing center/radius with `E86C20`; parent scalar is
then multiplied by source binary32 `1.100000023841858` (`1486BD0`). The
pool allocations and all source ordering affect the resulting tree.

`E86C20` computes a center difference and length with x87 intermediates,
commits length to binary32, and has two strict containment branches. If
second radius exceeds first radius plus length, copy second center/radius;
if first exceeds second radius plus length, copy first. Otherwise the radius
is half `(length + first radius + second radius)` in source SSE order. For
length below `1.52587890625e-05` (`1486BD8`), it averages the centers;
otherwise it forms the weighted center using reciprocal length, differences,
radius difference and half in the exact source instruction order. Ties and
floating control therefore matter. `E86EF0` checks whether either child
extends beyond an ancestor sphere and stops when both fit; otherwise it
recomputes via `E86C20`, then ascends. Unlike initial parent construction,
this recompute call does not visibly apply the `1.1` multiplier. x87 and
MXCSR policy, nonfinite arithmetic, allocation failures and alias domains
remain separate gates, not inferred from a mathematical sphere formula.

## Ray query and ordering

`E87270` first flushes pending entries by `E86FE0` if nonempty, then invokes
`E87170` at root. With count initially zero and capacity 512, recursion stops
when count equals capacity. For each visited node, `E87183..E87201` computes
binary32 squared distance from ray origin to node center. If it is within
the squared node scalar, it continues directly; otherwise `5A9C40` receives
origin, direction, node center and radius and must succeed with returned
parameter at most the supplied limit. For Cell `E6C7E0` this limit is the
source word `7F7FFFFF` (maximum finite binary32), not infinity. `5A9C40`
uses source-ordered SSE discriminant operations and x87 square root/root
comparison, with a conditional parameter write. Internal nodes recurse on
`+14` and then `+18`; a node with zero `+14` appends its `+18` handle.
No result distance is sorted. Since the caller takes the first accepted
candidate, a flat unordered scan or nearest-hit rule changes behavior.

## Selection filters and admission

`E6C7E0` resolves each returned handle with `B71D00`, excludes the actor
whose first word matches global `+411C`, and skips actor byte `+112` nonzero.
It accepts the first remaining actor when `E620D0` or, if that is false,
`E6B560` succeeds. `E620D0` resolves actor definition `+108`, tries the
definition `+A4` through a global named-data/localization lookup and returns
true on that route; otherwise it checks a nonempty definition string at
`+04`, then calls `E845D0` on actor triple `+FC`. This is a name/data
availability route and requires the actual providers, not just geometry.
`E6B560` resolves the player handle from global `+411C`, obtains the player
cache via `E67A50`, and calls `E57260(player,candidate)`. The latter clamps
the difference between candidate and player `+358` values plus two to
`0..4`; `FFFFFFFF` values use the `E4ECC0` progression fallback. Only
results 0 or 1 proceed. It resolves candidate definition `+108`, reads
definition `+B4`, and accepts when player cache byte `+1251` is nonzero and
that word is 3, or byte `+1250` is nonzero and the word is 2.

`cell_pick_filter_v1` exposes that second predicate with explicitly supplied
actor words, player cache bytes, candidate definition and optional progression.
It preserves the zero/self early exits, reached provider ordering, sentinel
fallbacks and wrapping signed size relation. Missing reached data and progress
outside the admitted `0..1000` range return checked errors; these are not
native fault behavior. This model does not supply an `E620D0` result or prove
selection in an application. Three focused analytic tests pass.

An initial independently reviewed filter host stopped with access violation
after 37,368 complete replies. Its source table omitted the threshold word at
`1484914` (source value 1000), so the first missing case's progress 999 could
run the original scan beyond the supplied table. The failed full run remains
invalid. An offline comparison of the 37,368 complete-reply prefix matched
all predicate and provider-call results; that is a partial bounded comparison,
not acceptance of the failed run.

The separately reviewed corrected host, SHA-256
`680580427f1dcc73f10113410144b64b7cf2df30e723075e308c5c3563ba12dd`,
includes the final source word. It executed the three original function ranges
with controlled synthetic player, cache, definition and temporary-provider
stubs, and the pinned 596-byte source progression table. The Rust comparator,
SHA-256 `dc17ceb20cb00170ee42974efdfe585735939da622a4e546499840724ff93a7e`,
matched all **54,000** predicate and reached provider-call replies. Input
SHA-256 `ddf9a1ea168e5b0b1b51dc8d8f4ada238df89cc313d199f7a9977760f40898a4`;
output SHA-256
`12f2665f1efce35f30fa6750776bdb0b0ff584e876f86feb1b63144676affa72`.
Cases exercise early exits, fallback thresholds, signed wraparound, raw cache
flag bytes and definition type words. The fixed global/table mappings,
synthetic providers and admitted progress range are part of the comparison;
no actual player, cache, definition resolution, name lookup or input was run.

`cell_actor_geometry_projection_v1` models the bounded `E6D7F0` geometry
producer with explicit actor/cache values. `cell_broadphase_gate_v1` instead
models an `E7F327` actor-pair collision gate and is not this query. The
smallest faithful selection model may accept an already-admitted ordered
tree snapshot, source ray and per-candidate resolved actor/filter outcomes;
that bounds traversal without claiming construction or provider fidelity.
Building it from supplied actors additionally requires the pending insertion,
reparenting, updates/removals, exact sphere arithmetic and float policies.
Application use further needs admitted active camera/viewport, pool/tree,
definition and name providers, player cache/progression, actual event routing
and source actor integration. None are supplied by prototype `CellWorld`.

`cell_pick_tree_v1` is a checked finite-domain supplied-snapshot model of
the traversal and ray/sphere branch, with explicit x87 context and scoped
MXCSR policy. It consumes the geometry projection's `Output` directly and
returns ordered handles; it does not construct a tree or perform filtering.
Three analytic tests pass for ordering, capacity, hit/miss/tangent and limit
cases, invalid reached graphs and overflow. They are model checks.

The independently reviewed no-CRT x86 host v6 copied the original unchanged
`5A9C40..5A9D3B` and `E87170..E87261` ranges to two regions at the same
relocation delta. Static audit confirmed internal jumps, only the ray-helper
and self-recursive direct calls, and no absolute memory accesses or indirect
transfers. Host `1f6f77c71b7fba654de418bec6a57001e72092df18ba41d0d75161a3ae4ef4b2`
ran 1,452 bounded synthetic requests across twelve explicit x87 policies
with MXCSR `1F80`; output SHA-256
`9f74266cc7fd3c8308c21567b5438058a61577b8d2c0a2401633dc3f72d4cb46`.
The independent Rust comparator matched every ordered handle and count, and
checked x87 and MXCSR control metadata; FP status flags are not modeled. The
input SHA-256 is `c76775ab13726c5bacc45e6b686751f613378e776a3b5ea31e316db5aa1d14e4`.
Cases cover bounded synthetic trees, branch order/capacity, tangent neighbors,
inside/outside rays, limit, signed zero, subnormal and seeded finite inputs.
This is source-instruction conformance for that admitted corpus, not complete
floating-domain equivalence, tree construction or live gameplay.

`cell_pick_tree_build_v1` separately models a bounded owned tree using the
traced pending insertion, sphere merge/refit, leaf update/requeue and removal
paths, with explicit floating contexts and checked atomic errors. It exports a
snapshot for the traversal model. Four focused analytic tests pass.
Its monotonic Rust IDs and capacity/error behavior are explicit safety choices,
not native allocator identity or native fault behavior. A static source audit
of the ten candidate functions and their four absolute constant reads is under
`target/source-cell/pick-build-audit-v1/`. An independently reviewed no-CRT
x86 host copied the ten audited original function ranges with one common
relocation delta, and supplied the four pinned constant words at their
original addresses. Host SHA-256
`d6bf1fe4d63320690e70664830eb781c68b0ab3727489dcebffa3b93976a06ba`
executed **1,944 bounded synthetic operation histories**, across twelve
explicit x87 policies with MXCSR `1F80`. The Rust comparator matched every
canonical preorder node's center/radius bits, kind and leaf handle, together
with pending-list order and control metadata; FP status flags are not modeled.
Input SHA-256 `e755ba019f151ba6bf83a9ee244d6170abf67a0b1281f3205e3a9c5d08867930`;
native output SHA-256
`79ec6fac9612649d3ca1fee0e9cf2aee1104c708e7feabec8ed23ba73d0f1b18`.
The offline comparator executable SHA-256 is
`3f95a31da24af23d1d69057df316614c0dc55b22454a5c3c1b7db6f92d1ff6ff`;
its Rust source SHA-256 is
`ed3bf375c8e1d2a7808696b280658e1ee6c03a6ab6e8500a62fefea0b8083c59`.
The corpus covers empty and queued trees, ordered insertion, ties,
containment, updates, removals, reinsertion and 1,800 seeded histories. This
is bounded source-instruction agreement, not exhaustive arithmetic or proof
of original runtime inputs, pool identity, concurrency or gameplay behavior.

Two prior host attempts remain invalid, not native conformance evidence: v4
exited 14 with no replies after requesting unaligned reservations; a separate
source-free preflight confirmed Windows rounded those requested addresses.
The aligned fixed-address v5 exited 21 with no replies; an independent x86
preflight observed its first region reserved as private and allocation failed
with error 487. V6 instead reserves a fresh region, with identical relocation
for both original sequences. No game initializer, game process, window,
network or live actor was used.

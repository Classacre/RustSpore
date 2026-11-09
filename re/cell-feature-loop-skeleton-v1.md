# E6ADE0 feature-loop skeleton preparation

Actor +FC is populated from E656C0 by E74A50 and can subsequently be replaced
by the +5190 saved triple. Actor +248 is conditionally allocated through
E78C30/E6D950/B71C50 and can later be cleared; neither the installed candidate
triple nor a synthetic handle proves live component/cache ownership.

On the both-feature route, E6ADE0 calls E57260 then obtains two E67A50 cache
results using the respective triples. It loops a signed count at cache +00 and
72-byte rows starting at cache +04; the row class at +04 is skipped when equal
to one, without renumbering. The existing feature snapshot filters classes but
its rows and first-word association are controlled inputs, not native caches.

The build-only v2 loop-skeleton host copies complete E6ADE0..E6B1D9 and
redirects relation/cache calls to guarded synthetic stubs. It replaces two
complete nine-byte geometry regions with controlled cutouts: the outer cutout
initializes or skips the admitted inner loop; the inner cutout records the two
native row indices and jumps to the original inner increment. The E6A450 call
is redirected to a nonreturning failure trap. All remaining bytes, call
relocations, cutout instruction boundaries and three source continuation
targets are checked. Five fixed cases cover empty, skipped and mixed rows and
3x3 eligible pairs. This is an instrumented traversal comparison only; it
bypasses feature geometry and contact evaluation and admits cache contents,
pointers, relation, stable ownership and no concurrent mutation. It does not
validate native cache resolution, actor lifetime, actual callbacks or feeding.
The initial build-only v1 failed on local label scope; it is retained and was
not executed. The v2 host has not been executed as of preparation and requires
independent review of register/stack preservation and cutout continuations.

Independent review found the original inner back-edge reads source local
`[esp+30]` even though the candidate cutout bypasses use of XMM6. The v2 host
was not executed. Build-only v3 explicitly initializes that stack word in the
outer cutout and its runner rejects nonempty stderr, while preserving v2.
V3 remains a review-only, instrumented traversal experiment; no native run or
contact/geometry claim follows from preparation.

The reviewed v3 host was executed and all five instrumented traversal cases
matched. Its receipt `target/cell-feature-loop-skeleton-result-v3/receipt.json`
SHA-256 is `37b8f6f7b6be5e0370ea453bc212c4b2dc030ef7b9f96bac373df42d5de19a79`.
A separate Rust adapter matched those five retained native outputs without a
native rerun; receipt `target/cell-feature-loop-rust-result-v1/receipt.json`
SHA-256 `6641bf3667e1440b60e56bb39379b7110a761660cd8f4cdd7fc73692792571ae`,
runner SHA-256
`1313dd1461abc4c08106d271cf0a8a694971dd55be33992e09893933ef2cb881`.
The Rust model rejects negative or over-64 counts and short arrays as added
fixture rules; native treats nonpositive counts as empty. The opt-in installed
fixture accepts separately supplied synthetic class arrays only, requiring a
passing checked pair, both-feature routes, actor-word/lifetime and cache
stability admissions. It does not infer E67A50 contents from retained rows.
All traversal-only geometry, contact, callbacks and live ownership limits hold.

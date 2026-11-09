# E6ADE0 first-candidate full-route comparison

The original both-feature callee E6ADE0 gets relation from E57260, resolves
both supplied actor triples via E67A50, iterates 72-byte cache rows, projects
both features, branches at squared distance `37800000`, and calls E6A450 on a
surviving candidate. Source then resumes traversal and E6A450 can mutate
shared actor/cache/selection state. The useful safe boundary is therefore the
first candidate under explicit stable synthetic inputs, not a claim that later
iterations see unchanged state.

The build-only v3 host copies the complete `E6ADE0..E6B1D9` body and uses
controlled relation/cache stubs and a nonreturning first-E6A450 capture stub.
It relocates only four calls and three floating constants, with source bytes,
targets and constant values checked. The capture inspects candidate arguments,
point and normal only on finite non-small cases. On the below-threshold case
it deliberately does not read the source's uninitialized point/normal locals;
the recorded marker identifies them as opaque. Fixed synthetic cases are
separated, tangent, overlap, exact threshold and below threshold, with one
row per actor and flag-clear direct transforms. Actor/cache/definition and
output pointers are guarded synthetic fixture inputs. On capture, the wrapper
restores its saved stack and callee-saved registers; native subsequent loops,
actual E6A450 and callbacks do not run. The v1 build failed due to scoped
labels and is preserved; v2 built but was not run and v3 adds saved-register
restoration and output guards. V3 was not executed as of preparation and
requires independent review of the source calling convention, 14 candidate
arguments, local pointer ranges, stack/FP state and all relocations.

The independently reviewed v3 host was subsequently run; all five fixed rows
matched its controlled Python model. Receipt
`target/cell-feature-first-candidate-result-v1/receipt.json` SHA-256
`4b90a8998e1c122c6bf6c078c5b7964b37c61d47ebb71fbe62eeefcc3f26db96`.
The first-candidate trap deliberately prevents E6A450 and later iterations;
the small-distance point/normal values remain unread and opaque. This is not
an observation of a native actor, E67A50 cache or live contact.

A bounded reusable Rust one-row-each composition now projects both admitted
features, computes the small/non-small geometry and stops at a pending first
E6A450 candidate. The small branch carries only an opaque marker; no point or
normal is initialized or read. Definition +B4 and +B0 are raw supplied code
words, and actor words are synthetic tokens, not definition or actor identity.
The separately pinned Rust adapter matched all five retained native rows without
rerunning native code. Receipt
`target/cell-feature-first-candidate-rust-result-v1/receipt.json` SHA-256
`71157aaa7c1fc0a2ef23dc4272f47233c0986cb9e26351c6ace4acdc1cbaade3`;
runner SHA-256
`c9cd92fc070d4cdfaed359eda6d2f55a5a4f72b2a9185790d5f041caaf4b6908`.
Finite inputs/intermediates, nonnegative radii, disjoint stable snapshots and
FP contexts are additional fixture admissions. E6A450, later iterations,
real cache/actor ownership, contact acceptance, feeding and callbacks remain
outside this model and comparison.

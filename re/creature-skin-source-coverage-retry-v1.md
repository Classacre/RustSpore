# Generated-skin source coverage and retry boundary

Static analysis uses pinned `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The retained 0473 payload has SHA-256
`dbbb55830f7f75eb5c45afd470a8bd0716cb08134e26f62a82674e28cd697137`.
Its earlier exact parse and this turn's raw-row audit report ten classes
`[5,5,5,5,5,5,2,2,2,2]`, with a set +08 mask 1 on the leading rows. The audit
receipt `target/creature-skin-row-branch-audit-20261108/result.json` has
SHA-256 `b89b4f3b108d2ead843815d9f0905f6b847911bf9e7088f3c465a64791f32e87`.
This confirms bytes, not that they were selected or remained unchanged in a
runtime object.

On the specific `413D70` builder path, the all-row flag gate is followed by a
leading class-5 loop and then a contiguous class-3 loop; it then resets the
query and calls `4FDA40`. For the *admitted* frozen 0473 rows, that class-3
loop has zero iterations, and the four trailing class-2 rows do not append
nodes or segments through these loops. Thus, under the already stated
same-object, no-mutation, successful vector/receiver and nonzero-threshold
admissions, the six direct class-5 nodes and five segment descriptors cover
the builder inputs on this path. The separate pinned native reference observed
one interior sample for each of those five segments at admitted 0.01, yielding
11 lanes. This conditional path coverage does not prove the saved creature's
runtime object identity, exclude other constructors/rebuilds, or imply the
class-2 rows have no role elsewhere in creature processing.

For other snapshots, `413D70` invokes `412890` only for contiguous class-3
rows whose +08 word has mask 1. The signed parent +04, signed links +02,
row +00, and a previous-node mapping select its inputs. `412890` calls a
virtual +58 provider using row +88/+84; a null result skips node/segment
creation, while nonnull results feed several property lookups and branches
selected by flags and property results. Row transforms and provider-supplied
tables feed geometry and keys. This is not yet a row-only reconstruction; an
implementation must first admit or trace the property provider, returned object,
tables, parent bounds and lifetime. It must not infer class-3 lanes from row
class or flags alone.

`414270` calls `4F9D60` on the intermediate's +38 query using its current
step. If that returns false, it exits; if true, `5402E0` is called with query,
intermediate +34 and step. Failure there also exits. If it succeeds,
`53EFE0` determines whether the overall attempt succeeds. If that returns
false, retry occurs only when the downstream object's +1C is nonzero and the
current step is strictly below binary32 `0.5`; the next step is a single SSE
multiply by `3F99999A` (1.2). On any exit the last attempted step is stored in
intermediate +3C and `540450` is called. Thus the final stored step alone is
not proof of success. `5402E0` calls preparation helpers and `53E8D0`; the
latter's result gates success. `53EFE0` processes a mutable work queue and can
return false after its +1C state becomes nonzero or another helper fails. Its
result cannot be determined from bounds and lane bytes alone.

Two state effects make retry modeling more than repeatedly calling a pure grid
function. First, `4F9D60` writes its aligned lower bound back to query +20
before later dimension validation; a subsequent attempt reads that modified
lower bound, while the upper bound is not equivalently reset in that slice.
Second, `5403C8..5403E2` clears MXCSR sticky flags and ORs FTZ plus the runtime
global at `15DC058` into the control word before downstream work. Whether and
how intervening callbacks change controls must be traced; a retry cannot be
claimed bit-equivalent to the existing scoped 1F80 fixture. The prior native
grid tests establish only their individual admitted calls, not this lifecycle.

The initial step is also path- and state-dependent. `418A80` begins with the
binary32 0.035 constant but may replace it through an indirect property lookup;
`414400` stores the supplied value on the intermediate +3C. `414370` later
calls `414270` using that field. A separate `4CB150` path receives a step and,
for sentinel -1, uses 0.028 or a property-dependent replacement; it dispatches
indirectly. Its caller at `4CD333` passes object +128, and the surrounding
routine can adjust that field after a measured work interval; an alternate
call passes +124. Those are separate statically traced paths, not proof that
any one chose 0473's step or that their objects are interchangeable.

The most actionable next implementation is a **controlled retry orchestrator**
that takes a frozen query snapshot, an explicit initial step, explicit outcomes
for the two downstream gates and a verified per-attempt floating-point policy;
it must carry the actual modified lower bound forward, record each attempted
step and distinguish success from a stored failed step. A fixed native host
should supply deterministic bounded gate stubs, compare attempt/exit sequence,
mutated bounds and stored step, and separately record MXCSR state. This can
validate the retry control boundary without pretending to reproduce the
untraced downstream queue or provider. Source-informed complete-grid claims
still require object identity, step/property selection and any additional
runtime changes. The read-only Ghidra log
`target/ghidra-creature-entry-v1/skin-retry-downstream-v1.log` and
`target/creature-skin-threshold-audit-20261108/callers-disassembly.txt` retain
supporting traces; instruction decoding controls the claims.

The checked `creature_skin_grid_retry_v1` orchestrator now accepts explicit
per-attempt downstream gate results and FP policies, carries the builder's
aligned lower bound forward, records the final attempted step and separates
success from failure. `align_lower` exposes the initial checked geometry
mutation, including the dimension-rejection case. The current supported policy
is scoped MXCSR 1F80; missing gate outcomes are errors, and the model omits
native callback effects and cleanup. Four focused unit tests passed.

A reviewed isolated host executed original `414270` and zero-lane `4F9D60`
instructions on five fixed synthetic cases with deterministic substitutes for
`5402E0`, `53EFE0` and `540450`. All 90 compared words matched the Rust model,
covering a retry that carries the modified lower bound, setup failure, retry
flag clear, the 0.5 stop gate and dimension rejection after alignment. The
receipt `target/creature-skin-grid-retry-native-20261108/result.json` has
SHA-256 `8f15b354b417fa932e16a2428debe7f0edd664e164a068aa5ffb9e612d29d836`.

The stubs deliberately leave MXCSR 1F80 unchanged; the observed comparison
does **not** reproduce the real downstream FTZ/runtime-global changes,
callbacks, queue, cleanup or other side effects. It uses zero lanes, not a
packed grid or retained 0473 input, and establishes no live object identity or
accepted creature step. It checks output canaries, not guard pages or sticky
flags. The loaded CRT path and file hash were checked but mapped-image bytes
were not independently verified; endpoint hashes do not prove continuous
stability or complete build provenance.

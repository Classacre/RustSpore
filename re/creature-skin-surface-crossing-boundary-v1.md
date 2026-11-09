# Conditional surface edge-crossing boundary

Static trace of pinned `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
These statements are conditional on successful preparation, valid bounded
storage and stable query/geometry pointers; no 0473 live selection is shown.

`53EFE0` processes its worker's mutable queue. It uses `53F540` to obtain or
cache corner field samples: a cache miss turns the packed lattice coordinate
into world coordinates with `53ED50`, calls the selected query +08 field
function, and stores both sample and associated record state. `540070` uses
the cube classification and edge tables to select edge pairs, calls `53FBA0`
for crossing vertices, and emits triangles for distinct resulting indices.
The precise tables, cache hashing/collision rules, work-list limits and
initial seed remain separate inputs; this trace alone is not a full surface
extractor or evidence of a successful queue run.

On an edge-cache miss, `53FBA0` obtains both endpoint samples through
`53F540`, evaluates the world midpoint with the same query +08 field
function, then calls `4FC7C0` on one half-edge. The sign of the midpoint
selects which half and orders its arguments. `4FC7C0` initializes parameter
bounds `lo=0`, `hi=1`, computes the fixed vector `end-start`, and sets the
field stopping tolerance to the smaller of `0.1 * query[+0C]` and
`query[+44]`, with the exact SSE operations and source comparisons. Its loop
always evaluates at least once: `t = (lo + hi) * 0.5`, then each world
component is `start + (end-start)*t`; it calls the query +08 function with
that point. A negative finite field replaces `hi` with `t`; a nonnegative
field replaces `lo`. It stops when `abs(field) <= tolerance` or after 20
field evaluations, and returns the last point plus optional t. Finite values,
masked exceptions, endpoint ordering, callback semantics and successful
storage are admissions; zero/negative tolerance and the 20-iteration boundary
must be handled as the instructions specify, not by an assumed root solver.

The `53FBA0` result is appended to geometry +94→+08 or accumulated into an
existing slot according to its edge/cache state. At successful queue finish,
`53F720` divides accumulated positions by per-position counts and may run
further correction and normal work. Only the resulting stable geometry +08
positions, through the conditional same-intermediate handoff, become the
`52B700` per-position skin requests. The boundary helper's one returned point
is neither the final mesh nor evidence that a particular palette is bound.

The most actionable next implementation is a bounded **4FC7C0 bisection
controller** with explicit ordered endpoints, threshold/tolerance and a
caller-supplied finite field callback. It should retain the actual SSE
multiply/add order, always-first evaluation, exact sign and stopping rules,
20-call bound and optional t; selected scalar/SSE/SSE2 field implementations
can be composed only under their own snapshot/dispatch admissions. A safe
native comparison can execute exact `4FC7C0`, `41D9B0`, `41DB40` and `41DAB0`
spans with a deterministic analytic query +08 callback substituted in a
synthetic receiver. Compare evaluated points, callback count, final point and
t separately on sign changes, exact-zero/tolerance boundaries and 20-call
exhaustion, while preserving FP controls and a failure receipt. It would test
the controller, not validate native field dispatch, queue topology, smoothing,
source identity or live isosurface.

Evidence: `target/ghidra-creature-entry-v1/skin-surface-position-trace-v1.log`,
`skin-surface-field-trace-v1.log`, `skin-surface-origin-interp-v1.log`, and
`target/creature-skin-threshold-audit-20261108/crossing-bisection-disasm.txt`.
Reproduce by PE-mapping `53EFE0..53F6F7`, `53FBA0..54006D`,
`540070..5402D7`, `4FC7C0..4FC9D1` and `53F720..53FB95`; original
instructions control floating-point and calling-convention claims.

## Bounded controller comparison

`creature_skin_crossing_v1` implements the admitted finite-input controller with
an explicit fallible field callback; the focused three tests passed via
`tools\build.bat test -p spore-core creature_skin_crossing_v1 --lib`. The
retained checked run `target/creature-skin-crossing-native-20261108/result.json`
(SHA-256 `1e8776e657fed6e13f97f9f8236dee2356f21cb03fbc17ccb8fe2fa9da723a73`)
reports both processes exit 0 and identical 1,700-byte output streams, SHA-256
`e9a902457fc228a5fbbc25280339bd23cdd8d38d5796568569c9eec931464973`.
The five synthetic cases cover exact midpoint, nonexact crossing, zero-tolerance
20-call exhaustion, negative-tolerance 20-call exhaustion, and reversed segment
with three changing coordinates. The comparison includes the final point and t,
count, and every callback point and returned field word. Pinned source spans,
inputs, host, comparator and runner are retained in that directory; the host
sets/checks MXCSR `1F80` and x87 control word `027F` and uses output canaries.

This validates only the sampled deterministic callback controller cases under
those controls. The host does not use guard pages or assert sticky FP flags;
hashes do not prove continuous file stability or build provenance. It does not
validate field dispatch, queue or edge-cache topology, smoothing, full mesh,
source creature identity, or a live isosurface.

## Corner-cache lookup boundary

`53F540` calls `540780` with the packed coordinate and its 32,768-entry,
12-byte-slot cache. At `5406B0`, the hash starts at `811C9DC5`, multiplies by
`01000193` with 32-bit wrap, then XORs bytes 2, 1 and 0 in that order. The
initial index is hash `& 7FFF`; after a collision, the stride is the hash
rotated by 16, OR 1, masked by `7FFF`. Slots whose key high byte is `FF` are
empty. The original tests the initial empty marker before equality, and on
subsequent probes tests equality first; either returns the same slot. After
more than 32,768 probes without a match or empty slot it returns `FFFFFFFF`.
`creature_skin_corner_lookup_v1` models this lookup with an explicit fixed-size
key table. Its three focused tests pass; the separately prepared original-code
comparison is pending review and is not a native result.

For `53F540`, a returned index below `8000` with a matching key is a hit and
returns two cached words. Otherwise the returned high bits are ORed into the
receiver status; world-coordinate conversion and field callback occur, a
24-byte corner record is written, and the key and result are written at
`index & 7FFF`, followed by a count increment. On a full table this means slot
`7FFF` is overwritten and status receives `FFFF8000`. Capacity, callback and
receiver state are separate admissions, and this trace does not establish edge
selection or final geometry.

The checked original-instruction lookup run in
`target/creature-skin-corner-native-20261108/result.json` reports all five
synthetic cases matched, including full-table exhaustion. Receipt SHA-256:
`5c6bb5a8c4410586465d84f16b42124a15b543187aa291c8296395937b60320b`.
Both processes exited zero and their 20-byte outputs hash to
`adca0b59fb699c0ca5442e1633c8af94269be7f908423b7eb8817eb4b9877531`.
This comparison covers hash/probe only, with canaries but no guard pages; it
does not validate mutation, field callback, queue, mesh or runtime identity.

### Source miss and callback ordering

The original `53F540` instruction sequence `53F5D7..53F6EF` first ORs
`lookup_result & FFFF8000` into worker +1C, then calls `53ED50` using the
packed key. At record base `worker[+50] + worker[+64]*18`, it writes key at
+00, the three transformed coordinate words at +0C/+10/+14, and `FFFFFFFF`
at +04. It does not write record +08 in this path. Then it calls the function
pointer at `worker[+90]` object +08, with that object as `this`, the local
coordinate pointer and the caller output pointer. After callback return it
sets output word +04 to the old count, copies key and both output words into
the cache slot `(lookup_result & 7FFF)*0C`, and finally increments worker +64
with 32-bit wrap. On a hit (`lookup_result < 8000` and exact key equality), it
copies both cached words to output and performs none of these miss writes or
callback. The callback's first output word is field data under the admitted
field-dispatch contract; its second output word is overwritten after return.

A bounded implementation can compose the already modeled lookup and world
transform with an explicitly supplied successful callback, record storage,
cache slots, status and count. It must preserve write/callback order, including
record +08, and represent or reject insufficient preallocated capacity before
mutation. A fallible callback abstraction must not imply native rollback: the
source has already changed status and the record before that call. Full-table
fallback, sentinel keys, count overflow, aliasing, and selected field/FP policy
need explicit admissions or source-faithful treatment. This remains a proposed
join and does not supply a runtime receiver or prove queue and geometry state.

### Bounded corner sample comparison

`creature_skin_corner_sample_v1` composes the lookup and world transform with
preallocated, nonaliasing key, value and record buffers and an explicitly
successful callback. Its three focused tests passed. The retained checked run
`target/creature-skin-corner-sample-native-20261108/result.json` (SHA-256
`2ce5237e532261a0604d24f700b2aec15c43783a088911944e211bcd6fa9d3e8`)
reports both processes exited zero and their 168-byte streams matched, SHA-256
`38a1df9236c315dfd6975dca2937e2dbdb9f4968fe82a84f712f3ae5ac829a4f`.
Three synthetic cases exercise hit, empty miss and full-table miss. The
comparison includes status, count, output, record words, affected slot and
callback count. The host executes original sample, transform, hash and lookup
spans with a fixed successful callback and preallocated receiver and buffers;
it checks table canaries, stack and 1F80/027F controls.

The callback fixture does not observe intermediate writes, so matching final
words does not independently establish their timing at callback entry. The run
has no guard pages or record/output canaries and does not test exceptions,
native allocation, runtime callback dispatch, live identity, queue or mesh.
The Rust model rejects invalid capacity and invalid world input before any
mutation, an explicit bounded admission rather than native failure behavior.
Hashes do not establish continuous file stability or build provenance.

## Ordered edge lookup and vertex boundary

`540070` selects directed edge traversal from source classification and
lookup tables, invokes `53FBA0`, and emits nondegenerate triples of returned
indices; this trace does not yet model those tables or queue topology.
`53FBA0` probes a 32,768-slot, 12-byte edge table via `50EB00`, using the
ordered pair, without canonicalizing it. `50EB00` takes `5406B0`'s hash of
the first key, wraps multiplication by `01000193`, XORs the entire second key,
and applies the same odd-stride probing and first-key high-byte `FF` empty
marker as the corner table. A full probe returns `FFFFFFFF`. The new bounded
`creature_skin_edge_lookup_v1` models this step; three focused tests pass and a
native comparison is prepared but not yet executed.

On an exact pair cache hit, `53FBA0` returns the slot's vertex index directly.
On a miss it ORs lookup high bits into status, samples both supplied corners,
and aborts if status is nonzero. It obtains the two position records, evaluates
the world midpoint, uses its field sign to select and order a half-edge for
`4FC7C0`, then either creates a geometry position associated with a corner
record or accumulates into its existing position and increments the associated
count. It finally inserts the ordered pair and resulting position index into
the edge cache. Vector allocation, interpolation, corner identity and cached
state are further dependencies. None of this lookup evidence establishes full
mesh topology, runtime field dispatch or a live creature identity.

The five controlled ordered-edge lookup cases matched native output in
`target/creature-skin-edge-lookup-native-20261108/result.json` (SHA-256
`530132ca3c57dfada2d4bc8fb7d6ff526f13637fcea38c4b4fe9f2647a4d5b95`). Both
processes exited zero; 20-byte stream SHA-256 was
`5450ea0f9e0c770a0e82ccef7ba84ae0141c933a800ca89caf45ee5b6974d649`.
This checks hash and probing with synthetic tables, not edge mutation, field,
geometry, queue or runtime identity. Table canaries do not provide guard pages.
Hashes do not establish continuous file stability or build provenance.

## Post-crossing geometry commit boundary

`creature_skin_edge_commit_v1` models the successful `53FE60..54006D`
post-crossing commit with explicit selected corner, first sampled corner,
ordered pair, finite crossing point, lookup result and nonaliasing preallocated
buffers. On an unused corner it stores the new position index and first-corner
index, appends the point and `[selected_corner,1]`; on a used corner the source
`41DC50` performs three SSE single-precision additions, then increments the
count with 32-bit wrap. Both paths store the pair and index in the selected
edge slot. Capacity, valid indices, finite inputs and results and the selected
MXCSR policy are admissions; invalid inputs reject before mutation, unlike
native failure behavior. A full-table lookup result is rejected because
`53FBA0` ORs its high bits into status and aborts before geometry commit. Caller
must also admit zero status, successful preceding operations and valid native
pointer arithmetic. The focused two tests passed; the model does not implement
selection, callbacks, allocation or complete mesh construction. The isolated
existing-position comparison and its limits are described below.

The isolated existing-position accumulation run
`target/creature-skin-edge-accum-native-20261108/result.json` (SHA-256
`e0d51edf96e89034f0b4ac20fb5be132e17d446809177f5bd5b0557a45081bba`)
matched all three synthetic cases: addition, cancellation and count wrap.
Both exits were zero and each 72-byte stream hashes to
`3ab52e5f1f10ca46a977c5e63102c3dc4543437913f2b1a34418dac2df7d1431`.
The host uses original `41DC50..41DCBF` (SHA-256
`80ce394855a831fad72ec371831e304817f407d2fad70d6663f91d128d0a44cd`)
and `53FF46..53FFCD` (SHA-256
`1618166d83f45fefa8c9e621998648506c2754485916007f7094b75d05c524e4`)
from the pinned PE SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The host constructs a synthetic frame and adds an epilogue at the fall-through;
it checks index, position words and count words under explicit FP controls.
This does not compare append, edge insertion, lookup, selection, allocation,
queue, full mesh or runtime identity. Canaries are not guard pages and sticky
FP flags are not asserted; hashes do not prove continuous file stability or
build provenance.

The checked append/insertion run
`target/creature-skin-edge-append-native-20261108/result.json` (SHA-256
`ac4397a3593bf25dd9ba31cb0b71b5011ead635422fbfd95ddd91e5366524475`)
reports matching 96-byte streams, SHA-256
`9fd2c6bc5f9295cce747753d4e26a9b4a22b433c2db4d52fdcbdda1a61cbbb03`,
for initial geometry lengths zero and one. The original slices
`53FE7A..53FF45` and `53FFCE..54006D` execute with a synthetic frame;
position and count append calls use preallocated-buffer stubs. The comparison
covers output index, corner links, length, appended position and count, and the
inserted ordered edge slot. Outer canaries are used without guard pages or
individual buffer canaries. This does not validate native allocator behavior,
edge selection, fields, queue, full mesh, or runtime identity; hashes do not
prove continuous file stability or build provenance.

### Controlled partial edge diagnostic

`creature_skin_edge_diagnostic_v1` composes an explicit ordered edge key and
sampled packed corner keys with the bounded edge lookup, corner sample,
midpoint/sign branch, crossing refinement and geometry commit. A caller
provides the field callback, origin, step, threshold, limit and FP admission;
state is synthetic scratch storage. The four focused tests cover a cache miss
through appended geometry followed by an edge-cache hit without another field
call, negative midpoint selection, shared-corner accumulation, and a partial
capacity failure. Errors can leave earlier state and callback effects intact.
The combined driver has not been independently compared with original
`53FBA0` as a whole; its component comparisons do not establish that larger
claim. It is a diagnostic point/edge fragment, not a queue, triangle mesh or
editor integration. It currently allocates a temporary 32,768-slot commit
buffer per miss, so it is not a performance-ready generator.

The existing edited-creature rendering seam is
`spore-bevy/src/viewer.rs` -> `AssetSource::edited_creature_assembly` in
`assets.rs`, returning `MeshData` through source-part assembly. A source-derived
body mesh would need complete lane production including class-3 and non-leading
rows, accepted retry/step and FP state, queue seeding and `540070` edge/table
traversal, finalized positions and triangle indices, normalization/correction,
normals, UV/material and skinning joins, plus independently admitted source
identity. The retained 0473 partial-grid adapter supplies frozen rows, a step
and host math and omits lanes; the current diagnostic must not be presented as
that creation's generated body or as an editor-ready mesh.

Boundary review: `50EB00` returns an initial slot marked empty before comparing
keys, but `53FBA0` then independently compares both stored words. Consequently
a synthetic `[FFFFFFFF,FFFFFFFF]` query against an identically initialized
empty pair is treated as a cache hit, returning that slot's admitted stored
value even if status is already nonzero. This reproduces the instructions and
does not establish that real queue keys can take that value. For a full edge
table, the driver preserves the high status bits and samples both corners
before the status abort, subject to its explicit successful finite callback and
capacity admissions. A malformed separately stored edge-value array is now
rejected rather than indexing out of bounds. Nonfinite midpoint or refinement
callback values reject before geometry commit; earlier corner/cache state and
callback effects may remain. The composed driver remains unverified as a whole
against native instructions.

## Queue seed, classification and triangle traversal trace

`53E8D0` first requires all three query-extent-to-step coordinates strictly
inside the source bounds, then scans packed lane groups (including padded lanes)
in lane order and each of the eight cube corners, transforming and field-testing
until a nonnegative sample is found. From that packed point it repeatedly adds
`[1,0,0]` to the three stored coordinate bytes with byte wrap and clears the
high byte (`53ECE0`); each candidate is transformed and field-tested. It
retains each nonnegative candidate, stops on the first negative field, and
fails after the counter exceeds `80h`. It enqueues the last retained packed
point with three queue metadata bytes zero using `53EF20`. This is a conditional
seed procedure, not evidence that retained 0473 supplies a successful seed.
The seed selection depends on complete ordered packed lane groups, accepted
step/origin and the actual selected signed-field callback and FP state.

`53EFE0` reads an eight-byte queue entry: packed coordinate and face mask in
byte 3, known-corner mask in byte 4 and negative-classification mask in byte 5.
Unless the known mask is already `FF`, it visits unknown corner bits 0..7 in
order, obtains cached samples via `53F540`, and ORs the corresponding class
bit only for strictly negative field. It writes known=`FF` and the resulting
classification back. It considers six face records, consults a 16,384-entry
queue cache, updates previously queued neighboring faces and masks, or enqueues
a missing neighbor only when the face's class bits are mixed. It checks status
before calling `540070`; successful exhaustion invokes finalization. Queue
hashing, capacity/overflow, cancellation, face propagation, and initial
classification values remain separate implementation boundaries.

`540070` maps the classification byte to a 12-bit crossing mask, finds its
least numbered set edge, selects its directed orientation from a corner-bit
mask, and follows directed edge alternatives in fixed priority: table at
`015E3928`, then `015E35F8`, then fallback `015E3870`. For each directed edge
it calls `53FBA0` with the directed corner pair, reversing the cache-key order
when its low direction bit is set. Each closed walk becomes a triangle fan:
first vertex stays fixed, next vertex advances, and a triangle is emitted only
when all three indices differ. Edge callback failure returns early. Coordinates
come from the eight 4-byte corner offsets at `015E3D08` and byte-wise wrapping
`53ECE0`, with high byte cleared. Complete triangle topology is conditional on
validated tables, callback success and the queue/finalization state.

The table addresses are not literal retained PE data: they are initially zeroed
virtual `.data` beyond the section's raw extent. Constructor `53D4F0..53E357`
reconstructs them from six source-defined oriented faces and a raw 24-byte
ordered edge-pair array at `0150DB8C`, SHA-256
`df42e1df6b64e47d5aab6022e6331945bbcd5b49526d64982c38f462418c1b9a`
(bytes `000404060206000200010405060702030105050703070103`). It initializes
orientation bits and the pair-to-directed-edge map, computes three successor
choices per directed edge from oriented face corners, then computes the 256
classification masks by XOR of endpoint membership. The constructor writes
three bytes of each four-byte successor entry; the remaining byte relies on
admitted initialization/state. Static decompilation logs are
`target/ghidra-creature-entry-v1/skin-surface-table-init-v1.log` and
`skin-surface-table-helpers-v1.log`; original instructions determine exact
byte and control behavior. No table initializer was executed for this trace.

A concrete next isolated piece is a bounded `540070` directed-walk driver with
explicit validated initializer-derived tables and an edge callback returning
vertex indices, emitting ordered nondegenerate triples; test all 256 classes,
ambiguous faces, repeated vertex indices and callback failure. Derive and pin
tables from constructor source operations, compare them and walk outputs to
original instructions in a reviewed preallocated host before joining the edge
diagnostic. This would produce per-cube triangle requests, not queue traversal,
finalized geometry, a complete source mesh or live creature identity.

`creature_skin_cube_walk_v1` now reconstructs crossing masks and directed
successors from the retained edge pairs and constructor-defined face order;
it explicitly admits zero initialization of the unwritten fourth successor
byte. Its bounded one-cube driver accepts a synthetic edge callback, retains
callback effects and already emitted triangles on failure, handles the source's
`FFFFFFFF` fan sentinel, and imposes a 256-step guard outside native behavior.
Four focused tests pass, covering all 256 classes, repeated-index suppression,
callback failure, byte-wrap and index sentinel. A pinned host is prepared to
compare original `540070`/`53ECE0` traversal over all classes and two special
callback cases. That host injects these model-derived tables and cannot
independently establish table-constructor parity; it has not yet been executed.
No queue, allocation, finalized mesh or runtime identity follows.

The first prepared cube host run failed before traversal: retained receipt
`target/creature-skin-cube-walk-native-20261108/result.json`, SHA-256
`7851d1dd573dff5b2b78cba84e817270b42b91de4f0e6a525cdddcd06792389c`,
records host exit 20 and empty stdout at the fixed-address allocation check.
The requested `015E3000` address was not allocation-granularity aligned; this
is a diagnosis of the failed check, not evidence of a comparison. A separate
host in `target/creature-skin-cube-walk-native-aligned-20261108` reserves the
aligned `[015E0000,015F0000)` range and checks all copied table ranges. It is
prepared for review, not yet a result. The original failure is retained.

The reviewed aligned host comparison in
`target/creature-skin-cube-walk-native-aligned-20261108/result.json` (SHA-256
`a8e2b2de78f7d7bd8fac8e12cb246e583a7d7ec2e2ebd14cee80a2079af99396`)
matched all 256 classifications plus repeated-index and callback-failure
controls. Both processes exited zero; their 176,472-byte streams hash to
`fe10fa31e28604c926ef4d6840f7b654ec465891c58ee9508514e066ba8fa63f`.
The comparison includes all supplied callback arguments and emitted triangle
indices for a byte-wrapping packed cube. Its frozen table fixture was generated
by the Rust table model and supplied to the original walk instructions; this is
conditional traversal evidence and does not independently validate constructor
tables or zero initialization of the unwritten fourth bytes. The host uses
synthetic callbacks, output canaries without guard pages and no live game,
queue, allocation, final mesh or runtime identity. Hashes do not establish
continuous file stability or build provenance. The preceding failed setup is
retained separately as documented above.

The first independent initializer host did not execute source instructions:
`target/creature-skin-cube-tables-native-20261108/result.json`, SHA-256
`27dc2d70e44b3f772a42913b23e4b54ef793b2d769ac623060e8eb90b97e16e6`,
records exit 20 and empty stdout at the aligned fixed-code `00530000`
allocation check. A separately prepared relocatable host preserves this failure
and is awaiting review. It patches the original switch's absolute operand and
six source switch-table entries to the allocated code base; the remaining
relative instructions retain their offsets. It uses a zeroed synthetic frame
and fresh zeroed table globals, so any future result remains conditional on
that initialization and on the explicitly omitted constructor setup.

The independent relocated initializer comparison in
`target/creature-skin-cube-tables-reloc-20261108/result.json` (SHA-256
`2a7452c92a1f1b6b37ecf7cba948c67a65c3a4ca3be7e70cb3d8a1778eef207d`)
matched all 880 table bytes, with both exits zero and both streams SHA-256
`82d16abec58cf5cdc7a9b1ea758ff54053d009625cc7bbf98814f23dea17da92`.
The source constructor slice and helpers ran with an explicitly zeroed
synthetic stack frame and fresh zeroed global pages; the switch's absolute
operand and its six target entries were relocated, while the other instruction
bytes retained their relative layout. This confirms these table bytes under
those admissions, including the untouched fourth successor bytes, not native
allocator lifecycle or concurrent constructor behavior. Output canaries are
not guard pages; no queue, finalized mesh or live identity is established.
The earlier fixed-code allocation failure remains separately retained above.

The controlled diagnostic now exposes a one-cube composition of the initialized
tables, directed walk, edge/corner caches, explicit field callback, crossing
refinement and preallocated geometry commit. Classification, packed cube,
origin, step and field remain caller supplied; no classification verification,
queue traversal, finalization or editor-ready mesh is inferred. Previously
emitted triangles and earlier state/callback effects remain on error. A focused
synthetic plane test exercises the complete one-cube path and cached repeat;
an exact-zero midpoint can merge three crossings at one corner and suppress
the degenerate triangle. This combined path has not been independently compared
against whole original `53FBA0`/queue execution despite the separately matched
components and tables.

## Bounded multi-cube continuation plan

The next independently testable source boundary is seeding, followed by queue
propagation and only then composition with the one-cube diagnostic. `53E8D0`
checks each extent divided by step strictly in `(0,255)`. It walks all four
lanes of each `ceil(query_count/4)` global packed group, including its last
padded lanes, and all eight corner offsets in order. `53EDE0` converts lane
centers using vector subtract, reciprocal-step multiplication, `CVTTSS2SI`
and low-byte packing. The seed field sequence starts with -1, seeks the first
nonnegative corner, then advances the low byte by one with wrapping until the
first negative result or the counter exceeds 128. On success `53EF20`
enqueues the last nonnegative point. A bounded seed API should accept the
ordered global lane groups including padding, aligned origin, extent, step,
selected finite field callback and exact FP admission; record all sampled
points, seed and failure reason. Do not substitute the separate per-cell
candidate grid for this global lane array. Test strict extent limits, scan
order, padding, zero/negative fields, byte wrap, 129-step limit and conversion
boundaries against isolated original instructions with preallocated buffers.

For propagation, the six 32-byte face records at `015E3A38`, written by the
constructor, are additionally required: the 880-byte table comparison above
did not include them. Extend the original initializer host in a separate
pinned receipt to compare those 192 bytes, including face coordinate, masks
and opposite-face mapping, under the same zero-init/relocation admissions.
`540580` uses 16,384 eight-byte cache slots, the `5406B0` hash, initial index
`hash & 3FFF`, odd stride `((hash >> 16) & 3FFF) | 1`, high-byte `FF` empty
marker, and `FFFFFFFF` full-probe sentinel. `53EF20` writes an eight-byte queue
entry and cache index, folds overflowing queue index into status, masks the
index to `3FFF`, and increments the count. A bounded queue must preserve order,
known/class/face byte updates, pending-versus-consumed cache hits, six-face
priority, mixed-sign neighbor admission and status timing. It should share the
same corner cache, field callback and geometry state as the existing diagnostic;
its checked-capacity failures are admissions, not claims of native overflow
behavior. Tests should include pending and consumed duplicate neighbors,
partial known masks, zero-valued fields, coordinate wrap, hash collisions,
full tables, finite callback and downstream failures, and retained partial
state. A pinned native host should compare queue/cache snapshots, status,
callback and cube-call sequences with preallocated bounded stubs before using
any emitted triangles beyond diagnostics.

The frozen retained 0473 rows conditionally cover six leading class-5 nodes
and five segment descriptors on the documented `413D70` path; the retained
reference has one interior sample per segment, hence 11 lanes. That conclusion
requires the same object, unchanged rows, successful construction and admitted
threshold; it does not establish live identity or selection. Other snapshots
may require class-3 virtual providers, property tables and non-leading row
processing. A real source join additionally needs ordered global packed lane
padding, stable selected field and query snapshot, source CRT arithmetic,
accepted property-derived step and retry/FTZ state. Successful queue exhaustion
still leaves source geometry finalization, normals, UV/material/skinning joins
and editor integration; intermediate triangles do not establish a complete
or live creature mesh.

The six constructor-derived face records were compared in
`target/creature-skin-cube-faces-native-20261108/result.json` (SHA-256
`e917aee98829cc6c537b656577f6a856fef661d22c77caf15b75dc38ef2ffaf2`):
both exits zero, all 192 bytes matched, stream SHA-256
`6b88a3eb78a3d70715706b6eba8e2defa4f32a23fb1a4478103f417af42103b9`.
The original initializer used the previously admitted switch relocation,
zeroed synthetic frame and fresh zeroed globals; its unwritten face bytes
therefore depend on that initialization. Canaries are not guard pages, and the
comparison does not establish allocator lifecycle, queue, mesh or live identity.

`creature_skin_seed_v1` now models the finite inverse coordinate conversion
and bounded source seed scan over caller-supplied global four-wide center
groups, including padding, with explicit logical count, query extent, aligned
origin, step, constructor-stored reciprocal and field/FP admission. It returns
a packed seed rather than inserting into a queue; invalid or exceptional
conversion behavior is checked rather than inferred from native failure.
Four focused tests passed, covering conversion, scan and callback order,
padding, strict extent and the 129th-forward-step failure. A six-case original
seed/inverse host with deterministic field and enqueue stubs is prepared for
review; it is not yet a native comparison result. Neither the per-cell grid nor
a particular live source object is inferred as its global lane buffer.

The reviewed seed host run
`target/creature-skin-seed-native-20261108/result.json` (SHA-256
`248649b77e3b8044b35b74877acc2c1dbf0dbfbb5c65444cfed8b6412cd7ba36`)
matched six controlled synthetic global-lane cases, with both exits zero and
both 9,432-byte streams SHA-256
`e313a10bb8b99be53674bb5109fcb6109e7c23a35a11b6f848c80d9560083d13`.
The cases exercise normal search, padding lane selection, 129th-forward-step
failure, strict extent 255, zero logical lanes and low-byte wrap; comparison
includes success, seed and every callback position word. Original source spans
are drawn from PE SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`;
the host supplies deterministic field and preallocated enqueue stubs, admitted
corner bytes and explicit MXCSR `1F80`/x87 `027F` controls. This does not
validate actual queue traversal, allocator lifecycle, source lane completeness,
mesh or live identity. Output canaries are not guard pages, and hashes do not
prove continuous file stability or build provenance.

`creature_skin_queue_v1` now provides a bounded synthetic queue beginning from
an explicitly supplied packed seed. It preserves source slot-zero seed-cache
insertion, the 16,384-slot hash/probe, ordered corner classification and six
face processing, pending-versus-consumed neighbor behavior, face/class/known
masks and the status check before invoking the existing one-cube diagnostic.
It uses independently compared face records and shared corner/edge state, but
its callback and storage are explicitly admitted and checked capacity errors
are not native overflow behavior. It returns `Drained` without native geometry
finalization. Five focused tests passed. A first-item original-instruction
comparison with controlled corner and cube stubs is prepared for review;
that host deliberately stops after the first cube and cannot establish full
queue execution, generated mesh or live source identity.

The checked first-item queue comparison
`target/creature-skin-queue-native-20261109/result.json` (SHA-256
`8f17963fdf6666eb7a16c3c42fe6837573e15c9d33b1f91a39660df9a7214cc1`)
matched all four controlled cases. Both exits were zero; 524,992-byte streams
hash to `0f51fd851602723389f71cb96211aec9f16a153b22ebdcfcc3fe3a7a0b52763c`.
The comparison covers uniform and mixed classifications, neighbor enqueue,
pending mask propagation and consumed-neighbor behavior, with cursor, count,
status, first 16 queue records, classification callback coordinates and all
16,384 cache key/index pairs. Original queue, face translation and hash code
uses independently compared face/corner bytes and relocated switch targets.
The host stubs corner field samples and lifecycle/progress calls; the cube
stub deliberately fails to stop after the first item. It uses preallocated
16-entry queue storage and outer canaries without guard pages. It does not
validate whole queue completion, actual cube/field composition, finalization,
native allocation, full mesh or live identity; hashes do not establish
continuous file stability or build provenance.

`Queue::run` adds an explicit item budget around the same shared diagnostic
state and returns when the synthetic queue drains, without source finalization.
The focused seven-test suite includes a compact synthetic sphere composed from
an explicit global lane seed through multiple cubes and partial triangles,
and a budget-exhaustion case retaining prior state. The model-only sphere
comparator reports 32 processed entries. A native original-queue comparison is
prepared for review using a pure sphere corner stub, successful recording cube
stub and no-op finalization; it is not yet a result and cannot establish native
edge geometry, final mesh or live source identity.

The reviewed multi-item comparison
`target/creature-skin-queue-multi-native-20261109/result.json` (SHA-256
`cdd3b3d54ed87aef231d453226b6560d4acbbd68ba7ac1ba095d6e83e86944b1`)
matched the synthetic compact-sphere queue through exhaustion: 32 processed
items, 32 queued entries, zero status, and matching cube request order. Both
exits were zero; the 135,184-byte streams hash to
`3a2d140c145f21397f70aa33fd94df912de9f4cf2d047a712b193b582da032db`.
The native run starts from explicit seed `020203`, uses original queue,
face and hash instructions with admitted matched tables and pure synthetic
sphere corner-field stub. Its cube stub only records and succeeds, and its
finalization and lifecycle stubs do no work. Thus the comparison covers queue
and cache state and cube request order, not the composed edge geometry,
native allocation/finalization, a completed mesh or live identity. The host
uses 16K preallocated queue/cache storage, a 256-cube guard and outer canaries
without guard pages; hashes do not establish continuous stability or build
provenance.

An isolated original-instruction queue-to-cube/edge/corner/bisection host is
prepared for review in `target/creature-skin-queue-geometry-native-20261109`.
It uses a fixed synthetic sphere and seed, bounded preallocated append stubs,
a deterministic field, matched face/cube tables and no-op finalization. The
model-only snapshot has 32 queue items, 81 corner records, 18 unfinalized
position accumulators, 32 triangles and 327 field calls. The planned direct
comparison includes queue/caches, corner records, partial geometry and callback
words. This prepared host is not yet a result and does not establish native
allocator behavior, finalized mesh, source-creature identity or live behavior.

The reviewed composed host run
`target/creature-skin-queue-geometry-native-20261109/result.json` (SHA-256
`bb97ff5151c4f199df92c6996d6f2f5bd8e363cfc808b0471787f63bc8c795d9`)
matched the synthetic sphere snapshot through queue exhaustion. Both exits
were zero; each 1,107,996-byte stream hashes to
`a7ab620ec825cf821fc069fbd0595a2f2648abad5287d0219a0b0989f3c2eabf`.
The compared snapshot has 32 queue items, 81 corner records, 18 unfinalized
position accumulators, 32 triangles and 327 field callbacks; it includes queue,
corner and edge cache words, corner records, partial positions/counts/triangles
and each callback position/value word. The host executes original queue,
cube, edge, corner, bisection and helper instructions from the pinned PE with
matched tables, explicit `1F80`/`027F` controls and a synthetic sphere field.
Position/count/triangle appends use bounded preallocated stubs; progress,
lifecycle and finalization are no-ops. The explicit seed and deterministic
field are synthetic. Outer canaries are not per-buffer guards or guard pages;
callback and append caps bound the fixture but do not validate native overflow
behavior. This result does not establish source creature identity, native
allocator/finalization behavior, or a finished/live mesh. Hashes do not prove
continuous file stability or build provenance.

The conditional 0473 model-only diagnostic uses retained frozen native lane
fixture `target/creature-skin-grid-cells-native-20261108/fixture1.bin`
(SHA-256 `7d61bef919b70509fd7721c9f31913ad99604fe91313e708596d39a773d89e37`),
itself bound to the retained 11-lane native segment receipt and payload. It
admits caller step `0.15` (not established as source-selected), zero padding
of the final global four-lane group, an explicitly constructed 512-cell grid,
scalar field dispatch and `1F80`/`027F`. The model-only snapshot
`target/creature-skin-0473-surface-native-20261109/snapshot.bin` hashes to
`557997a9633a69778e8151f519eed42129c002a7826d91f521674d3ecee72198`;
`prepared-model.bin` hashes to
`53ba615dfe1e6cab39d67ac6a1ab10d16e7c5e59ec3ff41e3ea452ff50cb4496`.
It reports seed `00020203`, 72 queued cubes, 154 corner records, 44 unfinalized
positions, 84 triangles and 500 field callbacks. This is model-only output;
the 0.15 grid snapshot is not independently compared to the original builder,
the field dispatch and runtime object identity are admitted, and no native
geometry parity or finalized/source-live mesh is claimed.

The reviewed conditional source-derived host run
`target/creature-skin-0473-surface-native-20261109/result.json` (SHA-256
`8ffd90b251ef5e2c1067bea55d36e5cdcca02f9df96bcdb280a417e1c1e332b2`)
matched the full 1,107,996-byte model snapshot. Both exits were zero and both
streams SHA-256
`53ba615dfe1e6cab39d67ac6a1ab10d16e7c5e59ec3ff41e3ea452ff50cb4496`.
Original scalar field, seed, queue, cube, edge, corner and bisection instructions
were exercised with a logging field wrapper, bounded preallocated append stubs
and no-op lifecycle/finalization. The snapshot had 72 queue items, 154 corner
records, 44 unfinalized position accumulators, 84 triangles and 500 field calls;
the comparison includes queue/caches, records, partial geometry and field words.

The runner checks retained payload/row and native segment receipt hashes,
verifies its 11 lane words and bounds against that receipt, re-derives the
pinned Rust-built 512-cell snapshot and model output, then compares native
output. This result admits frozen same-object rows, threshold `0.01`, caller
step `0.15`, a zero twelfth global padding lane, scalar dispatch and explicit
`1F80`/`027F`. Original grid construction at `0.15`, the actual runtime step
and dispatch, native allocator/finalization and live source-object identity
remain unestablished. The host uses outer canaries without guard pages; hashes
do not prove continuous file stability or build provenance. It does not
establish a finalized mesh or an editor-ready 0473 body.

The standalone `creature_skin_0473_surface_compare_v1` now states in its module
documentation and CLI help that it accepts a generic caller-supplied eleven-lane
fixture; only the pinned checked runner admits the retained 0473 provenance.
Probe mode also emits that warning, while binary output modes remain quiet for
comparison. Rebuilding changed the comparator/source hashes: the historical
`run_checked.py` pins are now stale, but its original matched receipt and raw
streams remain unchanged. `run_checked_current.py` is a separately named,
updated pinned runner (SHA-256
`9f90f09c2e7709406bddf4b5be4f245e7e9c28f6a30c9e1ca1824c8c95b1782f`);
it has not been executed. The rebuilt model's snapshot and geometry outputs
were checked byte-for-byte against the previously retained files without
running native instructions.

For the next small finalization boundary, `53F951..53F983` loads a count word
as a zero-extended 64-bit integer via `FILD`, computes x87 `1/count`, stores the
reciprocal to binary32, then calls `41DA40`, which multiplies each accumulated
position component by that binary32 reciprocal using separate SSE `MULSS`
operations and stores all three words. A bounded model can accept explicit
finite accumulated positions, nonzero unsigned counts, x87 precision/rounding
and MXCSR controls; zero count, nonfinite results and exceptional/status flags
remain outside that admission. An isolated native host can execute precisely
these source spans with a synthetic frame and controlled position/count data,
without the later `4F9910` gradient/correction branch. Useful cases include
counts 1, 2, 3, `01000001`, `FFFFFFFF`, signed zero and finite subnormal values,
under explicit `027F`/`1F80`; compare reciprocal and all position words. This
would establish only averaging, not `541380` index processing, gradients,
correction, normalization, allocator lifecycle or a finished mesh.

The separately pinned current runner was subsequently reviewed and executed.
`target/creature-skin-0473-surface-native-20261109/current-result.json`
(SHA-256 `92fa24b3febc90629b203989ca9f247a95735d26ba3eb8061414100d2006fe7c`)
reports matching host/model 1,107,996-byte streams, both SHA-256
`53ba615dfe1e6cab39d67ac6a1ab10d16e7c5e59ec3ff41e3ea452ff50cb4496`.
It retains the same frozen-row, supplied-step/padding/grid/dispatch and
no-finalization admissions as the historical receipt. The earlier receipt and
streams were not overwritten.

`creature_skin_average_v1` implements the bounded unsigned-count x87 reciprocal
spill followed by three scalar SSE multiplications; its three focused tests
passed. The independently reviewed run
`target/creature-skin-average-native-20261109/result.json` (SHA-256
`4055cfad2f23bd3b7ff7dd14c1c687c0b71cbb449a99bf23e0788bc49acc4a1b`)
matched all six controlled cases: counts 1, 2, 3, `01000001`, `FFFFFFFF`,
signed zero and smallest subnormal inputs. Both exits were zero, with
identical 96-byte streams SHA-256
`973e2f3d77be3273653a4e6e903cfbfa7034747e774cae9db4f39044f8236258`.
The host executes original `53F951..53F983` and `41DA40` with a synthetic
frame and fall-through trampoline under x87 `027F` and MXCSR `1F80`. The
finite nonzero-count model and position canaries do not cover native zero-count
or exceptional behavior, sticky flags, guard pages, later `541380` index
postprocessing, `4F9910` gradient and `4FC380` correction branches, normal
construction, allocator lifecycle, finalized mesh or live identity. Hashes do
not establish continuous stability or build provenance.

The next bounded finalization slice is `541380`: it passes the source index
range to the `11E0534` thunk, whose IAT entry `013CC47C` resolves to MSVCR90
`memcpy`, then returns the destination advanced by the copied whole-word count.
For admitted valid, nonoverlapping, whole-word ranges this is an ordered copy,
not triangle reordering. `creature_skin_index_copy_v1` models it with checked
preallocated capacity; its two focused tests passed. The retained input
`target/creature-skin-index-copy-native-20261109/indices.bin` (SHA-256
`01111d5dd13ff186458dcff418e4186c1d59d8d163d5ca50af7bba7db05b9028`)
contains all 252 index words of the matched conditional 0473 pre-finalization
snapshot, with its frozen-row, supplied-step/padding/grid and dispatch
admissions. An original `541380` host with an explicitly bounded memcpy stub
is prepared for review; it does not execute the native CRT allocator or prove
gradient, correction, normal, final mesh or live identity behavior.

The reviewed index-copy run
`target/creature-skin-index-copy-native-20261109/result.json` (SHA-256
`30cbbcc2f76884bfe1071e47f1accef1fb40dd1b6a2a4c700be9e845e31705bb`)
matched the empty, single-word and full 252-word conditional 0473 cases. Both
exits were zero and their 3,036-byte streams hash to
`5d6bd1ff3502e9445f47ff58985fdf7788846c63cdc8d62664b71f09789a2b34`.
The original `541380` instructions execute with a bounded nonoverlapping
cdecl memcpy stub, not the native CRT function. The runner binds the input
range to the matched conditional 0473 host output and receipt. Destination
canaries are not guard pages; this does not validate native allocation,
gradient/correction, normals, finalized mesh or live source identity. Hashes
do not establish continuous stability or build provenance.

All 44 conditional 0473 accumulators and counts were independently averaged
by the original slice in
`target/creature-skin-average-0473-native-20261109/result.json` (SHA-256
`6bef492ef52cc742f63f48df2d5ffb2e4be84f120bfc19e05d544d7d7046a997`).
Both paths exited zero; their 704-byte streams hash to
`ec7c9288caf3ba636782d98b91abab0e393ba0c2e637094109d10902d2340182`.
Inputs are bound to the matched pre-finalization output, and the result uses a
synthetic frame/trampoline, `027F`/`1F80`, finite nonzero counts and position
canaries without guard pages or sticky-flag assertions. Frozen rows, supplied
step, padding, scalar dispatch and no-finalization admissions remain; this is
not gradient/correction, normal or live-mesh evidence.

A gradient snapshot for subsequent work is separately retained at
`target/creature-skin-gradient-0473-native-20261109/gradient-snapshot.bin`
(SHA-256 `23aa1f974db1f3edfc7f6174f773e34efe4b52e036406492fae99de3f53c880a`).
It derives each lane +60 word as binary32 coefficient × scale from the admitted
scalar grid snapshot, matching the traced `4F8930` construction formula; the
previous scalar-only snapshot's +60 words were zero and must not be treated as
source gradient coefficients. This derived grid remains an admission pending an
independent original-instruction gradient comparison.

`creature_skin_gradient_v1` models finite scalar `4F9910/4F9A20` on an
explicit addressed grid with separately supplied lane +60 derivative words,
ordered SSE accumulation and the source magnitude threshold. Its two focused
tests passed. A model-only probe on the 44 matched averaged conditional 0473
points and enriched grid reports 44 usable gradients. The separate original
instruction host in `target/creature-skin-gradient-0473-native-20261109` is
prepared for review, not yet a native result. Its enriched grid derives +60 as
coefficient × scale; the caller step `0.15`, zero padding, grid construction,
scalar dispatch, frozen same-object source rows and FP controls remain
admissions. Usable gradients would enter `53F720`'s normalization path, which
calls the MSVCR90 sqrt thunk; neither that branch nor correction, full
finalization, native allocation or live identity is established here.

The reviewed gradient run
`target/creature-skin-gradient-0473-native-20261109/result.json` (SHA-256
`853c78c4ac52188efde5f4fa0fefe5304557e7cdc3e87ee5170b9887c3cb6231`)
matched all 44 gradient vectors and usability words. Both exits were zero,
their 704-byte streams hash to
`00eb6424c59bd3b53fd577ed28ac4962e26898eff955638e5b211f7c16873915`,
and all 44 vectors were usable. Inputs are the independently matched averaged
points and the separately enriched grid whose +60 words are derived as
coefficient × scale. Original scalar `4F9910/4F9A20` and address helpers ran
with the 0.15 model-built grid, frozen-row/padding admissions and `1F80`/`027F`.
The 0.15 grid builder, actual runtime dispatch and identity remain unestablished;
the comparison does not cover correction, normalization, allocator, full
finalization or finished mesh. Gradient canaries are not guard pages and sticky
flags were not asserted; hashes do not prove continuous stability or build
provenance.

The reviewed direct usable-gradient normalization run
`target/creature-skin-normal-0473-native-20261109/result.json` (SHA-256
`2307929623cd4e2499cb4867bde56d6d89bd5aff0017a4edfa1a697f5111782d`)
matched 44 three-word normals (528 bytes, SHA-256
`9e98416ac15a9e52e3bdb1c611b4c3d6e074e94bdfe43e37207496bd0c6e8651`)
with both exits zero. Original `436D60`, `453970`, `421BD0`, and the
`11E0528` thunk ran under `1F80`/`027F`; the checked runner verified the loaded
CRT path and installed file SHA-256
`ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185`.
The runner used private non-switched desktops for both children. It did not
verify mapped DLL bytes, sticky flags or guard pages. The x87 square-root spill
model is compared for these inputs, not admitted as general CRT equivalence.
Frozen partial rows, supplied step/padding, derived grid and scalar dispatch
remain conditional; correction, allocator, remaining finalization and live
identity remain unestablished.

A conditional partial-geometry artifact can join the three independently matched
streams by record index: the 44 averaged positions at bytes `4..16` of each
16-byte average record; the first 528 normal-host bytes as 44 three-word
normals; and the 252 little-endian index words in the last 1008 bytes of the
index-copy host output, whose preceding count is 252. The index range is 0..43
and all 44 positions occur, giving 84 index triples. An opt-in exporter should
verify all three receipt and stream hashes, matched status, finite position and
normal words, index count and range before writing, and retain these admissions
with the artifact. This join does not establish source winding conventions,
UVs, skin weights, materials, correction or remaining finalization, and is not
a production editor mesh or a claim about a live 0473 creature.

The opt-in `tools/creature_skin_0473_partial_export.py --output <new-path>`
validates the three fixed receipt and host-stream hashes, matched status, host
exit, stream lengths, finite vectors, index count and index bounds before
writing a new JSON inspection artifact. It does not fabricate UVs or materials.
The retained output `target/creature-skin-0473-partial-inspection-20261109.json`
has SHA-256 `c72cd25939c8bae1425c6a9b5dbaf47edfb5cd1b6a87ad37df783f54894c68cf`.
Its focused fixture test verifies the real join and rejects a modified stream,
nonfinite position, wrong index count and out-of-range index. The provenance and
partial-geometry limits above remain part of the artifact.

The reviewed influence-query run
`target/creature-skin-influence-0473-native-20261109/result.json` (SHA-256
`8978465edcd7c225dda41f5785e2769104ce3566caa12170d6bc07ba4243d061`)
matched all 23,736 bytes (stream SHA-256
`2b131ca6e00d4a0e0f8b922a117dbac0d9dc6f338f5ea8c9b7246869bcc38e9e`)
for the 44 conditional averaged positions and null-cell and out-of-range
controls. Each source point produced one to five ordered key/weight requests;
the controls produced none. Original `4F9C30`, lookup, and finite lane
instructions ran with `1F80`/`027F` and a checked 64-entry preallocated append
stub; both children used private non-switched desktops and were reaped before
closure. Native allocation, guard pages and sticky flags were not tested. The
frozen rows, model-built grid, 0.15 step, zero padding, same-object/lifetime
and scalar arithmetic admissions remain. These are raw influence requests,
not final skin records, palette identifiers, live identity or a complete mesh.
Endpoint hashes do not prove continuous stability or build provenance.

The compared 44-point requests use clear keys 0..5 and high-bit keys
`80000001..80000005`. Under the separate frozen-row 0473 inspection, the
same supplied ten-row payload gives ten constructed edges, eleven endpoint
member lists and the remap `[0,1,2,3,4,5,6,7,8,9]`; see
`creature-skin-row-edge-remap-static-v1.md`. These extents cover the observed
keys but numeric range does not establish same-object identity. The endpoint
+40 coordinate words required by `52B700` clear-key projection have not been
joined to this fixture. A bounded next comparison can use the pinned query
records and independently explicit valid endpoint coordinates, members, edges
and remap, with correction and smoothing counts admitted nonpositive, and
compare ordered accumulation, ranking and initial 20-byte records. Such a
comparison would remain conditional on the supplied coordinates and would not
establish actual native post-mutation preparation or final palette binding.

The reviewed conditional accumulation/ranking receipt
`target/creature-skin-weights-0473-native-20261109/result.json` (SHA-256
`5a8f511b0a98b50ec8f8fe5468c3ef36ed0905689482172c89aab7b8cd82aa09`)
matched all 44 ten-word accumulation snapshots (SHA-256
`a4fff9c21464e4b3423cdc77485c4b399f30ff952a72cfd6a908a293297714a4`)
and 44 four-entry rankings (SHA-256
`cf52c4648598a6ba04e42beffb3d393bc4a474707b4e5ef1f4a2c137a511c24a`).
Each of three children exited zero and its private non-switched desktop closed.
The 44 query sets and 0473 row-derived edges/members/remap were pinned, but
endpoint coordinates were explicitly synthetic `(i,0,0)` for `i=0..10`. The
same-object/lifetime relationship and no correction or smoothing remain caller
admissions, not observed native state. Source high/clear accumulation slices
ran under host-supplied outer-loop dispatch; ranking consumed pinned model
weights only after the accumulation comparison matched, and used a bounded
insertion substitute. Original outer-loop behavior, record normalization,
native allocation, correction, smoothing, palette identity, live runtime and
final mesh were not compared. Canaries were not guard pages, sticky flags were
not compared, and endpoint hashes do not prove continuous stability.

The reviewed conditional initial-record run
`target/creature-skin-record-0473-native-20261109/result.json` (SHA-256
`361dcd3d3e11c065a124357eb30678687c43d2c2927b5a0d30ff8c71d1dbeeca`)
matched 44 20-byte records (880 bytes, SHA-256
`739b7c31d21fc9cbaf171ae26294775eec4283196aef2638351d69d8ae2b4b14`),
with both exits zero and private non-switched desktops closed. The original
`52C2D7..52C3DF` record writer used the matched ranks, synthetic frame and
preallocated guarded output under `1F80`/`027F`; canaries were not guard pages
and sticky flags were not compared. The endpoint coordinates `(i,0,0)` were
synthetic, and same-object/lifetime plus nonpositive correction and smoothing
are admissions. Native outer-loop, allocation, later smoothing, actual
palette identity and live runtime remain unestablished.

An optional inspection-export extension can attach these 44 records by the
same position ordinal if it verifies this receipt and stream, pins the earlier
query/accumulation/ranking evidence chain, checks exactly 880 bytes and finite
weight words, and explicitly labels the result synthetic-coordinate initial
records. Each record contains four raw low-byte indices followed by four
binary32 weights; preserve raw words and order. Do not label these bytes as
bone/palette IDs or final skin weights: the retained identity remap applies to
this admitted snapshot only, earlier source identity is conditional, and later
smoothing may change the result. The current geometry-only exporter remains
unchanged; no palette binding, UVs or materials follow from this possible join.

The exporter now offers `--include-synthetic-initial-records` as a separate
opt-in. It verifies the pinned record receipt and native stream, validates
count and finite weight words, and stores raw four-byte indices and four
hexadecimal binary32 words per position with explicit synthetic-coordinate and
non-palette limits. Without the flag the output remains byte-identical to the
original inspection artifact (SHA-256
`c72cd25939c8bae1425c6a9b5dbaf47edfb5cd1b6a87ad37df783f54894c68cf`).
The optional artifact `target/creature-skin-0473-partial-synthetic-records-20261109.json`
has SHA-256 `cbbf8201b98e9ffbe5cc61468616e022ded775636b33924b03b65676a9185a38`.
Fixture-present tests passed; in a clean-checkout simulation, synthetic tests
passed and the two target-fixture tests skipped. These records are not final
skin weights or established bone/palette identifiers.

The reviewed frozen-row endpoint run
`target/creature-skin-endpoint-0473-native-20261109/result.json` (SHA-256
`d5b02ebac955bf06b379cddac20da2ab8143c24193d5d4474f602f8564f75391`)
matched all 11 map words and 11 xyz vectors: 176 bytes, SHA-256
`a51bad5fd6ce39e8604d98d8767fc02a83d526650b34bd073e3abbf36263b657`.
Both exits were zero and private non-switched desktops closed. Original
`4CB6AD..4CB9AB` and `41DAB0/41DB40` ran with fixed admitted class-5/class-2
rows, preallocated map/endpoints, synthetic frame and `1F80`/`027F`; class-3
calls were excluded by fixture preflight. Frozen post-mutation rows,
same-object/lifetime and successful allocation are admissions; `471D90`
mutation, native allocator, runtime identity, palette and final mesh remain
unestablished. Canary checks did not provide guard pages or compare sticky flags;
endpoint hashes do not establish continuous stability or build provenance.

The reviewed row-coordinate accumulation/ranking/record run
`target/creature-skin-weights-0473-rowcoords-20261109/result.json` (SHA-256
`90a1c5c82fa48258a43f5c8d79d2372029add2efaf0b943494aeda7ab0e4dffa`)
matched all 44 accumulation vectors, rankings and initial records in sequence;
all four children exited zero. The 880-byte record stream hashes to
`de642b7e6536e7676e081f7a5a17710c167f7a62573cfb5bd33dfbd418ed686a`.
These results use independently matched row-derived coordinates and query
records with admitted frozen post-mutation rows, same-object/lifetime, supplied
grid and nonpositive correction/smoothing. Outer dispatch, allocator, actual
runtime identity, palette and later smoothing are unestablished. Original
accumulation slices used supplied dispatch, ranking used a bounded insert stub,
and record writing used preallocated storage. Canaries were not guard pages;
sticky flags and continuous file stability were not established.

The separate exporter flag `--include-row-coordinate-initial-records` pins
that receipt and record stream and checks all four exits, length and finite
weight words. It emits raw index bytes and hexadecimal weight words with the
above admissions, without asserting palette identities or final skin weights.
`target/creature-skin-0473-partial-row-records-20261109.json` SHA-256 is
`7277fd4a0ab3f2049780dfeb58e71be78108b12535e672ba2b7bcc26c9486055`.
Default and synthetic-option outputs remain byte-identical to their previous
artifacts. Seven fixture-present tests passed; a clean-checkout simulation
passed four synthetic tests and skipped three unavailable-fixture tests.

The source selector for this route is not a field of the ten retained rows.
At `4CC0DC..4CC119`, `4CC060` reads global object `[015FE918]` and queries
`6A21C0(A0304059)` for a float, then `6A2110(4F6A84FD)` for an integer;
the latter is passed as argument 9 to `52B700`. Its signed-positive value
selects later `52B270` smoothing passes. `6A2110` returns zero on its missing
property branch, but this does not establish that the active global object
lacked that property or what value it returned. Independently, `4CC3B3..4CC44A`
calls `4BD000` on the intermediate +08 object's +18 word; only result
`2B978C46` passes the queried float as argument 10, otherwise source passes
positive zero. Neither the global property's value nor the runtime type word,
provider selection or object lifecycle is observed for retained 0473; therefore
no-smoothing and no-correction remain explicit admissions.

The hash-bound source audit
`target/creature-skin-0473-selector-audit-20261109/result.json` (SHA-256
`c36117e123395fb1d24be5488d8a06d7f8dd87bf79ab069db5276c3b86e80713`,
script SHA-256
`4882eb100df78a28df0dcf0ffebf09ba40a67681ab53898bb50ea13a61f25350`)
checks the ten frozen-row classes, the separately pinned identity remap and all
44 matched row-coordinate initial records. It finds 138 positive slots whose
indices cover 0..9 and 38 zero-weight slots. Under the admitted unsmoothed,
uncorrected path those positive indices are remap row ordinals; zero-weight
placeholders do not establish influences. This is a conditional numeric join,
not proof of a canonical palette binding: the constructor/finalizer path,
identical row identity and lifetime, model-world binding and active group
bounds must all hold as detailed in
`creature-generated-influence-palette-identity-static-v1.md`. No live property
or palette was observed; hashes do not prove continuous stability.

The global selector object has a concrete conditional initialization route.
Static initializer `12F1DD0..12F1DDB` calls `6A1750(0x52)` on static storage
`0163A450`; that constructor sets vtable `01408818`, initializes its named
variant storage and allocates a zero-initialized 0x52-word indexed array on
success. Startup `7E88FD..7E8907` passes that address to `67DB70`, which
writes `[015FE918]`. On the `6A9640` initialization path,
`6A9686..6A96BB` prepares a request with instance `08EA246B`, type
`00B1B104`, group zero, and invokes the manager's virtual +58. On its
successful loading branch `6A96D2..6A9716`, a virtual +0C supplies a property
object and the global object's virtual +2C receives it. For the constructor's
vtable, +2C is `6A2580`: it resets named storage, walks 0x18-byte source
entries and invokes its virtual +14 setter. The +14 target `6A2B70` routes
IDs >= 0x52 to `6A28D0`, which updates or inserts the named variant. Both
selector IDs are >= 0x52. These observations require successful constructor,
manager, request and virtual calls and no later changes. Generic setters and
indirect calls can change the global object; a scan for direct E8 calls found
five candidate `6A28D0` calls (including `6A2C12`), while indirect or encoded
references remain outside that scan. A single direct call to `67DB70` was found
at `7E8902`; this is not a proof against indirect calls or memory writes.

A separately retained source PROP audit
`target/creature-skin-selector-prop-audit-20261109/result-v2.json` (SHA-256
`e5e2521520f36f0459d8caa3b63b6b11605ab097c2ec6a5fb61392666f5cd0ad`)
records package/index/stored/decoded hashes and exact rows. It scanned 11,460
explicit `00B1B104` entries in PatchData, 7,030 in Spore_Game and 4,738 in
Spore_EP1_Data, with no decode/framing errors among substring candidates. Each
package had one exact named/count-bounded matching PROP, instance
`0000000008EA246B`. The original compatibility parser supplied incorrect group
labels; the canonical native-index correction below establishes scope zero. In each, `4F6A84FD` type `0009` is integer zero and `A0304059`
type `000D` is binary32 positive zero. Their decoded payload SHA-256 values are
respectively `9cbbf3cb4563a48cdfb72c5b80cd89cf2db581254a01adfcb504ff5de6d8a16b`,
`bec8b897d6defc4bc8360f967fbef313b7d705abfa70f2df857e51a65d23066e` and
`bb2f1d3a25da3e5a6ad954fe882d70d6a081485cae30df1676b1f2c4d4802a6b`.
Scan and index audit script SHA-256 values are
`c93a4b09aa148f61420db9f7e78b7fd8405cce16ad48978512369b140f478db8` and
`b8d6a8e2223a8b54cd1cbf7b663de3acebd50a33cb8430513ce8a4ede25f33ec`.
The candidate zero values support the no-smoothing/no-correction fixture if an
explicitly admitted matching PROP was loaded and remained unchanged. The
actual mount order/winner, other providers, later mutation, successful creation
path and live 0473 state are not proved by the scan. A synthetic getter test would not resolve these missing
runtime admissions. Endpoint hashes do not establish continuous stability.

A fresh canonical native-index audit corrects the compatibility group labels in
`result-v2.json`: `0001FFFF` and `00010000` were metadata words, not native
scopes. `target/creature-skin-selector-native-index-20261109/result.json`
(SHA-256 `54784379d8feb4915f44cf8fccad1c9859501f20bb1fa7e237aa1fc7d60b66f8`,
script SHA-256 `51c9b8c625bad8806dffde22e630b174905ad0ef058150b7c4280c0f8b143f94`)
scanned all 82 discovered installed package indexes, 158,720 records with no
index errors, and found exactly three copies of native key
`08EA246B:00B1B104:00000000`: PatchData, Spore_Game and EP1_Data. Their
retained payload hashes match the three decoded PROP candidates above. Index
hashes before/after match within the audit; this does not prove continuous
file stability or that the discovered packages are the runtime mount set.

On the inspected manager-construction path, `7E8BC6..7E8C13` constructs an
`8E1AB0` manager and publishes it both through `8DD7D0` and `67D9E0`;
`67D7F0` returns the latter slot. With primary vtable `01437740` still active,
`6A96BB`'s virtual +58 resolves to `8DD930`, which scans registered databases
in list order and returns the first whose virtual +34 reports success. Source
registration sorts descending signed priority, retaining earlier equal-priority
registrations. For the inspected `8D9600` database table, +34 is `8D9A50`,
which calls index +28 (`8DAE40`); `8DF530` compares all three key words for
equality. Thus scope zero is an exact key on this database/index path, not a
shown wildcard. The three copies exhaust this fresh installed-file exact-key
census, not necessarily the mounted runtime resolution path: mount list and
priorities, replacement database or provider behavior, successful subsequent
load and later global mutations remain unobserved. Existing manifest evidence
conditionally orders EP1 above PatchData when their documented priorities are
successfully applied, but does not establish registration or an active winner.

The strongest additional source-data bound is a fresh canonical scan of all
30,889 type `00B1B104` entries across the discovered installed packages.
`target/creature-skin-selector-all-props-20261109/result.json` (SHA-256
`e05a791c8aa48c32158a311dc8fe921965bbd6628a2865a1e153945a27adaae6`, script
SHA-256 `2047cc5e3d70b5c976f1b140464e059b27ab5c97486916c01cdc58b1267ae020`)
found exactly three substring candidates and three exact parsed local named
matches, all the `08EA246B` copies above; all decoded successfully. This is a
local-property census, not an inherited/effective value or a complete runtime
source census. A separate limited scan of ASCII tokens using lowercase FNV-1
and literal hex in `Properties.txt`, `ConfigManager.txt`, and `VideoCards.txt`
found neither selector; its receipt
`target/creature-skin-selector-config-audit-20261109/result.json` has SHA-256
`3749015395f5e09dbb0f68da0da8ed93621e402d2da7f6057b4482e6b7d54d1e`.
Other config/user data, transformed names and runtime writes remain outside it.

On the inspected startup route `8DE010` orders registered databases by signed
priority with first-success lookup; manifest-derived EP1 101 and PatchData 2
are conditional on both registrations and overrides, and cannot establish the
actual mounted winner. The global object also has a source mutation route after
initial population: option-selection code can call the attached global object's
virtual +30, which under table `01408818` is `6A10B0`; that routine iterates a
supplied list's 24-byte entries and invokes virtual +14 setters. The all-PROP
and named-config scans narrow the installed candidates but do not eliminate
unobserved user data, replacement providers, aliases, dynamic variants or
later generic setters. There is no observed live selector value.

A meaningful controlled comparison would decode each pinned `08EA246B` PROP
through the original binary property-list reader `6A2A10`/`693EC0`, copy it
through `6A2580`, and compare `6A2110/6A21C0` outputs, with reviewed stream,
allocator, list lifetime, variant/type, FP and failure admissions. A prior
suggestion to treat `6AAC20` itself as the binary decoder was premature:
its inspected path uses `93A520` line reads and `6AA4F0`, while `6A2A10` is
the documented binary entry reader. Exact binary factory/stream plumbing still
needs tracing. Even such a comparison would not prove active mounting, no
subsequent overwrite or live 0473 state.

The retained five full-PROP controlled inputs in
`target/creature-skin-prop-decode-compare-20261109/manifest.json` (SHA-256
`316407d1ace977d82a802cfb474859b60b3d509298af1d39e39651347d90a224`)
comprise three exact decoded installed payloads and two synthetic same-length
value changes. The independent projected manifest (SHA-256
`fdac9a6e243009a297391e9606ccd70592c2b7698b8caed494634c28ac1f6c22`)
retains exactly the two 12-byte PatchData entries at decoded offsets 3876 and
4146 following a synthetic count of two, plus changed-value and missing-int
controls. It deliberately omits the other 436 entries and is not an original
whole-file test. Neither artifact executes native code.

Further static tracing gives concrete stream admissions for a future bounded
host: `6A2A10` calls `93A3F0` for count and each identifier, and `693EC0`
for every 24-byte variant. Scalar types 9 and D reach `571DD0`, which calls
`93A3F0` for one word. Typed readers call the stream object's virtual +10
when mode is not 1, +14 for status, and +30 with destination and exact byte
count; a full read is required before source byte swapping. `693EC0` dispatches
through an absolute switch table at `006945B4`; its noncontiguous exception and
allocation paths must be mapped and checked before executing relocated code.
A trustworthy projection comparison must preserve that dispatch, the real
selected entry bytes, bounded stream behavior and valid vector/variant lifetime.
No reviewed host has yet established those conditions; the full 438-entry
source parse, original copy/getter chain and live property state remain untested.

The reviewed `52B420` merge receipt
`target/creature-skin-smooth-merge-0473-native-20261109/result.json` (SHA-256
`3d39a80887c69d1cdd3b86dcc30acce201ed3caeefe8e94c2261c1c1543fbaa4`)
matched all 272 bytes in four ordered two-record cases, counts 3/3/4/5, with
both exits zero. Source used preallocated eight-entry storage and an equivalent
patched zero constant under `1F80`/`027F`; canaries were not guard pages and
sticky flags were not compared. Factors were explicitly supplied, not derived
from actual neighbor traversal. Frozen-row/same-object/uncorrected initial
records remain conditional; no actual iteration, topology, runtime, palette or
complete smoothing is established.

Read-only Ghidra traces `target/ghidra-creature-entry-v1/smooth-topology-v1.log`
and `smooth-topology-helpers-v1.log` identify the next source topology input.
`509D30` uses geometry +58 triangle-index words in triples, initializes a
32768-slot edge-key table, and for each triangle processes `(index[2],index[0])`,
`(index[0],index[1])`, `(index[1],index[2])` in that order. It writes each
edge's source corner index as the latest +A8 entry for its first endpoint,
uses the unordered endpoint pair as hash key, and inserts a self-link or
splices a later incidence into the +94 linked cycle. A third incidence can
append a +BC exceptional edge; `50A180` subsequently pairs opposite directed
incidences and self-links unmatched incidences. The source helper `50EB00`
uses a 32768-slot open-address table with collision probing; the exact hash,
sentinel, original allocation and saturation semantics require further work.
The PE tables at `013F1890` and `013F1898` begin signed bytes `[1,1,-2]` and
`[2,-1,-1]`. Ghidra pseudocode is supporting evidence; independently decode the
relevant branches and helpers before implementing or running a native host.

A read-only census of the conditional 0473 252 copied index words found 44
referenced vertices, 84 triangles, 126 unordered edges, exactly two incidences
per edge, no self-edge, and opposite directions in every pair. Under successful
source allocation and the retained triangle ordering, this suggests a bounded
first topology comparison requiring only two-incidence cycles, but it does not
prove the geometry object or actual runtime +94/+A8 storage. Hash collisions,
exact source update order, mutation, FP/iteration settings, and topology-builder
lifecycle remain admissions until independently compared.

The reviewed topology v3 receipt
`target/creature-skin-topology-0473-native-v3-20261109/result.json` (SHA-256
`8642f23c9cd36f31456d213d5d3ae05006f4aeb33aaa3f1f61099da1005ef9a2`)
matched all 1,188 bytes of the conditional paired-edge fixture, stream SHA-256
`fbde153d4f9b34cb1e4e4786bd7e08fdf751a3dab16698645f62b91d8a7e8415`.
Original `509E1D..50A0F8`, hash/equality helpers and finite paired fixture ran
with a synthetic frame and preallocated hash, link, start and exceptional-edge
buffers. The earlier allocation/initialization/cleanup and `50A180` were
excluded. The runner independently verified embedded source blocks, branch and
call destinations, absolute operands and the paired-edge preflight; the host
checked its relocated trampoline and previous RW page protection. Both exits
were zero and private desktops closed. Canary checks were not guard pages;
actual runtime identity, smoothing settings, palette and live mesh remain
unestablished, and endpoint hashes do not establish continuous stability.

The callable smoothing draft was removed from the crate and retained under
`target/creature-skin-smoothing-draft-20261109/` after a scratch-range mismatch
was identified; no source smoothing comparison has passed. `52C5F0(0x40)`
allocates 64 eight-byte slots, initializes all from a zero two-word template
via `5313C0`, and sets the vector end to all 64 slots. `52B270` resets only
scratch +14 active count for each vertex; it does not clear the tail. `52B5E0`
passes the entire +0..+4 range to `52C930`, so sorted inactive entries from
previous vertices/passes can affect later output. For 64 entries `52C930`
uses `5338C0` partitioning followed by the >28-element insertion phases, not
only the stable short-array insertion algorithm. Traces are retained in
`target/ghidra-creature-entry-v1/smooth-scratch-sort-v1.log` and
`smooth-sort-helpers-v1.log`; Ghidra pseudocode requires independent instruction
checks, especially tie handling and recursion/heap fallback. A correct bounded
comparison must preserve all 64 slots and their state across every vertex and
pass, original sort order, caller-supplied destination bytes, explicitly
supplied iteration/parameter and the previously admitted inputs. Actual
runtime smoothing configuration remains unobserved.

The reusable paired-topology model was separately compared with the previously
matched v3 native stream; receipt
`target/creature-skin-paired-topology-verify-20261109/result.json` SHA-256
`6a15e75a4505e6b475f300620a0b49a98ed3ea98cf24c3344e398e73767eda81`.
It matched all 1,188 bytes on the admitted 0473 opposite-paired fixture. Its
checked rejections, source allocation/repair and runtime boundaries remain as
above; this does not validate smoothing.

The reviewed 64-slot sort receipt
`target/creature-skin-sort64-0473-native-20261109/result.json` SHA-256
`d62c64ac1bdb14ddfca4daed5acd01c3f7f5170ab8c550f8a3f5460c256a6931`
matched six controlled scratch cases, 3,072 bytes SHA-256
`7e57806eb6b9553eb78ec058c476c2e0b57351aaba01cb7d30fa646c1a5914e8`.
The checked Rust 64-slot model separately matched that retained native stream;
`target/creature-skin-sort64-rust-verify-20261109/result.json` SHA-256
`533bdc32cf8cf8b68ce25351f6b49542ec6a989ae07d91d89cc158b81a9a52b3`.
The original comparison covered supplied finite cases, including tagged ties
and synthetic stale-tail contents, under `1F80`/`027F`; unsupported heap
fallback was trapped. It did not establish a live scratch or smoothing pass.
The persistent-64 smoothing model is explicitly unverified until its prepared
complete original-instruction comparison is independently reviewed and run;
actual smoothing count, parameter, corrections, palette and runtime identity
remain unobserved.

### Controlled 52B270 smoothing and optional inspection records

The corrected v3 original-instruction comparison matched all 2,640 bytes for the conditional frozen-row 0473 fixture and explicitly supplied 0, 1, and 2 passes at alpha `0x3f000000` (0.5). The model and host exited zero; `target/creature-skin-smoothing-0473-native-v3-20261109/result.json` SHA-256 `c3fd128b4049e4f7ee40ca8c0a8f598cfd68f89562651e5f777846f0b0d3c1f8`, each output SHA-256 `e0ba1b69c678bf41b3a8eb0e4ea8ae6578b86df99dce93dc0a9d90b0d5fe9a22`. The prior v2 attempt failed in its harness ESI check after its own inter-pass copy; the independent diagnostic observed both pass checkpoints and check 38, and v3 refreshed the ESI/EDI baseline after that copy. Both records remain retained.

The comparison admits the preallocated topology, source-row records, normals, zero destination and persistent zero-initialized 64-slot scratch, explicit pass counts and factor, MXCSR 1F80 and x87 control 027F. The pinned native heap fallback is an INT3 trap whose containment assumes no debugger or exception handler intercepts it; no native allocator, active configuration, runtime creature identity, palette identity or final mesh is established. The source caller's independently traced factor is `0x3f333333`; this 0.5 test does not establish its result. The inspection exporter can explicitly opt into one of these matched .5 pass counts; it preserves raw index and weight words and carries the same limitations.

The separate caller-factor comparison used `0x3f333333` and explicitly supplied 0, 1, and 2 passes. The original closure and Rust model matched all 2,640 bytes with both processes exiting zero. Receipt `target/creature-skin-smoothing-caller-alpha-0473-native-20261109/result.json` SHA-256 `fce30d8553893daa8e4c7592722ca824edf821948cae81df9e803caa063de9e6`; both output streams SHA-256 `f634488e3bbdcd20f942dadb2c6a40966d060224851896ce794e7f3cc7b866be`. The same frozen-row, topology, preallocation, FP and trap admissions above apply; the selected live iteration count and palette remain unobserved. A separate inspection-export flag admits these pinned records for an explicit pass count and does not identify a live creature or final skin.

### Controlled corner-attribute projection

Original `46E776..46E7DE` with guarded append stubs matched 132 explicitly synthetic corner mappings over the three caller-alpha 0473 record sets: native receipt `target/creature-skin-corner-attributes-0473-prep-v2-20261109/result.json` SHA-256 `b8ef734dfc089b6ca115b31ab97c829a0fb094d8bc77531dfa600a0e211b11ed`, output SHA-256 `4c3e251f216551e5d9cce8ca1de3dba57a9ce62f7f46f8f8c7d16828bde23753` (3,168 bytes). The separately built reusable Rust projection matched the retained native stream; receipt `target/creature-skin-corner-attributes-rust-verify-20261109/result.json` SHA-256 `e50f27d8b25684822ea642316d06239a6d85c64dcadc020ef0ddd9a99ea901d1`. The bounded comparison uses x87 control 027F and synthetic preallocated append stubs, not native allocator/grow paths. Mapping `(corner ordinal * 17) % 44` is intentionally supplied for inspection; it is not an observed source subobject or corner triple. The optional exporter checks both source records and native output and retains raw offset and weight words. Palette identity, live pass selection, owner/lifetime, actual corner mapping and complete mesh remain unobserved.

### Conditional corner-triple production

`4CC540` calls `5098B0` on intermediate +34. For each supplied subobject face ordinal and its three lanes, `509B22..509C57` reads the global position, secondary and tertiary corner streams; a per-position cache appends the first triple for that position and reuses its ordinal on later appearances, even when their other indices differ. The original slice matched the controlled model over 252 conditional 0473 position indices, a synthetic single-subobject face list 0..83, and synthetic secondary/tertiary streams; receipt `target/creature-skin-triple-producer-0473-prep-20261109/result.json` SHA-256 `58492e37383d40d48737d006b6a3d1f9fd5d38cfe814e17c6f6e7174a54b49d9`, native output SHA-256 `f3f78ca38d7ae8e5918e5dcbd8b2c77be619f0ea5d7e0322fe8b5f9c2fff8cfe`. Its append stub uses zeroed preallocated storage, without modeling native allocation or the preceding source subobject construction.

The checked reusable Rust builder matched all 1,540 native producer bytes; composing its unique triples with the earlier caller-alpha pass-1 record projection also matched 44 corresponding entries from the independently compared native corner-attribute stream. Receipt `target/creature-skin-corner-triples-rust-verify-20261109/result.json` SHA-256 `e16381798253dce2d7afa85f4b93c67dba7aa704f0572a9fae55da1df6ef07e3`. Both components remain conditional on the synthetic face membership and secondary/tertiary streams; actual +E4 subobjects, their face lists and source attributes, active pass selection, stable owner lifecycle, and palette identity remain unobserved.

### Subobject and corner-index preparation boundary

On the inspected `4CC540` path, geometry is intermediate +34. The `53F796..53F88A` finalization slice resizes geometry +6C to its +58 index count and invokes `541380` to copy the +58 words; the retained conditional 0473 copy and helper were separately compared as documented above. Equality of those vectors at `5098B0` additionally requires successful lifecycle and no intervening mutation. `4CC540` tests +94 adjacency, runs `509D30` when empty, clears existing +E4 entries through `50A400`, and appends subobjects for rows with +08 mask 1 clear. On a conditional geometry path `50A990` then traverses faces and assigns subobject membership through paired +94 adjacency, face state +110 and the orientation, edge-continuity and UV-cache helpers `50B020`, `50B210`, `50A560`, `50A840`, and `50A570`. `50A840` appends the selected face ordinal to the subobject and fills geometry +80 corner UV indices using a per-position cache; `50A570` may project and append new UV values. `50B3B0` later packs subobjects and uses their +08 face lists with the +80 corner indices to transform the geometry +30 UV-coordinate array. These relationships are conditional on selected branches, valid vectors, helper behavior, allocation and no mutation; the retained source positions and initial +6C copy alone do not determine actual +E4 face lists or +80 indices. Supporting noanalysis Ghidra logs: `target/ghidra-creature-entry-v1/corner-producer-20261109.log`, `corner-setup-20261109.log`, `corner-uv-helpers-20261109.log`. The decompiler output is supporting evidence; exact operation and FP order need source instruction checks.

The original complete `50B210` edge-continuity function matched 252 directed retained paired edges and two separately altered endpoint controls. Its unchanged +6C input is conditional on the prior successful +58 copy and no intervening mutation; the two controls intentionally mutate one endpoint each. Native receipt `target/creature-skin-edge-continuity-0473-prep-20261109/result.json` SHA-256 `7d8d527d8dffbad288847a6d72363425f192834fdf70a05127c1b9121e3b3da0`, output SHA-256 `3cd5adf4595c0d8d7e6762703a594eaec16386aa207a433109f737234743a64b` (1,016 bytes). A reusable checked Rust predicate and example matched the retained stream; receipt `target/creature-skin-edge-continuity-rust-verify-20261109/result.json` SHA-256 `56a265b4d989491510eefc189d993dfea506abf9e1ba752867a6c700f1e3e81e`. This compares a single input to conditional grouping, not actual UV seams, group membership, live configuration or identity.

`50A990` conditionally traverses paired face adjacency and tests both `50B020` orientation and `50B210` reverse-edge continuity before extending a subobject. `50A560` is a constant-false function in the inspected PE; `50A840` mutates subobject face lists, UV-index cache and per-face state, and may enter projection helper `50A570`. Consequently the matched edge predicate alone does not recover the flood or UV values. The narrow next prepared comparison isolates `50B020` and its original vector-subtract, cross-product, absolute-value and largest-component helper closure on 84 conditional 0473 faces and three synthetic tie/sign/zero controls; no native result is asserted until the host has passed review and execution. Ties in `4FE4C0` fall toward the later component and the output's low bit records a strictly negative selected normal component. Retained source/noanalysis logs are `target/ghidra-creature-entry-v1/face-group-helpers-20261109.log` and `corner-setup-20261109.log`; branch, operation, lifetime, nonfinite, allocation and FP admissions remain material.

The original `50B020` orientation closure matched 84 conditional 0473 faces and three explicitly synthetic tie, negative and signed-zero controls. Native receipt `target/creature-skin-face-orientation-0473-prep-20261109/result.json` SHA-256 `634b2c1d89aee5bd2ee003687e5fabb7a7801a9e225c5d001587406fad1962d0`, 348-byte result SHA-256 `60538be1f2fbcb4d25bbcbcf3deeb5a22acd26d6a5dd46475236e5fb85a31a8c`. Its bounded Rust model matched the retained 87 result words; verification receipt `target/creature-skin-face-orientation-rust-verify-20261109/result.json` SHA-256 `8d5d72f07e5dbad1d13fb691a3478c650d693e8f33035555a3137ec3033c2acb`. This comparison admits finite binary32 intermediate operations, MXCSR 1F80, x87 027F and pinned conditional averaged positions; it does not establish actual grouping, UVs or runtime object identity.

The original full `50A840` cache-hit branch matched 252 conditional face/lane calls using the retained 0473 position indices and explicitly synthetic group 55 and cache values `1000+position`. Each snapshot contains the appended face, all 252 UV-index entries and all 84 face-state words. Native receipt `target/creature-skin-group-cachehit-0473-prep-20261109/result.json` SHA-256 `90a347cad43f23294ea670f5ed6e1d707eef4aa941bc047e7137fa7d16701f33`; 339,696-byte output SHA-256 `71a133e420d15afa90b3231c7275e816055a856f7d9ec606a7d7f47dac6786fc`. The checked Rust all-hit projection matched every snapshot, with receipt `target/creature-skin-group-cachehit-rust-verify-20261109/result.json` SHA-256 `2e48b71b2ed0dc22cf9593ea3c05a0033aa32c0c82603d158acbb7f02040458c`. Its bounded check rejects cache misses before mutation; original `50A840` has already appended a face before calling `50A570` on a miss, so the projection does not model that failure ordering or allocator behavior. The miss helper can append a projected UV pair and metadata, update the position cache and group bounds; actual cache contents, UVs, grouping and owner identity remain unobserved.

The `50A840` cache-miss helper `50A570` is a separate boundary. With ordinal `-1`, it selects a position from geometry +58/+08, calls `50A7B0` with group +04, appends the projected two-word pair to geometry +30 and `(group id, corner ordinal)` to +124, derives the new pair index from +30 begin/end and writes it to +80[corner]. It then writes `(group id, index)` to the per-position +1B8 cache and calls `50A690` to update group +58/+5C/+60/+64 bounds. With a supplied ordinal other than -1, the source instead rewrites +124 metadata for that ordinal and skips the append and +80 assignment before the common cache/bounds path. A bounded comparison needs valid preallocated append receivers or explicitly stubbed allocator/grow behavior, valid existing state, finite values and scoped FP controls. It cannot infer active group, runtime row identity or actual generated UVs from the retained source rows alone.

The original `50A7B0` bit-level projection matched 376 supplied position/group pairs: 44 conditional 0473 averaged positions plus three synthetic raw-bit vectors, each with group values 0..6 and FFFFFFFF. Signed zeros, infinities and quiet-NaN payloads are bit-selection controls; this function performs raw MOVSS/XORPS and does not establish meaningful UV coordinates for nonfinite values. Native receipt `target/creature-skin-uv-select-0473-prep-20261109/result.json` SHA-256 `a9b9f963b129d5ceb9cd8e43ec22d2b3acf3fea94db87668e488ceb3b8a030c6`, 3,008-byte output SHA-256 `e579f8f0e26158c3f908ac4a4064d0ef6e9c447127ea53073391ee15df702cdb`. A checked Rust bit projection matched the same stream; receipt `target/creature-skin-uv-select-rust-verify-20261109/result.json` SHA-256 `746840fdba02bfc0438ae7609d90991aa3ad1d2dc021a362b3f255576ec4a1b8`. The group inputs are explicit and synthetic; no actual grouping, cache ownership, allocation or runtime identity is inferred.

A guarded source host compared 266 controlled `50A570` calls: 44 conditional averaged positions crossed with six explicitly supplied axis/sign groups, and two existing-ordinal controls, one with signed-zero bounds ties. Original `50A570`, `50A7B0` and `50A690` ran with bounded preallocated vector append stubs and x87 027F/MXCSR 1F80. Native receipt `target/creature-skin-uv-miss-0473-prep-20261109/result.json` SHA-256 `3fed994c8f24ee5d490d9ffa54f743055e64235b0dd3aca88757e55388c849da`; 19,152-byte output SHA-256 `9fac3e7985e893ff0bd7b26308ced35cb890484b50bae085d8c8f2804495ee8d`. The checked Rust projection matched the complete retained output; receipt `target/creature-skin-uv-miss-rust-verify-20261109/result.json` SHA-256 `3502289ebc579075ebd31f2cebd5715887d9986a79741adeb23aeedb351c507f`. It requires finite inputs, consistent vector lengths and caller-preallocated capacity, and rejects before mutation on invalid state. The comparison does not establish native allocator/growth/failure paths, the preceding face append, actual group or UV state, or runtime identity.

The original `50AC9A..50AD9F` seed-neighbor loop matched 84 conditional retained seeds and two synthetic controls for prior-visited state and an altered endpoint. Its supplied orientation values and adjacency came from separate matched native comparisons; group 0 and starting states were explicit. Native receipt `target/creature-skin-group-seed-neighbors-0473-prep-20261109/result.json` SHA-256 `5ec267d775e80b6a38d143237b736fe2c33bf1a5435e336a9ac1a8ddfe3abb4d`; 31,992-byte stream SHA-256 `86abdd55b1bd389429c397e89fe0e64ce558eda4e2d4cb3ffc0e91880455e3f5`. A checked Rust first-expansion model matched the full retained stream; receipt `target/creature-skin-group-seed-neighbors-rust-verify-20261109/result.json` SHA-256 `ee5e3d0670d6eb4b835d1660c8801dcbb2328163aad6e52b8b66722bba0c4db2`. Its all-input prevalidation differs from native partial failure and it does not model the subsequent queue consumer, UV/cache updates or actual group identity.

A one-group composition attempt using original grouping and UV helpers failed before output with host exit `C0000005`; receipt `target/creature-skin-single-group-0473-prep-20261109/result.json` SHA-256 `db864d85302d86fbe5ded7103cbca9585ef24ef5df0a11110536f330305b426d`. No comparison succeeded. Independent static audits found that this host mapped the source sign mask page but did not map the `50A840` signed-byte tables at `013F1890/013F1898`; this is a likely explanation, not a proven fault address. A separate v2 host was prepared and independently reviewed with both fixed pages and all three high absolute/indexed source operands checked. No v2 outcome is assumed until its separately retained execution.

The separately repaired v2 one-group host matched its conditional model: receipt `target/creature-skin-single-group-0473-prep-v2-20261109/result.json` SHA-256 `7030914a503bf9ad6e31718558d99a9f373322d5bb8d8ef3862239aa56b13333`; native output SHA-256 `6b82e7ecb275f3c048a0f68b796225f787b17935a099be2957deef0226308992` (6,788 bytes). It uses retained conditional 0473 positions/topology and native-matched orientation values with one explicitly synthetic group 0, seed 0 and empty cache; checked preallocated vector stubs exclude allocator/grow/failure behavior. The reusable checked Rust single-group composition matched that entire native stream; receipt `target/creature-skin-single-group-rust-verify-20261109/result.json` SHA-256 `20f7627bee9b5ed88f1d4e6df13a6ccddec6559765bafd18270514c3508b7a09`. Its prevalidation and owned transactional result exclude partial-failure ordering and intermediate observation; it requires valid initialized cache indices. This is not the outer group-generation loop, observed group selection, live identity or palette binding. The earlier v1 access-violation receipt remains preserved, with no captured fault address.

The isolated original `50AB3C..50AB9A` seed selector matched twelve controlled cases: eight state snapshots from the conditional multi-group preflight and four synthetic direct/scan/exhaustion/unsigned controls. Native receipt `target/creature-skin-group-selector-0473-prep-20261109/result.json` SHA-256 `87a8627101bdeb3bc749331c4b12b33b41c2c1d6da2338eeb253f43b0ea4218a`; output SHA-256 `e0586202ff13997413a470d068f9505047089dd7da42d86ce2fca4f4e7de2f97` (48 bytes). The source external exit jump and success continuation were replaced by returns; each case admitted 84 states, an in-range candidate, a synthetic frame and a supplied group id. Preflight suggested eight groups for the frozen conditional fixture starting at group 4, but its generated states are model output, and this selector comparison does not validate the outer group loop, allocations, ownership, actual UVs or live identity.

The checked Rust selector matched the retained 48-byte native output for all twelve rows; verification receipt `target/creature-skin-group-selector-rust-verify-20261109/result.json` SHA-256 `94753131fb25897304ef9e1c057c32f0bd1f99ea44b570bc3b716272e615c21b`. Its API rejects an out-of-range candidate and returns `None` when the original scan reaches the admitted face count; it does not model invalid native dereferences or generate the outer-loop state.

A checked Rust multi-group composition carries cache, UV, metadata, corner and face-state buffers across groups, with an explicitly supplied initial group count, empty storage, reciprocal in-range links, supplied orientations, finite coordinates and preallocated successful writes. Against a separately retained Python model on the conditional 84-face fixture, all 3,696 bytes matched, including eight groups, their seeds/axes/faces/bounds and final shared arrays; receipt `target/creature-skin-multigroup-rust-verify-20261109/result.json` SHA-256 `1e593a044be9842d734576b78d9373cb832477baf18741ea460e108cc33a7728`, model/output SHA-256 `3350ca9dabb564c6cbab49153f4d6eb8073cd32d963319478f00ffcf0bc8519f`. This is model-to-model agreement; original selector and one-group slices have separate native comparisons, but the whole outer loop, allocator/lifetime behavior, source actual group ownership and live identity are not validated. Starting count 4 depends on admitted frozen row flags and preceding object lifecycle.

The inspection exporter has a separate `--include-model-only-uv-groups` option. It verifies the matched Rust/model receipt and streams, four pinned fixture inputs and exact position/index agreement with the existing artifact, then checks face coverage, finite UV words, cache/metadata and corner ownership. The section is explicitly model-only and retains the initial-group/lifecycle, supplied orientation, allocation and live-identity exclusions. Default output remains unchanged. A conditional inspection artifact is retained at `target/creature-skin-multigroup-export-draft-20261109/conditional-uv-groups.json` SHA-256 `5f6b4126a3ccdf28f8dca6adcd067dee8b0a5f7cc2144637b0dbc9221753118c`; it is not evidence of actual outer grouping or UV ownership. Exporter tests passed with the fixture (15/15) and with fixture files absent (8 passed, 7 skipped).

The separately reviewed original outer-instruction host subsequently matched the full 3,696-byte conditional 84-face result, including all eight generated groups and final shared arrays. Original-instruction receipt `target/creature-skin-outer-group-0473-prep-20261109/result.json` SHA-256 `4492803315018f5819322ba23f523ee1d6f11c87e0637748340a49e5b99a8775`. A direct byte-for-byte comparison to the existing checked Rust output also matched; receipt `target/creature-skin-multigroup-native-rust-verify-20261109/result.json` SHA-256 `c9dcc5cfb12dccf5627335fcfedadb77faa7764c977ee828aedcd58868a26eb8`, both streams SHA-256 `3350ca9dabb564c6cbab49153f4d6eb8073cd32d963319478f00ffcf0bc8519f`. This covers original 50AB28–50AFAE and original UV/edge helpers under supplied source-matched orientation, four synthetic existing groups, zero state and synthetic sentinel cache with a written-before-read preflight. Checked preallocated reset/queue/allocation/deep-clone/append stubs substitute native ownership operations. Native allocator, constructor/copy/destructor effects, post-loop cleanup, growth/failure, actual group ownership and live identity remain unverified; guard words are not guard pages. The prior model-only inspection artifact is unchanged and its conservative label remains appropriate.

The opt-in UV/group inspection export now also requires the pinned original outer-host receipt and stream, and the direct native-versus-Rust receipt, checking the latter's native/Rust receipt hashes and stream hashes. Its conditional section explicitly states the synthetic preexisting groups, zero states, sentinel cache, supplied orientation and preallocated reset/queue/allocation/clone/append stubs; it does not establish actual group/UV ownership or live identity. Default JSON remains unchanged. Updated artifact `target/creature-skin-multigroup-export-draft-20261109/conditional-uv-groups-native-evidence.json` SHA-256 `83b558c32c1411f7835fbb334e5c7dd912fd3e6d7e90947917f42253817405b7`; prior artifact remains retained. All 15 exporter tests passed with fixture evidence; a clean-target test passed with seven fixture-dependent cases skipped, and missing/tampered new evidence is rejected.

`4CC540` conditionally calls `50B3B0` after grouping, passing the installed PE float at `013F0594` (`3BC49BA6`, approximately 0.006); active receiver/lifetime and value stability remain admissions. The original first per-group slice `50B419..50B53E` expands +58/+5C/+60/+64 bounds by that margin, computes width and height with `subss`, accumulates their product by `mulss` then `addss`, and calls `50C570` if width is strictly greater than height. `50C570` toggles bit 0 at +68 and swaps the U/V bounds, using x87 load/store for two writes. This precedes x87 square root, sorting and packing, and eventually uses face lists and +80 indices to identify +30 UVs, optionally rotates, translates and scales them. Consequently the generated UV projections in the present conditional artifact are not the final packed UVs; the four earlier source-derived subobjects, their bounds, full atlas packing inputs and runtime texture/material binding remain unresolved. A separate bounded first-stage host is prepared under `target/creature-skin-atlas-bounds-0473-prep-20261109` for the eight conditional generated bounds and three synthetic controls, with explicitly supplied margin and running area. No native result is asserted until independent review and execution.

The reviewed original 50B419–50B53E plus complete 50C570 host matched all eleven cases: eight bounds from the conditional generated-group stream with explicitly supplied initial area zero and installed caller margin `3BC49BA6`, plus three synthetic rotate/no-rotate/equal controls. Receipt `target/creature-skin-atlas-bounds-0473-prep-20261109/result.json` SHA-256 `0f3dc8289c2c561a88668aee3c76cf0bb6c5283a79889741eca4b354dc479357`, 264-byte output SHA-256 `6b48bbf0e0d730f619dc18821b425014667daf0a0611f27539c862dd48db7b68`. A reusable checked Rust projection matched those native bytes; receipt `target/creature-skin-atlas-bounds-rust-verify-20261109/result.json` SHA-256 `21c571ac34c0fc199d3898ccb2d76d1d5a0e4064867c3edb83f6146b6b208593`, focused tests 2/2. The Rust API admits finite normal or zero inputs and intermediates, explicit MXCSR 1F80 and copied x87 values in that domain; it rejects subnormals/overflow/nonfinite values and returns owned output, so it does not model native partial mutation on rejected inputs. The omitted four preexisting groups and their area, actual full packing, later UV transforms, runtime texture binding and live identity are not established.

The reviewed complete original `50C600` shelf-append helper matched eleven controlled cases: eight conditional expanded/rotated group bounds with explicitly supplied shelf states, widths and prior pointer counts, plus three synthetic height-max and signed-zero tie controls. Its original `421F30` vector append was replaced by a checked preallocated pointer-append stub. Native receipt `target/creature-skin-shelf-append-0473-prep-20261109/result.json` SHA-256 `c7dbee507f605cc1aff968ff180de7b38cc0f33af338fe0f5210841453d6e3f7`; 176-byte output SHA-256 `386324e68f39a63bba8039a461c5d3db1589eec28eddf41744e7df7d3581b4ae`. A checked reusable Rust shelf projection matched the retained stream; receipt `target/creature-skin-shelf-append-rust-verify-20261109/result.json` SHA-256 `c3e5c0c434f002a8c5d2c2e0b8312322076cf33c1358b9d5e7fc6a8f13d80aa6`, focused tests 2/2. Rust admits finite normal or zero inputs/intermediates with MXCSR 1F80 and preallocated capacity; its checked errors are transactional, whereas native appends before arithmetic. Sorting, actual shelf selection, unknown four earlier groups/area, native allocation and full packing, texture binding and live identity remain open.

The post-bounds `50EE30` call sorts a copied group-pointer array. For admitted counts 8 and 12, the source `50FAD0` entry's small-range branch avoids partitioning, and positive computed depth avoids its heap fallback; `50FB90` then insertion-sorts by group height (+64 minus +60) descending for the caller's mode zero. For finite comparable heights, it moves only on a strict greater-than comparison, preserving input order on ties. The 12-entry source list would include four earlier groups whose actual bounds and lifecycle remain unobserved. Source evidence includes the noanalysis log `target/ghidra-creature-entry-v1/atlas-sort-20261109.log` and checked instruction audit. A bounded 8/12-entry original host is prepared with four explicitly synthetic predecessor groups and two equal/signed-zero controls; unreachable partition/recursion/heap/large-tail calls are trapped. No native result is asserted until independent review and execution. `50B6B9` and `50B788` choose a fit only when candidate width plus current shelf width is strictly less than `sqrt(total area) * 1.1`; equal values take the alternate path. Total area, sorted actual group list, row lifecycle, final UV transforms and texture binding remain unresolved.

The independently reviewed original 50EE30/50FAD0/50FB90 host matched four fixed n=8/n=12 mode-zero cases, including native-matched conditional bounds, four synthetic predecessor groups, equal heights and signed-zero controls. Native receipt `target/creature-skin-atlas-sort-0473-prep-20261109/result.json` SHA-256 `3cf0063b51449a543c31da90543804b1e9b6d168b4f89b9007ee781c2949156f`, 192-byte stream SHA-256 `e342a1dc12346b77949bbaa6569b8235d69def6faa25523f76dec7a76230d220`. Checked Rust small-range sorting matched the retained native stream; receipt `target/creature-skin-atlas-sort-rust-verify-20261109/result.json` SHA-256 `3d8bbded440ff58a563111d5b811c6ce097ce3e99a904f13385f076f43fd3ec0`; focused tests 2/2. The Rust model admits 2–28 finite normal-or-zero groups and intermediate heights under MXCSR 1F80, preserves strict descending stable-tie behavior, and rejects n=1 and >28; the native comparison covers only the pinned n=8/12 cases. Actual four prior groups, runtime order and source heap/partition branches are unverified.

The reviewed original `50B5EA..50B83A` shelf-selection loop with original `50C600` matched two controlled corpora: eight explicitly supplied sorted conditional generated-group bounds and supplied square-root area, and a three-group synthetic exact-threshold control. Native receipt `target/creature-skin-shelf-select-0473-prep-20261109/result.json` SHA-256 `bf8c775b3d2430a8c2589c25d915ee496f62dce0278b4ef4b93cdacc42e8b56c`; 1,544-byte output SHA-256 `a95f4bdf0b64fb00bbc6d3049585bf06df5bceb4e48fdd9417faec30f2ddcb5a`. Checked Rust shelf selection matched the full retained stream, receipt `target/creature-skin-shelf-select-rust-verify-20261109/result.json` SHA-256 `f9124645bd8f3a14feef4d181ccc7c9dd671ced96654034c7a0c8cb781f71347`; focused tests 2/2. The host supplies sort order, square root and an empty initial shelf; 12-slot preallocated constructor/clone/append/destructor stubs replace native allocation and lifecycle. Fit-in-empty-row is explicitly preflighted to guarantee progress and equality takes the new-row/scan branch. Rust accepts 1–12 groups whose positive widths strictly fit an empty row and finite normal-or-zero intermediates under MXCSR 1F80, and returns owned output; its failure behavior is not native partial mutation. The four earlier groups, actual sort and sqrt, allocation, full packing, final UVs and live identity remain unverified.

The per-UV source path at `50C198..50C2B2` reads the group's rotation bit and adds bounds +5C and +58, then iterates supplied UV ordinals: on rotation it maps `(u,v)` to `(bounds_sum-v,u)`, calls original `50E880` to add a supplied 2D offset, and calls original `50D910` to multiply by a supplied scalar. Operations and writes are binary32 in that order, with an x87 copy of the old U on rotation. A bounded host is prepared for the 85 conditional UVs and previously matched expanded group bounds/flags, with explicitly synthetic per-group offsets and scale and three controls. Each host invocation supplies one bounded UV ordinal and uses finite normal-or-zero values/intermediates. Source shelf pairing and actual offsets/scale, earlier groups, texture/material binding, allocation and live identity remain open; no result is asserted before independent review and execution.

The independently reviewed original per-group offset host `50BF38..50BFFC` matched eight conditional group/shelf cases and three parity/zero controls. Native receipt `target/creature-skin-atlas-offset-0473-prep-20261109/result.json` SHA-256 `9a3860258c6ad764d5535ef7a32d298ac51b927c911a7f3f84ad77791f02fccc`, 264-byte output SHA-256 `fbaeff3b4ece1f30344ae01ba82b3780917e07fe332d8ddd4234c87dc0f3402b`. Checked Rust offset projection matched all eleven retained cases, receipt `target/creature-skin-atlas-offset-rust-verify-20261109/result.json` SHA-256 `aa95e19a08b0736305dd955e4820e9f7ca30090f68089a4c21ad15303d9ae15a`, focused tests 2/2. The source arithmetic is reproduced under finite normal-or-zero inputs/intermediates and MXCSR 1F80; row_x is model-derived and row_y/shelf shift are synthetic. Pairing, actual atlas placement/scale, four earlier groups, texture and live identity remain unverified.

A pinned input audit established that all 85 conditional UVs map uniquely to their matching group bounds and offset case through shelf and sort records. All 85 previously native-matched UV-transform rows used different synthetic offsets; their output cannot stand in for the joined case. A separate host has been prepared with the matched group offsets and an explicitly synthetic 0.5 scale; actual row_y/shift and four prior groups remain unknown. It is independently reviewed, but no result is asserted until execution.

The separately reviewed original per-UV transform host using the eight matched per-group offset outputs then matched all 85 joined UV cases plus three controls. Native receipt `target/creature-skin-uv-offset-join-0473-prep-20261109/result.json` SHA-256 `a0744caa2abc8804a5876dc4edc908c24a79324aed62e1df3101e0cf8266e191`, 704-byte output SHA-256 `35de09a8231fd837c04dc54f6d8c3236efe12fec4a17a2a63228855c251723e6`. A target-only direct composition ran the pinned Rust offset example, joined its output to all 85 UVs through the supplied matched shelf/sort/group mapping, ran the pinned Rust UV example, and matched all 704 native bytes; receipt `target/creature-skin-uv-offset-composed-rust-verify-20261109/result.json` SHA-256 `582a43bf1de238e865679ed0afc8821837833f584561598ad5d3dca6a02039a9`. This is conditional arithmetic/composition evidence: row_x is model-derived, row_y/shelf shift and 0.5 scale are synthetic, four earlier groups and actual atlas placement/scale/texture binding are unknown, and no live identity or final mesh is established.

The inspection exporter now has a separate `--include-model-only-transformed-uvs` option. It also includes the conditional group section, checks the pinned native joined and composed Rust receipts/streams/cases and atlas/shelf/sort/offset receipts, reconstructs a unique group-to-offset map, and verifies all 85 original UVs, matched bounds/flags, supplied offsets/scale and 252 corner indices before attaching transformed numeric UV pairs and raw binary32 words. The default JSON was checked byte-identical to the prior default. A conditional artifact is retained at `target/creature-skin-uv-offset-composed-rust-verify-20261109/conditional-transformed-uvs-v2.json` SHA-256 `5448084adc8d22c867f706e1a2b88b743fbe92cc633bc3c4822864804ec91be6`; it explicitly marks model-derived row_x, synthetic row_y/shift and scale 0.5, omitted prior groups, unverified actual atlas/texture and live identity. Exporter tests passed with retained fixtures (17/17) and clean-target behavior (9 passed, 8 fixture skips). This is inspection data, not final UV or texture binding.

The original `50B83B..50BE09` shelf pairing/reversal and `50BE3A..50BE87` scale calculation matched three controlled cases: four conditional shelves, their first three as an odd-row control, and two synthetic equal-width shelves. Native receipt `target/creature-skin-atlas-pair-scale-0473-prep-20261109/result.json` SHA-256 `b4eedac21de34f071026bd8293f4dd346d141ece708592ace65dab63dc3770ed`, 2,340-byte output SHA-256 `571a4e967cc2417165010ce9367d8f3fe47f51ba55824a629b0a7f69eb3b1ea3`. A checked Rust model matched those native bytes; receipt `target/creature-skin-atlas-pair-scale-rust-verify-20261109/result.json` SHA-256 `9f43a1eba74a20966d1c6db4017302c2944c023293844ca652181bf70db6a926`. Focused tests pass 2/2 after correcting one initial synthetic assertion: two half-height rows with zero shift have combined height 1.0, not 0.5. Finite normal-or-zero arithmetic, positive group/shelf dimensions, bounded nonempty row arrays and MXCSR 1F80 are admitted. The fixture's input shelves depend on supplied order and sqrt and omit four earlier groups; it does not establish the actual live atlas, native allocation/lifecycle, texture binding or identity.

After pairing, `50C4AA..50C4C1` advances row_x by the group's computed width; `50C4C7..50C4E6` advances row_y as `(shelf_height + shelf_shift) + row_y`. The matched pairing fixture reverses even-row group order, so previous offset/UV outputs cannot be reused unchanged. A separate bounded host is prepared for `50BF38..50BFFC` plus these two original advance slices over all eight groups in the native paired row order. It supplies matched conditional shelf heights/shifts, starts coordinates at zero and uses explicit wrapper row boundaries. The required shelf selection still assumes supplied sort and square-root values and omits four earlier groups; this does not establish an actual live atlas. No result is asserted until independent review and execution.

The original `50BF38..50BFFC` offset and original `50C4AA..50C4C1` / `50C4C7..50C4E6` coordinate-advance slices matched all eight groups in the conditional paired-row order, with matched shelf heights/shifts and zero starting coordinates. Native receipt `target/creature-skin-paired-offset-0473-prep-20261109/result.json` SHA-256 `b09f431223a97a1ad13dd519f28796c5e5793d37f3b685587ad9046e077e2bb4`; 192-byte stream SHA-256 `c47c384cca54407be5baf6d8cf9908a1955cec14440b54edab4a2ea04fc120b6`. Checked Rust composition calls the paired-shelf and offset models and advances row coordinates in source order; all eight native rows matched, receipt `target/creature-skin-paired-offset-rust-verify-20261109/result.json` SHA-256 `b5afaab150d739d77097f390b3ec831e736061ac6d8b5ca60e57613c144e0cb7`, focused tests 2/2. The bounded host uses explicit row-boundary wrapper scheduling; source shelf selection still assumes supplied sqrt/order and omits four earlier groups. Actual atlas, native allocator/lifecycle, texture binding and live identity remain unverified.

The four initially preceding subobjects have a source-row construction rather than being inherent synthetic atlas entries. `4CC540` calls `50A400` to clear geometry +E4, then scans receiver +08 object rows (+98/+9C, stride 0x8C). For each row whose +08 word has bit 0 clear, `4CC655..4CC829` computes and appends one 0x94-byte subobject, with sequential +00, +04 zero, +58/+60 zero, +5C/+64 equal to a calculated side length and +68 zero. The retained `0473.creature`, SHA-256 `dbbb55830f7f75eb5c45afd470a8bd0716cb08134e26f62a82674e28cd697137`, contains ten 0x8C rows at payload offset 81 (row-region SHA-256 `24a7822442c1caa64c0f386e5bd645b172af51fb187ad9fc28c6cecfebe10a5c`). Rows 0–5 have mask 1 set; rows 6–9 have +08 words 0200, 0440, 0900, 0800 and therefore give four conditional entries in row order. The corresponding +0C/+10 capability spans [6,9), [9,11), [11,13), [13,15) contain no little-endian `74756F6D` key. Their 15-word capability ID region has SHA-256 `04c6151570a9cb4e5758847729214aafcfc284d0759e59f99acea6d47b7cc328`.

The original `41D9B0` subtracts row +14 from row +20 with three SUBSS instructions. `40ADC0` computes z², y², x², then `(x²+y²)+z²` in SSE binary32, calls the `MSVCR90.dll` `_CIsqrt` thunk at `11E0528` using x87, and makes binary32 stores. Back in `4CC540` it multiplies by row +2C; a matching mouth capability would multiply by PE constant `01485F74=400CCCCD` (2.2), but these four admitted rows do not; then MULSS by `013EC42C=3ECCCCCD` (0.4) and ADDSS `013EF548=3DA3D70A` (0.08). Zero comes from `01485E24=00000000`. These exact input bytes can support a bounded native construction comparison, but only with pinned CRT/module identity and floating-point state, successful preallocated constructor/append, and the explicit admission that the runtime object uses these frozen rows and capabilities without mutation. The isolated row result and subsequent original allocation/clear/group lifecycle are not yet native-compared, and neither live creature identity nor actual atlas placement is established.

The independently reviewed `4CC655..4CC77F` arithmetic and `4CC7C6..4CC81B` object-write host matched all four frozen 0473 rows plus zero-distance and synthetic-mouth controls. Native receipt `target/creature-skin-row-groups-0473-prep-v2-20261109/result.json` SHA-256 `2dd6698805542769411dd1c35c5c623298e469ca6e305ee7713e4dcb7a475d61`; compared 168 bytes SHA-256 `40f3278713bb9b77b9baffbb9400f51b906b8884d1db82de738d14f1eb1983b6`. Before original execution the host verifies the pinned loaded CRT path and `_CIsqrt` export address, with disk/mapped-image continuity admitted; it replaces construction/append with preallocated sentinel-filled storage. The checked `creature_skin_row_group_v1` model supplies sqrt spills explicitly, and its six-case comparator using Rust sqrt matched all retained native words; receipt `target/creature-skin-row-groups-rust-verify-20261109/result.json` SHA-256 `5a40f1edfefdbcb30d2189b0bbf379b742ffc7f54ed3a266fcb7623d1c9b57a2`, focused tests 2/2. This comparison does not prove general CRT equivalence, successful real allocation/lifecycle, frozen runtime identity, full atlas placement or no later mutation.

A separately reviewed original `50B419..50B53E`/`50C570` host matched all 12 conditional atlas-bound rows after supplying the four frozen-row-derived initial groups followed by the eight conditional native-matched generated groups, with source margin `3BC49BA6` and an initially zero area. Receipt `target/creature-skin-atlas-bounds-12group-0473-prep-20261109/result.json` SHA-256 `ec4f90b0c623ec5430798f2329527e581593f648d740c2e1061fb5f580ec3641`; output SHA-256 `73b947945227e84f585bc5d1e415f6424388ad55869737ffa2c779478912200c`. Native host inputs contain precomputed per-row carried area, not runtime chaining. A target-only composition of the existing Rust row-group and atlas-bound examples matched all 288 bytes and checked each subsequent area input against the preceding Rust output; receipt `target/creature-skin-atlas-bounds-12group-rust-verify-20261109/result.json` SHA-256 `33adebf9bcee17415bb8a6c3c25d41630574fcd5ae36ea5bd626a7bce648dd42`. The fixed 11-record example was run twice, with the 12th record repeated for the second invocation. Frozen same-object rows, successful clear/append/lifecycle, conditional generated groups, and sqrt-spill compatibility remain admissions; this does not establish live atlas placement, texture binding or full packing.

The reviewed original `50EE30` sort host matched all four controlled cases using the twelve matched bounds twice plus tie/signed-zero controls. Receipt `target/creature-skin-atlas-sort-12group-0473-prep-20261109/result.json` SHA-256 `3b01b1879bc5a89b0b59f02203fcfa880486c1d4e412635b1a098163ce126394`; the existing Rust sort matched all 192 bytes, receipt `target/creature-skin-atlas-sort-12group-rust-verify-20261109/result.json` SHA-256 `f6f62dd54adffccaf438e53d4e291651555b6f86b41d56d57cfccdb02fc546b3`. Separately, original `50B544..50B55D` with the verified loaded pinned CRT `_CIsqrt` matched the conditional final area `3FB6FB98` and zero/one/four controls, yielding sqrt `3F990AB4`; receipt `target/creature-skin-atlas-sqrt-12group-0473-prep-20261109/result.json` SHA-256 `6e10edfd6bdfd44f26b9cb43c22f73787a82797200cff94fc63b634cf2b83d58`. Disk/mapped-image continuity and x87 027F/MXCSR 1F80 are admitted. A fresh original shelf-selection host now uses those two matched outputs, explicit preallocated stubs and an empty initial shelf; it awaits independent review and has not yet been executed. Actual runtime object identity, native allocation/lifecycle, texture binding and full atlas remain unproven.

The independently reviewed original shelf-selection host accepted the twelve conditional group bounds in original-matched sort order and the original CRT-matched `3F990AB4` square root. Both the 12-group case and synthetic exact-threshold control matched, 1,544 bytes; receipt `target/creature-skin-shelf-select-12group-0473-prep-20261109/result.json` SHA-256 `39c16fa1472ff73ebab01e3f53cb56b6d48a544949beddc239dd893693a7bcaa`. The checked Rust selection model matched the same bytes, receipt `target/creature-skin-shelf-select-12group-rust-verify-20261109/result.json` SHA-256 `3687ebb78ddf85c2e8fd4dd72f580956205c2241ec354ea8d68316d62c3c10c9`. Three shelves were produced in the 12-group case. Initial shelf state is synthetic and native allocator, constructor, clone and destructor are guarded preallocated stubs; frozen same-object grouping and lifetime remain admissions. A new original pairing/scale host has been prepared for the three-row odd case, two-row truncated control and synthetic equal-width control; no native result is asserted until reviewed and executed.

The independently reviewed original three-shelf pairing/scale host matched all three cases, including the conditional odd-row case; receipt `target/creature-skin-atlas-pair-scale-12group-0473-prep-20261109/result.json` SHA-256 `9dbcd0d187cec05f2fd4a6fe62bbdad4d04bb3e6a76e891a9d35a3177685afbd`. Checked Rust matched 2,340 bytes, receipt `target/creature-skin-atlas-pair-scale-12group-rust-verify-20261109/result.json` SHA-256 `26f6a55fc051a2b88ebd2e536b892a39853689e42ec2510250d53f491d8810b0`. The conditional scale is `3F2C2996`. A new original group-offset/row-advance host is prepared for all twelve groups in paired source order across three rows, starting at zero with wrapper-supplied row boundaries; it awaits independent review and execution. Frozen same-object data, preallocated state and successful lifecycle are admitted; there is no live atlas or texture claim.

The reviewed original 12-group paired offset and row-advance host matched all 12 groups; receipt `target/creature-skin-paired-offset-12group-0473-prep-20261109/result.json` SHA-256 `dbe4c9ff6b85f619bf65e1e19f161f23a182b155a6284c68f403ec0a13d3672e`. A target-only comparator invoking the existing Rust composition for twelve groups and three rows matched all 288 bytes; receipt `target/creature-skin-paired-offset-12group-rust-verify-20261109/result.json` SHA-256 `f178b0cd5199b281a3d369bde68c1b6cbaf0df63bbb2483ca5f6272419dd6ae0`. Explicit zero-origin coordinates and wrapper row boundaries, preallocated ownership, frozen same-object state and successful lifecycle remain admissions. A separate original per-UV host has been prepared using these offsets and matched `3F2C2996` scale for the 85 conditional generated UVs and three controls; it awaits independent review and execution. No live atlas, texture or identity is established.

The reviewed original 12-group paired UV host matched 85 conditional generated UVs plus three synthetic controls, receipt `target/creature-skin-uv-paired-12group-0473-prep-20261109/result.json` SHA-256 `b7763cea0d2290c090eb26f7be1c88027cf3362566d0b94b02b24b3bcb52b940`. A target-only direct Rust composition independently matched the complete pairing and offset streams, mapped the twelve native-ordered groups to 85 UVs, and matched all 704 original output bytes; receipt `target/creature-skin-uv-paired-12group-rust-verify-20261109/result.json` SHA-256 `4c138be42db9db585be1a790630d0d5b602017fb785442c2b4b6a86355cc9e3a`, output SHA-256 `bb946f07d8e1ca05fed4516b960d48a5e77ab9f48276c8c8964766c5b58879d8`. Scale `3F2C2996` and the four initial row groups are conditional native-matched results, not observed runtime atlas state. Frozen same-object identity, zero-origin wrapper scheduling, guarded lifecycle stubs and successful source operations remain admissions; texture binding and actual live identity are unproven.

After `50B3B0`, the same conditional `4CC540` path calls `509090` before `5098B0`. `509090` consumes geometry +30 UVs, +80 per-corner UV ordinals and +124 per-UV `(group, corner)` metadata; it accesses neighboring corners through signed tables at `013F1890` and `013F1898`. For each non-sentinel metadata entry it subtracts the selected UV from the two neighboring UVs via `50D8A0`, computes the ordered binary32 determinant at `5092AA..5092C2`, and compares with `013F1894=33D6BF95` via COMISS/JBE: determinant <= threshold follows the fallback branch. This is an oriented comparison, not an absolute-area comparison. Positions and normals feed subsequent tangent/bitangent operations, and `5098B0` then forms subobject triples for the later `46E3C0` export path. Those static call relationships do not establish any texture/material binding or stable live object identity.

All 85 metadata entries in the conditional twelve-group fixture select a valid originating corner, and both source neighbor offsets stay within the 252-corner and 85-UV buffers; a binary32 model finds all 85 determinants strictly above the threshold (approximately 0.00304 to 0.01243). A separate source-instruction host has been prepared for `509119..5092D8`, complete `50D8A0`, mapped original tables and threshold, with equality/negative/zero synthetic controls and the branch redirected to bounded capture. It awaits independent review and execution. This slice does not validate tangents, material, allocation, full export, or actual runtime identity.

The independently reviewed original determinant and branch host matched 85 conditional UVs plus threshold-equality, negative and zero synthetic controls, 704 bytes; receipt `target/creature-skin-uv-determinant-12group-0473-prep-20261109/result.json` SHA-256 `5f5a20577f9434b92a50ef6f6c1d664edb2019ef84b19d10584af903c4fedc48`. All 85 conditional UVs took the strictly-above-threshold branch; each synthetic control took fallback. The checked Rust model `creature_skin_uv_determinant_v1` matched all 88 determinant/branch pairs, receipt `target/creature-skin-uv-determinant-12group-rust-verify-20261109/result.json` SHA-256 `f38a30dcc4dfae74d1a4ce951b0ea3274cf52e47934ff97b5cbdfea9b3a539ca`, focused tests 2/2. Stable supplied UV/corner/metadata buffers and original signed tables, finite normal-or-zero arithmetic and MXCSR 1F80 are admitted. Tangents, material, actual texture binding and live identity are not established.

The reviewed original `509119..509507` and vector helper closure matched 85 pre-orthogonalization tangent candidates using the conditional 44 averaged positions, 252 position indices and transformed UV/corner buffers; receipt `target/creature-skin-uv-tangent-prefix-12group-0473-prep-20261109/result.json` SHA-256 `8d18aaf662c4f632a26243ce6d84d8b6c82b705f7ff488a44fcaea9dfe475ea3`. The checked Rust projection matched all 1,020 bytes; receipt `target/creature-skin-uv-tangent-prefix-12group-rust-verify-20261109/result.json` SHA-256 `35e5cb36672343963036a7e3707a162ff22cb440c3d4892ecd760d6c8fedfa93`, focused tests 2/2. The host traps the fallback path, uses explicit source buffer/index identity and finite normal-or-zero operations with MXCSR 1F80. These are candidates before normal projection and normalization, not final tangents; native lifecycle, texture/material binding and live identity remain unverified.

The reviewed original `509508..509685` and full projection, normalization and cross-product helper closure matched all 85 conditional tangent/bitangent pairs, receipt `target/creature-skin-uv-tangent-final-12group-0473-prep-20261109/result.json` SHA-256 `a601e8d8781acf0e8f8508fb35aa41cd1c42db1286e6cb0a218541236809a9a2`. The host used the 44 retained normals and admitted +6C normal-index stream as the earlier copied +58 position indices; all projected squared lengths were finite and nonzero. It checked the exact pinned CRT loaded path and direct `_CIsqrt` export before original execution, with disk/mapped-image stability admitted. The checked Rust `creature_skin_uv_tangent_final_v1` projection matched all 2,040 native bytes; receipt `target/creature-skin-uv-tangent-final-12group-rust-verify-20261109/result.json` SHA-256 `6f20e861a1e872a515233a8bcae20bddc7cd72121b2067aa1abd6bef6802f7bd`, focused tests 2/2. Its square-root spill is explicitly supplied; the fixture's Rust sqrt agreement does not prove general CRT equivalence. Stable same-object indices and lifecycle, source FP controls, texture/material binding and live identity remain open.

A target-only inspection-join audit matched all 85 tangent/bitangent pairs to the corresponding existing twelve-group UV entries through retained per-UV metadata and 252 corner UV ordinals; the same 252 conditional position indices map the 44 exported normal vectors to the tangent host's admitted +6C inputs. Pinned native and Rust tangent streams agree over all 2,040 bytes and contain finite values. Audit receipt `target/creature-skin-tangent-export-audit-20261109/result.json` SHA-256 `2a0a6d2e32b2667f2ac189d6b120acd17f2303347bdf48e6a0e1dc361e4caf5f`. This establishes a feasible explicit inspection export only under the retained same-object +6C/+58 equality, frozen lifecycle, source-compatible sqrt spill and synthetic preallocation admissions; it does not establish a live material, texture, identity or unconditional mesh.

A first eight-generated-group `509B22..509C57` comparison exited zero but did not match the model; its full result and raw output remain in `target/creature-skin-triples-8group-0473-prep-20261109`. Static inspection identified swapped wrapper locals: original `509A98..509AF4` initializes/increments local face-list ordinal at EBP-120 and loads the subobject face-list value at that ordinal into EBP-124; `509B22..509B31` uses EBP-124 for global geometry corner input, while `509C37..509C55` uses EBP-120 for local output. The first wrapper reversed these values, so it read wrong global faces and wrote wrong local slots. A separate v2 corrects this framing and audits the linked wrapper sequence; it is prepared for independent review, not yet executed. The first failure is not native parity evidence. Conditional +6C=+58 source-index identity, eight generated-group face lists, preallocation, first four group exclusions and no live material/identity remain explicit.

The corrected original eight-group `509B22..509C57` host matched the complete 12,384-byte fixed output; receipt `target/creature-skin-triples-8group-0473-prep-v2-20261109/result.json` SHA-256 `5bd2dbece6a3682c7be9b376b08a82f4d2b602962838fadb85198ccc88fb0885`. It consumes the retained eight group face lists, position and UV corner streams, and a supplied normal-index stream equal to the copied position indices, resetting the position-key cache per group. The checked Rust eight-group composition matched all output bytes, receipt `target/creature-skin-triples-8group-rust-verify-20261109/result.json` SHA-256 `481a15fd57c80d9eef589f3893073f037042828a3a56efa6064847fbbca0c809`, focused tests 2/2. The prior v1 mismatch and original streams remain preserved. The first four row-derived groups' face lists, same-object +6C identity, real allocation/lifecycle, material and live creature identity remain unproven.

The independently reviewed original `4CCB10` constructor and `509981..509AE1` empty-face traversal matched four disjoint preallocated objects, 160 output bytes SHA-256 `b393978842a0fa3d3e1470196f098f473f9678e72463cb65ec4ab5581856c2e4`; receipt `target/creature-skin-empty-row-groups-0473-prep-20261109/result.json` SHA-256 `26ef80fe110098a929a83563da0bd89414a4bff4f46be5ce484562b5ffe4f73f`. The wrapper pins the four selected frozen rows 6–9, supplies successful disjoint preallocation and checked no-op constructor-member, zero-resize and empty-clear stubs, and traps the nonempty face branch. It confirms the supplied objects have empty face, local-index and triple descriptors at this boundary. It does not execute or validate the intervening grouping, atlas and tangent calls, complete `5098B0`, real allocation/lifecycle or runtime object identity; therefore it does not establish the actual initial exported vertex base as zero. Guard words are post-call canaries, not guard pages.

The independently reviewed `50AB28..50AFAE` grouping preservation host matched the 84-face conditional outer output and before/after snapshots of all nine face/index/triple descriptor words for four disjoint original `4CCB10`-constructed preexisting objects. Receipt `target/creature-skin-outer-preserve-0473-prep-20261109/result.json` SHA-256 `da4eb49f056dc26719d110c9c00013bb52fcfff44b2269f85a4c09cca26d1991`; 3,984-byte output SHA-256 `e57771e0e194550c9461227f1eccd8a786eeb02ec5b05bc9db07a7f6ee179888`. First-four pointer slots and object guards were checked, with original constructor member-init and grouping allocation, reset, queue, clone and append replaced by checked preallocated stubs. This establishes only those descriptor and pointer snapshots for the supplied slice; it does not execute the complete atlas, tangent, final assembly or native lifetime paths and does not establish actual live base zero. Canaries are post-call checks, not guard pages.

The reviewed same-object grouping→initial-atlas host matched the prior grouping snapshots, all 12 original `50B419..50B53E`/`50C570` bounds-and-rotation rows with sequentially carried area, and four post-atlas descriptor snapshots. Receipt `target/creature-skin-group-atlas-preserve-0473-prep-20261109/result.json` SHA-256 `5611654fc7a32233b50532be0aa44b935560ced46f6f15be34796802f5e75099`; 4,416-byte output SHA-256 `74ae8d28f591046aa9f523f31d3621e141bca7c9d377e967d786c37aa948c913`. The same four disjoint constructor-initialized objects and eight grouping outputs are used throughout; first-four bounds are supplied from the frozen-row fixture, all 12 starting bounds are checked against the retained cases, and margin and area order are fixed. The inherited checked grouping stubs and synthetic lifecycle/no-alias admissions still apply. Remaining atlas shelf selection, packing, UV transforms, tangent computation, complete assembly, real allocation and live identity were not executed; this result does not establish live base zero. Guards are not guard pages.

The independently reviewed same-object grouping→initial-atlas→shelf host matched the retained grouping and bounds outputs, the 12-group shelf selection and the first four objects' 12 selected descriptor words (including +44/+48/+4C) before and after shelf selection. Receipt `target/creature-skin-group-shelf-preserve-0473-prep-20261109/result.json` SHA-256 `b23ba70e21e98052092fb216fe0657e987538879ca8d66180d0b1d45d99c5286`; 5,572-byte output SHA-256 `ffcf4ab9dc6a8636f592848f66cf9434088aad2be5063ac48d6f830e4410ece3`. The four +44 descriptors were also checked immediately after original construction. The original grouping, initial bounds/rotation and shelf slices run on the same supplied 12 objects, with pinned source-matched sort order and sqrt, guarded preallocated shelf/row pools and checked lifecycle stubs. Sort/sqrt execution, later pairing, UV/tangent/final assembly, real lifecycle and live identity remain unverified. Selected snapshots do not establish full-object equality; canaries are not guard pages.

The reviewed same-object grouping→initial-atlas→shelf→pairing/scale host matched the retained three-shelf pairing output and the first four objects' 12 selected descriptor words after pairing. Receipt `target/creature-skin-group-pair-preserve-0473-prep-20261109/result.json` SHA-256 `c05d3c63151103d442bafac49be9eec6505974b66e6a56645fcebc3a485fefdd`; 6,544-byte output SHA-256 `843cbac72fc9b6ad58134a5d2bd9e50b210e42c71ff8a282a43505c17a63ce51`. Original `50B83B..50BE09` and `50BE3A..50BE87` slices consume the same three shelf structures and group pointers from the prior host. Source sort/sqrt are supplied from retained matched fixtures, and guarded preallocation/lifecycle stubs and same-object assumptions remain. Later offsets, UV processing, tangent, full assembly and cleanup, real lifecycle and live identity were not executed; this does not establish actual live base zero. Selected snapshots and canaries do not establish whole-object or guard-page behavior.

The reviewed same-object continuation through original `50C016..50C301` empty-face gather, empty UV and empty +44 attribute-loop guards matched all four initial objects and their 12 selected post-slice descriptor words. Receipt `target/creature-skin-group-uvempty-preserve-0473-prep-v2-20261109/result.json` SHA-256 `a3370595df9833820f43511d4210de988a64d561b356e4c2311b082bc5e55bc8`; 6,752-byte output SHA-256 `98525a3798df4e1b78fee5613c828d9cb9c819b70b361c10d078ca868406a49a`. Four empty-scratch reserve, sort, unique and clear calls were guarded stubs; nonempty face, UV and +44 loop bodies trap. The first unexecuted prep is retained: static review found its +44 nonempty fallthrough lacked the intended trap; v2 adds it and independently verifies the linked jump. The same-object chain still supplies sort/sqrt and synthetic preallocation/lifecycle. Per-group UV offsets, nonempty UV processing, rest of atlas and cleanup, tangent and complete assembly, native allocation and live identity remain unexecuted; selected snapshots do not establish whole-object equality or live base zero. Canaries are not guard pages.

The reviewed same-object continuation through original `50C3B1..50C4A9` matched the four initial groups' final bound add/scale/rotation-bit-clear words and 12 selected descriptor words. Receipt `target/creature-skin-group-final-bounds-0473-prep-20261109/result.json` SHA-256 `4008f872dbc75debd1d47e2c720e7aa2af332ba4241578bc0188b1e2ba8f1eef`; 7,024-byte output SHA-256 `ba94cff4d835e851d58d1d9f1300de14d782b6019c1942cb5dd539ed9b28c73a`. The four offsets are supplied from the independently matched conditional paired-offset fixture via the pinned sort and paired-row mapping; source paired scale is retained in the same wrapper. Source row scheduling, generated-group UV processing, full atlas cleanup/tangent/assembly, native allocation and live identity were not executed. Same-object/no-alias and synthetic helper lifecycle admissions remain, so this is not evidence of live base zero. Only selected descriptor words are checked; canaries are not guard pages.

The independently reviewed same-object continuation through the original `509981..509AE1` empty triple-builder guard and `46E6A6..46E6CD` export-base calculation matched the four supplied groups. Receipt `target/creature-skin-group-base-preserve-0473-prep-20261109/result.json` SHA-256 `f22a09129cb36c0a93a2784b2036f81842f815a2aa7f19a48f205f90b94efbf6`; 7,244-byte output SHA-256 `b9c67fd1eca45c271b92cb5ce8955229a579b65ac85898b69d8d104efea5aab9`. The wrapper supplies an explicitly empty preallocated output position vector, checks 44-pair cache bounds and zero-size resize/clear stubs, and traps the nonempty source branch. Source atlas cleanup, `509090`, generated-group interleaving and actual object lifecycle remain unexecuted; the observed zero base is conditional on the supplied vector and is not a live base assertion. Snapshots cover selected descriptor words only; canaries are not guard pages.

The reviewed original fresh export-object constructor→retain→reserve host matched five snapshots and the base calculation. Receipt `target/creature-skin-export-init-reserve-0473-prep-20261109/result.json` SHA-256 `6480105e6fa2d9248fb7a88b69063a178dd4d98a73b01863884adbdee153a2f0`; 104-byte output SHA-256 `d30ea8c291163250c6912ff21de278fb194204fd446614c047834bf9b753bdec`. It executes complete original `461630`, `428DA0`, direct `472F40` retain target, `474070`, `42D800` empty-copy and base arithmetic. A guarded 0xB0 object and successful 1,020-byte synthetic allocation are supplied; free traps. The PE vtable slot is checked to resolve to the direct retain target, but virtual dispatch, real allocation and live object lifetime were not executed. Reserve 0 and repeated reserve 85 were also checked; canaries are not guard pages.


### Conditional constructed export base join

The independently reviewed, externally pinned original-instruction host `target/creature-skin-group-constructed-base-0473-prep-20261109` matched its 7,264-byte expected stream with exit 0. Its `result.json` SHA-256 is `3db085761b2f235d9fe199f45829a7004bc94b7e76580a408d85a4ffbfc18c10`; native stdout SHA-256 is `3fb3d37b49bacf9861d90b45e18da308f2657fed8803f785ad784bec5d86835d`. The host runs original `461630` construction, the direct `472F40` retain target selected from the pinned PE vtable, and original `474070` reserve and `42D800` empty copy before the conditional same-object grouping/atlas/empty-builder/base slices. A checked synthetic successful 1,020-byte allocation supplies the initially empty position vector. The resulting descriptor and four base snapshots match the conditional model.

This is selected-field evidence under supplied no-alias and stable-lifetime assumptions, not a live base-zero observation. Virtual dispatch, the real allocator, atlas cleanup, `509090`, generated-group interleaving, and live creature identity remain outside the executed closure. Guards are post-run canaries rather than guard pages; the supplied sort, square root, offsets, and other prior fixture admissions remain in force.


The next `509090` same-object join is not yet validated. At `5090A5..5090D2`, the source obtains the number of eight-byte UV entries from `Geo+30/+34` and calls `50DCF0` on the 24-byte record vector at `Geo+44`. Its grow path enters `50F4E0`, which copies the two three-float halves from a stack value supplied by the resize caller; initial fill words and allocation behavior must therefore be admitted or modeled, not assumed zero. The loop consults `Geo+124` metadata, skips sentinel entries, and otherwise reads neighboring `Geo+80` indices and UVs. The retained 85-entry tangent fixtures validate non-fallback numerical slices independently, but do not execute this resize or establish the same-object descriptor preservation. A bounded full-loop host must explicitly inject or execute the matched transformed UVs, supply and check all metadata, corner, position and normal arrays, guard 85 tangent records, trap unsupported branches, bind the original CRT square-root export before use, and account for all direct helper calls. No such full-loop comparison is claimed.


`50DCF0` reserves 0x3C stack bytes and, on growth, passes the uninitialized local `EBP-18..EBP-1` as its six-float fill value to `50F4E0`: the resize instruction path does not write those bytes. For an initially empty vector with at least 85 records of supplied capacity, the helper takes `50F68F`; `511CD0` constructs the new records, and `50EA60` receives an empty old range. `41E610` copies each fill component with x87 `FLD`/`FSTP`, so an original-helper comparison should deliberately seed finite normal/zero stack words and check x87 state. Such a host can separately verify that all 85 metadata entries are non-sentinel and that the original loop overwrites both halves of every tangent record, but must admit the seeded stack, sufficient preallocated capacity and no intermediate observation. This does not cover native allocation, arbitrary inherited stack values or sentinel records.


### Conditional same-object tangent loop

The independently reviewed, externally pinned `target/creature-skin-tangent-loop-same-object-prep-v1-20261109` original-instruction host matched 9,304 output bytes, exited 0 and closed its private desktop. `result.json` SHA-256 is `99f1444bf3e0dc4985f9de01742a1b3887350e250d5c749c61d9d041e9d2e628`; native stdout SHA-256 is `7470541ad75822260ca5d8efc1d9b476f6b4502467281ec84b15a8f193362647`. It runs the full original `509090` non-fallback loop on the same conditional geometry object used by the grouping and selected-descriptor preservation fixture, and compares the existing 7,264-byte snapshot plus 85 complete 24-byte tangent/bitangent records to separately retained native results. The resize follows original `50DCF0`, `50F4E0`, `511CD0`, `41E610` and empty-range `50EA60` instructions. Every metadata entry is checked non-sentinel, and the retained determinant cases are checked non-fallback.

The wrapper supplies pinned transformed UVs, corner/position indices, positions and normals; seeds the relevant scratch stack with finite zero; and provides capacity for 85 records. The source's intermediate fill and any observer of it are not modeled. The CRT path and direct `_CIsqrt` export are checked before use, subject to mapped-image/disk stability. The grouping and atlas inputs remain conditional; selected descriptor snapshots, synthetic no-alias allocation, stable lifetimes, direct rather than virtual retain, omitted atlas cleanup and generated-group interleaving do not establish live creature identity or a live initial base. Post-run canaries are not guard pages.


### Conditional full subobject traversal

The independently reviewed, externally pinned original `5098B0` full traversal matched on the same conditional geometry object after the `509090` tangent loop. `target/creature-skin-full-triples-same-object-prep-v1-20261109/result.json` SHA-256 is `69d606c2f870de05e2222355c6f14e8d4be8aa2c4e6fb93b2d5c1aafa04e0ec6`; the 21,688-byte native stdout SHA-256 is `1b8cd2383a24bc32cd056836042da355b43d719be21edb9b7adbcaeb5f640896`. It matches the prior 9,304-byte selected snapshot and tangent stream followed by the independently retained 12,384-byte eight-group triple/corner stream. The wrapper checks the four initial subobjects have empty face and output descriptors, validates each of the eight generated groups' pinned face list and count before traversal, and supplies a 22-pair cache, disjoint per-group corner/triple storage, and a temporary triple vector. Nine source helper calls use bounded synthetic vector and lifecycle stubs.

The source geometry inputs, transformed UV injection, `+6C` equal to `+58`, finite-zero stack, preallocated storage, no-alias and stable-object assumptions remain conditional. Original allocation, actual vector lifecycle, atlas cleanup, later export iteration and live identity are not established; only the serialized records and selected prior fields are compared. Post-run canaries are not guard pages.


### Conditional same-object export loop

The independently reviewed, externally pinned original `46E65B..46E842` group/export loop matched after the same conditional object's original `509090` and complete `5098B0` traversal. `target/creature-skin-export-loop-same-object-prep-v1-20261109/result.json` SHA-256 is `bc4cb5abd10379fc9aa48bd3e0aae96b3e3bf7e592a6c2ba4fc3e61f083ef98c`; the 28,476-byte stdout SHA-256 is `1816afa9346922d3fa65c37523ef4bf8b37188e3ebf450c80644bedfbd3da772`. The first 21,688 bytes reproduce the prior snapshot and traversal; subsequent bytes match 85 position/normal/UV/tangent-first-half attributes, four u16 index-times-three values and four raw weights per vertex, and 252 dword indices. Prior retained native receipts independently supply the expected attribute, record projection and zero-base index streams.

The wrapper verifies the complete 12,384-byte prior triple stream before entering the original 12-group loop, supplies a synthetic owner with 44 caller-alpha 0.7 pass-2 conditional records, and uses guarded preallocated append stubs; the output position vector uses the earlier original constructor/direct-retain/reserve under synthetic allocation. The source loop computes each group base from the current position-vector end. This comparison does not execute the outer `46E3C0` owner/count validation or reserve calls, atlas cleanup, final palette-classification tail, native allocation and lifecycle, or establish live smoothing settings, palette identity, texture binding or creature identity. Position/normal/UV/tangent inputs and `+6C=+58` remain conditional, no intermediate observation is modeled, and canaries are not guard pages.


### Conditional same-object inspection export

Use `python tools/creature_skin_0473_partial_export.py --include-same-object-export --output <new-path>` to opt in to the same-object inspection section. The flag also includes the required existing conditional eight-group geometry, UV and tangent sections; the default export is unchanged. The exporter pins the matched source-loop and caller-alpha receipts/streams, verifies all 85 attribute and record mappings, reconstructs and checks 252 global indices, and preserves the u16 offset and float weight words as hex. It explicitly selects caller-alpha `0x3F333333`, two passes; it does not infer the live smoothing setting or palette identity.

The retained artifact `target/creature-skin-same-object-inspection-20261109.json` has SHA-256 `d65dd328b49283d9aaa283119c76d9e55553721aa405d7e3f9776ce6b1d8f828`. The focused exporter suite passed 27/27 tests, including missing-fixture, mapping and tamper checks. The comparison is conditional on frozen geometry, supplied transformed UVs and records, a synthetic owner, preallocated append buffers and stable no-alias lifetimes. Outer owner validation, atlas cleanup, real allocation/lifecycle, live creature identity, palette identity and texture/material binding remain unverified; it is an inspection artifact, not a finished runtime mesh.


### Conditional original export entry

The independently reviewed, externally pinned `target/creature-skin-export-entry-same-object-prep-v1-20261109` host matched the prior 28,476-byte output with original `46E3C0..46E842`, including the source owner/geometry check, 44-record versus 44-position count check, and original summation of all twelve groups before the export loop. Its receipt SHA-256 is `3b11010aa9d8de37c0b200e8b0ccd7e04bc5b8ac919e5de6dcef8c8b3943b8ff`, and native stdout SHA-256 is `1816afa9346922d3fa65c37523ef4bf8b37188e3ebf450c80644bedfbd3da772`. Seven synthetic no-growth reserve stubs verify the exact output descriptor, empty length, preallocated capacity and requested 252 index/85 vertex counts. The wrapper supplies the same synthetic owner, 44 caller-alpha 0.7 pass-2 records and guarded empty skeleton descriptor; an injected ret-8 trampoline cuts before the source palette tail.

This strengthens the conditional owner/count and source-order comparison, not live owner identity or source allocator behavior. Original reserve allocation, palette classification, atlas cleanup, live smoothing configuration, lifecycle and texture/material binding remain unverified; selected fields only are snapshotted, and canaries are not guard pages.

The same-object option now also requires the stronger original-entry receipt `target/creature-skin-export-entry-same-object-prep-v1-20261109/result.json` (SHA-256 `3b11010aa9d8de37c0b200e8b0ccd7e04bc5b8ac919e5de6dcef8c8b3943b8ff`) and requires its full native stream to equal the earlier loop stream. Original owner/44-record validation and group totals therefore ran under the supplied synthetic owner; seven reserve operations remain guarded synthetic no-growth stubs. The revised artifact `target/creature-skin-same-object-entry-inspection-20261109.json` has SHA-256 `2079819c9955efa5161e09a0beb34c2699f3dc04866506e825dd8429c99804c5`. The default export remains unchanged; 27 focused tests passed, including tampering with the new receipt and stream. Real owner identity, allocator/lifecycle, palette tail and live settings remain unverified.


### Conditional full export and row categories

The independently reviewed, externally pinned same-object host `target/creature-skin-export-category-same-object-prep-v1-20261109` matched the complete original `46E3C0..46E998`, including its true cdecl return and classification tail. Receipt SHA-256 `241789b27edf3fdec72f7cff7bccc6d05928b7502f09b2dabed7847d7d513a5e`; native stdout SHA-256 `3a117ba39ddf15ed58bf2daf4360e013a1738ae4183ee25f397e02a55ae7f41e`. The 28,516-byte stream reproduces the prior 28,476 bytes and appends ten classification dwords `[1,1,1,1,1,1,4,4,4,4]` for the exact pinned frozen 0473 rows. The wrapper supplies those rows through a synthetic same-owner skeleton descriptor and uses a guarded preallocated ten-dword resize stub. These are **row categories, not palette IDs**.

The same-owner association, row and smoothing state, stable lifetimes and no-alias storage remain admissions; original allocation, atlas cleanup, runtime identity and texture/material binding are not established. Canary checks are post-run and not guard pages.


### Conditional frozen-row categories

Use `python tools/creature_skin_0473_partial_export.py --include-row-categories --output <new-path>` to add ten conditional frozen-row classifications. This option includes the prerequisite same-object inspection sections and leaves the default and existing options unchanged. It pins the complete original-export category receipt and stream and the 1,400-byte frozen row snapshot; checks that the prior 28,476-byte output prefix matches the separately checked same-object stream; and recomputes all ten classifications from the source `+08` flags and `+0B` kind in source branch order. The values are `[1,1,1,1,1,1,4,4,4,4]`: **row categories, not palette IDs**.

The retained artifact `target/creature-skin-row-categories-inspection-20261109.json` has SHA-256 `cbbb6ee23a05b32ec45ed7b56b791828ec17febe53089be6417bf756d0e49219`. The focused suite passed 29/29 tests, including branch-priority, missing-fixture and receipt/stream/row tampering checks. Synthetic same-owner association, preallocated resize, frozen input and stable lifetimes remain admissions; live owner, allocation, smoothing state, palette identity and material/texture binding are unverified.


### Conditional row-category prefix

The independently reviewed, externally pinned `target/creature-skin-category-prefix-same-object-prep-v1-20261109` host matched original `467F79..468091` after the same-object export/category fixture. Receipt SHA-256 `81452cd40ac77d6890bb9583e167643a5259b49635513b6f1322d6f254706477`; 28,740-byte native stdout SHA-256 `0813ca304ebe9ae5fe8a70174870808b19bfc3c5525cb2bcbf03e83d400a12b3`. The original ten frozen rows have prefix length ten; three synthetic row-kind controls verify trailing-two, all-trailing and nontrailing class-1 behavior with lengths eight, zero and ten. The explicit nonnull external argument plus argument-8 value one takes the source bypass of external-count augmentation. A guarded synthetic ret-8 resize changes only the category vector's logical end; preserved backing bytes and destructor/allocation behavior are not established.

The row categories remain classifications, not palette IDs. The supplied row/object association, conditional export inputs, stable lifetimes, synthetic resize and absent live identity remain admissions; this comparison does not establish a canonical rig binding.

The row-category option now also pins the original `467F79..468091` category-prefix receipt `target/creature-skin-category-prefix-same-object-prep-v1-20261109/result.json` (SHA-256 `81452cd40ac77d6890bb9583e167643a5259b49635513b6f1322d6f254706477`) and its 28,740-byte stream. It requires the prior category stream to match exactly and checks the frozen prefix plus all three trailing-class-1 control snapshots, including their synthetic unchanged backing bytes. The conditional frozen-row prefix length is ten. The revised artifact `target/creature-skin-row-prefix-inspection-20261109.json` has SHA-256 `b1ac98c359aa1e9d215ed59fb2492419df99f3122bb3a04bc5ea9610ed50894c`; 29 focused tests passed. External-count bypass and logical resize are synthetic admissions; the categories are not palette IDs and no live binding or destructor behavior follows.


### Conditional category attribute descriptor

The independently reviewed, externally pinned `target/creature-skin-category-attribute-same-object-prep-v1-20261109` host matched the original `469218..46924F` consumer and its original descriptor/copy/append helper closure after the same-object prefix fixture. Receipt SHA-256 `3fd70d8d7ca414e5e11aed4d09065bd6428989ac98f5bfa318ecb57833b5fb57`; 28,780-byte native stdout SHA-256 `c1abed934a999ba538edce1dc416d4629f51789187712fd52c6e99acd1be32f4`. It checks and normalizes the pointers in an appended 32-byte `(0x18,0,6,5)` record whose descriptor references ten category dwords with element size and stride four. The pinned source vtable slot was mapped to the original retain target; three virtual retains advanced the conditional owner's reference count from one to four. The available-capacity append was bounded and growth trapped.

The cut occurs before descriptor cleanup and releases. The resulting record's ultimate semantic meaning and any renderer, matrix-palette or live-object binding remain unproven. Supplied category vector, synthetic owner, preallocated storage, stable lifetimes and no-alias conditions remain admissions; post-run canaries are not guard pages.


### Conditional category descriptor temporary cleanup

The independently reviewed, externally pinned `target/creature-skin-category-cleanup-same-object-prep-v1-20261109` host matched original `46924F..4692AF` temporary-descriptor cleanup after the conditional category-attribute append. Receipt SHA-256 `098e37d508652fd748df18a6fe26f4bb1984e86416fd3cf83b7d6f6bc75d8b5a`; 28,780-byte native stdout SHA-256 `ed924236f52885baa61ed3763451535dfc1957462797b6a8e349dfe6ead049fb`. Two virtual releases used the pinned source vtable target `517630` and original `41CC70` nonzero decrement path, reducing the synthetic owner's reference count from four to two while leaving the appended descriptor's normalized owner and category pointers intact. The zero-count destructor branch was redirected to a terminating trap.

This covers the two temporary references only; one conditional owner reference and one appended-record reference remain. Final appended-record release, real allocation and lifetime, descriptor meaning, renderer/palette binding and live identity are not established. Canary checks remain post-run rather than guard pages.


### Conditional three-stream source descriptors

The independently reviewed, externally pinned `target/creature-skin-three-attributes-same-object-prep-v1-20261109` original-instruction host matched `468F3F..4692AF` after the conditional same-object export and category cleanup. Receipt SHA-256 `03b7f6079ee72c74de4ba9e1cd053a7f9a9f916ac3f78644d55ca9a5056d6b25`; 28,896-byte stdout SHA-256 `6db22f66898bff432bb153bef23dd459db23b598f2b0a79c2f21f3306633506e`. Three appended 32-byte records reference the same conditional owner's 85-entry u16-offset stream with source header `(9,0,9,0)` and stride 8, 85-entry weight stream with `(10,0,4,0)` and stride 16, and ten-entry category stream with `(0x18,0,6,5)` and stride 4. Pointers were checked then normalized; the three four-byte ordinal entries were 0/1/2 with FFFF high halves. Original retains and temporary releases leave the synthetic owner's count at five, including previously appended ownership; available-capacity paths were bounded and growth/zero-count destruction trapped.

These numeric headers are source descriptor tags, not established shader semantics or palette IDs. The source/lifecycle inputs, preallocated storage, no-alias and stable-owner assumptions remain conditional. Final descriptor cleanup, downstream renderer acceptance, matrix-palette binding, texture/material selection and live identity remain unverified; post-run canaries are not guard pages.


### Conditional owning group handoff

The independently reviewed, externally pinned `target/creature-skin-group-handoff-same-object-prep-v1-20261109` host matched original `4695DC`/`472F60` one-element inline wrapper construction, a directly invoked original no-growth `41E940` pointer-vector copy, and original `41E8E0` temporary-wrapper destruction. Receipt SHA-256 `3cac86a4bac1c31f5236338113e1ddbbbfa3475125d9723b43184ba32ec0ab2c`; 28,916-byte native stdout SHA-256 `0a81e94aec1f0752e04afaa7bb918a9e7a216ff94751a98f5615e5b02e86f8b5`. A guarded synthetic group object's `+08` vector refers to the three previously matched source descriptor records; the destination retains that group after temporary destruction, and its separate reference count follows 1→2→3→2. The wrapper's inline-storage free sentinel and pointers were checked, and growth/free/zero-count deletion paths were trapped.

This is a supplied group-object and preallocated-destination ownership comparison. It does not execute the `757240` caller/resource service, final destination release, actual allocation or lifecycle, interpret the descriptor tags as shader semantics, or establish live renderer/palette binding or identity. Guards are post-run canaries rather than guard pages.


### Conditional source resource-field handoff

The independently reviewed, externally pinned `target/creature-skin-resource-field-handoff-prep-v1-20261109` host matched the previous 28,916-byte ownership stream while running original `7573D1..7573E6` and its original no-growth `41E940` helper to copy the supplied group wrapper into a guarded synthetic resource object's `+84` vector. Receipt SHA-256 `ab99d36e8df2fab333121f1ea0d4218e38b6468259cbbf9ee8404aa91c1b7dd9`; native stdout SHA-256 `0a81e94aec1f0752e04afaa7bb918a9e7a216ff94751a98f5615e5b02e86f8b5`. A bounded synthetic caller stack supplies the source pointer and captures the source status-byte write; the inline wrapper is then destroyed with its free sentinel checked, leaving the synthetic destination's group reference alive.

This source-slice comparison admits the resource object and its preallocated field, group object, no-alias storage and lifetimes. It does not execute the resource-service/constructor path, final destination release or GPU consumer, and does not establish actual descriptor semantics, texture/material or palette binding, or live identity. Guards are post-run canaries rather than guard pages.

### Conditional position-descriptor selection

The independently reviewed, externally pinned `target/creature-skin-position-selector-prep-v5-20261109` comparison matched 28,964 output bytes. Receipt SHA-256 `9b41013c9c61bd2bda4c00c2a0da60753c93b740ef9c7a4f0b6f06d93958c62e`; stdout SHA-256 `106899d6d5c6770eac03961aa17aa035ae8b5e7b37473bbac64aad78ca88ad7b`. Original `468A5E..468B05` builds and appends the `(1,0,3,0)` descriptor over the previously compared 85-position, 12-byte-stride vector. Original append helpers then copy the three previously compared records into a guarded, preallocated four-record vector on the same supplied synthetic group. Original `71DA50` selects the position record at ordinal 0 and the tag-9 record at ordinal 1; a separate three-record control misses the position query. These fixed inputs and owner references are supplied; the new construction and copies increase the synthetic owner reference count from 5 to 11, and the new retains are not subsequently released in this comparison.

The earlier `target/creature-skin-position-selector-prep-v3-20261109` run failed with shared trap exit 9 and empty stdout (receipt SHA-256 `cd0c04db9bace808f3572c48165d5426cfb83ffe7f0917b84b55231027962eec`). Its tag-9 control queried descriptor word 2 as 3 although the retained record has word 2 equal to 9; this is a source-confirmed harness inconsistency and a likely cause, but the shared trap did not reveal which guard failed first. V5 corrected that query and added separate diagnostic guard exits. The v3 failure and intermediate preparations remain preserved.

This establishes bounded descriptor construction and selection only for the admitted synthetic group and retained position stream. It does not execute the subsequent `72C0F0` spatial traversal, establish shader or palette binding, perform final releases or actual allocation, or identify a live creature, resource service, renderer, or lifecycle. Canaries are checked after execution rather than acting as guard pages.

### Conditional triangle bounds

The independently reviewed, externally pinned `target/creature-skin-triangle-bounds-prep-v1-20261109` comparison matched 30,980 output bytes (receipt SHA-256 `7e9c7562029e325fe36a76f4bf819133792dd83b07f1d5793822c4576587cfe0`; stdout SHA-256 `38b0f3fc4a24d8948c47935d30544bf279495421cd762d186e91f3b0f0cb75c9`). Following the matched position-selector stream, original `72C403..72C672` produced 84 six-word triangle minima/maxima from the retained 85 finite positions and 252 supplied, range-checked 32-bit indices. The harness supplied the index descriptor, identity mask table entry, loop counter, seeded stack and a preallocated guarded output; the comparison preserves source `COMISS` operand selection on ties.

This is the isolated simple, non-remap spatial loop with an explicitly supplied index mapping. It does not execute the preceding `72C0F0` group/primitive selection, the alternate remap branch, actual allocation or final ownership cleanup, and it establishes no renderer, shader, material, palette or live-creature identity. Canaries are post-run checks.

Use `python tools/creature_skin_0473_partial_export.py --include-triangle-bounds --output <new-path>` for a separate conditional triangle-bounds section. It includes the prerequisite same-object geometry, pins the position-selector and bounds receipts/streams, checks their complete prefixes and all 85 positions and 252 indices, and recomputes all 84 raw six-word extrema with the source tie selection. The default export is unchanged. The fixed inspection artifact `target/creature-skin-triangle-bounds-inspection-20261109.json` has SHA-256 `3d5bc4bbfbe840ccb20cd3df38585c05164ffbf04f362ca2fb08406a919a4a77`. Index mapping, group and seeded non-remap state are supplied; source group/primitive selection, alternate remapping, allocation, lifecycle, renderer, shader, palette and live identity remain unverified.

### Conditional aggregate bounds

The independently reviewed, externally pinned `target/creature-skin-bounds-reduction-prep-v3-20261109` host matched original `72CAEF..72CC46` for the 84 previously compared triangle bounds and two finite synthetic controls: signed-zero ties and an invalid-first box. Receipt SHA-256 `f3fbdb1bb91539d8dabcb70e8bd2ba65cdac644441a62fbc55a6559547ec7965`; 31,052-byte stdout SHA-256 `944ef8b538409bbd80f9ac0ee5ddb1d3f49f5ea3ac3e599f78037ac6d36f597f`. The harness supplied a nonzero count, stack and output pointer; on ties the original ordered comparisons retain the earlier accumulator, and an accumulator whose x minimum exceeds its x maximum is replaced by the next box. The comparison includes the full prior triangle stream and three six-word reduction results.

This isolated reduction does not execute the subsequent `72A790` recursive spatial-tree construction or establish its allocation, group selection, ownership, renderer, shader, palette, or live identity. The index mapping and group remain conditional; canaries are checked after execution. Preparation v1 and v2 assembly failures are preserved.

Use `python tools/creature_skin_0473_partial_export.py --include-aggregate-bounds --output <new-path>` to add the conditional aggregate bounds as six raw binary32 words. This option includes and checks the triangle-bounds prerequisites, pins the reduction receipt, output and control bytes, checks the full prior stream prefix, and recomputes the 84-box reduction plus signed-zero and invalid-first controls. The default export is unchanged. The fixed artifact `target/creature-skin-aggregate-bounds-inspection-20261109.json` has SHA-256 `87b0e41d40cc80e6dda204a328b4f78daff689a92f0c05388f582d1911a4a822`. The supplied group/index mapping and non-remap state do not establish a spatial tree, live ownership, rendering or creature identity.

### Conditional common-group count validation

The independently reviewed, externally pinned `target/creature-skin-common-group-validate-prep-v1-20261109` host matched original `4183D0` and its original selectors on the supplied four-record group. Receipt SHA-256 `5ecb518f85f482fdbfb3921286fc2fcb8350ae2f8d4690a2fffd5a48f5bc2b46`; 28,976-byte stdout SHA-256 `573c9a086a381f67c7ecb60aebaef17304ec700aaa535829f258ebd1b6a6da62`. Position, tag-9 and tag-10 counts of 85 return true; separate controls setting either latter count to 84 return false. Guarded wrappers require the supplied position descriptor and reject a missing or out-of-range tag-10 ordinal before the original source's `418451` recheck of the earlier tag-9 ordinal could permit negative indexing. The comparison does not execute or characterize the unsafe missing-weight source path.

The group, object association and no-growth lifetime remain synthetic; equal descriptor counts do not prove palette identity, shader binding, rendering, real allocation, cleanup or a live creature. Canaries are post-run checks.

### Conditional category-count gate

The independently reviewed, externally pinned `target/creature-skin-category-count-gate-prep-v2-20261109` host matched original `71DA10` and `4199FC..419A31` on the same supplied four-record group. Receipt SHA-256 `0f37d297c08b43869cadf129e9ec0a32f40dfc5e671fac4fc63f570f0cf6b98f`; 29,008-byte stdout SHA-256 `d5f3b85a56abce7b5b8e5bc43e4a1e36f58deea6fc34a09d3646b7d9bc9ca18a`. The source scan finds the first descriptor whose fourth header word is 5 and returns its count, or zero when absent. With supplied accumulator values 0, 118 and 119 and the category count 10, the sums are 10, 128 and 129; the original signed greater-than-128 branch is taken only for 129. A fourth control temporarily removes the matching category and yields zero. The source branch is relocated to a checked local capture; the enclosing caller and actual task/provider are not executed.

These category counts and the caller limit are not proven palette IDs or palette binding. Group ownership, allocation, lifecycle, renderer, shader and live identity remain unverified. Canaries are checked after execution.

### Conditional synthetic-palette CPU deformation

The independently reviewed, externally pinned `target/creature-skin-deform-position-prep-v1-20261109` host matched original `7314A0` for all 85 supplied vertices. The full 29,984-byte stdout SHA-256 is `38679f80f7ccbd81a2260c2ec20a2803317edff4b29cb72c3be8e307eeb9e0ba`; receipt SHA-256 `321e8f0083b3790ce0c075c10b60cd5d9883d4202b88ee9a5f4f88a133f138c3`; the 1,020-byte deformed-position tail/model SHA-256 is `4a1c40309c82d1a9d100f7d9414d88e92718e83268d6ab29342ea2767bf568c9`. The comparison uses the previously matched position, u16 register-offset and f32 weight streams, four supplied lanes, ten explicitly synthetic translated identity matrices (palette SHA-256 `5a946c16d7ab0dad7bf9cae07d79b4722d43da04b56e213f6b8bc0764b522f64`), disabled direction descriptors, guarded preallocated destination and supplied CW/MXCSR state. All 340 offsets were checked for alignment and range before source entry; FP order, x87 control and stack status were audited for this corpus.

The first external pre-pin command used a nonexistent guessed path and stopped at `Get-FileHash` before execution. The successful run used the separately pinned, independently reviewed exact path; the initial path error did not run a host.

This is a controlled CPU position comparison using synthetic matrices. It does not determine a live pose, actual palette or world identity, direction deformation, shader/GPU binding, final mesh state or gameplay. The retained group, four-record association and allocation/lifetimes remain conditional; canaries are post-run checks.

Use `python tools/creature_skin_0473_partial_export.py --include-synthetic-cpu-deformation --output <new-path>` for the separate controlled CPU position result. The option includes same-object prerequisites, verifies the matched native and selector receipt/stream prefixes and pinned 480-byte synthetic palette, checks all 340 register offsets and finite values, and recomputes all 85 positions using the source binary32 operation order. The fixed inspection artifact `target/creature-skin-synthetic-deformation-inspection-20261109.json` has SHA-256 `09884d6953894b4fcc6ef5a25bba8e8bab314287889cb41de9f053ca1ae0127f`. Default output is unchanged. The matrices are synthetic, only positions are deformed, and normals/tangents, actual palette/pose/world, GPU binding and live identity remain unverified.

### Offscreen synthetic-pose wireframe inspection

The opt-in Bevy command `target/debug/rustspore.exe --experimental --synthetic-0473-preview target/creature-skin-synthetic-deformation-inspection-20261109.json --shot target/creature-skin-synthetic-preview-result-v1/preview.png` accepts only the SHA-bound inspection artifact, validates the schema, 85 finite deformed positions and 252 in-range indices, and draws a position-only wireframe with a persistent synthetic/not-live label. It requires a new PNG under workspace `target`; it does not fabricate deformed normals, tangents, materials or texture bindings. Focused importer tests passed 2/2 and source review found no blocker.

The externally coordinated hidden offscreen capture completed with exit 0 using RTX 3060 Vulkan. Receipt `target/creature-skin-synthetic-preview-result-v1/receipt.json` SHA-256 `bacf16acace3a3b4c1d092f3c9264e18fcd2f6e9d6dab4bb06a3cf9cf24b1819`; `preview.png` SHA-256 `fbafbea22f72450c18003648e63090630c929de4e99d1ce0d0a05efc3774ef06`; executable SHA-256 `056ae77b3fbbcf0f1904a68c6d992814faf3c58ed51a9af0092a52b4bdb1ed7f`. Visual inspection showed the synthetic/not-live label and a small cyan partial wireframe. Logs contain loader error-13 informational and missing validation-layer warnings, with no observed Bevy pipeline failure. This is a single synthetic-matrix, position-only inspection frame, not an animated or complete creature, live pose, source renderer or gameplay visual.

### Synthetic direction deformation and separate inspection mode

The independently reviewed original `7314A0` direction host completed with a matched receipt at `target/creature-skin-deform-directions-prep-v2-20261109/result.json` (SHA-256 `6050374927dd93aea54e883c1ce726f584d2c73efebfcaab1719a69981d53c89`); its 32,024-byte stream SHA-256 is `09b8331e22b2b5318cf9a7d8defcd35d04ac89f64cd7d23f89940cb86dc11294`. It covers 85 normal and 85 tangent outputs under ten supplied synthetic matrices and the retained conditional streams. The host uses x87 control `027F`, selecting 53-bit significand precision. The original 64-bit Fraction oracle produced the same final words on this fixture but did not model that control setting. A separately retained 53-bit oracle replay matched all 170 retained direction vectors (`target/creature-skin-directions-cw027f-audit-v1/receipt.json`); this is a bounded finite-input comparison, not general x87 equivalence or evidence of a live palette, pose, material, rendering, or lifecycle.

`python tools/creature_synthetic_directions_export.py target/creature-skin-synthetic-directions-inspection-20261109.json` creates a new hash-bound artifact (SHA-256 `3393ad82bdb8b3ed8cf556bad192d49a0d8a28dc84e4759725b4ba33d055bd7f`). The separate experimental mode `--experimental --synthetic-0473-directions <artifact> --shot <new target PNG>` displays the conditional position wireframe plus sampled normal and tangent glyphs, explicitly labeling the synthetic matrices and absence of a live pose/material. Existing position-only output and mode are unchanged. A capture is required to verify visual presentation.

The reviewed hidden capture completed on RTX 3060 Vulkan: `target/creature-skin-synthetic-directions-preview-result-v1/receipt.json` SHA-256 `46f438ef95abd22bbc4229f928aa3851a4ae7e9ad6b4aaa0c7e680e2e935dc85`, PNG SHA-256 `27a4bc80afeb00895cec093f4085b2a9ae3220478a641f0511dcd7def66363ae`. The captured frame visibly contains the cyan wireframe, sampled green normal and orange tangent glyphs, and synthetic/not-live label. No matching Bevy error markers were observed. This remains a small partial diagnostic, not live animation or material.

The first half-turn Bevy capture attempt produced a PNG and exited 0, but its capture gate failed because default log filtering omitted the required AdapterInfo line; therefore no adapter or accepted capture claim is made from that attempt. The private desktop was reaped and closed. Its retained receipt `target/creature-skin-synthetic-halfturn-preview-result-v1/receipt.json` SHA-256 `bbf2b9d8ac02f6f0456ee5ae7210abdf9ed729139bf8b315075880a6690a7357`; PNG SHA-256 `d55153865702b1f46aacb871e4cfef25651f274c91b21e517bb77546829b39a0`. A separate revised capture runner explicitly requests the RTX adapter/Vulkan and enables renderer INFO logging; it requires independent review and a new output directory.

The separately pinned v3 runner enabled renderer INFO and explicitly selected RTX 3060/Vulkan, then captured successfully in a new private desktop with exit 0 and reap/close verified. Receipt `target/creature-skin-synthetic-halfturn-preview-result-v3/receipt.json` SHA-256 `af231162e826abbacaa1ec79ec8a39d26bf6482b3a5bbf486777cdce86c6a8e7`; PNG SHA-256 `d55153865702b1f46aacb871e4cfef25651f274c91b21e517bb77546829b39a0`. The immutable receipt says visual inspection pending; subsequent independent inspection by two reviewers showed the cyan partial wireframe, green normal/orange tangent glyphs and synthetic/not-live label. The v1 output is pixel-identical but its adapter gate remains failed. This is a fixed synthetic diagnostic, not live pose, material, animation or a full creature.

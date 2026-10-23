# Original Cell part geometry and capability-cache construction

## Current boundary

`[MEASURED]` The original `E83E60` geometry routine has a bounded, reviewed
original-byte conformance host and a complete source-row corpus. The Rust port
in `crates/spore-core/src/cell_part_geometry.rs` matches all 145,356 native
cases at every reply byte; its two focused tests also pass. The `E51EC0`
skip-flag geometry/count/index reduction also matches 28,398 complete native
cache tails across 12 explicit arithmetic contexts. The later scalar/flag/mask
path now matches 33,480 original-byte cases, with explicit source curves,
Globals, mutable identifier and arithmetic policy. Installed-package binding
and actor integration remain separate gates. A second reviewed original-byte host covers `E827A0` and
ordinary-list parent presence over 75,938 cases, including every source parent
candidate path. The earlier geometry fixtures retain their explicit classes;
they are not relabeled as a complete class-to-actor pipeline.

All source bytes are read from the immutable installed game. Raw resources,
machine-code ranges, host binaries and fixture streams stay under ignored
`target/`. No retail game process, initializer, window, desktop automation or
network service was run for this investigation.

## Source inventory

`tools/cell_capability_cache_probe.py` reads all 82 installed package indexes,
validates the original 128-byte CreatureData compressed-header framing, and
requires the declared arrays to consume the entire resource. The fresh v2
inventory contains:

| Denominator | Measured count |
|---|---:|
| Creature resource copies | 1,365 |
| All source 140-byte rows | 46,418 |
| Creatures with more than 64 rows | 73 |
| Rows omitted by original first-64 limit | 754 |
| Rows consumed by original Cell geometry | 45,664 |
| Distinct raw part identities | 654 |
| Direct part PROP copies | 1,274 |
| Part identities with a Parent reference | 654 |
| Parse failures | 0 |

The retained report is
`target/source-cell/capability-cache-census-v2/inventory.json`; complete raw row
records are in its `source-rows.jsonl`. The legacy greedy compound-property
walker failed on two copies of `C75C961B` at byte 252 in v1. V2 uses a declared,
count-bounded named-list walk that chooses a compound extent only when the
whole list consumes exactly. It retains raw fields and permits no anonymous
field recovery. V1's two failures remain historical evidence, not exclusions
from the v2 denominator.

Only nine distinct direct part definitions carry one of the classifier tags;
those fields are INT32 type `0009`, value one, rather than BOOL values. The
complete parent graph below supplies the ordered native presence predicates.
Neither a direct-only census nor BOOL conversion supplies the source class.

## Concrete native lookup chain

`[MEASURED]` `401010` reads global `15D1C04`; setter `4010C0` receives the
object created by `417CA0` at initialization callsite `5D7D68`. The factory
allocates `1248` bytes and invokes `403230`. The complete constructor body is
`403230..4035BC` (end exclusive), 908 bytes, SHA-256
`481412a2e4f619911fbb394fa848856b3155ee9bee895f9ad5be474095219f4d`.
Its final primary vtable is `13EB308`, whose slot `58` is `4155F0`.

`4155F0` combines its two supplied words into a cache key. A missing reference
calls the property-manager getter `67D950`, then its virtual slot `2C`, passing
the same two words and a reference output slot. A successful reference is
retained in that cache. Native dossiers 557 and 558 confirm the lookup and
constructor; their bounded source assembly is retained alongside the evidence.

The real property-manager getter loads `15FE8A8`. Setter `67DB60` receives the
`6AB150` allocation result at `7E66BB`, inside defined initialization function
`7E64E0`. `6AB150` allocates `1EC` bytes and calls `6AAED0`. That constructor's
final primary vtable is `1409190`; slot `2C` is `6A5B70`. Dossiers 568, 575 and
576 establish this second lookup chain.

`6A5B70` constructs the PROP resource key, uses the resource provider at manager
`18`, and invokes `6A3890` after successful loading. A bounded source decode of
`6A3890..6A39B1` shows the Parent path: test property `00B2CCCB` through the
returned list's virtual `1C`, obtain it through virtual `28`, and use its first
and third key words for recursive manager `2C` lookup. If the third word is
zero, the current list's word `10` supplies that value. The middle key word
does not choose this recursive lookup's resource type. A successful result is
installed through `6A11C0`, whose complete bounded body retains/releases the
parent reference at list `30` and updates the Parent property.

`[MEASURED]` Bounded manager initialization `6A9640..6A9BA1` registers the
manager's secondary interface at object `04` with the resource provider.
Its constructor-established vtable `1409158` has factory slot `1C = 6A2DE0`.
That producer allocates a `38`-byte list via `6A1640`, whose final vtable is
`14087C8`, then reads the resource through factory slot `24 = 6AAC20`.
The ordinary list's virtual `3C = 6A2A10` reads named 24-byte runtime entries
without replacing that list's vtable. The binary header can separately encode
an inline parent with its high count bit; the counted source census supports
ordinary positive-count lists and reports unsupported framing explicitly.

The ordinary list's virtual `1C = 6A1F20` calls `612410` for an unsigned
lower-bound search over the stored entry identifiers. A found identifier returns
one without reading its numeric or BOOL payload. Otherwise it tail-calls
parent `30`'s same virtual `1C`, or returns zero when no parent is bound.
The corresponding getter `28 = 6A1F80` follows the same local/parent path.
Dossiers 760..765 were admitted only after each actual procedure address
matched its requested source address. The static global at `15FE918` uses a
different indexed-list vtable `1408818`; it is not mistaken for the ordinary
part list returned by the factory.

These parent and predicate operations explain why a direct tag census cannot
classify the source parts. The complete source parent graph and all candidate
copies are measured below; actual package-priority binding remains a separate
operation. Numeric truthiness is not used for the native presence predicate.
The SDK reports no defined containing procedure at `5D7D68`; the initialization
callsite and factory chain were followed from bounded source instructions,
without assigning an unrelated containing function.

## Ordered classifier

`E827A0` obtains the part list through the first manager's slot `58` and tests
the low return byte of list virtual `1C` in this exact order:

| Property identifier | First-true class |
|---|---:|
| `100F0F5A` | 12 |
| `11B79A74` | 2 |
| `11B79A70` | 3 |
| `032F92E6` | 9 |
| `11B79A72` | 10 |
| `11B79A71` | 7 |
| `11B79A78` | 4 |
| `11B79A75` | 6 |
| `11B79A76` | 5 |
| `11B79A77` | 8 |
| `11B79A73` | 11 |

A missing list or all-false results returns class one. These literal class
numbers are not renamed from visual guesses or raw property values.

## Complete source parent and classification census

`tools/cell_part_classification_probe.py` follows every matching installed copy
of every reachable Parent key. It preserves the native lookup pair as
`(low, scope)`, uses the original unsigned lower-bound predicate, and computes
the union of inherited presence identifiers before applying first-true order.
It does not choose an archive-priority winner. The original native index reader
separately establishes the canonical resource identity `(low, type, scope)`;
the earlier tool field called `group` is retained only as a legacy label and
does not supply this Parent scope.

All 82 installed indexes contribute to discovery. The 654 part identities
reach 719 PROP keys and 1,411 raw PROP copies in three packages. Every copy
parses as a complete declared named list, every identifier list is strictly
unsigned sorted and unique, the maximum list has 31 fields, and the longest
parent chain has seven lists. There are no parse failures, missing parent
keys, parent cycles or unsupported Parent fields in this corpus. Parent-key
third-word zero falls back to the current list's scope exactly as the source
binding routine specifies.

Every candidate path produces the same effective tag mask and class for each
of the 654 source part identities. The agreed literal classes are:

| Class | Part identities |
|---|---:|
| 1 | 600 |
| 2 | 4 |
| 3 | 2 |
| 4 | 1 |
| 5 | 3 |
| 6 | 3 |
| 7 | 6 |
| 8 | 6 |
| 9 | 5 |
| 10 | 4 |
| 11 | 17 |
| 12 | 3 |

The retained census is
`target/source-cell/part-classification-census-v1/report.json`. Candidate
agreement removes package precedence from this bounded class result; it does
not establish the live loaded package set or replace actual manager binding.

## Bounded original class/list fixture

The new host executes only `E827A0..E828E1`, `6A1F20..6A1F79`, and
`612410..61245D`. Its owned manager returns one explicitly supplied root list
or a missing reference. Up to eight guarded lists contain up to 32 identifier
entries each, with forward-only parent indices. Fixed read-only vtables and
checked callbacks validate manager keys, original first-true query order,
parent receiver progression, identifier counts and call limits. The original
list predicate and binary search execute unchanged. No request supplies a
pointer, code target or property payload; native presence does not read a
property value.

The fixed-address native list spans and seven-byte owned manager getter are
linked in RX host sections. The original `E827A0` page is copied to an owned,
INT3-filled fixed allocation. Input, part object, nodes, entries and replies
have separate canaries. The host checks exact post-CALL ESP before restoration,
unchanged supplied x87 CW, empty TOP, and MXCSR control `1F80`; only its six
sticky exception flags may vary. Seven Kernel32 imports provide bounded file
pipes, memory allocation, instruction-cache flush and exit. It runs no archive
reader, game initializer, retail module/process, window or network code.

Reviewed artifacts are in
`target/source-cell/native-part-classification-host-v1`:

| Artifact | SHA-256 |
|---|---|
| Host assembly | `5d4435e2d34f54416dfa157ee5ab48cf55247c026a945ecac06cefa3b4f78b8a` |
| Host EXE | `624e29e203771827d764739cc5fe0652b78d686dd08e8f10bb576f9c50f871a3` |

`tools/cell_part_classification_native_run.py` completed 75,938 hidden native
cases with zero exit and exact reply count:

| Lane | Cases |
|---|---:|
| Every candidate source parent path | 23,306 |
| All 2,048 tag masks, local, twelve CW policies | 24,576 |
| All 2,048 tag masks, inherited, twelve CW policies | 24,576 |
| Missing-list cases | 12 |
| All field counts zero through 32 | 396 |
| Finite unsigned random/unsorted/duplicate identifier graphs | 3,072 |
| **Total** | **75,938** |

All 1,411 raw source copies were hash-checked against the census during
preparation and independently rechecked after native execution. The source
paths cover every reachable key and every candidate copy, using explicit
`027F`; synthetic paths cover all twelve x87 precision/rounding policies.
Ten protocol checks include fresh empty EOF, partial records, a complete record
followed by a partial one, excessive nodes/fields, self/out-of-range parents,
reserved precision and unmasked exceptions. All call-order, parent, buffer,
ESP and floating-context checks passed. An earlier slow extraction was
interrupted before any native execution and remains in
`part-classification-fixture-v1-preparation-partial` as preparation-only
evidence; the accepted cohort was regenerated with cached package indexes.

Accepted streams are under
`target/source-cell/part-classification-fixture-v1`:

| Artifact | SHA-256 |
|---|---|
| `input.bin` | `5e793c89519d228a84ac71c4bbf6845805449027572614c50ad0b830c5cba91f` |
| Native `output.bin` | `c47eeda476dd9a90e3e0c368c4c6917a2479ddd2ebb5603ef8a67567d7283854` |
| `properties.tsv` | `c7392dbaf0e69848be0a15cfea67bd1641de835b3ccd8ebb77567666c68d9ca1` |
| `source-chains.tsv` | `a57017304113ed859587e4014e89d22e936d3f600ce44fe51f2649a0b948cb59` |

The isolated Rust module `cell_part_classification.rs` retains original order,
unsorted-list binary-search behavior and reached-parent boundary errors. Its
independent example `cell_part_classification_verify.rs` passed all 75,938
complete class/manager/query/visit replies. It additionally reparsed every one
of the 1,411 original PROP copies through the core parser, required exact named
framing and identifiers, and checked Parent binding for all 23,306 candidate
chains. All 719 reachable keys have one agreed class and all twelve arithmetic
policies are represented. Native `output.bin` and `rust-output-v1.bin` match
all 1,215,008 bytes at SHA-256
`c47eeda476dd9a90e3e0c368c4c6917a2479ddd2ebb5603ef8a67567d7283854`.
The retained comparison is `rust-comparison-v1.json`. Both focused module tests
also pass in the parent-owned full workspace cohort.

## Exact geometry

`E83E60` consumes at most 64 original 140-byte rows. It writes 72-byte records:

| Output offset | Native value |
|---|---|
| `00` | Source word `06 == FFFF`, as byte zero or one |
| `01..03` | Unwritten caller padding |
| `04` | Explicit native classifier result |
| `08,0C` | Transformed center X/Y |
| `10` | Literal positive zero |
| `14..37` | Nine raw source matrix words from `30..53` |
| `38` | Radius |
| `3C..47` | Three raw source words from `60..6B` |

Each bound is multiplied by `2.5` before multiplying by source scale `2C`.
Centers use `(maximum + minimum) * 0.5`; half extents use
`(maximum - minimum) * 0.5`. The XY transform adds the Z and Y terms first,
then the X term, then translation multiplied by `2.5`, with separate scalar SSE
operations. Output Z is always literal positive zero on the ordinary path.

Radius selects the larger absolute X/Z half extent. Its two x87 `FABS`
load/store pairs are exact for finite binary32. Class two multiplies radius by
`0.5` unless the creation key's first word is `D9007204`; every other path
multiplies by one. The source's comparison and multiplication are retained,
including signed-zero/subnormal behavior under declared MXCSR control `1F80`.
The source creation key is distinct from the current part key.

Resource type `476A98C7` takes an early one-part branch: byte flag one, class
11, radius one, origin copied from globals `16B54E4..16B54EF`, nine matrix words
from `16B5534..16B5557`, and the same origin words copied to the output tail.
The Rust API takes those globals as explicit raw inputs; it invents no origin
or identity matrix for a live game.

`NativeCellPart::write_native` preserves the three unwritten padding bytes.
Unused matrix/tail components remain raw bits, including nonfinite encodings.
Ordinary arithmetic rejects nonfinite consumed inputs or intermediates at the
Rust API boundary; those errors are checked API behavior, not original-game
fallbacks. The first-64 limit precedes inspection of unused source rows and
their supplied classes.

## Bounded original-byte geometry fixture

Reviewed preparation tool: `tools/cell_part_geometry_probe.py`.
Reviewed host: `target/source-cell/native-part-geometry-host-v1`.

| Artifact | SHA-256 |
|---|---|
| Installed EXE | `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a` |
| Host assembly | `d708381148c7fb7d833fc84bfb1e96f15d5977853f887a875074a57f06d8abca` |
| Host EXE | `e17531352ba5a6e6e6929617cdc345c7289dbfd6d42b0b34da32b41339e8499e` |

The native code allowlist is only `E83E60..E8417A` (794 bytes). Read constants
are `1471B2C=3F000000`, `14861D0=3F800000`, and `1486BC4=40200000`.
The special branch's 12+36 bytes are supplied in bounded protocol fields at
the original global addresses. Every native direct branch remains inside the
allowlisted span, except calls to two owned providers:

- `4BC760`: a seven-byte absolute loader trampoline linked into the host's
  reserved `004B0000` RX image. It does not copy or execute the original loader.
- `E827A0`: a seven-byte absolute classifier trampoline into a checked owned
  callback. It returns the supplied class and does not execute or approximate
  property inheritance.

The loader validates the exact key pointer, zeroed native reference-stack-slot
address and once-only call. It exposes one bounded FakeCreature with a
read-only vtable. Classification validates the exact increasing source-row
address and the first-64 count. The sole native virtual release call therefore
targets the checked release callback. Neither fixed stub contains an outbound
relative branch. No protocol field can supply a pointer or callback address.

The 224-byte request contains source count `0..256`, explicit class, three key
words, one explicit x87 control word, one 140-byte source row, and raw special
globals. The owned source array repeats the row according to the supplied
count. The 160-byte reply contains count/provider counters and complete first
and last 72-byte output records. Input, creature object, source rows, output
parts and reply are separate guarded buffers. The host checks all three output
padding bytes per active row and every byte of the unwritten output suffix.

Only seven Kernel32 imports are linked: GetStdHandle, ReadFile, WriteFile,
VirtualAlloc, FlushInstructionCache, GetLastError and ExitProcess. Fresh-record
EOF accepts ERROR_BROKEN_PIPE only with zero transferred bytes; every partial
record and other ReadFile error fails. The host has no game initialization or
retail process dependencies.

`tools/cell_part_geometry_run.py` completed hidden native execution with exit
zero and exact record/reply counts:

| Lane | Cases |
|---|---:|
| Every source row, supplied class one | 45,664 |
| Every source row, supplied class two | 45,664 |
| Every source row, class two with `D9007204` exception | 45,664 |
| Synthetic classes/counts/flags | 4,608 |
| Synthetic radius exception branches | 180 |
| Opaque copied/unused words | 336 |
| Signed zero/subnormal bounds | 72 |
| Finite random geometry | 3,072 |
| Special raw-global branch | 96 |
| **Total** | **145,356** |

All 1,365 source resource copies were reloaded and checked against their census
hashes before these cases were written. Synthetic cases cover all 12 explicit
x87 precision/rounding combinations; source cases use `027F`. Each policy has
697 synthetic cases. There are zero consumed nonfinite arithmetic inputs, zero
nonfinite arithmetic outputs, and zero finite-case exclusions. Forty-eight
special cases deliberately copy nonfinite center words, and 4,767 cases retain
nonfinite opaque output words. Those copies are included, not discarded.

Nine protocol checks cover one valid empty input and eight malformed streams
or invalid count/class/control cases. All guarded boundaries, provider call
counts, padding and output suffix checks passed. The raw fixture directory is
`target/source-cell/part-geometry-fixture-v1`:

| Artifact | SHA-256 |
|---|---|
| `input.bin` | `bd03690553ee10ad423706503a9a52383653a53b1d7fef242e0771139d1b0a4b` |
| `output.bin` | `a5533b75c432cb3b9938ef0f68c7ea49dd839d04d5793ddcaf9d145817fbe0fb` |
| `source-map.tsv` | `538763cad985e883c4941e84dc3b95540ae652f9202778bb7d8aa711eba4a1bd` |

`crates/spore-core/examples/cell_part_geometry_verify.rs` passed all 145,356
cases: 145,260 ordinary, 96 special, and 3,059 count-clamped cases, across 12
explicit contexts. Both focused module tests passed. The independently written
`rust-output-v1.bin` matches all 23,256,960 native reply bytes at SHA-256
`a5533b75c432cb3b9938ef0f68c7ea49dd839d04d5793ddcaf9d145817fbe0fb`.
The retained comparison is `rust-comparison.json`. These are exact geometry
results with supplied classes, not original-classifier or actor agreement.

## Verified skip-flag cache reduction

`[MEASURED]` Dossier 449 contains complete `E51EC0`. The isolated Rust module
`cell_capability_geometry.rs` implements its skip-flag prefix. It consumes at
most 64 original 72-byte rows and clears thirteen class counters. A raw byte
equal to one counts twice and participates in the extent branches; every other
byte counts once. Extent updates retain the original `if maximum / else if
minimum` behavior, so one row cannot update both sides on that visit. Center
coordinates use the exact binary32 addition/half order. Class 12 selection uses
a strict initial Y limit of 10,000 and preserves the first equal-Y match.

The radius reduction retains x87 differences, `(dx² + dz²) + dy²`, square root,
and radius addition under an explicit arithmetic context. It commits a
candidate binary32 value while retaining the extended candidate, compares that
retained value to the current binary32 radius, then chooses the committed
candidate only when greater. A binary32 square-root shortcut or comparison of
the already rounded candidate changes this boundary.

The ordered class-five index slots retain the last row in each source branch.
The non-flagged Y limit is inclusive 1,000. If neither non-flagged slot is
present, a positive-Y flagged row takes precedence over a nonpositive-Y flagged
row. If either non-flagged slot exists, the positive-Y flagged slot is followed
by positive-X then nonpositive-X slots, omitting the negative-Y flagged slot.
Stack `FFFFFFFF` values are signed integer index sentinels, not NaN floats.
The integer statistics retain literal class-counter divisions, the last
negative-Y flagged class gate, and maximum ten. Unused index slots and all later
capability bytes survive `write_native_tail` unchanged.

The original-byte host has two source spans only: `E51EC0..E524A4` and
`E5271F..E52726`. Its caller supplies literal skip flag one. The conditional
transfer at `E5249E` therefore goes directly to the copied epilogue; every later
manager/curve instruction remains an uncopied `INT3` region. The sole direct
call, `E51ED3 -> 11E052E`, targets an owned absolute trampoline to a checked
52-byte zero fill. That callback verifies the exact native return address,
destination, value, size and once-only count before clearing thirteen words.
There are no indirect native calls.

Host v1 was prepared but never executed. Review added explicit post-call guards
in v2: ESP must equal its saved immediate pre-CALL value before restoration,
FNSTCW must equal the supplied control word, x87 TOP must remain zero, and
MXCSR control must remain `1F80` while its six sticky exception flags may vary.
The original code/data allowlists, seven imports and protocol stayed unchanged.
Input, cache and reply are bounded guarded buffers; all cache bytes below
`1204` are also checked unchanged. Active class words are restricted to `0..12`
before native entry, independently of ordinary `E827A0` returning `1..12`.

The reviewed host is `target/source-cell/native-capability-geometry-host-v2`:

| Artifact | SHA-256 |
|---|---|
| Host assembly | `44dbd9fddf470a2fbf7a9752c284f72e19dcbdc349c397afcef1e62c79334232` |
| Host EXE | `7eaa8d9d868327601a55f37da9d34d1791b1d12836b8d98bd700623a4b8e463c` |

`tools/cell_capability_geometry_run.py` streamed 28,398 finite-domain cases:
19,110 source-derived cases (1,365 Creature copies under fourteen explicit
class-assignment modes using all 45,664 verified geometry rows) and 9,288
synthetic cases. Source-derived cases use `027F`; synthetic cases cover all
twelve policies with 774 cases each. All 65 input counts, raw flag values,
class-zero memory boundary, selection permutations, strict/inclusive limits,
integer-statistic gates, signed zeros, subnormals and retained radius math are
covered. Nine protocol checks include one valid empty stream and eight invalid
streams or count/class/control inputs. Native exit is zero, guards and FP/stack
postconditions passed, and no finite case was excluded.

`cell_capability_geometry_verify.rs` matched every byte of all 28,398 complete
144-byte replies, including unchanged capability bytes and unused indices.
Native `output.bin` and independent `rust-output-v1.bin` both contain 4,089,312
bytes with SHA-256
`77b8109e8e7ce392cff2879d7d4762882e1c466b7fa6bc09972f26a366829064`.
The fixture directory is `target/source-cell/capability-geometry-fixture-v2`;
its `rust-comparison.json` retains counts and hashes.

Five additional source-input policy cases are retained separately, rather than
silently dropped from finite agreement: NaN center X, infinite center Y,
infinite radius, NaN empty-center X, and finite extent overflow. The native host
completed all five with its guards intact. The Rust checked boundary positively
rejects the first four as `NonFiniteInput` and overflow as
`NonFiniteIntermediate`; `rust-nonfinite-v1.tsv` records the native output bits
beside each typed rejection. Its SHA-256 is
`e35c73a886d289a9d852e80ddf0357711870b7d03eec3f19752495e91dc59a90`.

## Complete source classification-to-cache composition

`[MEASURED]` The accepted composition under
`target/source-cell/capability-source-composition-v1` connects the independently
verified class resolver to all 1,365 complete source creations. Preparation
reloads and hashes each original Creature payload, uses the agreed class from
every candidate Parent path, and supplies every original first-64 row to the
unchanged reviewed geometry host. That host's 45,664 replies match the earlier
independent class-one/class-two radius branch, with the exact source class word
retained. Its actual 72-byte parts then feed the unchanged reviewed skip-flag
cache host, producing 1,365 complete cache replies.

The fresh native runs complete with zero exit. Their full source/host ASM,
EXE, manifest, raw source copies and prepared inputs are hash-pinned before and
after execution. Exit/size/error records are saved before any acceptance
assertion; partial or failed outputs are retained as failures. The source
context is explicit x87 `027F`, MXCSR control `1F80`, and positive-zero
empty-origin words. Those environment inputs are not relabeled as measured
live actor globals.

The independent Rust example `cell_capability_source_verify.rs` reparses all
1,411 PROP copies and reclassifies all 23,306 Parent paths, then parses all
1,365 full Creature resources through the core parser. It requires exact
consumption, original per-part capability/morph ranges and raw part identities,
and checks each native request against the original 140-byte row and resolved
class. It derives and compares every geometry byte and every complete cache
reply. All 46,418 raw rows are accounted for: 45,664 are consumed, 754 are
omitted by the original first-64 limit in 73 creations. All 654 part identities
are covered.

| Native/Rust artifact | Cases | Bytes | Matching SHA-256 |
|---|---:|---:|---|
| Geometry | 45,664 | 7,306,240 | `dd1a4f372aa578f4f1d6261a333acc77fb63dcf7de9d8fa6fc1b508b7adbfc86` |
| Cache prefix | 1,365 | 196,560 | `a2a9964d1a5532970ed4a68106e472ac53329e3afdb57c51b9e37f4d9849e549` |

`rust-comparison-v1.json` retains this source-to-typed-parser-to-native-result
agreement. The classifier example was also rerun after its source loaders were
factored into `examples/support/cell_classification_fixture.rs`; all 75,938
complete replies still match the original classifier SHA. Active products,
archive-priority selection, live manager/reference lifetime and actor
integration remain separate boundaries. The later scalar fields are verified
under the explicitly supplied source inputs below.

## Scalar-cache construction and runtime boundary

`[MEASURED]` The isolated `cell_capability_scalar.rs` follows the complete
native `E524A4..E52726` path after geometry/count reduction. It preserves these
literal writes and source curve calls:

| Native cache field | Source behavior |
|---|---|
| Byte `1248` | Class-seven count greater than zero |
| Float `124C` | Curve `77540C34` at class-eleven count |
| Byte `1250` | Class-two or class-four count greater than zero |
| Byte `1251` | Class-three or class-four count greater than zero |
| Word `1254` | Propulsion mask: class ten bit4, class eight bit2, class nine bit1; zero becomes one |
| Float `1258` | Ordered propulsion curve branch below; no-part mask uses source Globals `90` |
| Float `125C` | Explicit mutable curve ID at class-six count |
| Float `1260` | Curve `E5015D0C` at class-six count |
| Float `1264` | Literal positive zero |
| Float `1268` | Curve `03AB589E` at class-six count |
| Signed word `126C` | Curve `AEE2835E` at class-six count, binary32 commit then SSE nearest-even `CVTSS2SI` |
| Float `1270` | Curve `92BAAE19` at class-five count |

The speed branch chooses class ten first. Its query is
`(count9 * Globals78 + count8 * Globals74) + count10`, with each scalar SSE
operation committed separately, then curve `2C574746`. Otherwise nonzero class
nine uses `count8 * Globals80 + count9` and curve `BBAB1BF5`. Otherwise nonzero
class eight still performs the native `Globals7C * +0 + count8` before curve
`2EF68471`. With no propulsion class it writes zero, derives a zero mask, then
replaces both mask and speed with one and Globals `90`. All other curve calls
retain their native order. The three bytes after `1248`, two after `1251`,
geometry/counts, indices and statistics remain unchanged.

`CellCapabilityScalarContext` explicitly supplies both x87 precision/rounding
and `native_sse::Policy::Mxcsr1f80`. The generic scoped SSE guard installs that
policy and restores the caller's complete MXCSR, including its incoming sticky
flags, on success, checked errors and unwinding. Source arrays are supplied
through the original curve callback; no part names or invented scalar formulas
replace a source lookup. Native first-64 counts are restricted to `0..128` in
this bounded API. Nonfinite consumed values/intermediates and out-of-domain
integer conversion return typed boundary errors; these are not original game
fallback values.

## Full original scalar fixture

`tools/cell_capability_scalar_probe.py` prepares a separate reviewed host at
`target/source-cell/native-capability-scalar-host-v1`. Its only original code
is the complete scalar span above, complete `E837F0..E8393C`, and epsilon helper
`1041E40..1041E57`. The original two float constants are copied exactly. The
mutable ID is explicit, pinned by the first admitted record, and its entire
page is read-only before native entry; attempted changes reject with exit8.

The owned entry wrapper reproduces the native frame and EBX-one/ESI-cache
contract, with both original argument slots reserved. Fixed RX absolute stubs
reach checked Globals-reference constructors/getters/releases and the checked
property provider. They validate exact source return sites, both native local
reference slots, null/fixed reference values and balanced reference counts.
Curve callbacks validate the scalar callsite and literal identifier, exact
local reference address and maximum seven calls. Immutable vtables can only
reach the checked manager/property/release functions. Points come from one of
nine bounded source slots, as at most sixteen Vector2 rows or one inline row;
no request supplies a pointer or code address.

Native code pages become RX and constant pages become R. Input, cache, Globals,
property, reference and reply buffers have separate 16-byte canaries. Full
input and Globals shadows, every cache prefix/unwritten tail byte, exact ESP
before restoration, supplied CW, empty TOP and MXCSR control `1F80` are checked.
All memory-boundary checks precede FP context stores. The host imports only
eight Kernel32 file/memory/cache/exit functions; it loads no original module
and starts no retail process, game initializer, window or network code.

| Artifact | SHA-256 |
|---|---|
| Host assembly | `aa2571c9efc37856b62757537a60cbaf45df5540b1e7ecc2784e8bde6950ffda` |
| Host EXE | `9f473c6c13b42214b970bbee6042153edd2a5e8be33d760720a0b2e1988d5111` |

The reviewed `cell_capability_scalar_run.py` corpus has 33,480 finite cases:

| Lane | Cases |
|---|---:|
| Every creation's actual-class native counts × two current Globals copies × two source PROP copies | 5,460 |
| Every class with every count zero through 128 | 20,124 |
| Propulsion/diet/electric branch product | 5,376 |
| Mixed finite counts and weights | 1,536 |
| Weight sign, signed zero and mask branches | 768 |
| Integer conversion ties and limits | 120 |
| Empty/inline/array/missing-list branches | 96 |
| **Total** | **33,480** |

Source cases use explicit `027F`; the 28,020 synthetic cases cover all twelve
x87 policies, 2,335 per policy. The two current 276-byte Globals copies are
decoded independently; the older 264-byte installed copy remains an explicit
schema exclusion. Both original capability PROP copies are reparsed as eighteen
named Vector2 arrays and 104 points each, including all nine curves reached
by this consumer. Sixteen protocol checks, a fresh-process alternate explicit
identifier, exact reply extents, zero native exit and all guards pass. There
are zero nonfinite scalar output words or finite agreement exclusions.

`cell_capability_scalar_verify.rs` passes all 33,480 native replies and binds
every one of the 5,460 source requests back to the actual creation count tail,
typed Globals fields and exact typed PROP points. Each case is repeated under
three incoming caller MXCSR states (`1F80`, `DF81`, `BF84`); every modeled bit
matches and every original caller control/sticky bit is restored.

The native reply contains 232 modeled bytes and eight bytes of original
MXCSR/CW/SW metadata. The latter are independently validated, excluded from
the modeled projection, and never copied into a false Rust output claim. The
232-byte projection includes every byte of the 140-byte cache tail, exact
curve ID/query bits, unused trace padding, provider counts and reserved bytes.

| Accepted artifact | SHA-256 |
|---|---|
| `input.bin` | `57495003b2988c7e20ee6b22554333c7b06f2099a2b7c5e0ee66e39dd1a0e11d` |
| Native 240-byte replies | `b4fb59ab2c03527ef54ef4a9fb4f968ed775c9eb8871985a15c28d5f98a6e740` |
| Native/Rust matching 232-byte projection | `e246a6ecd62c053b19fa9466aea4f6bffaac2a67228f492690a1b77dc4c900c9` |
| Separate native checked-policy replies | `4ff9fbe422024227457e4456f4fc5de64630aa0e3c2c6955693a61c63f40c7ab` |
| Positive Rust checked-policy report | `843c11de8cadc586123e4f5d8f2d4f8aa7854c72639807b2e4f02e24156f7ed9` |

Eight additional policy cases retain native outcomes separately: consumed NaN
or infinite weights, finite multiplication overflow, NaN curve x/y, a finite
curve result outside signed-integer conversion range, finite curve-difference
overflow, and NaN no-part speed. All complete natively with guards intact and
are positively rejected by the corresponding checked Rust error under all
three caller MXCSR states. None is silently dropped. Accepted evidence and
`rust-comparison-v1.json` are under
`target/source-cell/capability-scalar-fixture-v1`.

## Live consumer and identifier binding

The scalar branches combine actual class counts, literal CellGlobals
weights and the verified curves documented in
[cell-capability-curves.md](cell-capability-curves.md). The speed mask uses
count words `122C -> bit4`, `1224 -> bit2`, and `1228 -> bit1`. A zero mask
becomes one and uses the original no-part-speed global. One curve identifier is
read from mutable global `16B4C64`. A pinned-byte reference scan finds its one
static writer at `137EB64` and consumer at `E52683`. The complete aligned
initializer `137EB50..137EB6A` hashes the exact source string
`electric_recharge_vs_small_count` at `148663C`, using seed `811C9DC5` and mode
one, then stores EAX into that global. Bounded `932B30..932BDE` mode one calls
the original import `MSVCR90.dll!tolower` through `13CC2EC`, multiplies by
`01000193`, then XORs each mapped byte. The lowercase ASCII source string
therefore gives static identifier `5A38C4BA`, present in both actual capability
PROP copies and the verified curve corpus. Exact byte/hash/import evidence is
retained in `target/source-cell/capability-scalar-id-producer-v1.json`.
Ghidra currently reports the initializer undefined; the aligned, RET-terminated
source body is retained without assigning another function's decompilation.
The computed static initializer does not establish live initialization order
or an unmodified per-entry value, so the identifier remains an explicit input
for a bounded consumer until that runtime binding is established.

The actual steering bridge is visible in original `E6F620`: after target/orient
updates it calls `E67A50` with actor words `FC/100/104`, then passes returned
cache speed `1258` and mask `1254` directly to `E6EF30`, with literal multiplier
one and final argument zero. `E67CB0` instead selects source AI through
`E52770` and lets its nonzero `movement_style` override cache `1254`; a zero
actor `FC` returns literal one immediately. `E67D20` writes four literal ones
for zero `FC`, otherwise four copies of cache `1258`. Its complete getter has
no authored-AI speed branch. Source `E6FA00/E6FC20/E6FD50` separately pass the
selected AI's speed and turn-factor fields into steering. Exact original
dossiers and caller dataflow are retained before adding a source-facing
selector/getter port; live source/actor binding remains open.

The original floating policy can change after rendering initialization. The
full native policy evidence is in
[native-float-render-context.md](native-float-render-context.md). MXCSR `1F80`
fixture agreement does not establish equivalence under later retail FTZ/DAZ
state or prove the policy of a particular gameplay entry/thread.

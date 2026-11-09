# Continuation checkpoint — 2026-11-05

The target remains the complete original-fidelity game, including all five stages,
editors and campaign progression. No stage or editor has passed that acceptance gate.
The existing Cell and Creature scenes are experimental prototypes; the controlled
source-art and UI captures do not establish original gameplay or screen fidelity.
The preceding checkpoint and its broader validation are in
[`continuation-2026-11-04.md`](continuation-2026-11-04.md).

The latest completed full workspace checkpoint used
`tools\build.bat test --workspace --all-targets` and passed:
1092 passed, 0 failed and 16 ignored across 226 test groups. Its local log is
`target/workspace-tests-20261105-v17.log`, SHA-256
`bc745573cbb2db15064a1e1f6a07546d24f05dcff2edfb88c72871fc3efcb6ba`.
The opt-in installed-data tests were run separately; the
workspace run does not by itself prove source fidelity or application behavior.

## Source creature editing

`spore_core::creature_data_edit::patch_rest_transform` returns a copy of an exact
version 9 or 10 CreatureData payload with one serialized part row's absolute rest
rotation and translation replaced. It checks row bounds, source consumption and
finite inputs while preserving all other bytes, including the original compressed
header. Synthetic tests exercise both versions, multi-chunk headers, byte
preservation and rejection paths. An additional opt-in test passed against a
retained installed-data fixture with 10 parts, 15 capabilities and 24 morphs;
the fixture's SHA-256 before and after the test was
`dbbb55830f7f75eb5c45afd470a8bd0716cb08134e26f62a82674e28cd697137`.
The test log is `target/creature-data-edit-installed-20261105-v1.log`.
The test checks fixture structure; the endpoint hashes were checked separately and
do not establish that the file could not have changed transiently during the run.

The opt-in [`creaturedata_patch_translation` example](creaturedata-translation-example.md)
selects a unique resource from an explicitly supplied package, checks the payload
hash, and writes a new local file under `target`. A source-bound run changed one
translation and a separate file hash agreed with the intended buffer hash. The
command does not read its output back, so its reported hash is the buffer hash.
Package and output-directory stability remain caller requirements. These operations
do not regenerate the baked GMDL, establish valid editor placement, create a
shareable creation or integrate the edit into gameplay.

An [experimental offscreen assembly preview](experimental-source-part-assembly.md)
now resolves part models from the source packages and displays a hash-checked
edited payload. An installed-data geometry test resolved all 10 rows and verified
that 103 of 2,470 vertices moved by the selected row's translation delta while
the other vertices and indices stayed unchanged. The log is
`target/assembly-preview-geometry-test-20261105-v2.log`. The preview generated
`target/assembly-preview-edited-verified-20261105.png` on AMD Radeon Vulkan and
reported a completed, synced write; the existing-output and wrong-hash checks
also rejected their inputs. The frame is untextured and independently fitted,
so it is an assembly inspection rather than an original editor or comparison to
the source game's rendering.

A [source CreatureData editing session](source-creature-edit-session.md) now
supports exact-byte undo/redo and exclusive, readback-verified saves. It can
open a supplied payload or a uniquely indexed, hash-checked package resource.
Failed saves keep the session open for `save-as`; history stays in memory and
is bounded. Seven focused core tests passed, including a version-10 multi-chunk
fixture with nonempty owned arrays; an additional opt-in test on the retained
10-part installed payload passed edit/undo/redo with exact source restoration.
Ten focused CLI tests passed, including synthetic raw and RefPack packages,
duplicate-key and size rejection, and recovery from an existing output. These
tests do not regenerate a baked model or establish playable editor output.
The built CLI also opened `40616200A1C46DF2` directly from the local
`Spore_Content.package`, set row zero's translation to `(1, 2, 3)`, undid and
redid it, and saved a new file. The output readback SHA-256 was
`d8eaf29f289fc402347bfa99fc42643522fc3a914b8bee7d924c6a83ca6a520d`, matching
the previously geometry-checked edited payload. A wrong input hash was rejected
without creating output. An offscreen AMD Radeon Vulkan run of the saved bytes
resolved 10 parts and produced the same SHA-256
`6924ea5d287e8b6d00018278802e8883cf9efb101fde1465045cf8ae62782d09` as the
earlier untextured inspection frame. The CLI and capture logs are
`target/source-edit-session-installed-20261105-v1.log` and
`target/source-edit-session-render-20261105-v1.log`. Package hashes were
checked at endpoints; transient path changes remain an admission.

The CLI now lists source row ordinals, stored indices, prop instances, parent and
symmetric-part fields, and current translations in pages of 32. A relative
`move` operation joins the existing transform, undo and redo commands; a
nonfinite result is rejected atomically. These additions also passed the focused
CLI tests separately. A scripted run
on the retained 10-part installed payload listed its source rows, moved row 0,
undid and redid the edit, and saved with a successful readback. Independent
byte inspection found changes only in that row's 12 translation bytes. The
source and output SHA-256 values were respectively
`dbbb55830f7f75eb5c45afd470a8bd0716cb08134e26f62a82674e28cd697137` and
`84b8f9c0448dc2828a0803f73264171f5a8d06d0684b90243de9b3e0ffc1eb99`;
the local transcript is `target/source-edit-listed-moved-20261105-v1.log`.
The saved result also loaded all 10 source parts into an untextured, independently
fitted hidden assembly preview on AMD Radeon Vulkan. Its capture SHA-256 is
`9391934d4af40c55647324325ce0c6e3bea5327e1beaf7b0ef94e8eb95bb19b6`.
This does not validate attachment rules or produce a playable creation.

Static tracing also identified a generated skin-record producer downstream of
`4CC060`. A bounded Rust implementation of its final four-candidate arithmetic
matched 80 of 80 bytes from a reviewed original-instruction host on four fixed
synthetic cases. The source span was relocated unchanged and the outer-loop exit
was supplied by the host. Its preceding top-four weight selection now also
matches 32 original-instruction output words on four synthetic arrays, with a
checked fixed-capacity insertion substitute. A finite four-lane influence kernel
also matches four controlled original-instruction cases with a substituted
append sink; its spatial-query inputs are still caller-supplied. Separate
bounded models now cover the high-bit member-list and clear-high-bit edge
accumulation branches and match 12 and 20 native output words, respectively,
in controlled fixtures. A checked mixed fixture preserves supplied query order
and matches eight original-instruction words in two synthetic orders using a
host-supplied outer dispatch. Its source identity and original-runtime
composition remain open. Optional smoothing, row-to-palette
identity and baked geometry remain unresolved; see
[`creature-generated-skin-producer-static-v1.md`](../re/creature-generated-skin-producer-static-v1.md)
and [`creature-generated-skin-query-ranking-static-v1.md`](../re/creature-generated-skin-query-ranking-static-v1.md).

## Source art in the Cell prototype

An explicit `--source-rival-cell-id` option loads one selected installed Cell
model and skin for every visible prototype rival. A hidden capture using player
`7D00E384` and rival `EFE530A1` rendered on AMD Radeon Vulkan; its local PNG
SHA-256 is `cf553f0339575dd72e94844bb71dcdc505c08514eb387c6d43f7086e79807a7b`,
and its log records the resolved Cell, structure, model prop, model and skin
resources. The final capture includes a position-only label adjustment;
13 focused Cell frontend tests also passed. The
assignment, scale, orientation and prototype simulation are not
proven to match retail. The source models are static PBR views, and the broader
scene still contains prototype artwork and UI.

## Controlled source-player visual

`--source-player-visual` joins the staged startup calculation to one explicitly
admitted source candidate. It validates the selected PROP and checks the actual
GMDL and sampler-zero raster bytes used by the original-program renderer. The
other shader/material inputs are not pinned. With `--seed 1 --adapter amd`,
the local run produced `target/controlled-player-visual-amd-20261105-v1.png`
using a D3D9 HAL device reported as AMD Radeon with hardware vertex processing;
the capture log is `target/controlled-player-visual-amd-20261105-v1.log`.
The renderer used a fitted inspection camera and fresh lighting. No actor was
published, and the image has no source scene or animation. The other three
possible controlled creations remain unsupported by this visual command.

The default RTX-only run found no RTX adapter in this Windows D3D9 session and
failed without creating an image. Windows listed the RTX device with unknown
status, and `nvidia-smi` returned a permissions error. AMD is an explicit option
for this controlled path only; other existing native captures retain their RTX
requirement. Neither device result establishes retail rendering equivalence.

## Source UI and audio research

The controlled Creature frame fixture can now derive parent 966's dimensions from
the hash-bound authored SPUI rectangle. It still requires explicit admissions that
those dimensions survive the original runtime layout and that the caller-supplied
screen origin and identity translation apply. It does not recover an actual HUD
position. Four frame tests passed, including the selected installed layout and atlas;
the log is `target/creature-frame-authored-installed-20261105-v1.log`. See
[`creature-source-frame-overlay-v1.md`](../re/creature-source-frame-overlay-v1.md)
for the layout and rendering limitations.

For the candidate Cell collision event, an independent Python pass and the Rust
native index reader agree on both installed event variants and 40 sample rows. The
first-word hypothesis yields 20 distinct Audio1 keys per variant, each uniquely
matching a type `01A527DB` record. The arrays are identical while the event payloads
differ. This is a conditional key join: native sample-reference semantics, mounted
provider precedence, event dispatch and actual sound selection are not established.
The reproducible comparison and its endpoint-hash limits are described in
[`cell-audio-event-selection-lead-v1.md`](../re/cell-audio-event-selection-lead-v1.md);
the post-numeric-fix regression receipt is
`target/collision-audio-comparison-v5.json`.

A separately reviewed native instruction trace found a generic `samples`
consumer that constructs an SNR key from each 12-byte **in-memory** entry's
first word. The property loader and event path remain unjoined, so this supports
the candidate interpretation without proving the raw event bytes are selected
or played. See [`cell-audio-samples-consumer-static-v1.md`](../re/cell-audio-samples-consumer-static-v1.md).
A dedicated event reader and compatible property interface have also been
identified conditionally, but its provider identity and complete stream reads
remain unproven; see
[`cell-audio-event-loader-interface-static-v1.md`](../re/cell-audio-event-loader-interface-static-v1.md).
The same dedicated reader conditionally byte-swaps numeric types `0009`, `000A`
and `000D`; the Rust parser now decodes their scalar and stride-four array words
as big-endian. Unusual typed strides remain raw inspection data. Fourteen focused
tests passed, and the pinned collision event in both installed packages produced
gain `0.5` from source bytes `3f000000`. The log is
`target/collision-audio-gain-verify-20261105-v1.log`. Provider selection, complete
stream reads, event routing and playback remain separate gates.
Independent Rust and Python reads also agreed on numeric values across all
8,380 selected PatchData and 8,461 selected Audio2 events: 15,854 scalar fields
and 9,306 elements in 1,544 supported arrays. Neither package contained an
unsupported numeric array stride in this inspection. The reproducible receipt is
`target/audio-numeric-compare-20261105-v1.json`. This validates the specified
inspection walk and decoded bit stream, not original provider or playback
behavior; endpoint hashes do not establish continuous source or build provenance.

## Cell startup and movement

The executable also registers `02C21781` as a `cPlayer` factory and identifies
its conditional initialization target. Runtime lookup, interface identity and
the Cell actor connection are still open; see
[`cell-start-player-factory-static-v1.md`](../re/cell-start-player-factory-static-v1.md).
Separately, the Cell startup path conditionally allocates a pool slot and stores
its handle at manager `+411C`; a checked lookup validates the full handle.
The known transfer and clear paths, and the distinction from the simulator
`cPlayer` factory, are documented in
[`cell-current-player-handle-lifecycle-static-v1.md`](../re/cell-current-player-handle-lifecycle-static-v1.md).
The live actor and selected source creation are not yet joined.
The descriptor loader now has a traced conditional factory-registration and
fixup path; the selected mounted provider, cache result and PatchData precedence
remain unknown. See
[`cell-definition-loader-registration-static-v1.md`](../re/cell-definition-loader-registration-static-v1.md)
and
[`cell-start-definition-triple-publication-static-v1.md`](../re/cell-start-definition-triple-publication-static-v1.md).

An independently reviewed static trace now joins several conditional accumulator
paths to actor position fields `+4C..+54`. It distinguishes a direct update, an
indexed-part averaging path and a short-part path with optional flow; source
callback effects and the live player's branch remain unknown. See
[`cell-movement-accumulator-position-static-v1.md`](../re/cell-movement-accumulator-position-static-v1.md).
The normally initialized player path conditionally creates a nonzero component
before this update; its part count and indexed environment still depend on live
state. See [`cell-player-movement-component-lifecycle-static-v1.md`](../re/cell-player-movement-component-lifecycle-static-v1.md).
The separate no-handle short-part arithmetic has four passing unit tests,
including an explicit floating-point-control restoration check. It is not a
validated initialized-player tick or an original-instruction comparison.
The original environment's conditional field rebuild bins indexed vectors and
computes 300 per-bin outputs before later actor motion; the producer and its
unresolved state and callback boundaries are documented in
[`cell-flow-field-producer-static-v1.md`](../re/cell-flow-field-producer-static-v1.md).
`cell_flow_aggregation_v1` implements the finite-domain aggregation for supplied
post-callback bins. A reviewed, isolated host executed the original `E4DD50`
instructions for one fixed synthetic snapshot: all 1,500 output words matched
the Rust result, across 300 bins of which five contained 18 records in total.
This covers one controlled x87/MXCSR policy and does not establish a live field
or general native parity. The host receipt is
`target/field-aggregation-host-prep-20261105/reviewed-fixed-result-v1.json`;
the comparison log is `target/cell-flow-aggregation-compare-20261105-v1.log`.
The reviewed host uses a checked substitute for three zero-fill calls; input and
implementation endpoint hashes do not prove continuous path stability or full
compiler/build provenance. Model inputs and outputs are deliberately restricted
to finite, nonzero-denominator domains and do not model sticky flags.

`cell_flow_binning_v1` now constructs the earlier source-ordered 20-by-15 bin
snapshot from caller-supplied slots and frame values. In a separate reviewed
original-instruction host run on one fixed synthetic 1,500-slot fixture, all
300 bin counts and all 136 words of the 17 accepted records matched. Five bins
were populated, 1,480 inputs were outside the grid and three were dropped by
the eight-record capacity. The host receipt is
`target/field-binning-host-prep-20261105/reviewed-fixed-result-v1.json` and the
comparison log is `target/cell-flow-binning-compare-20261105-v1.log`. Unused
record storage was poisoned and was not compared as native output. This is one
synthetic fixture under explicit FP and stable-path admissions, not a live field.

The controlled `cell_flow_snapshot_v1` composition connects binning to
aggregation and exposes the vector field for the existing sampler. It requires
an explicit admission that the intervening force work preserved the copied bins;
it does not execute that work or construct the original environment. See
[`cell-flow-intervening-force-static-v1.md`](../re/cell-flow-intervening-force-static-v1.md)
for the nonaliasing, neighborhood-capacity and external-writer conditions.
An independently reviewed fixed host then ran the original binning and
aggregation instructions in sequence on a second 1,500-slot fixture; the Rust
composition matched all 300 counts, 136 accepted-record words and 1,500 field
words, with 17 records in five populated bins. Two accepted scalar inputs were
changed from signed subnormals to `+1` and `-1` to remain in the checked
aggregation domain. The host forced scratch read-only, skipped `E4E730` and
`E4E4C0`, and reset FP state between calls. The receipt is
`target/field-binning-aggregation-combined-host-prep-20261105/reviewed-fixed-result-v1.json`;
the comparison log is `target/cell-flow-combined-compare-20261105-v1.log`.
This is a controlled sequential comparison, not a reproduction of the full
native update or proof of a live player field.

The nonzero-handle short-part flow branch is also available as a bounded
supplied-sample arithmetic model. One reviewed original-instruction host ran a
fixed synthetic post-sampler fixture; its checked receipt matched 12 actor float
words and the flag/counter words from the Rust model under explicit FP controls.
Previous position and vector Z were preinitialized by the host, and the sampler,
source branch gates, live actor, callbacks and scheduling were not exercised.
The x86-64 implementation and the x87 state limitations are described in
[`cell-short-part-flow-adjustment-static-v1.md`](../re/cell-short-part-flow-adjustment-static-v1.md).

The allocator trace also shows that an environment can conditionally be served
from a previously used chunk without a payload-wide clear. Its initial field
and scratch values cannot be assumed zero from allocation alone; the separate
fresh-page inference is conditional. See
[`cell-environment-allocator-conditions-static-v1.md`](../re/cell-environment-allocator-conditions-static-v1.md).

An independently reviewed trace follows conditional indexed-input bits to the
player-target calculation. `cell_indexed_input_vector_v1` implements the four
finite vector-assembly groups from a frozen bit snapshot and explicitly supplied
callback return bytes. Six focused tests passed, including all individual input
alternatives and MXCSR restoration. A reviewed original-instruction host with
four callback stubs then matched the Rust vector words and aggregate C4/E0
effect for one synthetic snapshot; the receipt is
`target/cell-indexed-input-native-host-prep-20261105-v2/reviewed-fixed-result-v1.json`,
SHA-256 `eea707b4e6d581d05e0bd5d26eebb10654ecd9f980c571442394bdff9f28c26d`.
The host's final memory does not distinguish the individual side-effect stores.
Actual input dispatch, callbacks, normalization, actor target, runtime scheduling
and gameplay were not exercised. See
[`cell-indexed-input-player-target-static-v1.md`](../re/cell-indexed-input-player-target-static-v1.md)
and [`cell-input-target-angle-helper-static-v1.md`](../re/cell-input-target-angle-helper-static-v1.md).
The target helper's two finite early branches are implemented separately; a
reviewed original-instruction host matched the Rust model on two synthetic cases,
including signed-zero and subnormal words. Its receipt is
`target/cell-e5af20-early-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `1c947a44331584e143067df4e68aac215f11e4caefebd55be18c7ced8c014a34`.
The default target-update fixture still rejects the angle path without explicit
CRT-result inputs. The preceding normalization arithmetic has focused tests and
matched the original instructions for one
synthetic active vector, including its reciprocal and three candidate words.
The receipt is
`target/cell-candidate-normalize-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `91e4c735ae750d53f80c52c37c10263e3cbfefb6d446c97136036973b99c9909`.
The input-vector, normalization and early-return models also compose into a
checked synthetic target-update fixture. In one reviewed original-instruction
host, a synthetic pool record, four callback stubs, the original checked pool
lookup and the helper's early branch produced target, direction and flag fields
that match the Rust composition. The Rust fixture receives an already resolved
actor; it does not perform that pool lookup. The receipt is
`target/cell-player-target-integration-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `dc20425ac472b8389592b8fd38d135fc0af5b6bff8c9d79df6771fe597e48be6`.
This is one synthetic active case, not a live runtime or gameplay result; the
real callbacks, actor binding, live CRT results and scheduling remain open.
Its two underlying angle-wrap routines have also been modeled with explicit
period and iteration limits; a reviewed native host agreed on four wrapped
words and three sign/magnitude pairs in seven fixed synthetic cases. The
difference delta was not captured. See
[`cell-input-target-angle-helper-static-v1.md`](../re/cell-input-target-angle-helper-static-v1.md)
for the receipt and remaining CRT/runtime limits.
The angle-to-vector routine is also modeled for finite admitted inputs. A reviewed
original-instruction host matched all 18 output words in six fixed synthetic cases,
including signed zeros and both outer branches. The Rust branch identities were
inferred from the fixed inputs; native branch identity and immediate FSIN C2 were
not captured. The receipt is
`target/cell-angle-vector-native-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `9f7198a32655d88e2f022668f1236f59b7a80b265900c19f4e7a88835df5c256`.
This does not establish the live baseline, angle source, actor, or gameplay.
The longer angle branch is now modeled with two explicitly supplied committed
`_CIasin` words. A reviewed host substituted those words and ran the original
instructions on three fixed synthetic cases; Rust matched all nine vector words
and the model's direct-copy, step and overshoot selections. Native branch IDs and
immediate FSIN C2 were not captured, and the substitute is not the CRT. The
receipt is `target/cell-e5af20-long-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `05147d82e102eac006c3617cfcc832052472dfa66436469af06d0a4f6162076e`.
An optional synthetic player-target composition accepts the same supplied
words, while the existing default still rejects that path. Thirty focused Cell
input tests passed after this addition. No live keyboard, callback, CRT, player
or gameplay behavior has been established by these comparisons.
The conditional Windows message route has now been traced from key messages to
the indexed Cell setters, including the source's separate derived modifier word.
An opt-in virtual-key snapshot models admitted press and release stores and that
derived word; it takes the down gate and four layout callback values as inputs.
A focused synthetic press-to-target-to-release test passed. A relocated
original-instruction host also matched 80 indexed words and ten summary words
against the Rust key snapshot over ten fixed actions. A separate legacy CRT
host matched 14 committed results across two admitted return-global values; its
SSE2-disabled, CW-027F policy is not an observation of the running game. The
exact ±1 legacy endpoint cases also matched in four controlled original-instruction
cases. The optional controlled steering composition can use the closed-interval
legacy calculation from the actual candidate words, with focused tests passing. Real queue delivery,
receiver selection, focus handling, locale provider and CRT dispatch remain unverified. See
[`cell-indexed-input-windows-route-static-v1.md`](../re/cell-indexed-input-windows-route-static-v1.md).
One reviewed combined host executed the original indexed-vector assembly,
normalization, admitted legacy CRT branch and target update for a synthetic
W+D state and pool/actor. The Rust composition matched all three 256-byte
actor, state and indicator snapshots, including target and direction words.
Its receipt is `target/cell-player-target-legacy-full-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `cf0d418053c6099e80cef9b46ec7c16e70c0a2c1da7957487ae4b9791f011a32`.
The host performed a synthetic pool lookup with fixed callback stubs; Rust was
given the resolved actor. This does not establish live input or gameplay.
The conditional `fr-fr` comparison is traced separately in
[`cell-indexed-input-locale-static-v1.md`](../re/cell-indexed-input-locale-static-v1.md);
one constructed provider and its stack-preserving getter are identified in
[`cell-indexed-input-locale-provider-static-v1.md`](../re/cell-indexed-input-locale-provider-static-v1.md),
while the active runtime provider and locale remain unobserved.

An installed-package audit found manifest priority values of 2 for PatchData
and 101 for the EP1 Data package. A bounded model reproduces the native
descending signed priority and first-success selection for caller-supplied
registrations and frozen query outcomes. Under those explicit successful
overrides EP1 precedes PatchData, but actual mounts, provider/cache hits,
type mapping, fallback, save override and actor identity remain unproven. See
[`cell-startup-database-priority-boundary-v1.md`](../re/cell-startup-database-priority-boundary-v1.md).


A conditional Cell skybox export requests effect `05C7A4D0`; six admitted
`C1195706` directory variants map it to screen component 186 and texture ID
`BD08BD8A:00000000`. Graphics startup can register the requested `2F4E681C`
type alternatives in order `[2F4E681C, 2F7D0004, 2F4E681B]`, and a separate
registered converter accepts the B-to-C pair. The retained 82-package index
census contains just one exact candidate across those three types and scope 0:
`Spore_Graphics.package` `BD08BD8A:2F4E681B:00000000`. A fresh decode hashes
to `5705df1ab03e94c58c8a890383026e68c68dfe64ba488d4d767f2706d8ad8aec`.
Native database-first ordering, active mounts, type registration, provider
cache, conversion success, runtime color/material state, viewport and camera
remain admissions. The existing hidden near-white shader probe is an exploratory
candidate, not a retail-selected background. See
[`cell-skybox-texture-fallback-boundary-v1.md`](../re/cell-skybox-texture-fallback-boundary-v1.md).
An explicit inspection path now hash-checks the caller-selected decoded effect
directory, shader pack and texture, joins the export to one supported local
screen, and renders with supplied viewport, color and adapter. The controlled
AMD output differed by at most one channel value from the earlier RTX probe;
21,620 of 589,824 pixels differed. Deliberately mismatched hashes were rejected
before device creation, and a second viewport/color was exercised. See
[`cell-skybox-explicit-inspection-v1.md`](../re/cell-skybox-explicit-inspection-v1.md).
This remains an inspection with unverified runtime selection, not an automatic
gameplay background or source-fidelity acceptance.

The creature skin query trace now identifies constructed edge ordinals in
endpoint membership lists and a conditional row-derived key path into query
lanes. It does not yet establish selected CreatureData backing or final palette
identity; see [`creature-generated-skin-key-construction-static-v1.md`](../re/creature-generated-skin-key-construction-static-v1.md).
The ordinary typed reader can conditionally copy 0x8C-byte rows into the object
passed through the builder/consumer owner; cache, stream, mutation, callback
and lifetime conditions remain unresolved. A bounded owned bridge builds
endpoint membership from explicitly supplied edge pairs, preserving order and
self-edge duplicates. A separate admitted slice constructs the edge pairs from
frozen rows. On one explicitly hashed installed payload, the composition reports
10 rows, 11 endpoints, 10 edges and 20 membership entries; a translation-edited
copy yields the same topology. This result assumes the supplied bytes represent
the post-mutation rows and that prior native preparation succeeded. It does not
determine the final palette or generate a mesh; see
[`creature-skin-intermediate-row-identity-static-v1.md`](../re/creature-skin-intermediate-row-identity-static-v1.md).
The source inspection and hashes are detailed in
[`creature-skin-row-edge-construction-static-v1.md`](../re/creature-skin-row-edge-construction-static-v1.md).
The same bounded edge/membership inspection accepted all 1,365 retained pinned
source payloads, totaling 46,418 rows and 50,221 edge pairs, under those same
admissions. Four synthetic fixed fixtures also matched 28 original-instruction
edge words with preparation and allocation substituted.
A checked remap follows the row parents and extra-edge endpoint mapping; three
synthetic fixtures matched 22 original-instruction map words with vector
allocation substituted. On the explicit 10-row source candidate and its
translation edit, the remaps matched each other. These maps do not establish
the finalized palette; see
[`creature-skin-row-edge-remap-static-v1.md`](../re/creature-skin-row-edge-remap-static-v1.md).
All 1,365 pinned snapshots also passed the remap inspection under the same
admissions. In 917 files at least one edge-remap entry differed from its edge
ordinal, so ordinal identity cannot be assumed for this supplied set.
The bounded skin-query path now also computes the source grid address and
dispatches caller-supplied cell records into the existing four-lane kernel.
It preserves the original wrapping linear-index behavior and has four focused
synthetic tests. Six synthetic addresses matched the original lookup and two
unmodified helper instruction spans. Grid construction and original dispatch
into the lane kernel remain open;
see [`creature-skin-grid-query-static-v1.md`](../re/creature-skin-grid-query-static-v1.md).
An explicit one-point fixture composes the row/edge/remap, supplied-grid query,
accumulation and initial-record models. Its three focused synthetic tests passed;
it requires supplied post-mutation rows, endpoint coordinates and grid, and it
does not construct or validate a baked creature. See
[`creature-skin-vertex-fixture-v1.md`](../re/creature-skin-vertex-fixture-v1.md).
An audio trace conditionally follows a selected sample to an SNR key and provider
request, with provider selection and voice start still open; see
[`cell-audio-selected-snr-provider-static-v1.md`](../re/cell-audio-selected-snr-provider-static-v1.md).
Another conditional path queues and consumes a sound-control operation before
issuing a deferred request; concrete provider and device execution remain open.
See [`cell-audio-control-provider-command-static-v1.md`](../re/cell-audio-control-provider-command-static-v1.md).
The source bit reader's conditional header dispatch agrees with the codec,
channel and rate fields consumed by the existing XAS1/PCM16 decoders; source
memory identity and playback remain unverified. See
[`cell-audio-snr-header-decoder-static-v1.md`](../re/cell-audio-snr-header-decoder-static-v1.md).
An explicit event-row inspection now checks the count-driven supported event
framing, hashes an explicitly selected event and SNR payload, and decodes the
selected source sample. The installed example produced 33,165 mono values at
44,100 Hz; four incorrect-input checks failed closed. Its chosen row, loader
and provider assumptions are not evidence of a live gameplay cue; see
[`cell-event-snr-explicit-inspection-v1.md`](../re/cell-event-snr-explicit-inspection-v1.md).
One conditional Cell contact route also selects the source creature–plant
impact event when the supplied record flag and category values meet its gates;
the installed event and an explicitly selected sample resolve through the same
inspection. A newly inserted record's flag is conditionally set by the source
distance/definition comparison; actual actor inputs, activation and playback
remain open. See
[`cell-contact-event-dispatch-static-v1.md`](../re/cell-contact-event-dispatch-static-v1.md).
The explicit installed globals candidate has 5.0 at both collision thresholds;
the traced operand is actor-position separation, not automatically relative
velocity. Provider identity and mutation remain open; see
[`cell-contact-input-provenance-static-v1.md`](../re/cell-contact-input-provenance-static-v1.md).

## Next work

The next useful joins are the Cell startup factory to actor selection, event property
loading to the audio consumer, and actual Creature UI layout/transform state. The editor also needs
source part attachment rules, baked geometry regeneration and a source-backed UI
before its output can be evaluated in gameplay. Source animation binding, camera,
world simulation, later stages and end-to-end campaign acceptance remain open.
Original installed packages and local research output remain untracked by Git.

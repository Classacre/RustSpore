# Continuation checkpoint — 2026-11-08

The requested outcome remains the full original-fidelity game, including all
stages and editors. The experimental source-art scenes do not yet establish
retail behavior, camera, interface or complete source asset selection. The
previous checkpoint is [2026-11-05](continuation-2026-11-05.md).

The initial `tools\build.bat test --workspace --all-targets` for this checkpoint completed with
1,112 passed, zero failed and 18 ignored across 230 test groups. Its log is
`target/workspace-tests-20261108-v3.log`, SHA-256
`2530142304aff6e271656f3cf83ed286fb6410c164f120cf9a065758dd74049e`.
The ignored set includes the separately run installed-data audio and source
feature-retention checks. Later integration runs are recorded below.
Subsequent changes require separate validation; these checks do not establish
gameplay or visual fidelity.

An explicit event-row-to-SNR startup preview has been added to the experimental
Cell frontend. It checks selected package keys and payload hashes, requires a
loader/path admission and does not infer a live event or random sample. A
headless installed-data test decoded the selected event sample and matched its
retained PCM digest. Actual audio-device playback and gameplay selection were
not observed; see [the preview note](../re/cell-source-audio-preview-v1.md).

The Cell impact-flag comparison now has a separately pinned hidden original-
instruction run. Six synthetic flag results matched the Rust model. This does
not reproduce native floating-point status, earlier gates or live contact; see
[the model note](../re/cell-contact-impact-flag-model-v1.md).

An owned grid packer now connects caller-supplied candidate lists to the
one-point skin fixture. It bounds total allocation and rejects nonfinite selected
lanes; focused packing and fixture tests passed. The inspection of one installed
CreatureData payload used entirely synthetic supplied positions, lanes, grid
and candidates. Native candidate selection, grid construction, palette binding
and creature regeneration remain open.

On the current session, the hidden D3D9 source-shader inspection selected
`NVIDIA GeForce RTX 3060 Laptop GPU` with hardware vertex processing. Its
768x768 controlled candidate output was byte-identical to the earlier RTX
probe, SHA-256 `1135f77e830c04d874c9681d46290822d92f6deec96e26697059d8686a1538f5`;
the new log is `target/source-cell/screen-background-rtx-20261108-v1.log`,
SHA-256 `2406f9c679badd32d3452758971918275af9b2698fd918ef38667f62000235ba`.
This confirms the adapter for that hidden draw, not active Cell background or
retail visual fidelity.

The controlled Cell capability builder now retains the ordered complete
`NativeCellPart` model outputs and exposes an immutable view. A separately run
installed-input test confirmed ordering and byte retention against a rebuild
using the same implementation; this is a consistency check, not an independent
native comparison or a live actor/cache binding. The special `476A98C7` branch
remains outside this builder's contract.

The checkout contains extensive pre-existing uncommitted work and ignored
research evidence. Preserve it; do not blanket-clean `target` or claim these
inspections complete the remaining milestones.

The experimental offscreen Cell frontend now accepts an explicit, hash-bound
source UI fixture and draws its admitted atlas rectangles with Bevy. An installed-
asset run selected the RTX 3060 through Vulkan and completed 60 simulation ticks;
its capture is `target/cell-source-ui-rtx-20261108-v2.png`, SHA-256
`aa5f3678d084ae20f7610a59e33fe5b4523e5b6dd74ba6f7a7e1bf1d94c7e506`,
and its log is `target/cell-source-ui-rtx-20261108-v2.log`, SHA-256
`53844d6566f6b5a9642418a29849cfa82e15baa8acf18cccff2d82f0465e14a5`.
The capture contains experimental prototype gameplay and a controlled fixture:
the UI placement, game state and native shader execution are not verified.

A controlled feature snapshot can expose retained Cell part rows with their
original indices after an admitted first-word cache match, skipping class 1.
It does not establish the live actor or contact lifecycle. The prepared-cell
skin candidate filter now feeds the existing grid packer and vertex fixture;
48 focused skin tests passed, but its box and lane inputs remain caller supplied
and a native filter comparison and source geometry binding remain open.

To reproduce this experimental Cell run, first generate the fixture with the
command in [the drawable trace](../re/cell-ui-std-drawable-dispatch-v1.md), using
fresh output paths. Then set `WGPU_ADAPTER_NAME` to `RTX 3060` and run the built
binary with the following arguments (replace the fixture and shot paths with
fresh local paths):

```powershell
target\debug\rustspore.exe --experimental --cell --source-cell-install 'D:\Games\SPORE Collection' --source-cell-id 7D00E384 --source-rival-cell-id EFE530A1 --offscreen --demo 1 --shot target\cell-source-ui-new.png --source-ui-install 'D:\Games\SPORE Collection' --source-ui-layout 'D:\Games\SPORE Collection\Data\PatchData.package' --source-ui-atlas 'D:\Games\SPORE Collection\Data\PatchData.package' --source-ui-fixture target\source-ui\cell-progress-source-preview-v3.json --source-ui-clip shader --source-ui-half-pixel 0
```

The adapter selection must be checked in the `CELL_GPU` log line. The clip and
half-pixel options select fixture admission only; the Bevy draw does not execute
the source UI shader or prove its clipping, blending, or placement.

The first isolated native skin-filter host attempt exited with code 7 before
emitting comparison data. Review identified an unaligned fixed-address
VirtualAlloc reservation check; it provides no native comparison evidence.

After the changes above, `tools\build.bat test --workspace --all-targets`
completed with 1,122 passed, zero failed and 18 ignored across 232 groups. The
log is `target/workspace-tests-20261108-v4.log`, SHA-256
`1d0535fd730dfc11195f68424e6b8b5f06456ce57d226b3dcf24ce17a8f41313`.
The ignored installed-data checks and the native harness are separate from the
workspace suite. This suite does not establish gameplay or visual fidelity.

A source progress snapshot can now be evaluated through the source float mapping
and the five-segment selector. The selector matched 62 controlled original-
instruction records; the mapping plus selector matched 52 controlled records.
These checks use fixed values and controlled floating-point settings, not a live
save object, UI layout or cursor animation. See
[the bridge note](../re/cell-ui-progress-snapshot-bridge-v1.md).

The experimental source-player inspection now contains a short-lived owned
checked pool with an admitted borrowed feature snapshot and a controlled first-
feature projection. It is not the source pool or a live actor; manager callbacks,
provider identity and collision acceptance remain open. See
[the pool fixture](../re/cell-owned-player-pool-fixture-v1.md).

The creature skin class-5 prefix matched native helper output and bounds bit for
bit on six retained rows from an installed creation, using a synthetic preallocated
receiver and an explicit frozen post-mutation row assumption. The full grid,
segments, palette and actual runtime object remain unverified. See
[the upstream note](../re/creature-skin-grid-upstream-lead-v1.md).

Following the additional player, progress-input and class-5 work,
`tools\build.bat test --workspace --all-targets` completed with 1,131 passed,
zero failed and 19 ignored across 235 groups. The log is
`target/workspace-tests-20261108-v5.log`, SHA-256
`af0a130411567966dda4c7866c6dae4c56e65db79aad18073daafabf20a0b2ec`.
The separately run source-player focused test included its installed-data checks
and passed 9/9. These results are not a gameplay or fidelity acceptance test.

The controlled first-feature projection subsequently matched an isolated
original-instruction fragment on six synthetic inputs; see
[the projection note](../re/cell-controlled-feature-projection-v1.md). This is
not a native contact pair or observed actor lifecycle.

After the pair-prefix, listener and segment changes, the workspace suite passed
1,144 tests with zero failures and 19 ignored across 238 groups. The log is
`target/workspace-tests-20261108-v6.log`, SHA-256
`2e4487def49e062dcb3d222d2bcc69a106140b867ae2f173388fd7dd987c3dc2`.
The second feature and pair-prefix instruction fragment also matched six
synthetic cases, including both sides of the distance threshold; see
[the pair note](../re/cell-controlled-feature-pair-prefix-v1.md). Full contact
points and callbacks remain unresolved.

The controlled offscreen Cell frontend can now draw five source-atlas progress
segments. An RTX 3060/Vulkan run loaded 45 quads and reached tick 60; capture
`target/cell-source-five-ui-rtx-20261108-v1.png` has SHA-256
`10ec699ef487bca2cdebe0edf88a2b59878a556d6d79040f57834f357185415c` and
log `target/cell-source-five-ui-rtx-20261108-v1.log` has SHA-256
`52a82fffd643a731615b8ad38b9ec268e46991cb3ae0d8d17332299c8dc98a95`.
A region comparison with the preceding single-segment capture found changes
below the HUD only in the moving-player region, not a lost world. This remains
a controlled prototype render, with caller-supplied UI state, placement and
Bevy rendering; it does not establish retail appearance or gameplay.

The restricted synthetic creature segment sampler matched four retained native
output records after activating the original executable manifest for its CRT.
The first failed run and DLL initialization diagnostic remain preserved; the
loaded DLL identity was not independently verified. No real creature threshold,
complete grid or live runtime follows from this result; see
[the sampler note](../re/creature-skin-segment-producer-lead-v1.md).

The latest workspace-wide validation passed 1,153 tests with zero failures and
19 ignored across 241 groups. Log `target/workspace-tests-20261108-v7.log`,
SHA-256 `0f58818ff119d1336e64178df615fb61645b725cae9675c42c367f912d2b1de1`.
A controlled two-actor fixture now composes borrowed feature rows, finite
contact geometry and the existing producer prefix, but stops at pending calls.
Its source-data and lifecycle admissions are documented in
[the fixture note](../re/cell-controlled-two-actor-prefix-v1.md).

The subsequent workspace-wide run passed 1,155 tests with zero failures and
19 ignored across 242 groups. Log `target/workspace-tests-20261108-v8.log`,
SHA-256 `56260388aec8beada91309fab892accd6fedd5107a5f4e1e36b3b312dc85861f`.
A later review identified null-image and collapsed-rectangle edge cases in the
five-segment frontend; fixes and fresh validation are in progress.
The nonzero-threshold Rust sampler matches lane words and bounds for five
controlled source-derived segments with an experimental host-math adapter; the
CRT equivalence and live source identity remain open.

The latest full workspace run passed 1,160 tests with zero failures and 20
ignored across 245 groups. Log `target/workspace-tests-20261108-v9.log`, SHA-256
`35b199d4e7095336058e4a8d406531e5de4606d63ed3edce48c3d1f0d627d557`.
The separately run installed-data UI and player checks passed. Both an empty
source-image selection and a 20x60 collapsed layout completed hidden RTX/Vulkan
captures at tick 60 with zero quads; native null-image fallback is not modeled.
The controlled post-arbitration handoff matched the original distance fragment
on six synthetic cases and stops before E673A0. Query origin and scale matched
three original-instruction cases; native grid boxes and filtering remain open.

After the E673 record-phase changes, `tools\build.bat test --workspace
--all-targets` passed 1,163 tests with zero failures and 20 ignored across 246
groups. Log `target/workspace-tests-20261108-v10.log`, SHA-256
`97cc2bec27cfc7020b106d391003c0fbcb2410256e8b794cb71aa05a490e05e0`.
The controlled full grid-builder comparison matched cell counts and key order
for all 512 cells in two cases with explicit allocator and cleanup stubs; it
does not validate native allocation, every lane word, complete source identity,
or a live grid. See [the grid note](../re/creature-skin-grid-geometry-lead-v1.md).
The first x86 background upload fixture failed before adapter selection. A paired
no-window D3D9 inventory subsequently reported only AMD for both ordinary x86
and x64 processes in this session; earlier RTX probes requested high-performance
graphics per process. The x86 upload result remains unverified.

Corrected DXT5/BC3 color decoding in the Rust asset decoder and Python
extractor: four-color interpolation is now used for all endpoint orderings,
while DXT1 transparency remains unchanged. Focused Rust and Python tests pass.
A controlled RTX point-sampling comparison across the candidate's nine mips
found maximum per-channel differences of 1 for levels 0–6 and exact matches for
levels 7–8; no cause for the differences or live material selection is claimed.
See [the Raster note](../re/cell-skybox-converter-raster-boundary-v1.md).

The full workspace suite after these changes passed 1,171 tests, zero failed and
20 ignored across 248 groups. Log `target/workspace-tests-20261108-v11.log`,
SHA-256 `e2ae62a3640bc71af550caae695931407c5a9d489378f8e8a0dc89418b88f0d9`.
The retry model also matched five synthetic original-instruction cases with
controlled 1F80-preserving downstream stubs, not actual runtime FP transitions.

A fresh hidden RTX/Vulkan Cell capture after the BC3 fix completed 60 ticks with
45 controlled UI quads. `target/cell-source-five-ui-bc3-rtx-20261108-v1.png`
has SHA-256 `10ec699ef487bca2cdebe0edf88a2b59878a556d6d79040f57834f357185415c`,
identical to the earlier matching controlled capture; this single scene does not
exercise or prove all affected source blocks. Its log is
`target/cell-source-five-ui-bc3-rtx-20261108-v1.log`, SHA-256
`034f638674ea7fdcc4db44e1f5c76276c5883add9b6874ba78b4cd8620c4084b`.

The latest workspace suite passed 1,178 tests with zero failures and 20 ignored
across 250 groups. Log `target/workspace-tests-20261108-v12.log`, SHA-256
`6b490a409ca548698988bf52b76823cecc7373f8b9fa62d6273be9a31e1322e7`.
The Cell two-pass coordinator and creature signed-field model added since the
previous run remain controlled inputs; their original-instruction comparisons
are being prepared. The lattice-to-world transform matched four synthetic source
instruction cases. No full contact event or creature isosurface follows yet.

Workspace run `target/workspace-tests-20261108-v13.log` failed during
compilation: the new coordinator oracle example pulled in a test module that
referenced a missing crate-root module. The example integration is being fixed;
this failure supersedes no earlier passing evidence and must be followed by a
fresh full run.

The coordinator example test-target issue was fixed by including its real test
modules under `cfg(test)`. The subsequent full workspace run passed 1,211 tests,
zero failed and 23 ignored across 251 groups. Log
`target/workspace-tests-20261108-v14.log`, SHA-256
`f16b615a0d353a851c8dce78b15411bfd78305c3cbac384e32db9739d5255b6f`.
The count includes duplicate fixture tests compiled through the new example.

The E7F170 controlled two-pass coordinator matched seven isolated original
instruction cases with synthetic resolver, selector, event and cleanup stubs.
This verifies the bounded control flow only; no live actor, provider, or event
was observed. See [the coordinator note](../re/cell-controlled-pair-coordinator-v1.md).

The latest full workspace run passed 1,264 tests with zero failures and 26
ignored across 254 groups. Log `target/workspace-tests-20261108-v15.log`,
SHA-256 `31657573db4b20c6b67afbb3fdd8739eafb3e823f9d3072fab158b48b9e98d7e`.
The count includes fixture tests compiled in multiple example targets. The
bounded SSE2 field comparison matched seven synthetic safe-offset cases; runtime
CPU dispatch and isosurface generation remain separate. The case-10 gate matched
eight controlled original-instruction cases with a stubbed relation call; event
allocation and dispatch are not part of that comparison.

The current workspace suite passed 1,306 tests, zero failed and 29 ignored
across 256 groups. Log `target/workspace-tests-20261108-v16.log`, SHA-256
`d671279883b1fe7aa50ab05cd37dac07cac03a16206c352219145f6fc857bbd3`.
Some fixture tests run in multiple example targets. Four controlled case-10
event records matched original field-store fragments; allocation, ownership,
setters and dispatch were not executed. Five creature edge-crossing cases
matched with deterministic synthetic field callbacks. A controlled profile-2
RTX background probe produced the same PNG bytes as the earlier profile-3
candidate under matching render inputs; live shader/profile/camera selection
remains unproven.

The follow-on E69B00 second-lookup fragment matched four controlled original
instruction cases, with guarded actors and a supplied lookup result. The
comparison does not execute the preceding callback, subsequent constructor or
event dispatch. Receipt `target/cell-e69b-second-result-v1/receipt.json`,
SHA-256 `625837b00be8931ea6aed160e3f05f36288d00f21bf6b23ac5a4cfeef1a9662d`;
see [the trace](../re/cell-e69b00-prefix-v1.md) for the pinned model scope.
The creature corner-sample composition matched three synthetic cases with a
fixed callback, and ordered edge lookup matched five synthetic table cases;
neither establishes generated mesh topology or live runtime state. See
[the surface trace](../re/creature-skin-surface-crossing-boundary-v1.md).

The texture decoder now checks decoded-size arithmetic and confines base-mip
reads to the declared base extent. Twenty-two focused raster tests passed,
including oversized dimensions and a truncated base followed by a mip prefix.
This is an asset-loader correctness fix, not a rendering fidelity claim.

After the subsequent Cell constructor and creature edge-cache models, the full
workspace suite passed 1,475 test executions, zero failed and 38 ignored across
263 groups. Log `target/workspace-tests-20261108-v17.log`, SHA-256
`246bc0df400cc9ae3ccf1d63ef38b64c40af95c67e74ae4db5843f47fdac0a4c`.
The count includes repeated fixture tests in example targets. The installed-data
five-segment UI regression also passed separately. The corrected constructor host
matched four supplied-global cases; it does not execute the following E68070
path. The first host preparation was not executed because review caught an
incomplete return instruction; the corrected host's result and source-hash
scope are documented in [the trace](../re/cell-e69b00-prefix-v1.md).

The creature edge commit's existing-position path matched three synthetic
original-instruction cases; the append and cache-insert path matched two
synthetic cases with preallocated append stubs. These checks do not establish
native allocation, surface traversal, topology or an editor-ready mesh. The
current integration gap is documented in
[the surface trace](../re/creature-skin-surface-crossing-boundary-v1.md).

A fresh hidden experimental Cell smoke run after the raster bounds changes
loaded the installed player and rival assets, selected the RTX 3060 Vulkan
adapter and reached tick 60. The image
`target/source-cell/loader-smoke-20261108-v17b.png` has SHA-256
`71938d2c7330111a3733ca86e3903e53d21f7320987e5199adc7d097fde121b8`.
The initial attempt used a model ID as the Cell ID and exited with a missing
resource error; its retained log is `target/source-cell/loader-smoke-20261108-v17.log`.
This is an experimental frontend check, not evidence of retail gameplay fidelity.

The BC3 correction also affects an installed kind-0 Cell texture: 81 of
16,384 base-level texels differ from the old decoder. A controlled same-scene
RTX mesh comparison found 1,569 differing frame pixels with identical admitted
camera, pose, shader and material settings, using uncompressed decoded texture
variants. Receipt `target/source-cell/kind0-bc3-mesh-result-v1/receipt.json`,
SHA-256 `4034162e9acfb4efe4383ac6567bf4dadbf05ce5e70f28564ee0b256fd182fe1`.
This verifies an observable source-texture correction in that fixture; it is
not a capture of live gameplay or a compressed GPU-sampler comparison. See
[the asset note](../re/cell-kind0-asset-intake-v1.md).

The next full workspace run, including the partial edge driver and scheduled
one-frame Cell diagnostic, passed 1,547 test executions, zero failed and 41
ignored across 265 groups. Log `target/workspace-tests-20261108-v18.log`,
SHA-256 `c0a8bdcb69ee961260da196acec86fa2adff44893fbb3c72f3651734966f9463`.
Some fixture tests are compiled and run in multiple example targets. These
diagnostics use explicitly supplied state and do not complete a gameplay or
editor milestone.

After adding the opt-in source-player material and installed-frame peer,
workspace run `target/workspace-tests-20261108-v19.log` passed 1,555 test
executions, zero failed and 42 ignored across 266 groups, SHA-256
`1341dfcdaedc8d093993c183ae202c2b6e5dab349b00632fad9a9db8d3684597`.
The execution count includes repeated fixture tests in example targets.
An explicitly run installed-data test and CLI also exercised a synthetic
two-actor pair sharing one verified source definition, stopping before E7F170.
The subsequent cube/edge composition edit overlapped this run; v19 does not
validate that final composition and a fresh suite is required after it settles.
The fresh v20 run passed 1,558 test executions, zero failed and 42 ignored
across 266 groups. Log `target/workspace-tests-20261108-v20.log`, SHA-256
`fc7a8b5a8541f7f070e0caf0ea5c6c3800668b9d9c89183f7059ce7e50580faf`.
Repeated fixture tests contribute to the execution count.
After the registration prefix and creature seed changes, v21 passed 1,566 test
executions, zero failed and 43 ignored across 269 groups. Log
`target/workspace-tests-20261108-v21.log`, SHA-256
`2fa9dad4180a3337951c0b5274bb4f41756ca820bd9ebc1bb8a11b2f4e21ea9f`.
The source seed fragment matched six controlled cases with field and enqueue
stubs; native queue traversal, live lane selection and a finished mesh remain
open. Repeated example fixtures contribute to the test execution count.
The next v22 workspace run passed 1,573 test executions, zero failed and 43
ignored across 271 groups. Log `target/workspace-tests-20261109-v22.log`,
SHA-256 `7ead755d33bbe802aae98c4a3639b43f0fcb6277c15d1f8d20f81204db0f3e0a`.
Four controlled first-queue-item cases also matched the original instruction
path with corner, cube and lifecycle stubs; full queue/mesh behavior is open.
A controlled compact-sphere queue subsequently matched source instruction
execution to exhaustion across 32 items, using a supplied seed, synthetic
field, recording cube stub and no-op finalization. This does not validate
native edge geometry, allocation, finalization or a live creature.
A separately reviewed composed original-instruction comparison then matched
the queue, corner/edge caches, partial positions, counts, triangles and field
callbacks for one synthetic sphere: 32 queue items, 81 corner records, 18
position accumulators and 32 triangles. It used admitted tables, deterministic
field and preallocated append stubs with no-op finalization. This does not prove
a source creature's input, native allocator, finalized mesh or runtime identity.
The retained source-derived 0473 eleven-lane snapshot also matched the original
scalar field, seed, queue, cube, edge and crossing instructions for 72 queued
cells and 84 partial triangles. It admits frozen rows, a supplied 0.15 step,
zero padding and a Rust-built grid snapshot; native grid construction at that
step, runtime selection, native allocation and finalization remain unverified.
See [the surface trace](../re/creature-skin-surface-crossing-boundary-v1.md).
The next v23 workspace run passed 1,576 test executions, zero failed and 43
ignored across 273 groups. Log `target/workspace-tests-20261109-v23.log`,
SHA-256 `24f6497e30812a6e2f58503ddb046ddae1a3a219809c3c5e1be8db0e0c8ea6a8`;
fixture tests may repeat in example targets. The Cell rank/tag filter also
matched eight controlled original-instruction cases, independent of native
endpoint sorting or persistent registration.
The subsequent v24 workspace run passed 1,577 test executions, zero failed and
43 ignored across 275 groups. Log `target/workspace-tests-20261109-v24.log`,
SHA-256 `e97524e36901bc8c739ae7ae6bb8a841ee23a53a9b5a2a97c4dd78f0390e20da`.
Counts include repeated example fixtures. The endpoint comparator separately
matched eight controlled native cases; the original sort/merge remains open.

A synthetic hidden D3D9/Vulkan comparison of SRCALPHA/INVSRCALPHA blending
with depth writes enabled and disabled produced identical 64x64 RGBA buffers
for both controls on the RTX. Receipt
`target/source-cell/saved-blend-depth-result-v1/receipt.json`, SHA-256
`a509949e20246a8b9aa6ee3b4d08a009de99e1fa28a6c516c9bc5e9566735a20`.
This establishes only the controlled pipeline mechanics; shader, geometry,
scene order, Bevy integration and live material selection remain open.
An isolated Bevy specialized-material fixture subsequently matched those two
D3D9 controls exactly as well. Receipt
`target/source-cell/bevy-blend-depth-result-v1/receipt.json`, SHA-256
`ebe04e4fed153f83aeffbd8faf0b2a2fc8795faf716012c8458b4bf2d9ccb856`.
It uses synthetic geometry and shader inputs, not the game's live scene.
The experimental frontend now has an offscreen-only
`--source-kind0-saved-blend` option for the explicit source player. Controlled
tick-0 RTX captures with matching initial simulation state differed at 308
player-region pixels; the normal material remains the default. The option still
uses Bevy PBR, reverse-Z depth and Bevy's ordering, rather than the source
shader or verified live state. Receipt
`target/source-cell/saved-blend-player-frozen-v1/receipt.json`, SHA-256
`fccef3619c441ccd94ff4f55aa86b7cc9eb3d48100cfaa5348f9d5f9c9fb7954`.
An initial scratch same-mesh Bevy shader comparison against the controlled
D3D9 fixture matched its operational foreground coverage (166,269 pixels),
but interior color discrepancies reached 66 channel levels. Receipt SHA-256
`f97fd9b8a60b206fa0a41bb6054c93e82536b240bbce4caaf4a493043a366afc`.
The shader port is not validated or integrated; sampling, blending and other
possible causes require isolation. An earlier scratch run failed before render
because the image constructor was given a complete mip chain as base data;
that failure remains under `target/source-cell/kind0-wgsl-scratch-result-v1`.
A point-filter/base-mip control also matched coverage but differed by up to 72
channel levels in the common interior; receipt SHA-256
`bbaf79434ce00742a448270ec7f46e24885730768790ab052cc9625cf9cc75da`.
That test narrows the sampling conditions but does not identify the cause.
With blending disabled in both fixtures, the interior maximum difference was
186 (the Bevy render phase also changes). A separate synthetic UV-output probe
still matched operational coverage but differed by up to nine channel levels
in the common interior, without texture sampling. Receipts are
`target/source-cell/kind0-noblend-compare-result-v1/receipt.json` and
`target/source-cell/kind0-uv-compare-result-v1/receipt.json`; these isolate
conditions, not a proven root cause or shader parity.
For one isolated triangle, the shifted control had no operational coverage
disagreement and 819/832 common interior UV-output pixels within one channel
level; the unshifted control had 89 coverage disagreements and 784/792 within
one. A separate overlapping-mesh triangle-ID probe had many ID disagreements
despite equal coverage. Its encoding/attribute path still needs independent
validation before attributing those disagreements to depth selection.
The non-overlapping 896-triangle ID grid exposed that measurement issue: D3D9
matched all expected tile-center IDs, while Bevy differed at 239 centers by
one or two red-channel units. Receipt SHA-256
`a27db2cdcfd6701999626c633b760162ff88d6e45751af75eec45a3a7bd35c4c`.
Consequently the earlier raw ID disagreement count is not a valid depth-winner
comparison; a more widely spaced encoding is being prepared.
The spaced three-channel encoding calibrated all 896 non-overlapping tile centers
on both devices. In the overlapping control, all 166,269 common-covered pixels
decoded validly, but 97,363 triangle IDs differed (95,971 of 164,429 common
interior pixels). Receipt
`target/source-cell/kind0-id-nibble-overlap-compare-result-v1/receipt.json`,
SHA-256 `7316d42b9467211ecb992b29ee57addb00a2414b50d1ea36d12df8ab6f5a6e10`.
Depth state and tie behavior remain under investigation; this controlled
result does not establish the live game's rendering behavior.
A direct state query then found the native fixture had depth testing disabled
(`ZENABLE=0`, `ZWRITEENABLE=1`, `ZFUNC=LESSEQUAL`) while the Bevy fixture had
depth testing enabled. The prior overlap comparison therefore did not compare
equivalent depth states; a matched-state rerun is required. This query is a
fixture observation, not an assertion about the live game.
With depth testing explicitly enabled on both renderers, the calibrated ID
comparison had zero disagreements across all 166,269 common-covered pixels
and zero operational coverage disagreements. Receipt
`target/source-cell/kind0-matched-depth-overlap-compare-result-v1/receipt.json`,
SHA-256 `30dafad838833f78419a5d5bccd3a3742dde2030cf19bd0b56ccf6678f85f935`.
The native target used D24S8 and Bevy used Depth32Float. A matched-depth UV and
texture comparison is still needed; no live material state is established.
The matched-depth synthetic UV control subsequently had zero operational
coverage disagreements and 162,405/164,429 common interior pixels within one
channel level, with maximum difference two. Receipt
`target/source-cell/kind0-uv-depth-compare-result-v1/receipt.json`, SHA-256
`878d79fe28c6229bac36c312e28d74b1e927a1bb7afdeadbc95e2a424139ccbe`.
This corrects the earlier unlike-depth UV comparison, under the stated fixture.
The matched-depth no-blend point/base-mip texture comparison likewise had zero
operational coverage disagreements, 164,325/164,429 common interior pixels
within one channel level, and a maximum difference of two. Receipt
`target/source-cell/kind0-texture-depth-compare-result-v1/receipt.json`,
SHA-256 `f6e999ff412282a6e1472690a59f45af3555801521f08ea479a645d8851d7ca4`.
At that point, the combined linear/mip/blend behavior had not yet been compared.
The matched-depth linear-filter, point-mip and alpha-blend control also had zero
operational coverage disagreements, with 147,570/164,429 common interior pixels
within one channel level and a maximum difference of three. Receipt
`target/source-cell/kind0-saved-depth-compare-result-v1/receipt.json`, SHA-256
`f293e70f8a134cbb319ff6c20005107b740b964ad5af554c51d7d2c366a82d11`.
It uses explicit fixture state and an identity palette; it does not establish
live material selection, general palette behavior or the original scene.

The independent RW4 texture digest script also used the DXT1 color branch for
DXT5. It now forces the four-color branch, and the existing ordered/equal-endpoint
test checks both Python decoders. The new post-correction corpus run matched
2,954 standalone-raster and 8,915 RW4 per-resource RGBA digests, with zero
mismatches; receipt `target/texture-digest-comparison-20261109-v1.json`, SHA-256
`99ac98cf0168e7aee3602be1f142bd05f6fc9367fcfaf7b42662423a222a9c6b`.
Package hashes were recorded after the scans started, so stable reads are an
admission; FNV digest agreement is not byte-stream or native-render proof.

An opt-in installed-player one-frame diagnostic now retains the same-read,
hash-checked CELL bytes and source creation/feature provenance in a synthetic
checked registry actor. It uses caller-supplied bounds and modeled pre-override
geometry, rejects unsupported saved overrides, and stops before unresolved actor,
selector and event producers. Its six focused tests, explicitly run installed
data test and headless CLI passed. The one-actor fixture produces no pairs; it
does not demonstrate live gameplay. See
[the composition note](../re/cell-installed-player-frame-composition-plan-v1.md).
The opt-in peer subsequently exercised a synthetic pair sharing the same
inspected definition. A separate conditional registration prefix now derives
player tag and bounds from modeled geometry with explicit post-E86960 stability
and manager admissions; eight original-instruction cases and its installed-data
fixture passed. Persistent native registration and live identity remain open.
A controlled fresh-collection join subsequently matched two original-instruction
two-append cases and passed an explicitly run installed-data two-actor test;
allocation, reuse, removal and persistent source history remain unmodeled. See
[the fresh-registration note](../re/cell-fresh-sweep-registration-v1.md).
The uninstrumented fresh endpoint-sort comparison now matches three cases using
the pinned installed MSVCR90 `qsort` under the game's activation context.
The private desktops were not switched and were closed after both children were
reaped. Receipt `target/cell-fresh-sort-activated-result-v1/receipt.json`, SHA-256
`a638aafa53f3c5061fbd3ba70daa02d514776791e256066f216e245997ee85bd`.
This does not cover retained entries, tombstones, ties or live actor history.
The full fresh consumer subsequently matched four source-instruction cases,
including capacity limits, same-tag filtering, separated endpoints and active-list
removal order, with the same private-desktop and pinned-runtime admissions. Receipt
`target/cell-fresh-consumer-result-v1/receipt.json`, SHA-256
`371a2f0bfc665f7a56e0be7d58af824aa7950015f166dbad32cacb36af6a3912`.
This still does not establish persistent or live gameplay behavior.
The inspection fixture now retains sorted endpoints and slot ranks alongside
the pair result. A separate v2 oracle invoking that API matched the four
preserved native rows; receipt
`target/cell-fresh-consumer-v2-result-v1/receipt.json`, SHA-256
`ab2d2bba9eea49bb3d79279d917e406b56531459a1f2b4029966c1cadaf9c0c5`.
The checked pair routine still performs an independent second sort. Focused
core, frame and explicitly run installed-data tests passed; the fixture uses
synthetic handles and does not establish live or persistent actor behavior.
The bounded old/fresh endpoint merge also matched five original-instruction
cases, covering unchanged old entries, interleaving, tombstones and empty or
fresh-only inputs. Receipt `target/cell-old-fresh-merge-result-v1/receipt.json`,
SHA-256 `5611bf037260f201472790cd79c0d06cfa1d2786438710fc013b6a93ee054bce`.
A separate append-only two-frame diagnostic composes the checked snapshots;
actual update producers and persistent native actor lifetime remain open.
The pending-endpoint update matched four original-instruction cases against a
byte model; the checked Rust API then matched the retained native rows. Receipts
`target/cell-pending-endpoint-update-result-v1/receipt.json` and
`target/cell-pending-endpoint-update-rust-result-v1/receipt.json` hash to
`c155de4b3a6b2698067d3b05c90b5cdd275ef3635042f8ec003354701cead22d` and
`8aea1e6cce42f8752131bb184c5f0b182bd9912602d059fe11f5842b68765c95`.
Four synthetic removal cases likewise matched original instructions, with the
Rust API matching retained rows: receipts
`target/cell-sweep-remove-result-v1/receipt.json` and
`target/cell-sweep-remove-rust-result-v1/receipt.json` hash to
`b31f75f07d6f8fc12cd2075446d022e11c7004286d2e4acde3bb79568bb341bb` and
`08f34f82d6a28c10aca44262e37ad9056d4b826e6d9086736ac20d229d25767d`.
The two-frame pending-update composition is synthetic; sorted updates, source
actor ownership and persistent lifecycle remain open.
The four pair-disabled sorted crossing helpers subsequently matched four
controlled source-instruction cases, and the checked Rust adapter matched the
retained rows. Receipts `target/cell-sorted-crossing-result-v1/receipt.json`
and `target/cell-sorted-crossing-rust-result-v1/receipt.json` hash to
`aac7ff9d17613a647591507eb667f17441c0a6329666f543412460a6922b4f5b` and
`fee54756e7228457fdf5c74e8fe3e0baf3461bea228294d83ecf347d30a6a970`.
These exclude pair effects, ties, tombstones and a complete live update.
The full sorted-update instruction path subsequently matched the checked Rust
composition in four controlled cases, including signed zero; receipt
`target/cell-sorted-update-result-v1/receipt.json`, SHA-256
`58df0d9d0af849b1e6ff2a9f20bdd30b3d8377d39cb6772995ecf47e872b142f`.
It excludes pending links, ties, tombstones and live actor lifecycle.
The original actor-side `E6DB40` frame-update caller matched four controlled
cases with explicit geometry and manager-update stubs; receipt
`target/cell-actor-frame-update-result-v1/receipt.json`, SHA-256
`71fef1ede8d4c0a68c9aa84a82c0f6a8a7240202641f8879b54fe6b5395b0fd5`.
The stubbed geometry freshness, manager association and live actor identity
are not established by that caller check. A checked Rust caller model also
matched those four retained native rows; receipt
`target/cell-actor-frame-update-rust-result-v1/receipt.json`, SHA-256
`eda17e513c5080a02c01245deb8056595e9af86d78ca052ae21c92f315a19c12`.
The subsequent `E73E10` route dispatcher matched 17 controlled actor-state
combinations in both directions, using guarded route stubs. Receipt
`target/cell-selector-route-result-v1/receipt.json`, SHA-256
`5fb1474c53f2474ff4e78d3de9f89956262030131b57a212a4db2081d1a35431`.
The called contact routines, actor-state producers and live identities remain open.

The `E6ADE0` feature-loop skeleton matched five controlled cache cases against
instrumented original instructions, including empty and skipped lists. Geometry
regions were cut out, relation/cache calls were stubbed, and one otherwise
unused local was initialized by the harness. Receipt
`target/cell-feature-loop-skeleton-result-v3/receipt.json`, SHA-256
`37b8f6f7b6be5e0370ea453bc212c4b2dc030ef7b9f96bac373df42d5de19a79`.
This establishes the checked traversal only, not contact or live actor state.
The bounded Rust traversal matched those five retained rows; receipt
`target/cell-feature-loop-rust-result-v1/receipt.json`, SHA-256
`6641bf3667e1440b60e56bb39379b7110a761660cd8f4cdd7fc73692792571ae`.
The opt-in installed-data diagnostic accepts separately supplied synthetic
class arrays only after a passing pair and both-feature route, with explicit
cache stability. Its installed-data test passed; log
`target/cell-feature-loop-installed-check-v1.log`, SHA-256
`10e48805d559319665d086a2dc104cc06de592dcaa5a88b06c3c1337fe897b68`.
Both feature-side projections were moved into shared Rust code. Their four
output words matched the retained native streams in six cases per side;
receipt `target/cell-feature-projection-core-result-v1/receipt.json`, SHA-256
`31e52fce03f737e278a0d15e6da7622c950292f830d7aa5773fb6fafc4dfd746`.
Inputs remain supplied snapshots, not a live source actor or contact.
The complete `E6ADE0` one-row-per-side route matched five controlled cases
through the first candidate call, including separated, tangent, overlap and
distance-threshold inputs. The callback was replaced with a capture-and-stop
stub; small-branch point and normal locals were not read. Receipt
`target/cell-feature-first-candidate-result-v1/receipt.json`, SHA-256
`4b90a8998e1c122c6bf6c078c5b7964b37c61d47ebb71fbe62eeefcc3f26db96`.
Source relation/cache calls were stubbed, and real contact, later iterations
and live actor identities remain unverified.
The bounded Rust first-candidate composition matched those five retained native
rows; receipt `target/cell-feature-first-candidate-rust-result-v1/receipt.json`,
SHA-256 `71157aaa7c1fc0a2ef23dc4272f47233c0986cb9e26351c6ace4acdc1cbaade3`.
Its actor tokens and definition code words are explicit inputs, and it stops
before `E6A450` rather than applying contact effects.
The original `E6A450` B7 branch and `E59B40` arbitration matched five
controlled cases, including player/nonplayer priority, a lower-depth no-write,
and early exits. Receipt `target/cell-b7-producer-arbitration-result-v1/receipt.json`,
SHA-256 `526e8aaa51d01d49d12bfa977c3ea8e817adf63793f66d3ddf3f250d4bf1f1bf`.
Actor words and geometry were synthetic; unsupported branches were trapped,
and event callbacks and the second actor pass were not executed.
The Rust B7 selector matched the same five native rows, and its bounded
first-candidate integration test passed. Receipt
`target/cell-b7-selection-rust-result-v1/receipt.json`, SHA-256
`6afe4c885ce5cbaf94fd4330d21c403a75764965c87de8ece0147420cab6f5f3`.
Actor-token-to-word mapping is supplied and stable by admission; the result is
an intermediate selection, not a final event or live contact.
The original two-pass `E7F170` coordinator, with scripted selector bridges into
the original B7 producer and arbitration, matched four controlled cases.
Receipt `target/cell-two-pass-result-v2/receipt.json`, SHA-256
`f64cfb93c2ae53cb448a7bb2bac57a7dd039f2f78b7c84d400b51ac0f3842ef4`.
Resolver, geometry, cleanup and event calls were controlled or stubbed; the
event was captured, not dispatched, and no live contact is established.
The corresponding checked Rust composition matched all four retained rows;
receipt `target/cell-two-pass-rust-result-v1/receipt.json`, SHA-256
`0e5cf5fef2639740e68a6c08de7a78845e821a8408dfcbfdef5337345dcca111`.
The equal-ID control is a checked rejection matching the source short circuit;
source actors, resolver, callback and event execution are not established.
Selector 19 follows the `E7B600` removal-related route, rather than the
separate selector-10 event path. Its original flag gates and pending `E780D0`
call arguments matched four controlled cases; receipt
`target/cell-selector19-gates-result-v1/receipt.json`, SHA-256
`c3ec1af99709c4f0b21daef9a267db9b69ef21786099e920061a9faf2db17b84`.
The resolver and removal calls were stubs, so no lifecycle or side effect is
established.
The checked Rust continuation matched those four retained rows; receipt
`target/cell-selector19-gates-rust-result-v1/receipt.json`, SHA-256
`0e5cf5fef2639740e68a6c08de7a78845e821a8408dfcbfdef5337345dcca111`.
Its resolution and holder/cleanup counts are supplied stub admissions.
The original `E780D0` prefix and complete `B71CC0` validator matched six
controlled pool/player-word cases, including index equal to the native bound
with that slot explicitly allocated. Receipt
`target/cell-e780-prefix-result-v2/receipt.json`, SHA-256
`7501a8bd821254a09f0ca0fa5331da1be88895da57c50baafeb49871bb4ae408`.
The test stops before the fresh actor lookup and teardown; it is not a live
pool, lifecycle or removal result.
The checked Rust projection matched the six retained native rows; receipt
`target/cell-e780-prefix-rust-result-v1/receipt.json`, SHA-256
`ca08e20b0c3a9c515bb6dcda8b6221ea945313c8166f5b3de5c1c3ad1518e6b0`.
Its equal-bound case requires an explicitly admitted readable slot.
The subsequent original fresh lookup and strict root-unlink branch matched
two controlled cases with null and non-null prior free heads. Receipt
`target/cell-e780-root-unlink-result-v1/receipt.json`, SHA-256
`8049f31c92c59a8aaf472e6a80c60d4a861d897406b2252623fe3d156cde6d27`.
The caller supplied disjoint structures and a validated, stable actor slot;
nonroot unlink, actor ownership, later teardown and live state remain open.
The checked Rust projection matched those two retained native rows; receipt
`target/cell-e780-root-unlink-rust-result-v1/receipt.json`, SHA-256
`24ea09b31208ca7bdbbe2aaea8b07f2f8634370d88df495d8a1ff0f9e225d746`.
The following actor `+364` sentinel/manager handoff matched four controlled
retained-removal states and a no-manager sentinel control. Receipt
`target/cell-e780-manager-handoff-result-v1/receipt.json`, SHA-256
`ff7e844ed05c7adeaf8d1f6f09ce71019b4989c2222c8abb74ffbbd56d7116f4`.
The test stops before actor `+248`; manager and slot ownership are admissions.
The checked Rust handoff matched all five retained native rows; receipt
`target/cell-e780-manager-handoff-rust-result-v1/receipt.json`, SHA-256
`44d1930d2eed44a51cbf6113b6d26f43551a8274d363d6d0363aac91c3942023`.
The separate original `E59080` component cleanup and its two pool helpers
matched three controlled first-invalid, second-invalid and both-valid cases;
receipt `target/cell-e590-component-release-result-v2/receipt.json`, SHA-256
`41eb94931f99796892a56479c9640ea60011862254bc706b44996643e747b150`.
The pools, slots, records and linked actor were supplied as stable disjoint
objects. The neighboring sentinels are not guard pages or a full write audit;
the result does not establish the actor's live ownership or complete teardown.
The checked Rust projection matched the three retained native rows; receipt
`target/cell-e590-component-release-rust-result-v1/receipt.json`, SHA-256
`b27ed901cfc12aa231ce6eec1a3d82beff430f1767943e0f85d09ad6a7782761`.

After the material-preview and index-copy changes, `tools\build.bat test
--workspace --all-targets` passed 1,584 test executions across 280 groups, with
zero failures and 43 ignored. Log `target/workspace-tests-20261109-v25.log`,
SHA-256 `32f4274c0621637ef74269cdd41b0f8233ab71006513c0a3ac3087f4c6963e68`.
Later changes require their own checks.
After the synthetic-palette preview, gradient/normal models and checked
two-frame diagnostic, a subsequent `tools\build.bat test --workspace
--all-targets` passed 1,595 test executions across 284 groups with zero failures
and 43 ignored. Log `target/workspace-tests-20261109-v26.log`, SHA-256
`3d16a27696079ed8982d7339e957cd398645886b9805a214a340324fbad76be9`.
The later v27 suite, after the pending-update, removal and sorted-crossing models,
passed 1,603 test executions across 289 groups, with zero failures and 43
ignored. Log `target/workspace-tests-20261109-v27.log`, SHA-256
`6c18761e553a57494c9a2c4caa8a62ae6d7252eb8b84c5ab77c03b27332c1674`.
The subsequent v28 suite, including the checked multiframe diagnostic, passed
1,607 test executions across 291 groups, with zero failures and 43 ignored.
Log `target/workspace-tests-20261109-v28.log`, SHA-256
`158cd1b83b33f74a90503d60f67a11de0159723987f623a38d4f5a81beaa236b`.
After the opt-in installed-data multiframe integration, v29 passed 1,611 test
executions across 292 groups, with zero failures and 44 ignored. The new ignored
installed-data integration test was also run explicitly and passed. Log
`target/workspace-tests-20261109-v29.log`, SHA-256
`715169fac9a4912d026b0a85a189ea5bbdaff7db948fef8107108444534db989`.
The installed-data test was independently retained in
`target/cell-installed-multiframe-check-v1.log`, SHA-256
`cb5424382d117f6b4136c9028cb40dd5fcc27941961f7dfb62e7758c9be3b71e`:
`tools\build.bat test -p spore-bevy cell_source_sweep_fixture_v1::frame_fixture::tests::installed_multiframe -- --ignored`.
After the opt-in source-route fixture, v30 passed 1,614 test executions across
294 groups, with zero failures and 45 ignored. Log
`target/workspace-tests-20261109-v30.log`, SHA-256
`76622564b60f0cab924eaea6b2e58de254488df0ab9339967b16b071d0589825`.
Its installed-data route test was run separately and passed; retained log
`target/cell-selector-route-installed-check-v1.log`, SHA-256
`f3ef666cf30c9b6373e976166d0ed3fbc4aebe8cd718198a54997382a266ef62`.
After the feature traversal and shared projection, paired topology builder,
and source cursor inspection, v31 passed 1,621 test executions across 298
groups, with zero failures and 46 ignored. Log
`target/workspace-tests-20261109-v31.log`, SHA-256
`1132fd6f3fe8c7cc922c5a823a7919cf711a8f7d30f9b36ff1b021d05e5cb1b6`.
The new installed-data checks were also run explicitly as described above.
After the mixed source roster, first-candidate composition and bounded creature
sort/smoothing work, v32 passed 1,628 test executions across 301 groups with
zero failures and 46 ignored. Log `target/workspace-tests-20261109-v32.log`,
SHA-256 `e0d725d48c1ea571c932f5535cd772200cee42f57eee3dc2b523b56b25979903`.
At the time of v32, the smoothing module had only passed smoke tests; its
later bounded native comparison is recorded below.
After the B7 selection composition and source-plane inspection example, v33
passed 1,632 test executions across 304 groups, with zero failures and 46
ignored. Log `target/workspace-tests-20261109-v33.log`, SHA-256
`cedec895f798d265e86d70dd1f7848e734d199f88d16fa4ab99654ee0bf11795`.
After the scripted two-pass composition and caller-factor comparison example,
v34 passed 1,696 test executions across 306 groups, with zero failures and 49
ignored. This count includes tests repeated by example targets. Log
`target/workspace-tests-20261109-v34.log`, SHA-256
`4a18880a689c71d708426976067ecc05d127c6235bde8c1ca03264616b07e226`.
After the source-plane app inspection, corner joins and actor-operation
projections, v35 passed 1,771 test executions across 311 groups, with zero
failures and 52 ignored. Example targets repeat some tests. Log
`target/workspace-tests-20261109-v35.log`, SHA-256
`093f091427a961d429235e371d6f2ad6b30de994b5dda7e0eff329a520a02dd7`.
After the checked manager handoff, edge predicate and plane-only material, v36
passed 1,774 test executions across 313 groups with zero failures and 52
ignored; example targets repeat some tests. Log
`target/workspace-tests-20261109-v36.log`, SHA-256
`10c1bc4c71b8543ea6c44dc809bc3517bbf549904da982425c327799aef905a4`.
After the component-release, face-orientation, cache-hit and effect-curve
models, v37 passed 1,779 test executions across 318 groups, with zero failures
and 52 ignored; example targets repeat some tests. Log
`target/workspace-tests-20261109-v37.log`, SHA-256
`70fa7c3b8b1fca342eb26e513a2681d031a6b484259146e2480270c96ca02f86`.
After the UV selector and effect-pool route, v38 passed 1,783 test executions
across 320 groups with zero failures and 52 ignored; example targets repeat
some tests. Log `target/workspace-tests-20261109-v38.log`, SHA-256
`4193270535876aed2192f6a3437140fa81b58856eea72062cac19ddd100216f0`.
After the UV-miss and zero-weight handoff models, v39 passed 1,786 test
executions across 322 groups, with zero failures and 52 ignored; example
targets repeat some tests. Log `target/workspace-tests-20261109-v39.log`,
SHA-256 `ee8b49c74fa409cc7005198273457e1f833e00f419fb3d36b5419f2fe2910e21`.
After the corrected placement, seed/flood models and opt-in second-effect app
route, v40 passed 1,791 test executions across 325 groups, with zero failures
and 52 ignored; example targets repeat some tests. Log
`target/workspace-tests-20261109-v40.log`, SHA-256
`c46a177bddccb5ae0dc7e701b0cf08ab1864dcea9d0a7106a228be7a9aaba4f7`.
After the single-group Rust composition, v41 passed 1,792 test executions
across 326 groups with zero failures and 52 ignored; example targets repeat some
tests. Log `target/workspace-tests-20261109-v41.log`, SHA-256
`97549c44ad3d7b9a16087391a961abbd0202fd23bad10a880f7efbe10cdfe7ef`.
After the tier/scale model, v42 passed 1,793 test executions across 327 groups
with zero failures and 52 ignored; example targets repeat some tests. Log
`target/workspace-tests-20261109-v42.log`, SHA-256
`60d2be2617a39836ee0300ca2b2593d227a2766372256c08ceda039086d3555d`.
After the selector and existing-effect handoff models, v43 passed 1,797 test
executions across 329 groups with zero failures and 52 ignored; example
targets repeat some tests. Log `target/workspace-tests-20261109-v43.log`,
SHA-256 `2f57739c3b00454fb15a4797d2aa641ab473b904cee466ee23c7f47304e4d6ea`.
After the model-only multi-group composition and checked ending gate, v44
passed 1,800 test executions across 331 groups with zero failures and 52
ignored; example targets repeat some tests. Log
`target/workspace-tests-20261109-v44.log`, SHA-256
`f4bac638d30e1891c564bea45c3b9e20aa8f37b9b24223fc0253bc63c760d27f`.
After the checked player-state gate, v45 passed 1,802 test executions across
332 groups with zero failures and 52 ignored; example targets repeat some
tests. Log `target/workspace-tests-20261109-v45.log`, SHA-256
`7cd3f88b006d7a9f51a6a36857917a21719968b63ada0c37b61e9a781eb01cca`.
After the transition-player gate, v46 passed 1,803 test executions across
333 groups with zero failures and 52 ignored; example targets repeat some
tests. Log `target/workspace-tests-20261109-v46.log`, SHA-256
`43baac64e9f2edb351f4a4c5acde897ff26de91a49857d7c165c3f07ce9c15e9`.
After the transition event/player composition, v47 passed 1,804 test
executions across 334 groups with zero failures and 52 ignored; example
targets repeat some tests. Log `target/workspace-tests-20261109-v47.log`,
SHA-256 `e2a3148196a3215d42a086f30573afb78fb8008bc375ab6a32674a9e1b19f8db`.
After the checked atlas-bounds model and refreshed event comparator, v48
passed 1,806 test executions across 335 groups with zero failures and 52
ignored; example targets repeat some tests. Log
`target/workspace-tests-20261109-v48.log`, SHA-256
`5646acc7249dc4a87c389c5c3c3e12463ff91f7fd2cd67232b3396a8e42f177c`.
After the mode-manager and shelf-append models, v49 passed 1,810 test
executions across 337 groups with zero failures and 52 ignored; example
targets repeat some tests. Log `target/workspace-tests-20261109-v49.log`,
SHA-256 `6eeec71ce3ef819dfeb28ba7fa689dbc4683efddf75fdb87d9463774e807afaf`.
After the direct-name, timeline-sender and atlas-sort models, v50 passed 1,814
test executions across 340 groups with zero failures and 52 ignored; example
targets repeat some tests. Log `target/workspace-tests-20261109-v50.log`,
SHA-256 `294492e8c01a0f8895b1625d3a4d868818d3715d2c5e6734708458d5711bc2ff`.
After the receiver and generic mode-name models, v51 passed 1,816 test
executions across 342 groups with zero failures and 52 ignored; example
targets repeat some tests. Log `target/workspace-tests-20261109-v51.log`,
SHA-256 `73e427593bacfa9e033364f0989a7f3571ca57025c09a31bbc721f6f2851c0f2`.
After the record-producer and shelf-selection models, v52 passed 1,819 test
executions across 344 groups with zero failures and 52 ignored; example
targets repeat some tests. Log `target/workspace-tests-20261109-v52.log`,
SHA-256 `ed4a5764e26b7313a3e0e79b6c45e602a29cf2c2e75bc72544a0d8f66fa6dc95`.
After the checked record initializer, v53 passed 1,820 test executions across
345 groups with zero failures and 52 ignored; example targets repeat some
tests. Log `target/workspace-tests-20261109-v53.log`, SHA-256
`4ca509549442ebe8ff58288894c896a8738dabe3c404344e03591921d619ee4e`.

The first original-instruction cube-walk comparison failed its fixed-address
allocation check before entering the source instructions; its failure receipt is
`target/creature-skin-cube-walk-native-20261108/result.json`. A corrected host
with aligned allocation subsequently matched all 256 classifications and two
callback controls. The comparison supplies model-generated tables to both
implementations; it does not independently validate the original table initializer,
native allocation, queue traversal or a finished creature mesh.
The first independent initializer host also exited before source execution
because its fixed code address was unavailable; its preserved failure is
`target/creature-skin-cube-tables-native-20261108/result.json`. A reviewed
relocated-host comparison subsequently matched all 880 initialized table bytes,
using a zeroed synthetic frame and globals, original edge pairs and relocated
switch targets. It does not establish the runtime queue or finished mesh.
The source-derived tables and one-cube walk are now composed with the partial
edge diagnostic under caller-supplied field and classification inputs; 11
focused tests pass. An initial synthetic triangle expectation failed because an
exact-zero midpoint legitimately collapsed the fan; that case is now tested
separately from a distinct-triangle case. At that point, full-queue and
whole-composition comparisons were still open; the later bounded comparisons
are recorded above.

The experimental Cell frontend now has an offscreen-only, explicitly selected
kind-0 source-fragment-inspired player preview. The source asset note records
the matched-depth D3D9/Bevy fixture comparison and the correction that earlier
native captures had depth testing disabled. A tick-0 RTX Vulkan application
pair exited 0 with both player models visible at the controlled center and no
pipeline or image errors; 372 pixels changed within the player region. Receipt
`target/source-cell/fragment-preview-player-frozen-v1/receipt.json` SHA-256
`ac118c6356ccca30c39261846b457be996cffe4d0721a53694bd684afcbec907`.
Both runs warned that the Vulkan validation layer was unavailable. This is a
white-tint, unweighted bind-position approximation with Bevy camera, depth
comparison and ordering; it does not establish live state or retail parity.
See [the asset note](../re/cell-kind0-asset-intake-v1.md).

The opt-in five-segment UI inspection also accepts an explicit static cursor
progress and target-pixel vertical coordinate. The installed-data test checks
the source child rectangle and atlas coordinates. Paired offscreen RTX Vulkan
captures exited successfully and differed at 455 pixels within the expected
cursor rectangle; receipt `target/source-ui/cursor-app-pair-20261109-v1/receipt.json`,
SHA-256 `7d450408f765fdd332d3031b489994009b11c8f8f08a6f0b3c5e259d792cade4`.
The screenshot's surrounding scene remains prototype artwork. Cursor animation,
visibility, native placement and source-backed progress remain unverified;
the Vulkan validation layer was unavailable in both captures.
An additional opt-in mixed rival roster loads each explicitly named Cell
model and its own atlas before launch. A paired RTX capture using two source
IDs assigned to controlled prototype actors changed 1,043 scene pixels;
receipt `target/source-cell/mixed-roster-app-20261109-v1/receipt.json`, SHA-256
`88ab8062536432e6d5de767327d4e6b074a99dff16cb92fc936725b7dd533526`.
An invalid second ID failed before GPU startup and produced no screenshot.
Prototype spawning, camera, simulation and static PBR rendering remain; this
is not a source scene or native selection claim. Both captures reported that
the Vulkan validation layer was unavailable.
An independently selected Cell effect-map chain resolves to installed model
`44C489B7`, a textured four-vertex plane. Its source-only inspection validates
the saved mesh, UVs, stage-0 raster/sampler and nine texture mips. A controlled
RTX Vulkan textured/white pair passed and changed 146,199 pixels; receipt
`target/source-cell/effectmap-plane-inspection-20261109-v3/receipt.json`,
SHA-256 `02a2e35ad30c9cc3052e5061adaf2ef3b404b53de2b92550c769a24dd2de4079`.
The example uses PBR and assumed sRGB with caller-supplied camera and light;
the original shader, dynamic effect placement and live provider remain open.
The validation layer was unavailable. See
[the background-model note](../re/cell-effectmap-background-model-boundary-v1.md).
The candidate cached original shaders for the same plane also rendered on RTX
D3D9 with explicitly supplied matrices, white tint, saved sampler and depth
state. A same-size, same-format, nine-mip white BC1 control differed at 146,107
pixels; receipt `target/source-cell/effectmap-plane-native-result-v2/receipt.json`,
SHA-256 `feb131a56e4e0ca4d4487ceb85a00252e66e461f0fc9e8b10a090c7948fc2e92`.
Actual shader selection, native effect placement and a matched-state Bevy
comparison remain unverified. The PBR and native images use different lighting
and color assumptions and cannot establish shader-only differences.
A subsequent matched-input Bevy/WGSL comparison had identical 147,456-pixel
foreground coverage; the maximum channel difference over 145,924 common
interior pixels was four levels. Receipt
`target/source-cell/effectmap-plane-bevy-result-v1/receipt.json`, SHA-256
`c49cff66581cdcc041527c753b977e9032d94262280417fb168faeba04a78bd1`.
The compared paths used D24S8/hardware BC1 and Depth32Float/CPU-decoded BC1,
respectively; full shader equivalence and live effect selection remain open.
A controlled sRGB-target fixture with inverse transfer and opaque source mips
also matched coverage exactly; its maximum channel difference over 145,924
interior pixels was three levels. Receipt
`target/source-cell/effectmap-plane-srgb-result-v1/receipt.json`, SHA-256
`0aeabf8bc6231dc12ddd5ebb2728c12dcfff08bed1342244e603cbba3035c238`.
Depth formats and BC1 decode paths still differ, and this does not validate
the app's reverse-Z or general alpha blending.
The controlled plane is also available in the experimental Cell app with an
explicit world rectangle. Earlier app captures were occluded or black; after
fixing the prototype overlay and camera MSAA mismatch, the source texture was
visibly rendered behind the prototype scene. Two same-seed, same-tick captures
with different supplied rectangles changed 489,637 pixels within the larger
plane footprint. Receipt `target/source-cell/effectmap-plane-app-20261109-v4/receipt.json`,
SHA-256 `5945ed39931b5966a5651f46aaa26e9b40f1e4202a52465da472aada43001b18`.
The app uses an sRGB target and Bevy reverse-Z; it does not inherit the
standalone forward-depth comparison, and retail placement remains unverified.
After separating the plane-only inverse-sRGB material from the player shader,
the fresh app rectangle-control pair changed 489,641 pixels. Receipt
`target/source-cell/effectmap-plane-app-20261109-v5/receipt.json`, SHA-256
`af2c4d716d18ca6239a7ad507d1df01fd9681184dd182cf1064e07b7c2a89b83`.
The loader requires opaque decoded mips for this path; the shader comparison
does not establish the app's camera, placement, reverse-Z or live effect state.
The second effect-map model `154C862B` has 1,701 vertices, 3,200 triangles
and two saved BC3 textures. A controlled original-template-generated D3D9
program rendered it on the RTX with supplied matrices, state and tint. A white
stage-0 control changed 147,456 pixels; a white stage-1 control changed none
under that generated recipe. Receipt
`target/source-cell/effectmap-154-native-result-v1/receipt.json`, SHA-256
`9fbd1f57861a404df0588bd85734a239d511fbb24ec12884819e5507f4bf8471`.
The selected recipe, transform and effect are conditional, and the result
does not determine the active retail provider, shader, camera or placement.
The matched-input Bevy/WGSL comparison had the same 147,456 foreground pixels
and no coverage disagreement; 145,924 common interior pixels had a maximum
channel difference of three levels. Receipt
`target/source-cell/effectmap-154-bevy-result-v3/receipt.json`, SHA-256
`706c8cee43c4cc8b220267e3c33c75d3f634ea5d65f05e4f0e249279736e3af9`.
The paths use hardware BC3/D24S8 versus CPU-decoded BC3/Depth32Float. This
unorm-target comparison does not establish the app's sRGB blending or live
selection, and the unavailable Vulkan validation layer remains a limitation.
A separate inverse-transfer sRGB-target control for this translucent effect
also matched foreground coverage, but 731 of 145,924 common interior pixels
had a maximum RGB delta greater than three and the largest was eight; alpha
delta was at most one. Receipt
`target/source-cell/effectmap-154-srgb-result-v1/receipt.json`, SHA-256
`df8df7c8719d3053e16c8a4c8f362571b668be4248d8912339ceb48c9e1c57fd`.
This single controlled destination does not establish general blending parity
or the app's reverse-Z scene composition.
In a separate two-pass control, drawing the whole test scene to unorm and then
converting the finished image to sRGB preserved the intermediate bytes exactly.
Both stages matched the native foreground coverage; over 145,924 interior
pixels the maximum RGB difference was three and alpha difference one. Receipt
`target/source-cell/effectmap-154-twopass-result-v1/receipt.json`, SHA-256
`d9ccbf456de08b7acd5fd9e145e37430f5e01b15bc7379dbbfd34d4585a23457`.
The controlled test does not establish composition with app actors, UI or
reverse-Z depth.
The experimental Cell app now offers an explicit offscreen-only `154C862B`
inspection, rendering the scene to unorm before a final sRGB conversion.
Its supplied-rectangle/no-effect pair changed 489,642 pixels within the
effect footprint; receipt `target/source-cell/effectmap-154-app-20261109-v1/receipt.json`,
SHA-256 `7b920801dc744ab9651c8560071ff3ec3888882af35138b4eeb0861fc447801a`.
A separate controlled source-UI/cursor pair retained the actor and effect
composition; receipt `target/source-cell/effectmap-154-ui-20261109-v2/receipt.json`,
SHA-256 `2f7c4a85a000a86c09b00945965f39030c0d7041cab9df25cc6734a0ca076ed1`.
The earlier UI comparison script's failed v1 is retained. The default and
first-effect reference captures were byte-identical after the changes. The
unorm scene makes the prototype visibly darker; effect placement, live shader
selection, app depth parity and actual UI state remain unverified.
Both effect-inspection option combinations documented in the README also
completed separate RTX captures at 90 prototype ticks without source-player
options. Receipt `target/source-cell/readme-effect-examples-20261109-v1/receipt.json`,
SHA-256 `7f942d6482a82cef16336a4339ef5eff9342949ad9e1cfa379689fd1b4705d78`.
An additional paired RTX capture combined the same source plane and two-rival
roster with the five-segment source UI and supplied cursor. Both runs exited
zero; enabling that UI changed 25,744 pixels inside x=47..1034, y=40..116 and
left the remaining RGB pixels unchanged. Receipt
`target/source-cell/composed-source-controls-20261109-v1/receipt.json`, SHA-256
`4078c6bfd032f0f0763654e7b48e1993d0b954269a8e423cc37b81b8a6af6bb9`.
The fixture still supplies UI state and placement, and the scene remains a
prototype. The Vulkan validation layer was unavailable.
The same controlled composition also completed a separate one-second RTX
prototype demo, reporting 60 simulation ticks and five prototype feeding
events. Receipt `target/source-cell/composed-source-demo-20261109-v1/receipt.json`,
SHA-256 `9ec6aa0243062cd18cca7f51ee9c76f537255efba87689c0bd9b35fed679ffff`.
Its animation, decisions, controls and source UI state do not establish
original-game behavior.
Original `E50080` and its helper matched 92 controlled cases for the two
installed effect-map rows, including boundary inputs at x87 controls 027F and
037F with MXCSR 1F80. Receipt
`target/source-cell/e50080-selection-result-v2/receipt.json`, SHA-256
`d5e218d29f45474da58ac0dff03efd1dc53fdb7e4b2574fb826d1ddcd88075f0`.
These are supplied scalar values; live provider, pool state, effect placement
and scheduling are not established.
The checked Rust curve matched all 92 retained native results; receipt
`target/source-cell/e50080-selection-result-v2/rust-curve-run-v1/receipt.json`,
SHA-256 `c46c239b9996f1a45570ddc05166b7b61d062a8288c8e320414f3e9254f67121`.
Its separate join with the executable's source start table matched an independent
calculation for all 1,001 supplied progress inputs; receipt
`target/source-cell/e50080-selection-result-v2/rust-join-run-v1/receipt.json`,
SHA-256 `f2b6597dcf1bfe9e9955508e9f303df132adfef974973d04dc8911f15d51f69e`.
This composition assumes the startup value remains stable and the selected
installed rows are used; actual progress, provider and pool remain unobserved.
The original `E5E100` per-candidate decision prefix matched nine supplied
type, level and finite-weight combinations, including signed zero and three
retained curve outputs. Receipt `target/cell-e5e100-route-result-v2/receipt.json`,
SHA-256 `6804fa8d5f4a2dbfddc9c6add921444f7a13fef45edd1e608f54ca57f2777293`.
The comparison cuts before cleanup or placement; pool traversal, allocation,
callbacks and actual active candidates remain open.
The checked Rust route matched all nine retained native rows; receipt
`target/cell-e5e100-route-rust-result-v1/receipt.json`, SHA-256
`f2a8389570250b7c4e20d2163c5dcce0f71319f7fabce8715f03044a196183ca`.
Its optional start-table and curve composition still requires supplied stable
values and does not observe or traverse a runtime pool.
The original zero-weight cleanup slice matched four scripted null, retained,
callback-cleared and callback-replaced cases, including the pointer re-read and
clear-before-second-callback order. Receipt
`target/cell-e5e100-zero-result-v1/receipt.json`, SHA-256
`3843f19f3dcf2a8d6c7963531b837d302f9572c093722eba09b708f1bf960e1e`.
Callbacks and object lifetimes were supplied synthetic fixtures, not observed
destruction or live ownership.
The checked Rust projection matched the four retained callback-order rows;
receipt `target/cell-e5e100-zero-rust-result-v1/receipt.json`, SHA-256
`0e5cf5fef2639740e68a6c08de7a78845e821a8408dfcbfdef5337345dcca111`.
It returns the second callback as pending after the slot clear, without
asserting that callback's effects or the subsequent slot state.
The original nonzero placement prefix matched four controlled level, grid,
scalar and existing/absent-object cases, preserving its floating-point order
with a synthetic scale-helper result. Receipt
`target/cell-e5e100-placement-result-v1/receipt.json`, SHA-256
`75ee0aebc515b3b1cec24315cb19c893d6ac9de72b02fbab251824c174125e64`.
Both paths cut before object creation/update; real helper state, global
ownership, callbacks and live effect placement remain unverified.
The checked Rust placement-prefix projection matched the four retained native
rows; receipt `target/cell-e5e100-placement-rust-result-v1/receipt.json`,
SHA-256 `0e5cf5fef2639740e68a6c08de7a78845e821a8408dfcbfdef5337345dcca111`.
Review subsequently found that the model ordered the independent grid SSE
work after, rather than before, the helper multiplication. After correction,
the four retained native rows matched again; the v2 receipt is
`target/cell-e5e100-placement-rust-result-v2/receipt.json`, SHA-256
`0e5cf5fef2639740e68a6c08de7a78845e821a8408dfcbfdef5337345dcca111`.
The helper output, global state and effect presence remain explicit inputs.
The separate original `E51E50`/`E4ECC0` helper matched 4,004 controlled
level/progress/x87-control cases using the installed 22-row table and synthetic
Cell/save pointers. Receipt `target/source-cell/e51e50-tier-result-v2/receipt.json`,
SHA-256 `2c1b3b6d2c3437403e5cf16ce9b6122f4882df2ff509e22f3351366a04fb6f1b`.
The fixed corpus scan was checked before execution. Actual save ownership,
progress, call reachability and state stability remain unverified.
The checked Rust tier/scale model matched all 4,004 retained native results
and composed with two separately retained placement cases; receipt
`target/source-cell/e51e50-tier-rust-compare-v1/receipt.json`, SHA-256
`971c29e2c36f9c7c74ed5f09537f953e46ee039ddfddcabd9468e6f80850acf3`.
The composition supplies progress and global scale independently; it is not
an observed contiguous native call or live-state claim.
The original existing-effect callback handoff matched four supplied nonzero
weights and a synthetic stable object/vtable, including receiver, argument
address and stack cleanup. Receipt
`target/cell-e5e100-existing-result-v3/receipt.json`, SHA-256
`a96afe306fda86d3dd206733f94b1babd8973f617b8fc3b6ae0f8b4f46cabb4a`.
Actual callback effects, object creation and live lifetime remain open.
The checked Rust handoff matched all four retained native rows; receipt
`target/cell-e5e100-existing-rust-result-v1/receipt.json`, SHA-256
`53cf86042bcc0883b0d33f6be4cb969c916c39e94141e35002acc3a036bd6d40`.
It stops at a pending callback with explicitly admitted stable tokens.
The original completion gate at `E7D043` matched 27 scripted combinations of
fresh progress and globals reads, with the ending call trapped. Receipt
`target/source-cell/cell-ending-gate-result-v1/receipt.json`, SHA-256
`55c75aacf7e1ce969bef017bab7fae86950fef3d5e170134465d31a30a63ca61`.
This does not establish actual ending effects or entry into Creature stage.
The checked Rust completion gate matched all 27 retained native rows; receipt
`target/source-cell/cell-ending-gate-rust-compare-v1/receipt.json`, SHA-256
`f2ba2ff8585c9d4a94d9087371ceb54d16e8f1a16edadf79244a47ff3e947f42`.
Original callback effects and transition behavior remain pending.
The original `E6E920` prefix matched 20 scripted globals/flag/player cases;
both the effect branch and successful-player continuation were trapped before
effects. Receipt `target/cell-e6e920-prefix-result-v3/receipt.json`, SHA-256
`436e7053fa2ee92b137d680ca0fd5266283ef8616613fffad62b777524a136ef`.
Resolver/pool behavior, live actor identity and ending execution remain inputs
or pending actions, not a completed transition.
The checked Rust prefix matched the 20 retained native rows; receipt
`target/cell-e6e920-prefix-rust-result-v1/receipt.json`, SHA-256
`5107260178af5568d5910a477b2c1330cfc307925aac718da56aac9efa0619c8`.
It preserves the pending effect/player boundaries and requires supplied stable
snapshots.
The transition-button player's original `E5D8F0` gate matched four scripted
lookup/flag cases and its conditional root +5158 write; both surrounding
calls were trapped. Receipt
`target/source-cell/transition-player-gate-result-v1/receipt.json`, SHA-256
`6d0b43dd3a338b19dc6902764e3a79a84393e37ca0b5cc34c6d351b8a71006d9`.
The handler's side effects, actual UI dispatch and stage switch remain open.
The checked Rust player gate matched the four retained native cases; receipt
`target/source-cell/transition-player-gate-rust-compare-v1/receipt.json`, SHA-256
`f04f69d7a2ac71a9ddd349500c14e435c35978617de40eb799b1c0a3c44d8e28`.
Root and player snapshots are supplied after the unmodeled prior helper and
lookup; the following call remains pending.
The original transition-button event dispatcher matched four synthetic
type/ID cases with all action calls trapped; receipt
`target/source-cell/transition-event-gate-result-v2/receipt.json`, SHA-256
`8e60d318cb7a943adb66c3c16cd295d5242c53e1133756b7abf24b67e0dec767`.
This validates the bounded dispatch branch, not an actual UI event or stage
switch.
The checked Rust dispatcher matched four retained event rows and four separate
player-gate rows, with 16 modeled pairings; v2 also checks the player floating-
point control words directly. Receipt
`target/source-cell/transition-event-gate-rust-compare-v2/receipt.json`, SHA-256
`f41850cc4810547d60296c07a9190e0afc49dc19e0b0d0ab41d7f96793c76437`.
The pairings are not a contiguous native-call comparison.
Static analysis connects a conditional Cell path to a request for mode ID
`01654C01`, registered as `Game_Creature`. The original manager routine
`B1E280` matched six scripted current/requested/sentinel cases with service
callbacks stubbed; receipt `target/cell-b1e280-manager-result-v1/receipt.json`,
SHA-256 `aaf249ae8e02b2b3ced78dcbab8da683ca7a86cf368642950872eece6c3c5628`.
It requests service work and sets a manager flag; actual mode completion and
live scheduling remain unverified.
The checked Rust manager projection matched all six retained native cases;
receipt `target/cell-b1e280-manager-rust-result-v1/receipt.json`, SHA-256
`6c03a07cfe5f91c4122158962dc3bb1590136339f424c85bc4326472bdd1b319`.
Service effects and mode-switch completion remain pending.
The separate original `B1DF30` direct-name branch matched six controlled
editor/load requests with a synthetic service; receipt
`target/cell-b1df30-direct-result-v1/receipt.json`, SHA-256
`0a837773ff049dbae0f23cc0676f6aa41b7b35ef07c583c9f5f8e1961437cce4`.
The generic `CreatureGame` plus `FromCellGame` construction and service
execution remain separate, unverified boundaries.
The checked Rust direct-name projection matched all six retained rows; receipt
`target/cell-b1df30-direct-rust-result-v1/receipt.json`, SHA-256
`914fc4d859d69fbcaf82f6e634b28c7e7daf7db7cc939898d6c2cd7bcb54f02a`.
Generic string construction and real service lookup remain pending.
The original generic Cell-to-Creature branch subsequently reached a synthetic
service with `CreatureGameFromCellGame` in one controlled case, using bounded
scripted string/cleanup helpers. Receipt
`target/cell-b1df30-generic-result-v1/receipt.json`, SHA-256
`d94e01eb6ac96d5fe685100aeac0b05a9166d92d14300c8a398b4a441a6fbdf8`.
The original allocator, string helpers, service lookup and completed mode
switch are not established by that fixture.
The original `7E3868` service-token equality gate matched six controlled
current/requested cases, trapping both continuations. Receipt
`target/cell-service-action-gate-result-v1/receipt.json`, SHA-256
`d695036b8b2ab69fb3434ac5013b14c7bf0ff7ec8e41886917920d9d7697e07c`.
Prior exception-frame setup, service side effects and live mode completion
remain unverified.
The checked Rust service gate matched the six retained native cases; the
joint UV/service comparison receipt is
`target/pending-models-verify-20261109-v1/receipt.json`, SHA-256
`c95ea553b4847611ef607901e8140b9e79916db1220657cd9fea55cdb2a01166`.
The checked Rust generic projection matched the retained controlled row;
receipt `target/cell-b1df30-generic-rust-result-v1/receipt.json`, SHA-256
`5bdb2b655435caaebdaffa1f99e0382929504ed4983fb43508ef9c140a3562fb`.
Its scripted string and service operations remain admissions.
The original timeline sender slice matched six scripted receiver/helper cases,
including clearing the sender flag after a helper mutation and before sending
message `04471C60`. Receipt
`target/source-cell/timeline-sender-result-v1/receipt.json`, SHA-256
`ade223d0417f472f5028d4463daa5897460e6a38ddca2ec498ee26395a01748f`.
The receiver, helper and service were supplied fixtures; live UI delivery and
mode completion remain unverified.
The checked Rust sender matched all six retained native rows; receipt
`target/source-cell/timeline-sender-rust-compare-v1/receipt.json`, SHA-256
`d89ddb7b4fdad1a419f751c61d2340e6ef7ed91c2853a23580c755f088c302c8`.
Its returned flag is before pending callbacks, whose real effects are unknown.
The original Cell callback receiver matched five scripted post-prefix root
values, including changed values before the fresh read; receipt
`target/source-cell/timeline-receiver-result-v2/receipt.json`, SHA-256
`d6a527127be98e8807bf4d7a04e06e466ae40da138de49115be61e3e7ae76620`.
The preceding callback work and subsequent `E72CD0` call were supplied or
trapped; actual service delivery remains unverified.
The checked Rust receiver matched all five retained native rows; receipt
`target/source-cell/timeline-receiver-rust-compare-v1/receipt.json`, SHA-256
`01741bbcc0a83916cb238f179270479233bb55957b26c9d982d0216d65a79493`.
Its input is explicitly the post-prefix snapshot; `E72CD0` remains pending.
The original `E72CD0` pre-allocation gate matched eight scripted cases,
including exact-byte flag comparisons and a fresh mode read after lookup.
Receipt `target/source-cell/e72cd0-gate-result-v1/receipt.json`, SHA-256
`7571d44dfe0cfe070f6bdc7b48ff0d9747f481fbf5e8dcc12b0013cf45c779ff`.
Effects, record allocation and real player lookup remain unverified.
The checked Rust gate matched all eight retained native cases; receipt
`target/source-cell/e72cd0-gate-rust-compare-v1/receipt.json`, SHA-256
`de980eab3fa04e9f729107f6978729ae727c439d14d50ae6a9d852d3f7325140`.
Its snapshots are supplied after lookup; subsequent effects and allocation
remain pending.
The original `E72E03` record-initialization tail matched eight controlled
records, including unchanged sentinel bytes, with supplied root, vectors,
handle and float and a trapped notification. Receipt
`target/source-cell/e72e03-record-result-v1/receipt.json`, SHA-256
`1dd6c5edabf5e0e1b152e955fcc1811dbd44057093650ca8a454d31a3e46685b`.
Native allocation, notification, scheduling and live ownership remain open.
The checked Rust initializer matched all eight retained record/root snapshots;
receipt `target/source-cell/e72e03-record-rust-compare-v1/receipt.json`, SHA-256
`a5f7df13ebbcf67335beb8626f2a3e79e03aea2ebc79a51232e2378f04671f77`.
Allocator, notification effects and scheduler ownership remain admissions.
The original pool allocation, lookup and iteration helpers matched a bounded
two-slot fixture, including full capacity and an invalidated-slot scan. Receipt
`target/source-cell/pool-allocate-iterate-result-v1/receipt.json`, SHA-256
`235b6d2d441a3935dee263f3eefe526cc85ffa9eb021b716ba35e59466174686`.
Free-list reuse, scheduler timing and actual pool ownership remain open.
The checked Rust fresh-pool projection matched the retained native sequence;
receipt `target/source-cell/pool-allocate-iterate-rust-compare-v1/receipt.json`,
SHA-256 `a8ca601207b6b265e1e3f6f7f479d560de01753d5e20e1e80dacbadc57b389a1`.
It does not model free-list reuse or establish a live scheduler.

The fragment preview now also accepts the explicit
`--source-kind0-synthetic-palette` flag. It validates the candidate's UBYTE4
indices, UBYTE4N weights and their exact geometry correspondence, and applies
the separately compared synthetic translations; this is not a live animation
pose. In a controlled tick-0 RTX Vulkan pair, both modes exited 0 and the
player was visible; 306 player-region pixels changed. Receipt
`target/source-cell/synthetic-palette-player-frozen-v1/receipt.json` SHA-256
`d523668c341c94c1f8f3d6683ff92a171e68d13c9fd116575eba02611c1488f0`.
Both runs warned that the Vulkan validation layer was unavailable. The exact
invocation, source comparison, hashes and remaining provider/camera/state
limits are in [the asset note](../re/cell-kind0-asset-intake-v1.md).

The candidate kind-0 RW4 contains a nine-matrix serialized identity buffer
linked by its SkinsInK section. An initial guarded x86 setter/upload host
failed its fixed-address allocation before executing source instructions;
receipt `target/source-cell/kind0-state4-host-result-v1/receipt.json` SHA-256
`897028cd11064330e1307860a5c40056088e3fd70e5b388d075c07a07af6b29c`.
The reviewed relocated v2 matched two cases, receipt
`target/source-cell/kind0-state4-host-result-v2/receipt.json` SHA-256
`3a711befe94f0ac32587e10b9db3cbaf7c07814ccecc52a6631a1254ce7ff222`.
With saved-state-shaped `(start,count)=(1,8)` and caller-supplied base `2`,
the callback receives c5–c28; c2–c4 remain unresolved. The buffer pointer,
caller base and active branch are fixture admissions, not live binding or
pose observations. See [the palette boundary](../re/cell-kind0-palette-source-boundary-v1.md).

For the conditional 0473 partial surface, the averaging slice subsequently
matched all 44 point accumulators and counts against original instructions:
`target/creature-skin-average-0473-native-20261109/result.json`, SHA-256
`6bef492ef52cc742f63f48df2d5ffb2e4be84f120bfc19e05d544d7d7046a997`.
The bounded nonoverlapping index-copy slice matched empty, single-index and
252-index inputs using a copy stub; receipt
`target/creature-skin-index-copy-native-20261109/result.json`, SHA-256
`30cbbcc2f76884bfe1071e47f1accef1fb40dd1b6a2a4c700be9e845e31705bb`.
The supplied grid and source-row admissions still apply; native allocation,
correction, normals and a finalized live creature remain open. With an enriched
grid whose derivative coefficients were explicitly supplied from a model,
the subsequent gradient comparison matched all 44 points; receipt
`target/creature-skin-gradient-0473-native-20261109/result.json`, SHA-256
`853c78c4ac52188efde5f4fa0fefe5304557e7cdc3e87ee5170b9887c3cb6231`.
Native construction of that enriched grid and the live source state remain open.
The direct usable-gradient normalization then matched all 44 normals against
the original instructions and pinned installed CRT; receipt
`target/creature-skin-normal-0473-native-20261109/result.json`, SHA-256
`2307929623cd4e2499cb4867bde56d6d89bd5aff0017a4edfa1a697f5111782d`.
The joined opt-in inspection artifact has 44 positions, 44 normals and 84
triangle triples: `target/creature-skin-0473-partial-inspection-20261109.json`,
SHA-256 `c72cd25939c8bae1425c6a9b5dbaf47edfb5cd1b6a87ad37df783f54894c68cf`.
It has no recovered UVs, weights or materials, and is not a finalized mesh.
The subsequent influence-query host matched 23,736 output bytes for those 44
points plus null-cell and out-of-range controls using a bounded append stub.
Receipt `target/creature-skin-influence-0473-native-20261109/result.json`,
SHA-256 `8978465edcd7c225dda41f5785e2769104ce3566caa12170d6bc07ba4243d061`.
Its ordered requests are not yet final skin weights or palette identities;
grid construction, allocator behavior and live creature identity remain open.
With explicitly synthetic endpoint coordinates and independently row-derived
edge/member/remap inputs, the follow-on accumulation and ranking instructions
matched all 44 cases. Receipt
`target/creature-skin-weights-0473-native-20261109/result.json`, SHA-256
`5a8f511b0a98b50ec8f8fe5468c3ef36ed0905689482172c89aab7b8cd82aa09`.
Outer producer execution, normalization, correction, smoothing and real palette
identity are not established by that comparison. The following initial
20-byte record writer also matched all 44 conditional ranks in a separately
bounded source-instruction comparison: receipt
`target/creature-skin-record-0473-native-20261109/result.json`, SHA-256
`361dcd3d3e11c065a124357eb30678687c43d2c2927b5a0d30ff8c71d1dbeeca`.
It still uses synthetic endpoint coordinates; correction, smoothing, palette
identity, outer producer execution and live identity remain open.
An independent original-instruction comparison then matched all 11 endpoint
coordinates derived from the retained 0473 rows; receipt
`target/creature-skin-endpoint-0473-native-20261109/result.json`, SHA-256
`d5b02ebac955bf06b379cddac20da2ab8143c24193d5d4474f602f8564f75391`.
With those row-derived coordinates, all 44 accumulation, ranking and initial
record comparisons matched; receipt
`target/creature-skin-weights-0473-rowcoords-20261109/result.json`, SHA-256
`90a1c5c82fa48258a43f5c8d79d2372029add2efaf0b943494aeda7ab0e4dffa`.
Frozen post-mutation rows, same-object/lifetime, supplied grid, no correction
and no smoothing remain admissions; palette and live identity remain open.
The separate `--include-row-coordinate-initial-records` exporter option retains
those raw record bits alongside the conditional partial geometry. Artifact
`target/creature-skin-0473-partial-row-records-20261109.json`, SHA-256
`7277fd4a0ab3f2049780dfeb58e71be78108b12535e672ba2b7bcc26c9486055`;
these are not yet final skin or identified bone-palette entries.
The isolated smoothing record merge matched four controlled two-record cases
against original instructions; receipt
`target/creature-skin-smooth-merge-0473-native-20261109/result.json`, SHA-256
`3d39a80887c69d1cdd3b86dcc30acce201ed3caeefe8e94c2261c1c1543fbaa4`.
Directed-link topology and the active smoothing iteration count remain open.
The retained 0473 index triples contain 126 distinct unordered edges, each
with exactly two oppositely directed incidences. Static tracing of original
`509D30` identifies a candidate directed-link construction for those triples.
The bounded original loop and an independent model matched 1,188 output bytes
for this paired-edge fixture; receipt
`target/creature-skin-topology-0473-native-v3-20261109/result.json`, SHA-256
`8642f23c9cd36f31456d213d5d3ae05006f4aeb33aaa3f1f61099da1005ef9a2`.
The prefix allocation and nonmanifold repair were excluded; live topology
identity and the smoothing pass were unverified at that point.
The reusable paired-edge Rust builder also matched the retained native output;
receipt `target/creature-skin-paired-topology-verify-20261109/result.json`,
SHA-256 `6a15e75a4505e6b475f300620a0b49a98ed3ea98cf24c3344e398e73767eda81`.
It rejects nonpaired, degenerate and unreferenced-vertex inputs rather than
reproducing the original allocator, partial writes or repair behavior.
The original 64-entry scratch sort matched an independent model across six
controlled cases, including equal-key and synthetic stale-tail inputs. Receipt
`target/creature-skin-sort64-0473-native-20261109/result.json`, SHA-256
`d62c64ac1bdb14ddfca4daed5acd01c3f7f5170ab8c550f8a3f5460c256a6931`.
The sort can reorder equal weights. Heap fallback, complete smoothing and
live scratch contents were not established by these cases.
The checked Rust 64-slot sort also matched those six retained original rows;
receipt `target/creature-skin-sort64-rust-verify-20261109/result.json`, SHA-256
`533bdc32cf8cf8b68ce25351f6b49542ec6a989ae07d91d89cc158b81a9a52b3`.
The persistent-scratch smoothing implementation was initially an unverified
model; the later complete-pass comparison and its limits are recorded below.
The first reviewed full-pass comparison at explicitly supplied alpha 0.5 did
not complete: the model exited 0, but the native host exited 7 before emitting
output. The cause is not yet isolated. Failure receipt
`target/creature-skin-smoothing-0473-native-v2-20261109/result.json`, SHA-256
`bd656034cafbfd827127160b16220413bf5b56ed9033a4fe82a9d0a8c85de9be`.
The optimized-Python rejection was also exercised and stopped before any child.
A separate instrumented diagnostic reached both passes and identified check 38,
the post-second-pass ESI comparison. The host changes ESI and EDI itself when
copying the first result but had saved their comparison baselines only before
the first pass; this is a harness-baseline defect, not a smoothing match.
Diagnostic receipt `target/creature-skin-smoothing-0473-diagnostic-v2-20261109/result.json`,
SHA-256 `546ebfe6fb4f5ced38615ba883a62860ebd2c4cead2ee101ddbbfc4fc8672d64`.
After correcting those baselines, the full original smoothing closure and Rust
model matched all 2,640 bytes for the same conditional records at an explicitly
supplied 0.5 factor and zero, one and two passes. Receipt
`target/creature-skin-smoothing-0473-native-v3-20261109/result.json`, SHA-256
`c3fd128b4049e4f7ee40ca8c0a8f598cfd68f89562651e5f777846f0b0d3c1f8`.
The same conditional zero/one/two-pass comparison also matched all 2,640 bytes
at the original caller's exact `3F333333` factor. Receipt
`target/creature-skin-smoothing-caller-alpha-0473-native-20261109/result.json`,
SHA-256 `fce30d8553893daa8e4c7592722ca824edf821948cae81df9e803caa063de9e6`.
The active iteration count, real palette identity and live creature state
remain unverified.
The partial geometry exporter can include the separately selected, raw 0.5
factor smoothing records for zero, one or two passes. A pass-one artifact is
`target/creature-skin-0473-half-alpha-pass1-inspection-20261109.json`, SHA-256
`a18dea7e81851754c1789ca25f54b261c3dc9650ba6f17b8ec70227b59371ee4`.
Nine exporter tests passed. The records remain conditional and do not identify
a live creature, active iteration count or bone palette.
The exporter also provides a separate caller-factor option. Its pass-one
artifact is `target/creature-skin-0473-caller-alpha-pass1-inspection-20261109.json`,
SHA-256 `6787d9565ea29f405fca688f6b3dd8ec6903ddaa3498a9b932bb09a69b6dd3fb`.
Eleven exporter tests passed after adding it; the same runtime and identity
limitations apply.
The original corner-attribute slice matched 132 explicitly mapped records
(3,168 bytes) across the three caller-factor pass counts. Receipt
`target/creature-skin-corner-attributes-0473-prep-v2-20261109/result.json`,
SHA-256 `b8ef734dfc089b6ca115b31ab97c829a0fb094d8bc77531dfa600a0e211b11ed`.
The corner mapping and append storage were supplied; the result does not
identify the native corner table, palette or live mesh.
The reusable Rust join matched those native bytes; receipt
`target/creature-skin-corner-attributes-rust-verify-20261109/result.json`,
SHA-256 `e50f27d8b25684822ea642316d06239a6d85c64dcadc020ef0ddd9a99ea901d1`.
The explicit synthetic-mapping export passed 13 Python tests; its pass-one
artifact SHA-256 is `e931c53e340018a5bb84d43063390044242f8426c2b8fb326a36fe0a0419a85d`.
The original `509B22..509C57` corner-triple producer matched the independent
model for the 252 retained positions using explicitly supplied face ordinals
and synthetic secondary/tertiary streams. Receipt
`target/creature-skin-triple-producer-0473-prep-20261109/result.json`, SHA-256
`58492e37383d40d48737d006b6a3d1f9fd5d38cfe814e17c6f6e7174a54b49d9`.
Actual subobject face lists, the secondary streams, owner and palette remain
unverified.
The reusable Rust triple producer matched the retained output, and its
composition with the prior attribute projection matched 44 controlled entries.
Receipt `target/creature-skin-corner-triples-rust-verify-20261109/result.json`,
SHA-256 `e16381798253dce2d7afa85f4b93c67dba7aa704f0572a9fae55da1df6ef07e3`.
The complete original `50B210` edge-continuity function matched 252 retained
paired inputs and two synthetic altered-endpoint controls, with the initial
position-index copy admitted unchanged. Receipt
`target/creature-skin-edge-continuity-0473-prep-20261109/result.json`, SHA-256
`7d8d527d8dffbad288847a6d72363425f192834fdf70a05127c1b9121e3b3da0`.
This does not establish the subsequent face grouping, UV seams or live object.
The checked Rust predicate matched the same 254 retained results; receipt
`target/creature-skin-edge-continuity-rust-verify-20261109/result.json`,
SHA-256 `56a265b4d989491510eefc189d993dfea506abf9e1ba752867a6c700f1e3e81e`.
The original face-orientation routine and helper closure matched the binary32
model for 84 conditional faces and three synthetic tie, negative and signed-zero
controls. Receipt `target/creature-skin-face-orientation-0473-prep-20261109/result.json`,
SHA-256 `634b2c1d89aee5bd2ee003687e5fabb7a7801a9e225c5d001587406fad1962d0`.
Actual grouping, UVs and live creature identity remain open.
The checked Rust classifier matched all 87 retained words; receipt
`target/creature-skin-face-orientation-rust-verify-20261109/result.json`,
SHA-256 `8d5d72f07e5dbad1d13fb691a3478c650d693e8f33035555a3137ec3033c2acb`.
Original `50A840` also matched 252 conditional face/lane calls and 339,696
snapshot bytes with synthetic preseeded UV cache entries and an append stub;
receipt `target/creature-skin-group-cachehit-0473-prep-20261109/result.json`,
SHA-256 `90a347cad43f23294ea670f5ed6e1d707eef4aa941bc047e7137fa7d16701f33`.
The cache-miss projection, actual group and UV identity remain unverified.
The checked Rust cache-hit projection matched the retained snapshot bytes;
receipt `target/creature-skin-group-cachehit-rust-verify-20261109/result.json`,
SHA-256 `2e48b71b2ed0dc22cf9593ea3c05a0033aa32c0c82603d158acbb7f02040458c`.
Its checked miss rejects before mutation, while native appends before its miss
helper; this projection does not model native miss or allocation behavior.
The original `50A7B0` coordinate selector matched 376 combinations of 44
conditional averaged positions and three synthetic bit-pattern controls with
explicit group values. Receipt
`target/creature-skin-uv-select-0473-prep-20261109/result.json`, SHA-256
`a9b9f963b129d5ceb9cd8e43ec22d2b3acf3fea94db87668e488ceb3b8a030c6`.
Actual group choice, UV cache, live ownership and texture seams remain open.
The Rust bit-preserving selector matched all 376 retained native pairs; receipt
`target/creature-skin-uv-select-rust-verify-20261109/result.json`, SHA-256
`746840fdba02bfc0438ae7609d90991aa3ad1d2dc021a362b3f255576ec4a1b8`.
Original `50A570` and its coordinate/bounds helpers matched 266 conditional
UV-miss and existing-ordinal cases, using supplied positions and bounded
preallocated append stubs. Receipt
`target/creature-skin-uv-miss-0473-prep-20261109/result.json`, SHA-256
`3fed994c8f24ee5d490d9ffa54f743055e64235b0dd3aca88757e55388c849da`.
Original vector growth, preceding face append, real group selection and live
identity remain unverified.
The checked Rust UV-miss model matched all retained output bytes; receipt
`target/creature-skin-uv-miss-rust-verify-20261109/result.json`, SHA-256
`3502289ebc579075ebd31f2cebd5715887d9986a79741adeb23aeedb351c507f`.
Its invalid-input checks reject before mutation and do not model native
allocation failure or the preceding face append.
The original seed-face neighbor-admission loop matched 84 conditional faces and
two synthetic previsited/seam controls, using retained source-matched paired
links and an orientation stub. Receipt
`target/creature-skin-group-seed-neighbors-0473-prep-20261109/result.json`,
SHA-256 `5ec267d775e80b6a38d143237b736fe2c33bf1a5435e336a9ac1a8ddfe3abb4d`.
It covers only initial neighbor queueing, not the later flood, actual group
state, UV writes, allocation or live creature identity.
The checked Rust seed-neighbor projection matched all 31,992 retained native
bytes; receipt
`target/creature-skin-group-seed-neighbors-rust-verify-20261109/result.json`,
SHA-256 `ee5e3d0670d6eb4b835d1660c8801dcbb2328163aad6e52b8b66722bba0c4db2`.
The original subsequent queue consumer matched 86 controlled seed snapshots
and 117,648 output bytes, with `50A840` replaced by a face/state stub; receipt
`target/creature-skin-group-flood-0473-prep-20261109/result.json`, SHA-256
`ee6e4f77b6eabe1cf69ab80d578f125f62a82f76f61c73ab6ae66b8ef0323658`.
This does not cover the stubbed UV/cache operations, allocation or live state.
The first one-group join of original traversal and UV operations failed with
access violation `C0000005` before producing output; it is not a match.
Failure receipt `target/creature-skin-single-group-0473-prep-20261109/result.json`,
SHA-256 `db864d85302d86fbe5ded7103cbca9585ef24ef5df0a11110536f330305b426d`.
Host and fixture assumptions are under investigation; earlier isolated
comparisons do not establish this joined result.
The review found a likely missing dependency: the joined host mapped the
sign-mask page but did not map or initialize the separate signed-byte lookup
tables at `013F1890` and `013F1898`, which `50A840` reads. The failed receipt
does not localize the fault address, so causation is not yet proven.
After adding the separately mapped source tables and auditing the copied
instructions' high-address operands, repaired v2 matched the 6,788-byte
single-group output; receipt
`target/creature-skin-single-group-0473-prep-v2-20261109/result.json`, SHA-256
`7030914a503bf9ad6e31718558d99a9f373322d5bb8d8ef3862239aa56b13333`.
It uses one synthetic initial group, supplied orientation and bounded
preallocated vectors; the outer grouping policy, allocator and live creature
identity remain unverified.
The checked Rust one-group composition matched all 6,788 retained native
bytes, covering 16 faces and 15 UVs for that fixture; receipt
`target/creature-skin-single-group-rust-verify-20261109/result.json`, SHA-256
`20f7627bee9b5ed88f1d4e6df13a6ccddec6559765bafd18270514c3508b7a09`.
The owned projection rejects invalid state rather than modeling native partial
failure or the outer group-selection loop.
The isolated original outer seed selector matched eight model-generated group
snapshots and four synthetic controls; receipt
`target/creature-skin-group-selector-0473-prep-20261109/result.json`, SHA-256
`87a8627101bdeb3bc749331c4b12b33b41c2c1d6da2338eeb253f43b0ea4218a`.
This does not validate outer-loop allocation, cleanup or live grouping state.
The checked Rust selector matched all 12 retained native outputs; receipt
`target/creature-skin-group-selector-rust-verify-20261109/result.json`, SHA-256
`94753131fb25897304ef9e1c057c32f0bd1f99ea44b570bc3b716272e615c21b`.
The checked multi-group Rust composition matched a separately retained Python
fixture reference across eight groups and all 84 conditional faces; receipt
`target/creature-skin-multigroup-rust-verify-20261109/result.json`, SHA-256
`1e593a044be9842d734576b78d9373cb832477baf18741ea460e108cc33a7728`.
This comparison is model-to-model: the initial group count, shared state,
allocation and complete original outer loop remain admissions.
The original outer loop and UV/edge helpers subsequently matched that same
conditional 84-face fixture with bounded synthetic lifecycle and allocator
stubs. Receipt `target/creature-skin-outer-group-0473-prep-20261109/result.json`,
SHA-256 `4492803315018f5819322ba23f523ee1d6f11c87e0637748340a49e5b99a8775`.
The supplied orientations, initial four groups and successful synthetic
allocation/clone/reset behavior remain admissions; native lifecycle, live
group ownership and creature identity are not established.
The checked Rust multi-group output also matched the retained original-code
output directly across all 3,696 bytes; receipt
`target/creature-skin-multigroup-native-rust-verify-20261109/result.json`,
SHA-256 `c9dcc5cfb12dccf5627335fcfedadb77faa7764c977ee828aedcd58868a26eb8`.
The same supplied orientation, synthetic starting state and bounded lifecycle
stubs apply; this does not establish live group selection or ownership.
The original atlas-bounds expansion and axis rotation matched eight conditional
generated groups plus three synthetic controls. Receipt
`target/creature-skin-atlas-bounds-0473-prep-20261109/result.json`, SHA-256
`0f3dc8289c2c561a88668aee3c76cf0bb6c5283a79889741eca4b354dc479357`.
The four prior groups and their accumulated area were not included; complete
packing, final UV transforms, texture binding and live identity remain open.
The checked Rust bounds/area/rotation model matched all 11 retained native
cases; receipt `target/creature-skin-atlas-bounds-rust-verify-20261109/result.json`,
SHA-256 `21c571ac34c0fc199d3898ccb2d76d1d5a0e4064867c3edb83f6146b6b208593`.
The original shelf-append routine matched eight conditional group inputs and
three synthetic controls using a bounded append stub; receipt
`target/creature-skin-shelf-append-0473-prep-20261109/result.json`, SHA-256
`c7dbee507f605cc1aff968ff180de7b38cc0f33af338fe0f5210841453d6e3f7`.
Shelf ordering, selection, the four earlier groups and final packing remain open.
The checked Rust shelf projection matched all 11 retained native rows; receipt
`target/creature-skin-shelf-append-rust-verify-20261109/result.json`, SHA-256
`c3e5c0c434f002a8c5d2c2e0b8312322076cf33c1358b9d5e7fc6a8f13d80aa6`.
Its checked failure behavior differs from the original append-first path.
The original atlas height-sort closure matched four controlled eight- and
twelve-group inputs, including equal-height controls; receipt
`target/creature-skin-atlas-sort-0473-prep-20261109/result.json`, SHA-256
`3cf0063b51449a543c31da90543804b1e9b6d168b4f89b9007ee781c2949156f`.
The additional four groups were synthetic; larger sorting paths, actual
preexisting groups and shelf selection remain unverified.
The checked Rust height sort matched all four retained native cases; receipt
`target/creature-skin-atlas-sort-rust-verify-20261109/result.json`, SHA-256
`3d8bbded440ff58a563111d5b811c6ce097ce3e99a904f13385f076f43fd3ec0`.
Only the controlled eight/twelve-entry native cases were compared; other
sorting paths remain outside that evidence.
The original shelf-selection slice matched a conditional eight-group case
and a synthetic exact-threshold control with bounded preallocated shelf stubs.
Receipt `target/creature-skin-shelf-select-0473-prep-20261109/result.json`,
SHA-256 `bf8c775b3d2430a8c2589c25d915ee496f62dce0278b4ef4b93cdacc42e8b56c`.
Sorted input, square-root value and shelf state were supplied; four earlier
groups, native allocation, final UVs and live identity remain open.
The checked Rust shelf selector matched both retained native corpora; receipt
`target/creature-skin-shelf-select-rust-verify-20261109/result.json`, SHA-256
`f9124645bd8f3a14feef4d181ccc7c9dd671ced96654034c7a0c8cb781f71347`.
It requires supplied order and square-root value, fit-in-empty-row admission
and bounded shelves.
The original per-UV rotate, translate and scale slice matched 85 conditional
UVs and three controls with supplied offsets and scale. Receipt
`target/creature-skin-uv-transform-0473-prep-20261109/result.json`, SHA-256
`03e16621ea8faec1232b39d684d4fc5bb774ecf832b8be50d6e9dff0dbb63842`.
Actual atlas offsets/scale, four preceding groups and texture binding remain
unverified.
The checked Rust UV transform matched all 88 retained native cases in the
same `target/pending-models-verify-20261109-v1/receipt.json`. Its offsets and
scale remain explicitly supplied rather than observed runtime atlas values.
The 0473 inspection exporter can optionally include the model-only UV and
group output, with fixture and reference hashes checked. The default export
remains unchanged; 15 Python tests passed with fixtures, and seven
fixture-dependent tests skip in an empty-target simulation. The conditional
artifact `target/creature-skin-multigroup-export-draft-20261109/conditional-uv-groups.json`
has SHA-256 `5f6b4126a3ccdf28f8dca6adcd067dee8b0a5f7cc2144637b0dbc9221753118c`.
Its eight groups and UV indices do not establish live assignment or palette.
The option was subsequently updated to verify the retained original-outer-loop
and direct Rust/native receipts as well. Its fresh conditional artifact is
`target/creature-skin-multigroup-export-draft-20261109/conditional-uv-groups-native-evidence.json`,
SHA-256 `83b558c32c1411f7835fbb334e5c7dd912fd3e6d7e90947917f42253817405b7`.
All 15 focused Python tests passed again; synthetic starting state, source
orientation admissions and allocation/lifecycle stubs remain part of scope.

The inspection exporter has an explicit `--include-synthetic-initial-records`
option, preserving raw index bytes and weight words without claiming a bone
palette. Its conditional output is
`target/creature-skin-0473-partial-synthetic-records-20261109.json`, SHA-256
`cbbf8201b98e9ffbe5cc61468616e022ded775636b33924b03b65676a9185a38`.
The default export remains unchanged; the optional records are not final skin.

The original `757BC0` section-type check matched three controlled cases using
the candidate RW4's 20 serialized section tags, a substituted positive-tag
control and an empty collection. Receipt
`target/source-cell/kind0-rw4-200af-host-result-v2/receipt.json`, SHA-256
`cfc18443f82ce0f9702f4bfaeb0b3038cb14930f5dfbe7192ac19ff5d76dbcaf`.
With the supplied enumeration, the candidate takes the direct-attach path.
The live provider, native collection identity and subsequent binding remain
unverified; this controlled result does not establish runtime selection.
The `2001A` section hook and its lookup matched the candidate's serialized
`[13, 1, 14]` reference words and a count-zero control using a supplied stable
lookup table. Receipt `target/source-cell/kind0-2001a-hook-result-v2/receipt.json`,
SHA-256 `9f9b522a197f1114b0b2811609228c2d4b82ca7941ad35ce99704e46fea88172`.
This does not establish the actual loader table, provider or live object identity.

On 2026-11-09, the pinned REA 6.5.0 doctor reported the Codex registration
aligned and Ghidra 12.1.4 plus its Java 21 and headless components healthy.
The current agent session exposes no REA MCP tools; the doctor cannot verify a
live connection. Reconnection is needed to test it. Existing direct headless
and instruction-host workflows remain usable; prior full-Spore provider
timeouts are not resolved by this registration check. Raw doctor output is
retained at `target/rea-doctor-20261109-v1.json`.

Further bounded original-instruction checks matched the Cell service's changed
prefix in three cases, including a callback changing the observed current token.
The receipt is `target/cell-service-changed-prefix-result-v2/receipt.json`,
SHA-256 `6f5a372d1c20df0635e4b308833adb54b2dc793de4814956a9d8429b2301c9b7`.
Callbacks and map outcomes were scripted, and the later transition was not run.
The checked Rust prefix matched the three retained native rows; receipt
`target/cell-service-changed-prefix-rust-result-v1/receipt.json`, SHA-256
`1024ca1a66c49d2d53788a4353b01a0593452c44d925e548d6b2b3f409d0dd09`.
Its field value is before the pair callback; the comparator admits stubs that
preserve it and restricts the entry values beyond the original slice.
The Cell record timing prefix matched six supplied timer/helper cases; receipt
`target/source-cell/e7e6f0-timing-result-v1/receipt.json`, SHA-256
`0eb9bb1b056f475b0d2869b40a622dd3d8224c1a0e1cb23f8a6139d517ab31e9`.
Both downstream branches were trapped. The real helper can resolve actors and
invoke a callback, so the scripted result is not evidence of live timing.
The checked Rust projection matched the six retained native rows with explicit
floating-point controls; receipt
`target/source-cell/e7e6f0-timing-rust-compare-v1/receipt.json`, SHA-256
`ecf77e5bdf2425e056e3a6d479968e8794497f8d427551438595b93750e2f2c8`.
It exposes the value before the helper and does not execute either later path.
The original `E59A90` helper was separately checked against eight synthetic
actor, callback and alternate-pool scripts; receipt
`target/source-cell/e59a90-helper-result-v1/receipt.json`, SHA-256
`22347850d77f8924887e13faa1ea1be37a6e05c85ce8fc0fe3cb7429902d088b`.
The nonzero auxiliary route was trapped; actual callback, pool and `E50480`
behavior remain unverified.
Its checked Rust projection matched all eight retained rows; receipt
`target/source-cell/e59a90-helper-rust-compare-v1/receipt.json`, SHA-256
`26134c582814d8c41c6ce32d1a9598a023b973ad36fae1fd8c3c5d97ac4d48a3`.
The alternate marker represents a trap, and the compared lookup stub preserves
the tested flag; neither is a claim about the live helper's external effects.
The original timing prefix and helper were subsequently run contiguously for
seven synthetic scripts, all matching; receipt
`target/source-cell/e7e6f0-e59a90-join-result-v1/receipt.json`, SHA-256
`3b37ca57fbb314b61ec09273d4b4fddd1a597822d7b23ddbbd92bb1abc28336c`.
The scheduler, actual callbacks and nonzero auxiliary branch remain outside
the joined run.
The composed Rust timing/helper projections matched all seven retained joined
rows; receipt `target/source-cell/e7e6f0-e59a90-join-rust-compare-v1/receipt.json`,
SHA-256 `1f67bb91e907018d71d0bf9450850d07e4662369c42f2a53248056729e1d8a55`.
The comparison admits inert fixture callbacks that preserve the timer field.
The separate original `E7D8B0` progress/identity prefix matched six synthetic
cases, including nonzero enable-byte values and 32-bit wrap; receipt
`target/source-cell/e7d8b0-progress-result-v1/receipt.json`, SHA-256
`c1b2d6cfb9b8ec87b49457988c1ad2013dcd67d94cc39205a3b6c936899c0a5e`.
Its excluded branches and the call following the fresh read were trapped; live
actor identity and callback reachability are not established.
The checked Rust projection matched the six retained rows; receipt
`target/source-cell/e7d8b0-progress-rust-compare-v1/receipt.json`, SHA-256
`987f57e1576e94caa46efb1948ae15613fe6477b550548183da727deeb4f58ef`.
The original `E61390` fresh-save constructor matched its complete 0xEC-byte
output for two controlled successful-allocation cases; receipt
`target/source-cell/e61390-fresh-save-result-v1/receipt.json`, SHA-256
`56929e74d31c6cae73dedea3888e0123af80b41b11b2452d9ec753eba38732e5`.
Allocation and helper behavior were stubbed; loaded/live state and ownership
are not established.
The checked Rust constructor matched both retained full-object cases; receipt
`target/source-cell/e61390-fresh-save-rust-compare-v1/receipt.json`, SHA-256
`0beda91b061877725e86795e67b73c78c9d4f9b45107cd3fe95f0e608d4de9e0`.
The separate original `E80D75` retain/store/release slice matched seven
synthetic pending/fresh pointer cases; receipt
`target/source-cell/e80d75-retain-result-v2/receipt.json`, SHA-256
`b65b8cf247888791d511153505a05b9620054d201f023b6531c16fcf9b0cf059`.
Reference callbacks and fresh resolution were scripted, so this does not prove
live ownership or lifetime.
The checked Rust projection matched the seven retained rows; receipt
`target/source-cell/e80d75-retain-rust-compare-v1/receipt.json`, SHA-256
`c6c9aa4830b60facc96eac6b832798c3ea3ba4fea6604e62e7bc5d7f802fad27`.
The fresh-branch old-slot snapshot is supplied after the helper, and callbacks
are admitted inert with respect to the slot.
The original `E61350` restoration boundary matched eight scripted cases;
receipt `target/source-cell/e61350-restore-result-v1/receipt.json`, SHA-256
`17e9dce6f6b5d337b3c0d015bafee7230de1c77e9baf39c23e091f92a0ee9602`.
The provider and release callbacks were scripted; actual restoration and
destruction are not established.
The checked Rust restoration projection matched all eight retained rows;
receipt `target/source-cell/e61350-restore-rust-compare-v1/receipt.json`, SHA-256
`bd7e034c62e54454eacf719d0ce1e22fc1f77549121ae9a4828452b371dcac71`.
The original `E69E570` cache-hit store/callback slice matched four scripted
cases, including mutation of the slot by the callback; receipt
`target/source-cell/e69e570-cache-hit-result-v1/receipt.json`, SHA-256
`3b43e26aa10f70659c6d5783a05f420fae00089b222f807c1809bb70c62e7ef9`.
Map lookup, reference effects and lifetimes remain supplied fixture inputs.
The checked Rust cache-hit projection matched all four retained rows; receipt
`target/source-cell/e69e570-cache-hit-rust-compare-v1/receipt.json`, SHA-256
`c2a46d93c2af7bb1626ec942f7330116cdcddf188009d33d7289804df00ed221`.
The original `E69E499` inner-lookup/factory slice matched six scripted cases;
receipt `target/source-cell/e69e499-registry-result-v1/receipt.json`, SHA-256
`91fc5251f08fcb91f44c4cc060cb30c677ede1d907bb4a3e134bf481021410af`.
Actual maps, factory callbacks and cache insertion remain unverified.
The checked Rust registry projection matched all six retained rows; receipt
`target/source-cell/e69e499-registry-rust-compare-v1/receipt.json`, SHA-256
`4c0533f049fa1c33c7152650cc5fb3a806291b7e3ed72d77de4eb03de924b71b`.
The contiguous original two-lookup route matched ten scripted cases, including
callback mutation of the slot; receipt
`target/source-cell/e69e46f-two-lookups-result-v3/receipt.json`, SHA-256
`7ef94b14d1fbb0b03685b706398a227756a9540ba3c9e498886afd046dce2862`.
Map lookups, factory/callback behavior and lifetimes remain synthetic inputs.
The composed Rust two-lookup projection matched all ten retained rows; receipt
`target/source-cell/e69e46f-two-lookups-rust-compare-v1/receipt.json`, SHA-256
`cd84a148d11750eaf4d3903c0976ec240e8d027b992cee0d5e90fbbcc8412955`.
The original `E91FBE0` factory wrapper matched six scripted cases, and the
checked Rust projection matched the retained rows. Their receipts are
`target/source-cell/e91fbe0-factory-result-v1/receipt.json`, SHA-256
`cb85dd1a78cf179a927e5a58e16f929557f8dd58b7b2a23b8a493e00b16eaa22`, and
`target/source-cell/e91fbe0-factory-rust-compare-v1/receipt.json`, SHA-256
`83dbc7481ee8b709526c36cae9ce0d443c62eabb5ea00a1d451754dc0b06bd64`.
Acquisition, validation, release and object lifetime remain scripted.
The contiguous registry-miss-to-factory path matched eight scripted cases;
receipt `target/source-cell/e69e499-factory-join-result-v1/receipt.json`,
SHA-256 `b400d7d1fb1118f41a344c0f48dd65c36820eec01fdbcee368ac41f8c9b06e7a`.
Intermediate candidate and validator-return identities remain distinct, with
lookup and callbacks synthetic and no live registration claim.
The composed Rust projection matched all eight retained rows; receipt
`target/source-cell/e69e499-factory-join-rust-compare-v1/receipt.json`, SHA-256
`d8ac1a81154705295e0d38c76875c4fa9e51ec6870cbda29c0d30ce1a2621b3a`.
The original `E91FBA0` descriptor-provider wrapper matched eight scripted
cases; receipt `target/source-cell/e91fba0-provider-result-v1/receipt.json`,
SHA-256 `7f393b821c962fc839a71146a6b30c7f9ab6ac9c176ee0154e50ba3ceb5fa63f`.
Descriptor lookup, callbacks and lifetimes remain supplied fixture behavior.
The concrete CellSaveGame provider's original `E61A00` constructor matched
four guarded success/null allocation cases; receipt
`target/source-cell/e61a00-cellsave-result-v1/receipt.json`, SHA-256
`7a67daa63374dd4ce6b0674b0b2cad5119640725a7d1587bbd086245dce9eb49`.
The receipt's static review reminder predates the independent review performed
before execution. Actual allocation, later initialization and live ownership
remain unverified; this constructor is not equivalent to `E61390`.
The original provider enumeration and class type callbacks matched eight
synthetic argument cases and agreed on key `ABCDEF00`; receipt
`target/source-cell/e4f490-cellsave-key-result-v1/receipt.json`, SHA-256
`7bce7da312c10d7b355a92f722505d05accc400e5e75652cb6a7d00ab8cd97b3`.
Its review reminder predates the independent review performed before execution;
live virtual dispatch, registry membership and ownership remain unverified.
The Rust key projection matched all eight retained rows; receipt
`target/source-cell/e4f490-cellsave-key-rust-compare-v1/receipt.json`, SHA-256
`860bd8cec433747f71250b7286d4002035db4552414c6022fdc5fd4d86eab778`.
Its Rust base-object projection matched all four retained snapshots; receipt
`target/source-cell/e61a00-cellsave-rust-compare-v1/receipt.json`, SHA-256
`a19aeff1e68a6dfe16b4c7dcf50fb9a5d5ce924c8f5ef4130241a96608d32256`.
The Rust descriptor projection matched the eight retained native cases; receipt
`target/source-cell/e91fba0-provider-rust-compare-v1/receipt.json`, SHA-256
`8cdb08fc85ea24b0f0cd55c27c21c56f1a1240ce47fb8a653ce1a5f485974846`.
The original `E92055C` registry priority slice matched twelve synthetic cases,
including signed priorities and ties; receipt
`target/source-cell/e92055c-priority-result-v1/receipt.json`, SHA-256
`699de6c7667920aeff71ee4ee48f15f715a68c9fd6fe822d9a6f3958ddd7a610`.
Node links and lifetimes were supplied; locking and live registry state were
not exercised.
The Rust priority projection matched the 12 retained rows; receipt
`target/source-cell/e92055c-priority-rust-compare-v1/receipt.json`, SHA-256
`c11db5988c5815f7f35d314e82d748d7a0f8c17b6a457ab0b995fc5620ee04b9`.
The original bucket lookup and priority scan then matched 12 synthetic joined
cases; receipt `target/source-cell/e920510-map-priority-result-v2/receipt.json`,
SHA-256 `60727bfbce7ccd5127016563be580578037217cb04a9a08ffc5de644445aaf76`.
Its static message still says wrapper review is required, but the revised linked
wrapper was independently reviewed before execution. An initial invocation
omitted the required manifest argument and stopped before host launch. Locks,
node lifetime and actual registry state remain synthetic admissions.
The checked Rust map/priority composition matched all 12 retained rows; receipt
`target/source-cell/e920510-map-priority-rust-compare-v1/receipt.json`, SHA-256
`852e063af57540626a5e037b6fac6ce4b51f661eac2938296a8b1e1310e32fdb`.
The original no-growth insertion and subsequent lookup matched 12 synthetic
cases; receipt `target/source-cell/e91ffe0-insert-lookup-result-v1/receipt.json`,
SHA-256 `0c31107cfe727acec827b6ce669fc10fd3f3df4c2987702a00234456e2c5cf7e`.
Its static field retains a review reminder; the no-growth allocator and linked
wrapper were independently reviewed before execution. Real locks, allocation,
lifetimes and registration remain unverified.
The Rust insertion/lookup projection matched the 12 retained output rows;
receipt `target/source-cell/e91ffe0-insert-lookup-rust-compare-v1/receipt.json`,
SHA-256 `4ebba6349b6905ba96ffa89b0eb88f950f78bf25e5b7d3174fbf656198c8def7`.
Link order is asserted by the original host, rather than independently emitted
in those raw comparison rows.
The original fresh-constructor-to-adoption route then matched two contiguous
mode-3 cases; receipt
`target/source-cell/e80d75-fresh-contiguous-result-v2/receipt.json`, SHA-256
`e6da769097107d9851572eb2c2c2b43a34a19aa4e398bd1a99691edf2a3ec1be`.
Allocator and helper behavior, including a post-construction synthetic vtable,
were scripted; vector and alternate routes and live lifetime remain excluded.
The Rust composition matched both retained complete-object cases; receipt
`target/source-cell/e80d75-fresh-contiguous-rust-compare-v1/receipt.json`,
SHA-256 `05b845da774545eafad6ff0015ad8d7396828223d4f33b8b06f9778d77c1c540`.

The original atlas pairing and scale slices matched three conditional or
synthetic shelf cases; receipt
`target/creature-skin-atlas-pair-scale-0473-prep-20261109/result.json`, SHA-256
`b4eedac21de34f071026bd8293f4dd346d141ece708592ace65dab63dc3770ed`.
The supplied shelf order and dimensions, four earlier groups, native allocator
and live atlas remain outside the evidence.
The checked Rust pairing and scale model matched all 2,340 retained native
bytes; receipt `target/creature-skin-atlas-pair-scale-rust-verify-20261109/result.json`,
SHA-256 `9f43a1eba74a20966d1c6db4017302c2944c023293844ca652181bf70db6a926`.
The comparison is limited to the three supplied shelf fixtures.
The original group-offset and row-advance slices subsequently matched eight
groups in the paired fixture's row order; receipt
`target/creature-skin-paired-offset-0473-prep-20261109/result.json`, SHA-256
`b09f431223a97a1ad13dd519f28796c5e5793d37f3b685587ad9046e077e2bb4`.
Starting coordinates and row-boundary markers were supplied, upstream ordering
remains conditional, and live atlas/texture identity is not established.
The Rust composition matched all eight retained native rows; receipt
`target/creature-skin-paired-offset-rust-verify-20261109/result.json`, SHA-256
`b5afaab150d739d77097f390b3ec831e736061ac6d8b5ca60e57613c144e0cb7`.
The original UV slice then matched 85 conditional UVs plus three controls using
those paired offsets and the matched conditional scale; receipt
`target/creature-skin-uv-paired-join-0473-prep-20261109/result.json`, SHA-256
`88ba87a0a4599fb5e8f3f747e2ac192dc19566d0709d6e57e79fa62db2a6a873`.
Supplied upstream sort/square root and zero-origin scheduling, excluded earlier
groups and actual live atlas selection remain limitations.
The composed Rust models matched the same 704-byte UV stream after matching
their paired-scale and paired-offset inputs; receipt
`target/creature-skin-uv-paired-composed-rust-verify-20261109/result.json`,
SHA-256 `8aa327f4d1d2e7b201ea50e64f34cb954c4130e6c41a6df0252affdbf9f41dda`.
The comparison pins its executables but does not independently prove their
source-to-build provenance.
The exporter also offers `--include-paired-transformed-uvs` for that newer
conditional result, retaining the earlier option and default unchanged.
Nineteen focused Python tests passed with fixtures; in an empty-target simulation
10 passed and nine fixture-dependent cases skipped. Its inspection artifact
`target/creature-skin-uv-paired-composed-rust-verify-20261109/conditional-paired-uvs.json`
has SHA-256 `c301db60f8bf1694231e3f52e6c37bbeab9b16c8e33ce1dac5546582e898a196`.

The experimental Cell frontend has an opt-in controlled startup-art path for
seed 1. It checks the staged source triple and the exact content-package keys
and decoded hashes, requires a stable-package admission, and renders the
candidate on a prototype player. An offscreen RTX 3060 run completed 60
prototype ticks; the capture remains small and dark and is not a retail match.
The source identity, pose, camera and lighting remain unverified. Reproduction,
the retained captures and the projected-bounds investigation are documented in
[the candidate note](../re/cell-controlled-player-visual-candidate-v1.md).
The optional startup-only CPU normal approximation is admitted only for the
hash-checked candidate. In paired RTX captures with the same seed and 60-tick
simulation, 52 pixels near the player changed by at most one channel level;
receipt `target/startup-source-normal-capture-result-v1/receipt.json`, SHA-256
`5a806a099080fd0884ef6c33c463e38fe02ae87dd6fe8a15525fa2016c39e306`.
No render/pipeline errors were seen in the retained logs, but the Vulkan
validation layer was unavailable and overlay warnings were present. This small
change does not establish retail lighting or visible likeness.
The separate optional StartupCell unorm texture interpretation changed 131
pixels around the player by at most 72 channel levels in a same-seed, 60-tick
RTX pair; receipt `target/startup-unorm-capture-result-v1/receipt.json`, SHA-256
`9c8d1ce74263ecc82685a1ddfed04da4ef13e5a83de0d91f492a9108e4a9b84f`.
No pipeline errors appeared in the retained logs; the validation layer was
unavailable and overlay warnings were present. This uses Bevy PBR lighting and
does not establish source-material or retail visual parity.
The offscreen `--startup-material-preview` option binds both admitted source
textures and runs the tested shader subset with synthetic basis and lighting.
At the same seed and tick, 131 pixels in a fixed player region changed, by at
most 218 channel levels; receipt
`target/startup-material-capture-result-v1/receipt.json`, SHA-256
`8ce62c23a1bb123b12d8b62a12255f92c47211757938632a61ff171d06af74b0`.
The whole-frame difference also includes a diagnostic label. No pipeline errors
were reported, but validation was unavailable and overlay warnings appeared.
The result is darker under controlled lighting and does not establish retail
pose, material, lighting or visual parity.
An independent Python index audit reconstructed the controlled renderer's
82-package inspection order, matched all nine observed first-hit resource
providers and payload hashes, and found no standalone RASTER for sampler 2 in
that inventory. Receipt `target/startup-provider-inventory-result-v1/receipt.json`,
SHA-256 `8a9b7560fbbc88e40fc8274c9da7812b8af982190423c0847b3ebf47bf13b56d`.
Full file hashes and inventory matched before and after, but cannot exclude a
transient change. The selected-pack fragment is outside those nine observations;
normal non-optimized Python execution and stable files are admitted. This does
not establish retail provider order.
The exact candidate's packed vertex/index hashes matched the renderer trace,
and the Rust geometry decoder matched all 2,555 untransformed positions and UVs
and 12,732 indices; its reported sanitization count was zero. Receipt
`target/cell-startup-geometry-result-v1/receipt.json`, SHA-256
`b23b408f6749c077557ab40befc06fa63b734a01b90ec1b1fe31a688afb35d95`.
Normals were checked only for finiteness, and pose, basis and lighting remain
outside this comparison. Source and executable pins alone do not prove build
provenance.
The controlled renderer's selected cached vertex program was extracted from
the pinned shader pack and matched the retained pre-draw bytecode hash. Receipt
`target/cell-startup-cached-vs-result-v1/receipt.json`, SHA-256
`0a002925eccb52b55a54fde77d3323651ebd692e6921057835f9129405cfde65`.
This identifies the program for further normal-unpack analysis, but does not
yet establish its intermediate values or equivalence to the Bevy path.
An independently reviewed x64 D3DX disassembly-only host completed; receipt
`target/cell-startup-vs-disassemble-result-v2/receipt.json`, SHA-256
`eed7e5ff583cd629a5dc4186131ffafc264a8cc57c83b36dff73cabbec4bd2ea`.
Its normal path uses a different byte conversion and transform/normalization
order from the current Bevy decoder. This is a static formula difference;
its effect on the dark preview is not established.
The unmodified captured vertex shader was subsequently run on the RTX with
synthetic point positions, an identity transform and a diagnostic pixel shader.
All 2,555 candidate points and six synthetic controls had the expected coverage;
receipt `target/cell-startup-normal-gpu-result-v2/receipt.json`, SHA-256
`c2f21c711e2d0484c788d4d79bdda60dc7e1c1ca2bab4276ffbd9ba8a25b58e9`.
The external host precheck used an incorrect path and did not succeed; the
reviewed runner checked its actual host hash before and after execution, and
the host hash was checked separately afterward. This is controlled normal
output, not a live pose, lighting or visible-effect comparison.
Against the actual Rust-decoded normals, the candidate-only maximum component
difference was 0.0054192 in this identity-transform fixture; receipt
`target/cell-startup-normal-rust-result-v1/receipt.json`, SHA-256
`79b07834ee281cfd120930df3798684cdd94c1c4d43bdf0d289c2736b93a35fa`.
The six synthetic controls are excluded from that statistic; binary/source pins
alone do not prove build provenance or a visible effect.
An independently reviewed RTX sampler fixture compared 16 fixed texel-center,
nearest/LOD0 samples from the candidate's pinned BC3 base mip. D3D9 with sRGB
sampling disabled and WGPU BC3 unorm differed by at most approximately
`5.87e-8`; the unorm and sRGB RGB values differed by up to `0.2873` in this
small sample. Receipt `target/startup-sampler-fixture-result-v1/receipt.json`,
SHA-256 `c69843a099a5c47b507a521b5926e00faf7bb3f90853f893dd4c5cdc8b2ac876`.
This lighting-free test does not establish linear/mip filtering, the Bevy CPU
texture upload, full material or retail parity.
The Rust BC3 decoder used by the preview matched those 16 retained unorm samples
within a reported maximum component difference of approximately `2.69e-8`;
receipt `target/startup-sampler-cpu-result-v1/receipt.json`, SHA-256
`d75ddfe67d5fa70c1730b19d21dc78eb81bbda26aabc30f52471d0298be36a16`.
This comparison is descriptive and does not check Bevy's upload or filtering.
For sampler 1, an exact-key CPU inspection matched all nine decoded RGBA mip
levels byte-for-byte to the retained source-format BGRA trace after channel
conversion; receipt `target/startup-sampler1-inspection-result-v2/receipt.json`,
SHA-256 `123fd6dde69d0776f3a0fd5498a969c6d10c3157cb8fe94f16303ecdd83ab920`.
The successful run retained both streams; its runner would not retain partial
streams on timeout. Same-file stability was admitted and GPU upload, filtering
and live resource selection were not checked.
An independent RTX D3D9/WGPU sampler-1 fixture then compared 16 controlled
fractional-coordinate samples using the nine retained mips and captured
clamp/linear/mip settings. Its maximum unorm component difference was about
`5.59e-8`; receipt `target/startup-sampler1-linear-result-v1/receipt.json`,
SHA-256 `527af56a88472aa969e21c633f7e5a2f6d43c80f5a6d57aadac318b0605cee7e`.
This is a small controlled sampler check, not full source-material or retail
rendering parity.
The separate 16-point fractional-coordinate, linear/mip fixture used all ten
captured source BC3 levels and reported a maximum D3D9 versus WGPU unorm
component difference of approximately `5.50e-8`; receipt
`target/startup-sampler-linear-result-v1/receipt.json`, SHA-256
`be3e6ab2f28567bdf2d897dd906923c426af146a2d5e537572e8255d4ac21497`.
It is a controlled sample, not a full material or retail-rendering comparison.
The unchanged cached source VS was separately compared with a WGPU computation
over all 2,555 captured vertices and inspection constants, using synthetic point
positions and clip. Maximum component differences were approximately
`1.19e-7` for tangent, normal and diffuse outputs and `3.58e-7` for specular;
receipt `target/startup-vs-outputs-result-v1/receipt.json`, SHA-256
`65b251f5dc250923df8f54c172f2d98c2e8f6bafac6e69a5fbf7f4c2ce876021`.
This does not establish actual live pose, lighting or rasterization parity.
Eleven retained D3D vertex outputs and actual UVs were then supplied manually
to the unchanged source PS and a WGPU calculation using captured inspection
constants. The maximum measured component difference was approximately
`2.98e-8`; receipt `target/startup-vs-ps-join-result-v2/receipt.json`, SHA-256
`02550fb3db856876fc296b61cd49e24f8925bbc1ba76787370e8011c339bb3af`.
This is synthetic pointwise handoff, not a contiguous draw or live-scene result.
The unchanged source VS and PS were then run together for 11 point draws using
captured vertex attributes and constants, synthetic positions/clip and nearest
mip0 sampling. Against the WGPU calculation the maximum measured component
difference was approximately `5.96e-8`; receipt
`target/startup-vsps-contiguous-result-v2/receipt.json`, SHA-256
`5353cfa917864a2c696135464cc5abdb7a654e679e3b31cda36a4b027e711746`.
Triangle interpolation, actual camera, live pose and retail scene remain open.
The point host's inherited `compiler_path_checked=true` log text was inaccurate:
it bound pinned precompiled shaders and did not invoke D3DX. The retained output
does not establish a compiler-path check for that run.
One source-index triangle was then drawn with the unchanged source VS/PS and a
WGPU graphics calculation under synthetic geometry/identity clip, nearest mip0
and explicit D3D9 half-pixel compensation. Coverage agreed on 1,272 pixels;
the 1,078 shared eroded interior pixels had maximum component difference
approximately `2.98e-8`. Receipt
`target/startup-vsps-triangle-result-v1/receipt.json`, SHA-256
`c57c30a77ba904584abd9c998d418ec987ecc2e5e5cfe00243d51ab89f4e66fc`.
This does not establish general geometry, implicit LOD, live camera or scene
parity.
The exact cached source pixel shader and a constrained WGSL calculation were
then compared on 11 fixed points with synthetic interpolants and lighting. Both
the captured sampler-1 texture and a white control produced identical measured
D3D9/WGPU values in this fixture; the source D3D outputs changed by up to
approximately `0.21525` between those sampler-1 inputs. Receipt
`target/startup-source-ps-fixture-result-v1/receipt.json`, SHA-256
`eb0c9ab02442ce00ee0c5f7de6cedcc34411918cb21356f806b1b52eff0d162c`.
The fixture uses a synthetic vertex stage, fixed constants and nearest LOD0;
it does not establish general pixel-shader or retail material parity.
An alpha-only sampler-1 control under a different fixed gloss/SH setup also
produced identical measured D3D9/WGPU values at 11 points; changing sampler-1
alpha changed the D3D output by up to approximately `0.38823`. Receipt
`target/startup-source-ps-gloss-result-v1/receipt.json`, SHA-256
`d8151a000ab47297befe4710b772b1728647756c81ffaa4befbf82d091b9c3bd`.
These synthetic constants and points do not establish general shader parity.
After correcting the initially proposed reflection formula before execution,
the source direct-specular alpha fixture completed for the same 11 controlled
points; maximum D3D9/WGPU component differences were approximately `2.33e-10`
and `1.49e-8` in its two modes. Receipt
`target/startup-source-ps-specular-result-v2/receipt.json`, SHA-256
`a2af1c35e7db769080915de84e9f9dbf6dd70db57c86beccc172db56b8ebb6fa`.
This isolates a synthetic exponent-1 path and is not general shader parity.
With the captured inspection material constants and synthetic lighting, the
corrected exponent-60 fixture reported maximum D3D9/WGPU component differences
of approximately `7.75e-7`, `3.34e-6` and zero across its three modes. Receipt
`target/startup-source-ps-material-result-v2/receipt.json`, SHA-256
`eb7b02cef79b78e255852e035721771c8be3d3dd7b2e4e417fef8fef9cfd4476`.
Its alpha and no-specular controls had measurable effects; this remains an
11-point synthetic-input test, not full shader or live lighting parity.
The separately reviewed nonidentity probe also used the unchanged source VS
with a synthetic `(y,z,x)` rotation and uniform scale 2; all 2,555 candidate
points were compared against Rust normals. Receipt
`target/cell-startup-normal-gpu-result-v3/receipt.json`, SHA-256
`cedd09141140949802666f3acbefea6572548b429a204d6aa04e6c88889c43ab`.
Maximum candidate component difference was approximately 0.0054191; the
larger combined maximum belongs to a synthetic control. The probe does not
establish nonuniform-transform, live-pose or visual parity.

The four previously synthetic initial creature groups were traced to retained
rows 6–9 of the selected data under frozen-row/capability assumptions. The
reviewed original initialization slice matched those four cases and two
controls; receipt
`target/creature-skin-row-groups-0473-prep-v2-20261109/result.json`, SHA-256
`2dd6698805542769411dd1c35c5c623298e469ca6e305ee7713e4dcb7a475d61`.
The CRT path and direct export address were checked before source entry; mapped
image continuity was admitted. Allocation, construction, append and live
identity were not exercised.
The checked Rust model matched all six retained rows and controls; receipt
`target/creature-skin-row-groups-rust-verify-20261109/result.json`, SHA-256
`5a40f1edfefdbcb30d2189b0bbf379b742ffc7f54ed3a266fcb7623d1c9b57a2`.
Its square-root compatibility is limited to the compared values and admitted
floating-point context.
The original atlas bounds slice then matched all 12 conditional groups using
the four source-row-derived bounds followed by the eight generated bounds;
receipt `target/creature-skin-atlas-bounds-12group-0473-prep-20261109/result.json`,
SHA-256 `ec4f90b0c623ec5430798f2329527e581593f648d740c2e1061fb5f580ec3641`.
Area inputs were independently precomputed in group order; lifecycle, actual
packing and live identity remain unverified.
The composed Rust comparison matched all 288 retained native bytes and checked
each next area input against its preceding Rust output; receipt
`target/creature-skin-atlas-bounds-12group-rust-verify-20261109/result.json`,
SHA-256 `33adebf9bcee17415bb8a6c3c25d41630574fcd5ae36ea5bd626a7bce648dd42`.
The native host did not chain area outputs directly between calls.
The original height-sort closure matched two copies of those 12 conditional
groups plus tie and zero controls; receipt
`target/creature-skin-atlas-sort-12group-0473-prep-20261109/result.json`,
SHA-256 `3b01b1879bc5a89b0b59f02203fcfa880486c1d4e412635b1a098163ce126394`.
Shelf selection and live identity remain outside this sort check.
The Rust sort matched all four retained native cases; receipt
`target/creature-skin-atlas-sort-12group-rust-verify-20261109/result.json`,
SHA-256 `f6f62dd54adffccaf438e53d4e291651555b6f86b41d56d57cfccdb02fc546b3`.
The original CRT square-root slice matched the conditional accumulated area and
three controls; receipt
`target/creature-skin-atlas-sqrt-12group-0473-prep-20261109/result.json`,
SHA-256 `6e10edfd6bdfd44f26b9cb43c22f73787a82797200cff94fc63b634cf2b83d58`.
The checked CRT's mapped-image continuity and frozen upstream state are admitted.
The original shelf selection then matched the conditional 12-group order and
an exact-threshold control using the confirmed square root; receipt
`target/creature-skin-shelf-select-12group-0473-prep-20261109/result.json`,
SHA-256 `39c16fa1472ff73ebab01e3f53cb56b6d48a544949beddc239dd893693a7bcaa`.
The empty initial shelf and preallocated lifecycle stubs are fixture admissions.
The checked Rust selector matched both retained native cases and all 1,544
bytes; receipt `target/creature-skin-shelf-select-12group-rust-verify-20261109/result.json`,
SHA-256 `3687ebb78ddf85c2e8fd4dd72f580956205c2241ec354ea8d68316d62c3c10c9`.
The original pairing and scale slices matched the resulting three-shelf case
and two controls, including the odd final shelf; receipt
`target/creature-skin-atlas-pair-scale-12group-0473-prep-20261109/result.json`,
SHA-256 `9dbcd0d187cec05f2fd4a6fe62bbdad4d04bb3e6a76e891a9d35a3177685afbd`.
The Rust pairing model matched all three retained cases; receipt
`target/creature-skin-atlas-pair-scale-12group-rust-verify-20261109/result.json`,
SHA-256 `26f6a55fc051a2b88ebd2e536b892a39853689e42ec2510250d53f491d8810b0`.
The original offset and row-advance slices matched all 12 paired groups over
the three shelves; receipt
`target/creature-skin-paired-offset-12group-0473-prep-20261109/result.json`,
SHA-256 `dbe4c9ff6b85f619bf65e1e19f161f23a182b155a6284c68f403ec0a13d3672e`.
Zero starting coordinates and wrapper-supplied row boundaries remain explicit.
The Rust composition matched all 288 native bytes; receipt
`target/creature-skin-paired-offset-12group-rust-verify-20261109/result.json`,
SHA-256 `f178b0cd5199b281a3d369bde68c1b6cbaf0df63bbb2483ca5f6272419dd6ae0`.
The original UV slice then matched 85 conditional generated UVs and three
synthetic controls using the 12-group offsets and matched scale; receipt
`target/creature-skin-uv-paired-12group-0473-prep-20261109/result.json`,
SHA-256 `b7763cea0d2290c090eb26f7be1c88027cf3362566d0b94b02b24b3bcb52b940`.
Frozen ownership, successful lifecycle and wrapper row scheduling remain
admissions; live atlas and texture binding are unverified.
The composed Rust models matched the same 704-byte UV stream after checking
the full pairing and offset streams; receipt
`target/creature-skin-uv-paired-12group-rust-verify-20261109/result.json`,
SHA-256 `4c138be42db9db585be1a790630d0d5b602017fb785442c2b4b6a86355cc9e3a`.
The opt-in `--include-twelve-group-transformed-uvs` exporter section verifies
the source-row, sort, pairing, offset and UV evidence and retains these fixture
limitations. All 21 focused Python tests passed with fixtures; an empty-target
simulation passed 11 and skipped 10 fixture-dependent cases. The inspection
artifact `target/creature-skin-uv-paired-12group-rust-verify-20261109/conditional-twelve-uvs.json`
has SHA-256 `e7e3bf1108d7590a7fa864006fd9bb58bda6aa0b332866e74a821b70c3848129`.
The original tangent-preparation determinant and branch slice matched the 85
conditional UV records and three threshold/negative/zero controls; receipt
`target/creature-skin-uv-determinant-12group-0473-prep-20261109/result.json`,
SHA-256 `5f5a20577f9434b92a50ef6f6c1d664edb2019ef84b19d10584af903c4fedc48`.
The downstream tangent, material, allocation and live identity paths remain
outside this check.
The Rust determinant model matched all 88 retained native pairs; receipt
`target/creature-skin-uv-determinant-12group-rust-verify-20261109/result.json`,
SHA-256 `f38a30dcc4dfae74d1a4ce951b0ea3274cf52e47934ff97b5cbdfea9b3a539ca`.
The original usable branch then matched 85 pre-orthogonalization tangent
candidates using retained conditional UVs, positions and indices; receipt
`target/creature-skin-uv-tangent-prefix-12group-0473-prep-20261109/result.json`,
SHA-256 `8d18aaf662c4f632a26243ce6d84d8b6c82b705f7ff488a44fcaea9dfe475ea3`.
Final normal projection, normalization, material and live identity remain open.
The checked Rust prefix matched all 85 retained vectors; receipt
`target/creature-skin-uv-tangent-prefix-12group-rust-verify-20261109/result.json`,
SHA-256 `35e5cb36672343963036a7e3707a162ff22cb440c3d4892ecd760d6c8fedfa93`.
The original projection, normalization and cross-product closure subsequently
matched 85 conditional tangent/bitangent pairs; receipt
`target/creature-skin-uv-tangent-final-12group-0473-prep-20261109/result.json`,
SHA-256 `a601e8d8781acf0e8f8508fb35aa41cd1c42db1286e6cb0a218541236809a9a2`.
This admits the copied normal-index relation, frozen same-object state and CRT
continuity; material, allocation and live identity remain unverified.
The Rust projection matched all 85 retained pairs across 2,040 bytes; receipt
`target/creature-skin-uv-tangent-final-12group-rust-verify-20261109/result.json`,
SHA-256 `6f20e861a1e872a515233a8bcae20bddc7cd72121b2067aa1abd6bef6802f7bd`.
The CRT-compatible square-root spill and normal-index relationship are explicit
inputs, not a general CRT or live-identity claim.
An initial eight-group vertex-triple host exited normally but disagreed with its
expected bytes; failed receipt
`target/creature-skin-triples-8group-0473-prep-20261109/result.json`, SHA-256
`47ab22d3746569ad97e4d64f18977342780c32b786f0473b3610b6e35d58677a`.
Independent instruction-aligned review of the original caller established
that the wrapper reversed global and local face ordinals. The failure and
output are preserved and do not establish native agreement.
The corrected, independently reviewed v2 host matched all eight generated
groups; receipt `target/creature-skin-triples-8group-0473-prep-v2-20261109/result.json`,
SHA-256 `5bd2dbece6a3682c7be9b376b08a82f4d2b602962838fadb85198ccc88fb0885`.
The first four subobject face lists, native allocation and live identity remain
outside that comparison.
The separately tested original constructor and empty-face traversal matched
four preallocated groups; receipt
`target/creature-skin-empty-row-groups-0473-prep-20261109/result.json`,
SHA-256 `26ef80fe110098a929a83563da0bd89414a4bff4f46be5ce484562b5ffe4f73f`.
Intervening grouping and atlas operations were not executed, so this alone does
not establish that the four groups stay empty or yield a live base of zero.
In a separate grouping-slice host, four disjoint constructed objects retained
their checked face/index/triple descriptors; receipt
`target/creature-skin-outer-preserve-0473-prep-20261109/result.json`, SHA-256
`da4eb49f056dc26719d110c9c00013bb52fcfff44b2269f85a4c09cca26d1991`.
That observes descriptor words and pointers only; atlas, full assembly, native
lifecycle and live identity remain outside the comparison.
The same constructed objects also retained the checked descriptors through
grouping and the initial 12-group atlas bounds/rotation slice; receipt
`target/creature-skin-group-atlas-preserve-0473-prep-20261109/result.json`,
SHA-256 `5611654fc7a32233b50532be0aa44b935560ced46f6f15be34796802f5e75099`.
Shelf packing, UV transformation, later assembly and actual lifecycle were not
executed in that closure.
The checked Rust composition matched all 12,384 retained native bytes; receipt
`target/creature-skin-triples-8group-rust-verify-20261109/result.json`, SHA-256
`481a15fd57c80d9eef589f3893073f037042828a3a56efa6064847fbbca0c809`.
The original attribute append slice matched all 85 retained triples for those
eight groups, copying position, normal, UV and the tangent half; receipt
`target/creature-skin-export-attributes-8group-0473-prep-20261109/result.json`,
SHA-256 `04bdb04705a1c5e8c448e3bb0ed63b8688ba858b1d6e5c20db0af709478fe79e`.
The first four groups, allocation/lifecycle and material binding remain outside
the comparison.
The checked Rust export projection matched all 3,740 retained bytes; receipt
`target/creature-skin-export-attributes-8group-rust-verify-20261109/result.json`,
SHA-256 `bd36f8a1e903e6b298e81432cf50e36125b39de9c93558a4c1649a0638ed2978`.
The opt-in `--include-eight-group-export` inspection section reconstructs the
eight local triangle lists and checks their attributes against retained upstream
evidence. The default remains unchanged; 25 focused Python tests passed with
fixtures. Artifact `target/creature-skin-eight-group-inspection-20261109.json`
has SHA-256 `4dbf4407bd0aa524c8ef1d81054a8e5c3095c8bc15c9111b3fe4b960a341e1ee`.
The four earlier groups, live identity, lifecycle and material remain excluded.
The original available-capacity index append matched 252 local indices at each
of two supplied initial bases plus four wrap controls; receipt
`target/creature-skin-export-indices-8group-0473-prep-20261109/result.json`,
SHA-256 `93361f7912da651d997a56b58a9b10b1779ac75105fab02563b7787030b2b689`.
The actual vertex base contributed by the first four groups remains unknown.
The Rust index projection matched all 508 retained words; receipt
`target/creature-skin-export-indices-8group-rust-verify-20261109/result.json`,
SHA-256 `0831d685db53deae73c17ac364c0ce8420446a180d9506829827f80538e17f44`.
The exporter offers `--include-tangent-pairs` as a separate conditional
inspection section, checking the retained native/Rust streams and their UV,
corner and normal-index mapping. Default and older exports remain unchanged.
All 23 focused Python tests passed with fixtures; an empty-target simulation
passed 12 and skipped 11 fixture-dependent cases. The artifact
`target/creature-skin-tangent-export-audit-20261109/conditional-tangents.json`
has SHA-256 `9139bff07d27f5dd99343e09fb039c0eedef9046cc47d66a1180167d9e9897a1`.

After the timing, service and atlas-offset projections, the full workspace
suite v55 passed 1,828 test executions, with zero failures and 52 ignored across
351 groups; some examples repeat tests. Log
`target/workspace-tests-20261109-v55.log`, SHA-256
`0fa24c6bef1cb1bc0d4ac29f2e9a4b091870779f744861f468521b03c06cac37`.
This suite does not establish live behavior or complete a gameplay milestone.
The subsequent v56 workspace suite passed 1,833 test executions with zero failures
and 52 ignored across 354 groups; some examples repeat tests. Log
`target/workspace-tests-20261109-v56.log`, SHA-256
`1dfda9942fdae1367e8d6b4549c9af04cabe393961a2d9acfef40dccc34ed6aa`.
It includes the controlled startup-art path and further bounded models, not
retail stage or editor acceptance.
The subsequent v57 workspace suite passed 1,837 test executions, zero failures
and 52 ignored across 357 groups, with repeated example tests. Log
`target/workspace-tests-20261109-v57.log`, SHA-256
`db54f78ecd58dae9f7048dd24d59d728bdefa195310d2e3ed2e62359a9741b55`.
The separately run native-feature trace checks remain limited to controlled
inspection, not a retail gameplay or editor gate.
The subsequent v58 workspace suite passed 1,842 test executions, zero failures
and 52 ignored across 363 groups, with repeated example tests. Log
`target/workspace-tests-20261109-v58.log`, SHA-256
`f3db7427c7fcccbca09f6004132d789e3a77aa1d9d5cf72e5bc96355bb5e49ea`.
The subsequent v59 workspace suite passed 1,846 test executions, zero failures
and 52 ignored across 368 groups, with repeated example tests. Log
`target/workspace-tests-20261109-v59.log`, SHA-256
`2dc10f319032bbfc7643c1e1e888ad7f1447800523870e49ad8cc512ea2d7948`.
The subsequent v60 workspace suite passed 1,852 test executions, zero failures
and 52 ignored across 373 groups, with repeated example tests. Log
`target/workspace-tests-20261109-v60.log`, SHA-256
`984c9eb49d608611b6114b00fc75e2df4ab2486a81411cc62b3a309f8d38944a`.
The subsequent v61 workspace suite passed 1,855 test executions, zero failures
and 52 ignored across 375 groups, with repeated example tests. Log
`target/workspace-tests-20261109-v61.log`, SHA-256
`37090f3ae09adea5ec65ed26a41adcd49c5a4febabcae7baaa7600e73e138eb7`.
The subsequent v62 workspace suite passed 1,858 test executions, zero failures
and 52 ignored across 377 groups, with repeated example tests. Log
`target/workspace-tests-20261109-v62.log`, SHA-256
`61142a6a354765b9d812e11dd8c2bff68be04a475fc046ae9efbbe09fd95006f`.
The subsequent v63 workspace suite passed 1,861 test executions, zero failures
and 52 ignored across 379 groups, with repeated example tests. Log
`target/workspace-tests-20261109-v63.log`, SHA-256
`4b3dc01fd463455b84a874d180f60dbe07406fdb236adbca2fceda55fc22bcdd`.
The subsequent v64 workspace suite passed 1,862 test executions, zero failures
and 52 ignored across 380 groups, with repeated example tests. Log
`target/workspace-tests-20261109-v64.log`, SHA-256
`73caf72801d8447c2d56a53bd1a12bb770fe76092ae4178a8f64fa6597e32775`.
The subsequent v65 workspace suite passed 1,866 test executions, zero failures
and 52 ignored across 383 groups, with repeated example tests. Log
`target/workspace-tests-20261109-v65.log`, SHA-256
`4c7ab532452f43a8e13e81e85e4b5f82d225229c34accac8ff41773559b7d079`.

The original Creature atlas offset slice matched eight conditional group inputs
and three synthetic controls. Receipt
`target/creature-skin-atlas-offset-0473-prep-20261109/result.json`, SHA-256
`9a3860258c6ad764d5535ef7a32d298ac51b927c911a7f3f84ad77791f02fccc`.
Row placement was supplied, and four earlier groups, final scale, texture binding
and live creature identity remain unverified.
The checked Rust projection matched all 11 retained native rows; receipt
`target/creature-skin-atlas-offset-rust-verify-20261109/result.json`, SHA-256
`aa95e19a08b0736305dd955e4820e9f7ca30090f68089a4c21ad15303d9ae15a`.
The original UV transform was then run with the separately matched group offsets:
85 joined UVs and three controls matched. Receipt
`target/creature-skin-uv-offset-join-0473-prep-20261109/result.json`, SHA-256
`a0744caa2abc8804a5876dc4edc908c24a79324aed62e1df3101e0cf8266e191`.
Row x is model-derived, row y and shift are synthetic, and scale is a supplied
synthetic 0.5; the four earlier groups and live atlas remain outside this join.
The composed Rust offset and UV examples matched the same 704-byte native stream;
receipt `target/creature-skin-uv-offset-composed-rust-verify-20261109/result.json`,
SHA-256 `582a43bf1de238e865679ed0afc8821837833f584561598ad5d3dca6a02039a9`.
Input and binary hashes were checked around the comparison; binary provenance is
supported by the separate example builds, not established by the runner alone.
The partial Creature exporter now accepts `--include-model-only-transformed-uvs`
to include this conditional result and its group/corner mapping. It verifies the
pinned input and comparison artifacts and retains the synthetic-placement
limitations in the output; the default export stays byte-identical. Focused
Python checks passed 17/17 with fixtures, and 9 passed with 8 fixture-dependent
skips in a clean-target simulation. The inspection artifact
`target/creature-skin-uv-offset-composed-rust-verify-20261109/conditional-transformed-uvs-v2.json`
has SHA-256 `5448084adc8d22c867f706e1a2b88b743fbe92cc633bc3c4822864804ec91be6`.

The original Creature grouping, initial atlas bounds and shelf-selection slices
were exercised on the same twelve conditional group objects. The four initial
`+44` descriptors were checked at construction, and twelve selected descriptor
words were preserved before and after shelf selection. The retained receipt
`target/creature-skin-group-shelf-preserve-0473-prep-20261109/result.json`
has SHA-256 `b23ba70e21e98052092fb216fe0657e987538879ca8d66180d0b1d45d99c5286`.
The sorted order and square root were supplied from separate checks, with guarded
lifecycle stubs and preallocated shelf storage. Full sorting, packing, UV/tangent
processing, native object lifecycle and live creature identity remain unproven.

The subsequent v66 workspace suite passed 1,867 test executions, zero failures
and 52 ignored across 384 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v66.log`, SHA-256
`3f46f997674d576ce1605a0aa81e823a5e2dcc46abae5851728bc885c951959d`.
It does not establish retail stage or editor acceptance.

The subsequent v67 workspace suite passed 1,868 test executions, zero failures
and 52 ignored across 385 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v67.log`, SHA-256
`9aefbc0e31d11190ef77efdc61427f9080c97cf0efec6d6f5b281dfceaf85d4a`.
The separately captured RTX inspection still does not establish retail stage
or editor acceptance.

Workspace v68 did not complete: the package-intake test hit `AlreadyExists`
while parallel tests created temporary files from the same process ID and clock
value. The failed log `target/workspace-tests-20261109-v68.log` has SHA-256
`017c508dc7c4b0af13b61a60416d3f73318a3d597d45f57a896e5002f6c92b86`.
The fixture now adds a process-local atomic sequence and retries exclusive
creation on collisions; it does not overwrite the existing path. Three focused
package-intake tests passed; log `target/package-intake-tests-20261109-v1.log`,
SHA-256 `8972e5e4201803a27ad1e9fa0fa7c777fd58194d058dd22e42a586bbc804e12f`.

After the temporary-name fix, workspace v69 passed 1,870 test executions,
zero failures and 52 ignored across 389 groups, including repeated example
tests. Log `target/workspace-tests-20261109-v69.log`, SHA-256
`b5890b52e519f2049a5390f48f24bbee853fb8185b82440b2b39029105dc7333`.
This does not establish gameplay or editor acceptance.


The subsequent v70 workspace suite passed 1,873 test executions, zero failures
and 52 ignored across 393 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v70.log`, SHA-256
`0429c115c78a14acc0bbdeeb5195a7a857cfcd343b58cb78fcf48925f3243a47`.
This does not establish gameplay or editor acceptance.


### Conditional same-object inspection export

Use `python tools/creature_skin_0473_partial_export.py --include-same-object-export --output <new-path>` to opt in to the same-object inspection section. The flag also includes the required existing conditional eight-group geometry, UV and tangent sections; the default export is unchanged. The exporter pins the matched source-loop and caller-alpha receipts/streams, verifies all 85 attribute and record mappings, reconstructs and checks 252 global indices, and preserves the u16 offset and float weight words as hex. It explicitly selects caller-alpha `0x3F333333`, two passes; it does not infer the live smoothing setting or palette identity.

The retained artifact `target/creature-skin-same-object-inspection-20261109.json` has SHA-256 `d65dd328b49283d9aaa283119c76d9e55553721aa405d7e3f9776ce6b1d8f828`. The focused exporter suite passed 27/27 tests, including missing-fixture, mapping and tamper checks. The comparison is conditional on frozen geometry, supplied transformed UVs and records, a synthetic owner, preallocated append buffers and stable no-alias lifetimes. Outer owner validation, atlas cleanup, real allocation/lifecycle, live creature identity, palette identity and texture/material binding remain unverified; it is an inspection artifact, not a finished runtime mesh.

The subsequent v71 workspace suite passed 1,874 test executions, zero failures
and 52 ignored across 395 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v71.log`, SHA-256
`f5538093242dde86cfcc87b2b765c4b22ce95f170411a40a209d84acd0813e35`.
This does not establish gameplay or editor acceptance.

The same-object option now also requires the stronger original-entry receipt `target/creature-skin-export-entry-same-object-prep-v1-20261109/result.json` (SHA-256 `3b11010aa9d8de37c0b200e8b0ccd7e04bc5b8ac919e5de6dcef8c8b3943b8ff`) and requires its full native stream to equal the earlier loop stream. Original owner/44-record validation and group totals therefore ran under the supplied synthetic owner; seven reserve operations remain guarded synthetic no-growth stubs. The revised artifact `target/creature-skin-same-object-entry-inspection-20261109.json` has SHA-256 `2079819c9955efa5161e09a0beb34c2699f3dc04866506e825dd8429c99804c5`. The default export remains unchanged; 27 focused tests passed, including tampering with the new receipt and stream. Real owner identity, allocator/lifecycle, palette tail and live settings remain unverified.

The subsequent v72 workspace suite passed 1,876 test executions, zero failures
and 52 ignored across 397 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v72.log`, SHA-256
`d25c304482796e82b612c674b765328785c06f4f133f855848ef9e6f5e2f962d`.
This does not establish gameplay or editor acceptance.


### Conditional frozen-row categories

Use `python tools/creature_skin_0473_partial_export.py --include-row-categories --output <new-path>` to add ten conditional frozen-row classifications. This option includes the prerequisite same-object inspection sections and leaves the default and existing options unchanged. It pins the complete original-export category receipt and stream and the 1,400-byte frozen row snapshot; checks that the prior 28,476-byte output prefix matches the separately checked same-object stream; and recomputes all ten classifications from the source `+08` flags and `+0B` kind in source branch order. The values are `[1,1,1,1,1,1,4,4,4,4]`: **row categories, not palette IDs**.

The retained artifact `target/creature-skin-row-categories-inspection-20261109.json` has SHA-256 `cbbb6ee23a05b32ec45ed7b56b791828ec17febe53089be6417bf756d0e49219`. The focused suite passed 29/29 tests, including branch-priority, missing-fixture and receipt/stream/row tampering checks. Synthetic same-owner association, preallocated resize, frozen input and stable lifetimes remain admissions; live owner, allocation, smoothing state, palette identity and material/texture binding are unverified.

The subsequent v73 workspace suite passed 1,878 test executions, zero failures
and 52 ignored across 399 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v73.log`, SHA-256
`48cd554bb903d5fc2163baec252befda2c407fb23406ab96684ab02cd579d792`.
This does not establish gameplay or editor acceptance.

The row-category option now also pins the original `467F79..468091` category-prefix receipt `target/creature-skin-category-prefix-same-object-prep-v1-20261109/result.json` (SHA-256 `81452cd40ac77d6890bb9583e167643a5259b49635513b6f1322d6f254706477`) and its 28,740-byte stream. It requires the prior category stream to match exactly and checks the frozen prefix plus all three trailing-class-1 control snapshots, including their synthetic unchanged backing bytes. The conditional frozen-row prefix length is ten. The revised artifact `target/creature-skin-row-prefix-inspection-20261109.json` has SHA-256 `b1ac98c359aa1e9d215ed59fb2492419df99f3122bb3a04bc5ea9610ed50894c`; 29 focused tests passed. External-count bypass and logical resize are synthetic admissions; the categories are not palette IDs and no live binding or destructor behavior follows.

The subsequent v74 workspace suite passed 1,880 test executions, zero failures
and 52 ignored across 401 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v74.log`, SHA-256
`7ef87ee1efec0474339cc259d7e4c4b6aa2ab8f4803cef682d36bb8859befa93`.
This does not establish gameplay or editor acceptance.

The subsequent v75 workspace suite passed 1,882 test executions, zero failures
and 52 ignored across 403 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v75.log`, SHA-256
`dfb0b3f74eb50fdc213a021046d63aaace0716f5602e2244c9db8849c4194ae2`.
This does not establish gameplay or editor acceptance.

The subsequent v76 workspace suite passed 1,884 test executions, zero failures
and 52 ignored across 405 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v76.log`, SHA-256
`b444b6fb46ccd1fbf179045ebf299517344991ef12953fc3c4acab95b542b3da`.
This does not establish gameplay or editor acceptance.

The subsequent v77 workspace suite passed 1,886 test executions, zero failures
and 52 ignored across 407 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v77.log`, SHA-256
`a847d6fa89b47aca5ade411c0658202d57684c26817536a88b2b7aef32d4597c`.
This does not establish gameplay or editor acceptance.

The subsequent v78 workspace suite passed 1,888 test executions, zero failures
and 52 ignored across 409 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v78.log`, SHA-256
`6d40783736a76e700d63efe19065df9f37b6b250a82cf25cf2349015ac9c2b16`.
This does not establish gameplay or editor acceptance.

The subsequent v79 workspace suite passed 1,890 test executions, zero failures
and 52 ignored across 411 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v79.log`, SHA-256
`492561ed326ff88198c567796251628cbd01622c18b9c19de9f27934842efc01`.
This does not establish gameplay or editor acceptance.

The subsequent v80 workspace suite passed 1,892 test executions, zero failures
and 52 ignored across 413 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v80.log`, SHA-256
`87aff5fc8c8cb0e034b4985ae7ded355ffc2188740375a63d3efedcb187db0a2`.
This does not establish gameplay or editor acceptance.

The subsequent v81 workspace suite passed 1,893 test executions, zero failures
and 52 ignored across 415 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v81.log`, SHA-256
`612ee86b9db5647b4e8826b2de0d98ac94e4e9dc7138cf41103a745c8cdedbdb`.
This does not establish gameplay or editor acceptance.

The opt-in `python tools/creature_skin_0473_partial_export.py --include-triangle-bounds --output <new-path>` adds 84 conditional source-loop triangle extrema as raw binary32 words. It verifies pinned position-selector and bounds receipts and complete prefixes, the same-object 85 positions and 252 supplied indices, finite values, and source tie selection. Its artifact `target/creature-skin-triangle-bounds-inspection-20261109.json` has SHA-256 `3d5bc4bbfbe840ccb20cd3df38585c05164ffbf04f362ca2fb08406a919a4a77`. Default output is unchanged; supplied group/index mapping and non-remap state do not establish source group selection, actual allocation/lifecycle, renderer or live identity.

The subsequent v82 workspace suite passed 1,894 test executions, zero failures
and 52 ignored across 416 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v82.log`, SHA-256
`4ffb7db590827188688581df1c822bcf199d5a02355ffcff1ec5f8ab0d04c949`.
This does not establish gameplay or editor acceptance.

The opt-in `python tools/creature_skin_0473_partial_export.py --include-aggregate-bounds --output <new-path>` adds six conditional aggregate-bound binary32 words and includes the triangle prerequisites. It checks the matched source-reduction receipt, full triangle prefix and both finite tie/invalid controls. Fixed artifact SHA-256 `87b0e41d40cc80e6dda204a328b4f78daff689a92f0c05388f582d1911a4a822`; no spatial-tree, live ownership, rendering or identity claim follows.

The subsequent v83 workspace suite passed 1,895 test executions, zero failures
and 52 ignored across 418 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v83.log`, SHA-256
`ee6815bc1736de25222f497b386575207f0997dda2ecc15e248ff38b6f759b6e`.
This does not establish gameplay or editor acceptance.

The subsequent v84 workspace suite passed 1,897 test executions, zero failures
and 52 ignored across 420 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v84.log`, SHA-256
`298e0ce8291245879c1cf608b66f48d0f7f95bdf0179c161961505ae0b3e33d1`.
This does not establish gameplay or editor acceptance.

The subsequent v85 workspace suite passed 1,899 test executions, zero failures
and 52 ignored across 422 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v85.log`, SHA-256
`fc609f551cf96a9851f6a0b5c7637ec694be351dcff528a817b3ac4dfa15191b`.
This does not establish gameplay or editor acceptance.

The subsequent v86 workspace suite passed 1,901 test executions, zero failures
and 52 ignored across 424 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v86.log`, SHA-256
`c399f36b52a2bfee9fe190c883ae8726250130900a45778f69cd8c8d80f7b649`.
This does not establish gameplay or editor acceptance.

The opt-in `python tools/creature_skin_0473_partial_export.py --include-synthetic-cpu-deformation --output <new-path>` adds 85 native-matched CPU-deformed positions using ten explicit synthetic matrices. It pins and verifies the source receipts, full prefix, palette, all lane offsets and finite inputs, and compares source-order binary32 output. Fixed artifact SHA-256 `09884d6953894b4fcc6ef5a25bba8e8bab314287889cb41de9f053ca1ae0127f`. Default output is unchanged; normals/tangents are not deformed and no live pose, palette, world or GPU claim follows.

An opt-in Bevy offscreen wireframe is available via `target/debug/rustspore.exe --experimental --synthetic-0473-preview target/creature-skin-synthetic-deformation-inspection-20261109.json --shot <new target PNG>`. It admits the fixed SHA-bound artifact, displays an explicit synthetic/position-only/not-live label and does not fabricate normals or materials. The reviewed hidden capture receipt SHA-256 is `bacf16acace3a3b4c1d092f3c9264e18fcd2f6e9d6dab4bb06a3cf9cf24b1819`; preview SHA-256 `fbafbea22f72450c18003648e63090630c929de4e99d1ce0d0a05efc3774ef06`. It is a small partial wireframe from one synthetic pose, not live gameplay or a complete creature. Loader error-13 informational and missing validation-layer warnings appeared in the log; no Bevy pipeline failure was observed.

The subsequent v87 workspace suite passed 1,904 test executions, zero failures
and 52 ignored across 427 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v87.log`, SHA-256
`0b6c066202d901e1c1455044e009e6f4f6bbcc51e6b9cf80c111a4593c7f64ba`.
This does not establish gameplay or editor acceptance.

The independent original CPU direction fixture matched 85 normal and 85 tangent vectors using the explicitly synthetic matrices. A separate hash-bound artifact `target/creature-skin-synthetic-directions-inspection-20261109.json` (SHA-256 `3393ad82bdb8b3ed8cf556bad192d49a0d8a28dc84e4759725b4ba33d055bd7f`) and opt-in `--experimental --synthetic-0473-directions <artifact> --shot <new target PNG>` visualize sampled green normals and orange tangents on the conditional wireframe. Parent-coordinated hidden RTX/Vulkan capture receipt SHA-256 `46f438ef95abd22bbc4229f928aa3851a4ae7e9ad6b4aaa0c7e680e2e935dc85`, PNG SHA-256 `27a4bc80afeb00895cec093f4085b2a9ae3220478a641f0511dcd7def66363ae`; the labels and glyphs were visually inspected. This is a synthetic partial diagnostic, not live pose, material, animation or generalized x87 proof. Existing position-only mode and default output remain unchanged.

The subsequent v88 workspace suite passed 1,909 test executions, zero failures
and 52 ignored across 429 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v88.log`, SHA-256
`bd63b2d5ed54e14723fc0695f496733836b3b23f6e1fe84a72446d2dad7bd273`.
This does not establish gameplay or editor acceptance.

The subsequent v89 workspace suite passed 1,911 test executions, zero failures
and 52 ignored across 431 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v89.log`, SHA-256
`15430bb54a57ed380f202d754f48ec5d01ee41f16b7795b086bb3e978e556a13`.
This does not establish gameplay or editor acceptance.

Correction: the direction host used x87 control `027F` (53-bit significand precision), while its initial Fraction oracle incorrectly used 64 bits. A separately retained corrected 53-bit replay matched all 85 normals and 85 tangents in the fixed fixture (`target/creature-skin-directions-cw027f-audit-v1/receipt.json`). This does not establish general x87 equivalence. The original host and receipt remain unchanged.

The subsequent v90 workspace suite passed 1,912 test executions, zero failures
and 52 ignored across 433 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v90.log`, SHA-256
`417be864bc91d692a86984fa1df31c86381f6fc6d9f06e0defb4011f8f475a58`.
This does not establish gameplay or editor acceptance.

The reviewed original initializer → source insertion sort → builder → lookup
fixture matched all 22 descriptor keys at x87 controls 027F and 037F. Its
23-record sort includes the PE `FFFFFFFF` callback terminator, which sorts
last and is excluded by the builder. Receipt
`target/source-cell/e137edc0-sorted-descriptors-result-v2/receipt.json` SHA-256
`235a6654cd18fab1c8cb14f88b3c662a6b7cb4253adae0b5f47e5ae2683668f2`;
native stdout SHA-256
`20e1c971028267655ca8d6919584bd21be8f9dbdebf109d17378b2124ae3efdb`.
The earlier initializer-only misses do not describe this sorted fixture. The
startup trace places the two initializer pointers in that order inside the
normal CRT `_initterm` range; this remains conditional on normal completion
and unmodified tables, not an observation of live state or file behavior.
The checked Rust fixed-key sort and lookup matched both retained rows; focused
tests passed. Comparison receipt
`target/source-cell/e137edc0-sorted-descriptors-rust-compare-v1/receipt.json`
SHA-256 `ed7f7e7ceadaa44568c83659937996c2be437979d0fdd6ca4dfceaee206d854c`.

The subsequent v91 workspace suite passed 1,916 test executions, zero failures
and 52 ignored across 436 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v91.log`, SHA-256
`b8eace4e81f7492768ba505f136e5e782d79fdf23fb9b424c91a8201778cb689`.
This does not establish gameplay or editor acceptance.

The original quaternion/product/gather/deformation v4 comparison matched a genuinely nonidentity composite for explicitly synthetic identity rest bases and alternating identity/180-degree live quaternions; receipt `target/creature-skin-quaternion-product-deform-prep-v4-20261109/result.json` SHA-256 `4da5fd8744fe5208a7441a8ab5d53fb09493d0d1239ecf1b2d3a87ae1250ac5c`, stdout SHA-256 `6ffcfe6dad216cf13f4128a9b4022a36fd09cf1be046ac81571332b218fa446d`. Separate artifact `target/creature-skin-synthetic-halfturn-inspection-v2-20261109.json` SHA-256 `7f83a46dd1487e7d542002a0b7fe977b07b915bde26a858ee3260b74a374439e` is produced by `python tools/creature_synthetic_halfturn_export.py <new target JSON>`; opt-in preview command is `target/creature-skin-halfturn-preview-bin-v1/rustspore.exe --experimental --synthetic-0473-halfturn <artifact> --shot <new target PNG>`. Tests: Python 3/3 and focused Rust 1/1; promotion and runner reviewed. The first capture child exited 0 but failed the adapter-info gate under default logging (receipt SHA-256 `bbf2b9d8ac02f6f0456ee5ae7210abdf9ed729139bf8b315075880a6690a7357`), and is retained as failed. A fresh reviewed v3 runner explicitly requested RTX 3060/Vulkan and INFO logging; captured receipt SHA-256 `af231162e826abbacaa1ec79ec8a39d26bf6482b3a5bbf486777cdce86c6a8e7`, PNG SHA-256 `d55153865702b1f46aacb871e4cfef25651f274c91b21e517bb77546829b39a0`. Its receipt leaves visual inspection pending; later separate inspection confirmed the cyan partial wireframe, colored normal/tangent glyphs and synthetic/not-live label. This is not a live pose, material, animation, palette identity or full creature.

The subsequent v92 workspace suite passed 1,921 test executions, zero failures
and 52 ignored across 438 groups, including repeated example tests. Log
`target/workspace-tests-20261109-v92.log`, SHA-256
`280a18d63d64eb6e3a0260685eb5d42c6300b95504bbc78ae5fbf17a4414c99f`.
This does not establish gameplay or editor acceptance.

The v93 workspace suite exited successfully (1,923 test executions, zero
failures, 52 ignored), but an example changed during its build; it is retained
as a concurrent run, not a stable source-tree verification. Log SHA-256
`1ee490e97f06b3f09bb1b5ed03822e19c1a43a90bf405089d9086c51e7a1b669`.
The subsequent v94 suite ran after shared Rust edits were held: 1,923 test
executions passed, zero failed and 52 were ignored across 439 groups,
including repeated example tests. Log `target/workspace-tests-20261109-v94.log`,
SHA-256 `26a548bdb67205bdbf34f0e25b5e136acce79a30d76b1d6a275f8348e39f6da0`.
Neither suite establishes gameplay or editor acceptance.

REA connection check: `npx -y rea-agents@6.5.0 doctor --client codex --json`
reported the Codex registration aligned and Ghidra 12.1.4 headless/Java healthy.
The current session does not expose REA MCP tools despite that registration;
its live server identity is unknown. Existing CLI and retained binary analysis
remain available. No client configuration was changed.

The subsequent v95 workspace suite ran with shared Rust edits held and passed
1,929 test executions, zero failed and 52 ignored across 442 groups, including
repeated example tests. Log `target/workspace-tests-20261109-v95.log`, SHA-256
`e694cbb21119c4991e6e5ec7090eebeb74c475c5b64dd034c71110747ec57d49`.
This does not establish gameplay or editor acceptance.

The subsequent v96 workspace suite ran with shared Rust edits held and passed
1,931 test executions, zero failed and 52 ignored across 445 groups, including
repeated example tests. Log `target/workspace-tests-20261109-v96.log`, SHA-256
`3acc8fcb4bbfba97a1168f5f559350a5ff1da3c4f7e46258d76166757fead464`.
This does not establish gameplay or editor acceptance.

The subsequent v97 workspace suite ran with shared Rust edits held and passed
1,934 test executions, zero failed and 52 ignored across 446 groups, including
repeated example tests. Log `target/workspace-tests-20261109-v97.log`, SHA-256
`b95e233ab5a0cc4058eaf0607602eddd722e268fbf877d729855f653161c3d56`.
This does not establish gameplay or editor acceptance.

The subsequent v98 workspace suite ran with shared Rust edits held and passed
1,939 test executions, zero failed and 52 ignored across 449 groups, including
repeated example tests. Log `target/workspace-tests-20261109-v98.log`, SHA-256
`07ed0b5b42a4798f342b42410e9e9dfdf466ae9b408a4d3d5113aabad307a697`.
This does not establish gameplay or editor acceptance.

The subsequent v99 workspace suite ran with shared Rust edits held and passed
1,941 test executions, zero failed and 52 ignored across 450 groups, including
repeated example tests. Log `target/workspace-tests-20261109-v99.log`, SHA-256
`516abb1f477514dc07e8b0701919eaa1a6709819578f16f6c46946a65a615a74`.
This does not establish gameplay or editor acceptance.

The subsequent v100 workspace suite ran with shared Rust edits held and passed
1,944 test executions, zero failed and 52 ignored across 451 groups, including
repeated example tests. Log `target/workspace-tests-20261109-v100.log`, SHA-256
`720f18b4937e6f9b7414fb71bb9d5ded661fa3b4a90c9ff12db7d8f8494125c9`.
This does not establish gameplay or editor acceptance.

The subsequent v101 workspace suite ran with shared Rust edits held and passed
1,947 test executions, zero failed and 52 ignored across 456 groups, including
repeated example tests. Log `target/workspace-tests-20261109-v101.log`, SHA-256
`f5111c6e7f0b9ed6c8874165e3155c51e6d4c035e65613f213db8d5522027d1f`.
This does not establish gameplay or editor acceptance.

The subsequent v102 workspace suite ran with shared Rust edits held and passed
1,948 test executions, zero failed and 52 ignored across 458 groups, including
repeated example tests. Log `target/workspace-tests-20261109-v102.log`, SHA-256
`036cb8823b69f4e36a15c80e206cda41afdced0cc0bd3dfc712df88ad79f8ff8`.
This does not establish gameplay or editor acceptance.


The independently reviewed original `73BB00` fallback assembly matched one
controlled 10+5+7+7+7 group case, with a supplied ten-matrix prefix and four
scripted identity callbacks. The retained native result is
`target/animation/e219-fallback-assembly-result-v1/receipt.json`; stdout SHA-256
`81be67cf29226b220ec448ac9deab578fca57f5beb553adf152cfceccd5c9682`.
The child was reaped and its private desktop closed. `spore-core` example
`e219_fallback_gather_compare_v1` independently compared the existing Rust
palette gather's 1,728 matrix bytes and the two output counters against that
retained native output using hash-pinned input files. Its log is
`target/e219-fallback-gather-compare-v1.log`, SHA-256
`882a4bcd606d1bc27db952206b882682e8fb559378bf0bf13c44fab859514c4f`.
This comparison does not model the original evaluator or establish resource
fixup, live actor identity, a GPU draw, or a real animation pose. The example
does not compare the native FP/status words; the native host checks them.


The subsequent v103 workspace suite ran with shared Rust edits held and passed
1,948 test executions, zero failed and 52 ignored across 460 groups, including
repeated example tests. Log `target/workspace-tests-20261109-v103.log`, SHA-256
`6dccdbcac8b34199143e0798f1d575fad4b84534aa5855b7e6b95284fc6047f4`.
This includes compilation of the two new opt-in examples and does not establish
gameplay or editor acceptance.


The subsequent v104 workspace suite ran with shared Rust edits held and passed
1,953 test executions, zero failed and 52 ignored across 464 groups, including
repeated example tests. Log `target/workspace-tests-20261109-v104.log`, SHA-256
`6324e78e4402966df69deb1ad3459dcbd81b133a7bcd93206273009f96debc21`.
This does not establish gameplay or editor acceptance.


The subsequent v105 workspace suite ran with shared Rust edits held and passed
1,954 test executions, zero failed and 52 ignored across 465 groups, including
repeated example tests. Log `target/workspace-tests-20261109-v105.log`, SHA-256
`7d0466b00857c637ba078f48ced8a4c366a87dcc24de26a99372b668ab465d39`.
This does not establish gameplay or editor acceptance.


The subsequent v106 workspace suite ran with shared Rust edits held and passed
1,956 test executions, zero failed and 52 ignored across 466 groups, including
repeated example tests. Log `target/workspace-tests-20261109-v106.log`, SHA-256
`a3e130ddc2418a90d8443d738a96603c27ebf9a05890d45cb87583ff1090928c`.
This does not establish gameplay or editor acceptance.


The subsequent v107 workspace suite ran with shared Rust edits held and passed
1,957 test executions, zero failed and 52 ignored across 467 groups, including
repeated example tests. Log `target/workspace-tests-20261109-v107.log`, SHA-256
`ccc47fee6766367f4caf9bbf581b8643f64267566c5e46a0e760edfd2da79515`.
This does not establish gameplay or editor acceptance.

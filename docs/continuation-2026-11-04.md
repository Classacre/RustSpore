# Continuation, 2026-11-04

The user resumed the full source-faithful Rust remake after freeing disk space.
The earlier published checkpoint is `cdb7350d3e26aea3d556ea4f21eb8c8255af5f3c`.
No stage or editor has passed the original-fidelity gate. Existing untracked
research drafts and ignored original-data captures are preserved.

- The Cell prototype's source-atlas inspection now decodes and uploads all
  declared standalone RASTER mip levels, verifies the base level against the
  selected source asset, and clamps the sampler to the available levels. The
  separate kind-0 RW4 path now also uploads its eight checked DXT5 levels.
  An opt-in installed-data test checked all 65 loadable baked Cell definitions: all 65
  had supported samplers and complete matching mip chains. The explicit
  Jawhead run loaded 10 levels and rendered with `NVIDIA GeForce RTX 3060 Laptop
  GPU`/Vulkan using the command below. Its output is an experimental PBR scene
  and prototype simulation, not a retail source-game capture. The PNG SHA-256
  is `617f6518ab66ab65767536bc2a86998641afbd6f95bad323ba20e4cc77706e1d`.
  A separate kind-0 RTX/Vulkan capture, using the same command with Cell ID
  `FD24574B`, produced `target/cell-kind0-mips-20261104-v1.png` (SHA-256
  `ef0d70ac57bfac0f06a638521c4ef021346b3593d6a5ef3e5ab671aacd5d6f14`).
  Its effect and second shader stage remain unresolved and unrendered.
- The Creature inspection path now uploads the selected standalone source
  texture's complete validated mip chain and follows its source mip setting;
  unsupported RW4 mip filtering fails closed. A hidden Vulkan capture on the
  RTX 3060 loaded all 10 levels for `40616200A1C46DF2` and wrote
  `target/creature-source-mips-20261104-v1.png`, SHA-256
  `7651813e73968058d0e542a0f17c76ba955da3b19370f520a9ddfbc4789a37b3`.
  This shows static source art in the prototype scene and does not establish
  retail Creature visuals, animation, active mip selection, or gameplay.
- An explicit, offscreen-only Creature frame experiment renders nine quads
  from a selected installed SPUI layout and source atlas. A hidden RTX capture
  shows the frame but it overlaps prototype target text; separate baseline and
  overlay captures differ outside the frame and do not establish confinement or
  native rendering equivalence. The screenshot predates its inspection notice.
  Fixture inputs and limits are in `re/creature-source-frame-overlay-v1.md`.
  An alternate fixture now computes the frame's local rectangle from its
  serialized SimpleLayout procedure and a supplied parent span/translation;
  the parent procedure and native root-to-screen transform remain unresolved.
- The RW4 animation viewer preserves the loop sampling policy across pause;
  explicitly selected fixed poses remain clamped. Its focused playback-system
  test passed. This is not evidence of retail interpolation.
- The generic viewer no longer silently substitutes an unrelated indexed
  texture when source texture resolution fails. Its texture-choice regression
  passed; manual indexed-texture selection remains an inspection option. A
  headless graphics-package viewer capture displayed mesh 0 as untextured with
  the explicit missing-source-texture label in
  `target/viewer-source-texture-20261104-v1.png`.
- The experimental editor now selects the validated creation root on startup,
  load, history fallback and deletion instead of assuming root ID 1. The
  nondefault-root load/history regression passed. The catalog label states that
  it is a prototype.
- Full `tools\build.bat test --workspace --all-targets` passed 947 tests,
  with 0 failures and 11 ignored, across 196 result groups. Local log:
  `target/workspace-tests-20261104-v18.log`, SHA-256
  `576d29998ac09e1fc147e1a4c00e52da678a529cf932ccc486cbd92039f6eae9`.
- The static Cell UI progress route is documented in
  `re/cell-ui-progress-value-route-v1.md`. It is a source-code observation,
  not evidence of the active layout or a complete runtime HUD. The traced
  input is incremented through a foodValue-linked feeding path; the meaning of
  the separate event-derived displayed value remains unestablished. An
  installed census found the eight traced controls in both explicit
  `GlobalUICell-3` copies (748 SPUI records scanned); the graph audit at
  `target/source-ui/cell-ui-progress-graph-audit-v1.json` records their
  references. The bounded proportional-layout helper passed three synthetic
  tests, and its source coefficient decoder matched 12 relevant records across
  both candidates. A separate independent binary32 comparison matched 1,032
  controlled operation rows under stated rounding and input assumptions; this
  did not execute the original game. Active layout, parent transforms and
  native draw order remain open. A bounded `StdDrawable` state/slot selector
  also matched 8,192 controlled state and occupancy cases against an independent
  branch model; 80 raw slot words also matched independently decoded installed
  candidate data. A mode-2 rectangle emitter matched 706 controlled
  fixtures against an independent transcription, and explicit single-segment
  source-atlas diagnostics rendered from both candidate packages. These use
  supplied layout/state assumptions and are not observed runtime HUDs. The
  original-shader offscreen GPU result was reproduced after extracting a
  reusable renderer, including checks with unfavorable inherited D3D state.
  An opt-in Cell capture composites this single controlled UI segment over the
  original-program 3D model; pre/post baseline and outside-region pixels match.
  It is not the live or complete HUD.
- The conditional animation-owner-to-palette path and unresolved provider
  identity are recorded in `re/cell-animation-owner-palette-draw-static-v1.md`.
- A bounded Rust XAS1 SNR RAM decoder matched six installed Spore_Audio2
  samples against vgmstream byte for byte; its package survey admitted 2,067 of
  2,873 records and classifies other codecs and streamed/looped cases separately.
  The port comparison subsequently matched all 2,067 admitted records against
  vgmstream; mid-run and post-run package hashes match but there was no pre-run
  hash. Separately, all 45 admitted looped RAM records matched PCM and loop
  start values with matching pre/post package hashes; 8 streamed/gigasample
  records were excluded from that RAM survey. Two explicitly paired streamed
  resources have since matched vgmstream; other stream variants remain open.
  These are port comparisons, not independent codec or game-runtime tests. Ten bounded PCM16BE records were also compared
  against vgmstream and direct byte conversion, all matching; package stability
  was not independently admitted for that scan.
  `source_snr_wav` now exports an explicitly keyed supported resource to a new
  WAV under `target/` without an external decoder; see README. This is not
  original-game playback or event-routing evidence.
- An earlier source audio note incorrectly classified a RIFF ACON animated
  cursor as WAV. `re/payload-formats.md` now records the correction and the
  bounded package scan; original audio decoding and routing remain open.
- Source Cell player-target and conditional mouse-button selection paths are
  recorded in `re/cell-player-target-input-static-v1.md`. Dispatch, active
  camera and live actor association are not yet established.

```powershell
$env:WGPU_ADAPTER_NAME='RTX 3060'
tools\build.bat run -p spore-bevy -- --experimental --cell --source-cell-install 'D:\Games\SPORE Collection' --source-cell-id 7D00E384 --offscreen --demo 1 --shot target/cell-source-mips-20261104-v1.png
```

Use a new output path for a repeat capture. Preserve installed game files and
keep their original assets and captures out of Git. The `target/` output is local
and may be absent after cleanup.

The EAL31b framing experiment reconstructs free-format MPEG1 frames and reports
PCM insertions separately; it does not yet produce complete source audio. The
source waveform comparison used the same mpg123 backend as the reference, and
FFmpeg rejected the free-format reconstruction. See `re/payload-formats.md`.
Literal planar PCM insertions can now be extracted as interleaved samples. A
bounded WAV verifier matched 47 stereo frames from one selected insertion to
the retained vgmstream output (`target/eal31b-insertion-verify-v3.log`); this
does not establish insertion timing, MPEG decoding or complete playback.
A narrow first-insertion assembler also accepts caller-supplied decoded PCM and
an explicit alignment. Two selected sources matched retained WAVs within
explicit post-exploration limits (maximum difference 2, at most 5,000 samples
over one). Both used the same mpg123 backend; the decoder/source association,
general insertion placement and live audio remain unverified.
An opt-in Windows example can now load a caller-supplied trusted mpg123 DLL
and export that narrow layout to a new WAV under `target/`. Two source previews
reproduced the bounded comparisons, and the focused Windows test verified
handle-based cleanup after an injected write failure and no overwrite of an
existing file (`target/eal31b-preview-tests-20261104-v1.log`). The DLL and its
dependencies, package paths and caller-supplied alignment require trust and
stable-file admissions; no decoder binary is included or game event mapped.
The experimental Cell app separately supports an explicitly selected non-looping
PCM16/XAS source-audio inspection preview. The selected installed XAS payload
decoded to the expected PCM digest, and three headless tests verified selection
guards, the PCM iterator, and startup player/asset wiring
(`target/cell-source-audio-preview-headless-v2.log`). No audio device or
audible playback was tested, and this does not establish source event selection.
A supplied-tree Cell picking model matched 1,452 ordered replies from bounded
relocated original instructions on synthetic inputs. The first two host attempts
failed at address allocation before source execution; both are retained.
A separate bounded original-instruction comparison matched 1,944 synthetic
tree construction/update histories, including ordered nodes, geometry bits and
pending lists. A separately corrected filter host matched 54,000 synthetic
predicate/provider-call replies; its earlier partial failed run is preserved.
Live population, name lookup, actor selection and status-flag equivalence remain
open.

The conditional Cell definition name lookup matched an independently decoded
489-copy cohort under explicit en-US provider and raw-return assumptions: 128
locale names, 12 embedded fallbacks and 349 unresolved creation-provider paths.
The active locale, cache and provider remain unverified. The alternate raw
manifest prefix passed synthetic tests; no matching zero-discriminator record
was found in the six scanned base-package indexes.

Further Cell source trace: the primary camera update obtains the manager camera
used by cursor picking under an unchanged-pointer assumption; viewport, player,
curves and active setter still need runtime admission. The player constructor
uses source size and orientation draws, but saved-state transfer and its
event/service side effects remain conditional. See
`re/cell-cursor-ray-producer-static-v1.md` and
`re/cell-player-actor-pick-producers-static-v1.md`.

Explicit source building/vehicle and UFO model candidates were located and
rendered as untextured geometry; their saved sampler availability is inherited
and the candidate draw state is unresolved. See
`re/civilization-source-model-inspection-v1.md` and
`re/space-source-ufo-inspection-v1.md`. Do not assume the asset category proves
live stage selection or package precedence.

An opt-in `--source-player` inspection now loads authenticated installed
PatchData and Spore_Content resources, takes an explicit nonzero seed and
progress value (default zero), and classifies the selected creation's part/Parent
chains in an explicit package. A seed-1 run resolved 10 parts and 41 chain
visits against retained independent classification evidence. The controlled
composition now also computes a capability snapshot from those parts and pinned
source curves, and projects it using a deliberately supplied actor position,
matrix, handle and cache association. It still returns `UNRESOLVED` without
publishing a player. Empty cache, list-to-runtime, RNG continuity, scope,
resolver behavior and stable file paths remain controlled assumptions; they
are not live retail observations.

Controlled progress selection reads its table from a bounded, hash-checked
executable snapshot; installed-data checks for progress 0, 50 and 100 passed
(`target/source-player-progress-test-v1.log`). The rebuilt CLI also ran the
progress-50 case and returned its intended unresolved status, with output in
`target/source-cell/source-player-progress50-cli-v1.log`. The original runtime progress,
mounted resource and player identity remain open; see
`re/cell-player-start-definition-static-v1.md` for the memory assumptions.

For seed 1, the controlled cache center and radius match the retained bounded
original-instruction cache prefix for creation `E219E5D3` (ordinal 473):
`3713D4A0 BEC363C6 00000000 3FAFBDA8`. The retained
`target/source-cell/capability-source-composition-v1/creatures.tsv` maps that
ordinal to the same creation digest and ten rows; `cache-output.bin` has status
1 and those words at offset `473 * 144 + 56`. Both comparisons use x87 control
`027F`, MXCSR `1F80`, and a zero empty-center input. The retained cache file's
SHA-256 is `a2a9964d1a5532970ed4a68106e472ac53329e3afdb57c51b9e37f4d9849e549`;
the controlled run is under `target/source-cell/controlled-source-player-capability-run-v1`.
This checks the four cache geometry words, not the scalar fields, actor
projection, mounted provider, cache-to-actor association, or live gameplay.

The opt-in `--source-sweep-fixture` runs one headless Bevy update joining a
checked synthetic actor pool, conditional selected-actor target copy, bounded
sweep dispatch and the strict 3D gate.
An optional explicit installed CELL definition is retained as opaque provenance;
its inspection package order is not a verified retail mount order. The actor
fields, 2D bounds and independently supplied 3D geometry remain synthetic.
The focused run passed six tests, including the local installed-definition
case, stale and foreign handles, signed pair words, capacity, tied endpoints,
target-copy identity checks, and strict gate pass, equality, failure and
arithmetic error. Its log is
`target/source-cell/sweep-fixture-tests-20261104-v2.log`. This is a fixture
handoff, not live CellWorld integration, native registration or gameplay.
The rebuilt command-line executable ran the installed-definition case with
target-copy integration; its output is
`target/source-cell/sweep-fixture-cli-20261104-v2.log`.

An independently reviewed static trace of the Cell contact tail identifies
a direct selector `0x16` path that can suppress later feeding dispatch. See
`re/cell-feeding-tail-e6a2b0-static-v1.md` for the float-state and unresolved
producer, mutation, cache and reachability boundaries. No original instruction
execution or live feeding behavior is claimed by that trace.
The bounded finite-input tail model in `cell_feeding_tail_e6a2b0_v1.rs`
implements the direct write and leaves arbitration as an explicit pending
submission. Its three focused tests pass; external outcomes and refreshed,
non-aliasing snapshots are caller admissions. A separately reviewed relocated
original-instruction host matched all 12 output words in 864 finite cases for
the fixed relation-0 direct branch, with MXCSR `1F80`; the one-shot receipt is
`target/cell-tail-direct-results-v1/receipt.json`. Build receipts are post-build
observations, dependencies are not fully pinned, and source paths must remain
stable. Relation 5, callbacks, the complete producer and gameplay are not
covered by that comparison. A second reviewed host matched all 12 words in
1,008 relation-5 direct-path cases with a controlled nonzero `E6A250`
substitute, recording the expected stub call and argument. Its receipt is
`target/cell-tail-relation5-results-v1/receipt.json`; it does not test the
original callback or its effects. See `re/cell-tail-direct-native-preflight-v1.md`
and `re/cell-tail-relation5-stub-preflight-v1.md` for invalidated preparations,
runtime and build limitations, and review boundaries.

The preceding `E673A0` call can mutate a shared record before later contact
decisions. Its unsigned low-half pair key and write ordering were independently
checked against installed instructions; see
`re/cell-feeding-e673a0-producer-side-effect-v1.md`. A complete producer needs
ordered stop/resume handling and fresh snapshots across that call.
The bounded `cell_feeding_producer_prefix_v1.rs` models the early exits and
two pending candidate submissions, stopping before all remaining operations.
Five focused tests pass; continuation must preserve retained pointers and
admit the source's per-site re-reads, callback mutations and x87 state. This
prefix has not yet been compared to original instruction execution or connected
to live gameplay.

A separate x86 wrapper smoke test passed on a synthetic thunk, checking the
specified FCW, FSW, abridged tag, MXCSR and eight XMM fields on normal return.
The one-shot receipt is `target/cell-tail-x87-smoke-results-v1/receipt.json`.
It does not cover complete FP state, AVX, fault recovery, or source code. A
subsequently reviewed host compared 216 finite relation-5 submission cases
against original instructions with three controlled pure stubs: six submitted
scalar words, six model geometry words and all 12 unchanged output words
matched. It checked ordered calls, pointers and input snapshots; its receipt is
`target/cell-tail-submit-results-v1/receipt.json`. Actual callback effects and
arbitration remain unverified, along with the other branch admissions and
stable-file, build-provenance and fault boundaries in
`re/cell-tail-relation5-submit-comparison-plan-v1.md`.

Fresh REA CLI processes can use the installed Ghidra 12.1.4 headlessly: a
separate small-binary check returned Ghidra pseudocode and assembly. The
already-running MCP server inherited an older environment and still needs
reconnection to see the user-level `GHIDRA_INSTALL_DIR`. Full Spore analysis
timed out after both the default 330 seconds and an explicit 900-second retry
without a function dossier; diagnostics are retained in
`target/rea-ghidra-provider-check-20261104.json` and
`target/rea-ghidra-provider-check-900s-20261104.json`.
Direct headless Ghidra with automatic analysis disabled subsequently
disassembled and decompiled one explicitly seeded function at `00A00B30`; the
retained project and logs are under `target/ghidra-selective-v1`. Its two
binding-call addresses were independently checked against the executable, but
the incomplete decompiler context does not establish signatures or live actor
binding. Installed bytes and paths were assumed stable during import.

The controlled `cell_source_anim_binding_v1.rs` handoff separately preserves
creation, ANIM, actor, owner, cache and binding identities, and requires explicit
admission of the nonnull reference-projection path. Its opt-in installed test
passed for a selected Jawhead creation and PatchData ANIM metadata, with a
separately supplied retained channel fixture; a modified ANIM fails the pinned
digest. The fixture comes from the same selected clip, so this is not independent
clip provenance, live actor selection, complete binding or playback. The local
log is `target/cell-source-anim-binding-installed-test-v1.log`; package paths
must remain stable during the check. The native ANIM key in this test has group
`00000000`; the compatibility index's `0001FFFF` spelling is not interchangeable.

The zero-mapping state-4 consumer arithmetic is available as a checked relative
upload-argument diagnostic in `gmdl_state4_upload_diagnostic_v1.rs`. Two tests
cover the recorded Jawhead descriptor values and checked bounds; their local
log is `target/source-cell/state4-upload-tests-20261104-v1.log`. Runtime payload
start, mode, matrix pointer and descriptor-to-mesh selection remain unverified.
No animated draw or pointer-lifetime claim follows from this diagnostic.
The source producer can replace and conditionally free its previous matrix
vector when growing it, and a separate failed-initialization cleanup can free
the vector. The independently checked trace is in
`re/cell-palette-vector-growth-cleanup-v1.md`; it does not establish a live
dangling read or update-to-draw lifetime.

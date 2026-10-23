# Continuation checkpoint, 2026-10-23

## Paused at the user's request

Implementation is paused. The user requested a progress report and a local Git
commit on2026-10-23. All three workers stopped with no running jobs or known
partial writes. Resume only after the user asks to continue. The roadmap is
incomplete; the accepted foundation work below is not a playable identical game.

The last full workspace run remains V16:586 successful executions,574 primary
unit tests and two deliberate installed-corpus ignores. Later focused/corpus
checks are recorded separately; no newer full-workspace result is claimed.

Since V16, these gates passed:

- Cell full-channel curves:84,544 original-byte cases, including50,367 source
  schedules across all818 clips/2,086 data1..4 components and44,127 source rows.
  Rust agrees on84,256 modeled replies and positively rejects288 reached
  nonfinite inputs. All32 execution pins remained unchanged;23 protocol checks
  pass. See `target/source-cell/runtime-curves-native-v1/rust-comparison-v1.json`.
- Cell data2 movement multiplier:37,669 native cases, including16,584 authored
  source rows,17 protocol checks and28 unchanged pins. Rust agrees on33,117
  modeled replies and positively checks4,552 finite-API errors. Three focused
  tests and the independent raw-reply/source-integer review pass. Actual actor,
  handle-map and clip association remain explicit unresolved inputs. See
  `target/source-cell/runtime-modifiers-fixture-v1/rust-comparison-v1.json`.
- Font selector1 quotient scaling:379,589 source-instruction/reference/Rust
  records agree, SHA256
  `cfb9b1846f2ab23ce84ae6528b4aaaddcb442a69041f490bf190c9d05fd13eb1`.
  Five focused tests pass. Together with selector0, ordinary15ppem scaling now
  covers all37,691 simple glyphs/3,039,380 points in allfive installed faces.
  Composite flattening, hinting, rasterization and complete screens remain open.
- Body finalizer fields:all4,590 prepared projected replies match original
  bytes, independent Python and Rust, SHA256
  `e09a5b85cc07bfbe51cd97179b82f2b4630ac746f7af3190a4857d3e7b62367b`.
  Native34 protocol checks and52 unchanged pins pass; seven focused Rust tests
  pass. Root fixed three redundant trailing `?` return-type errors before the
  passing build; the failed build log and original preparation pins remain.
  Current Rust source SHA256 is
  `08cbfd1cdadcff12652ac23c8bb6a2fea278c69a05a68bd3c4c0e33b9c43c739`.
  Both examples are promoted/exported. Fresh all-source Rust reconstruction and
  its formal combined comparison remain pending. Do not call that final gate
  accepted yet. Native preparation retains3,966 admitted and129 rejected
  resource/context records, covering all82 indexes/1,365 source creations.
- FPGM setup:root reproduced the source-instruction oracle exactly:66,626
  records,137 definitions and159 top-level dispatches across allfive faces,
  SHA256 `2d261c579c37dcc2972eeb7474531ee88b44ff4e50d067363570b077ed7b3f2b`.
  `spui_font_hint_program` is exported and compiled through the finalizer
  example build. Its five focused tests and Rust corpus comparison are pending.
  This oracle interprets decoded instructions; it is not a new native run.

Saved unfinished work and precise resumption order:

1. Complete the finalizer fresh-source gate without rerunning the passing host:

   ```powershell
   tools\build.bat run -p spore-bevy --example anim_body_finalizer_source_verify -- "D:/Games/SPORE Collection" target/animation/body-finalizer-source-rust-v1.txt
   python tools/anim_body_finalizer_compare.py --input-dir target/animation/body-finalizer-fields-inputs-v2 --native-run target/animation/body-finalizer-fields-native-v3-run --rust-output target/animation/body-finalizer-fields-rust-v1.bin --rust-source-digest target/animation/body-finalizer-source-rust-v1.txt --out target/animation/body-finalizer-fields-native-v3-run/comparison-v1.json
   ```

2. Run five focused FPGM tests and `spui_font_hint_program_verify` against the
   Graphics package, comparing to the already reproduced
   `target/source-ui/font-hint-program-root-reproduction-v1.tsv`.
3. Review frozen `target/source-ui/font-hint-control-reference-packet-v1.json`.
   CALL/LOOPCALL core is unexported and its verifier remains `.rs.pending`.
   Reproduce its oracle before promoting that verifier, as the oracle pins the
   pending filename.65,699 prepared reference rows are not Rust acceptance.
   Actual FPGM bodies/PREP/glyph hint/raster remain unimplemented.
4. Review the unexported `cell_runtime_contributions.rs` and
   `tools/cell_runtime_contributions_native_probe.py` draft. Preparation under
   `target/source-cell/native-runtime-contributions-host-v1/` is static only.
   Fix the draft linker option `/OUThost-x86.exe` to `/OUT:host-x86.exe` before
   compilation. There is no linked audit, native execution, cohort or verifier.
5. Continue ordinary-root binding constructor9A0E30. Saved evidence is under
   `target/animation/root-selector-binding-source-prep-v1/`. Root selector mode1
   chooses0 and an emitted ordinary record retains byte0C=0, conditional on
   descriptor enable/slot/claim gates. All2,086 source data1..4 components use
   this source mode, but builder pruning and live association remain unresolved.
   Mirror helper evidence distinguishes2,071 explicit-mirror returns and15
   zero returns. No constructor implementation has started.
6. Refresh the full workspace suite after these gates. Then continue actual
   body/pose binding, source Cell controller integration and original UI text;
   do not repeat unchanged static rendering as a gameplay acceptance check.

REA last exact queries are875..878. The individual complete evidence is retained
under `target/rea/20261022-source-runtime-v3/`; the latest exported bundle is
through873 and the index is V11. The own REA session is being closed for this
pause; do not assume PTY47347 is still available after resuming. Retain all
ignored target evidence. Local legacy texture/capture PNGs under
`re/data/skins/` are now explicitly ignored and preserved without redistribution.

## Acceptance target and working boundaries

The requested goal is the entire game roadmap, including five gameplay stages and
editors, at original SPORE fidelity in Rust using the user's installed assets.
The goal is paused as recorded above. Parser coverage, controlled shader draws and prototype
simulations do not close stage/editor acceptance gates. See
`functional-milestones.md` for the complete open register.

Use executable/package analysis, shell tools and offscreen GPU rendering. The
user asked to avoid desktop automation. Do not send desktop input or open visible
render windows. The RTX 3060 Laptop GPU is available and has executed both
Vulkan/Bevy and hidden offscreen D3D9 checks. Original game files under
`D:\Games\SPORE Collection` remain read-only. Payloads, original bytecode,
disassembly, decompilation and captures stay under ignored `target/`.

Build only through `tools\build.bat`; serialize heavy builds on this machine.
Preserve existing uncommitted work. Do not copy GPL implementation code from
modding tools. Recovered interface facts and independently written code are the
implementation basis. The REA skill's broader-investigation workflow explicitly
directs parallel workers for independent questions. Root owns the live SDK
session, all Cargo builds, GPU checks and UI renderer code. Current workers
trace native animation/body binding, UI procedure/effect behavior and Cell
behavior; see the active chat's agent inventory before assigning overlapping
work. Workers must not start separate heavy Ghidra sessions or write root's
REA PTY.

## Latest measured integration

- `[MEASURED]` The latest full workspace all-targets run passed **586 test
  executions**: 505 core, 44 binary, 25 archive and twelve repeated asset tests
  compiled through two example targets. There are 574 primary unit tests and two
  deliberate installed-corpus ignores. It
  includes native unpack selection, mip framing, identity SH float rounding
  and the main application's default native-renderer integration, plus local
  UI anchors, image sizing/clipping, Image regions, packed UI geometry,
  native ANIM descriptor preparation and source Cell definitions/branches.
  It also covers ordered SimpleLayout event assertions, native foreground/fade
  quantization, the complete CreatureData header, bounded runtime ordering,
  Cell globals/axial targeting and linear UI fades. Complete output is in
  `target/workspace-tests-20261023-native-cohort-v16.log`. The first header-wide
  run exposed five old 68-byte animation test fixtures; those fixtures were
  corrected without weakening assertions before this passing run.
  This run includes the canonical viewer/source-catalog migration, complete
  bounded ROT sampler, inherited Cell classes, scalar cache and explicit scoped
  SSE state, source body capabilities, locale grammar, movement getters,
  movement size scaling, bounded body root selection, source extent/origin/anchor
  fields, optional foot construction, Cell runtime aggregates, ordinary source
  glyph packets, simple phantom coordinates and bounded selector-0 scaling.
  The installed Cell asset test was additionally invoked and passed.
- `[MEASURED]` Optional foot construction now matches all1,212 original-byte,
  independent Python and Rust projected replies:636 source call contexts and
  576 synthetic contexts across12 explicit x87 policies, with36 protocol checks.
  Fresh all82-index intake covers1,365 resources/46,418 rows under three contexts
  and both initial flags:8,190 exact resource digests, zero exclusions. Root
  independently reproduced the assembly, code and data; only decoded COFF/debug
  timestamps differ. The peer independently closed the316-byte source stack
  bound. All42 execution pins stayed unchanged. Accepted comparison is
  `target/animation/body-optional-native-v2-run/comparison-v2.json`. This closes
  projected construction, not all constructor padding/debug text, allocator
  behavior, final local orientations, finalized hierarchy or live animation.
- `[MEASURED]` Complete Cell aggregate reader/scalar/pair writers now pass129,331
  original-byte/Rust cases. All125,319 finite/API-modeled172-byte replies agree;
  3,840 reached nonfinite inputs and172 nonfinite final commits are positively
  checked typed errors with their native outputs retained. All44,127 authored
  value/weight rows contribute88,254 explicit blend-one source cases;41,077
  synthetic cases cover12 x87 policies and raw writer NaNs. Eighteen protocol
  checks pass; all25 execution pins stay unchanged and every Rust call preserves
  three incoming MXCSR states. The comparison is
  `target/source-cell/runtime-data-fixture-v1/rust-comparison-v1.json`.
  Full-channel curve sampling subsequently passed as recorded in the pause
  handback above. Actual blend policy and actor construction remain separate
  gates; earlier16-row preparations remain unexecuted.
- `[MEASURED]` Simple font preparation now independently agrees on37,691
  phantom-point sets across allfive installed faces. Selector-0 scaling at15ppem
  agrees on37,254 glyphs/3,007,496 real-plus-phantom points across the three2048-UPE
  faces. The source-instruction oracle evaluates all65,536 signed coordinates
  and root reproduces its27,347,887-byte output; Rust matches exactly. The other
  437 simple glyphs subsequently passed selector1 as recorded above;13,939
  composites and50 empty glyphs remain explicit exclusions. See
  `target/source-ui/font-scaling-rust-comparison-v1.json`. This is decoded integer
  instruction/Rust agreement, not new machine-code execution or hinted pixels.
- `[MEASURED]` All 14,355 compiled source shader programs in 12 packs resolve
  through native zero-terminated recipe keys; independent Python checks agree
  on lookup and uniform dependency masks.
- `[MEASURED]` Native declaration features agree independently for 13,269
  declarations / 51,437 elements in all 11,041 GMDLs. This is the saved
  declaration, not proof of the final runtime declaration.
- `[MEASURED]` Shader-data `210` parameters agree independently as float bits
  for 17,730 records. The callback tests marker `5C 44 26` before reading W;
  legacy records have W = 1.0. Source `20D` texture bindings agree on 42,446
  records, including conditional sampler-state updates.
- `[MEASURED]` PROP RGB/RGBA decoding now preserves LE float components rather
  than taking the first bytes. Independent agreement covers 28,161 colors /
  85,210 components in 30,885 certified named lists. Signed/HDR values and
  nonfinite bit patterns are preserved; four original framing exclusions remain.
  The earlier 847,835-value primitive corpus still passes after this correction.
- `[MEASURED]` Original `UTFWin_UIShader` bytecode executes on the RTX through
  the optional `native_shader_probe` feature. Direct-package draws of a
  512 x 256 atlas and a 128 x 128 source thumbnail have zero RGBA error.
  Both IDs 1 and 2 now also pass copy/source-alpha blend fixtures with the
  actual 24-byte vertex declaration, sixteen objects, native pixel viewport
  and separately composed local transforms. All four 131,072-pixel outputs
  have zero RGBA error. See `target/source-ui/native-batch-v1/`. Separate
  alpha and sRGB are explicitly controlled, not inferred retail state.
  This proves controlled UI drawing, not an entire original screen.
- `[MEASURED]` Standalone RASTER levels are length-prefixed individually.
  Rust/Python agree on all 5,931 resources / 56,936 levels / 2,804,636,276
  pixel/block bytes across all 82 indexes. There is no unexplained 36-byte
  footer in the ten-level example; these are the nine later size words.
- `[MEASURED]` All 28 resolved primary Cell models execute original 3D programs
  on the RTX with untouched GMDL buffers, native unpacking, source material
  parameters and three source texture units, including complete mip chains.
  `target/source-cell/native-corpus-v1/report.json` retains each result and
  the tested executable/inventory digests. The 65 resolved definition rows
  refer to these 28 models; the full definition population is 223.
- `[MEASURED]` Main `--source-cell --shot` now selects this original-program
  renderer by default. Fresh Jawhead/Stabber main captures exactly match
  the corpus images (zero RGBA error). `--pbr-inspection` explicitly selects
  the prior Bevy diagnostic view; its new capture matches the previous stable
  PBR image exactly. No visible window or desktop input is used.
- `[MEASURED]` All 15,489 serialized Image regions independently agree on
  resource reference, flag word, dimensions, source UV and mapped UV fixture.
  Every reference selects a container image/atlas; all Image flags are zero.
  Preserve the 1,347 UV rectangles outside 0..1. Local SimpleLayout event
  order is authored order, with a clamp after each layout and floor once
  after dispatch. Title Window 7's actual local right edge is 805 under the
  explicit 800-wide fixture; 799 from deferred clamping is incorrect.
- `[MEASURED]` Native ANIM component preparation agrees independently for
  all 5,022 clips / 48,700 channels / 301,684 descriptors, including 43,772
  deformation bindings. The native initial fallback is proven empty through
  `A0F204 -> 9A9730`, not assumed from missing entries. The fresh
  `--native-initial` Rust digest agrees with the independent Python records.
  Original curves, complete generated-body construction and binding remain open.
- `[MEASURED]` Source Cell metadata and Rust agree on all 489 resource copies
  / 86,064 typed leaves across 82 indexes. Independent branch fixtures agree
  on 2,445 AI selections, 4,890 gates and 34,230 numeric eating cases. These
  are bounded native consumers, not complete movement/progression behavior.
  Main Cell loading now uses the typed native schema and 80-unit label bound;
  its fresh original-program Jawhead capture is unchanged at zero RGBA error.
- `[MEASURED]` REA Frida instrumentation works on our own hidden renderer:
  shader constants, state and two original-program draws are observed, no
  visible window/Present occurs, cleanup is clean and output is unchanged.
  No retail executable was started by this validation. Evidence is
  `target/source-cell/frida-own-renderer-validation-v1.json`.
- `[MEASURED]` Native version-9/10 CreatureData retains a full 128-byte header,
  with correct cross-chunk repeat state, malformed-RLE failures and source
  array limits. All 1,365 source resources / 174,720 header bytes independently
  agree. The extra 81,900 bytes are zero in this install, and all prior
  1,365 rig digests remain unchanged. Native loading copies all source rows;
  it does not establish a class-one body filter. See
  `re/creature-header-native.md` and the pending generated-body work.
- `[MEASURED]` Runtime block ordering agrees on 21,109 explicit synthetic
  hierarchies / 105,095 blocks. Native palette assembly agrees on 18,288 cases
  / 600,440 matrices, including the actual first-group generated-prefix clamp.
  Inputs are explicit runtime groups/poses, not inferred source-row indices.
  See `re/anim-block-order.md` and `re/anim-palette-native.md`.
- `[MEASURED]` Current Cell globals pass six independent records: five supported
  records include two real 276-byte copies, while the older 264-byte base copy
  is explicitly excluded. Axial targeting matches 4,119 CPU golden cases;
  flow sampling matches 2,220 and movement-delta application matches 4,126.
  These are bounded native operations under supplied state and MXCSR1F80,
  not the complete swimming integrator. Application fixture v2/v3 are invalid:
  the comparison caught wrong producer count/environment offsets. Retain them
  as failed evidence; only application-fixture-v4 passed the unchanged Rust
  arithmetic. See `re/cell-behavior.md`.
- `[MEASURED]` Explicit x87 arithmetic agrees with 2,880 independent exact
  rational/integer reference cases across three precisions and four rounding
  directions. Retained intermediates, binary32 commits and comparisons agree.
  The original process control word remains unknown; see `re/native-x87.md`.
- `[MEASURED]` Five linear FadeEffect tests cover 110 independent ticks and
  ten sequences / 156 frames. All 1,161 source records independently agree:
  1,099 supported, 61 unproven interpolation modes and one other trigger.
  Actual TitleScreen child eligibility and the separate SporeTitleSeq state
  graph are retained in `re/spui-title-native.md`. Text/font drawing, selected
  retail overlay, external playback and complete screen execution remain open.

The 514-test cohort includes palette, flow, movement-application, swimming,
x87, bounded ANIM position curves, source-font metadata, DXT3 glyph-alpha
packing and the authored creature asset/CLI paths.

- `[MEASURED]` Canonical authored creature links now use actual PROP LOD keys:
  704 direct source creatures, 3,770 available LOD references, and 661 source
  creations without local PROP. All 704 first-group count constraints agree
  freshly on 21,753 rows / 17,752 body-prefix rows. This count relation alone
  does not prove runtime palette identities. Three complete mesh/map chains
  independently agree, including a same-instance GMDL missed by namespace
  substitution. See `re/creature-model-links-native.md`.
- `[MEASURED]` Main `--source-creature --creature 40626200067C5390 --shot ...`
  follows the authored LOD0 and exact model ResourceKey into original programs.
  Its 6,967-vertex / 11,070-triangle RTX capture exactly matches the standalone
  draw. Source texture mip chains and original vertex bytes are retained.
  Uniform219's zero vector is established only for a fresh world and the
  measured plain preset, with absent optional overrides. Reused lighting state
  is not supported by that shortcut. The Bevy `--shot ... creature=...` path is
  separately labeled material inspection. Retail scene/animation remains open.
- `[MEASURED]` Bounded position curves agree bit for bit on all 32,256 original
  x86 INFO/POS cases across twelve explicit x87 contexts; 9,024 native endpoint
  copies separately preserve the full 100-byte INFO/POS row. The host keeps
  quaternion/deform branches unreachable. A copied memcpy stub's outbound
  relative jump was corrected and machine-audited before any execution.
  `re/anim-position-native.md` documents the exact scope and reproduction.
  The all-source comparison now passes all692,878 finite samples; four authored
  NaN endpoint copies reject explicitly. The census covers39,633 matched POS
  channels/731,075 rows across all5,022 clips. There are238 empty channels,
  8,829 absent POS channels and116,334 disabled interpolation samples across
  7,466 channels. These unsupported interpolation branches are not approximated.
  `target/animation/source-position-fixture-v1/rust-comparison.json` retains
  exact bit/neighbor agreement, independent original-source selection and the
  separately checked four nonfinite rejections.
- `[MEASURED]` Cell turn-in-place agrees on 12,924 original-byte cases;
  flagella/cilia and jet propulsion agree on 45,432 cases. These supplied-state
  kernels do not establish the entire actor update or retail control word.
- `[MEASURED]` All five source fonts agree independently on 51,680 glyph
  metrics/bounds and 40,246 format4 mappings (91,967 sorted records). DXT3 page
  clear and byte-alpha packing agree on 5,201 synthetic cases, including all
  alpha values, tails, pitch and preservation checks. Native text rasterization,
  hinting, complete layout and screen events remain open. See
  `re/spui-font-native.md` and `re/spui-title-native.md`.
- `[MEASURED]` Rest/live body-palette math agrees on297 independent cases and
  13,134 matrices per phase. Canonical source intake agrees for all1,365
  CreatureData creation resources/46,418 rows under three explicit precision
  settings, totaling4,095 resource-context records/139,254 evaluated rows.
  Imperfect matrices remain literal; no normalization or tolerance filter is
  inserted. This population includes flora. Finalized hierarchy and actual
  animated binding remain open. See `re/anim-rig-native.md`.
- `[MEASURED]` Cell capability curves agree on107,208 original-byte cases,
  including94,440 independently decoded source-property cases over both
  original PROP copies,18 unique curves/104 points and12 explicit x87 contexts.
  Provider/guard/protocol checks and108 missing/empty defaults pass. See
  `re/cell-capability-curves.md`; the cache and actor integration remain open.
- `[MEASURED]` Installed CRT finite SSE2 acos arithmetic agrees on62,624 cases
  over all six branches. This is an explicit MXCSR1F80 policy, not recovered
  retail dispatch. Later original initialization writes PC24/nearest and
  FTZ/conditional DAZ; actor-entry state remains unresolved. See
  `re/crt-ciacos-native.md` and `re/native-float-render-context.md`.
- `[MEASURED]` Rust vertex HLSL generation matches954 unique original cached
  profile3 recipes in executable instruction tokens and all uniform/register
  metadata. An additional uncached original flora recipe compiles and draws
  on RTX. Actual HLSL/templates remain installed or ignored external inputs.
  The Creature capture now follows the correct original editor thumbnail
  camera link. See `re/shader-codegen-native.md` for commands, hashes and limits.
- `[MEASURED]` All 704 creations with explicit authored model links now pass
  hidden RTX captures, including expansion models, using original geometry,
  maps, mip chains and programs. The corpus totals 7,804,809 triangles and
  exercises two distinct vertex and four pixel programs. One uncached vertex
  recipe compiles on demand in 346 processes. Results, executable/inventory
  digests and per-creation logs are retained in
  `target/source-creatures/native-corpus-v2/report.json`. The earlier v1 run
  passed 361 and failed 343 because its reader searched only four base
  packages; it remains a failed baseline. The all-82-index reader preserves
  an explicit inspection order, not an asserted retail package priority.
  Jawhead's fresh capture remains byte-identical. These are controlled static
  art checks; retail scenes, animation and gameplay remain open.
- `[MEASURED]` Native quaternion primitives agree on 85,932 finite cases;
  another 192 tiny normalization cases return the explicit nonfinite-output
  error matching the native boundary. The 86,124-row v2 fixture covers twelve
  supplied x87 contexts, SSE2/MXCSR1F80 and guarded output/context state.
  `target/animation/quaternion-native-v2-run/comparison.json` retains the
  accepted comparison. The independent rational oracle covers normalization,
  hemisphere addition and the near-dot branch; hardware FSIN is separately
  compared, not labeled an exact transcendental oracle. Wider rotation-curve
  extrapolation and the whole ROT branch require their own native acceptance.
- `[MEASURED]` Cell part geometry agrees on all 145,356 native cases, including
  136,992 source-derived evaluations and 8,364 synthetic cases across twelve
  explicit x87 contexts. The native and Rust 23,256,960-byte outputs match
  exactly. First-64 clamping, special-center globals, opaque NaN copies and
  preserved padding are checked separately. See
  `target/source-cell/part-geometry-fixture-v1/rust-comparison.json` and
  `re/cell-capability-cache.md`. Actual inherited class resolution, the later
  capability cache and actor integration remain open.
- `[MEASURED]` The complete Cell cache geometry/count/index prefix agrees on
  28,398 native cases under twelve explicit x87 contexts, including 19,110
  source-derived cases with explicitly supplied classes. The full 4,089,312-byte
  native/Rust stream matches; five separate nonfinite-policy cases reject
  explicitly. Later capability fields and unused index slots are preserved.
  `target/source-cell/capability-geometry-fixture-v2/rust-comparison.json`
  retains evidence. The full capability/actor update is still open.
- `[MEASURED]` Quaternion extrapolation now also agrees on all 86,520 native
  cases over twelve supplied contexts and fraction range[-1,2]. Full ROT100
  curve evaluation has four focused Rust tests passing but awaits its separate
  reviewed native-host comparison. It is not accepted from the primitives alone.
- `[MEASURED]` Canonical native archive decoding now agrees independently on
  all 82 packages / 158,720 records, including correct common-header fields
  and same-record compression metadata. All 101,312 coded and 57,408 raw
  payloads read to their declared sizes, totaling 10,447,076,586 decoded bytes.
  The original-program renderer uses this typed96-bit key path. Four hidden
  RTX regressions remain PNG-byte-identical. Five focused archive tests pass;
  this and the newly exported ROT module need the next full workspace cohort.
  Historical catalog/shape APIs remain compatibility views; see
  `re/archive-index-native.md` and `target/native-archive-index-census-v1/`.

Relevant reproducible evidence and code are in `re/shader-selection.md`,
`re/shader-execution.md`, `re/material-links.md`, `re/cell-lighting.md`,
`re/prop-numeric.md` and `crates/spore-bevy/examples/source_cell_shader_audit.rs`.

## Original 3D shader implementation and remaining gates

`[MEASURED]` An explicitly controlled static context for Jawhead and Stabber
selects the source `generic_static` builder (`80000002`), profile 3, render type
0, with the saved declaration and material data, source lighting `219` present
and CellStage `236` absent. Native MaterialManager initialization creates that
fallback builder. Native `760FB0` / `73EDF0` also set marker `22C` from the
first semantic-2 row being UBYTE4; both measured unpack enables are one.
Later retail overrides remain unestablished.

Both meshes resolve original programs with recipes:

- VS `14 17 19 15 31 30 29 01`, 3,196 bytes, five uniform descriptors.
- PS `05 09 0C 11 3B 19 2A`, 3,580 bytes, five uniform descriptors.

`[MEASURED]` SDK reflection of these exact cached bytes identifies modelToWorld,
modelToClip, materialColor, worldCameraDirection, SH coefficient prefixes,
materialParams, dirLightsWorld and tintParams. The pixel program consumes units
0 and 1. The source records also name a third unit, whose global texture
`552A5077` resolves as a real 64 x 64 DXT5 RW4; this permutation does not sample it.

`[MEASURED]` The selected original unpack fragment decodes the saved UBYTE4
directions itself. No CPU vertex conversion is needed in this path. The earlier
2,964-byte VS was selected without `22C` and omitted that required operation.
The similarly named standard `CellSkinDrawShaderSH` still has a different
vertex interface and is not a valid name-based substitute. The
source identity-frame SH prefix now retains float32 undo/accumulate/redo and
exposure ordering, with independent Decimal-100 synthetic golden values.
Direct-light upload accounts for two native direction negations.

`[UNKNOWN]` Complete retail camera anchor/timing, material overrides, culling/
blending, frame rotation, animation, effects and UI are not completed by these
controlled draws. Native D3D9 rendering is enabled by the default Windows
`native_renderer` feature. The UI conformance example remains separately gated
by `native_shader_probe`; `--no-default-features` omits native main rendering.

The source normal/paint lighting keys are `40200100FCED73A2` and
`40200100A9CA81EB`. EditorSetup names both through native numeric properties;
each has 25 float RGB coefficients. Do not put their proprietary arrays in
source code. Native `70A820`, `783780`, `7834E0`, `708890`, `77E190` establish
the load, normalization, rotation and upload route; remaining transforms are
documented in `re/cell-lighting.md`.

## Persistent native analysis session

REA 6.5.0 / Ghidra 12.1.4 is available through the existing SDK helper
`target/rea/source-runtime-session-v3.mjs`. Do not restart expensive analysis
or repeat MCP setup. The focused profile disables Stack, x86 Constant Reference
Analyzer and Decompiler Switch Analysis; their missing results remain unknown.

- Source executable SHA256:
  `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
- Focused profile SHA256:
  `07956626c4c75f6e693f1731c6ed414b61cf009d055c0d72055d224a32d58a49`.
- Evidence/query files: `target/rea/20261022-source-runtime-v3/`.
- Latest exported bundle at this checkpoint: `evidence-through-query873.json`
  (272,200,851 bytes, 750 Evidence records); query874 confirms export.
- `functions-index-v11.json` audits 690 analyze-function operation records
  through query873: 568 exact entries (563 unique), ten containing-body
  results and 112 explicit errors. This is not a complete function/CFG census.
  Batch decompilation has a separate per-item address audit under
  `target/source-ui/font-ghidra-batch-v1/`; status-ok results that identify an
  unrelated fallback procedure are quarantined. Use `tools/rea_dossier_index.py`
  for a reproducible refresh. Later queries continue; preserve the live session.
- Query 118 records built-in uniform callback registration; queries 124 onward
  extend those callbacks. Undefined functions require bounded SDK disassembly,
  not an unrelated decompiler fallback.

Important corrected address arithmetic: `16F7DDC` is shader-data `20D`, while
`201` is at `16F7DAC`. No `20D` to `201` pointer remapping was established.

## Immediate continuation

Fresh native gates completed since the earlier geometry/primitive checkpoint:

- `[MEASURED]` The complete bounded INFO+ROT100 branch matches all 30,240
  baseline contexts with zero arithmetic exclusions, plus 3,456 fresh finite
  overflow/unordered-dot cases. Both native blend branches, all sixteen mode
  pairs and twelve explicit CW policies are covered. The source-compatible dot
  branches preserve native COMISS predicates; bounded primitive APIs retain
  their previous typed errors. Evidence is in
  `target/animation/rotation-native-v2-run/comparison-source-v2.json` and
  `target/animation/rotation-source-dot-native-v1-run/comparison-v1.json`.
  The all-source ROT census covers 5,022 clips / 731,383 rows, not an all-source
  native sample execution claim. Generated rig binding and deformation remain open.
- `[MEASURED]` Inherited Cell classification matches all 75,938 native cases,
  including 23,306 candidate source chains. Rust reparses all 1,411 original
  PROP copies / 719 keys and proves the same classes for all 654 part identities.
  `target/source-cell/part-classification-fixture-v1/rust-comparison-v1.json`
  records exact native/Rust output SHA256
  `c47eeda476dd9a90e3e0c368c4c6917a2479ddd2ebb5603ef8a67567d7283854`.
  Actual-class geometry/cache composition and the bounded scalar tail now also
  pass the fresh gates below; actor/controller integration remains open.
- `[MEASURED]` AssetSource, PackageSet and SourceCellLibrary now use canonical
  native indexes and typed keys. All 158,720 catalog keys in 82 packages,
  27,295 mesh selections and 20,625 texture selections agree with the independent
  canonical census. The catalog SHA256 is
  `9a0f3d5a3c04f1d7de92e5e9adb20607424d324d823ea729023480ac244b82d2`.
  Main Cell/base/expansion/flora RTX captures remain PNG-byte-identical across
  2,359,296 pixels; proof is
  `target/native-archive-index-census-v1/render-regression-v2/comparison.json`.
  The inspection loader now reaches the strict v8/v9 tail reader for expansion
  models instead of rejecting the legacy v8 skipped-tail flag. All 704 linked
  saved mesh/map chains pass through the actual CPU asset loader, with zero
  failures (`complete-authored-intake-v1/report.json` in the same index census).
  Older CLI/corpus compatibility APIs remain separately
  documented; retail package ordering and complete scenes are still open.
- `[MEASURED]` Typed source PROP classification, source CreatureData intake,
  part geometry and native geometry/cache prefix compose exactly for all
  1,365 creations / 45,664 native part rows. All 46,418 raw rows are accounted
  for; the original first64 clamp omits 754 rows in 73 creations. Native/Rust
  geometry SHA256 is
  `dd1a4f372aa578f4f1d6261a333acc77fb63dcf7de9d8fa6fc1b508b7adbfc86`;
  cache prefix SHA256 is
  `a2a9964d1a5532970ed4a68106e472ac53329e3afdb57c51b9e37f4d9849e549`.
  See `target/source-cell/capability-source-composition-v1/rust-comparison-v1.json`.
- `[MEASURED]` The scalar cache tail matches all 33,480 finite native cases,
  including 5,460 real count/Globals/curve-copy contexts. Twelve explicit CWs
  and three incoming MXCSR states per Rust case pass, with full caller control
  and sticky-bit restoration. Every modeled232-byte reply agrees, SHA256
  `e246a6ecd62c053b19fa9466aea4f6bffaac2a67228f492690a1b77dc4c900c9`;
  eight native FP metadata bytes per case are checked separately. Eight further
  checked-policy errors are retained positively. The two 276-byte Globals copies
  and both curve PROP copies are reparsed independently; the older264-byte schema
  stays explicitly excluded. See `target/source-cell/capability-scalar-fixture-v1/`.
  Source-style selection and base-speed getters now also pass below; actual
  actor/controller wiring and selected AI steering remain open.
- `[MEASURED]` Body capability expansion, duplicate maximum/compaction and both
  presence/nonzero masks match all58,802 native/Python/Rust projections, including
  every46,418 source row in1,365 resources. Each full168-byte projection agrees,
  SHA256 `582aca8c6e189c9d8385749d68798bb03efdd9bb8da0446f7ac3c01032c6b337`.
  Fresh Rust intake reparses all82 indexes and the three identical source PCTP
  aggregate tables. Evidence is in
  `target/animation/body-capabilities-native-v1-run/comparison-v1.json`.
  Bounded root selection now passes below. Optional foot construction and the
  finalized hierarchy remain open.
- `[MEASURED]` Original body root selection matches all8,624 native/Python/Rust
  projections, including4,976 source contexts from1,244 resources and3,648
  synthetic contexts. Each2056-byte projection agrees, SHA256
  `0388646b646ea94479966b037aa04ff2e3ef6efcf8ff4b944fcf203c2dfd2cef`.
  Twenty-five protocol checks pass. Fresh Rust intake reparses all82 indexes,
  1,365 resources and46,418 rows. The106 resources requiring the optional-foot
  driver and15 other initial nonspine-root resources remain explicit separate
  gates. All1,092 nonadmitted candidate outcomes remain in the comparison.
  This establishes root policies and ordering/reversal projection, not anchors,
  generated local orientations, relabeling or runtime pose binding. Evidence:
  `target/animation/body-root-native-v1-run-v2/comparison-v1.json`. The first
  failed allocation run and every bounded allocation-only retry are retained.
- `[MEASURED]` The source-pinned119-instruction extent/origin tail and strict
  rest head independently agree with fresh Rust intake for all1,365 resources,
  46,418 rows and three explicit precision contexts:139,254 evaluated rows /
  4,095 resource digests. SHA256 is
  `fe9cde4af3ec66ad2a94ae39addccc7ef7abdc40ffdf2404f55da1b25d3f5138`.
  Four focused tests include positive-subnormal clamp behavior and exact
  hostile caller-state restoration. This tail gate uses a bounded decoded
  instruction oracle, not a new original-byte host. Its half-extents, local
  origins and effects origins do not establish anchors or optional feet. See
  `target/animation/body-source-fields-rust-comparison-v1.json` and
  `re/anim-body-source-fields-native.md`.
- `[MEASURED]` Canonical source anchors, parent deltas and retained x87 length
  independently agree for the same4,095 contexts /139,254 evaluated rows,
  SHA256 `af519227ecee9e6822f927d64d39e23428b120e7f5eadef86dea2262150ceb8f`.
  Each precision context retains35,014 ordinary translation rows,10,039 class5
  midpoint rows and1,365 parentless constructor defaults. Four focused tests
  pass. Root freshly reproduced the decoded-instruction oracle and the Rust
  intake; no row is excluded. This is the pre-optional source anchor boundary,
  not a new original-byte host or final local-orientation/pose binding gate.
  See `target/animation/body-source-anchors-rust-comparison-v1.json` and
  `re/anim-body-source-anchors-native.md`. Failed oracle preparations v1/v2 are
  retained and are not source conformance evidence.
- `[MEASURED]` Full source movement getters E52770/E67CB0/E67D20 match68,340
  finite cases, including all489 source definitions and5,460 real scalar-cache
  replies with an explicitly zero-style definition. The32 modeled bytes agree,
  SHA256 `15780218eb69b5015e61fcd64d9b6c4f641c062e17b95e1ada9a1984af40f0c6`;
  eight native FP metadata bytes are validated separately. All12 CWs and three
  hostile incoming MXCSR states pass. A separate192-case policy lane retains96
  reached nonfinite errors and96 unused-cache default successes. Ten protocol
  checks pass, source/host/input pins remain unchanged. Source and cache lanes
  are explicit inputs, not claimed active creation/definition bindings. See
  `target/source-cell/movement-source-fixture-v1/rust-comparison-v1.json`.
- `[MEASURED]` Original movement size scaling matches261,960 finite-operand
  cases and3,864 separate nonfinite-table cases under12 explicit CWs. All24
  modeled bytes agree, including all ten retained x87 bytes, final32-bit store,
  denominator index and raw DIVSS ratio. Finite SHA256 is
  `9119d20ef6ccdbf0a78ef805e612511526cbaa32f6f2858725aaa4dbfbd80bb5`;
  nonfinite-table SHA256 is
  `5fe49fa6a8d2af92b184054ef6063d78bfdc818cb0fc3627068f9be6e1b1695d`.
  All252,252 combinations of21 actor-index policies,1,001 progress values and
  12 CWs are included, plus60 ignored-progress defaults and9,648 synthetic
  cases. Eighteen protocol checks and three incoming MXCSR states pass with
  exact caller-state restoration. No arithmetic outcomes are excluded.
  Actor/progress/table words are explicit inputs; live actor creation and
  controller wiring remain open. See
  `target/source-cell/movement-scaling-fixture-v1/rust-comparison-v1.json`.
- `[MEASURED]` All4,497 locale tables in23 language packages independently agree
  on419,485 stored entries /27,408,306 converted source units. Native byte grammar,
  duplicate assignment and four-byte Unicode truncation are instruction-derived;
  the native parser itself has not been executed. All current tables are BMP-only.
  Corpus TSV SHA256 is
  `caf1aed378f249d517c98793e0d2dfa738f02a4f159a2f9e9825f8bcf54a0059`.
  See `re/locale-table-native.md` and `target/source-locale/corpus-v1/`.
  Runtime locale selection, detokenization, shaping and full UI text remain open.
- `[MEASURED]` The unscaled ordinary glyph-packet reader matches all51,680
  glyphs in all five installed faces, including2,888,616 simple points,
  28,483 composite descriptors and13,084,181 original instruction bytes.
  All64,705,863 canonical TSV bytes agree with the independent FontTools-backed
  packet oracle, SHA256
  `d3e152068cc845caba0a62bc0a43af65b9e7049916507c90d1d52d53156d2c8e`.
  Five focused tests pass. Both fresh intakes now use native type/scope/instance
  keys; older compression-metadata cache labels remain only filenames. The
  earlier v2 rows are retained and proved identical apart from canonical keys.
  This gate does not flatten components, synthesize phantom points, scale,
  execute hints or rasterize. See
  `target/source-ui/font-outline-rust-comparison-v1.json`.

The ordinary15ppem native font hosts are frozen at
`target/source-ui/font-glyph-native-v1/` and `font-glyph-native-v2/`. Both have
independent static reviews and root reproduction of preparation/compiled code,
apart from explicitly parsed linker timestamps. V2 substitutes one checked
four-byte dispatch pointer to an owned one-byte RET for the source EIF RET;
every other retained source/font/page byte is unchanged. Normal hidden runs
failed before source entry during mapping. The shared fatal branch does not
distinguish reserve failure from commit failure; occupation/owner is unproven.
All failures and unchanged input pins are retained. A debug-created hidden child
successfully completed every mapping/protection step, but continuations v5/v6
stopped before source entry because WOW64 context readback changes EFLAGS bit1.
V6 records every other GPR/segment/ESP unchanged. A separately reviewed owned
PUSHFD diagnostic proves actual flags remain202 while the API serializes200;
only this fixed-one reserved bit is canonicalized in the reviewed continuation.
V8 then completed glyph0 with7x7 original A8 pixels,13 native setjmp calls,
63 allocations/frees and zero live heap. An independent raw-reply audit agrees.

The broader original-font reference corpus now has80 completed native requests,
4,624 glyph observations and all578 glyphs, across four explicit heap policies
and fresh-versus-sequential contexts. All1,156 identical ordered-input groups
agree across heap policies. There are2,352 font contexts and7,154,800 guard
checks;57 proved pre-source mapping retries remain retained. Seven glyphs
(354,357,364,366,371,373,442) differ between fresh and ascending64-glyph chunk
contexts, consistently across all four heap policies. Their cause/retail reset
policy remains open. The initial executor's incorrect fresh-equals-sequence
assertion failed after every native case had completed; its report and exact
executor are preserved. Read-only re-audit verifies all replies, alpha exports
and185 current pinned files without rerunning the host. Evidence:
`target/source-ui/font-glyph-corpus-v1/native-context-audit-v1.json`; the1,156
semantic records have SHA256
`bad091f7c103d0b1a7bc7e86ef11d24124122871098013bc5b12f04e1c4156be`.
This is one original font at15ppem, mode3/flags8B, CW037F/MXCSR1F80 and explicit
debug startup. Native reference pixels do not establish Rust raster equivalence,
all font/style coverage or full UI screens. No retail process or desktop input
was used.

The eight-case order counterfactual `[354]` versus `[342,354]` reproduces the
gamma glyph's9x12 versus9x13 output under every heap policy. The only captured
CVT-array difference before gamma is14:-192 versus-256; storage and callback
functions agree. Raw E0/E1 is1 after PREP versus0 after the preceding glyph;
native setup explicitly clears it. The independent raw-wire/source/pin review
is `target/source-ui/font-cvt14-order-independent-audit-v1.json`. This proves
preceding-glyph sufficiency for this observed output, not that every possible
hidden state or all seven causes were isolated. A separate ignored Rust
Swash0.2.10/Skrifa0.44 diagnostic matches only six empty glyphs out of578 with
either hint setting. It is unadopted and changes no game backend/dependency.
Its failed comparisons are retained under `font-candidate-swash-v1/`.

The actual Cell runtime aggregate is now traced from authored ANIM kind3 root
components through prepared slots to pose+A0 records and the movement reader.
All2,086 data1..4 components across818 clips use literal primary-root mode;
extra selector flags remain retained rather than assuming the bound pose index.
The44,127 original value/weight rows are extracted with source offsets0/4.
The first census's4/8 endpoint diagnostic mistake remains retained and is
superseded by fresh v2; all counts and other columns are unchanged. See
`target/source-cell/runtime-data-source-census-v2/` and
`runtime-data-source-curves-v1/`. Complete source reader and scalar/pair writers
have a reviewed guarded host. Root reproduced/compiled/audited it separately at
`native-runtime-data-host-root-reproduction-v1/`. Its full129,331-case native/Rust
gate now passes, including4,012 positive finite-API errors. Full160-row kind3
sampling also passes84,544 native cases with288 positive Rust input errors.
The data2 movement consumer passes37,669 native cases with4,552 positive Rust
errors. Live actor/controller binding, actual clip association and blend policy
remain unresolved. These arithmetic gates do not establish gameplay integration.

1. Establish source camera framing, frame-dependent lighting, original material
   state and a retail image comparison before counting renderer fidelity.
2. Continue source UI class/layout/procedure reconstruction. Native v3 queries
   164 onward locate class metadata. SimpleLayout's local apply/inverse/clamp
   operations now pass 7,919 independent source records in all 748 SPUIs;
   all sixteen low-bit combinations occur. ImageDrawable sizing/defaults also
   agree for 6,299 instances under the explicit fixture. Local native clipping
   and its UV branches are implemented. Equal-priority SimpleLayout dispatch,
   packed drawing and bounded linear FadeEffect are now checked. Image
   effects/slicing, text/font drawing and complete UI rendering/events remain open.
3. Resume native creature binding/animation and source stage
   behaviours through the still-open functional milestone register.
   The initial ANIM fallback and full CreatureData header are now established.
   Bounded runtime ordering and model-palette gathering now pass independent
   fixtures. Generated hierarchy/rest poses, actual resource/pose binding and
   rotation/deformation curves remain open; do not infer source-to-palette IDs.
   Direct model PROP/LOD associations, all-source finite POS curves, bounded
   rest/live matrix math and canonical source intake now pass. Original-byte
   quaternion primitive comparisons and all-source part geometry now pass.
   The bounded complete ROT100 branch now passes; full-source runtime binding,
   deformation and the real generated hierarchy remain active. Preserve the source's extreme finite rotation rows rather
   than introducing a quality filter or implicit normalization.
4. The broader hidden authored-art corpus passes 704/704 after discovering
   all installed indexes. Recover actual package registration/lookup priority
   and active-product context before replacing the explicit inspection order.
   Continue toward animated creatures and actual scenes; do not repeat the
   unchanged static corpus unless a relevant renderer change requires it.
5. Source Cell controller dispatch and shared swimming functions are located.
   The native metadata has difficulty inheritance and eating numerics, plus
   a separate 276-byte current CellGame globals schema. The 264-byte base
   copy is a version difference, not silently compatible current data. Axial,
   sampled-field and delta-application kernels now pass CPU goldens. Swimming
   steering/integration, field population, progression and effects remain open.
6. Source Title initialization, procedure ordering and Fade callback/tick
   behavior are traced without running the retail executable. TitleScreen's
   ordinary children are initially ineligible; the separate SporeTitleSeq has
   actual timed visibility calls and a caption hierarchy. Native text/style
   and package precedence are being traced. Do not substitute a flat logo atlas
   or invent phase/clock inputs to claim a finished Title screen.

The full roadmap remains incomplete and is paused at the user's request.
Wait for the user to resume; do not turn this checkpoint or any controlled draw
into a completion claim.

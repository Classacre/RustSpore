# Full game implementation milestones

Scope confirmed by the user on 2026-10-22: complete the entire game roadmap, including
all five gameplay stages and editors. Work is paused at the user's request on2026-10-23.
The implementation goal remains incomplete until its
functional acceptance gates pass. A prototype or parser result does not complete a
retail-fidelity milestone.

The user explicitly clarified the acceptance target later on 2026-10-22: an identical
Rust remake of SPORE, using the installed game's assets. Original procedural art and
invented gameplay rules do not satisfy that target. Existing experimental simulations
are retained as engineering work only. Further implementation prioritizes recovered
source resource chains, native semantics, original UI layouts/materials and animation.
Desktop automation was stopped at the user's request. Continue through package and
executable analysis, offline comparisons and automated application captures.

## Completion rules

- Each milestone needs implementation, automated checks, real-data checks when it reads
  shipped content, and a running-application check for user-visible behaviour.
- Gameplay/editor completion also requires source content and behaviour fidelity.
  Procedural placeholders, palette approximations and original rule sets cannot close it.
- Keep exact resource/package denominators and a reproducible command or evidence path.
- Record unsupported formats, missing source assets and unresolved semantics explicitly.
- Preserve the independent viewer, immutable user-supplied game data and licensing rules.
- Use the available GPU for rendering and deformation. Establish the selected adapter
  from the running application; do not infer acceleration from installed drivers.
- Build through `tools\build.bat`. Keep corpus checks independent of renderer success.

## Milestone register

| ID | Functional outcome and acceptance gate | State |
|---|---|---|
| D0 | Existing archive/geometry/texture paths continue working; all relevant codec and digest checks remain green | Canonical index and viewer catalog agree on82packages/158,720records; allpayloads and four main RTX regressions pass; older compatibility CLI callers remain explicit |
| D1 | Typed RW4 skeletons, bind matrices, skin weights and animation tracks; valid references and finite values across the measured corpus | Typed-data gate passes on all 13,498 RW4; 4 static mesh attachment cases remain unsupported |
| D2 | ANIM keyframe values, channel identities and safe sampling; independent byte-level and full-corpus validation | Keys/preparation pass 5,022 resources; 32,256 controlled and 692,878 finite all-source POS samples agree; four NaN copies reject explicitly; whole bounded ROT branch matches30,240 baseline plus3,456 boundary contexts; deformation and full binding remain open |
| D3 | A real skinned model animates on the RTX GPU; bind pose matches source geometry, two times produce different finite poses, switching/restarting clips works | Source sample rendering and CPU agreement pass; actual playback-system control tests pass |
| D4 | Creature channel-to-rig mapping and deformation; full assembled creatures move with their own texture and stable attachments | Full 11,041 GMDL tails agree; all704 linked creations pass original-program RTX static captures; ordering/palettes,8,624 root-policy cases,4,095 source contexts and optional construction agree; finalizer fields match4,590 prepared native/Python/Rust cases and seven focused tests, fresh-source reconstruction comparison pending; evaluated animation/full binding remain open |
| D5 | Terrain/blockdata semantics, collision geometry and placement validated against source data | Open |
| D6 | Usable audio, event routing, UI schemas, text/locales, materials and remaining required resource types | Typed SPUI framing passes748/748; image/layout/original UI programs pass bounded checks; all4,497 locale tables agree on419,485 entries;51,680 unscaled glyph packets and37,691 simple phantom/scaling sets agree across five faces; selector1 passes379,589 records; FPGM source oracle reproduced, Rust check pending; native references cover578 glyphs/one font at15ppem with seven context differences; composites/hint/raster, full screens/events, audio and runtime locales remain open |
| G1 | Cell gameplay: controls, feeding/diet, swimming, growth, predators/combat, death/restart, editor integration and evolution into Creature | Original-program draws pass all 28 resolved primary Cell models through main capture; source camera orientation/FOV recovered; retail framing, draw state, animation, effects and behavior remain open |
| G2 | Creature gameplay: locomotion and animation, food/combat, social behaviours, nests/species, progression and transition to Tribe | Original simulation and GPU frontend pass initial checks; source animation/editor/transition and fidelity gates open |
| G3 | Tribe gameplay: group selection/orders, food/resources, tools/buildings, social/combat interactions, opposing tribes and progression | 23 core and 5 frontend checks pass, including a complete original diplomacy route; GPU/runtime and fidelity gates open |
| G4 | Civilization gameplay: cities, vehicles, economy, military/religious/economic interactions, AI opposition and Space transition | Open |
| G5 | Space gameplay: galaxy/system/planet generation and travel, missions, trade, colonies, terraforming and persistent simulation | Open |
| E1 | Cell/creature editors: parts, valid attachment/transforms, palette/paint, undo/redo and persistent authored creations that work in gameplay | Original creation graph/transaction/save tests pass; UI and gameplay integration in progress; source catalog open |
| E2 | Building/vehicle/adventure editing and saved-content integration, with the subset defined by the source install documented | Original graph formats implemented; user interface and gameplay/source integration open |
| X1 | New-game and cross-stage progression, versioned save/load, settings, input remapping, sound/UI and robust failure handling | Open; per-stage save foundations begin with G1 |
| X2 | End-to-end campaign and editor acceptance, performance/memory/GPU profiling, no proprietary redistribution, release packaging | Open |

## Current work ownership

- Native creature body/part palettes, animation binding and measured matrix conventions.
- Original material links, compiled render states, fragment builders and shader bindings.
- Source Cell camera classes, property inheritance and original asset provenance.
- Source UI/data integration, offscreen renderer checks and acceptance evidence.

The original Cell simulation and procedural frontend are retained behind
`--experimental`. The separate `--source-cell` preview loads the installed artwork
without generated substitutes. Neither currently passes the original game's behavior,
camera, material and interface acceptance gates. Authored animation sampling policies
also must not be described as recovered proprietary interpolation without evidence.

## Evidence log

- Starting baseline: 171 tests and 5,022/5,022 animation metadata resources across 82/82
  packages, recorded in `continuation-2026-10-22.md`.
- Runtime selection confirmed `NVIDIA GeForce RTX 3060 Laptop GPU`, Vulkan, DiscreteGpu.
  The original static creature viewer also rendered with that adapter.
- ANIM: 82/82 indexes, 5,022/5,022 resources, 48,700 channels, 878,791 rows. Independent
  per-resource digests agree exactly, including unsupported/raw bytes. Four nonfinite
  source position keys give explicit sample errors. See `re/anim-channels.md`.
- RW4: 13,498/13,498 resources, 9,966 skeletons, 26,061 bones, 3,053 bindings,
  7,803 clips, 584,177 keys. Independent Python/Rust digests agree on 28,724 canonical
  records from all 3,294 resources with relevant objects/influences. 4,849/4,853 mesh
  pairs decode 5,007,037 influence records; 4 static declarations have unknown attachment
  semantics. See `re/rw4-animation.md`.
- GPU skin sample `40606100AD73C528`: 632 vertices, 1,064 triangles, 3 bones. Bind
  maximum error 0.000000015 source units; time 0.25 maximum displacement 0.073260702.
  Identically framed GPU/CPU pose images differ by more than 2 channel levels at only
  1/1,958,400 pixels below the HUD. Bind versus animated GPU images change 69,023 pixels.
  Reproduce with `tools/compare_skin_renders.py`; local evidence is
  `target/animation/rtx-render-agreement.json`. This is renderer agreement and motion,
  not a retail material/curve-fidelity claim.
- Cell RTX eight-second demo: 480 ticks, 13 nutrients, 40% growth, alive. Actual control
  system tests cover movement, pause, diet, save/load, restart and failed-load preservation.
  A boost regression subsequently fixed acceleration and exhaustion; its test passes.
- Creature RTX twelve-second demo: 720 ticks, 80 DNA, 3 parts, 1 resolved species,
  1 pack member, full health. Headless actual-input demo reaches TribeReady through
  normal bones, social and pack actions. The captured UI was visually inspected.
- Workspace suite passed 233 tests: 20 dbpf, 199 core, 14 binary. Subsequent editor
  core addition passed 18 focused tests; the next full suite must include later changes.

Source game files remain read-only. GPU captures, native Evidence and digests stay under
ignored `target/`. The full game roadmap is not complete, and the active goal continues.

- Latest native integration:586 test executions pass across all workspace targets,
  comprising574 primary unit tests and12 repeated asset-example tests, with two
  intentional installed-corpus ignores. The canonical viewer/source catalog agrees
  on all158,720 keys and all704 linked creations pass strict saved-mesh/map intake.
  Whole bounded ROT sampling matches30,240 baseline plus3,456 arithmetic-boundary
  contexts. Inherited Cell classes, all1,365 source geometry/cache compositions and
  all33,480 finite scalar-cache cases match original-byte fixtures. Body capability
  expansion matches58,802 cases and movement source getters match68,340 finite
  cases. Movement scaling matches261,960 finite and3,864 nonfinite-table cases;
  bounded root selection matches8,624 native/Python/Rust projections, retaining
  every nonadmitted candidate. Source extents/origins/anchors agree across139,254 row
  evaluations. All4,497 locale byte grammars and all51,680 unscaled source glyph
  packets independently agree. Native font
  references cover4,624 observations while preserving seven fresh/sequence
  differences; these are reference pixels without a Rust raster comparison. Live actor/controller
  wiring, generated hierarchy, original text/screens and all gameplay/editor
  fidelity gates remain open; see the checkpoint for exact scope and reproduction.

- Tribe gameplay fixes passed 23 core and 5 frontend tests: distinct interaction
  approach slots prevent hut congestion, and repeated AI raid orders preserve earned
  diplomacy. The normal-cost demo equips four members, delivers 32 food and allies both
  opponents. Raid timing/damage and victory gates are unchanged.
- PCTP typed parsing and three independent resource digests pass across 82 packages.
  Creature source ranges and bounded initial selector candidates are implemented;
  native generated-block/root/order finalization is explicitly unsupported.
- Source Cell chain audit: 223 definitions, 222 resolved structures, 90 baked references;
  65 rows resolve 28 unique actual creations with their own source geometry, painted
  atlases and thumbnails. 25 rows name 17 IDs absent from model/art resources across all
  82 indexes; one other definition names structure zero. Five focused/installed-source
  checks pass. This is asset intake coverage, not Cell gameplay fidelity.
- SPUI typed decoding: 748/748, 93,726 elements, 1,183,787 recursive properties.
  All independent Python/Rust resource records and digests match; native reader/writer
  analysis corroborates framing. See `re/spui-layout.md` for remaining runtime semantics.

- Latest integration: 406 workspace tests passed (350 core, 36 binary, 20 archive),
  with two intentional installed-corpus ignores. Native uniform dispatch and `210`
  compatibility conversion are covered; all 17,730 real parameter records agree
  independently. Original UI bytecode still gives zero RGBA error on the RTX after
  descriptor validation. These are rendering/data gates, not completed stages.

- Float-color correction: 85,210 independently matching components in 28,161
  source colors; signed/HDR lighting values preserved. GMDL sampler bindings
  independently agree on 42,446 records. Refreshed Jawhead/Stabber RTX captures
  use the recovered skin sampler. Original 3D programs and lighting transforms
  remain the next rendering gate; no stage/editor milestone is closed by this.

- 2026-10-23: full workspace suite passes 411 tests with two deliberate corpus
  ignores. The original-program renderer passes all 28 resolved primary Cell
  models on the RTX. Main `--source-cell --shot` captures match that corpus
  exactly; the separate `--pbr-inspection` regression is unchanged. Standalone
  RASTER mip framing/content independently agrees on all 5,931 resources /
  56,936 levels. Native unpack marker, float SH prefix and complete source
  texture chains are implemented. Retail camera/state, animation, UI and
  gameplay fidelity gates remain open.

- Latest 2026-10-23 integration passes 434 workspace tests (378 core, 36 binary,
  20 archive), with two intentional corpus ignores. Native UI copy/blend draws
  agree on all 16 object slots, both original clipping programs and 131,072
  source pixels per fixture. All 15,489 Image regions independently agree.
  Native ANIM descriptor preparation agrees on 5,022 clips; typed Cell fields
  and bounded AI/eating branches agree on all 489 copies. Full 128-byte
  CreatureData headers agree on 1,365 resources and preserve every prior rig
  digest. These are measured implementation gates; full stages/editors and
  retail screen/animation acceptance remain open.

- The next 2026-10-23 integration passes 448 tests (392 core, 36 binary,
  20 archive), with two intentional corpus ignores. Bounded ordering agrees
  on 21,109 explicit hierarchies; all 1,161 FadeEffect records agree, including
  62 explicitly unsupported modes/triggers. Subsequent focused/native-math
  checks cover palette gathering, sampled Cell flow, movement application and
  explicit x87 contexts. These require the next full integration cohort;
  neither data agreement nor supplied-state arithmetic closes a gameplay gate.

- Latest 2026-10-23 integration passes 484 tests (420 core, 44 binary,
  20 archive), with two deliberate installed-corpus ignores. It includes native
  Cell swimming kernels, explicit x87 arithmetic, position curves, source-font
  metadata, DXT3 glyph-alpha packing and authored asset/CLI routing. A full saved
  11,070-triangle creature renders with original programs and mip chains through
  main --source-creature; standalone/main RGBA agree exactly. Position arithmetic
  agrees on 32,256 original-byte cases over twelve explicit contexts; its
  692,882-case all-source comparison is still pending. Font metadata agrees on
  91,967 records and glyph packing on 5,201 cases. Retail animation, text
  raster/layout, complete scenes, gameplay stages and editors remain open.

- Latest 2026-10-23 integration passes 512 tests (448 core, 44 binary,
  20 archive), with two deliberate installed-corpus ignores. All 704 linked
  creations now pass hidden original-program RTX captures, including expansion
  assets, with 7,804,809 triangles across the corpus. The all-source position
  comparison passes 692,878 finite samples and rejects four authored NaN copies.
  Source body intake, 107,208 Cell capability-curve cases, 145,356 Cell geometry
  cases and 86,124 quaternion primitive/boundary cases have independent native
  or source comparisons. Actual ROT curves, generated binding, inherited part
  classes, complete UI text/screens and all retail gameplay/editor gates remain
  open. The checkpoint retains exact scopes and reproduction paths.

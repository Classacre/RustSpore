# Continuation checkpoint, 2026-10-22

## Latest scope correction

The user requires an identical Rust remake of SPORE using the installed game's assets.
The procedural gameplay/editor renders shown during this continuation do not meet that
acceptance target. Retain their deterministic simulation/save work as experiments, but
do not count their original art or rules as completed game milestones. Source resource
chains, materials, UI layout and native animation semantics are now the critical path.
The user asked to avoid desktop automation; continue with packages, executable analysis
and automated renderer checks. The original game was launched and its startup/title
display briefly inspected before the user stopped Computer Use with Escape. No further
desktop input is authorized by this continuation workflow.

Current source UI investigation: `tools/spui_layout_probe.py` finds 748 SPUI resources
in top-level Data/DataEP1/bp1content packages, with 3 unsupported package index shapes
recorded explicitly. Executable literal hashes identify TitleScreen
`404642000910F2B9`, SporeTitleSeq `404642005C5AC8CB` and GlobalUICell-3
`4046420026B068D2`. The body is a typed property stream, not opaque designer data;
the corrected Python and Rust decoders now walk all 748/748 exactly, including 93,726
elements and 1,183,787 recursive properties. All 748 independent digest records agree.
Native reader 0x82CE10 and writer 0x82DCE0 corroborate stream framing. Class defaults,
procedure execution and faithful UI rendering remain open. See `re/spui-layout.md`.
Raw resources and source artwork stay under ignored `target/source-ui/`.

The default launcher no longer opens experimental art. Prototype gameplay/editor modes
require `--experimental`; source-data tools retain their own flags. A shared offscreen
GPU helper now creates an image render target with no desktop window. Its first static
viewer capture passed; the first capture lacked HUD because Bevy's default UI camera
requires a primary window. An explicit default UI camera fix subsequently passed the
source Cell captures below. No desktop input is used by these render checks.

### Latest source reconstruction evidence

`[MEASURED]` Latest regression suite: 406 workspace tests passed (350 core, 36 binary,
20 archive) and two deliberate installation-specific corpus checks were ignored.
This includes the compiled-recipe and native constant-dispatch tests. The default executable and the
optional native D3D9 example build through `tools/build.bat`.

`[MEASURED]` Original `UTFWin_UIShader` vertex/pixel bytecode now executes on the
RTX 3060 through a local Rust D3D9 probe. Direct-package draws of a 512 x 256 UI
atlas and Jawhead's 128 x 128 source thumbnail have zero changed RGBA pixels,
including transparency. The Optimus DWORD needed an actual PE export; that fix is
local to the optional example, with no global GPU setting changes. Controlled
bindings, hardware identity and limitations are in `re/shader-execution.md`.

`[MEASURED]` Offscreen capture now waits for pipeline completion, required GPU
images, laid-out text and its glyph atlases, with eight stable render frames.
Transient ShaderNotLoaded/import states wait; permanent shader failures abort.
Three source Cell captures passed: Jawhead twice and Stabber once. HUD, thumbnail
and caption pixels are present in all three; repeated Jawhead captures differ at
zero pixels above two channel levels. Evidence is
`target/source-cell/readiness-verification-v1.json`. Source camera orientation
and perspective are recovered in `re/cell-camera.md`; framing and lighting remain
inspection choices. The animation capture also completed through this barrier.

`[MEASURED]` Every one of 14,355 compiled source shader programs resolves through
its native zero-terminated recipe key, with independent Python agreement and no
ambiguous recipe within a table. Uniform dependency masks agree too. See
`re/shader-selection.md`. This enables original permutation retrieval; the actual
Cell draw's material registration, feature mask and constant bindings remain open.

`[MEASURED]` Native constant dispatch now has a typed implementation in
`shader_binding.rs`, including the distinction between callback and per-data
dirty IDs. The native `210` callback explicitly tests marker `5C 44 26` and
uploads a marked fourth component or legacy 1.0. All 17,730 source payloads
agree independently as float bits; two focused tests cover truncation, each
marker byte, separate dirty IDs and forced/mixed dependency masks. This corrects
the earlier padding hypothesis. See `re/material-links.md`. The source UI
descriptor check also rebuilt and retained zero-error RTX thumbnail execution.

`[MEASURED]` The selector's required mask comes from current vertex declaration
`16F75A0 + 10`, not generic renderer options. Native `11F2FD0` ORs the measured
semantic table's bits. Rust `VertexFormat::native_shader_features` and independent
Python agree on all 13,269 declarations / 51,437 elements in 11,041 source GMDLs.
Jawhead's saved declaration gives `0008C045`. The context field is now named
`declaration_features`; all nineteen focused shader checks pass. The earlier
406-test integration predates this small feature accessor/context rename.
See `re/shader-selection.md` and
`target/source-shaders/declaration-features-comparison-v1.json`.

`[MEASURED]` A second PROP semantic defect was corrected while recovering source
lighting: RGB/RGBA values were previously the first bytes, instead of three/four
LE floats. Independent agreement now covers 28,161 colors and 85,210 components
in all 30,885 certified named lists, including signed/HDR values. Four original
framing exclusions remain excluded. The source Cell path now retains native
per-mesh declaration features, `210` constants and the normal/paint lighting
coefficient resources. Raw source coefficients are not fed to PBR as a substitute
for the native SH pipeline. See `re/prop-numeric.md`.

`[MEASURED]` Workspace integration after the float-color and source-input changes
passes 407 tests, with two intentional corpus ignores. Native RGB consumers and
source SH data remain available in v3 queries 92, 109 and 110. Source normal
lighting is `40200100FCED73A2`; paint lighting is `40200100A9CA81EB`. Both contain
25 RGB coefficients. Transform/normalization and exact 3D execution remain open.

`[MEASURED]` Shader-data `20D` texture records are now typed from the native
builder callback, with 42,446/42,446 independent semantic records agreeing.
Important address arithmetic correction for future work: `16F7DDC` is `20D`,
not `201`; the earlier investigation summary mixed up those indices. `201` is
at `16F7DAC` and is a separate draw selector input. No pointer remapping between
them is needed to explain the texture path. See `re/material-links.md`.
Jawhead's units 0/1/2 are clamp/linear; global texture `552A5077` resolves as a
64 x 64 DXT5 RW4, not a missing generated resource. Three focused binding tests,
eight source Cell tests and the latest full input capture pass. The sampler
change built successfully and passed new Jawhead/Stabber RTX offscreen captures.
Both wait for eight stable render frames; the capture records are in
`target/source-cell/native-sampler-verification-v1.json`. These captures show the
source sampler change, not original 3D shader or lighting fidelity.

`[MEASURED]` The UI camera fix passed: Jawhead and Stabber source Cell creations
render with their own painted atlases, source thumbnails and provenance HUD on the
RTX 3060 in a windowless image target. Captures are in ignored
`target/source-cell/`. These inspect original artwork; Bevy PBR and an inspection
camera still differ from original rendering. The installed Cell corpus contains
223 definitions, 222 resolved structures, 90 baked-reference rows, 65 available
baked rows and 28 unique available creations. Missing references remain explicit.
See `re/cell-source-assets.md`.

`[MEASURED]` A real numeric bug was corrected: PROP scalar float/int32/uint32
payloads are big-endian, including arrays. Independent native reader/writer
analysis and 847,835 primitive values in 30,885 resources agree. Four framing
exclusions remain documented. This correction is necessary for recovering actual
camera/gameplay properties; size-only parsing did not certify those values.
See `re/prop-numeric.md`.

`[MEASURED]` Full GMDL tail agreement now covers 11,041 resources and 46,576
external bone records. Independent influence agreement covers 1,053 unambiguous
meshes and 5,105,226 vertices. Native analysis recovers the external bone record
enable branch and exact linked-skeleton order; generated body hierarchy remains
unresolved. See `re/gmdl-skin.md` and `re/anim-rig-native.md`.

`[MEASURED]` New source shader decoders agree independently on all 12 header-1
fragment libraries and all 12 header-7 shader packs. Packs preserve 3,136 standard
shaders, 1,040 builders, 11,064 compiled vertex programs and 3,291 compiled pixel
programs across package/profile copies. Original material/state joins, selector
predicates, shader execution and faithful appearance remain separate gates.
See `re/shader-resources.md`. Original HLSL, bytecode, fonts and artwork stay in
the user's packages or ignored `target/` diagnostics.

The reduced native REA session remains persistent for bounded queries. Root is
the sole owner; the previous agents are no longer live. Do not start another
expensive Ghidra analysis or mix writes to the helper. Current session output is
`target/rea/20261022-source-runtime-v3/`, with the base executable and focused
profile digests recorded there. Cargo builds must also remain serialized.

`[MEASURED]` Integration check after the new source material/shader readers:
388 workspace tests pass, with two deliberate installation-specific corpus
ignores. A subsequent shader-focused check passes 10/10 after native optional-name
flags were added. Main binary and both shader corpus examples rebuilt successfully.
The existing unused-field and incremental-lock cleanup warnings remain warnings.

`[MEASURED]` Animation `--shot` now uses the same windowless renderer as the static
and source Cell previews. A new RTX Vulkan capture of source rig
`40606100AD73C528` at 0.25 seconds passes the CPU reference comparison:
3/793,600 pixels below the HUD differ by more than two channel levels; animated
versus bind changes 30,861 pixels. Source-space bind error remains 0.000000015
and displacement 0.073260702. Evidence:
`target/animation/offscreen-render-agreement-v1.json`. This validates preservation
of the GPU path during the desktop-free change, not original material/curve fidelity.

## Where the project was

The latest project log, `.workbuddy-ai/memory/2026-10-22.md`, closed task #25:
palette-based creature skin generation. The application is a package reader and static
asset/creature viewer. Animation playback, skeletal deformation and gameplay remain open.
The same log left task #2 open after Ghidra's 330-second startup timeout.

The review covered the folder structure, Git state, project logs, current Rust workspace,
roadmap and binary-analysis evidence. The initial 161-test suite passed. Existing
uncommitted work was retained; no commit or push was made.

## Changes and verification

- CLI identifiers now trim surrounding whitespace and CRLF, accept `0x`/`0X`, and reject
  malformed or overflowing values with exit code 2. Mesh indices and explicit screenshot
  targets are validated too. A bare hexadecimal screenshot target now selects a prop,
  matching the documented interface; it was previously silently ignored.
- The animation metadata parser checks the channel-table boundary and bone offsets,
  and bounds the path terminator search to its fixed field. Truncated input no longer
  triggers the identified out-of-bounds slices. Keyframe values are still not exposed.
- Five CLI and five malformed-animation regression tests were added. The combined
  workspace suite passed **171 tests: 20 dbpf, 146 spore-core, 5 CLI**. After the final
  screenshot-target adjustment, all five CLI tests passed again and the binary rebuilt.
- Fifteen malformed real CLI invocations exited 2. A whitespace/CRLF-padded creature
  instance resolved as `406B62000660EACA` and executed successfully.
- `crates/spore-bevy/examples/anim_verify.rs` provides a reproducible Rust corpus check,
  using index-only package reads and loading one animation at a time.
- README, roadmap and payload documentation now distinguish parsing, usable metadata,
  generated colouring, resolved geometry and unfinished animation/gameplay.

`[MEASURED]` Corpus run against the user-supplied install:

| Package | Animation resources parsed |
|---|---:|
| Data/PatchData.package | 2,223 / 2,223 |
| Data/Spore_Game.package | 2,091 / 2,091 |
| DataEP1/Spore_EP1_Data.package | 683 / 683 |
| bp1content/BoosterPack_01.package | 25 / 25 |
| Total | **5,022 / 5,022** |

All **82 / 82** package indexes opened, exposing **48,700** animation channel metadata
records, with zero read/parse failures. This is parser success, not exact byte-consumption,
keyframe-value, bone-mapping or playback validation.

```powershell
tools\build.bat test --workspace
tools\build.bat run -p spore-bevy --example anim_verify -- "D:\Games\SPORE Collection"
```

## Reverse-engineering environment

REA MCP 6.5.0 was installed and responded to `binary_session`. Its initial session had no
`GHIDRA_INSTALL_DIR`, although Ghidra 12.1.4 and a suitable Java 21 JDK were installed.
The user-scope environment already held the path; the desktop process had stale values.

The Codex REA configuration now explicitly supplies the Ghidra path and
`REA_GHIDRA_STARTUP_TIMEOUT_MS=1200000`. A timestamped backup was made before that
two-setting change, and TOML parsing plus a semantic comparison verified preservation of
other settings. An already-running MCP process needs reconnection to receive those values.

`tools/rea-spore.ps1` applies the same values to a fresh CLI process immediately, pins REA
6.5.0, restores its process environment afterward, and keeps Evidence and snapshots under
ignored `target/rea/`. Snapshots cache query Evidence; they do not preserve a Ghidra project.
A new CLI query may repeat startup, so keep one MCP session open for successive queries.

```powershell
.\tools\rea-spore.ps1 -Operation trace -Query 'SPSkinPaintFloodEffect'
.\tools\rea-spore.ps1 -Operation function -Procedure '0xADDRESS'
```

The full default profile still exceeded the new 20-minute startup budget. An explicit
`-Focused` option skips `Stack`, `x86 Constant Reference Analyzer` and
`Decompiler Switch Analysis`, using separate snapshots. This reduces automatic recovery
of local stack variables, computed references and switch/jump-table targets; absence of
those facts is not evidence that the executable lacks them.

`[MEASURED]` The focused profile completed analysis in 676 seconds (Ghidra log), and
seven selected functions decompiled in the same persistent MCP session. Two flood-effect
registration strings led to `0x535900` and a construction path involving `Skinner` objects.
The exact painter remains unknown; three indirect call sites remain unresolved.
Ten Evidence records were exported to ignored `target/rea/20261022-paint-focused-v2/`,
and the analysis session closed. Task #2 is partially unblocked, with reduced coverage.
See `re/binary-analysis.md` and `re/paint-entrypoints.md` for the profile and Evidence IDs.

## Next functional milestone

Decode usable RW4 skeleton, skinning and animation structures, then establish how channel
identifiers map to the assembled creature. The current `AnimBone` is metadata only and
RW4 object-size validation does not establish deformation semantics. Preserve the existing
static viewer and independent corpus/digest checks while adding those capabilities.

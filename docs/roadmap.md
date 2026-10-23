# RustSpore roadmap and scope

Current acceptance target (2026-10-22): reproduce the original game's artwork,
interfaces and behavior using the user's installed assets. The five stages and
editors remain open under that gate. Original procedural simulations are retained
only behind `--experimental`. Current functional status and source/native evidence
are tracked in [functional-milestones.md](functional-milestones.md) and
[continuation-2026-10-22.md](continuation-2026-10-22.md); earlier size-based parser
percentages below are historical coverage metrics, not semantic or game completion.

## 1. What this project is

A from-scratch reimplementation of SPORE (Maxis/EA, 2008) in Rust on Bevy. The
reimplementation reads assets from a copy of the game the user owns. It does not ship any
EA asset, and the repository excludes `*.package` files and anything under
`assets/extracted/`.

The first milestone is not "a game". It is a toolchain that can read the shipped data
correctly, because without that there is nothing to build a game on. That milestone is
now partly met.

## 2. Scope, in measured numbers

Worth stating plainly, because the numbers set the shape of the work.

| Quantity | Value | Source |
|---|---|---|
| Asset archives in this install | 82 | `find` over the install |
| Total archive size | 6,173,405,831 bytes (5.75 GiB) | file sizes |
| Resource records | **158,720** | `rustspore --types`, summed |
| Archives whose index layout is unknown | **0** | all 82 open |
| Resources in a type with a Rust parser, historical size-based inventory | **151,009 (95.14%)** | `rustspore --types`; not a semantics/fidelity score |
| Coded resources decompressed exactly | **101,312 of 101,312** | `rustspore --verify` |
| Distinct resource types | 90 | `rustspore --types` |
| Engine executable size | 20,447,744 bytes (base), 24,888,320 (EP1) | `re/binary-analysis.md` |
| Executable code section | 16,560,128 bytes | section table |
| Imported DLLs / imported functions | 20 / 495 | import table |
| Game stages to implement | 5 | Cell, Creature, Tribe, Civilization, Space |

For calibration: a full SPORE recreation is a many-person, multi-year effort. The
executable alone is 16.5 MB of compiled code with a custom engine, a procedural creature
system, a galaxy-scale simulation and five distinct gameplay modes. Any plan that implies
otherwise is not honest.

What is achievable is a sequence of independently useful milestones, each of which can be
verified against the real game data.

## 3. Current state

### Done and verified

- **Canonical DBPF index verified.** The original reader `8DAFE0` establishes shared
  type/scope/high words and current-record compression metadata. Independent Python/Rust
  keys and payload spans agree on all 82 packages / 158,720 records, and all payloads
  read/decompress to their declared size. The canonical viewer catalog also agrees
  on every key. Native identity is instance/type/scope; exact-size record-shape fits
  alone did not establish that identity. See `re/archive-index-native.md`.
- **Historical package-shape investigation preserved.** The earlier contiguity/size
  work recovered correct payload spans for the audio and compact locale archives.
  Its apparent group/trailer labels are superseded by the native reader. `RecordShape`
  and older CLI/corpus callers remain compatibility tools; new source loaders use
  `NativePackageIndex` and typed native keys.
- **Engine architecture mapped.** RTTI is absent, so the class names were recovered from
  string literals instead: 344 simulation classes, 340 ArgScript names, 332 console
  commands, and a full middleware inventory including RenderWare 4, Havok 3.1.0, Cal3D,
  RenderWare Audio, EAPd and EASTL. `docs/engine-architecture.md`.
- **RefPack codec.** The Rust port decompresses **101,312 coded resources across all 82
  packages** in one retail install, each to exactly the size declared by the DBPF index,
  using only the size declared inside the stream (`rustspore --verify`). Two independent
  sources of truth agree, on the whole install rather than a sample. Python reference in
  `tools/refpack.py`, Rust port in `crates/dbpf/src/refpack.rs`.
- **Raw versus coded rule.** `stored_size == decompressed_size` means raw. Measured split
  in `Spore_Game.package`: 2,215 raw, 9,314 coded.
- **Binary fingerprinting.** Sections, imports, Rich header, mitigations, and the
  corrected finding that the two executables' `.text` sections are 93.96% different
  despite near-identical sizes.
- **Rust workspace.** Three crates: `dbpf`, `spore-core`, `spore-bevy`. The whole
  workspace builds and a viewer renders real assets. Current continuation checks and gameplay gates are tracked in `docs/functional-milestones.md`.
- **Payload formats largely decoded.** Nine major types now have working
  decoders or validated structural analysis:
  - **anim (0xEE17C6AD): typed keys and generic sampling on all 5,022 resources.**
    Independent Python/Rust digests agree on 48,700 channels and 878,791 rows.
    Source nonfinite values produce explicit sampling errors; authored curves and
    channel-to-assembled-creature rig binding remain open. `re/anim-channels.md`.
  - **gmsh (0x01C135DA): 100% validated.** Header, position data, attribute sections,
    index v1, index v2+ (variable footer). All 2,482 samples parse with valid
    indices; the index magic accepts `15 00 06 08` and the variant `14 00 06 08`.
  - **RASTER (0x2F4E681C): pixels decoded.** DXT1, DXT3, DXT5, raw BGRA and L8
    convert to RGBA8 in Rust, including textures embedded in RW4 containers.
  - **Cell definition (0xDFAD9F51): partial.** Fixed 796-byte records; source structure
    links and developer labels identified. The earlier string-table claim was wrong.
  - **RW4 (0x2F4E681B): container and per-object payloads 100% validated.** 10,797 of
    10,797 resources, every object inside them too. Container: magic, header
    (`rw_type_code`@28, `section_count`@36, `p_section_infos`@48, `p_buffer_data`@68),
    the 24-byte section-info table, every object's type and data range, and full file
    tiling. Objects: Raster, VertexDescription, VertexBuffer, IndexBuffer, Mesh,
    CompiledState, SkinnedInK, Skeleton, KeyframeAnim, Animations, BlendShape,
    BoundingBox and the 32-byte `0x7000B`, each consuming exactly its `data_size`.
    `tools/rw4_objects.py` reports `All-objects-exact: 10797 (100.0000%)`.
  - **prop (0x00B1B104): 100% decoded.** Two array markers, mixed endianness,
    length-prefixed string arrays, low-word separator detection, and a named-first /
    anonymous-fallback backtracking walk. 23,762 of 23,762 resources parse exactly
    across the six prop-bearing packages.
  - **blockdata (0x5CAA5E28): 100% structurally decoded.** A 36-byte header (bbox,
    one float, an 8-byte flag context) plus an adaptive 4-ary quadtree. The data
    length is a closed form of the flag context: let `c` be the number of `0x04`
    bytes in bytes 28..36, then `total = 36 + 12 * 4^(c-1)`. All 3,098 samples
    satisfy it exactly. Record semantics (node vs leaf) remain unknown.
  - **audio event (0x02B9F662): 100% decoded.** Variable-length entries: a bool
    value is 1 byte, the `flags` field carries the same two array markers as prop
    (`0x009C`, `0x001C`), string arrays are length-prefixed per element, and
    property names are FNV-1 32-bit hashes (parent, gain, is3d, samples, codec).
    16,841 of 16,841 resources across `PatchData` and `Spore_Audio2` parse exactly.
    The sibling type **0x03F51892** uses the identical entry rules and also parses
    at **100% (90 of 90)**; the two share one type-agnostic parser.
  - **gmdl model (0x00E6BCE5): 100% exact (4,209 of 4,209).** Full structure
    decoded: header (offset 4 is a big-endian `ref_count`, header =
    `64 + ref_count*12`), referenced files, triangle buffers, vertex formats with
    per-element stride, vertex buffers, meshes, materials, and skeleton. The last
    blocker was shader data `0x238`, which is a single byte: read as a
    length-prefixed blob it merges with the following `count1` field and looks
    like a size (e.g. `0x111`). Part records are a fixed 132 bytes.
- **Type census completed.** The full-install census covers 158,720 resources across
  90 distinct types; section 4 gives that population. An earlier 91,267-resource,
  80-type subset established these layouts: hash-pair table `0x02D5C9AF` (1,631),
  content manifest
  `0x030BDEE3` (1,641), creature editor save `0x0F43029A` (1,022), the spui
  container `0x250FE9A2` (601), and the creature body editor model `0x1A99B06B`
  (1,733). The public SporeModder catalog names them SummaryFile, PollenMetadata,
  creaturedata, spui and bem. SPUI's typed body now decodes on all 748 resources;
  native class defaults and UI procedure semantics remain open. `0x01A527DB` (snr,
  audio) has its container decoded (2,898 of 2,898)
  but its codec payload is out of scope.
  Text formats: XML (9 types), Lua, PureData, WAV, OLE, nested DBPF.
- **Rust format parsers.** Fourteen formats have Rust implementations in
  `spore-core`: `gmsh`, `anim`, `string_table`, `raster`, `blockdata`, `prop`,
  `rw4` (container plus per-object extents), `audio_event`, `gmdl`,
  `hash_pairs` (0x02D5C9AF), `content_manifest` (0x030BDEE3), `creature_data`
  (0x0F43029A), `spui` (0x250FE9A2), `bem` (0x1A99B06B). The `mesh` module converts
  geometry and `skin` generates palette-based creature atlases. `spore-core` has
  independent parser and gameplay tests. The dated continuation evidence records the latest executed suite; historical parser counts are not current gameplay coverage.
- **The asset pipeline runs end to end.** `rustspore --viewer <package>` renders real
  models with real textures. 6,714 meshes in `Spore_Graphics` (2,422 gmsh plus all 7,402
  RW4 meshes across 4,292 files, blend shapes included) and 4,208 gmdl in `Spore_Content`
  convert with zero failures: 36.4 million vertices and 27.7 million triangles. The DXT
  decoder matches the Python reference byte for byte on all 2,954 RASTER resources and on
  all 8,915 textures embedded in RW4 containers, and the gmsh parser matches on positions,
  indices, normals (both encodings) and UVs for all 2,422.

### Verification and remaining blockers

**The workspace builds and its tests pass.** `tools\build.bat build --workspace` finishes
in about three minutes cold and produces `target/debug/rustspore.exe` (91 MB).
The starting continuation baseline was **171 tests, 0 failures** (`dbpf` 20,
`spore-core` 146, CLI 5); later parser and gameplay checks are recorded in
`docs/functional-milestones.md`. Run `tools\build.bat test --workspace` for current checks. The binary opens a real package and reports:

```
  index entry count: 11529
  index offset     : 19887288 (0x12F74B8)
  index size       : 322820
  index end        : 20210108 vs file size 20210108 => consistent
  record shape     : 28-byte records, 4 byte prefix, 4 byte trailer (Wide28)
  resources        : 11529 (9314 coded, 2215 raw)
```

and then starts the Bevy app (Vulkan on the reported GPU). Earlier revisions of this
document said the workspace could not be built. That was wrong: Visual Studio 18 Community
with MSVC 14.51 and the Windows SDK 10.0.26100 are installed, and `link.exe` was present
all along. The `STATUS_STACK_BUFFER_OVERRUN` crashes were a transient environment problem,
not a property of the dependency tree.

**The Rust reader has now been run against the whole install.** `rustspore --verify`
decompresses every coded resource and checks the result against the size the index
declares:

```
packages=82 opened=82 coded_total=101312 exact=101312 problems=0
```

All 82 packages open, including the three index variants in `docs/dbpf-format.md` section 6
and the single-resource packages. Every coded payload decompresses to exactly its declared
size. That is the first time the Rust codec has been pointed at real data; the earlier
"800 resources across two packages" figure came from the Python prototype.

Three bugs surfaced once the code actually ran, which is why running it mattered more than
the build succeeding:

- A RefPack test asserted the wrong thing about the end-of-stream marker: it used `0xFC`
  (low two bits `0`), which emits no trailing literals, but expected three. The decoder
  was right and the test was wrong.
- The decoder accepted an end marker that arrived before the declared output size was
  reached, returning a short buffer as success. It now returns
  `RefPackError::Incomplete { produced, declared }`. The declared size is the invariant
  the codec was validated on, so silently under-producing would have hidden truncated
  resources.
- Index shape selection used storage contiguity as the sole arbiter. Contiguity needs two
  records, so a single-resource package (`Locale/en-us/UI.package`) scored 0 and was
  refused even though the 28-byte shape fit its 36-byte block exactly. When `N == 1` the
  exact fit now decides.

**Deep binary analysis has been limited by provider startup.** Three recorded attempts
exceeded the Ghidra provider's 330-second startup deadline during auto-analysis of the
20 MB MSVC binary, without yielding decompilation or a call graph. Current provider
status and recovery evidence belong in `re/binary-analysis.md` section 8. RTTI is sparse
in these executables; the existing engine map comes from string literals, so further
RTTI recovery is not an assumed route to the remaining engine behaviour.

## 4. Staged plan

Each phase is independently useful and verifiable against real data.

### Where the data layer actually stands

`[MEASURED]` Census over all **82 packages** in one retail install, produced with
`rustspore --types` and summed: **158,720 resources across 90 distinct type ids**. Of those,
**151,009 (95.14%) are in a type that has a Rust parser**, and **7,711 (4.86%) are not**.

Two caveats on that percentage, because a single number hides both:

- **4,521 of the unparsed count are not a missing decoder.** They are the locale
  `Text.package` resources, whose index uses the 20-byte variant with no type field at all
  (see `docs/dbpf-format.md` section 6). The parser sets `type_id = 0` and the type is
  implied by the package. Whether that reading is right is an open question about the
  format, not about the decoder.
- **The remaining 3,190 resources span about 73 distinct types**, most under 100 instances.
  The largest are `0x024A0E52` (485, Lua), `0x4B9EF6DC` (376), `0x617715D9` (318, PureData),
  `0x02393756` (153), `0x497767B9` (126), `0x055ADA24` (124) and `0xD92AF091` (115).

`[MEASURED]` Types with a Rust parser, by instance count:

| Type | Name | Count | Parser covers |
|---|---|---|---|
| `0x01A527DB` | snr (audio) | 33,781 | container only; codec not decoded |
| `0x00B1B104` | prop | 30,889 | every byte, as a generic value tree |
| `0x02B9F662` | audio event | 19,631 | every byte |
| `0x2F7D0004` | PNG | 14,694 | decoded to RGBA8 via `image` in `spore-bevy` |
| `0x2F4E681B` | RW4 | 13,498 | container and object extents; geometry and textures usable |
| `0x00E6BCE5` | gmdl | 11,041 | every byte; vertex data typed |
| `0x2F4E681C` | RASTER | 5,931 | header and pixels; DXT1/3/5, raw BGRA and L8 |
| `0xEE17C6AD` | anim | 5,022 | typed keys and generic safe sampling; native rig binding and authored curves open |
| `0x5CAA5E28` | blockdata | 3,260 | every byte; record semantics unknown |
| `0x01C135DA` | gmsh | 2,756 | every byte; positions and indices typed |
| `0x1A99B06B` | bem | 2,621 | every byte |
| `0x030BDEE3` | content manifest | 2,598 | every byte |
| `0x02D5C9AF` | hash-pair table | 2,590 | every byte |
| `0x0F43029A` | creaturedata | 1,365 | container, rigid part transforms and paint palette |
| `0x250FE9A2` | spui | 748 | container and typed properties; runtime UI semantics open |
| `0xDFAD9F51` | Cell definition | 489 | label and structure link; behaviour fields open |
| `0x03F51892` | audio event (variant) | 93 | every byte |

### Phase 1: finish the data layer

"Parsed" here means "consumes exactly the right bytes". It does not mean "modelled", and
the difference is where the remaining work is:

| Gap | What it blocks | Size | State |
|---|---|---|---|
| **DXT1/DXT5 to RGBA in Rust** | textures cannot reach the GPU | 5,931 RASTER resources | **done**, byte-exact against the Python decoder on 2,954 of 2,954 |
| **PNG decoding in Rust** | same, for the other texture type | 14,694 resources | **done**, via the `image` crate in `spore-bevy` only |
| **Mesh conversion** | geometry cannot reach the GPU | 6,630 gmsh + gmdl | **done**, 6,630 of 6,631 convert (one is a valid empty placeholder) |
| **RW4 geometry** | models inside RW4 containers | 13,498 resources | **done**, all 7,402 meshes including 1,391 blend shapes |
| **RW4 textures** | binding the right texture | 8,915 of 9,368 files carry one | **done**, all decode and match the Python reference |
| **RW4 skinning and animation** | deformation and animation playback | 13,498 resources | typed skeleton/bind/weights/clips validated; RTX GPU skinning checked against CPU on a source model; assembled-creature retargeting open |
| **ANIM keyframe values** | evaluating animation channels | 5,022 resources | typed keys and independent full-corpus digests complete; native interpolation/rig mapping open |
| **snr codec** | audio playback | 33,781 resources | open; container known, codec not |
| **blockdata record meaning** | terrain height and placement | 3,260 resources | open; sizes known, node/leaf not |
| **spui runtime semantics** | the UI | 748 resources | typed elements decoded; class defaults, procedures and rendering open |
| **prop schema** | reading a game property by name | 30,889 resources | **keys resolved**: 4,403 model and 306 texture references follow to real resources, across packages, including type-0 keys. Ident names are not recoverable, so roles are measured, not named |
| **73 minor types** | the long tail | 3,190 resources | open |
| **locale type field** | identifying text resources without the package | 4,521 resources | open |

The pixel and geometry paths are implemented, along with prop resolution and static
creature assembly. Remaining work includes deformation and animation, further material
semantics, terrain (blockdata), audio and UI.

### Phase 2: asset browser in Bevy

**Implemented for static assets.** `rustspore --viewer <package>` opens a package and
renders one mesh at a time with its texture. Arrow keys cycle meshes, WASD orbits, `+` and `-` zoom, `T` cycles
the package's textures, `M` switches between the model's own texture and that list, and the
heads-up line names the resource key, vertex and triangle counts, whether normals came from
the file, and the bound texture.

What exists:

1. An asset registry (`crates/spore-bevy/src/assets.rs`): resource key to parsed asset,
   with the package as the source.
2. A texture path: RASTER, PNG and RW4-embedded rasters to `Image`, RGBA8, with a repeat
   sampler because UVs are not confined to 0..1.
3. A mesh path: gmsh, gmdl and RW4 to `Mesh`, with normals and UVs decoded from the format.
4. **A material path.** An RW4 model supplies its own texture, and a prop supplies both a
   model and its colour maps (`--prop-models`, and a prop instance as the last argument to
   `--shot`).
5. A camera controller, a heads-up panel, and a checkerboard mode that makes a broken UV
   map visible.
6. **Per-creature texture assignment.** The viewer binds a shipped atlas when available
   and generates a base colouring from the creature's palette otherwise. This paints the
   surface; it does not deform the mesh.
7. **A static creature assembler.** Each rig row names a part prop and gives its absolute
   rotation and translation. `--creature-info` assembles those parts and
   `--shot ... creature=<instance>` renders them together. Missing source models are
   reported rather than invented.

Animation playback and skeletal deformation remain open in Phase 3.

Cross-package resolution itself is done: `RUSTSPORE_PACKAGES` attaches a set of packages,
`PackageIndex` reads index blocks only, and `--locate` answers whether a reference is
dangling or unresolved.

### Phase 3: creature data

**The rig and palette read, and static creatures render with a texture.** A
`.creaturedata` file holds one 0x8C-byte row per part: a part prop instance, a 3x3 rotation
and a translation. `--creature-info` assembles
those rows into one mesh and `--shot ... creature=<instance>` renders it.

What is measured:

- The rotation is orthonormal on 31,704 of 31,704 rows, so the transform layout is settled.
- Every row's part prop resolves; 88.2% of part models exist in this install, and 773 of
  1,022 creatures have all of their parts.
- Transforms are absolute, not hierarchical, and they reproduce the merged model: over the
  93 creatures that have both, the median mean vertex distance is 2.1% of the model's
  diagonal, worst 10.4%.
- **The skin is assigned, and generated where none ships.** A creature's texture is per
  creature, not per part, and no prop names it: it is a RASTER whose instance shares the
  creature's low half. **366 of 1,022** creatures ship one and the viewer binds it. The
  other 656 carry a palette (nine floats at offset 32 of the decompressed header) and
  `spore_core::skin` rasterises their UVs and paints it: back colour on upward faces,
  second colour underneath, third colour as markings. `--creature-skin` reports and dumps
  the result.

What remains limited or missing:

- **The generated skin is a base colouring, not a reconstruction.** The shipped atlases are
  hand-painted, 169 to 2,312 distinct colours each, and across 40 creatures with both a
  skin and a palette only a median 14 percent of the skin's opaque pixels lie within
  0.08 RGB of a palette colour. Reproducing the painted detail would need the paint
  layers, which are not in this data.
- **Assembled creatures still use rigid placement.** The RW4 animation viewer now
  deforms a real source model on the GPU, and ANIM exposes typed keys. Retargeting
  those channels onto the assembled creature remains open.
- **No creature editor or procedural creature generation.** The assembler reads existing
  authored parts and transforms; it does not create a new creature layout.

### Phase 4: one gameplay stage

Cell is the smallest stage and the right target: a 2D plane, eat and grow, a handful of
part types. Shipping one complete stage proves the engine loop, the input path, the
renderer and the asset pipeline together. The viewer already has camera and browsing
input. The continuation added deterministic Cell and Creature simulations, playable
GPU frontends, controls, food/combat/social progression and versioned saves. Editor
and cross-stage integration plus retail fidelity remain open. This foundation is
not completion of the whole stage; see `docs/functional-milestones.md`.

### Phase 5 onward

Creature stage, Tribe, Civilization, Space, then the editors. Space is effectively a
second project: galaxy generation, star systems, planets, terraforming, missions and the
trading economy.

### What "a full port" means in size

The archive readers, static viewer and a measured RW4 GPU animation path work.
Original Cell, Creature and Tribe gameplay foundations and a creation editor now exist;
Civilization and Space are being developed. A full campaign, source creature retargeting,
retail content/behaviour fidelity, audio and remaining data semantics are still open.
The current per-milestone evidence and unresolved gates are maintained in
`docs/functional-milestones.md`. A faithful full recreation remains a large project;
working prototypes are not completion of those gates.
## 5. Legal and licensing position

- **Assets.** Everything in the `.package` archives is EA's copyright. This project reads
  them; it must never redistribute them. `.gitignore` already excludes `*.package` and
  `assets/extracted/`.
- **Third-party tool code.** `docs/prior-art.md` section 6.2 catalogues the licences. The
  short version: SporeModder FX and S3PI are GPL-3.0 and cannot be copied into a
  permissively licensed project. Several tools have no licence at all, which defaults to
  all rights reserved. `chieltbest/dbpf` is a Rust DBPF implementation and therefore the
  most tempting to reuse, but its licence is an unresolved multi-licence `REUSE` setup and
  must be checked before any copy.
- **What this project did instead.** Every layout in `docs/dbpf-format.md` was derived by
  measuring the shipped data. Public sources were consulted to cross-check and to flag
  conflicts, which is why the version-field conflict in section 2.1 is recorded rather
  than smoothed over. This keeps the format work clean-room.
- **This is not legal advice.** Reverse engineering for interoperability and modding has a
  substantial community history around SPORE and the wider EA back catalogue, but the
  position varies by jurisdiction and no lawyer has reviewed this project.

## 6. How to work on this

```bash
# Read the evidence before changing a parser.
docs/dbpf-format.md      # container format, evidence graded
re/binary-analysis.md    # executable analysis and its limits
docs/prior-art.md        # what public sources say, and where they conflict
docs/resource-types.md   # resource type IDs with sources

# Re-run the analysis that produced the format.
PY=C:/Users/nieuw/.workbuddy-ai/binaries/python/envs/default/Scripts/python.exe
$PY tools/dbpf_fields.py
$PY tools/validate_refpack.py "<a real package>" 20
```

Rules that this project has already found it needs:

1. **Grade every claim.** `[MEASURED]`, `[INFERRED]`, `[UNKNOWN]`. Several conclusions in
   this repository were corrected by measurement after being stated too confidently,
   including the compression flag in `docs/dbpf-format.md` section 4.2 and the shared-code
   claim in `re/binary-analysis.md` section 6.
2. **Report the denominator.** "800 resources validated" only means something alongside
   "out of 101,312 coded resources across 82 packages".
3. **A gap is a result.** The undecoded snr codec, the unknown blockdata record semantics
   and the missing RTTI are recorded as gaps, not omitted.
4. **Do not assume one global layout.** `Spore_Audio2.package` fits the 28-byte record and
   `Spore_Audio1.package` does not, in the same install.

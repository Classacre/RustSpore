# RustSpore

A clean-room reimplementation of the 2008 EA/Maxis game *SPORE*, written from scratch in
Rust on top of the [Bevy](https://bevyengine.org) engine.

RustSpore contains no EA code, no EA assets, and no EA data. It reads game data that you
supply yourself from a legitimate installation of the original game. Nothing proprietary
is redistributed by this repository.

## Status

The acceptance target is a faithful Rust remake using the installed SPORE assets and
recovered behaviour. The current game is not identical to SPORE. Procedural simulations
and the original creation studio are engineering experiments and now require an explicit
`--experimental` prefix. They do not satisfy the remake milestones. Launching without
arguments reports the current status without opening a placeholder game.

Source Cell artwork can now be resolved from the shipped definition, structure, baked
GMDL and its own painted skin. `--source-cell --audit` checks that source chain, and
`--source-cell --cell 7D00E384 --shot target/source-cell.png` produces an offscreen
capture. On Windows this now uses original compiled shaders, untouched vertex
buffers, source normal/specular maps and full mip chains on the RTX. All 28
resolved primary Cell creations pass controlled draws. Source camera framing,
final game draw state, effects and gameplay remain under reconstruction.
Add `--pbr-inspection` for the separate Bevy comparison view.
Saved Creature artwork also follows explicit creation PROP/LOD references:
`--source-creature --creature 40626200067C5390 --shot target/creature.png`
executes original shaders and source mip chains offscreen. The measured fresh
lighting context and original editor thumbnail camera are inspection controls.
All 704 creations with explicit model links pass hidden RTX captures, including
expansion assets. Complete animated creatures and retail scenes remain in progress.
The canonical asset catalog now agrees independently on all 158,720 native keys
in 82 packages. All 704 linked creations also pass strict CPU mesh/material intake
with their own painted maps, including version9 expansion models.
`--shot` static-viewer and animation
captures also run offscreen without creating a desktop window.

The optional [native shader probe](re/shader-execution.md) executes the original
SPORE UI bytecode on the RTX GPU. Both original UI programs pass controlled copy
and alpha-blend checks across all sixteen batch slots with zero RGBA error.
Source layout, image regions and linear fades also have independent checks.
Complete screens, text/events and retail draw-state fidelity remain open. The
same document records the controlled original 3D draws and their remaining gates.

The container format is reverse engineered, the compression codec is validated against
real game data, resource formats have Rust parsers, and the workspace builds and
runs. **A viewer renders real models and textures from a package.** Gameplay development
has begun with a playable Cell foundation. The full game, faithful source animation,
editors and cross-stage campaign remain in progress; see
[`docs/functional-milestones.md`](docs/functional-milestones.md) for acceptance gates.

| Crate | Purpose | State |
| --- | --- | --- |
| `crates/dbpf` | DBPF `.package` container: header, resource index, payload extraction, RefPack codec, index-only reader | implemented, **25 tests pass; native flag-driven keys and compression metadata verified** |
| `crates/spore-core` | Format parsers, mesh conversion, prop and rig resolution, creature skin generation, engine-agnostic core types | implemented, **parser and gameplay checks; see milestone evidence** |
| `crates/spore-bevy` | The `rustspore` binary: package reports, verification modes, and a Bevy asset viewer | implemented, **viewer and gameplay frontends, control/save checks** |

Verified against the shipped game data:

- The canonical DBPF index follows the original flag-driven reader, including shared
  type/scope fields and each record's compression metadata. Native keys are three
  words: instance, type and scope. Independent Python/Rust records agree for all
  158,720 entries. See [the corrected native index evidence](re/archive-index-native.md);
  the older shape-based API remains an explicit compatibility view.
- **All 82 packages in a retail install open**, including the three index variants and the
  single-resource packages. 101,312 coded resources were decompressed and checked against
  the size each index record declares: **101,312 of 101,312 exact**, by the Rust reader
  (`rustspore --verify`), not by a Python prototype.
- **Fourteen formats have Rust parsers.** Per-format validation scopes and denominators
  are documented in `re/payload-formats.md`; field measurements and full-population
  parsing checks establish different things.
- **Every mesh in two packages converts to renderable geometry**: 6,714 in
  `Spore_Graphics` (2,422 gmsh and 4,292 RW4 files carrying all 7,402 of their meshes,
  blend shapes included) and 4,208 gmdl in `Spore_Content`, with zero conversion failures.
  That is 36.4 million vertices and 27.7 million triangles (`rustspore --mesh-selftest`).
- **The pixel decoder matches the Python reference byte for byte** on all 2,954 RASTER
  resources in `Spore_Content` and on all 8,915 textures embedded in RW4 containers in
  `Spore_Graphics`, across DXT1, DXT3, DXT5, `A8R8G8B8` and `L8`.
- **A model can be resolved from the prop that describes it.** In `Spore_Content`, 1,162
  props name a model through a four-slot LOD chain and all 1,162 resolve to a resource in
  the package; 153 of them also name their own colour maps. `rustspore --prop-models`
  prints the counts.
- **Diagnostic rigid assembly reads the source rig.** The `.creaturedata` table holds 0x8C-byte rows,
  each a part prop plus a 3x3 rotation and a translation; the rotation is orthonormal on
  **31,704 of 31,704 rows**, every row's part prop resolves, and 88.2% of part models
  exist in the install. `--creature-info` assembles one and compares its bounds against
  the merged model the game ships for the same instance.
- **The diagnostic painter can generate a palette skin.** 656 of the 1,022
  creatures ship no skin atlas; `spore_core::skin` rasterises their UVs and paints the
  back, belly and markings from the three palette colours. On five creatures that have
  both, the shipped alpha mask lies inside the generated one to two decimals, so the
  rasteriser lands on the artist's islands. `--creature-skin` reports and dumps it.
  These generated skins do not satisfy source-art fidelity. The authored creature
  viewer and original-shader capture load the GMDL's own texture references.
- **References resolve across packages.** `Spore_Graphics` holds 2,613 model props; 2,443
  resolve once type-0 keys are matched on the instance alone, and the other 170 are
  dangling references to instances that exist in no package in the install
  (`rustspore --locate`). A package directory is indexed in about 2 seconds, reading index
  blocks only.

## Building

The workspace targets `x86_64-pc-windows-msvc`. `tools/build.bat` finds Visual Studio with
`vswhere`, picks the newest MSVC toolset and Windows SDK that has a complete x64 CRT, sets
`PATH`, `LIB` and `INCLUDE`, then forwards to `cargo`. It does not call `vcvars64.bat`,
which needs `reg.exe`. If the toolchain is missing it names what is missing instead of
failing later with `LNK1181`.

Reconstruction checks pin source-file bytes. Keep the line endings specified by
`.gitattributes`; `python tools/source_checkout_hash_audit.py --out target/checkout-audit.json`
checks that Git filters preserve them without changing the index or source files.

```
tools\build.bat build --workspace     # about 3 minutes cold
tools\build.bat test --workspace
tools\build.bat run -p spore-bevy -- "C:/path/to/Data/Spore_Game.package"
```

### The viewer

```
target\debug\rustspore.exe --viewer "C:/path/to/Data/Spore_Graphics.package"
```

### Cell gameplay foundation

```powershell
$env:WGPU_ADAPTER_NAME = 'RTX 3060' # optional adapter selection
target\debug\rustspore.exe --experimental --cell
target\debug\rustspore.exe --experimental --cell --diet omnivore --seed 42
target\debug\rustspore.exe --experimental --cell --demo 8 --shot target/cell.png
```

W/S swim, A/D turn, Shift boosts, and Space bites. P pauses, R restarts, and
1/2/3 starts a new herbivore/carnivore/omnivore world. F5 saves and F9 loads.
The default save is `%LOCALAPPDATA%/RustSpore/saves/cell.rsp`; `--save-file` selects
another path. A failed load leaves the active world intact. Saves use a temporary
file and replace the destination after a complete write.

This is original simulation and procedural art. Its tuning is not a reconstruction
of retail gameplay. The growth milestone currently ends at readiness to evolve;
Creature transition and editor integration are still open. `--demo` drives the
same keyboard input mapping with an automated food-seeking controller, captures
after the requested duration, and exits. Screenshot outputs must be new paths.

### Creature gameplay foundation

```powershell
target\debug\rustspore.exe --experimental --creature-game
target\debug\rustspore.exe --experimental --creature-game --demo 20 --shot target/creature.png
```

W/S walk, A/D turn, Shift sprints, E eats/collects/recruits, and Space attacks.
1/2/3 sing/dance/charm to match a target's cue. H rests at home, Tab cycles targets,
arrows orbit, and +/- zoom. P pauses, R restarts, F5 saves, and F9 loads. The
original ruleset has food, health, stamina, parts, alliances, combat, pack followers
and Tribe eligibility. Source creature retargeting and the stage transition remain
open; procedural creatures are clearly a functional baseline.

### Creation editor foundation

```powershell
target\debug\rustspore.exe --experimental --editor creature
target\debug\rustspore.exe --experimental --editor cell --save-file target/my-cell.rsc
```

The creation model supports Cell, Creature, Building, Vehicle and Adventure graphs,
with attachment constraints, mirrored parts, transforms, palette, budget, undo/redo
and bounded versioned saves. Its catalog and traits are original prototype tuning.
Source references remain opaque until their metadata and rendering are established.
Editor interface and gameplay integration are under active validation.

### RW4 skeletal animation

```powershell
target\debug\rustspore.exe --animate "C:/path/to/Data/Spore_Graphics.package" 40606100AD73C528
target\debug\rustspore.exe --animate "C:/path/to/Data/Spore_Graphics.package" 40606100AD73C528 --pose bind --shot target/bind.png
target\debug\rustspore.exe --animate "C:/path/to/Data/Spore_Graphics.package" 40606100AD73C528 --pose 0.25 --shot target/animated.png
```

This path reads one source RW4 on demand, validates its bind pose numerically,
and uploads real joint indices, weights and inverse-bind matrices to GPU skinning.
Space pauses, B toggles bind pose, N selects the next matching skeletal clip,
R restarts and Left/Right orbit. Sampling uses RustSpore linear/slerp interpolation.
Morph tracks, source texture/material reconstruction and assembled-creature
retargeting remain separate gates. Unsupported bindings fail with an error.

### Viewer controls

Left and right cycle through the package's meshes, WASD orbits, `+` and `-` zoom, `T`
cycles the package's textures, `M` switches between the model's own texture and that list,
`R` resets the camera. The heads-up line names the resource key, the
vertex and triangle counts, whether the normals came from the file, and which texture is
bound.

`--shot <package> <out.png> [mesh-index] [checker] [prop-instance]` renders a few frames,
writes a PNG and exits, which is how the pipeline is checked without a person looking at a
window. The word `checker` swaps the bound texture for a generated checkerboard, which is
the only way to tell a working UV map from one that collapses to a single texel. Passing a
prop instance shows the model that prop names, with the prop's own texture.

### Creatures

A creature is not one resource: a `.creaturedata` rig names a part prop per row, and each
part prop names a model. `--creature-info` assembles the rig and compares it against the
merged model when one exists:

```
target\debug\rustspore.exe --creature-info "C:/path/to/Data/Spore_Content.package" 406B62000660EACA
rig   : 4 of 4 parts, 0 missing, 0 failed
        1872 vertices, 2090 triangles
        bounds x 0.156..0.627  y -0.643..-0.039  z 0.173..1.117  radius 0.472
merged: 00E6BCE5:406B62000660EACA, 2119 vertices
        bounds x 0.119..0.615  y -0.654..-0.008  z 0.126..0.934  radius 0.404
```

A creature's skin is not named by any prop: it is a RASTER whose instance shares the
creature's low half and differs in the high half. 366 of the 1,022 creatures in
`Spore_Content` ship one, and the viewer binds it, so a creature renders with its own eyes
and markings.

The other 656 ship no skin. They still carry a **paint palette**: nine floats at offset 32
of the decompressed header, three RGB triples. `spore_core::skin` rasterises the creature's
own UVs and paints them from that palette: the back colour where the surface faces up, the
second colour underneath, and the third as markings on the upper 30 percent of a smooth
wave field. For a creature whose geometry resolves, the viewer uses the shipped skin when
there is one and the generated atlas otherwise. Missing source models still limit which
creatures can be assembled; a palette does not supply missing geometry.

`--creature-skin <package> <instance> [out.png]` reports which path applies, the UV coverage
of the assembled mesh, and how much of the painted surface lands nearest each palette
colour, and writes the atlas as a PNG. `--force` regenerates even when a skin ships, which
is how the generated atlas is compared against the hand-painted one.

What it is not: a reconstruction. The shipped skins are hand-painted, with 169 to 2,312
distinct colours each, and across 40 creatures that have both a skin and a palette only a
median 14 percent of the skin's opaque pixels lie within 0.08 RGB of a palette colour. The
palette gives the hues, not the picture, and the module says so.

`--shot <package> <out.png> 0 "" creature=<instance>` renders the assembly, and
`--creature-overlap` measures it against the merged model by nearest-neighbour distance
between vertices. Over the 93 creatures in `Spore_Content` that have both, the median mean
distance is **2.1% of the model's diagonal** (worst 10.4%), so the rig reproduces the shape
the game ships.

### Verification modes

`--verify` decompresses every coded resource and checks each result against the size the
index declares, exiting non-zero on any failure. It is the real-data test for the RefPack
decoder, which unit tests cannot provide:

```
target\debug\rustspore.exe --verify "C:/path/to/Data/Spore_Game.package"
package          : .../Spore_Game.package
resources        : 11529 (9314 coded, 2215 raw)
coded decompressed to the declared size: 9314 of 9314 (100.0000%)
result           : OK
```

`--report` prints the package summary and exits without opening a window, which is what
CI should use. `--types` prints the full resource type histogram, one `<type> <count>` line
per type, for aggregating across packages with `awk`. `--mesh-selftest` converts every mesh
to renderable geometry and reports failures. `--raster-digest` and `--gmsh-digest` print
per-resource hashes that `tools/raster_digest.py` and `tools/gmsh_digest.py` reproduce from
the Python implementations, so a diff is a byte-exact cross-check.

The animation metadata parser has a whole-install check too:

```powershell
tools\build.bat run -p spore-bevy --example anim_verify -- "C:/path/to/SPORE Collection"
```

On the current install it opens 82 of 82 packages and parses 5,022 of 5,022 animation
resources, exposing 48,700 channels and 878,791 typed key rows. Independent
Python/Rust per-resource digests match exactly. Generic safe sampling is implemented;
native rig binding and authored curves remain unfinished. Malformed CLI identifiers and mesh
indices exit with code 2. Surrounding whitespace, including CRLF, is accepted for IDs.

### Resolving references across packages

A prop's parent, and sometimes its texture, lives in another package. Set
`RUSTSPORE_PACKAGES` to a `;`-separated list of packages or directories and every mode will
resolve references outside the open one. Only index blocks are read, so a whole install
indexes in about two seconds:

```
set "RUSTSPORE_PACKAGES=C:\path\to\SPORE Collection"
target\debug\rustspore.exe --prop-models "C:/path/to/Data/Spore_Graphics.package"
target\debug\rustspore.exe --locate 00000000BE37F695
```

`--locate` takes an instance and an optional type and prints which package holds it, or
says that it is in none. It is how a dangling reference is told apart from a resolver bug.

Running the binary with a package path prints the header, the index record shape and the
resource type histogram, then opens the Bevy window:

```
package            : D:/Games/SPORE Collection/Data/Spore_Game.package
  file size        : 20210108 bytes
  magic            : DBPF
  index entry count: 11529
  index offset     : 19887288 (0x12F74B8)
  index size       : 322820
  index end        : 20210108 vs file size 20210108 => consistent
  record shape     : 28-byte records, 4 byte prefix, 4 byte trailer (Wide28)
  resources        : 11529 (9314 coded, 2215 raw)
```

## Documentation

Read these before changing a parser.

| Document | Contents |
| --- | --- |
| `docs/dbpf-format.md` | The container format, every claim graded `[MEASURED]` / `[INFERRED]` / `[UNKNOWN]` |
| `re/binary-analysis.md` | Executable analysis: sections, imports, Rich header, mitigations, and its limits |
| `re/paint-entrypoints.md` | Focused Ghidra evidence for flood-effect registration and construction paths |
| `docs/roadmap.md` | Honest scope in measured numbers, staged plan, legal position |
| `docs/continuation-2026-10-22.md` | Resumed checkpoint, current checks, REA setup and next animation milestone |
| `docs/prior-art.md` | What public sources say about these formats, and where they conflict |
| `docs/resource-types.md` | Resource type IDs with sources and confidence |

Analysis scripts that produced the above live in `tools/`. They are reproducible and are
the evidence behind the format documents.

## Game data

Extracted game data is expected under `assets/extracted/`, which is gitignored, along with
any `*.package` file. Supply your own.

## Licence

Dual licensed under MIT or Apache-2.0, at your option.

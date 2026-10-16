# SPORE engine architecture, recovered from the executables

Status: measured 2026-10-16 from the shipped binaries.
Evidence grades: `[MEASURED]` read from bytes, `[INFERRED]` reasoned from measurements,
`[UNKNOWN]` not established.

Source data: `re/data/strings-classified.txt`, plus the registries extracted into
`re/data/simulator-classes.txt`, `re/data/argscript-functions.txt`,
`re/data/ea-source-paths.txt`, `re/data/console-commands.txt` and
`re/data/rtti-types.txt`. Extractor: `tools/pe_engine_map.py`.

## 1. Method, and why it is string based

`[MEASURED]` The executables contain only **2 RTTI type names**: `EA::ArgScript::cError`
and `type_info`. A normal MSVC C++ build of this size would emit thousands.

`[INFERRED]` The engine was therefore compiled with RTTI disabled (`/GR-`), which is
common for game engines because it saves binary size and removes virtual dispatch
overhead. This closes the "recover class names from RTTI" avenue definitively: there is
nothing there to recover. `re/binary-analysis.md` section 8 listed RTTI recovery as the
highest-value missing item; this section replaces that with a measured negative result.

`[MEASURED]` The class names are present anyway, as string literals. The game registers
its simulation classes, scripting functions and console commands by name at startup. The
engine also leaks its own build paths through assert messages. String mining therefore
yields what RTTI would have, which is why this document exists.

`[INFERRED]` Caveat on completeness. String-derived registries are biased towards things
that need a name at runtime: classes that are saved, serialised, scripted or debugged.
Pure internal helper classes leave no trace. So 344 simulation classes is a floor, not a
total.

## 2. Middleware inventory

`[MEASURED]` All of the following are named in the binaries themselves.

| Layer | Middleware | Evidence |
|---|---|---|
| Graphics | RenderWare 4 with a DX9 device driver | `RW4w32`, `RWGOBJECTTYPE_*`, and the build path `\Spore\SporeEP1ML\Core\RenderWare\platform\src\graphics\core\device\target\dx9\rwgdrvgraph.cpp` |
| Audio | RenderWare Audio, plus EAPd | `RWAudioCore` (60 hits), `RWAC` (56), `RWAudio3D` (22), `EAPd/*` (56 modules) |
| Physics | Havok **3.1.0** | `Havok-3.1.0`, `hkWorld.cpp`, `hkError.cpp`, `hkStructureLayout.cpp`, `hkMopp*VirtualMachine.cpp` |
| Animation | Cal3D | `Cal3DBoneName`, `Cal3DOrigin` in the ArgScript registry |
| Scripting | EA ArgScript | `EA::ArgScript::cError`, plus 340 registered names |
| Container / base | EA UTFKernel, UTFNetwork, EASTL | `UTFKernel\PPMalloc\source\cmn\EAAllocationRecorder.cpp`, `UTFKernel\MemoryMan\source\CoreAllocatorGeneral.cpp`, `UTFNetwork\UTFSSL\source\SSLManager.cpp`, `UTFKernel\EASTL\include\EASTL/allocator.h` |
| TLS | OpenSSL via UTFSSL | `UTFSSL\source\SSLManager.cpp`, `OpenSSL internal error, assertion failed` |
| PNG | libpng | `png.h` |
| UI / debug tooling | Tcl/Tk | Tcl canvas calls such as `.x%lx.c create line %d %d ... -tags %sR` |
| Platform | Direct3D 9, DirectSound, DirectInput 8 | import table, see `re/binary-analysis.md` |
| Distribution | Steam API | import table |

### 2.1 This is EA's shared framework, not a bespoke engine

`[MEASURED]` The assert paths use relative paths reaching outside the game tree:
`..\..\UTFKernel\...`, `..\..\..\..\Core\Audio\...`.

`[INFERRED]` SPORE is built on **EA's internal cross-title technology stack** (the "UTF"
line: UTFKernel for memory and core services, UTFNetwork for networking, plus a shared
Core/Audio). It is not a one-off Maxis engine. This matters for the reimplementation
because it means SPORE's architecture is representative of other EA titles of the same
era, and public knowledge about that stack transfers.

`[MEASURED]` The build path `spore\CMBuild\SporeEP1\Core\UTFKernel\EASTL\...` shows the
expansion was built from a project named `SporeEP1` through a build directory called
`CMBuild`.

### 2.2 Physics is abstracted behind an interface

`[MEASURED]` The console exposes:

```
[physics mode <havok | generic | no_collision>]
[physics cache <clear | dump | manual | auto>]
[physics stats <reset | dump>]
```

`[INFERRED]` Physics is behind an abstraction with at least three implementations: a
Havok backend, a generic one, and a null one. That is a useful design to copy, and it
means the Rust port can ship a simplified physics backend first and swap in a real one
later without touching gameplay code.

## 3. The RenderWare 4 object model

`[MEASURED]` The binaries name RenderWare's object type constants directly:

- Graphics objects: `RWGOBJECTTYPE_SHADER`, `RWGOBJECTTYPE_VERTEXSHADER`,
  `RWGOBJECTTYPE_PIXELSHADER`, `RWGOBJECTTYPE_SHADERCODE`, `RWGOBJECTTYPE_PALETTE`
- Core objects: `RWCOBJECTTYPE_BBOX`, `RWCOBJECTTYPE_CLUSTEREDMESH`,
  `RWCOBJECTTYPE_KDTREEMAPPEDARRAY`

`[INFERRED]` RenderWare 4 is built on a typed object registry, split into graphics
(GOBJECT) and core (COBJECT) families, with types identified by a numeric id. Assets
declare their type, which is why a separate resource type `0x2F4E681B` exists in the DBPF
index for RW4 content: the DBPF type identifies the container, and the RW object type
identifies what is inside it.

`[INFERRED]` This gives a concrete decoding strategy for resource type `0x2F4E681B`.
Rather than guessing at a byte layout, look for the RW object type id at a known position
in the payload, then dispatch on it. That is the single most valuable lead for Phase 1 of
`docs/roadmap.md`.

`[MEASURED]` `CLUSTEREDMESH` and `KDTREEMAPPEDARRAY` in the core family indicate the mesh
representation is clustered with a KD-tree acceleration structure, which is what a
renderer needs for culling large procedural worlds.

## 4. The simulation object model

`[MEASURED]` **344 distinct simulation class names** are registered, all under a
`Simulator/` or `SP_Simulator/` namespace, most with a `/COM` suffix.

`[MEASURED]` Distribution by leading word:

| Prefix | Count | Stage |
|---|---|---|
| `cMission*` | 23 | cross-stage |
| `cTribe*` | 17 | Tribe |
| `cCity*` | 14 | Civilization |
| `cScenario*` | 12 | cross-stage |
| `cGame*` | 10 | core |
| `cCreature*` | 9 | Creature |
| `cBuilding*` | 7 | Civilization |
| `cSpace*` | 7 | Space |
| `cVehicle*` | 6 | Civilization |
| `cCultural*` | 5 | Civilization |
| `cTurret*` | 5 | Civilization |
| `cCiv*` | 4 | Civilization |

`[INFERRED]` The distribution is a direct map of the game's content model onto the five
stages, with Civilization the heaviest by class count and missions cutting across all of
them. For a reimplementation this is effectively a module list: the class names say which
subsystems exist and roughly how much of each there is.

`[MEASURED]` Examples showing the shape of individual classes:
`cCitySimulator`, `cCombatSimulator`, `cCivMissionManager`, `cCreatureAttackEvent`,
`cBadgeManager`, `cCollectableItems`, `cPlanetGfx`, `cPlanetRecord`,
`cScenarioTerraformEconomy`, `cObstacleManager`, `cTreeData_BackwardsCompatibility`,
`LocomotionStrategy_Template`, `cAbductToolStrategy`, `cArtilleryWeaponStrategy`.

`[INFERRED]` The `c` prefix is the Maxis class convention, visible in the one surviving
RTTI name too (`cError`). The `/COM` suffix on most entries is likely a component or
"common" marker in the registration system. `[UNKNOWN]` Its exact meaning.

`[MEASURED]` Names such as `cTreeData_BackwardsCompatibility` and `ScearioMode` (a
typo in the shipped binary) are useful as fingerprints: they confirm this list came from
SPORE itself rather than a shared EA library.

## 5. Scripting: ArgScript

`[MEASURED]` **340 distinct names** are registered under an `ArgScript/` namespace. The
scripting language is EA's ArgScript, independently confirmed by the surviving RTTI name
`EA::ArgScript::cError`.

`[MEASURED]` The registered names are not general-purpose language builtins. They are
mostly content-authoring parameters:

- Terrain and creature painting: `BrushCond`, `BrushCondRange`, `BrushEffect`,
  `BrushFilter`, `BrushFractalCond`, `BrushGradientCondRange`, `BrushIntensity`,
  `BrushIntensity255`, `BrushLife`, `BrushRate`, `BrushRibbon`, `BrushRotate`,
  `BrushSize`, `BrushSpacing`, `BrushTexture`
- Animation and rigging: `Animation`, `BlendInTime`, `Cal3DBoneName`, `Cal3DOrigin`
- Control flow and structure: `ArrayCreate`, `BoolProp`, `BoolProperty`, `Choice`,
  `Cheat`, `Alert`
- Gameplay: `AllowLocomotion`, `Camera`, `Cap`, `Card`, `Bnd`

`[INFERRED]` ArgScript is the data-authoring and tuning layer for the editors. The brush
family in particular is the sculpting and painting API used by the creature, building and
vehicle editors, and the Cal3D entries bind scripts to skeleton bones. A reimplementation
does not need to reimplement ArgScript as a language first; it needs the parameter set,
because that is what the shipped content data actually references.

`[MEASURED]` There is no Lua and effectively no Python: the string scan found **0 Lua
strings and 1 Python string**. `[INFERRED]` The single Python hit is almost certainly an
incidental word rather than an embedded interpreter. So ArgScript is the only scripting
layer present.

## 6. Console and debug surface

`[MEASURED]` **332 console command candidates** were recovered from usage strings. They
are a functional map of the engine, and several leak gameplay constants:

- `[name_gen <planet | star | tribe | creature | city | building | vehicle | all> (count)]`
  shows the procedural name generator covers those seven categories.
- `-eradicate <row> <column>` documents the planet terraforming grid as
  **3 rows by 6 columns**.
- `pollen -getBatch <type:creature|building|vehicle> <count>` ties to the Pollinator
  asset-sharing backend.
- `editor load | save | rename | validate | dumpGeomInfo | dumpPalettes | timing | zcorp`
  is the editor command set, including `zcorp` for a model complexity score.
- `[creatureanim (filename) [numloops]]`, `[menus <debug | normal>]`.

`[MEASURED]` Model validation is a named pipeline. The binaries contain:
`Model failed the Editable test`, `Loadable`, `Playable`, `Pollinatable`, `Warning`.
`[INFERRED]` Every creation asset is validated against a set of named predicates before
being accepted, which is a clean design worth copying: it gives the editors a shared
notion of a valid asset.

`[MEASURED]` Registry keys include
`HKEY_LOCAL_MACHINE\SOFTWARE\Electronic Arts\EA Games\SPORE(TM)\ergc\` (the EA serial
key location), and `Software\Electronic Arts\SPORE\AppDir` and `...\PlayerDir`.
`[INFERRED]` Save and user data locations are registry-driven, so the Rust port should
not hardcode paths.

## 7. What this means for the Rust port

Concrete consequences, in the order they affect `docs/roadmap.md`.

1. **Do not reimplement RenderWare.** The graphics work is Bevy and wgpu. What has to be
   reproduced is the RW4 *asset* decoding: read the object type, then the object body.
   The `RWGOBJECTTYPE_*` and `RWCOBJECTTYPE_*` constants above are the dispatch table.
2. **Keep physics behind a trait from day one.** The original shipped with three
   backends, and one of them was a null implementation. Start there.
3. **The 344 simulation classes are the module list for `spore-core`.** They tell us the
   subsystem boundaries without needing decompilation.
4. **ArgScript is a data format, not a runtime priority.** The parameter names are what
   the shipped content references.
5. **Cal3D matters for creatures.** Skeletal animation is a third-party format, so
   `Cal3DBoneName` and `Cal3DOrigin` give a concrete entry point for creature rigging
   rather than an open-ended problem.
6. **Middleware that must not be copied.** Havok, RenderWare, Cal3D, EASTL and libpng are
   third-party. Havok and RenderWare are proprietary and closed. Cal3D and libpng are
   permissively licensed but are separate projects. See `docs/prior-art.md` section 6.2
   for the tooling licences.

## 8. Gaps

- **No function-level data.** Class and command names are recovered; the code inside them
  is not. No decompilation, no call graph, no field layouts. Ghidra could not complete
  analysis of a 20 MB binary within its 330 second deadline.
- **No data structure layouts.** Knowing that `cCitySimulator` exists says nothing about
  its members. The class model is a map, not a specification.
- **The `/COM` suffix is unexplained.**
- **String-derived lists are biased** towards runtime-registered things, as noted in
  section 1.
- **The 50284-entry UNCLASSIFIED bucket** in `re/data/strings-classified.txt` has not been
  mined. It is the largest remaining unexamined source in this project.

## 9. Reproduction

```bash
PY=C:/Users/nieuw/.workbuddy-ai/binaries/python/envs/default/Scripts/python.exe
$PY tools/pe_strings.py      # rebuild the classified string inventory
$PY tools/pe_rtti.py         # confirm RTTI is absent, emit rtti-types.txt
$PY tools/pe_engine_map.py   # extract the registries into re/data/
```

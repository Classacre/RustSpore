# Source Cell lighting inputs

`[MEASURED]` Cell Editor Setup `406001003615A30B` names lighting through
`B00DB8C8` and `014140DD`. Native editor handler `5791F0` chooses the former
and replaces it with the latter in mode 1. Native loader `70DAB0` enumerates
namespace `40200100`, confirmed by the saved namespace word at `1536B38`.

The [community property registry](https://raw.githubusercontent.com/Spore-Community/SporeModder-FX/master/reg_property.txt)
cross-checks the names `editorLight`, `editorPaintModeLight` and `shCoeffs` for
these keys and `0100EAC5`. Only interface names are taken from that registry;
the following values and native behavior were measured independently.

| Selection | Source preset | Saved coefficient count |
|---|---|---:|
| Normal editor | `40200100FCED73A2` | 25 RGB values |
| Paint mode | `40200100A9CA81EB` | 25 RGB values |

`[MEASURED]` Both overlay winners in this install have the same saved coefficient
array. They contain signed/HDR floats, which exposed and motivated the float-color
parser correction documented in `prop-numeric.md`. The normal preset also retains
separate direct-light settings. Raw property payloads and coefficient values stay
in the user's packages or ignored `target/source-shaders/` diagnostics.

## Native path and remaining transformation

`[MEASURED]` Reader `70A820` obtains `0100EAC5` as an array, expands RGB values
into four-component records and calls `783780`, then `709640`. The latter can
apply optional source color/scaling/combination properties and accumulates
coefficients in the lighting object. The reader subsequently calls `7834E0`.
These two helpers divide/multiply basis factors, with band-dependent branches.
Their exact operation ordering and optional transforms must be retained.

`[MEASURED]` Runtime `708890` can transform the coefficients when the lighting
frame changes, then binds shader-data `219`. Uniform callback `77E190`, selected
by callback ID `21A`, reads that object's coefficient count byte at `+1` and data
at `+30`. It uploads the requested prefix and supplies missing registers through
a separate initialized fallback buffer. It is not a direct upload of the raw
three-component property array. Its bounded SDK disassembly and REA byte read
are retained as `sh-coeff-sdk.asm` and query 100.

`[MEASURED]` The Rust source Cell path now resolves both preset references and
retains 25 finite float RGB values from each, without changing the diagnostic
PBR lighting. The running RTX offscreen check logs both exact resource keys,
the recovered GMDL declaration masks and native material constants. Its image
matches the previous stable capture exactly before the sampler-state change.
Evidence: `target/source-cell/jawhead-native-inputs-v1.log` and
`native-inputs-verification-v1.json`.

`[UNKNOWN]` Complete basis normalization/rotation, draw-time band order, fallback
buffer initialization, exposure and all original 3D shader uniforms still need
implementation and runtime comparison. These input checks do not close lighting
or gameplay fidelity. The existing viewer's PBR material remains an inspection
surface until the original 3D programs have their complete bindings.

Native query records are in `target/rea/20261022-source-runtime-v3/`, notably
83-100. Query 99 has no recovered Ghidra function at its requested address;
`sh-initialization-sdk.asm` records the independently bounded startup instructions.

## Identity-frame coefficient prefix, 2026-10-23

`[MEASURED]` `shader_binding::identity_sh_prefix` implements the plain source
coefficient route for the first sixteen entries: RGB expands with W=1, native
`783780` applies a rounded float32 reciprocal/multiply, `709640` adds into
an empty accumulator, `7834E0` multiplies the basis factors, and `70A820`
finally multiplies all components by exposure `0100EAC4`. Skipping the
undo/redo changes some float bits, including W, so the steps remain separate.

Bounded SDK `sh-factors-sdk.asm` and REA reads 156-162 establish the startup
factor formulas and the stored float32 PI. `sh-undo-rounding-sdk.asm` confirms
DIVSS/MULPS rather than an unrounded real-number division. A Decimal-100
synthetic golden calculation covers all sixteen factors with signed/HDR
inputs, plus native zero-accumulation sign behavior. Unsupported prefixes and
nonfinite values fail explicitly.

`[MEASURED]` Normal editor lighting has cameraSpaceLighting=false and no
optional SH color/combination fields. The original-program renderer requires
that measured property set and an explicit identity lighting frame. It reads
source coefficients/exposure at runtime; no source array is embedded. Native
`708890` negates a source directional-light vector into its runtime light;
callback `779140` negates it again for dirLightsWorld. The controlled draw
retains both operations' resulting direction, correcting the initial probe's
single-negation input. The absent `20C` tint callback uses white, confirmed by
SDK instructions and the sixteen bytes at `153AFF8`.

These bindings pass the original shader draw for all 28 resolved primary Cell
models. Full frame rotation, optional combinations, all 25 coefficients,
capture-time camera/lighting state and final retail pixel comparison remain
open. The implementation only exposes the independently checked prefix and
must not be applied to other presets while silently dropping their fields.

## Fresh-world uniform 219, 2026-10-23

`[MEASURED]` Registrar `77E540` binds callback `779410` for uniform `219`.
That callback reads registered object pointer `16F7E0C` and uploads its vector
at `+4`. Runtime `708890` registers the lighting world's shader object at
world `+260`, so this vector occupies world `+264`, `+268`, `+26C`, `+270`.
It is separate from the coefficient array at world `+290`.

`[MEASURED]` Native world constructor `70CC10..70D008` installs vtable `140C708`
and explicitly stores XORPS-produced positive zero in all four vector words at
`70CE26`, `70CE2E`, `70CE36`, `70CE3E`. Preset constructor `706770..706A64`
initializes optional-lighting scalar `+D0` and VECTOR2 `+124/+128` to positive
zero at `706944`, `706984`, `70698C`. Reader `70A820` first constructs a default
preset and assigns it through `7082F0`; stores `7083BF`, `7084BB`, `7084C7`
retain those defaults in the destination before optional properties are read.

The exact property and producer paths are:

| Input | Native read/write | Effect |
| --- | --- | --- |
| `02478ED7`, scalar float type `0D` | `70B5BD..70B5E3` to preset `+D0` | `7092B8..7092C7` enables world `+424` only for a value greater than zero |
| `049B94D5`, VECTOR2 type `30` | `70B7FC..70B809`, getter `6A0B60` to preset `+124/+128` | `709380..709386` copies the first component to uniform `219.w` |
| World optional-light branch | Guards `709FA7..709FAE` and `70A024..70A02B` | `70A461..70A46D` copies world `+240/+244/+248` into `219.xyz` only on that branch |

`[MEASURED]` `6A0B60` is a VECTOR2 getter, correcting the initial scalar-getter
interpretation. Instructions `6A0B81..6A0B98` require type `30` and copy two
words. A missing property returns without modifying the destination. The full
declared property lists of all four installed normal/paint preset copies lack
both `02478ED7` and `049B94D5`; none carries a parent property. Their source
hashes and complete property IDs are retained in
`target/source-shaders/uniform-219-source-presets-v1.json`.

`[MEASURED]` These constructor, getter and branch instructions establish
`219 = [+0,+0,+0,+0]` for a **fresh world using the checked plain preset**.
This is a static derivation from native state transitions. Rebinding a world
that previously used optional lighting can retain its earlier RGB; a copied
per-object shader object can have additional producers. The renderer therefore
keeps the fresh-world contract and rejects unsupported source fields. Zero is
not a universal replacement for unresolved `219` state.

Source-pinned Capstone evidence is
`target/source-shaders/uniform-219-native-trace-v1.json`, ten bounded ranges,
report SHA256 `5ad0c13a3cf41e334571470e9056bc8c8afeb34527c112ae71798ad7b6641aa9`.
The original executable SHA256 remains
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
REA query 455 establishes registration, query 85 the world binding, and query
95 the preset constructor; the additional exact stores and control flow come
from the original instruction bytes. No original-game runtime was launched
for this trace, and all proprietary instructions remain under ignored `target/`.

`[MEASURED]` The resulting controlled original-program draw succeeds for saved
creature GMDL `40627100067C5390` on the RTX 3060: 6,967 untouched source vertices,
11,070 triangles and 48,105 non-background pixels. It uploads the source skin's
ten mip levels and the second map's nine levels, using the original programs.
Evidence is `target/animation/creature-native-static-v2.png` and its log. The
identity light frame, inspection camera fit and static pose remain explicit;
this does not establish animated creature or retail scene pixel equivalence.

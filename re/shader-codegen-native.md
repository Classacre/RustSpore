# Original vertex-fragment generation

`[MEASURED]` Base executable SHA256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`
contains the vertex source builder at `6FE4D0`. Native v3 queries510..517,
521,525 and526 establish its source assembly, metadata intake, uniform lookup
and builder header. The independent Rust implementation is
`crates/spore-core/src/shader_codegen.rs`.

| Original address | Observed operation |
|---|---|
| `6FD730` | Input structure fields and their source declaration order |
| `6FD8F0` | Intermediate structure and varying widths |
| `6FDA90` | Output structure and semantics |
| `6FDB60` | Input to intermediate assignments |
| `6FDD10` | Intermediate to output assignments |
| `6FE400` | Uniforms and additional declarations |
| `6FDEF0` | First exact declaration-string match for uniform deduplication |
| `6FD560` | Placeholder substitution |
| `6FF150` | Vertex fragment metadata reader |
| `7776F0` | Seven-word shader/builder header reader |

`[MEASURED]` Fragment IDs are one-based vertex slots. The metadata carries
input/output masks, varying count/width and a used-constant count. Uniform
registers advance by the descriptor's third big-endian ushort. Duplicate
declaration strings retain the first descriptor and register. Source fragments
and every original HLSL template are read from the installed packages and
executable. No original HLSL is embedded in the Rust module.

`[MEASURED]` The native placeholder helper recognizes an ASCII letter and
one digit between angle brackets. Only the vertex caller's lowercase `t`
receives a varying offset. Its literal greater-than-ten branch gives a colon
for exactly ten; the independent implementation preserves this behavior.
The bounded API rejects more than eight varyings and recipes exceeding the
native fixed key size instead of overrunning storage.

`[MEASURED]` The installed Microsoft `System32/d3dx9_27.dll`, SHA256
`f9dd288c9895973f8db1856d172779041c6dee173ad1ef53b1727fc85cb6b75f`,
successfully compiles all955 checked recipes under `vs_3_0`, flags`12`.
Of these,954 unique recipes have original cached programs in the profile3
source pack. All954 match the original uniform descriptors/registers and
every executable instruction token exactly. D3D comment tokens are excluded
only from comparison; actual compiled streams are uploaded intact.

Evidence is `target/source-shaders/rust-codegen-all-v1/comparison.json` and
`results.tsv`, SHA256
`0bf3ed8d791381937601fb847c19d4e0375c1efc213a6b2897b8728c2bb26de9`.
The two initially investigated recipes also produce HLSL byte-identical to
the independently written local Python source-assembly probe.

```powershell
tools/build.bat run -p spore-bevy --example shader_codegen_verify -- 'D:/Games/SPORE Collection' target/source-shaders/new-codegen-check --all
```

`[MEASURED]` The default native static renderer compiles an absent vertex
recipe from its actual selected fragments. It checks the original executable
identity, uses the original entrypoint and applies the builder flags. The
source model `40667100D1B4BB56` needs previously uncached recipe
`14 17 18 15 31 30 2B 01`; its3795-byte source compiles to1268 bytes.
This original flora model now draws2201 vertices/1854 triangles using its
own source maps on the RTX. `creature-main-native-second-v3.png/log` retain
that controlled draw. The general CreatureData resource format also contains
flora; its population must not be described as a Creature-stage species count.

`[MEASURED]` Pixel uniform callback `779450`, registered as`21B`, computes
distance from the current world translation to an override camera position,
or current camera translation if that pointer is absent. Differences, squares,
the Y/Z sum followed by X and square root remain in x87 until one float32
store. Its register is `[distance,0,0,1]`. The new independent helper accepts an
explicit arithmetic context. The inspection renderer supplies its own model
translation/camera and explicitly logs CW`027F`; this is not retail state.
The source pixel material uses that distance for its own specular fade.

`[MEASURED]` Creature inspection now reads thumbnail camera key`B0D43C20`
through `B02D871C` in the original Creature Editor Setup`CF2DC8D0`.
Both Creature and Cell thumbnail cameras inherit `3F3B2714`, whose actual
class is`00EDD3BB`; the measured orbit/projection semantics therefore apply
to both. This replaces the earlier explicit Cell-camera inspection of the
Creature model. The new actual-source draw is
`target/animation/creature-main-native-creature-camera-v1.png/log`.
The read-only census covers82 indexes/189 camera and setup records with no
framing failures, retained as `creation-camera-census-v1.json`.

`[MEASURED]` The all-installed-package renderer passes all 704 creations with
explicit model links in `target/source-creatures/native-corpus-v2/report.json`.
Every capture runs hidden on the RTX, with original geometry, texture chains
and actual selected programs. The corpus totals 7,804,809 triangles, two
distinct vertex programs and four distinct pixel programs. The uncached
vertex recipe is compiled in 346 processes. The tested executable SHA256 is
`4af9dee47dac44b1aef0d4d56b7ea1fcc2a9e478f2328b26518bbb75b5e7f2b8`.
The prior four-base-package v1 corpus passed 361 and failed 343 on missing
expansion models; retain it as the failed reader baseline. The new package
discovery searches all 82 installed indexes under explicit inspection
priority. Actual loaded-product/retail priority is a separate open question.
Jawhead's fresh all-package regression remains byte-identical over all
589,824 RGBA pixels. Its PNG SHA256 is
`b06a039f1c3d8ff838a699ba991cc480a9cb973ac66059a51d94d074f96ebfad`.

```powershell
python tools/source_creature_native_corpus.py 'D:/Games/SPORE Collection' target/animation/creature-model-links-python-v1.json target/debug/rustspore.exe --out target/source-creatures/new-corpus
```

The reusable camera census is `tools/creation_camera_probe.py`; its v2 output
agrees on every v1 camera/setup record. Static inspection does not establish
the original animated pose, retail scene lighting or gameplay.

`[UNKNOWN]` Missing pixel recipes, other shader profiles and every flags`32`
permutation have not received this954-recipe conformance comparison. Complete
retail object/camera anchoring, scene draw state, animation, effects, editors
and gameplay are separate open gates. Source program generation and a fitted
static draw do not close the original-fidelity roadmap.

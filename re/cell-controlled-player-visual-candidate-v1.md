# Controlled Cell player visual candidate

This is a source-data join for the existing controlled seed-1 inspection, not an observed retail player, provider winner, rendered frame, or gameplay-equivalence claim. The retained `target/source-cell/controlled-source-player-run-v2/run-0.stdout` reports triple `E219E5D3:3D97A8E4:40616200`; its seed, fresh cache, disk-row and no-intervening-RNG-mutation admissions are documented in [the player producer trace](cell-player-actor-pick-producers-static-v1.md). A later saved-state override can change this triple.

`E67A50` is the separate capability-cache path, not a drawable-model loader: it searches by the first word, and on a miss calls `E83E60` then `E51EC0` with flag zero. A cached result may have been populated from a different full triple sharing that first word. See [the geometry boundary](cell-actor-geometry-consumer-static-v1.md) and [zero-flag bridge](cell-capability-zero-flag-source-bridge-v1.md). The conditional visual route instead passes the actor triple through `E6D950 -> E5CCE0 -> E4EE10` into the animation world; `4BC760` first asks for `(first, 0F43029A, third)` on the documented non-special branch. `A0B110` and `9C7ED0` can then reach model-world virtual +0C. The model-world path reads explicit LOD properties; the full gates and target identities are documented in [the animation ownership trace](cell-animation-owner-palette-draw-static-v1.md) and [model loader trace](creature-model-links-native.md). Neither route establishes runtime cache, provider, scheduling or actor identity.

For a bounded *explicit package* join, fresh read-only decoding found:

| Input | Native key `(instance:type:scope)` | Decoded SHA-256 |
| --- | --- | --- |
| Spore_Content CreatureData | `E219E5D3:0F43029A:40616200` | `dbbb55830f7f75eb5c45afd470a8bd0716cb08134e26f62a82674e28cd697137` (existing controlled inspection pin) |
| PatchData PROP | `E219E5D3:00B1B104:40616200` | `34f7aa2f8f8e57b14c7e25f60d6474d13d5e77cb8fe449edd9688a151358db8d` |
| Spore_Content PROP | same key | `563e75d585b589631dba74a5a6bfa154140cf583cad0caff4ddc7637f619eba0` |
| Spore_Content GMDL | `E219E5D3:00E6BCE5:40617100` | `95ba0f5e5e71761ccf0f2ef919cce9bcc8d40303be4b1b7a9fa954a7c7cb8ff4` |
| Spore_Content raster | `E219E5D3:2F4E681C:40612900` | `5c7889ac480c635daca5b27f7ae38ee0fc4fac5fc47b9040809f481354c5f89c` |

Both PROP copies decode `00F9EFBB` to the exact GMDL key above; `00F9EFBC` and `00F9EFBD` name GMDLs at scopes `40616201` and `40616202`. The copies differ as whole payloads, so equality of these references does not identify the runtime PROP. The selected GMDL has one mesh, 2,555 vertices and 4,244 triangles according to the existing Python framing readers. Its material field `20D` has sampler-zero instance `40612900E219E5D3`, matching the raster key above, with state bytes `0f03020201521400`. This is an explicit saved material binding, not a same-low-ID texture guess. Parsing the saved input does not establish that a runtime material, draw, pose or source shader uses it.

The two installed package hashes before and after this scan were PatchData `cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff` and Spore_Content `0b849bd79d9c13dec5737159655bf7e2b7763cc2b79949c81370fa0490256e8c`. Before/after checks do not prevent transient file replacement. Native key parsing used `tools/cell_kind4_list_source_verify_v1.py` and `tools/native_archive_index_probe.py`; RefPack and PROP decoding used `read_payload` and `prop_gmdl_refs` in `tools/cell_kind4_candidate_asset_audit_v1.py`; GMDL counts and sampler came from `geometry_counts` and `source_sampler` in `tools/authored_creature_asset_probe.py`. Reproduce without writing assets by importing those functions from `tools`, selecting the exact keys above from `native_rows(Path(...))`, applying `read_payload` to each row, SHA-256 hashing decoded bytes, and calling `prop_gmdl_refs` or `geometry_counts` followed by `source_sampler` for the GMDL. All package keys must be selected exactly and duplicate keys rejected in a production implementation.

The practical next bounded integration is to extend the staged inspection with a caller-admitted package choice and exact decoded PROP/GMDL/raster identities, while preserving its unresolved status. Before publishing an actor, independently establish runtime provider precedence, full-triple/cache identity and lifetime, component creation and model-world success, texture and material behavior, animation binding, and actor registration. A source-mesh preview alone does not meet those gates.

## Opt-in prototype startup art

The app can attach the seed-1, progress-0 staged candidate's static art to its
prototype Cell player with explicit stable-package admission:

```powershell
target\debug\rustspore.exe --experimental --cell --seed 1 --source-startup-install 'D:\Games\SPORE Collection' --admit-startup-stable-packages --offscreen --demo 1 --shot target\startup-art-new.png
```

It admits only the exact staged triple and four unique decoded content-package
resources listed above, verifies their hashes before and after loading, and
disables generic package fallback. Files remaining unchanged throughout the
reads is an explicit admission, not an atomic snapshot guarantee. The player
identity, saved-source override, retail animation, gameplay, camera and lighting
remain unverified. Cell prototype save/load remains available.

The hidden offscreen capture `target/startup-art-capture-v1/receipt.json` exited
zero on the reported NVIDIA GeForce RTX 3060 Laptop GPU (Vulkan) and logged
all four exact keys and 60 prototype ticks. `startup.png` SHA-256 is
`6ffdfe9e3a5e219f5d23baa82c1396941782ca69ac2be96d24ebd60e2621e2a2`.
The visibly small, dark candidate is in the wide prototype scene, not a matched
retail frame. A second capture at `target/startup-art-capture-v2` used the same
command except its output path, reported the same adapter and byte-identical
PNG. Its diagnostic measured longest 3D extent 1.271509, projected screen
width 1.030625, height 0.555560 and depth 1.271509 before normalization.
The existing 2/longest-extent scale therefore gives a projected silhouette
about 81% as wide and 44% as high as that normalized longest dimension, before
prototype radius and the 76-unit viewport. This accounts for part of its small
appearance; the cause of its darkness has not been established. The directional
light already belongs to the source camera's render layer. No speculative
retail camera, size or lighting correction was applied.


## Dark-appearance evidence

The separately retained original-D3D9 inspection image
`target/controlled-player-visual-amd-20261105-v1.png` also shows a very dark
body for this hash-bound candidate. Its renderer uses a fitted editor inspection
camera and fresh lighting; it is a different view and not a retail scene or a
pixel-matched comparison with the prototype screenshot. Its log is
`target/controlled-player-visual-amd-20261105-v1.log`.

A read-only decode of the exact Spore_Content sampler-zero raster
`E219E5D3:2F4E681C:40612900`, decoded payload SHA-256
`5c7889ac480c635daca5b27f7ae38ee0fc4fac5fc47b9040809f481354c5f89c`,
produced a 512×512 DXT5 base level. Across all 262,144 atlas texels, mean
RGB was 42.85/41.46/41.72 out of 255, mean alpha was 27.40, and 32.27%
had all RGB components below 32. These are whole-atlas statistics, including
unused/transparent regions, not UV-weighted visible-surface statistics. They
make dark source texture/material a plausible contributor but cannot attribute
the prototype's appearance to texture, lighting, shader or pose, and do not
establish retail appearance.

## Controlled pre-draw CPU trace

The independently reviewed opt-in trace was run with a separate untraced
control under private, non-switched desktops. Both processes exited zero and
both desktops closed after the children were reaped. The PNG files were
byte-identical (SHA-256 `ed33c7e33072673d491b7bcca222869a1c7c4a70d0414468fb3ea83d3e14fd55`).
The selected D3D9 adapter was NVIDIA GeForce RTX 3060 Laptop GPU (vendor
`10DE`, HAL, hardware vertex processing). Receipt
`target/startup-render-trace-result-v1/receipt.json` SHA-256 is
`bb5f3589fc02f2787919c1a62fbe76548d96a7dc33d552d6848903486210f54f`;
the raw `trace.stdout.bin` in the same directory has SHA-256
`829cc2026ce52b8b557a561a300c937601365e95c8501f81a4a78a38b5edca94`.
Executable SHA-256 was `b0163b9d9e23df79f7d7b1c9c3baec3e6e433cac702524e0494d360857e2730c`.

At the pre-draw CPU boundary the trace recorded a 2,555-vertex mesh, 36-byte
stride and 12,732 32-bit indices. Vertex bytes hashed to
`52304dc47a2ba1097fef45ef21c315aab36e677a7b3883ce8c744e3d5240dcb4`,
little-endian indices to `6891da4e135b3dbd32426d96b663448f6978427df99006d95dee06ea150a2fad`,
selected vertex bytecode to `9457add04b2a59b3f82832a048ac40523a80ecaf64efa8a7eb4ccbac5f95012a`,
and pixel bytecode to `89fb1e87bc7333fdfd6d229558807c6479bde472cff6aebac143f54e6d4d48e3`.
The raw log contains each selected callback, register and exact float-bit
values, including model/clip and lighting inputs; it also identifies the three
texture resource keys and inspection camera and light. Package stability was
explicitly admitted by the caller. This is not a GPU input readback: texture
bytes, declaration and compiler identity were not captured, provider selection
was the inspection ordering, and neither identical PNGs nor these hashes prove
retail scene, live actor, or cross-renderer parity.

The separate v2 trace additionally retained all 26 source-format CPU mip
payloads as hex in `target/startup-render-trace-result-v2/trace.stdout.bin`;
the runner decoded them and verified each recorded SHA-256. These are the
per-level source bytes supplied for upload, including compressed DXT blocks,
not decoded RGBA or GPU readback. It recorded seven declaration rows including
the terminator, immediately before D3D declaration creation. For each of the
three samplers the selected address, min/mag and mip filter values were 3, 2,
and 2. Provider and compiler identity remained unresolved, and package
stability was caller-admitted. The two separate private-desktop captures both
exited zero and were byte-identical; their PNG SHA-256 was again
`ed33c7e33072673d491b7bcca222869a1c7c4a70d0414468fb3ea83d3e14fd55`.
The v2 receipt SHA-256 is
`27f61770d4c084a1ed01cfce19ae2375f0bdcc0b67bab9c3e7d45bdceb95b11d`,
the full raw trace SHA-256 is
`5175a0c071923ec3bb8401ae054836bfe4e1197c378e434a0bbc34a110b86c69`,
and the executed binary SHA-256 is
`99ad42a589272b0eb16b38b45f124097351f8c78afe769e780554feadf0ef6b3`.
The selected adapter was NVIDIA GeForce RTX 3060 Laptop GPU, D3D9 HAL with
hardware vertex processing. These are controlled inspection inputs, not proof
of submitted GPU state, a live actor, retail lighting, or renderer parity.

The independently reviewed v3 trace additionally recorded nine first-hit
renderer resource reads and SHA-256 hashes of their returned payloads, under
the same caller-admitted stable-package assumption. The exact observations are
in `target/startup-render-trace-result-v3/trace.stdout.bin` (SHA-256
`0475ae7b2ec34ffee4e73ebf239a7b3a6b22d9fe8a2e1286f2c01acb848c0180`).
The model and both candidate RASTERs were read from `Spore_Content.package`;
the profile-3 shader pack, editor light, editor setup and selected camera from
`PatchData.package`; the camera parent and RW4 fallback for sampler 2 from
`Spore_Graphics.package`. The raw trace gives all nine exact keys, payload
sizes and hashes. This was the renderer's inspection first-hit ordering; the
runner did not assert a complete expected provider inventory, independently
prove stable files, or establish retail precedence or compiler identity.

Both private-desktop processes exited zero, both desktops closed after child
reaping, all 26 mip hashes and seven declaration rows passed, and the control
and traced PNGs were byte-identical with SHA-256
`ed33c7e33072673d491b7bcca222869a1c7c4a70d0414468fb3ea83d3e14fd55`.
The receipt SHA-256 is
`c0016736f29b29516a1624a0b14be8aab145213bb7403604a99807890cc0d3c6`,
and the executable SHA-256 is
`481c991788488f6696e32092ca43b1dfe2471380719bba1085dfb9e38bafd0e9`.
These observations do not establish GPU readback, actual live source state,
or cross-renderer equivalence.

An independent read-only inventory audit subsequently reconstructed the
inspection's ordered 82-package list, parsed their flag-driven indexes and
matched all nine observed first-hit paths and decoded payload hashes. It also
found no standalone sampler-2 RASTER key in that inventory, supporting the
observed RW4 fallback. Full package hashes and inventory membership matched
before and after the audit; these snapshots cannot rule out transient changes.
The validator uses the independent index probe under normal (non-optimized)
Python, whose assertions perform required checks. The separate
selected-pack-fragment path was not among the nine observed first-hit rows.
Receipt `target/startup-provider-inventory-result-v1/receipt.json` SHA-256 is
`8a9b7560fbbc88e40fc8274c9da7812b8af982190423c0847b3ebf47bf13b56d`;
the ordered inventory and full-file-hash manifest SHA-256 is
`5ccf80025c1745f5a2a4f528053e3c7298dd151bc767dd1bc9496c928cbfb90e`.
This independently reconstructed order is not a recorded original traversal
and does not establish retail precedence or live provider identity.

The independently reviewed v4 inspection recorded both selected shader stages
as `pack_cached`; neither invoked D3DX compilation in this run. Both hidden
control and trace processes exited zero, their screenshots were byte-identical
(SHA-256 `ed33c7e33072673d491b7bcca222869a1c7c4a70d0414468fb3ea83d3e14fd55`),
and the existing mip, declaration and nine provider observations passed.
The receipt SHA-256 is
`c6eec8bf50a83326f345e6aa52a0ad50539ef588bb5c874b76f4f7482c309ec0`,
raw trace SHA-256 is
`2138da8f6f60324d22a25bcf10a837ac88acaab522f48ae2f288291fe54b4292`,
and executable SHA-256 is
`918d9d31fc24c7facf7156e611a4dbdad6633831b5cb804220104c79c4416475`.
The selected adapter was NVIDIA RTX 3060 Laptop GPU, D3D9 HAL with hardware
vertex processing. The generated-compiler tracing branch was not exercised:
its module path, disk hash and template/HLSL capture remain unvalidated, and
its version and mapped-image equivalence remain unverified. These results do
not establish retail compiler identity, live state or renderer parity.

An independently reviewed CPU-only geometry inspection compared the pinned
candidate model's raw one-mesh data to the current `MeshData` conversion. The
source vertex and little-endian index bytes matched the retained v4 pre-draw
hashes. All 2,555 untransformed positions and UVs matched the corresponding
packed source bits, and all 12,732 indices matched; the run reported zero UV
or total sanitizations. Decoded normals were checked for finiteness only, not
against the source normal-unpack shader. The receipt at
`target/cell-startup-geometry-result-v1/receipt.json` has SHA-256
`b23b408f6749c077557ab40befc06fa63b734a01b90ec1b1fe31a688afb35d95`.
The independently pinned source and binary do not by themselves establish
build provenance. Package stability is admitted and this untransformed static
comparison does not validate camera or basis conversion, animation, native
normal shader, lighting or retail appearance.

The exact cached 3,196-byte inspection VS was subsequently extracted from the
pinned pack and matched the retained pre-draw program SHA-256
`9457add04b2a59b3f82832a048ac40523a80ecaf64efa8a7eb4ccbac5f95012a`.
The reviewed D3DX disassembly-only run retained raw text SHA-256
`f06d244918d005ca0855c002283581e88e452db00e384d013fa5d0177e9d5884`
and receipt SHA-256
`eed7e5ff583cd629a5dc4186131ffafc264a8cc57c83b36dff73cabbec4bd2ea`.
With the recorded UBYTE4 declaration, the disassembly unpacks NORMAL0 with
`mad` using binary32 coefficient `0x3c010204` (approximately 1/127) and
offset −1, applies the model-to-world 3×3, then executes `nrm`. The CPU mesh
decoder instead calculates each component as `byte/127.5−1` in f64, casts to
f32, then normalizes before Bevy's transforms, with a near-zero fallback.
These are different formulas and operation orders. No GPU intermediate was
measured, and floating-point implementation, zero cases, actual game pose,
lighting and visual consequences remain unresolved.

An independently reviewed diagnostic ran the unmodified captured VS on the
RTX 3060 D3D9 HAL with synthetic point positions, identity world/clip matrices,
a diagnostic pixel shader and a float target. The runner checked all 2,561
expected pixels for finite output and alpha 1, and all other pixels for alpha
0. These comprised 2,555 packed candidate normals and six synthetic controls;
zero source-unpacked normals were excluded. The maximum difference from the
runner's source-formula reference was `4.74e-7`. The raw float target SHA-256
was `43e8241904643e9b21d74ac39d8a74c99a18ff4feba281abc3f551625f29a975`,
and the GPU-run receipt SHA-256 was
`c2f21c711e2d0484c788d4d79bdda60dc7e1c1ca2bab4276ffbd9ba8a25b58e9`.

Separate CPU analysis compared those floats with a Python double-precision
approximation of the CPU formula; it is not the exact Rust f32 computation.
For the 2,555 candidate vertices, mean/median/p95/maximum per-vertex maximum
component differences were approximately 0.004181/0.004391/0.005283/0.005419.
The larger 0.57735 maximum belongs to the synthetic `[127,127,128]` control,
not a candidate vertex. The analysis receipt SHA-256 is
`90615813c99a180b744b07de4eb8ecea932ca4b898224a60f8e0ccd588af27c6`.
This is an input-normal diagnostic, not a lighting or visual-impact result.

A separately reviewed CPU-only comparison then used the actual Rust
`MeshData` normals rather than the Python double approximation, for the
2,555 candidate vertices only. Mean/median/p95/maximum per-vertex maximum
component differences against the retained identity-transform GPU floats were
`0.004181073870`, `0.004390954971`, `0.005282819271`, and `0.005419194698`;
none exceeded 0.01. Synthetic controls are excluded. The receipt at
`target/cell-startup-normal-rust-result-v1/receipt.json` has SHA-256
`79b07834ee281cfd120930df3798684cdd94c1c4d43bdf0d289c2736b93a35fa`.
It relies on the earlier background coverage check and separately pinned
source/binary do not alone establish build provenance. These figures do not
establish a visible lighting difference.

The external precheck mistakenly targeted a nonexistent `host.exe`; its
PowerShell errors were nonterminating, so it was not a successful external
pre-pin. The reviewed runner itself checked the actual nested host hash before
and after execution, and the actual host hash was separately checked afterward.
The source and executable pins do not alone establish build provenance. No
retail scene, live actor, animation or visual causation is established.

A separate reviewed v3 GPU run kept the captured VS unchanged but supplied a
synthetic world transform mapping `(x,y,z)` to `(y,z,x)` with positive uniform
scale 2; the clip layout and pixel shader remained diagnostic. Coverage and
finite checks passed for 2,555 candidate normals and six separate controls.
The candidate-only Rust `MeshData` comparison reported mean maximum-component
delta `0.004181073667`, p95 `0.005282819271`, and maximum `0.005419135094`;
the combined 0.57735 worst remained a synthetic control. The GPU result's
maximum error against the source-formula reference was `4.7312e-7`. The
receipt SHA-256 is
`cedd09141140949802666f3acbefea6572548b429a204d6aa04e6c88889c43ab`,
and the float target SHA-256 is
`24665be3c6c6fa74cb13acd867910b4f38380567b37615365610c6df017e539b`.
This tests one explicit rotation and scale, not arbitrary Bevy transforms,
retail pose, lighting, or visual impact.

The prototype now offers a separate `--startup-source-normals` option, only
with the admitted `--source-startup-install` seed-1 path. It verifies the exact
model, packed vertex/index hashes, declaration and full decoded mesh before
replacing its normals with a finite, nondegenerate CPU approximation of the
source formula. It does not alter other source modes or claim GPU shader,
lighting or retail parity. The original startup mode remains available when the
option is omitted.

The paired windowless RTX 3060 Vulkan run at
`target/startup-source-normal-capture-result-v1` used the same executable,
seed 1, demo duration and tick-60 simulation for both modes. Of 1280×800
pixels, 52 differed by at most one channel level, inside inclusive bounding box
`[666,400,682,409]`. Baseline PNG SHA-256 was
`6ffdfe9e3a5e219f5d23baa82c1396941782ca69ac2be96d24ebd60e2621e2a2`;
source-normal PNG SHA-256 was
`993603d267aa7778783e4bd6cf5a194a9c2aec264a10d7ca10d545aef5a2ee73`.
The receipt SHA-256 was
`5a806a099080fd0884ef6c33c463e38fe02ae87dd6fe8a15525fa2016c39e306`.
Both stderr logs reported the unavailable Vulkan validation layer, duplicate
OBS hook and older overlay API versions; no pipeline error appeared in either
retained log. The validation layer was unavailable, so this is not a
validation-layer-clean claim. This result does not establish visible likeness
improvement or retail appearance.

The independently reviewed lighting-free sampler fixture used 16 synthetic
texel-center coordinates into this candidate's captured BC3 level zero, with
nearest filtering, explicit LOD 0 and clamp. RTX D3D9 with sRGB read disabled
and RTX Vulkan/WGPU BC3 unorm samples differed by at most `5.87e-8` per
component; WGPU unorm versus sRGB RGB controls differed by as much as `0.28731`.
Alpha matched between WGPU controls. The D3D9 shader's alpha remapping was a
coverage marker, reversed for the comparison. Receipt
`target/startup-sampler-fixture-result-v1/receipt.json` SHA-256 is
`c69843a099a5c47b507a521b5926e00faf7bb3f90853f893dd4c5cdc8b2ac876`;
raw D3D and WGPU SHA-256s are
`9e9d4b21c5fcffbac6852538d4ae311c0ce2dae2c72d134c9ed795dd312d3c48` and
`2846137f95585d326db3081e71eeded7eb1dcd465f98c3953527eb979779773e`.
This controlled compressed-GPU sampling result does not validate the app's
CPU decoder/upload, linear filtering, mip blending, source fragment shader,
lighting, runtime selection or retail appearance.

A separate peer-reviewed CPU-only check called the same Rust `decode_pixels`
function used by the Bevy standalone mip loader. At those 16 level-zero
texels its decoded RGBA8 values divided by 255 differed from the retained WGPU
BC3-unorm values by at most `2.6881e-8`. The descriptive comparison receipt
`target/startup-sampler-cpu-result-v1/receipt.json` SHA-256 is
`d75ddfe67d5fa70c1730b19d21dc78eb81bbda26aabc30f52471d0298be36a16`.
The source and binary were separately pinned, not bound by a build-provenance
proof; this check does not execute Bevy's GPU upload or validate other texels,
filtering, mips or lighting.

The app exposes `--startup-unorm-texture` as a separate experiment only for the
admitted StartupCell path. It keeps the decoded RGBA mip bytes and sampler,
changing the image format from sRGB to unorm; normal and other art modes are
independent. A focused test checked mip order, bytes, count, sampler and mode
isolation. It remains a Bevy PBR preview, not the source shader or lighting.

In a reviewed paired windowless RTX 3060 Vulkan capture with the same
executable, seed 1 and tick-60 simulation, the unorm option changed 131 of
1,024,000 pixels, with maximum channel delta 72 and inclusive changed bounds
`[665,399,682,410]`. The baseline PNG SHA-256 was
`6ffdfe9e3a5e219f5d23baa82c1396941782ca69ac2be96d24ebd60e2621e2a2`,
the unorm PNG SHA-256 was
`fb620f32812211523bfd618b6a8a4c0183d4d2f70224dfa7e4d34a1b81a1650e`,
and `target/startup-unorm-capture-result-v1/receipt.json` SHA-256 was
`9c8d1ce74263ecc82685a1ddfed04da4ef13e5a83de0d91f492a9108e4a9b84f`.
Both stderr logs reported an unavailable Vulkan validation layer, a duplicate
OBS hook and older overlay API versions; no pipeline error appeared in the
retained logs, but this is not validation-layer-clean. The pixel difference
does not establish retail likeness, complete scene equivalence or a general
visual improvement.

A separate reviewed fixture used all ten captured sampler-zero BC3 mips,
linear min/mag/mip filtering, clamp, 16 quarter-texel coordinates and explicit
LODs 0, 0.5, 2.25, 5.5 and 9. D3D9 sRGB-off versus WGPU BC3-unorm samples
had maximum component delta `5.4949e-8` in these controls; WGPU unorm versus
sRGB RGB differed by up to `0.25038`. The latter is a control comparison,
not a transfer function applied after filtering. The receipt
`target/startup-sampler-linear-result-v1/receipt.json` SHA-256 is
`be3e6ab2f28567bdf2d897dd906923c426af146a2d5e537572e8255d4ac21497`;
D3D and WGPU raw outputs have SHA-256s
`bbecfc3fb83fd3350320a89b95a450e9f29adc505b2994e2d62b0ab53a6f7e41` and
`f943bed1dab715e11ba8f01e451a785d6bce4816cf2f054c55c6ca5a37da3a2b`.
These are finite synthetic samples of compressed GPU textures, not general
filtering equivalence or the app's CPU-decoded GPU upload.

The pinned cached inspection pixel program (recipe
`[05,09,0C,11,3B,19,2A]`, SHA-256
`89fb1e87bc7333fdfd6d229558807c6479bde472cff6aebac143f54e6d4d48e3`)
was extracted from the selected pack and disassembled without rendering or
mutating it. Raw disassembly SHA-256 is
`9c0a52c3d76df8fdddd1b11f453790925fef120804735f2356575713881e62b5`;
the disassembly receipt SHA-256 is
`5a920aadf3f1b576de42e0b5589a81b9e69f13d2d780bb0bc9983718d24db502`.
The program declares and samples s0 and s1, not s2, although the inspection
renderer prepares and binds a third texture. It uses s1 RGB in its normal-map
calculation, s1 alpha with material parameters for specular/gloss terms, and
s0 with vertex color, explicit SH/direct-light inputs and tint for final
output. The Bevy StartupCell preview currently uses s0 in StandardMaterial
PBR with fixed roughness; it does not implement this selected source pixel
program. This static dataflow does not establish a live material, sampler-2
purpose, visual cause, or retail lighting.

An independently reviewed constrained cross-GPU diagnostic ran that unchanged
cached PS with synthetic point positions and VS interpolants, s2 unbound,
captured s0 BC3 and s1 BGRA level-zero data, nearest/clamp sampling, white
vertex color/tint, orthogonal normal `(0,0,1)` and tangent `(1,0,0)`, material
`[1,0,1,0]`, camera direction `(0,0,1)`, and zero SH/direct-light inputs
except `c2.rgb=1`. A separately reviewed WGPU shader implements only the
resulting constrained algebra. At 11 selected nonzero-alpha samples, D3D9 and
WGPU float output components matched exactly in both captured-s1 and synthetic
all-white-s1 modes; replacing s1 changed a D3D RGB component by up to
`0.2152511`. All four hidden processes exited zero and coverage checks passed.
Receipt `target/startup-source-ps-fixture-result-v1/receipt.json` SHA-256 is
`eb0c9ab02442ce00ee0c5f7de6cedcc34411918cb21356f806b1b52eff0d162c`.
The source/white D3D raw hashes are
`9f20c2f4a31de4261850d1a764e66d8b32fd6aa739d46ed86207509ec7ae770a` and
`6b7903a8f64340be7058e5918ec88c92d4a4cbb2580a114916475ee33396c14f`;
WGPU raw hashes are
`4f94e61b5c7cf1c522b964d5d59ea87cfccc79345261c38d4dd96212dad2d554` and
`6dc1039714a58f92be6a082cc3d123415dfbd2a675fb1973198934b557710883`.
The synthetic interpolants, lighting constants and white texture are controls;
this is not general PS, filtering, lighting, live-state or retail parity.

A separately reviewed alpha/gloss control used the same unchanged cached PS,
11 sample coordinates, captured s0/s1 mip-zero data, nearest/clamp sampling,
white vertex/tint and orthogonal synthetic normal/tangent. Its synthetic VS
supplied `v4.rgb=1`; material `c0=[1,0,1,1]`, `c2.rgb=1`, with other
SH/direct terms zero. This constrains output to diffuse times a blend of
`(2/3)*normal.y` and 1, weighted by s1 alpha; direct specular is suppressed.
The alpha-only control set each BGRA s1 alpha byte to 255 while retaining RGB.
Both source and control modes had maximum D3D9/WGPU component delta `0.0`
at these 11 points; the control changed source D3D RGB by up to
`0.3882324546575546`. All four private-desktop processes exited zero and the
coverage/control checks passed. Receipt
`target/startup-source-ps-gloss-result-v1/receipt.json` SHA-256 is
`d8151a000ab47297befe4710b772b1728647756c81ffaa4befbf82d091b9c3bd`.
The 0.001 comparison is descriptive, and synthetic constants, interpolants,
point sampling and the altered alpha are explicit controls. This does not
establish general shader, scene lighting, live material or retail parity.

A fresh independently reviewed direct-specular diagnostic preserved the same
unchanged source PS, texture mip inputs, 11 point coordinates and alpha-only
s1 replacement. Synthetic `c0=[1,1,1,0]` suppresses the gloss blend while
using exponent 1 and s1 alpha as direct strength; `c17=(0,1,0)`, `c18.rgb=1`,
`c19=(0,0,1)`, `c2.rgb=1`, and other SH/direct coefficients are zero.
With synthetic orthogonal normal/tangent and white tint/vertex, its constrained
WGSL models the source reflected-vector dot as
`clamp(-2*normal.z*normal.y,0,1)`. For the 11 points, maximum D3D9/WGPU
component deltas were `2.3283064365386963e-10` with source s1 and
`1.4901161193847656e-8` with alpha replacement; replacing only s1 alpha
changed source D3D RGB by up to `0.12649909895844758`. All four hidden
processes exited zero, with coverage/control checks passing. Receipt
`target/startup-source-ps-specular-result-v2/receipt.json` SHA-256 is
`a2af1c35e7db769080915de84e9f9dbf6dd70db57c86beccc172db56b8ebb6fa`.
The unexecuted v1 had an incorrect reflection approximation and is preserved.
This is a finite synthetic control, not general source shader, live material,
lighting or retail parity; the 0.001 tolerance was descriptive.

A separately reviewed material control used the exact inspection-renderer PS
`c0` words `[42700000,3F733333,3F800000,3EB33333]` (60, 0.95, 1, 0.35),
asserted against the pinned trace. The original PS and both mip-zero textures
were unchanged; synthetic c2, c17=(0,0,-1), c18, c19, interpolants and point
geometry constrain the test. Source and alpha-only-s1 modes exercise the
combined gloss/direct calculation; a third synthetic no-direct control changes
only c0.y to zero. At 11 points the maximum D3D9/WGPU component deltas were
approximately `7.75e-7`, `3.34e-6`, and `0` respectively. The observed D3D
source/no-direct difference was `0.20832522958517075`; the alpha-only control
changed RGB up to `0.9915856523439288`. Six hidden processes exited zero and
coverage, finite-value and observable-control checks passed. Receipt
`target/startup-source-ps-material-result-v2/receipt.json` SHA-256 is
`eb7b02cef79b78e255852e035721771c8be3d3dd7b2e4e417fef8fef9cfd4476`.
The preliminary v1 remains unexecuted; its synthetic light direction made the
60th-power term too small to be a useful direct-specular test. The 0.001
D3D/WGPU threshold is reported, not enforced. This is a finite controlled
inspection, not general shader, lighting, live material or retail parity.

The checked CPU sampler-1 inspection example reads the exact unique content
resource `E219E5D3:2F4E681C:40612901` with decoded payload SHA-256
`ceecebb4937eef972474e222dc2069238ce171f8f0a97ab89a8981328da96150` and
requires an explicit stable-file admission. It checks all nine raw BGRA mip
rows against the pinned v4 CPU upload trace, and compares the independent
per-pixel BGRA-to-RGBA channel mapping against the same core raster decoder
used by the Bevy inspection loader. The externally pinned CPU run matched all
nine levels, with source/example/trace pins unchanged across the run; receipt
`target/startup-sampler1-inspection-result-v2/receipt.json` SHA-256 is
`123fd6dde69d0776f3a0fd5498a969c6d10c3157cb8fe94f16303ecdd83ab920`.
The focused channel/extent unit test passed. The successful v2 runner retained
complete streams, but would not retain partial streams on timeout; the separate
reviewed v3 runner corrects that failure handling and was not run. Repeated
reads are not atomic and stable files remain caller-admitted. This establishes
CPU input/decoding correspondence only, not Bevy GPU filtering, live provider
identity or retail rendering.

A separately reviewed sampler-1 GPU control used all nine source BGRA mips,
with the CPU-decoder-matched RGBA channel order on WGPU, clamp/linear
minification, magnification and mip filtering, and 16 explicit quarter-texel
coordinates at LOD 0, 0.5, 2.25, 5.5 and 8. At those samples the maximum D3D9
versus WGPU unorm component delta was `5.587935447692871e-8`; the reported
3/255 comparison was true. The WGPU sRGB control changed RGB by up to
`0.2871783375740051`, with alpha unchanged. Both hidden processes exited zero
and coverage checks passed. Receipt
`target/startup-sampler1-linear-result-v1/receipt.json` SHA-256 is
`527af56a88472aa969e21c633f7e5a2f6d43c80f5a6d57aadac318b0605cee7e`.
These are finite explicit-LOD sampler measurements; sRGB filtering is a
separate control. They do not validate implicit LOD, material lighting,
live state or retail rendering.

The separately opted-in `--startup-material-preview` uses the admitted
startup candidate, both exact-content textures and the captured c0 values with
the constrained WGSL subset above. Its tangent/basis, light coefficients and
camera direction are synthetic; Bevy supplies camera, implicit LOD, blending,
depth and ordering. The sampler-1 payload is checked by exact unique key and
hash before and after decoding nine mips, with stable files admitted. The
mode requires offscreen admitted startup art and excludes other material and
normal experiments; default PBR remains unchanged. The focused startup CLI
test passed and the reviewed application build succeeded.

A reviewed same-seed/tick hidden RTX Vulkan capture compared default PBR and
this synthetic material; both reported tick 60 and exit zero. Receipt
`target/startup-material-capture-result-v1/receipt.json` SHA-256 is
`8ce62c23a1bb123b12d8b62a12255f92c47211757938632a61ff171d06af74b0`.
The whole image had 2,607 changed pixels (includes the changed HUD label).
Within fixed rectangle `[650,390,700,430)` around the player, 131 pixels
changed with max channel delta 218 and inclusive changed bounding box
`[665,399,682,410]`; this is not actor segmentation. The separate analysis
SHA-256 is `b7d6c7c41a5a963b15137e3d216981b6cd476664db1af407015ed02189d70d5f`.
Both full logs reported Vulkan missing validation layer and overlay warnings;
no shader or pipeline error was reported, but this is not validation-layer
clean. The shared initial asset-intake log says `renderer=pbr` even in the
material mode; the later `CELL_STARTUP_MATERIAL` log identifies the selected
experimental material. Tick-summary agreement is not complete simulation
identity. This result does not establish retail material/lighting, live actor
state, general shader parity or a visual likeness improvement.

A reviewed vertex-output fixture ran the unchanged cached VS over all 2,555
pinned packed vertices using captured inspection constants c1..c13. It changed
positions to a synthetic point grid and c14..c17 to identity clip solely for
readback; separate synthetic pixel shaders captured o3 tangent, o4 normal,
o5 diffuse SH and o6 specular SH. An independent WGPU compute model used the
same captured inputs. Maximum D3D9/WGPU component deltas were
`1.1920928955078125e-7` for each of tangent, normal and diffuse SH, and
`3.5762786865234375e-7` for specular SH. Five hidden processes exited zero;
float coverage and finite checks passed. Receipt
`target/startup-vs-outputs-result-v1/receipt.json` SHA-256 is
`65b251f5dc250923df8f54c172f2d98c2e8f6bafac6e69a5fbf7f4c2ce876021`.
Results are measured deltas, not a universal tolerance guarantee; synthetic
positions/clip and captured inspection state are not live actor pose, camera,
lighting or retail-scene evidence.

A separately reviewed pointwise handoff supplied 11 selected original vertex
UVs and retained D3D VS tangent, normal and specular-SH outputs as identical
synthetic passthrough interpolants to the unchanged cached source PS and an
independent WGPU computation. Both used trace-captured PS c0..c20, the two
source texture mip-zero payloads, nearest/clamp sampling, and white vertex
color; point positions were synthetic. Maximum D3D9/WGPU component delta was
`2.9802322387695312e-8`; 0.001 comparison was reported true. Both hidden
processes exited zero, with finite/coverage checks passing. Receipt
`target/startup-vs-ps-join-result-v2/receipt.json` SHA-256 is
`02550fb3db856876fc296b61cd49e24f8925bbc1ba76787370e8011c339bb3af`.
The unexecuted v1 approximation had a c16 SH term error and is preserved.
The v2 fixture manually hands off retained outputs: it is not a contiguous
VS-to-PS draw and does not validate triangle interpolation, implicit filtering,
live state, or retail rendering.

A reviewed contiguous point fixture bound the unchanged cached source VS and
PS in one D3D9 draw for the same 11 selected original packed vertices and UVs.
Only XYZ positions were replaced by a synthetic grid and VS clip constants
c14..c17 by identity; other VS and PS constants came from the inspection
trace. The two captured texture mip-zero payloads used nearest/clamp sampling.
An independent WGPU compute composed the reviewed VS and PS algebra from the
same raw packed vertices. Maximum component difference was
`5.960464477539063e-8` (the descriptive 0.001 comparison was true); both
hidden processes exited zero and coverage/finite checks passed. Receipt
`target/startup-vsps-contiguous-result-v2/receipt.json` SHA-256 is
`5353cfa917864a2c696135464cc5abdb7a654e679e3b31cda36a4b027e711746`.
The unexecuted v1 had an inaccurate synthetic-VS docstring and is preserved.
This demonstrates a contiguous source VS/PS point draw under these synthetic
clip and filtering settings; triangle interpolation, implicit mip selection,
actual camera, live actor state and retail parity remain unverified.
The contiguous host stdout also inherited `compiler_path_checked=true` and
`mapped_image_continuity=admitted` text, but that host binds precompiled,
pinned VS/PS bytecode and does not invoke D3DX. Those stdout claims do not
record an actual compiler check for this point draw.

A separately reviewed coverage-first triangle fixture drew the original
source VS and PS in D3D9 using source index triple `(0,1,2)` and the exact
packed normals, tangents and UVs. Positions were replaced by a synthetic
large triangle, clip constants by identity, and D3D clip positions shifted
by a half pixel to align with WGPU pixel centers. Other VS/PS constants and
two mip-zero textures were trace-pinned; sampling was nearest/clamp, with no
depth, blending, culling or MSAA. A separate WGPU graphics pipeline modeled
the same vertex/fragment algebra. The initial unshaded coverage controls both
covered 1,272 pixels with zero edge disagreements; 1,078 shared pixels
survived a 3x3 interior erosion. In that interior the largest component delta
was `2.9802322387695312e-8` and p95 was zero; the reported 0.001 comparison
was true. All four hidden processes exited zero. Receipt
`target/startup-vsps-triangle-result-v1/receipt.json` SHA-256 is
`c57c30a77ba904584abd9c998d418ec987ecc2e5e5cfe00243d51ab89f4e66fc`;
raw masks and float outputs are retained beside it. This is one controlled
triangle with synthetic geometry, affine interpolation, and explicit mip0;
actual clip/camera, implicit LOD, animation, live scene and retail equivalence
remain unverified.

A reviewed batch fixture drew 16 source-index triangles (every 250th of the
pinned mesh), preserving each corner's packed normal/tangent and UV. Positions
were replaced by synthetic tiled triangles; clip was identity, with a D3D9
half-pixel correction. The unchanged VS/PS used captured constants, both full
source mip chains and traced clamp/linear/mip-linear state; WGPU used matching
algebra and implicit sampling. Separate unshaded controls covered 1,056 pixels
on each API with zero edge disagreements, leaving 448 shared 3x3-eroded
interior pixels. In that aggregate interior the maximum component difference
was `5.960464477539063e-8`, p95 `4.470348358154297e-8`; the descriptive 0.001
comparison was true. All four hidden processes exited zero. Receipt
`target/startup-vsps-batch-result-v1/receipt.json` SHA-256 is
`3dfffdfce75802bbd8e0e9ecdf6249638e275e101a331d73e9c49c7abcf2db5b`.
The interior threshold was aggregate, not a per-triangle coverage guarantee.
The synthetic geometry, captured inspection state, fixed sample set and
single adapter do not establish live pose/camera/lighting or retail parity.
A separate post-run analysis of the retained masks and hash-verified material
buffers found 66 covered pixels and 28 eroded interior pixels in each of the
16 tiles; each tile's maximum material delta was at most
`5.960464477539063e-8`. The original receipt did not pin the masks, so this
is a post-run observation rather than a receipt-enforced per-tile gate.

The initial frozen traced-material Bevy capture failed before producing the
traced image: Naga rejected module-level `bitcast` expressions in constants.
The baseline completed; the failure receipt
`target/startup-traced-material-capture-result-v1/receipt.json` SHA-256 is
`0bd17dd9b0a00117aec8b018ee38395d5a956767539bfc675a947f8c78b1110f` and
full streams are preserved. A subsequent shader revision moved the exact
bitcast words into function expressions; an offline Naga 27 parser/validator
passed with Bevy imports replaced by synthetic signatures. That static check
is not a Bevy GPU pipeline or rendering result.

The corrected opt-in `--startup-traced-material` uses the hash-bound candidate's
packed normal/tangent bytes and the captured inspection VS/PS constants with
the two admitted source textures. Its shading transform, camera direction and
lighting are frozen to that inspection trace; Bevy supplies a separate clip
transform, camera, implicit LOD, blending, depth and scene ordering. Static
nonzero/noncollinear vertex checks do not prove all interpolated fragment bases
are nondegenerate. The source basis is a static candidate; this is not a live
pose, source-camera or retail-material claim.

The reviewed v2 hidden RTX Vulkan paired capture completed with both modes at
reported tick 60 and exit zero; receipt
`target/startup-traced-material-capture-result-v2/receipt.json` SHA-256 is
`d6cd8ae7c0e26c47fb0c480c15b5e428d04e29460683e90e94b155c2a122a8d1`.
Whole-frame changes were 2,756 pixels including the diagnostic label. In the
fixed player-region rectangle `[650,390,700,430)`, 131 pixels changed, max
channel delta 172, inclusive changed bbox `[665,399,682,410]`; analysis SHA
`9530bbf7b24445735334c26262a5797e8b7f1037db6392e62b738c264bc04917`.
Both complete logs were checked: no error/panic/shader/pipeline failure was
reported, but missing Vulkan validation layer and OBS/overlay warnings were
present. Matching tick summaries are not full-state equivalence. The traced
player remained tiny/dark on visual inspection; the difference does not
establish retail likeness or a visual improvement.

A reviewed full-static-mesh fixture bound the unchanged source VS and PS over
all 2,555 pinned vertices and 4,244 source triangles with the captured
inspection model-to-clip and shading constants. Both APIs used full source
mips with linear/clamp/mip-linear filtering. D3D retained its captured clip;
WGPU applied a half-pixel position correction. Synthetic harness depth-on
D24S8 versus Depth24PlusStencil8 used less-equal/write/clear 1, with separate
depth-off controls. Both coverage masks contained 41,420 pixels with no edge
or nonedge disagreements; 39,956 shared 3x3-eroded interior pixels were
compared. Maximum D3D9/WGPU component delta was `0.00027466192841529846`,
p95 `5.960464477539063e-8`; the descriptive 0.001 comparison was true. Depth
changed 20,946 interior pixels on each API. All eight hidden processes exited
zero. Receipt `target/startup-vsps-fullmesh-result-v2/receipt.json` SHA-256 is
`29e9b8e68edd59e6c580644e2a3c2cc019dedced9a651ca457ab36f93b884886`.
The GPU outputs used floating targets, not the original inspection renderer's
A8 output; depth formats/state are controlled choices and the retained camera
is a fitted editor-inspection camera. This does not establish gameplay camera,
live pose or retail parity.

The first opt-in `--startup-inspection-camera` app capture completed with both
runs reporting tick 60 on RTX Vulkan. Receipt
`target/startup-inspection-camera-capture-result-v1/receipt.json` SHA-256 is
`671ed7384eef5084c3eac8e60bc42571022c2bcaa7387525d8ab21fc8e2a53bf`.
The inspection image shows a larger candidate, but the prototype playfield is
absent inside the 768-square region and UI is confined to that viewport.
There were 539,672 whole-image changed pixels, including viewport and UI
changes, so this count is not an asset-only comparison. Full stdout/stderr
contained no reported error/panic/shader-failure markers; Vulkan validation
layer and overlay warnings remain.

The camera is explicitly given `IsDefaultUiCamera`, and the offscreen routing
also chooses the highest-order camera for UI; both select the 768-square 3D
camera. This explains the UI viewport selection. Pixel inspection found the
prior playfield color `(6,23,28)` at (300,400) in baseline but the default
clear-like `(3,8,12)` there in inspection, while (255,400), just outside the
viewport, retained the playfield color. The inspection 3D camera explicitly
uses MSAA off while the ordinary 2D camera retains its default sample count;
a camera-target/sample-count interaction is a plausible cause, not yet a
verified diagnosis. `ClearColorConfig::None` alone did not preserve the prior
playfield in this capture. Further controlled camera routing/clear tests are
needed before claiming a fix, full-scene comparison, live camera or gameplay
parity.

The opt-in `--startup-inspection-camera` mode now routes UI through a separate
full-window camera and uses MSAA off for its scene and UI cameras; it excludes
source effect-plane modes with a competing composite. The original v1 capture
is preserved above. A separately reviewed v2 paired capture on RTX Vulkan
completed both runs at reported tick 60; receipt
`target/startup-inspection-camera-capture-result-v2/receipt.json` SHA-256 is
`d4e4882f294cef45054cba6182d474f675ca12fe4644c0bc47c963fcaff83df2`.
In the captured image the enlarged static candidate appears over the visible
prototype playfield and the HUD spans the full window. Two background probes
`(300,400)` and `(255,400)` exactly matched baseline `(6,23,28,255)`; a
small full-window `CELL` text ROI contained 309 bright pixels in each run.
Those checks do not establish complete UI or playfield equivalence. The
routing and MSAA changes were tested together, so the specific cause of the
v1 missing background was not independently isolated. Full stdout/stderr
contained no error/panic/shader-failure markers, while missing Vulkan
validation-layer and OBS/overlay warnings remained.

The source clip matrix produces a frozen editor-style inspection view; it
ignores the prototype actor's changing transform for clip placement. Bevy
still supplies the scene target and ordering, and the asset remains a static
candidate with frozen inspection shading. This is a diagnostic overlay on a
prototype, not a gameplay camera, live actor pose or retail parity. Reproduce
with a new screenshot path:

```powershell
target\debug\rustspore.exe --experimental --cell --seed 1 --source-startup-install 'D:\Games\SPORE Collection' --admit-startup-stable-packages --offscreen --demo 1 --shot target\startup-inspection-camera-new.png --startup-material-preview --startup-traced-material --startup-inspection-camera
```

A fresh bounded in-memory decode of the exact Spore_Content E219E5D3 GMDL SHA-256 `95ba0f5e5e71761ccf0f2ef919cce9bcc8d40303be4b1b7a9fa954a7c7cb8ff4` found mesh 0 uses vertex buffer 0: 2,555 records at decoded offset 25,668, stride 36. The source declaration places UBYTE4 usage 2 / rw_decl 14 at +28 and UBYTE4N usage 1 / rw_decl 15 at +32. All 2,555 raw weight-byte sums are 255; 6,195 nonzero lanes have offsets divisible by 3, with maximum 105. Its serialized direct-draw descriptor pair is (0,36). This is distinct from the saved first group count 10 and from the 85-vertex generated-group fixture. Under the explicit direct-draw path and offset/3 diagnostic admissions, active offset 105 addresses ordinal 35; a 10-matrix supplied prefix cannot cover it. Neither that arithmetic nor the serialized pair proves runtime draw selection, shader behavior, actual palette identity, or that the original 7314A0 CPU path consumes this authored stream. Package endpoint SHA-256 was `0b849bd79d9c13dec5737159655bf7e2b7763cc2b79949c81370fa0490256e8c` both before and after; reads are non-atomic and require stable files.

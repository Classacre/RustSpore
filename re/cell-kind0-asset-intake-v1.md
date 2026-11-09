# Candidate kind-0 Cell asset intake

A bounded, explicit-package inspection joined raw Cell `FD24574B` to the
candidate structure of the same ID, then its kind-0 row to PROP and RW4
`DAE8CE17`. This is a disk projection: native reference fixup, provider
precedence, route selection, actual actor construction and the kind-1 row
remain unverified or unsupported. The model is not a claim of in-game appearance.

The coordinating agent reports this build and invocation:

```powershell
tools\build.bat build -p spore-bevy --example cell_kind0_asset_intake_v1
target\debug\examples\cell_kind0_asset_intake_v1.exe "D:\Games\SPORE Collection\Data\PatchData.package" "D:\Games\SPORE Collection\Data\Spore_Graphics.package"
```

The retained `target/source-cell/cell-kind0-asset-intake-v1.log` (SHA-256
`e0df813252cb0e1dfd7fe56933002d327988fc20aa7f17dcb8bf1a4ae9d39b6b`) records
20 RW4 sections, one decoded mesh, and a generic projection of 454 vertices
and 896 triangles, source normals and zero sanitised values. This projection
can skip or synthesize attributes; **mesh topology completeness is unverified**.
The v1 log predates the added explicit completeness marker. A rebuilt v2 run
reported the same mesh counts and the marker in
`target/source-cell/cell-kind0-asset-intake-v2.log` (SHA-256
`f17f64293c8e589b00b4a66872446d9aa58a70b2d6d7a41d5caa7168d3d2f426`).

It records compiled state section 14 with shader `80000004`, and section 15
raw link words `13, 1, 14`. Two raster sections, 17 and 19, each decode to
128×128 DXT5 with eight mip levels. Subsequent independent state-decoder review
reports saved stage 0 raster word `00000011` and stage 1 word `00000013`,
corresponding to sections 17 and 19 in this candidate. The rebuilt v2 intake
reports the same candidate-specific correspondences and saved state pairs,
consistent with that independent review. Runtime pointer fixup and shader
execution are not established. The PROP's LOD0 names the RW4 type
`2F4E681B` and instance `00000000DAE8CE17`; the loaded package is an explicit
inspection choice, not verified retail resolution.

The coordinating agent supplied these SHA-256 hashes for the exact payloads:

- PatchData Cell `FD24574B:DFAD9F51:00000000`: `f8950cef073bc3536f1a736436a4048a8b6d5bf0066bc7db9b06c979185e72bd`.
- PatchData structure `FD24574B:4B9EF6DC:00000000`: `307a7008d90f638e8c16c6367b3cae2090e3b639d83b15ef282c0ba9d48ee3e5`.
- Graphics PROP `DAE8CE17:00B1B104:00000000`: `15ebf08981b6b45ec8ce98d899135126281a30c4ecde4caaacbd496c8e82b9e8`.
- Graphics RW4 `DAE8CE17:2F4E681B:00000000`: `bbd99a741d41a3606a310f242e14e5f85dc7b49af454972ff02124b16e318592`.

Current package hashes measured separately are PatchData
`cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff` and
Spore_Graphics `e01bb86d8ddf02c56d52394b4e1db7181c1e1ebe1878bf6fb72c4ee523fcedd1`.
Package preflight and index reads reopen the input, requiring trusted stable
files; this does not remove the replacement window.

## Controlled BC3 decoder comparison

The integrated DXT5/BC3 fix uses four RGB colors even when color endpoint 0
is less than or equal to endpoint 1, while retaining DXT1's separate
transparent-palette rule. On the pinned `DAE8CE17` RW4, the saved stage-zero
raster is section 17 backed by section 16. Of 1,024 base-mip BC3 blocks, 13
have `c0 <= c1`; the corrected decode differs from a reconstruction of the
previous DXT1-style color branch at 81 of 16,384 source texels. Encoded alpha
is preserved in both sides. The package and payload hashes above were checked,
and the package hash matched at both read endpoints under stable-path
admission. `target/source-cell/kind0-bc3-compare-20261108-v1/receipt.json`
hashes to `814352f7471371154548a4ac20444315d837a0ce4a9bf07fc3a7844b0dc1721e`
and binds the two decoded 128×128 RGBA images and exact source data.

A separate CPU-only, 768×768 nearest-neighbor texture inspection placed each
decoded image into the same 640×640 rectangle over the same dark background;
1,425 composited pixels differ. Its images and receipt are under
`target/source-cell/kind0-bc3-offline-render-20261108-v1/`. This illustrates
the data correction, but is not a mesh, Bevy or original-shader render, nor a
claim that the affected texels are visible from any in-game camera. The legacy
side is a reconstructed old decoder, not a captured historical frame.

A separately reviewed temporary mesh fixture then rendered the two decoded
versions under identical controlled inputs on the NVIDIA RTX 3060 Laptop GPU.
Stage zero used eight levels of explicitly decoded uncompressed BGRA; stage
one remained the source compressed raster. Both passes used the same candidate
mesh, generated source-fragment shader, 64 identity palette matrices, white
material/tint, bounds-fit camera, sampler and render-state assumptions; the
runner checked matching adapter, assumptions, generated HLSL and compiled
shader bytecode. Both processes exited 0 with empty stderr. The 768×768 frames
differed at 1,569 pixels, so affected texels are visible under this controlled
view. Legacy frame SHA-256 is
`b78642474e89e438cc8bd925593da84140899bbf18c889aebf7d4da21647b52b`;
corrected frame SHA-256 is
`c6a2faa819de217e06b9bc6a42fe5d9f2044db5a5570bdb7a74e21ee6e1eed95`.
The receipt `target/source-cell/kind0-bc3-mesh-result-v1/receipt.json` hashes
to `4034162e9acfb4efe4383ac6567bf4dadbf05ce5e70f28564ee0b256fd182fe1`
and records complete input and artifact hashes. These are uncompressed-stage
diagnostics rather than a comparison against the compressed GPU sampler or a
live game frame; pose, material, camera, resource choice and display visibility
remain controlled or unverified. No legacy switch was added to the frontend.

## Saved blend/depth feasibility boundary

The candidate compiled state stores alpha blending enabled, source factor
`SRCALPHA`, destination factor `INVSRCALPHA`, and depth writing enabled. A
separate synthetic 64×64 comparison used D3D9 and Vulkan/wgpu on the RTX
`10DE:2520`, both with non-sRGB 8-bit targets, identical factors for color
and alpha, `LESSEQUAL` depth and two full-screen draws: translucent red at
depth 0.25 followed by opaque green at 0.75. With depth writes disabled the
center pixel was `(0,255,0,255)`; with them enabled it was `(128,0,0,191)`.
All four processes exited 0, and after BGRA-to-RGBA conversion every channel
matched exactly between backends in each control. The runner admitted a
one-unit tolerance but observed no difference. Receipt
`target/source-cell/saved-blend-depth-result-v1/receipt.json` SHA-256
`a509949e20246a8b9aa6ee3b4d08a009de99e1fa28a6c516c9bc5e9566735a20`.
This tests synthetic blend/depth mechanics only, not the source mesh, shader,
texture filtering, scene ordering, color management, or live material state.

A separate no-window Bevy specialized-material fixture reproduced both D3D9
buffers exactly on RTX Vulkan, including both center pixels above. Its receipt
`target/source-cell/bevy-blend-depth-result-v1/receipt.json` has SHA-256
`ebe04e4fed153f83aeffbd8faf0b2a2fc8795faf716012c8458b4bf2d9ccb856`.
It uses synthetic quads, non-sRGB targets, explicit depth clear and comparison,
no MSAA or tonemapping, and controlled draw ordering. This establishes only
the tested pipeline mechanics, not source shader, asset, or live-scene parity.

The experimental Cell frontend exposes `--source-kind0-saved-blend` only for
an offscreen run explicitly selecting source cell `FD24574B`. It specializes
the player's Bevy PBR material with those two saved blend factors on both
channels and depth writes enabled; the normal material remains the default.
It retains Bevy's reverse-Z-compatible depth comparison and transparent draw
ordering. The source shader, original scene order, live material state and
camera are not established by this diagnostic.

An offscreen pair using seed 42 and a 0.000001-second demo captured the
initial prototype state: both runs reported tick 0, position `(0,0)` and
health 60. Both exited 0 on RTX 3060 Vulkan. Of 1280×800 pixels, 308 RGB
pixels changed within bounding box `(629,389)-(651,411)` around the player.
Baseline PNG SHA-256
`faf08eae6051193e2099ef3abb1083a043be4456238a34cb52f53edc286259c7`;
opt-in PNG SHA-256
`f9e1124e1b9036e329a6de90e0a1e1d551a5ffec91b97273b3e9fad80c1542b6`;
receipt `target/source-cell/saved-blend-player-frozen-v1/receipt.json`
SHA-256 `fccef3619c441ccd94ff4f55aa86b7cc9eb3d48100cfaa5348f9d5f9c9fb7954`.
No pipeline or readiness errors were reported; both runs warned that the
Vulkan validation layer was unavailable. A separate one-second pair differed
in player position as well as 391 pixels, so it cannot isolate this pipeline.
These captures retain the PBR shader, inspection camera and prototype scene;
they do not establish original scene or live material equivalence.

A subsequent isolated Bevy/WGSL scratch harness used this same D3D9 mesh
fixture's raw geometry, corrected eight-level stage-zero texture, white
material/tint and identity-palette inputs. It hypothesized the D3D9-to-Vulkan
half-pixel shift and used the stated blend/depth and sampler settings. On RTX
Vulkan its clear-color-derived foreground mask agreed with the corrected D3D9
capture on all pixels (166,269 foreground pixels), but among 164,429 common
eight-neighbor interior pixels only 15,849 were exact and 62,569 were within
one channel unit; maximum channel difference was 66. This is a diagnostic
comparison, not an accepted equivalence test: the mask is inferred from clear
color, and sampler/LOD, interpolation, blend and rounding causes have not been
isolated. The identity palette does not validate blend-index component order.
Receipt `target/source-cell/kind0-wgsl-scratch-result-v2/receipt.json` SHA-256
`f97fd9b8a60b206fa0a41bb6054c93e82536b240bbce4caaf4a493043a366afc`
records inputs and outputs. The first scratch attempt failed during image
initialization and remains preserved under `kind0-wgsl-scratch-result-v1`.
Do not promote this tentative shader into gameplay until source-declaration,
sampler, matrix and pixel-center controls and the same-mesh color comparison
are independently validated.

An additional point-filter/base-mip pair kept the same mesh and constants but
overrode both devices to nearest sampling at mip zero. The D3D9 host's two
identical-input captures matched; receipt
`target/source-cell/kind0-point-mip-native-result-v1/receipt.json` SHA-256
`e99aba9a7be55b73e12d181ad96fa21911bcc46ba07f674fbc6d070889cdc544`.
The Bevy comparison again had 166,269 matching clear-color-derived foreground
pixels and zero mask disagreements. Among 164,429 common interior pixels,
62,977 were within one channel unit and the maximum difference was 72;
receipt `target/source-cell/kind0-point-mip-compare-result-v1/receipt.json`
SHA-256 `bbaf79434ce00742a448270ec7f46e24885730768790ab052cc9625cf9cc75da`.
This single controlled point-sampling case does not isolate the discrepancy or
establish saved linear-filter, shader, or live material equivalence.

The point/base-mip fixture was also run with blending disabled on both devices.
The clear-color-derived masks again agreed, but 70,853 of 164,429 common
interior pixels were within one channel unit and the maximum difference was
186. The Bevy material also moved to the opaque render phase, so this control
does not isolate a single cause. Comparison receipt
`target/source-cell/kind0-noblend-compare-result-v1/receipt.json` SHA-256
`e8aee12105272b3112efcf973c9b077ea6d998d8dbd8ce07677bef7176e31140`.
Equivalent-state fragment-output and UV tests remain needed.

With no blending, a further synthetic fragment replaced texture sampling on
both devices with `(fract(U), fract(V), 0, 1)` under the same mesh and depth
fixture. The operational foreground masks agreed; 69,762 of 164,429 common
interior pixels were within one channel unit, with maximum UV-output channel
difference 9. Receipt
`target/source-cell/kind0-uv-compare-result-v1/receipt.json` SHA-256
`634e7acf8d8c85069894e50c4b71e26db04aa61eca96706758d2c73e784e4469`.
This is a synthetic diagnostic rather than the generated pixel shader. It
shows a difference before texture sampling, without establishing its cause;
interpolation and geometry coordinate offsets require separate controls.

Removing the hypothesized half-pixel clip adjustment from the Bevy UV probe
increased the clear-color-derived mask disagreement from 0 to 653 pixels;
68,240 of 164,102 common interior pixels were within one channel unit, with
maximum difference 9. Receipt
`target/source-cell/kind0-uv-noshift-compare-result-v1/receipt.json` SHA-256
`26c5c56f4d184bc6e1fbfb9726f56a82cf7b5f1f6a35a9827015c3d2ca789cc9`.
This favors the shifted control for this fixture's observed coverage; neither
control establishes UV interpolation equivalence or a unique cause of color
differences.

An isolated triangle control removed mesh overlap and per-backend vertex
transformation as confounders. Source triangle 46 (indices 95, 92, 94) used
the exact source binary32 UV bytes and one shared precomputed binary32
clip-position stream; both devices used passthrough vertex shaders, synthetic
`fract(UV)` output, no blending and forward-depth controls. With the Bevy
half-pixel shift, the clear-color-derived coverage masks agreed and 819 of
832 common interior pixels were within one channel unit, maximum difference
2. Receipt `target/source-cell/kind0-triangle-compare-result-v1/receipt.json`
SHA-256 `0bb9b5c3afc1149d3fa49b6b01d2a038f936655808bcb2ca5c371d1e71671de8`.
Without that shift, coverage differed at 89 pixels; 784 of 792 common interior
pixels were within one, maximum difference 2. Receipt
`target/source-cell/kind0-triangle-noshift-compare-result-v1/receipt.json`
SHA-256 `0efc5e51c95aebb66440b3654f6787c41d069e600ba4ca10b108b861be65224d`.
This controlled triangle favors the shift for coverage, but does not establish
full-mesh interpolation, overlapping depth selection or original shader parity.

The overlap primitive-ID probe used spaced nibble colors, calibrated at all
896 non-overlapping tile centers on both devices within two channel units.
With identical clip streams it found 97,363 differing decoded IDs among
166,269 common foreground pixels, with no operational coverage disagreement;
receipt `target/source-cell/kind0-id-nibble-overlap-compare-result-v1/receipt.json`
SHA-256 `7316d42b9467211ecb992b29ee57addb00a2414b50d1ea36d12df8ab6f5a6e10`.
The result does **not** compare equal depth states: a subsequent native query
after fixture state setup returned `ZENABLE=0`, `ZWRITEENABLE=1`, and
`ZFUNC=4` (LESSEQUAL). Bevy's comparison enabled depth testing and writes.
The native fixture binds D24S8, while Bevy uses Depth32Float. The native
BC3 same-mesh and earlier shader captures inherited the same device setup
and did not explicitly enable Z testing; they should therefore be treated as
depth-disabled fixture evidence, not a source of live game depth state.
Explicit equal-depth controls and depth-format/tie analysis remain necessary.

In a follow-up primitive-ID control, native Z testing was explicitly enabled
and queried (`ZENABLE=1`, `ZWRITEENABLE=1`, `ZFUNC=4`); Bevy also used depth
writes and LESSEQUAL, with both clearing depth to 1 and blending disabled.
The common binary32 clip stream and the calibrated spaced-nibble decoder
yielded zero decoded triangle-ID disagreements across all 166,269 valid
common foreground pixels and zero operational coverage disagreements. Receipt
`target/source-cell/kind0-matched-depth-overlap-compare-result-v1/receipt.json`
SHA-256 `30dafad838833f78419a5d5bccd3a3742dde2030cf19bd0b56ccf6678f85f935`.
This controlled result corrects the earlier comparison of unlike depth-enable
states; the native surface remains D24S8 and Bevy's is Depth32Float. It does
not establish shader, texture, live depth-state or full scene parity.

The synthetic full-mesh UV-output comparison was then repeated with native
depth explicitly enabled and queried. The operational coverage masks agreed;
162,405 of 164,429 common interior pixels were within one channel unit and
the maximum was 2. Receipt
`target/source-cell/kind0-uv-depth-compare-result-v1/receipt.json` SHA-256
`878d79fe28c6229bac36c312e28d74b1e927a1bb7afdeadbc95e2a424139ccbe`.
This materially narrows the earlier UV discrepancy under these controlled
inputs, after correcting the unlike-depth setup. D24S8 and Depth32Float still
differ, and this synthetic UV output does not establish texture or live parity.

With native depth explicitly enabled, the controlled stage-zero texture
comparison used the generated D3D9 fragment and a tentative Bevy WGSL port,
white material/tint, identity palette, point min/mag and base mip, no blending,
and the same candidate mesh and camera assumptions. The operational coverage
masks agreed; 164,325 of 164,429 common interior pixels were within one
channel unit, maximum difference 2. Receipt
`target/source-cell/kind0-texture-depth-compare-result-v1/receipt.json`
SHA-256 `f6e999ff412282a6e1472690a59f45af3555801521f08ea479a645d8851d7ca4`.
Native depth was D24S8 and Bevy depth was Depth32Float. The comparison uses
controlled input and does not establish saved linear/mip/blend behavior, live
shader selection, pose, scene ordering or original camera parity.

The controlled full-mesh comparison was also repeated with saved stage-zero
linear min/mag, point mip and source-alpha/inverse-source-alpha blending, and
with native depth explicitly enabled and queried. Operational coverage masks
agreed across 166,269 foreground pixels. Of 164,429 common interior pixels,
147,570 were within one channel unit; maximum difference was 3. Receipt
`target/source-cell/kind0-saved-depth-compare-result-v1/receipt.json` SHA-256
`f293e70f8a134cbb319ff6c20005107b740b964ad5af554c51d7d2c366a82d11`.
This compares generated D3D9 fragments to a controlled WGSL implementation
using the candidate mesh, decoded stage-zero texture, identity palette, white
material/tint and bounds-fit camera. It does not establish live selection,
pose, camera, scene order, compressed sampling or complete shader parity;
native D24S8 and Bevy Depth32Float remain different.

The experimental Cell frontend now offers an offscreen-only
`--source-kind0-fragment-preview` for the explicitly selected `FD24574B`
player. It uses the decoded stage-zero mip chain as a non-sRGB texture, the
saved repeat U/V and linear min/mag with nearest mip sampler, white material
and tint, source-alpha blend factors on both channels, and depth writes. Its
vertex path uses unweighted bind positions as an identity-pose approximation;
the camera, reverse-Z comparison and transparent ordering are Bevy's. It is a
source-fragment-inspired preview, not an executed source vertex shader or a
claim about live pose, material selection or screen appearance. The default
PBR and separate saved-blend diagnostic are unchanged.

In a no-window RTX Vulkan run with seed 42 and a 0.000001-second demo, the
default and preview both exited 0 at tick 0, with no pipeline/readiness errors.
The preview reported repeat U/V, linear min/mag, nearest mip and LOD 0–7.
Its 80×80 player region contained 372 changed pixels, bounding box
`(629,389)-(651,411)`; full-frame differences also include the changed label.
Baseline PNG SHA-256
`faf08eae6051193e2099ef3abb1083a043be4456238a34cb52f53edc286259c7`;
preview PNG SHA-256
`4f19e8fdc36ec0afa8cb20b294eeb78c4dada136e6b5c5ae204eb5ac627c8ff7`;
receipt `target/source-cell/fragment-preview-player-frozen-v1/receipt.json`
SHA-256 `ac118c6356ccca30c39261846b457be996cffe4d0721a53694bd684afcbec907`.
The app binary SHA-256 was
`9588eb2b0e65fe7e26396126a4638493920da08a51b0a81697fd33d065a37739` and
the embedded WGSL source SHA-256 was
`860559ed21dbcd7408d7aa3deaced01b28d3cbcdd94c7c3fe18708749e679c24`.
Both runs warned that the Vulkan validation layer was unavailable. These
screenshots validate the opt-in application path under controlled state, not
retail rendering or the identical camera used in the separate D3D comparison.

The candidate's saved section-7 declaration uses type 5 at offset 28 for
`BLENDINDICES0` and type 8 at offset 32 for `BLENDWEIGHT0`. The installed
Windows D3D9 definitions identify these as `D3DDECLTYPE_UBYTE4` and
`D3DDECLTYPE_UBYTE4N`, respectively; `D3DDECLTYPE_D3DCOLOR` is type 4 and is
not this declaration. Thus an earlier suggestion of D3DCOLOR channel swizzling
for these indices was incorrect. A nonidentity palette is still needed to test
component order and the index-to-register interpretation; the identity fixture
cannot distinguish different index mappings. All 454 inspected stored records
have fourth weight zero and first three weight bytes summing to 255, but that
does not establish the active runtime palette or exact GPU arithmetic.

A further matched-depth same-mesh comparison used synthetic nonidentity
palettes: matrix `k` translated X by `k × 0.004` and Y by `(k mod 3) × 0.004`.
The original generated D3D9 vertex shader and the controlled WGSL model used
the candidate's raw UBYTE4 index lanes and UBYTE4N weights. The 454 vertices
include 380 with one active weight, 10 with two and 64 with three; active
register offsets span `0,3,...,24`. Both operational coverage masks contained
167,870 pixels with zero disagreement. Among 166,003 common interior pixels,
149,271 were within one channel unit and the maximum difference was 3.
Receipt `target/source-cell/kind0-nonidentity-compare-result-v1/receipt.json`
SHA-256 `9db2ede64ab8361cb36cd514815da4149b8b7674bfc19b49bd14b2983c0c7a2b`.
The retained native frame differs from the identity control at 129,611 RGBA
pixels. This exercises the observed lanes and register offsets under one
synthetic palette, saved linear/mip/blend assumptions and matched enabled
depth; it does not establish live palette contents, all declaration cases,
original pose, exact arithmetic, or a unique runtime actor/resource.

An optional application diagnostic now carries admitted raw indices and weights
alongside matching decoded positions, UVs and indices. It validates the exact
candidate mesh, declaration and ranges, verifies the projection matches the raw
data and requires both `--source-kind0-fragment-preview` and
`--source-kind0-synthetic-palette`. The synthetic palette applies the formula
above; the existing fragment preview retains its unweighted default. Example:

```powershell
target\debug\rustspore.exe --experimental --cell --seed 42 --demo 0.000001 --offscreen --shot target\synthetic-palette-new.png --source-cell-install "D:\Games\SPORE Collection" --source-cell-id FD24574B --source-kind0-fragment-preview --source-kind0-synthetic-palette
```

For the paired unweighted capture, use the same command with a different new
`--shot` path and omit `--source-kind0-synthetic-palette`.

In the controlled tick-0 RTX Vulkan application pair, the unweighted and
synthetic modes both exited 0 with the player visible and no pipeline or
readiness errors. The synthetic run logged its formula, unverified runtime
pose, Bevy inspection camera, reverse-Z comparison and transparent ordering;
the sampler used repeat U/V, linear min/mag, nearest mip and LOD 0–7. There
were 306 changed pixels in the 80×80 player region, bounded by
`(629,389)-(651,411)`; full-frame changes also include the label. Receipt
`target/source-cell/synthetic-palette-player-frozen-v1/receipt.json` SHA-256
`d523668c341c94c1f8f3d6683ff92a171e68d13c9fd116575eba02611c1488f0`;
unweighted PNG SHA-256
`4f19e8fdc36ec0afa8cb20b294eeb78c4dada136e6b5c5ae204eb5ac627c8ff7`;
synthetic PNG SHA-256
`ba71dc4102d4e007b8625ef433d9d5a7962d7e92311e45faeb0c456f9385fb9f`;
binary SHA-256
`eb77e92adc3a22d992f9747380a824e692307be2ef1fd7e7b72be2eadc712c3a`.
Both runs warned that the Vulkan validation layer was unavailable. This removes
the unweighted approximation for the specified synthetic control only; source
palette ownership, animation scheduling, provider choice, model identity and
draw-time state remain unverified.

For an opt-in Bevy implementation, `AlphaMode::Blend` alone is insufficient:
the installed Bevy 0.18.1 `bevy_pbr/src/render/mesh.rs` alpha-blend branch
sets `depth_write_enabled = false`, and wgpu's standard `ALPHA_BLENDING` uses
an `OVER` alpha component rather than this fixture's source-alpha factors on
both color and alpha. A custom specialized material/pipeline would have to
explicitly set the desired depth write, compare function and both blend
components, while preserving transparent draw order and attachment formats.
The currently integrated StandardMaterial additionally uses PBR lighting and
opaque sRGB interpretation; those choices do not reproduce the generated
source-fragment shader. The controlled original-fragment fixture's pixel
program multiplies vertex color, stage-zero texture and supplied tint; its
vertex path applies a supplied skin palette and transform. A source-matching
WGSL path needs validated declaration/attribute conversion, UV and mip/filter
semantics, shader arithmetic, matrix convention, palette and runtime state,
followed by a same-mesh comparison against the D3D9 fixture. The candidate's
454 source vertices all have a zero fourth stored weight and first three
weights summing to 255, so a controlled identity-palette comparison is
plausible; this does not establish the live palette or its numerical
equivalence. No rendering change follows from the synthetic test alone.
The fixed-byte weight audit is
`target/source-cell/kind0-identity-palette-audit-20261108-v1.json`, SHA-256
`95602acb4c87d7bee49607d0c8d7cd815b216489490bd45a3c0e4cee1453cb9b`.

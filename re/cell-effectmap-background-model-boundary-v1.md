# Cell effect-map background model candidates

This is a conditional source and installed-data join, not an observed Cell
frame. The installed SporeApp.exe SHA-256 is
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.

`E63510` acquires the Cell globals object, resolves its `+3C` handle via
`E4CAA0`, then iterates the returned records. When a record's `+04` word is 5,
`E635AE..E635B7` passes that record and other dynamic arguments to `E5E100`.
On a conditional path inside `E5E100`, `E5E282..E5E295` passes the record's
first word and group zero to `627DE0` with CellBackgroundEffectsWorld. That
function also uses environment records, current global scale and `E50080`
output to position and gate effects. This is a source path, not proof that a
given effect is active or of any particular transform.

The globals initializer at `137C047..137C068` declares `+3C` as type-3 field
`effectMapEntry` with descriptor `016B1D60`, whose initializer at `137B630`
names `effectMap`. Its hash is `433FB70C`. The pinned current PatchData
globals payload `A426730B:2A3CE5B7:00000000`, SHA-256
`e8c4b80348fd8f3a1821533c87110315ba584f66f081d46305ff61ab6cbf8c72`, has
raw `+3C = AACC3A75`. Exact `AACC3A75:433FB70C` entries exist in PatchData
and Spore_Game. PatchData's 680 decoded bytes SHA-256
`e6d627dc5e3e55e71670395a5c3a29e10c4a194d9545aec8410e9c3aeb3303aa` project
24 rows of 28 bytes at offset 8. The first two have raw type 5 and IDs
`5A314EBD` and `5A314EB9`. Treating the raw array as the native fixed-up
array requires explicit loader, handle, bounds and lifetime admissions.

The explicit PatchData EFFDIR `40450200C1195706`, decoded SHA-256
`578cd63c8efc8fe5141d47c0101987d65e2f256e81bb88bc76895c9d303a6036`, exports
those IDs as visual indices 4809 and 4810. Each contains one valid type-8
component, respectively 483 and 484. A complete bounded walk of that type-8
section reaches its exact end; interpreting its component records using
[SporeModder FX's ModelEffect format](https://github.com/Spore-Community/SporeModder-FX/blob/master/src/sporemodder/file/effects/ModelEffect.java)
gives group-zero model IDs `44C489B7` and `154C862B`, size 1, white RGB,
alpha 1, no animation and default material. The visual definitions, provider
precedence, runtime modifications and effect selection are not observed.

PatchData PROPs for both IDs each explicitly name same-ID `2F4E681B` at
`MODEL_LOD0`; Spore_Graphics holds the corresponding RW4s. The
`44C489B7` RW4 SHA-256 is
`bbb3fca35d99c40de940a6d5bf9f6b59717661c79be3f7eae164cbcec2844ec2`.
Its mesh-state link is `[7,1,8]`; mesh 7 has four vertices and six 16-bit
indices `[0,1,3,1,2,3]`. Positions are the unit square at z=0, with UVs
`(0,0),(1,0),(1,-1),(0,-1)`; V wrapping is essential. State 8 has shader
`80000033` and saved stage 0 points to raster section 11, a 256x256 DXT1
texture with nine mip levels stored in base resource 10. Its saved sampler
pairs `(1,1),(2,1),(5,2),(6,2),(7,1)` encode wrap U/V, linear min/mag and
nearest mip filtering. Saved state does not establish live shader or sampler
selection. The other RW4 SHA-256 is
`7b54e34f27d07c1526f475f37fa259c10aa8fe4d0ae5683dc149b7398e4f4d5e`;
its linked mesh has 1,701 vertices and 9,600 indices (3,200 triangles), shader `80000002`, and
two embedded rasters.

The narrow useful next test is a source-only, hash-checked plane inspection
with exact vertex/UV data and saved stage-zero raster and sampler, an explicit
caller-provided camera/transform and lighting, and a controlled offscreen RTX
capture. Original shader execution or a separately labeled PBR approximation
must be stated explicitly. It cannot supply actual effect transforms, environment
identity, active resource precedence or retail camera until those are traced.

The separate `cell_effectmap_plane_inspection_v1` example now performs this
bounded inspection. It checks the exact PROP and RW4 hashes, link, saved shader,
stage-zero raster and sampler, geometry and all nine mip extents before device
creation. It uses Bevy PBR and assumes sRGB for the texture; it does not execute
shader `80000033`. Translation and rotation are fixed to zero and identity,
while plane scale, camera z and ambient intensity are explicit arguments.
Camera projection and disabled culling are inspection choices. Files must remain
stable across package reads.

A no-window RTX 3060 Vulkan pair used scale 1, camera z 3 and ambient 300.
Textured and white-material controls both exited 0; the textured output visibly
shows blue-gray rocky imagery. The images differed at 146,199 pixels, and the
textured image contained 11,200 distinct colors. Receipt
`target/source-cell/effectmap-plane-inspection-20261109-v3/receipt.json`
SHA-256 `02a2e35ad30c9cc3052e5061adaf2ef3b404b53de2b92550c769a24dd2de4079`
records the exact commands and image hashes. The example executable SHA-256 was
`cc4e5cc9ed1b364e569c040502abf8d92f3a7a85dcb0c5ff0bd57e372bb5aadb`.
Both logs report that the Vulkan validation layer was unavailable; no pipeline
error was logged. Invalid scale and mode were rejected before device creation.
An earlier v1 white control was stopped after it waited on an image deliberately
not retained by that material; its failed receipt and output remain preserved.
The v2 and v3 runs corrected that readiness requirement. This confirms the
explicit source texture is visibly bound under the stated approximation, not
that the source effect or terrain appeared at that location in the game.

## Conditional cached shader fixture preparation

For the plane's four saved declaration semantics `[0,2,19,6]`, the inspected
feature-bit projection is `00080045`. Under an explicit profile-3 PatchData,
render-type-zero context with saved data `210`, synthetic `24E=0`, all other
shader data absent, sampler zero present and sampler one absent, a provisional
selector replay gives vertex recipe `03 17 19 15 01` and pixel recipe
`05 09 2A`. The pinned pack has one cached program for each recipe. Their
bytecode SHA-256 values are
`f4fe3ac7d1faff33b17e58770f18aa8c455006447ce546ee908b4fd1e57d2540` and
`737f8a1c15fca8ce34c338aacf9ceb67996e46486c5ac758c0f4298aed100b6c`.
Cached uniform records specify vertex callbacks `22`, `6`, `8` at starts
0, 1 and 5 with 1, 4 and 3 rows, and pixel callback `20C` at start 0 with
one row. These are conditional inputs, not observed live shader selection.

A prepared, unexecuted source-only D3D9 comparison under
`target/source-cell/effectmap-plane-native-review-v1/` uses those cached
programs, source declaration, saved raster and sampler, explicit identity
model, clip-z-half, white tint and fixed target/depth inputs. A synthetic white
texture is a separate control. It uses a private non-switched desktop's
existing desktop handle and creates no application window or presentation.
Host, runner, device wrapper, paths and pins require independent review before
execution. This is a texture/control comparison, not a matching Bevy shader
comparison or proof of source effect placement or camera.

The separate v2 preparation keeps the source texture and white control at the
same 256x256 DXT1 format and nine mip levels; the control's compressed blocks
encode solid white. Its runner additionally requires its reviewed own hash and
pins its Python/Pillow modules and package-verification scripts. The v1
preparation remains preserved. Neither prepared version has been executed at
this checkpoint; independent review is required before use.

A subsequently reviewed v2 native pair executed the cached programs and
returned zero for both textured and same-format, same-size, nine-mip solid
white BC1 controls. It reported an RTX 3060 HAL, 43,704 source compressed
bytes, D24S8, z-enable 1, z-write 1, z-function 4 (`LESSEQUAL`), alpha blend
1 with source/destination 5/6 (`SRCALPHA/INVSRCALPHA`), disabled sRGB and
no culling. Both used the explicit identity model, clip-z-half matrix and
white material/tint. The two images differed at 146,107 pixels. Receipt
`target/source-cell/effectmap-plane-native-result-v2/receipt.json` SHA-256
`feb131a56e4e0ca4d4487ceb85a00252e66e461f0fc9e8b10a090c7948fc2e92`.
Its textured PNG SHA-256 is
`876877f5e70d59e62265e1a0cab7f381398fe8bbbe428c483cd7c0f8f3f3b9ee`.
The inspected source fragments for this conditional recipe project position
through the supplied clip transform, pass UV, set white vertex/material color,
and multiply stage-zero texture by white tint; this fixture has no skin-palette
or light uniform. Cached bytecode was executed; that observation does not
establish the active game recipe, camera, material, placement or draw.

A separate hidden Bevy/WGSL source-fragment interpretation drew the same raw
plane vertices and indices, identity model, clip-z-half, white material and
tint, repeat/linear/nearest-mip sampling, cull-none, sRGB-off, and source-alpha
blend factors on both color and alpha. It used a forward depth clear of 1 with
LessEqual and depth writes, and applied a D3D9 pixel-center alignment. The
native reference used D24S8 and GPU BC1 sampling; Bevy used Depth32Float and an
independently CPU-decoded BC1 mip chain. On the RTX 3060 Vulkan adapter, both
operational foreground counts were 147,456 with zero disagreement. Across
145,924 common 8-neighbor interior pixels, maximum per-pixel channel deltas
were distributed as `{0:17734, 1:103147, 2:24042, 3:956, 4:45}`. Receipt
`target/source-cell/effectmap-plane-bevy-result-v1/receipt.json` SHA-256
`c49cff66581cdcc041527c753b977e9032d94262280417fb168faeba04a78bd1`.
Bevy exited zero; the log reports an unavailable Vulkan validation layer and
no pipeline failure. The difference has not been attributed to any one cause;
the comparison does not establish general shader equivalence or live state.

## Conditional dynamic placement inputs

`E63510` obtains the fixed-up effect-map handle from resolved globals `+3C`.
It passes each type-5 row, an `E50080` result evaluated from its four float
fields at row `+08`, and row word `+18` into `E5E100`. That function scans an
object pool from global `016B4BE4 +38`, requiring each candidate's `+04` to
equal 1 and its `+08` to equal the supplied row word. Its outcome also depends
on the floating argument; no persisted pool snapshot is known.

On the conditional create path, the pool object's `+08`, `+0C` and `+10`
feed a level-indexed spacing table whose first 16 inspected binary32 words are `[10,30,100,300,1000,3000,10000,30000,100000,300000,1000000,3000000,10000000,30000000,100000000,300000000]` and grid coordinates.
The code calls `E51E50` with the level and a separately reached global object's
`+5190/+1C` state. It multiplies that result by binary32 `0.85`, divides the
current global object's `+514C` by that product, and takes the reciprocal with
ordered SSE operations. The resulting scalar multiplies both grid-coordinate
edges; their midpoint is formed with binary32 `0.5`, and the x-edge difference
is supplied with the effect object. A 3x3 basis copied from mutable globals at
`016B4D8C` and mutable vector globals `016B4C08/0C/10` also contribute to the
submitted transform. These are operation/field observations under valid finite
inputs, not a complete reconstructed transform or a live geometry claim. The
source object identity, active level, grid indices, `+514C`, current player
state, basis, callback result and update/lifetime ordering remain inputs to a
bounded placement model; the on-disk record alone cannot determine the image's
retail position or scale.

### Opt-in app plane capture and composition controls (2026-11-09)

The experimental offscreen Cell app accepts `--source-effect-plane-install <install> --source-effect-plane-rect <x,y,width,height f32 hex words> --admit-effect-plane-stable-packages`; the caller supplies placement. It loads the pinned source quad and nine CPU-decoded BC1 mips with wrap/linear/nearest-mip sampling, and uses the source-fragment-inspired white-tint diagnostic. The underlying gameplay, actor assignment and camera remain prototype or controlled inputs; neither runtime effect selection nor placement is established. The app uses an sRGB render target, Bevy reverse-Z and transparent ordering, unlike the separately matched forward-depth unorm comparison above. No app shader parity is claimed.

Initial capture v1 was occluded by the prototype's solid border rectangle; removing it in v2 exposed a black composition with mixed camera MSAA. Both are retained as failed visual controls. In the opt-in path only, the two opaque prototype background rectangles are suppressed and all composing cameras use MSAA off. The default path is unchanged. The RTX Vulkan v3 paired capture, seed 14 and tick 0, shows the source rocky image behind prototype objects and HUD: `target/source-cell/effectmap-plane-app-20261109-v3/plane.png`, SHA-256 `a092724732581937fdc811c94d314412db976186ffed2ebc1ab441301e7dae9b`; receipt SHA-256 `bd84edac5df36b69afc14c5b38d9173e00738b581bed9445fbf3df79943b756f`. Vulkan validation layer was unavailable; no shader/pipeline error was logged.

A second pair holds seed and tick at 14 and 0 and changes only the explicit rectangle from 40×28 to 80×56. Both exit zero; 489,637 pixels changed in bounding box x=219..1060, y=105..694. Receipt `target/source-cell/effectmap-plane-app-20261109-v4/receipt.json` SHA-256 `5945ed39931b5966a5651f46aaa26e9b40f1e4202a52465da472aada43001b18`; runner SHA-256 `575ddb7a90ee05c0ae3ca38561df67fb6199381ade4156063454a6e2e2dd2c3c`. This tests plane-dependent coverage, not runtime identity, color or depth parity.

### Controlled sRGB target comparison

A separate target-only fixture retains the matched source quad, white tint, point of view, sampler, blend and forward-depth controls above, changes the Bevy target to sRGB, and applies inverse sRGB to sampled RGB before output. It admits this correction only after checking every source mip alpha byte is 255. The clear color is expressed in sRGB to match the native reference bytes. The native D3D9 fixture uses sRGB off, D24S8 and hardware BC1; Bevy uses Depth32Float and CPU-decoded BC1. On RTX Vulkan, the comparison had 147,456 foreground pixels on each side with zero coverage disagreement. The 145,924 common interior pixels had maximum channel delta histogram 0:73,839, 1:66,674, 2:5,158, 3:253. Receipt `target/source-cell/effectmap-plane-srgb-result-v1/receipt.json` SHA-256 `0aeabf8bc6231dc12ddd5ebb2728c12dcfff08bed1342244e603cbba3035c238`; runner SHA-256 `168c2055d28a06b068d28b8c8ee01a2f30954f10ccdccb7c12433da6e2531f43`. This controlled opaque draw supports only that conversion under these inputs; it does not establish application reverse-Z, general translucent blending, effect selection or retail camera parity. The shared app shader was not changed in this comparison.

The opt-in app now uses a separate plane-only shader with this inverse transfer. The source loader rejects any mip with non-opaque alpha; the player preview shader and default app behavior are unchanged. A fresh RTX Vulkan app capture at seed 14/tick 0 shows the source texture behind the prototype scene. The 40×28 versus 80×56 controlled pair changes 489,641 pixels within x=219..1060, y=105..694; both runs exited zero. `target/source-cell/effectmap-plane-app-20261109-v5/plane.png` SHA-256 `131729f3e9355c38a3952c3ccffa49c2a127b9ab2c647d1bcf956fdf60ab74dc`, receipt SHA-256 `af2c4d716d18ca6239a7ad507d1df01fd9681184dd182cf1064e07b7c2a89b83`. No pipeline error was logged; Vulkan validation layer was unavailable. This app result remains a reverse-Z composition with supplied placement, and is not itself a matched native comparison.

## Conditional start-scale and installed effect curves

The pinned PatchData `AACC3A75:433FB70C` payload has two type-5 rows: ID `5A314EBD`, threshold words `[-1,-1,1.5,1.899999976158142]`, pool level 3; and ID `5A314EB9`, `[5700,14950,20000,22000]`, pool level 11. `E6354B..A0` reads the current Cell global `+514C` and supplies it with each row's four thresholds to `E50080`. The helper treats `-1` as a sentinel and uses `1041E40` with a source binary32 `2^-16` denominator addition. The first row gives a clamped falling curve; the second has a rising curve, plateau and falling curve. `E5E100` additionally requires a candidate pool record `+4 == 1` and `+8 == row+18`; a nonzero computed weight can reach the create/update path. Source/provider resolution, successful fixup and stable matching pool identities remain admissions.

A separately reviewed relocated original-instruction fixture tested 92 inputs: both installed rows, 23 finite scalar words including threshold neighbors, x87 CW `027F` and `037F`, MXCSR `1F80`. Original `E50080..E501FA` and `1041E40..1041E56` executed without the game; every result matched an independent rational x87 oracle. Result receipt `target/source-cell/e50080-selection-result-v2/receipt.json` SHA-256 `d5e218d29f45474da58ac0dff03efd1dc53fdb7e4b2574fb826d1ddcd88075f0`. The bounded Rust curve separately matched all 92 retained native outputs; receipt `target/source-cell/e50080-selection-result-v2/rust-curve-run-v1/receipt.json` SHA-256 `c46c239b9996f1a45570ddc05166b7b61d062a8288c8e320414f3e9254f67121`. These samples do not establish all x87 controls, nonfinite inputs or live state.

A separate controlled composition feeds explicit progress 0..1000 through the existing `cell_start_table_v1` model using the source PE snapshot (SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`) and its 616-byte table slice (SHA-256 `901bbc043939f2631bde8c16d457cb126dd1df3513b5fb07fbaa419a959c6583`), then into the two bounded curves. All 1001 results matched an independent table/curve oracle, whose TSV SHA-256 is `d3cb2a3c363856dbac44a4f880ff7908ab86ce9f4b0b53d34aa2a574e21fdfde`; comparison receipt `target/source-cell/e50080-selection-result-v2/rust-join-run-v1/receipt.json` SHA-256 `f2b6597dcf1bfe9e9955508e9f303df132adfef974973d04dc8911f15d51f69e`. Under these supplied conditions the first curve is one for progress 0..199 and zero for 200..1000; the second is zero for 0..899 and one for 900..1000. This composition assumes the startup path executed, the same +514C persists to E63510, the selected raw rows resolve as admitted, and the pool condition holds. It is a model join, not 1001 native executions or evidence of actual progress, placement, provider, camera or visible effect.

## Second installed ModelEffect candidate

The other type-5 row conditionally points through `5A314EB9` to ModelEffect model `154C862B`; actual row activation, mounted provider, effect instance, transform and camera are unobserved. The pinned RW4 `7b54e34f27d07c1526f475f37fa259c10aa8fe4d0ae5683dc149b7398e4f4d5e` has mesh section 7, state 8 and link section 9 `[7,1,8]`. Vertex section 0 supplies 1,701 records of stride 28 and index section 5 has 9,600 unsigned-16 indices, all within 0..1700, forming 3,200 triangles. Source positions span x/y -0.5..0.5 and z about -0.01..0.01, and U spans 0..1 while V spans -1..0. Declaration records `(offset,type,usage,rw-word)` are `(0,2,0,0)`, `(12,5,3,2)`, `(16,5,6,19)`, `(20,1,5,6)`; typed conversion must retain the recorded declaration rather than assume float normals. This corrects the earlier mistaken wording of 3,200 vertices.

State 8 saves shader `80000002`, raster stage 0 reference 11 -> DXT5 payload section 10 and stage 1 reference 13 -> DXT5 payload section 12. Both are 256×256 and nine mips, wrap U/V, linear min/mag and nearest mip. Compressed nine-mip payload SHA-256 values are respectively `263e68d27a259409e3112c7e09ece1dbeef0cf75dc9e5f1967b9c97441c97dd2` and `5a75fb7ee42641bc02e9eea3fc44f54765b34f16166daa6983b0e0c388813f1a`. CPU decode found alpha ranges 225..255 and 0..142 respectively, across all mips, so the first plane's opaque-only shader/correction cannot be assumed here. Source state records depth-write enabled and source-alpha/inverse-source-alpha blend; runtime application and cull are separate.

Under explicitly supplied profile-3 PatchData `generic_static` render-type-zero selection, declaration bits `00080045`, saved 20-byte shader data `210` (`025c4426a69b84400000803f0000803f00000000`), synthetic `24E=0`, stages 0 and 1 available, and all other shader data absent, the checked selector produces VS `17 19 15 31 01` and PS `05 09 3B 2A`. Neither occurs in that pack's compiled caches. Read-only original-template generation from the same package's pinned fragment library (SHA-256 `aaed3767f67dcae934b65f1ea2884f76c22213ae8b166fe6f8be19df16afba16`) and pinned EXE yields VS HLSL SHA-256 `9b5583a24c499791e666b8d56757696c289304d42e135c18ca76f788895d7080`, PS `38fdc205f51f7594aded79150b892865d5a9c5045c4c0f8deabc0896b067d3b5`. The generated PS samples only sampler 0; stage 1's existence in saved state is not itself proof this conditional shader samples it. Generated callbacks are VS `22@0` (1 row), `8@1` (3), `6@4` (4), and PS `20C@0` (1). These are an admitted fixture, not a live profile or runtime uniform observation.

A three-control original-template D3D9 host is prepared at `target/source-cell/effectmap-154-native-review-v1`, not executed at this checkpoint. It pins generated HLSL, inputs and the D3DX compiler, and draws source-both, stage-0-white and stage-1-white BC3 controls using explicit identity model/clip and white tint on a private desktop. Host and runner require independent review before execution. No inference about visual fidelity or unused texture at runtime follows from generated-source inspection alone.

The reviewed original-template host subsequently ran three no-Present controls on an RTX 3060 HAL using the explicit context above, identity model/clip-z-half and white tint, D24S8 with Z enabled/write/LESSEQUAL, SRCALPHA/INVSRCALPHA, cull-none inspection override and sRGB off. The source-both and stage-1-white images were identical; changing only stage 0 to white altered 147,456 pixels. All three exited zero and private desktops closed. This supports stage-0 dependence and the conditional generated shader's stage-1 independence for this draw only. Receipt `target/source-cell/effectmap-154-native-result-v1/receipt.json` SHA-256 `9fbd1f57861a404df0588bd85734a239d511fbb24ec12884819e5507f4bf8471`; generated vertex bytecode SHA-256 `615481ebca795ad6a2bb9057441dc057ec95cec50851cedb9f1cfcbea5cad1f5`, pixel `43ee500b0714d546e0abb42bde46acd0914de49e36846b3c2b300a00679abff0`. Actual effect selection, other runtime shader inputs and scene remain unknown.

A separately reviewed Bevy/WGSL scratch draw used the same 1,701 positions and 9,600 indices, stage-0 CPU-decoded BC3 mip chain, explicit clip and tint, wrap/linear/nearest mip, source-alpha blend factors for both channels, cull-none and a forward depth clear of 1 with LessEqual/write. Stage 1 is not sampled by the specified generated pixel recipe. Native uses D24S8 and hardware BC3, while Bevy uses Depth32Float and CPU BC3. Under these fixed inputs on RTX Vulkan, both operational foreground counts were 147,456 with zero disagreement; 145,924 common 8-neighbor interior pixels had maximum channel deltas `{0:14081,1:111630,2:19882,3:331}`. Receipt `target/source-cell/effectmap-154-bevy-result-v3/receipt.json` SHA-256 `706c8cee43c4cc8b220267e3c33c75d3f634ea5d65f05e4f0e249279736e3af9`. This is a conditional forward-depth unorm-target comparison, not general shader or application parity. In particular, the earlier opaque-plane inverse-sRGB approach has not been validated for this translucent stage-0 texture: attachment blending and destination color must be compared explicitly before any app use.

A separate reviewed sRGB-target control drew this translucent source mesh using the same inputs, inverse-sRGB sampled RGB, unchanged source alpha and source-alpha factors. The sRGB clear round-tripped to `(24,32,46,255)`. On RTX, both operational foreground counts were 147,456 with zero disagreement; 145,924 common interior pixels had RGB maximum-delta histogram `{0:80338,1:63999,2:775,3:81,4:214,5:454,6:62,8:1}`, and alpha delta `{0:143663,1:2261}`. The 731 RGB pixels above three units and maximum eight distinguish this from the earlier opaque fixture; no single cause is established, and D24/D32 plus hardware/CPU BC3 remain differences. Receipt `target/source-cell/effectmap-154-srgb-result-v1/receipt.json` SHA-256 `df8df7c8719d3053e16c8a4c8f362571b668be4248d8912339ceb48c9e1c57fd`. The inverse transfer is not promoted into application rendering for this translucent effect. One fixed background and camera cannot establish general compositing behavior.

A reviewed two-pass control instead rendered the entire fixed test scene into an unorm target with the native-like blend and depth settings, then fetched each finished pixel without blending, inverse-transferred RGB and wrote once into an sRGB target while preserving alpha. On RTX the two captured raw attachments were byte-identical. Both had 147,456 operational foreground pixels and zero coverage disagreement against native; across 145,924 common interior pixels the RGB max-delta histogram was `{0:14141,1:111570,2:19882,3:331}`, and alpha delta `{0:143663,1:2261}`. Receipt `target/source-cell/effectmap-154-twopass-result-v1/receipt.json` SHA-256 `d9ccbf456de08b7acd5fd9e145e37430f5e01b15bc7379dbbfd34d4585a23457`. This demonstrates a two-pass byte-preserving conversion for the complete controlled test scene, not the ordering, depth or color behavior of additional app actors/UI.

### Opt-in 154 app composition

The experimental offscreen Cell path now admits an explicitly supplied `--source-effect-154-install`, `--source-effect-154-rect` (four world-space f32 hex words) and `--admit-effect-154-stable-packages`; `--source-effect-154-no-effect` is a same-route control. It hash-checks the model and PROP, validates the mesh and both saved BC3 raster/mip/sampler chains, and renders the controlled stage-0 conditional fragment with source alpha. For this option only, the scene cameras route into one unorm intermediate and the final camera converts the completed image once to sRGB. UI is explicitly routed to the existing scene camera before conversion. This changes color handling of the entire opt-in scene and remains a prototype view; supplied placement, profile and shader context are not retail state, and Bevy reverse-Z/ordering are not validated against the native scene. The existing 44 model and default routes remain separate.

At seed 14/tick 0 on RTX Vulkan, no-effect, 40×28 and 80×56 rectangle captures exited 0. No-effect to large changed 489,642 pixels, small to large 488,878, both only within x=219..1060/y=105..694. The visible large capture SHA-256 is `2d19849f6102f8441e61ea86fb6b2dddc528684c1e11255f41fedf936c102777`; receipt `target/source-cell/effectmap-154-app-20261109-v1/receipt.json` SHA-256 `7b920801dc744ab9651c8560071ff3ec3888882af35138b4eeb0861fc447801a`. A separate pair with the controlled five-segment source UI and cursor showed actors/effect under the overlay: effect/no-effect differed by 489,586 pixels in the same footprint, while enabling UI changed 25,591 pixels in x=47..1034/y=40..116. Receipt `target/source-cell/effectmap-154-ui-20261109-v2/receipt.json` SHA-256 `2f7c4a85a000a86c09b00945965f39030c0d7041cab9df25cc6734a0ca076ed1`; its effect/UI PNG SHA-256 is `964b851c25f2076106b4c0bf630da9b0218942509afb5e986d9111e4140f7a89`. The earlier v1 UI receipt records a comparison-script name check error after captures and remains preserved. The Vulkan validation layer was unavailable; no pipeline error was observed.

Same-input default and 44-model tick-0 captures were byte-identical to prior references (default SHA-256 `7f1c228f6039649881a9da8a175cc37fa7ec125585c01fe6c36c5fcacda7bb0f`, 44-model `131729f3e9355c38a3952c3ccffa49c2a127b9ab2c647d1bcf956fdf60ab74dc`). Missing installed data fails before graphics. These captures establish controlled visibility and ordering for the stated fixtures, not original effect selection or a complete Cell scene match.

### Conditional placement scale helper

`E51E50` reads `[[016B4BE4]+5190]+1C` into EDX and calls the original `E4ECC0`. For explicitly bounded progress 0..999, the hash-bound source table returns tier `floor(progress/100)+1`; progress 1000 uses its separate special record and returns 10. The helper subtracts that tier from its first stack argument with 32-bit integer operations, then on the admitted non-overflowing differences returns source constants: d≤-1 -> 1, d=0 -> 0.5, d=1 -> 0.25, d=2 -> 0.125, d=3 -> 0.0625, d≥4 -> 0.03125. Its second stack argument is unused by this body. The E5E100 placement caller supplies the candidate `+8` as that first argument; candidate `+4` is the second. Actual save identity, progress, pool record and stable root remain unobserved. The values above are static source analysis, not a live placement or callback observation.

A read-only preparation at `target/source-cell/e51e50-tier-prep-v1` links original `E51E50` and `E4ECC0` spans with the pinned 22-row source table, five source constants and relocated six-target dispatch. Fixed synthetic Root/Save memory supplies progress 0..1000 and levels 3/11 under CW 027F/037F and MXCSR 1F80; the 4,004-case comparison is unexecuted at this checkpoint and requires independent review. The helper's lack of XMM instructions is visible in these bounded source bodies, but a separate joined call with E5E100 and its full native helper effects remains to be tested.

After independent review, the v2 native helper fixture executed 4,004 bounded records: levels 3/11, progress 0..1000, CW 027F/037F and MXCSR 1F80. All original helper outputs matched the source-table model; receipt `target/source-cell/e51e50-tier-result-v2/receipt.json` SHA-256 `2c1b3b6d2c3437403e5cf16ce9b6122f4882df2ff509e22f3351366a04fb6f1b`, raw output SHA-256 `a19e23af0dc22840428fa649b90092c98cfcf85d822fb8be167cf25fda18bf31`. Checked Rust `cell_effect_tier_v1` separately matched all 4,004 retained outputs and composed two earlier E5E100 placement-prefix records whose scripted helper returns are reachable. Comparison receipt `target/source-cell/e51e50-tier-rust-compare-v1/receipt.json` SHA-256 `971c29e2c36f9c7c74ed5f09537f953e46ee039ddfddcabd9468e6f80850acf3`. That composition supplies progress and +514C independently and assumes each stable; it is not one contiguous native invocation, and actual save identity, active effect, provider, grid and camera remain unobserved.

### Combined frozen inspection and source plane (2026-11-09)

The source-plane option can also be combined with `--startup-inspection-camera`;
source effect 154 remains excluded because its composite competes with that
camera routing. The two independent package-stability admissions and explicit
plane rectangle are still required. In the reviewed paired RTX/Vulkan capture,
both runs used seed 1 and tick 60; the second added the caller-supplied 80×56
plane. Both exited zero and had the same reported simulation summary. The
source texture fills the playfield with a bright crystalline pattern, making
small prototype actors hard to see; the HUD and enlarged static Cell remain
visible. The UI probe reported 309 bright pixels in each image; 477,156 pixels
changed overall, a figure including the diagnostic label. No error markers were
found in the logs. Receipt `target/startup-inspection-plane-capture-result-v1/receipt.json`
SHA-256 `061bef0b2d238f624d91a27de19d6f182e23b6a4cc134929aa60f4d514e79cee`;
plane image SHA-256 `0a417717cc71f80adf6fd8c7b5a3e5182ca5d4aff5978a0306cbb8501342c533`.
The selected asset and rectangle do not establish the game's active environment,
placement, lighting, live camera or visual match.

### Original two-row iteration and curve handoff

A separately reviewed original-instruction fixture executes E63510's bounded
iteration, E50080 and its arithmetic helper with the two exact first type-5
rows copied from the pinned PatchData payload. A synthetic two-row array and
supplied scalar/global storage are admitted; the original row stride and
callback argument handoff run in source order. Four scalar inputs under each
of CW 027F and 037F matched all 16 row/weight/level captures. E5E100 is a
scripted trap, and resolver, full 24-row ordering, destructor, actual pool,
placement, camera and live ownership are excluded. Stable package paths are
admitted. Receipt `target/cell-effect-row-join-result-v4/receipt.json` SHA-256
`d80e57b7791b4bd3b0c8827cade46df70de9dda8172a4dfc6a66894ade6c4c28`.
Earlier v1 failed assembly and the v2/v3 drafts were not executed; all are
preserved. This fixture does not establish which effect is active in gameplay.
The separate Rust comparator `cell_effect_row_join_compare_v1` reuses the
checked E50080 curve projection and matched all 16 retained row, weight and
level outputs and eight FP-control rows. The comparator does not execute source
instructions or resolve a live effect.

### Supplied-state grid placement in the inspection app

`--source-effect-plane-grid level,progress,grid_x,grid_y,514c_hex` is an
alternative to the explicit rectangle for the same hash-admitted plane. It
composes the checked tier, placement-prefix and transform projections using
CW 027F/MXCSR 1F80, fixed source table words and an explicitly synthetic
identity basis. The validated source mesh is the centered unit square; the
computed midpoint and uniform X span map its corners to the controlled grid
corners. Grid and rectangle options are mutually exclusive; offscreen mode,
install path and stable-package admission remain required. Candidate state,
progress, +514C, grid, effect branch and identity basis are supplied; this is
not a live selection or gameplay camera.

The externally pinned three-run RTX/Vulkan capture used grid input
`3,0,0,0,3f0ccccd`. The resulting rect words were
`41e7d174,41e7d174,4267d174,4267d174`; a separate run using those explicit
rect words produced byte-identical pixels. All three runs had the same
reported tick-60 simulation and no matching error markers. A third no-plane
configuration differed at 72,372 pixels in the selected ROI; it also removes
the plane camera, so that difference is a configuration control rather than
an isolated geometry test. Visual inspection shows the bright crystalline
plane in the upper-right and behind the HUD while much of the field remains
dark; this supplied placement is not representative of the live environment.
Receipt `target/startup-plane-grid-capture-result-v2/receipt.json` SHA-256
`2a741f13c241bd6f23808c33c2390a95cfebe57de91b4a4570d1c5cb2a460d06`.
Focused grid and mapping tests passed 2/2; the separate argument-gating test
passed 1/1. The capture does not establish native compositing or visual parity.

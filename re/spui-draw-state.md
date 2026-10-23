# Native UTFWin image draw state, 2026-10-23

## Measured source path

`[MEASURED]` Base SporeApp.exe SHA-256 is
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
ImageDrawable draw `97D1F0` obtains Graphics2D through `95BD30`. Its plain
image path clips in `95C1F0` and calls Graphics2D virtual slot `58`. The
measured vtable at `1441470` resolves that slot to `956860`, which obtains
Image interface `01BE6AB3` and calls image-geometry builder `955410`.
That builder maps relative UVs through the Image's saved atlas origin/span
before collecting four image vertices. This is the ordinary image path;
tiling, slicing, text and effects have separate consumers.

`[MEASURED]` Native `952800` converts the Image's flags word at offset `24`
into a batch flags word. The masks have different meanings in these domains:

| Image mask | Batch effect |
|---|---|
| `2` present | Set batch mask `2` |
| `4` absent | Set batch mask `1` |

Native `833CB0` constructs resolved serialized images and atlas images with
flags zero through constructor `957A50`. Thus this source loading branch
starts with batch flags `1`. This does not exclude later caller mutations or
other dynamic Image constructors. The saved Image word is retained, including
uninterpreted bits; it is not a directly copied batch word.

## Sampler and blend policy

`[MEASURED]` UI initialization `82F8B0` reads shader ID 1 from initialized
global `154824C` and calls `9630A0` with that ID and uniform ID `234`.
The latter builds its own compiled state. None of the 5,132 compiled states
in the twelve installed companion libraries has shader ID 1, and none of the
5,120 SMT records links to that shader. The native construction explains why
requiring such a packaged material link would miss this UI path.

The generated state explicitly configures texture units 0 and 1 as follows:

| D3D sampler state | Value |
|---|---|
| ADDRESSU `1`, ADDRESSV `2` | CLAMP `3` |
| MAGFILTER `5`, MINFILTER `6` | LINEAR `2` initially |
| MIPFILTER `7` | NONE `0` |

`[MEASURED]` Batch flush `82E990` updates MAG/MIN on both units to LINEAR `2`
when batch mask `1` is present, otherwise POINT `1`, if either texture is
bound. It updates source blend to ONE `2` when batch mask `2` is present,
otherwise SRCALPHA `5`. Destination blend is always INVSRCALPHA `6` in this
consumer. Combined with the zero Image flags in the serialized resource
branch, ordinary source images therefore use linear filtering and source
alpha blending. This is not the point-filtered, blend-disabled pixel-copy
configuration used by the earlier shader conformance probe.

The seven render states explicitly configured by `9630A0`, incorporating the
batch source-factor update, are:

| D3D render state | Value |
|---|---|
| SRCBLEND `19` | SRCALPHA `5`, or ONE `2` for batch mask `2` |
| DESTBLEND `20` | INVSRCALPHA `6` |
| ALPHABLENDENABLE `27` | `1` |
| CULLMODE `22` | NONE `1` |
| ALPHAFUNC `25` | ALWAYS `8` |
| ZENABLE `7` | `0` |
| FOGENABLE `28` | `0` |

`[MEASURED]` Numeric enum names were checked against Microsoft's installed
Windows SDK 10.0.28000.0 `shared/d3d9types.h`. In particular, state `28` is
FOGENABLE; ALPHATESTENABLE is `15`. The above native writes do not establish
alpha-test enable, depth writes, separate-alpha blending, color-write masks,
address W, sRGB state or the rest of the inherited device state.

`spore_core::spui_draw` implements this measured subset without a Windows
dependency. It retains raw Image and batch words, exposes only the measured
render/sampler pairs, and distinguishes the two source clipping shader paths.
The module does not load a texture or turn these facts into completed SPUI
screen acceptance.

## Shader clipping branch and uniform upload

`[MEASURED]` Native `82E630` selects shader ID 1 initially. When the field at
`[15FE918 -> +3C] + B8` is nonzero, it saves the device's scissor enable/rect,
enables D3D SCISSORTESTENABLE `174`, and selects shader ID 2 instead. Later
draw code converts clip edges to integers and updates the scissor rectangle.
The actual field value in a retail frame has not been captured here.

The measured PatchData profile-3 pack, key
`0469A3F7:4021200400000003`, contains both source programs for render type 0:

| ID / name | VS bytes | PS bytes | Vertex uniform |
|---|---:|---:|---|
| `1` UTFWin_UIShader | 1,560 | 696 | `234 / 234`, 96 registers, flags `8` |
| `2` UTFWin_UIShader_NoClip | 1,500 | 632 | same |

SDK reflection of the original bytecode shows texkill clipping in ID 1's
pixel shader and its absence in ID 2. Both sample unit 0 and produce the
same multiply-plus-add texture/color expression. These are inspected original
programs, not new shaders compiled from a reconstruction. Both now pass the
controlled packed-vertex copy and blend fixtures described below.

`[MEASURED]` Uniform callback `82E3E0`, registered by `82E550`, reads shader
data `234` at `16F7E78`. The first word is an object count and data follows at
offset 4. The callback uploads exactly six float4 registers per object, using
the supplied register start and stage flag. The descriptor capacity is 96
registers for sixteen UI objects; the callback does not always upload all 96.

## PNG conversion evidence and limits

`[MEASURED]` Native general PNG reader `84E5C0` uses its bundled libpng
1.2.25 and calls `84DEC0`. In the latter's 32-bit destination branch, source
PNG color type 6 copies R to destination byte 2, G to byte 1, B to byte 0 and
A to byte 3. Color type 2 uses the same RGB swizzle and sets A to `FF`.
There is no alpha multiplication in these conversion loops. Rows advance by
the supplied destination pitch.

`[MEASURED]` The decoded title atlas `31A448931264E677` has 56,652 texels with
partial alpha; 54,041 of those have at least one RGB byte greater than alpha.
It also has 11,618 fully transparent texels with nonzero RGB. Thus the source
image cannot be treated as already premultiplied merely because the output
target supports alpha. This is a source-image observation, not a capture of
the retail upload buffer.

`[UNKNOWN]` The complete PNG-resource-to-UI-raster upload route, all libpng
transform behavior, later Image flag mutations, runtime texture formats and
color-space decisions still need a targeted capture or more native consumers.
The observed general decoder and zero Image flags support preserving straight
RGBA for ordinary resources, but do not certify every dynamic texture source.
Batch mask `2` establishes a different source blend factor; it alone does not
establish where any required premultiplication occurs.

## Evidence and next verification

Native retained Evidence is under
`target/rea/20261022-source-runtime-v3/`: queries 205, 214-219, 229-231,
243 and 244. Bounded Microsoft SDK disassembly is under
`target/source-ui/ui-*-sdk.asm`; original shader bytes, reflection and hashes
are in `ui-state-source-programs-v1.json` and its referenced target files.
Original payloads and decompilation remain ignored.

Both source UI programs now execute offscreen with partially transparent
source PNGs and the measured blend/filter policy. Scaling, edge-crossing
linear versus point fixtures, complete PNG upload policy and a retail screen
comparison remain additional gates. A matching controlled draw validates our
binding and arithmetic; it does not close original-screen acceptance.

The public [Graphics2D interface declarations](https://raw.githubusercontent.com/Spore-Community/Spore-ModAPI/master/Spore%20ModAPI/Spore/UTFWin/Graphics2D.h)
and [Image interface declarations](https://raw.githubusercontent.com/Spore-Community/Spore-ModAPI/master/Spore%20ModAPI/Spore/UTFWin/Image.h)
were consulted for method/type names after native slot recovery. Their
implementation was not copied; the policy above comes from the installed
executable and package data.

## Native vertex, viewport and object batches

`[MEASURED]` `9630A0` and declaration finalization `11F2FD0` establish a
24-byte stream: FLOAT2 position at 0, D3DCOLOR COLOR0 at 8, D3DCOLOR COLOR1
at 12 and FLOAT2 TEXCOORD0 at 16. `82E990` copies a 20-byte command vertex
and inserts the control word. `82EDC0` packs object index times `600` plus
`FF` when textured. The normalized G channel selects six-register object
offsets, while B selects textured versus solid drawing. Indices 0 through
15 are valid; larger indices are not silently wrapped by `UiVertex`.

`[MEASURED]` `95BE80` creates viewport scale `2/width, -2/height, 0` and
translation `-1-(left+h)*sx, 1+(top+h)*(2/height), 0.5`, where the backend
explicitly selects h = 0 or 0.5. `95E930` stores a row-major scale/translation
matrix. `95F980` composes local times viewport, using the kind-zero identity
shortcuts. `45F430` accumulates four float32 products sequentially without
FMA. `UiMatrix` and `UiViewport` preserve those rules, native kind words,
clip order and the caller's explicit half-pixel capability.

`[MEASURED]` Each object's six-register upload is RGBA tint, sixteen matrix
floats and projected clip. Tint conversion multiplies each byte by stored
float32 reciprocal `3B808081`; division gives a different float bit pattern
for 126 of the 256 possible components. Shader clipping stores left, bottom,
right, top after projection. The scissor branch supplies a generous shader
clip instead. Current retail backend selection remains unobserved.

`[MEASURED]` The optional `source_ui_shader_probe` now uses the actual packed
vertex contract and native viewport. Sixteen differently translated local
strips exercise all 96 uniform registers, 64 vertices and 32 triangles. The
source 512 x 256 title atlas contains 56,652 partial-alpha pixels. Both
original shader IDs pass copy and source-alpha blend fixtures on the RTX:
all four 131,072-pixel outputs have zero RGBA error against the independent
integer/pixel calculation. Separate-alpha and sRGB are explicitly disabled
for these fixtures, not inferred as retail settings. There is no Present,
visible window or desktop input.

Evidence: `target/source-ui/native-batch-v1/{runs,independent-comparison}.json`.
The earlier single-object packed-input fixtures also pass in
`packed-state-v1/`. Reproduce with `tools/build.bat build -p spore-bevy
--example source_ui_shader_probe --features native_shader_probe`, then run
the executable with install, `png:31A448931264E677` and a new output path.
Optional `--native-blend` and `--scissor` select the two independent branches.

Native v3 queries 285, 286, 288-290 and bounded SDK evidence in
`ui-native-vertex-uniform-v1.json` retain the interface and float provenance.
Full WinProc ordering, screen transforms, effects and events remain open.

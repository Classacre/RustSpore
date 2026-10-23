# Original shader execution on the RTX

`[MEASURED]` The optional Rust `source_ui_shader_probe` runs the installed
`UTFWin_UIShader` vertex/pixel programs directly through hardware D3D9. It reads
the original token streams from PatchData profile 3, selects shader ID 1 and
render type 0, and validates both shader names and profiles before creation.
The programs are not embedded, rewritten or recompiled in the repository.

Two direct-package checks passed on `NVIDIA GeForce RTX 3060 Laptop GPU`, vendor
`10DE`, with hardware vertex processing:

| Source package / PNG packed instance | Dimensions | Pixels changed | Maximum channel error |
|---|---:|---:|---:|
| Graphics / `31A448931264E677` | 512 x 256 | 0 / 131,072 | 0 |
| Content / `40616200780A9EAF` | 128 x 128 | 0 / 16,384 | 0 |

`[MEASURED]` The second image is Jawhead's actual saved thumbnail. Comparison
covers all RGBA bytes read back from the GPU against the decoded source PNG.
Both commands exited zero. A missing draw would retain the deliberately different
clear color and fail that comparison. Images and complete logs remain under
ignored `target/source-ui/original-shader-{atlas,thumbnail}-v1.*`.

## Device and interface

The probe creates an invisible, nonactivating 1-pixel helper window required by
D3D9 device creation. It never calls ShowWindow or Present, never targets another
window, and never reads or sends desktop input. Rendering uses a separate
offscreen surface at the source image's dimensions, with explicit GPU readback.
The helper window and COM objects are released at the end of the run.

The measured shader interface has POSITION, COLOR0, TEXCOORD0 and COLOR1 vertex
inputs, plus a 96-float4 constant array for sixteen UI objects. Each object has
tint, four transform rows and a clipping rectangle. The conformance draw uses
white tint, identity transform, a generous clip rectangle and the source image
with point sampling. A D3D9 half-texel correction aligns source pixel centers.
These controlled bindings isolate the original program's texture/color path;
they do not claim a complete SPUI layout or the original game's sampler policy.

## Optimus export correction

`[MEASURED]` The first executable contained no PE export directory despite its
Rust `no_mangle` global. D3D9 exposed only AMD and the probe deliberately refused
to substitute that adapter. Adding the actual `NvOptimusEnablement` PE export
made the RTX appear on the next run. The MSVC linker directive is local to this
example's object file, and the normal build command now produces that export.

NVIDIA's [Optimus policy documentation](https://developer.download.nvidia.com/devzone/devcenter/gamegraphics/files/OptimusRenderingPolicies.pdf)
specifies that the driver looks for that exported DWORD and uses its low bit.
The implementation was checked against the resulting PE export directory and
the runtime adapter identity; no global GPU setting or driver profile was changed.

## Reproduction

```powershell
tools/build.bat build -p spore-bevy --example source_ui_shader_probe --features native_shader_probe
target/debug/examples/source_ui_shader_probe.exe '<install>' png:31A448931264E677 target/source-ui/original-shader-atlas-new.png
target/debug/examples/source_ui_shader_probe.exe '<install>' png:40616200780A9EAF target/source-ui/original-shader-thumbnail-new.png
```

Output paths must be new. The UI conformance example is skipped by default
workspace builds. Native Cell rendering now has a separate default-enabled
`native_renderer` feature on Windows; `--no-default-features` excludes it.
It requires Windows/MSVC and the measured profile-3 source pack. File PNG input
also exists for standalone conformance checks, but the evidence above uses direct
package reads with the exact resource key logged.

`[UNKNOWN]` This proves original bytecode can run on this GPU and reproduce the
controlled source-image draw exactly. It does not prove SPUI class behavior,
font layout, event handling, creature material selection, bone upload, game
lighting or stage fidelity. Integrating the original shaders with actual game
draw bindings remains work; the general scene viewers still use Bevy PBR.

## Controlled original 3D draws

`[MEASURED]` On 2026-10-23, all 28 unique primary Cell creations resolved
from 65 source definition rows executed their original compiled 3D programs
on the RTX. Each draw uses untouched GMDL vertex bytes and the saved D3D9
declaration, source indices, material parameters and all three source texture
units. Exact source DXT blocks or raw BGRA and every declared mip level are
uploaded. A global RW4 texture is resolved by its own source key. No substitute
paint, generated normals or hand-authored shader replaces these inputs.

The native draw functions `760FB0` and `73EDF0` bind shader-data marker `22C`
when the first semantic-2 element is UBYTE4 and native unpacking is enabled.
Both measured default enable bytes are one. This selects the original normal/
tangent unpack fragment; a saved declaration mask alone misses that draw flag.
Jawhead and Stabber now resolve VS recipe `14 17 19 15 31 30 29 01`
(3,196 bytes), PS `05 09 0C 11 3B 19 2A` (3,580 bytes). The earlier 2,964-byte
VS omitted unpacking and was only a deliberately incomplete static audit.

`[MEASURED]` The controlled light input reads the actual editor RGB coefficients
and exposure. The first sixteen coefficients retain the native float32 basis
undo/accumulate/redo sequence, checked against an independent Decimal-100
calculation. Direct-light packing also accounts for the runtime direction
negation followed by the upload callback's second negation. See
`cell-lighting.md` for exactly which frame and source options are supported.

Evidence: `target/source-cell/native-corpus-v1/report.json`, its 28 images/logs,
and native v3 queries 124-162. The binary digest is retained in the corpus
report. Every process exited zero, confirmed the RTX, wrote a nonempty 768 x 768
draw and never presented a window. The first two pre-fix captures under
`*-original-program-v1.*` visibly exposed the mip-prefix defect and are not
acceptance images. Corrected corpus images retain full original source mips.

`[UNKNOWN]` This is a defined `generic_static`, profile-3, render-type-0
inspection context. The camera uses recovered source orientation/FOV with an
explicit bounds fit. Model/world fit, culling/blending, lighting frame, white
material/tint controls and later retail material overrides still require game
draw comparison. A controlled original-program draw is not an identical retail
screenshot, animated creature, functional stage or completed editor.

```powershell
tools/build.bat build -p spore-bevy --example source_cell_shader_probe --features native_shader_probe
target/debug/examples/source_cell_shader_probe.exe '<install>' target/source-cell/new-original.png 40617100780A9EAF
python tools/source_cell_native_corpus.py '<install>' target/source-cell/cell-model-inventory.json target/debug/examples/source_cell_shader_probe.exe --out target/source-cell/new-native-corpus
```

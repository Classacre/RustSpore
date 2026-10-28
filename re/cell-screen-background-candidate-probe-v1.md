# Screen-background candidate offscreen probe

This is an exploratory combination of installed source assets, not an in-game
capture or a validation of the actual runtime material, texture, or appearance.
The 768×768 output is a near-white textured pattern with the assumed white
color. No desktop presentation was used.

The coordinating agent reports this build and direct invocation for v2. The
executable was launched with Windows `CREATE_NO_WINDOW`; the log itself does
not retain the command. Use a new output path when reproducing it:

```powershell
tools\build.bat build -p spore-bevy --features native_shader_probe --example source_screen_background_probe
target\debug\examples\source_screen_background_probe.exe "D:\Games\SPORE Collection\Data\PatchData.package" "D:\Games\SPORE Collection\Data\Spore_Graphics.package" "target\source-cell\screen-background-candidate-probe-v2.png"
```

The v2 log records the exact PatchData pack key
`00000003:0469A3F7:40212004`, profile 3, render type 0, and named shader
`70000091 ScreenEffectsTextureShader`. The explicitly supplied Graphics package
provided key `BD08BD8A:2F4E681B:00000000`: a 256×256 DXT5 RW4 raster,
nine mips, with 87,408 source bytes uploaded. The 681B resource is an
inventoried fallback candidate for the 681C request, not proven runtime
resolution. Package authenticity, provider precedence and uniqueness of the
first raster are unverified.

The probe used the source quad helper with 768×768 assumed viewport, zero
unverified offsets, white assumed color, and MXCSR `1F80` fixture. Its
float3/color/float2 declaration, indices `0,1,2,0,2,3`, disabled culling,
no mip filtering, sRGB disabled, and clear color `FF18202E` are diagnostic
choices. The candidate stored render states disable depth, depth write, alpha
test, and alpha blending, with alpha reference 127/function 8; the stored
sampler uses clamp U/V and linear min/mag. Effective runtime state remains
unknown. The first successful device identified an NVIDIA RTX 3060 Laptop GPU;
`presented=false` was logged. Primitive type is supported by static evidence,
while actual index-buffer contents and shader/vertex binding remain unresolved.

Evidence retained under ignored `target/source-cell/`:

- `screen-background-candidate-probe-v2.png`: SHA-256 `1135f77e830c04d874c9681d46290822d92f6deec96e26697059d8686a1538f5`.
- `screen-background-candidate-probe-v2.log`: SHA-256 `a6efc50f7c062a8340fd66d4ad88525d19648f99e88b478c4a50bb7d79b987e4`.
- `screen-background-candidate-probe-v1.log`: SHA-256 `5fdac4cdf23ad36dc865428cd7082a0ae3a23a393d300dae7726482f09be7029`. It found only the AMD adapter and failed the renderer's RTX 3060 requirement. The example then gained the same Optimus export used by the existing native renderer before v2.

The coordinating agent reports a second successful offscreen run, v3, using
the profile-3 `7000008C ScreenEffectsTextureShader` from Spore_Graphics and
the same texture package. Its 768×768 pixels had zero differences from v2;
both PNG files have SHA-256
`1135f77e830c04d874c9681d46290822d92f6deec96e26697059d8686a1538f5`.
The v3 log SHA-256 is
`41ee325c2558a06a0ebf08f581a4fdd9417ab3997e9434261de2aeb75fdf7420`.
This comparison applies only to these particular probe inputs and outputs;
it does not establish general shader equivalence or runtime selection.

The package index size and count are preflight bounded, but the subsequent
reader reopens the file; inputs must remain trusted and stable across that
TOCTOU window. No actual game execution, active skybox, runtime material
selection, final state, or visual equivalence is established by this probe.

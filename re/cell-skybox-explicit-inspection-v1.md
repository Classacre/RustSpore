# Explicit Cell skybox candidate inspection

The `source_screen_background_probe --inspect` mode joins one explicitly
supplied decoded EFFDIR, shader-pack resource, and texture resource before an
offscreen draw. It does not select the active game directory or background.
The EFFDIR bytes, decoded shader pack and decoded RW4 are each checked against
caller-supplied SHA-256 values before the D3D device is created. The directory
reader requires one `00000000:05C7A4D0` local visual export, exactly one local
type-9 screen reference, and the inspected mode-5, flags-1 path. The screen's
texture group and instance determine the requested B-type resource key. Other
effect components and opaque section bodies are not interpreted by this join.

The shader package, shader ID, texture package, viewport, offsets, color and
hardware adapter are explicit inputs. The named profile-3 shader and unique
render-type-0 variant are checked, as are the selected candidate raster's
dimensions, mip count, format and complete compressed mip lengths. The type-B
resource is a candidate for a type-C request; actual provider selection,
conversion and runtime render state remain unobserved. Shader and texture
hashes bind the decoded bytes consumed in this run, not package authenticity.
The index reader still reopens the path after bounded preflight, so the caller
must supply trusted files stable across those reads. The shader, offsets,
viewport, color, blend/depth state, index order, camera and world selection
are inspection admissions, not recovered live state.

One test used the retained PatchData variant 0 EFFDIR. It joined export
`05C7A4D0` to component 186 and texture `00000000:BD08BD8A`, decoded shader
pack SHA-256 `ab5cc3cf86340ec1f7489cda7f801e8bc2d969e6e8a3babd8b1f34734a506409`,
and RW4 SHA-256 `5705df1ab03e94c58c8a890383026e68c68dfe64ba488d4d767f2706d8ad8aec`.
The available D3D9 device was AMD Radeon, hardware vertex processing, with no
presentation. The white 768x768 inspection PNG has SHA-256
`923ab6913a814026af3f30f49dbd87df49e6ee48ca6bb3c7738b2914cd20e8bc`, and its
log SHA-256 is `73f9de0b5fed18b1457c3aeba6fb78ceb5292d834485d66cd08bac6076db821f`.
Compared with the earlier controlled RTX probe PNG
`1135f77e830c04d874c9681d46290822d92f6deec96e26697059d8686a1538f5`,
21,620 of 589,824 decoded RGBA pixels differed; every channel difference was
at most one. This comparison does not isolate the cause of the differences or
establish equivalence to the game.

Three deliberately wrong expected hashes were each rejected before device
creation and produced no PNG. A finite but overflowing offset was also rejected
when it produced a non-finite quad, before device creation. A separate 320x240 render with offsets `(1,-1)`
and color `(0.25,0.5,1,1)` completed, exercising the supplied values; its
center pixel was `(56,111,222,255)`. Its PNG SHA-256 is
`7f34a0b183bdd8724b5fcf4b39062d8ee77260d32f74411b937c28be819a635f`.
The combined rejection and tint log SHA-256 is
`4d4e0f79f8dec469c1e41c4fb7fa1dbae2d7c711d3259ef262866a3b9413022e`.
These retained files live under ignored `target/source-cell/` and are not
retail screenshots. Three focused selection tests passed.

Rebuild using `tools\build.bat build -p spore-bevy --features
native_shader_probe --example source_screen_background_probe`. On this install,
the following supplies the exact tested files and assumptions; use a new output
path and a hidden process if reproducing the offscreen check:

```powershell
target\debug\examples\source_screen_background_probe.exe --inspect `
  target\source-cell\skybox-effdir-payloads-v1\PatchData-0001FFFF-EA5118B0-40450200C1195706.bin `
  578cd63c8efc8fe5141d47c0101987d65e2f256e81bb88bc76895c9d303a6036 `
  "D:\Games\SPORE Collection\Data\PatchData.package" `
  ab5cc3cf86340ec1f7489cda7f801e8bc2d969e6e8a3babd8b1f34734a506409 70000091 `
  "D:\Games\SPORE Collection\Data\Spore_Graphics.package" `
  5705df1ab03e94c58c8a890383026e68c68dfe64ba488d4d767f2706d8ad8aec `
  768x768 0 0 '1,1,1,1' amd target\source-cell\new-inspection.png
```

The example records the admitted inputs and selected adapter; it does not
establish a Cell gameplay or visual-fidelity milestone.

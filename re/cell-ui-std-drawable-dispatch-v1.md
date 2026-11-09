# Cell StdDrawable source dispatch audit v1

This bounded static trace uses `SporeBin/SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Reproduce with `python tools/cell_ui_std_drawable_dispatch_audit_v1.py
'D:\Games\SPORE Collection\SporeBin\SporeApp.exe' --out
target/source-ui/cell-ui-std-drawable-dispatch-v3.json`. The audit verifies the
digest, class name, constructor primary vtable and mode jump table, and retains
instruction bytes and linear disassembly of nine bounded ranges. This is not
native execution or proof of a displayed frame.

`988430` constructs StdDrawable with primary vtable `1445DA8`, stores 1 at
object +30, and zeros eight pointers at +10..+2C. Its vtable +10 entry is
`987D50`. That entry reads the word at its third stack argument, calls selector
`987A00`, and enters its drawing route only if the selected pointer is nonzero.
The empty-pointer branch instead calls `95D8A0`, with a further `95DB60` call
under a state bit. Selection of any source state remains unobserved.

`987D50` calls helper `987C00` with the selected pointer. In the helper,
object +30 selects a four-entry jump table `987D40`: values 1, 2, 3 and 4
branch to `987C70`, `987C8F`, `987CC6` and `987D06` respectively; other
values return. Property initializer `1300AD0` maps `EF3C000A` to +30,
`EF3C000B` to +44 and `EF3C000C` to +34. All five progress segment
StdDrawables in each explicit candidate serialize `EF3C000A = 2`, two +44
floats of 1, and four +34 floats with bit pattern `3EAAAAAB`. Thus, conditional
on successful application of those properties and the selected pointer being
non-null, the relevant helper route is `987C8F` and calls `95D120` with the
two +44 values and a locally constructed rectangle containing the four +34
values. Runtime mutation or a different source copy can change that route.

## Mode 2 operands and calls

With entry-stack arguments numbered from 1, `95D120` receives a Graphics2D-like
receiver, destination rectangle D, selected Image pointer, relative UV rectangle
U, split rectangle S, and horizontal and vertical scale floats. Its Image fields
+1C and +20 feed signed-integer-to-float conversion; `957A50` stores the two
constructor size arguments at these offsets. `987C00` constructs S as
`[+34,+38,1-(+3C),1-(+40)]`. The candidate source yields approximately
`[1/3,1/3,2/3,2/3]`, with binary32 operations preserved.

`95D120` forms scaled border lengths from the image integer dimensions, U spans,
scale inputs and S edges. It adds the first border to D's left/top and subtracts
the second from D's right/bottom. If opposite interior edges cross it replaces
both with `(edge1+edge2)*0.5` in that order. It independently forms UV split
positions from U and S. The constants are exact binary32 `1`, `0.5` and `0`.
For finite ordered inputs, the four corner calls are unconditional; if interior
width is positive it additionally calls the top and bottom center strips, if
interior height is positive it calls left and right center strips, and if both
are nonzero it calls the center. The maximum is nine calls, in that order. Each
call passes a destination rectangle, the same Image pointer, and a relative UV
rectangle through the receiver's virtual +58. The previously measured
Graphics2D vtable `1441470` maps +58 to `956860`, which requests Image interface
`01BE6AB3` and calls geometry builder `955410`; its UV mapping is described in
`spui-draw-state.md`. Actual receiver identity is conditional. The helper does
not itself prove the clip state or a retail draw.

`spui_std_drawable_mode2::mode2_calls` now accepts explicit by-value D, U,
split, scale and integer image dimensions and returns ordered destination and
relative-UV pairs for an image path. Two focused synthetic tests passed with
`tools\build.bat test -p spore-core spui_std_drawable_mode2`. A separately
written Python transcription of the source arithmetic and individual callsite
stores matched 706 fixtures (four collapse variants, one nonfinite rejection,
one finite-edge strip-test overflow rejection,
500 source-third split cases and 200 non-unit UV/scale cases) bit-for-bit with
the Rust driver. Reproduce via `tools\build.bat build -p spore-core --example
spui_std_drawable_mode2_verify` and `python
tools/spui_std_drawable_mode2_reference_v1.py
target/debug/examples/spui_std_drawable_mode2_verify.exe`. Both implementations
assume ordinary round-to-nearest, no FTZ/DAZ, and distinct by-value inputs;
the Python comparison rejects nonzero subnormal intermediates. The finite-input
and output rejection, including an overflowed strip-test subtraction, is model
admission, not a native error path. Source reloads destination and UV arrays
after indirect calls; the model assumes stable inputs and a callback that does
not mutate them. This is an
independently written instruction-order transcription, not execution or
emulation of the native instructions; correlated interpretation errors remain
possible.

An actionable fixture can explicitly select either source layout and atlas
copy, a resolved Image and state, parent spans and transforms, plus D, U and
scale inputs. Its admitted calls can be fed to the existing image-region and
rendering path as a controlled source-asset fixture. `987D50` also
combines effects and may invoke the helper repeatedly, so a single mode-2 pass
is a controlled fixture, not full native draw behavior. Parent matrices, draw
order, resource precedence, text and dynamic state remain separate evidence
gates.

## Controlled single-segment source-atlas preview

The opt-in example `cell_ui_progress_source_preview_v1` resolves a caller-chosen
segment and atlas from explicitly supplied package paths. It admits only the
two previously audited layout payload hashes and two audited PNG payload
hashes, checks the exact segment → one ProportionalLayout → StdDrawable →
selected Image references and mode-2 property shapes, and refuses unsupported
UVs, missing/ambiguous references, empty output and overlapping nontransparent
samples. Its explicit parent spans, origin, state and slot occupancy are
fixture inputs; it assumes one proportional dispatch, a clamp and floor,
identity parent matrix plus translation, a single mode-2 pass, white tint and
software bilinear sampling. It does not implement effects, full clipping,
blend-state parity, other controls or actual runtime visibility. Image and
layout hashes identify decoded bytes at intake, not atomic stable-file
admission of concurrent package changes.

After `tools\build.bat build -p spore-bevy --example
cell_ui_progress_source_preview_v1`, an explicit PatchData fixture was run:

```powershell
target\debug\examples\cell_ui_progress_source_preview_v1.exe 'D:\Games\SPORE Collection\Data\PatchData.package' 'D:\Games\SPORE Collection\Data\PatchData.package' 051B54C0 447A0000 42700000 42200000 42200000 0 01 300 100 target\source-ui\cell-progress-source-preview-v3.png target\source-ui\cell-progress-source-preview-v3.json
```

This yielded 9 modeled calls and 4,997 nontransparent samples. A separately
selected Spore_Graphics layout and atlas with segment `051B54C4` and x-origin
bits `C43E0000` yielded 9 calls and 5,033 samples. Both reports include input
hashes, output PNG SHA-256, fixture input bits and all emitted rectangle pairs;
the output hashes were independently verified. The v3 PNGs were byte-identical
to the visually inspected v2 PNGs after the model admission change. A zero
occupancy mask failed without creating outputs. These images
are source-asset diagnostics, not complete or observed game HUDs.

## Controlled original-shader segment gate

`cell_ui_progress_source_shader_v1` is an isolated feature-gated D3D9
offscreen example, derived from the measured `source_ui_shader_probe`. It
accepts explicitly selected layout, atlas and retained rectangle report,
checks the decoded payload hashes against that report and the audited source
copies, loads that Image's actual atlas region, and sends its ordered rectangle
calls through the source profile-3 UI shader. Texture size is 512×512 and
viewport/target is independently specified. The fixture selects the shader or
scissor clip path and half-pixel flag explicitly. It uses linear/clamp sampling,
SRCALPHA/INVSRCALPHA, transparent clear, white tint and separate-alpha/sRGB
disabled; these are controlled settings, not an observed retail state. The
fixture report is an explicit input, not evidence that a native runtime issued
those calls. No game process, visible window or Present is used.

Build using `tools\build.bat build -p spore-bevy --example
cell_ui_progress_source_shader_v1 --features native_shader_probe`. For the
PatchData v3 software fixture, both `shader 1` and `scissor 1` clip/half-pixel
arguments produced nine-call RTX 3060 readbacks with 4,994 nonzero-alpha pixels
and identical PNG SHA-256
`b710a38226eb382eaa99725c39947485e58897ef8f0080c164c198e346720047`.
Shader-pack SHA-256 is
`ab5cc3cf86340ec1f7489cda7f801e8bc2d969e6e8a3babd8b1f34734a506409`;
the shader-clip vertex/pixel bytecode hashes are
`470b5f56244fe399133d2407134a2bdd9e7d0a507cecaba89e547cf242f144b7` and
`110962870cc1d7c9196a5b3f9c3ee1638273977a4d0799019588d75063c25395`.
The independently run original `source_ui_shader_probe --native-blend` title
atlas control passed with zero RGBA error across 512×256 pixels.

`tools/cell_ui_progress_shader_compare_v1.py` compared all 30,000 GPU pixels
to the software fixture after applying the stated source-alpha blend over
transparent. Maximum per-channel error was 1; software had 4,997 nonzero-alpha
pixels and GPU had 4,994, with 4,994 shared, 3 software-only and 0 GPU-only.
The paired outputs and hashes are retained in
`target/source-ui/cell-progress-shader-comparison-v1.json`. This controlled
agreement is not complete UI or live-runtime acceptance. An eventual app
renderer still needs explicit admission of parent transforms, state, clipping,
effects, ordering, provider and runtime mutation, and verified behavior in its
actual graphics backend.

The D3D draw code is now extracted into `native_source_ui_renderer_v1::draw`,
which accepts a caller-owned device, target and active scene. It does not clear,
read back, end the scene or restore device state. Its caller must supply an
`A8R8G8B8` single-sample target of the stated extent, trusted validated source
shader tokens, atlas bytes and explicit ordered rectangles. Profile/end-token
checks do not establish shader safety; D3D shader creation has no token-length
argument. The fixture verifies the retained shader-pack digest before use.
The draw sets solid fill, clipping enabled, user clip planes and WRAP0 disabled,
stencil disabled and stream-zero frequency 1, in addition to its documented
shader/sampler/blend state. The standalone fixture seeds wireframe, disabled
clipping, an enabled clip plane and WRAP0 before calling it. The extracted
shader and scissor outputs both remain byte-identical to the pre-extraction
fixture (`b710a38226eb382eaa99725c39947485e58897ef8f0080c164c198e346720047`),
and the original source-shader control again returned zero RGBA error. The
post-extraction comparison is retained at
`target/source-ui/cell-progress-shader-extracted-comparison-v2.json`. It checks
the controlled fixture only; inherited states beyond those explicitly set and
integration with 3D capture require separate review and validation.

## Opt-in single-frame source Cell capture

`--source-cell` can now opt into one explicit fixture on its original-program
offscreen 3D target. All five options are required: `--source-ui-layout`,
`--source-ui-atlas`, `--source-ui-fixture`, `--source-ui-clip` (`shader` or
`scissor`) and `--source-ui-half-pixel` (`0` or `1`). The fixture supplies
synthetic ordered screen-space rectangles and state, not gameplay values. This
is excluded from PBR, audit, diagnostic translation, creature, coverage and
multi-frame paths. Admission checks bounded package headers before opening the
three packages, bounded resource sizes before decoding, hashes and source graph
relationships before starting a scene. Readers reopen package paths, so stable,
trusted inputs throughout admission remain a precondition.

The 3D scene ends and is read back first; its independent non-background count
must pass before a second UI scene starts. Each scene is ended on a draw error;
an incomplete frame is not written. A controlled Jawhead run with the v3
PatchData fixture produced `cell-ui-overlay-v3.png` with SHA-256
`8954b851210b91e6915ca768b327149f3f602e01a95416ef7f5d167ad8e85938`.
Two pre-change and one post-change no-UI captures had byte-identical decoded
RGBA and PNG SHA-256
`b06a039f1c3d8ff838a699ba991cc480a9cb973ac66059a51d94d074f96ebfad`.
The 3D baseline had 50,589 non-background pixels. The UI run changed 4,997
pixels inside a conservative one-pixel-expanded union of the fixture rectangles
and zero outside it. Shader-clip and scissor runs were byte-identical in this
fixture. Evidence is `target/source-ui/cell-ui-integrated-comparison-v3.json`,
reproducible with `tools/cell_ui_integrated_capture_compare_v1.py`. Existing
output and an oversized-index source were rejected without replacing/creating
the requested output. This validates a bounded capture fixture, not the original
screen, package selection, dynamic visibility, gameplay route or full HUD.

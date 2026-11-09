# Cell auxiliary camera: recovered transform and projection arguments

This is a static follow-up to [cell-gameplay-camera-static-v1.md](cell-gameplay-camera-static-v1.md). The installed `D:\Games\SPORE Collection\SporeBin\SporeApp.exe` SHA-256 is `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a` (rechecked); addresses below are virtual addresses in that image. The code is associated with the auxiliary camera-like object at `016B5298`. It is **not established as the primary gameplay camera**. No game or GPU code was executed.

## Matrix construction

At `E548BA..E54938`, let `P` be state `016B4BEC` +48/+4C/+50, `O` be +6C/+70/+74, and `s` be the float at +68 of the object returned by `E4CCA0`. The call to `10436F0` receives:

- first vector (eye): `E = P + O + s * (0,0,1)`;
- second vector (target): `P`;
- up reference: `(0,1,0)`;
- output: a 16-float matrix subsequently passed to `7C45E0` on `016B5298`.

The vectors `(0,1,0)` and `(0,0,1)` are read directly from executable globals `015A8C34` and `015A8C40`. At `10436F0..10438D4`, the function computes `F = normalize(P - E)`, `R = normalize(F cross up_reference)`, and `U = R cross F`, then writes four consecutive four-float groups `(R,0), (F,0), (U,0), (E,1)`. This is a pose/basis with eye translation, not an already inverted world-to-view matrix. The normalizations incorporate the small executable constant `9.99999993922529e-09` at `013EC4B0`; exact bit-level replication should follow the native arithmetic rather than assume perfectly unit vectors. Degenerate and nonfinite inputs need separate analysis.

`E541DF..E54247` initializes P and the two neighboring vectors +54 and +60 from the executable global `(100,100,0)` at `015A8D3C`. `E542B6..E542E5` initializes O to `(0,0,20)`. If that initializer has just run, the state is unchanged, and `20+s > 0`, the eye is `(100,100,20+s)` and the nominal axes are right `+X`, forward `-Z`, up `+Y`. This is a conditional derived initial pose, **not** an observed frame, nor proof that the object is the main view. If `20+s` has another sign or is zero the axes or normalization behavior differ.

## Projection arguments and unresolved selection

`E5493D..E54972` passes `50.0` to `7C4CD0`, `1.0` to `7C4540`, and `2000.0` to `7C4560`. The source constants are `0148467C` and `0145CC2C`; the middle argument is loaded with `fld1`. The 50 is degrees at the common projection routine; the aspect-dependent `0.75` tangent branch and finite depth mapping are documented in [cell-camera.md](cell-camera.md). These arguments describe this invocation only: they do not prove the active viewport dimensions, subsequent state, main camera or actual rendered frame.

The scalar source remains unresolved. `E5489F` calls `E4CCA0`, then loads its returned object +68; retained Ghidra dossier `target/rea/20261028-source-runtime-v6/829-analyze_function.json` (evidence `ev_9e7eda34332bafc69471d3d80bf8b57b645514d1e8066086918e6e0288a43685`) identifies `E4CCA0` as forwarding to `E823D0` with global `015A858C` and the caller's argument. This does not establish that scalar's value or runtime provenance. The selected main-camera class/key and attachment remain unresolved as described in the earlier note. Follow those paths before transferring these values into the gameplay renderer; use the above as a separately labeled auxiliary-camera fixture in the meantime.

Evidence for the additional numeric and arithmetic claims is a read-only PE byte read and x86 disassembly of `E54890..E54989`, `E541DF..E542E5`, and `10436F0..10438D4`, using `pefile` and Capstone against the digest above. This is static instruction evidence with no claim of runtime execution or selection.

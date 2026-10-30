# Controlled source Creature stencil coverage

This is an opt-in, offscreen inspection of the explicitly selected Creature `40616200A1C46DF2` and its one-mesh authored LOD0 model `40617100A1C46DF2`. It does not establish that the game selected this Creature or a Cell actor, and does not implement animation. The source-program capture and its inspection camera and lighting limits are described in [the earlier draw note](cell-fallback-original-program-v1.md).

The coordinating agent reports two focused source-Creature CLI tests passed and ran:

```powershell
target/debug/rustspore.exe --source-creature --install 'D:\Games\SPORE Collection' --creature 40616200A1C46DF2 --shot target/source-creatures/cell-fallback-source-stencil-v1.png --coverage target/source-creatures/cell-fallback-source-coverage-v1.png
```

The source draw is performed once without instrumentation and again with stencil replacement; the renderer requires the two RGBA color readbacks to match exactly before producing the mask. It then clears only the color target and reads a stencil-equality mask. The retained stdout reports NVIDIA GeForce RTX 3060 Laptop GPU, 4,840 triangles, `color_equal=true`, and 52,364 coverage pixels. The source render uses the existing controlled generic-static program, chosen source assets and inspection render state; it does not establish retail material, depth, blend, cull, actor or animation behavior.

Both files are 768 x 768. The color PNG SHA-256 is `2e189579eb312fc9b1e63a53c766b5aef745fbe387ba6bf98a62f32edfd20dc0`, identical to the earlier noninstrumented `cell-fallback-original-program-v1.png`. The mask PNG SHA-256 is `c84ef180987a0e02d570a20c941a0d5e2faf2647b992e72b2bba4c54ac4cc246`. Independent RGBA inspection finds only opaque black (537,460 pixels) and opaque white (52,364 pixels), with white bounding rectangle `(119, 285, 607, 489)` using exclusive right and bottom edges. The stdout log SHA-256 is `312b4cf0980208e9ad70ab84c9918263eedfc13558ebb993e4b32d0784718932`; stderr is empty, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`. All capture artifacts remain under ignored `target/`.

The preceding synthetic D3D9 fixture checked stencil coverage for synthetic shader discard, depth and alpha cases, including black geometry against a black clear. It uses a separate mask implementation; that fixture's success does not directly validate the source renderer's mask helper or establish original game behavior. The source run's binary mask and equal colors are evidence for this single controlled draw only, not a proof of retail depth semantics or complete pixel ownership.

At documentation time `target/debug/rustspore.exe` SHA-256 was `c419f6bfb1c157124cb37773baf4f5779f454318922efdb60d0430108392fefc`; it is not independently bound to the capture processes. The two CLI test results and command are reports from the coordinating agent, not independently bound to a test log here. Output creation uses new paths, but a failure while writing the second PNG can leave the first or a partial file and is reported as such.

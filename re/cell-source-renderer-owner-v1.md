# Source renderer ownership and inspection-frame restoration

`native_cell_renderer.rs` now separates `prepare_source_renderer` from frame calls. It reads and validates source resources, creates one fixed 768 × 768 offscreen D3D9 device and its prepared mesh resources, and returns a `PreparedSourceRenderer` plus an `InspectionContext`. The owner can outlive the preparation call and serve repeated inspection draws. Prepared objects are dropped before the device target and its hidden window. A non-Send/non-Sync marker keeps the owner on its creating thread; the draw uses the same device that created its resources. Geometry remains in CPU vectors and is sent using `DrawIndexedPrimitiveUP`.

The repeated inspection draw is currently enabled only for the explicitly validated one-mesh Creature `40616200A1C46DF2` / model `40617100A1C46DF2`, with its checked resource identities, shader recipes, geometry, uniform extents and budgets. It renders both plain and stencil-instrumented passes and requires identical color readbacks; it then reads the mask. The context uses the existing thumbnail inspection fit and fresh inspection lighting. The owner is an internal diagnostic API, not a live Cell renderer. It has no device-loss recovery, D3D device reset, target resize, shared depth integration, gameplay camera, actor animation, or established runtime selection. A failure after beginning a scene can leave device state changed; no recovery contract has been established.

## Retained comparisons

The coordinating agent reports successful focused tests and RTX captures after factory extraction. The retained `target/source-creatures/factory-{zero,moved}-{first,first-mask,second,second-mask}-v1.png` files were compared byte-for-byte here against their matching `prepared-{zero,moved}-...` references: all eight pairs match. The single-frame two-mesh Jawhead capture `target/source-cell/jawhead-factory-regression-v1.png` has SHA-256 `b06a039f1c3d8ff838a699ba991cc480a9cb973ac66059a51d94d074f96ebfad`, matching the earlier retained baseline. These are specific static captures, not general rendering equivalence.

The later `target/source-creatures/restore-moved-v1.stdout.log` identifies an NVIDIA GeForce RTX 3060 Laptop GPU, reports the second inspection translation `[0.25, 0.0, 0.0]`, and reports `SOURCE_3D_THIRD_FRAME restored_color_and_mask_equal=true same_owner=true`. The first draw uses the initial inputs, the second uses that translation, and the third restores the initial inputs on the same owner after the second stencil readback. The code compares the third complete RGBA and mask readbacks to the first and fails on inequality. No third image is saved, so the restoration comparison is represented by the log and its exercised check. This is draw-state restoration, not a D3D device-reset test.

The retained first and moved second images differ in their pixel content; the prior independent inspection reports 66,738 RGB-changed pixels and 66,775 RGBA-changed pixels for these same image hashes. Their masks also differ. The four retained files are:

| File under `target/source-creatures/` | SHA-256 |
| --- | --- |
| `restore-moved-first-v1.png` | `2e189579eb312fc9b1e63a53c766b5aef745fbe387ba6bf98a62f32edfd20dc0` |
| `restore-moved-first-mask-v1.png` | `c84ef180987a0e02d570a20c941a0d5e2faf2647b992e72b2bba4c54ac4cc246` |
| `restore-moved-second-v1.png` | `5dd465c17e1988870aacf81b341aef2f4a564799196540bc306b2bd2aa52e16e` |
| `restore-moved-second-mask-v1.png` | `ef5e6544021868a0817554fa8f088bffde45239ac19f315e370ede01b1e346cb` |

The stdout SHA-256 is `be8291a8c4538aa51b63b3aa13b919d1cf782f458e20754cb1e5a94113cbb8c4`; stderr is empty. These logs and images do not independently bind the executed binary to the current source. The stencil mask measures this inspection draw, subject to [the stencil note](cell-fallback-source-stencil-v1.md); it does not establish scene occlusion or retail behavior.

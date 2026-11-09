# E219 controlled palette position diagnostic

This opt-in example deforms the explicitly selected E219 GMDL's 2,555 source
positions using a fixed 36-matrix diagnostic palette. The first ten matrices
come from the conditional, source-mapped synthetic pose fixture; the other 26
are source-stored identity matrices from four explicitly selected RW4 buffers.
Those stored values are supplied without runtime evaluation. The direct fresh
`769A80`/unmodified path is an explicit caller admission.

The output is a copy of the 36-byte vertex buffer. Only position bytes change;
normals, tangents, UVs, weights and indices are not updated. It is a RustSpore
position diagnostic, not original CPU or GPU deformation, a live actor pose,
runtime palette selection, evaluated RW4 animation or a material/render match.
Do not feed its output to a skinning shader as an already verified pose.

The example accepts only the exact hash-bound inputs from the reviewed local
fixtures and creates a new output file. With those fixtures present, run:

```powershell
tools\build.bat run --offline --locked -p spore-core --example e219_controlled_palette_position_v1 -- `
  --admit-synthetic-direct-path `
  target/e219-source-groups-result-v2/model.gmdl `
  target/creature-skin-ten-node-source-map-deform-prep-v1-20261109/pose-palette.bin `
  target/animation/e219-authored-36-palette-diagnostic-result-v2/fallback.bin `
  target/cell-startup-geometry-result-v1/vertices.bin `
  target/cell-startup-geometry-result-v1/indices-le.bin `
  target/animation/e219-controlled-position-new.bin
```

The retained capture's index stream contains 12,732 little-endian **u32**
values. The example checks all 2,555 records, the complete index stream, all
four offsets and weights, descriptor `(0,36)`, an all-identity control and the
fixed output hash. The reviewed target-only diagnostic receipt is
`target/animation/e219-authored-36-palette-diagnostic-result-v2/receipt.json`;
its limits and the earlier failed u16-width draft are recorded in
`re/creature-generated-influence-palette-identity-static-v1.md`.

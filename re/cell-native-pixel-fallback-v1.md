# Jawhead native pixel fallback verification

A controlled offscreen Jawhead draw verified the generated-pixel path for a
recipe that also had a cached shader. It does not establish behavior for a
truly uncached model or prove in-game visual fidelity.

With feature `native_shader_probe` enabled and
`RUSTSPORE_VERIFY_PIXEL_FALLBACK=1`, the renderer reported `forced=true` and
`cached_available=true` for both meshes. Each used selected pixel recipe
`05 09 0C 11 3B 19 2A`, original profile-3 fragment library from
`Data/PatchData.package`, pinned executable templates, and system D3DX9_27
compiler flags `12`. The draw used the `generic_static` builder and render
type 0, and the log reports an NVIDIA GeForce RTX 3060 Laptop GPU, 9,988
triangles and `presented=false`.

The forced image, the cache-hit control image after the fallback change, and
the prior baseline PNG have identical bytes, with SHA-256
`b06a039f1c3d8ff838a699ba991cc480a9cb973ac66059a51d94d074f96ebfad`.
Identical PNG bytes imply identical decoded pixels for these artifacts. The
retained forced log reports generated PS compilation and uniform uploads for
both meshes; the cached control exercises normal cache-first behavior.

Evidence under ignored `target/source-cell/`:

- `jawhead-pixel-forced-v1.png` and `.log`; log SHA-256 `776bd22ba2bc4c4493f11d97981469fac7151b8bfb10f21c8335ac875b0dac71`.
- `jawhead-pixel-fallback-regression-v1.png` and `.log`; log SHA-256 `eecf7d4d2026ea5c75350cd655acd364bf576bd9f9078997de5bcd7d3e62c406`.
- `jawhead-baseline-20261028-v2.png`, the prior baseline.

The force switch is compiled only with the probe feature and changes behavior
only for the exact value `1`. The image uses inspection camera fitting, identity
lighting frame, fresh lighting inputs and explicit material-color choices.
Runtime shader choice, provider precedence, animated pose, live game scene,
and retail fidelity remain unverified. This result is confined to the stated
asset, recipe, inputs, hardware and capture.

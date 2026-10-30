# Cell source Creature inspection v1

This controlled prototype capture explicitly assigns Creature instance `40616200A1C46DF2` to the Cell player. The instance is associated with the kind-4 query's static fallback constant, but the capture does not demonstrate native fallback, selection, spawning, or an original Cell actor. See [the candidate asset audit](cell-kind4-candidate-assets-v1.md).

The coordinating agent reported the following command, after 11 `cell_game` tests passed:

```powershell
$env:WGPU_ADAPTER_NAME = 'NVIDIA GeForce RTX 3060 Laptop GPU'
$env:WGPU_BACKEND = 'vulkan'
target/debug/rustspore.exe --experimental --cell --source-creature-install 'D:\Games\SPORE Collection' --source-creature-instance 40616200A1C46DF2 --offscreen --demo 2 --shot target/source-cell/cell-source-creature-fallback-v1.png
```

The stdout records the NVIDIA RTX 3060 Laptop GPU on Vulkan, a static PBR player, base mip, inspection overlay order and inferred `source_y_z_x` basis. At tick 120 it reports `Playing`, 8 eaten, 0 kills, progress 0.246154 and health 72.8. The image shows a small pale player amid stylized procedural rivals and food in the prototype Cell scene. Saving and loading are disabled in source inspection mode. This still does not validate original animation, pose, materials, gameplay, or selection. Stderr reports an unavailable Vulkan validation layer and overlay-layer warnings.

The capture stdout identifies these resources from `Data/Spore_Content.package`:

| Role | Native key | Decoded payload SHA-256 from prior v3 audit |
| --- | --- | --- |
| Creature | `A1C46DF2:0F43029A:40616200` | `6852040fa9a9376ed9025f4823e05024a22685a147e00e64d1b0f536b33e8cd8` |
| PROP | `A1C46DF2:00B1B104:40616200` | `1f102f9542a5eb3d7770a99935e1e9011e8093a4a61433b370b421c914c4de09` |
| Model | `A1C46DF2:00E6BCE5:40617100` | `cc9290ab9c9efadda974c76c21aa8edbeb5b762ae58f70a283d26a5276bdaa65` |
| Texture | `A1C46DF2:2F4E681C:40612900` | Not recorded by that audit |

Those payload hashes come from the separate disk audit, not the capture log. The installed Spore_Content package SHA-256 when checked for this note was `0b849bd79d9c13dec5737159655bf7e2b7763cc2b79949c81370fa0490256e8c`; that check does not bind the capture process to package bytes at capture time.

- `target/source-cell/cell-source-creature-fallback-v1.png`: SHA-256 `a2dcb379870d8ba5a8d77a0e97634e75b4304abbc42b00917cf75f674822b342`.
- `target/source-cell/cell-source-creature-fallback-v1.stdout.log`: `a7697eb25b0088e17ad4f4e84dd2e9a84a5bb2d7c7e06986bc4401b059e36d89`.
- `target/source-cell/cell-source-creature-fallback-v1.stderr.log`: `683920861ebc150d11d7adbc3284b50b44c74ef3878c6005c469349623e43094`.

A separate Jawhead regression v6 capture retained at `target/source-cell/jawhead-gameplay-regression-v6.png` has SHA-256 `d03837d8fc11a6bf7bf6e159d77ef77511a030be76a204ffe6b16aaf3e3fc2aa`. Its stdout identifies the Jawhead source chain and a static PBR inspection at tick 120; its stdout and stderr hashes are `ed72fb6214911aa57949a8a33f028bf3e507d3b9c5501259b953674372bc56ba` and `642ee734e73b8353d959f65db1bd267d7e4dacc061fa80b9dbd175661bbd6716`. It is a separate regression observation, not a fidelity or spawn-selection test.

The 11-test result and capture command are reports from the coordinating agent; this note does not independently bind the test result to a test log or binary. The note author inspected the existing capture and logs and did not run Cargo or a native program.

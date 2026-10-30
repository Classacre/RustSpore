# Cell fallback candidate original-program reference v1

This is a controlled offscreen source-art inspection of the explicitly requested Creature `40616200A1C46DF2`. Its low ID appears in a static Cell query fallback path, but this capture does not show that the original game selected that fallback, spawned an actor, or played an animation. The broader disk and prototype evidence is in [the candidate asset note](cell-kind4-candidate-assets-v1.md).

The coordinating agent reported this command:

```powershell
target/debug/rustspore.exe --source-creature --install 'D:\Games\SPORE Collection' --creature 40616200A1C46DF2 --shot target/source-creatures/cell-fallback-original-program-v1.png
```

Stdout identifies Creature `A1C46DF2:0F43029A:40616200`, PROP `A1C46DF2:00B1B104:40616200`, and authored LOD0 model `A1C46DF2:00E6BCE5:40617100`. It reports original D3D9 programs with animation unimplemented, the NVIDIA GeForce RTX 3060 Laptop GPU, one draw with 2,866 vertices and 4,840 triangles, and untouched vertex bytes. Vertex shader fragment IDs are `[14, 17, 19, 15, 31, 30, 29, 01]`; pixel fragment IDs are `[05, 09, 0C, 11, 3B, 19, 2A]`. These are the recorded selected fragments, not a proof of the retail draw path.

The draw log records sampler 0 resource `40612900A1C46DF2` as 512×512 DXT5 with 10 levels, sampler 1 `40612901A1C46DF2` as 256×256 format `00000015` with 9 levels, and sampler 2 `00000000552A5077` as 64×64 DXT5 with 7 levels. Each reports source blocks and an exact full mip upload. The log reports 82 indexed packages in inspection order, not verified retail precedence.

The rendered still shows a tan textured creature with large eyes and narrow appendages against a dark background. The result reports 52,364 non-background pixels and `presented=false`. It uses the `generic_static` profile-3 builder and render type 0, a fresh lighting context with identity light frame, white material and fallback tint, and a fitted inspection camera. The log explicitly marks `retail_fidelity=false`; it does not establish original scene lighting, camera, draw state, animation or actor binding. The renderer uses a hidden window and an offscreen readback, not a presented game window.

- PNG `target/source-creatures/cell-fallback-original-program-v1.png`: SHA-256 `2e189579eb312fc9b1e63a53c766b5aef745fbe387ba6bf98a62f32edfd20dc0`.
- Stdout `target/source-creatures/cell-fallback-original-program-v1.stdout.log`: SHA-256 `92e9234297aaad06b24216811edcf1e8ae0351e64aa7f86f0dd74c6c7a570be2`.
- Stderr `target/source-creatures/cell-fallback-original-program-v1.stderr.log`: SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` (empty).

The note author inspected the existing image and logs, but did not run Cargo, native code or a GPU capture. The command was supplied by the coordinating agent; the logs do not independently bind executable or input package hashes to the capture.

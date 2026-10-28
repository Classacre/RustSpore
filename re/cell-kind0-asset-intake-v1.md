# Candidate kind-0 Cell asset intake

A bounded, explicit-package inspection joined raw Cell `FD24574B` to the
candidate structure of the same ID, then its kind-0 row to PROP and RW4
`DAE8CE17`. This is a disk projection: native reference fixup, provider
precedence, route selection, actual actor construction and the kind-1 row
remain unverified or unsupported. The model is not a claim of in-game appearance.

The coordinating agent reports this build and invocation:

```powershell
tools\build.bat build -p spore-bevy --example cell_kind0_asset_intake_v1
target\debug\examples\cell_kind0_asset_intake_v1.exe "D:\Games\SPORE Collection\Data\PatchData.package" "D:\Games\SPORE Collection\Data\Spore_Graphics.package"
```

The retained `target/source-cell/cell-kind0-asset-intake-v1.log` (SHA-256
`e0df813252cb0e1dfd7fe56933002d327988fc20aa7f17dcb8bf1a4ae9d39b6b`) records
20 RW4 sections, one decoded mesh, and a generic projection of 454 vertices
and 896 triangles, source normals and zero sanitised values. This projection
can skip or synthesize attributes; **mesh topology completeness is unverified**.
The v1 log predates the added explicit completeness marker. A rebuilt v2 run
reported the same mesh counts and the marker in
`target/source-cell/cell-kind0-asset-intake-v2.log` (SHA-256
`f17f64293c8e589b00b4a66872446d9aa58a70b2d6d7a41d5caa7168d3d2f426`).

It records compiled state section 14 with shader `80000004`, and section 15
raw link words `13, 1, 14`. Two raster sections, 17 and 19, each decode to
128×128 DXT5 with eight mip levels. Subsequent independent state-decoder review
reports saved stage 0 raster word `00000011` and stage 1 word `00000013`,
corresponding to sections 17 and 19 in this candidate. The rebuilt v2 intake
reports the same candidate-specific correspondences and saved state pairs,
consistent with that independent review. Runtime pointer fixup and shader
execution are not established. The PROP's LOD0 names the RW4 type
`2F4E681B` and instance `00000000DAE8CE17`; the loaded package is an explicit
inspection choice, not verified retail resolution.

The coordinating agent supplied these SHA-256 hashes for the exact payloads:

- PatchData Cell `FD24574B:DFAD9F51:00000000`: `f8950cef073bc3536f1a736436a4048a8b6d5bf0066bc7db9b06c979185e72bd`.
- PatchData structure `FD24574B:4B9EF6DC:00000000`: `307a7008d90f638e8c16c6367b3cae2090e3b639d83b15ef282c0ba9d48ee3e5`.
- Graphics PROP `DAE8CE17:00B1B104:00000000`: `15ebf08981b6b45ec8ce98d899135126281a30c4ecde4caaacbd496c8e82b9e8`.
- Graphics RW4 `DAE8CE17:2F4E681B:00000000`: `bbd99a741d41a3606a310f242e14e5f85dc7b49af454972ff02124b16e318592`.

Current package hashes measured separately are PatchData
`cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff` and
Spore_Graphics `e01bb86d8ddf02c56d52394b4e1db7181c1e1ebe1878bf6fb72c4ee523fcedd1`.
Package preflight and index reads reopen the input, requiring trusted stable
files; this does not remove the replacement window.

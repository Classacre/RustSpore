# Kind-0 Cell gameplay inspection v1

This is a controlled prototype-player inspection of the installed candidate Cell `FD24574B`, not evidence of retail player selection or faithful gameplay. The current bridge is `crates/spore-bevy/src/cell_kind0_inspection.rs`, invoked by `cell_game.rs` for this explicit source Cell ID. The capture predates this note; the exact invocation and source revision used for the capture were not recovered from the log. No new game or GPU run was made for this note.

## Evidence

- `target/source-cell/cell-kind0-gameplay-v1.log`: SHA-256 `080a7e62ef55cdedbd3c52184ff71726aa798dd2873d792e5baec074f64ae413`.
- `target/source-cell/cell-kind0-gameplay-v1.png`: SHA-256 `5851a19b3a23e392cd11212ab9412d69892a70493225190331e4070f3f14f66f`.
- The log records explicit PatchData Cell and structure resources `FD24574B:DFAD9F51:00000000` and `FD24574B:4B9EF6DC:00000000`, and Spore_Graphics model PROP and RW4 resources `DAE8CE17:00B1B104:00000000` and `DAE8CE17:2F4E681B:00000000`. The kind-1 reference `988D8969` remains unresolved.
- The logged adapter is an NVIDIA GeForce RTX 3060 Laptop GPU using Vulkan. The logged result is `Playing`, tick 120, 8 eaten, 0 kills, progress 0.246154, health 72.800, position (4.7678, 0.6509). Vulkan validation layer was unavailable; the log also contains overlay-layer warnings.
- The image shows the prototype arena and HUD, a small green textured object near its center, other stylized actors and food, and the explicit static inspection label. A single screenshot does not establish animation, interaction correctness, or original appearance.

## Interpretation and limits

The loader validates a specific candidate structure, model, saved material references and sampler, and decodes stage-zero texture and model geometry. It hardcodes the two package paths instead of proving runtime provider lookup or overlay precedence. Current rendering uses a static mesh, PBR approximation, opaque sRGB base color and only the base mip; saved stage 1 and the kind-1 effect are not rendered. Runtime fixup, pose, topology completeness, scale, shader behavior, and active selection remain unverified. The gameplay orientation uses a fixed rotation derived from a different source thumbnail convention, so this candidate's forward axis is not established. A later 3D camera overlays the 2D world, so its ordering does not establish original occlusion.

Related source-chain and resource-hash evidence is recorded in `re/cell-kind0-asset-intake-v1.md` and `re/cell-world-population-source-v1.md`. Establishing candidate-specific runtime pose and forward direction, resolving the kind-1 entry and stage-1 material behavior, and comparing against a source runtime capture are necessary next steps toward a fidelity assessment.

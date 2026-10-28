# Creature source gameplay inspection v1

This note records the existing v2 capture of source Creature `40626200067C5390` assigned as the controlled player in a prototype scene. It is a static PBR inspection, not evidence of faithful original gameplay or runtime behavior. The capture occurred before the HUD correction now in progress; the image retains the earlier HUD and save/load labels. Source inspection saves are disabled according to the capture log.

## Capture

The reported command was:

```powershell
$env:WGPU_ADAPTER_NAME = 'NVIDIA GeForce RTX 3060 Laptop GPU'
$env:WGPU_BACKEND = 'vulkan'
target/debug/rustspore.exe --experimental --creature-game --source-creature-install "D:\Games\SPORE Collection" --source-creature-id 40626200067C5390 --offscreen --demo 2 --shot target/source-creatures/creature-source-gameplay-v2.png
```

SHA-256 hashes:

- PNG `target/source-creatures/creature-source-gameplay-v2.png`: `98d805fe34f7220741934174510bd2a5aea85771742b45ae152e2fe527b5fc6a`.
- Standard output `target/source-creatures/creature-source-gameplay-v2.stdout.log`: `d701b51c397df3b03fc0527cd3896bc8f00a9c0877efe85f4b2d7a16f7c0a2b8`.
- Standard error `target/source-creatures/creature-source-gameplay-v2.stderr.log`: `325bce5b44d1d5702db56c2ce147d06d38851f7612caaf99bd7d271a4f41ee1b`.

The output records the NVIDIA adapter and Vulkan backend, state `Playing` at tick 120, DNA 30, 2 parts, 0 resolved, 0 pack, health 100, and position (10.197, 4.506). Standard error reports that the Vulkan validation layer was unavailable and contains overlay-layer warnings.

## Observations and limits

The image shows a small green textured Creature in a stylized terrain scene, with prototype actors, food and HUD. It is one still frame and does not demonstrate original animation or interactions. The logged creature, PROP, model and texture resources came from `Spore_Content.package`; the log identifies the assignment as a controlled prototype player and the renderer as PBR with static pose and base mip only. The source axis mapping `source_y_z_x` and ground at `source_z_min` are inferences. Camera scale and overlay order are inspection choices. Original source selection, pose, animation, behavior, material semantics and gameplay semantics remain unverified; the screenshot cannot establish fidelity to the original game.

No new cargo or GPU run was performed for this note.

## Final HUD capture v3

The final HUD update was captured with the same environment and command above, changing only the shot path to `target/source-creatures/creature-source-gameplay-v3.png`. In the v3 image, the F5/F9 labels are absent and the on-screen notice explicitly says save and load are disabled. This is consistent with the stdout inspection record; the image alone does not test key behavior.

- PNG SHA-256: `1cc4277d07b1db35ca5b75721794bc3377f142359bb641da4134cc1c897c83c5`.
- `target/source-creatures/creature-source-gameplay-v3.stdout.log` SHA-256: `36c173ff4b0d60319f9e4eb78c95c604d72aa58545122d003f710e7cf3c99f89`.
- `target/source-creatures/creature-source-gameplay-v3.stderr.log` SHA-256: `aa8cddbaec6d91f3747f6e1984bd2271ce9979d51eec85a2a5cd07e1a89a4e20`.

The v3 stdout reports `Playing` at tick 120, DNA 30, 2 parts, 0 resolved, 0 pack, health 100, and position (10.188, 4.515). The same static-art and fidelity limitations apply.

The recorded workspace test run passed 751 tests with 2 ignored. Its log `target/workspace-tests-20261028-resume-v46.log` has SHA-256 `a2928e6d7e17230cf1c306efc2bacdbec920c8fdee7acb23704731885f1fde9f`. This note did not rerun those tests.

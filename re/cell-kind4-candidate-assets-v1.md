# Cell kind-4 candidate asset audit and static capture

This is a disk-key audit and a controlled prototype inspection. Neither establishes the game's active metadata registry, query result, fallback use, Cell actor, or runtime selection. The fallback ID is a static query failure-path constant, as described in [the source-list note](cell-kind4-source-list-v1.md).

## Corrected candidate asset audit

The reported v3 audit used the 42 pinned metadata candidates from `target/cell-kind4-metadata-scan-v2/matches.json`. It checks each selected record against the live native package index and payload extent, decodes and hashes the payload, and records references from parsed PROP key values. Metadata selection is by pinned package, low ID, type and scope; Creature, PROP and GMDL records are broader same-low-ID census joins. The corrected PROP interpretation is little-endian instance-low, type, instance-high, consistent with `spore_core::prop` and [the model-link trace](creature-model-links-native.md). The earlier v2 audit reversed the first and third PROP words; its reference mapping is discarded.

The v3 report records 42 metadata, 42 Creature, 86 PROP and 130 GMDL records. It reports 260 explicit PROP GMDL references, each with an exact full-key match in the audited candidate census subset. These counts include separate PROP copies and do not prove provider precedence or that the native query selects any candidate. A key with third word `FFFFFFFF` requires inherited PROP context and is not counted as an exact hit by this tool. The audited metadata range predicates and registration remain unresolved; the joins cannot establish actor identity or original behavior. The audit emits keys, sizes and hashes, not raw payloads.

The reported command was:

```powershell
python tools/cell_kind4_candidate_asset_audit_v1.py 'D:\Games\SPORE Collection' --census target/native-archive-index-census-v1/records.tsv --matches target/cell-kind4-metadata-scan-v2/matches.json --out target/cell-kind4-candidate-asset-audit-v3
```

The successful report `target/cell-kind4-candidate-asset-audit-v3/report.json` has SHA-256 `035dfd2ddf605d6e2434f48564390230f4022f088017808a4e68e810bc8740d4`; `candidates.json` has `69cc9fa3d958898cca498c90f07834571bb1c02938f5b07baad60f852983e3d9`. The script at documentation time hashes to `4a2a42242116f68849d30e176c137ed3dd3f251c7edf6ab05edca3735e445858`; the report does not bind the script hash. The report records matching before/after package SHA-256 values for PatchData (`cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff`) and Spore_Content (`0b849bd79d9c13dec5737159655bf7e2b7763cc2b79949c81370fa0490256e8c`). This does not eliminate temporary replacement between individual reads.

## Controlled source candidate capture

The reported capture explicitly assigned source Creature `40616200A1C46DF2` to the prototype player:

```powershell
$env:WGPU_ADAPTER_NAME = 'NVIDIA GeForce RTX 3060 Laptop GPU'
$env:WGPU_BACKEND = 'vulkan'
target/debug/rustspore.exe --experimental --creature-game --source-creature-install 'D:\Games\SPORE Collection' --source-creature-id 40616200A1C46DF2 --offscreen --demo 2 --shot target/source-creatures/cell-fallback-candidate-v1.png
```

The stdout identifies the creature `A1C46DF2:0F43029A:40616200`, PROP `A1C46DF2:00B1B104:40616200`, model `A1C46DF2:00E6BCE5:40617100`, and texture `A1C46DF2:2F4E681C:40612900`, each from Spore_Content. It records a static PBR inspection on an NVIDIA GeForce RTX 3060 Laptop GPU using Vulkan, with inspection overlay and camera choices, inferred `source_y_z_x` basis, base mip only and saves disabled. It reports tick 120. Stderr reports an unavailable Vulkan validation layer and overlay-layer warnings.

The still image shows a small pale creature in a stylized prototype terrain with food, other actors and a HUD. It demonstrates this controlled source asset was displayed in that scene; it does not establish retail Cell selection, fallback use, material fidelity, animation, pose, or gameplay behavior.

- PNG `target/source-creatures/cell-fallback-candidate-v1.png`: SHA-256 `a2af8549a932c67e866e75f0b86fa176f3d42467978d1c3cd68ff63b98524dda`.
- Stdout `target/source-creatures/cell-fallback-candidate-v1.stdout.log`: SHA-256 `8099064572a8b0371c0e65837a47658e87066a496fabc8edc11b8378d81287b7`.
- Stderr `target/source-creatures/cell-fallback-candidate-v1.stderr.log`: SHA-256 `6948fd7467af4adda44ab436104420968851304673264e74e7927b6b1e15fb19`.

The note author inspected the existing image, logs and report; no Cargo, native program or GPU capture was run to document them.

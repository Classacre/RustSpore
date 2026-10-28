# Cell kind-4 source lists and selection boundary

This is a read-only disk projection and static x86 trace, not an observed actor, post-fixup value, provider result or animation. The executable SHA-256 is `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. The source population routes and their limitations are in [the world population note](cell-world-population-source-v1.md); native key order is explained in [the archive index note](archive-index-native.md).

The eight route-0 candidate Cell definitions previously identified as kind 4 lead, in PatchData disk projections, to structures with an array offset of `1C`, count 1 and a 40-byte row with kind 4. The raw row word at +0C has a matching installed native key of type `F9C3D770`, scope zero. These are source joins; actual fixup and lookup remain unverified.

| Cell low ID | Structure low ID | List low ID |
| --- | --- | --- |
| D6C8F725 | 621FEB62 | D7E124A7 |
| D6C8F726 | 621FEB61 | D7E124A4 |
| 620D8C55 | A2B0DC84 | 4122E50F |
| 620D8C56 | A2B0DC87 | 4122E50C |
| 7A3376CD | 54FF6054 | B17DEBCF |
| 7A3376CE | 54FF6057 | B17DEBCC |
| F29C08A5 | 407FB790 | BC092047 |
| F29C08A6 | 407FB793 | BC092044 |

All eight list keys appear in the retained native census for both PatchData and Spore_Game. The selected PatchData payloads are 36 bytes each. Their disk words are count 1, array offset 8, then one 28-byte row with type 1, word +4 zero and weight `3F800000` (1.0). The remaining four signed row words are:

| List low ID | Row +0C | Row +10 | Row +14 | Row +18 | Payload SHA-256 |
| --- | --- | --- | --- | --- | --- |
| D7E124A7 | -1 | 4 | 2 | 5 | `791d5049ad6721625fd89b834c6ba1c216bed980b14a1300065c96eb40cff868` |
| D7E124A4 | -1 | 1 | 3 | -1 | `97415972c97b11cfc3d17a9f404483ea7f73d6f6c4782866d6896fb7da217045` |
| 4122E50F | -1 | 2 | -1 | 4 | `9c1d72d5fe1a7c59e4cbe1313f8326617869d432c05f6eb17cd1e3468a8706ed` |
| 4122E50C | -1 | 4 | -1 | 2 | `ee18ae09a9901677e88fe547cbbee5edbd8aa76e6b7cda37b6a8e1b40f0d187a` |
| B17DEBCF | -1 | 4 | 1 | 4 | `1e3c9756d20dce21fd1c5575f0256e5fb7e8d1eb200d4eba4aea74eb3817af17` |
| B17DEBCC | -1 | 4 | 2 | 4 | `4802ddc259489cf393d0eb357b208e32286827c77e3bcb814bd36644cf3a9c50` |
| BC092047 | -1 | 4 | 3 | 8 | `0ce47b95537792d7e780c591e8408da5920593dfba08f71837b895d0953a0c5f` |
| BC092044 | 2 | -1 | 1 | 5 | `d4418610112f75fa7651727b2efdb3cc947fe13656d94fcbca93f3898899e107` |

Retained Ghidra q849 shows `E5C0F0` resolves the list handle and performs weighted selection over runtime rows at stride `1C`. For type 1 it passes row values and global state to `E84FA0`; it does not directly return a model ID from these source rows. `E84FA0` constructs predicates using `02DD90AF`, type `3D97A8E4`, and conditional ranges labeled by `DDDD0001` and `DDDD0000`, then calls the object returned by `67C680` through virtual slot +24 with a predicate collection and an output vector. On success it copies the first three-word result; on failure it writes `A1C46DF2:3D97A8E4:<global word>`. Exact query semantics, registry contents, result ordering and provider precedence are unresolved; the fallback is not evidence that it was used.

The installed native index census contains thirteen `3D97A8E4` instances in PatchData and Spore_Game. A read-only payload inspection of `0625A974:3D97A8E4:40616200` found a `sporemodel` XML document with model type `DFAD9F51` and authored block references. This is an example of a possible query-result resource class, not a demonstrated match to any of the eight predicates. The next source-backed step is to recover `67C680`'s manager and slot +24 filter semantics, project the full predicate metadata and compare candidate `3D97A8E4` resources without treating a matching XML as a runtime-selected actor.

## Integrated disk projection and source comparison

`spore_core::cell_kind4_list_disk_v1` exposes a bounded, provisional disk projection: an eight-byte header, 28-byte opaque rows, and the original payload including gaps and trailing data. It checks count and ranges and imposes a 4,096-row API limit; this cap and the empty-offset policy are implementation choices, not recovered native limits. The example `crates/spore-core/examples/cell_kind4_list_disk_v1.rs` reads an explicit decoded payload with a 1 MiB limit and prints the raw row bytes. Neither parser nor example implements native reference fixup, weighted selection, `E84FA0` query filtering, or actor association.

The coordinating agent reports that the three focused parser tests passed. The retained `target/cell-kind4-list-source-verify-v1/report.json` reports eight successful byte comparisons against the explicitly pinned PatchData payloads above: native index key selection was checked against the legacy reader's physical span, and the example output matched each source row. Its SHA-256 is `68e0bac614f67b55e350be1c31a89d92adfe572a1215f918526899dd81fd881a`. The report records package SHA-256 `cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff` and example executable SHA-256 `67fdfe97f434ca9d9820f11257a1c3262a2943fba96e75381dc10681bf948900` before and after the run. This is source-byte agreement on eight chosen disk lists, not a runtime filter result or an independent proof of the upstream joins.

From the repository root, using the existing pinned example and a new output directory, the comparison can be repeated with:

```powershell
python tools/cell_kind4_list_source_verify_v1.py 'D:\Games\SPORE Collection\Data\PatchData.package' --example target/debug/examples/cell_kind4_list_disk_v1.exe --example-sha256 67fdfe97f434ca9d9820f11257a1c3262a2943fba96e75381dc10681bf948900 --out target/cell-kind4-list-source-verify-v1-repeat
```

The verifier runs the supplied executable, writes the eight decoded payloads and its report under the new target directory, and requires a trusted, stable package and example. Hashes before and after do not eliminate temporary path replacement between reopening and use. The note author did not rerun Cargo or the example while documenting the report.

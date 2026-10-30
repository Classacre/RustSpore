# Cell definition-only intake

`SourceCellLibrary::definition_record(id)` reads an explicitly requested CELL record without requiring a structure, baked creation or artwork. It returns the parsed `CellRecord`, including the original 796-byte `raw` data, and the exact `SourceIdentity` package path and canonical key. It uses the existing inspection order: `PatchData.package`, `Spore_Game.package`, `Spore_Content.package`, then `Spore_Graphics.package`; the first matching record is selected, and a malformed selected record is an error rather than a fallback to a later copy. The artwork-oriented `definition()` now calls the same method before resolving its structure, preserving that inspection selection behavior.

The synthetic two-package test passed. It checks first-package precedence, fallback to the next package when absent, exact identity and raw bytes, successful definition-only intake without a structure, malformed-first-record failure, and a missing-record error. The ignored opt-in test also passed against the local install for explicit candidate `CBCD1287:DFAD9F51:00000000`: it selected `PatchData.package` under inspection order, read 796 bytes, and returned structure ID `CBCD1287`. It compares the parsed record's retained bytes to a second call through the same library reader; that is a **self-comparison**, not an independent extraction or proof of provider correctness.

A separate canonical package-index and decoded-payload investigation in [cell-player-start-definition-static-v1.md](cell-player-start-definition-static-v1.md) reports the PatchData copy's decoded SHA-256 as `f12d98c0f59af6d4d58174016f1e04d709647480310103341193df9b0fa76542`. That independent investigation also found a different Spore_Game copy for the same key. The passed intake test does not itself hash the bytes or establish that the runtime native manager selected the PatchData copy.

From the repository root, reproduce the tests with the installed Windows build environment. The second test is opt-in and defaults to `D:/Games/SPORE Collection`; set `RUSTSPORE_INSTALL` to the installation root if needed:

```powershell
tools\build.bat test -p spore-bevy --bin rustspore definition_record_uses_inspection_order_without_requiring_structure_art
tools\build.bat test -p spore-bevy --bin rustspore installed_definition_only_reads_explicit_start_cell_candidate -- --ignored --nocapture
```

This is asset-backed intake of an explicitly chosen candidate. It does not establish the original runtime overlay winner, native descriptor/reference fixups, player selection, actual actor creation, or live gameplay. No Cargo, GPU or original game code was run while preparing this note.

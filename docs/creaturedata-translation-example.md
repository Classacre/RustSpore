# Local CreatureData translation example

This opt-in CLI extracts one explicitly selected CreatureData resource (type `0F43029A`) from a local package and patches one serialized row's absolute rest translation. Supply the scope/group followed by instance-low as exactly 16 hexadecimal digits, the expected SHA-256 of the **decompressed payload**, a zero-based row ordinal, and three finite `f32` values. The output parent must already exist and resolve within this checkout's ignored `target` directory; the output file must not exist.

```powershell
tools\build.bat run -p spore-bevy --example creaturedata_patch_translation -- "C:\path\to\Spore_Content.package" 40616200A1C46DF2 6852040fa9a9376ed9025f4823e05024a22685a147e00e64d1b0f536b33e8cd8 0 1.0 2.0 3.0 target\edited.creaturedata
```

The example uses a unique native key, bounds the index and selected resource, verifies the payload hash and compares package SHA-256 before and after reading. The caller must ensure the package path and file remain stable throughout: separate opens and before/after hashes cannot rule out transient replacement. It checks that every byte outside the selected translation is unchanged. The input stays untouched and the output is created exclusively. Keep locally extracted game data private.

The output is a raw CreatureData payload, not a complete playable creation. Its baked GMDL/model is not regenerated, and game/editor acceptance is not established.

The target directory and output parent must remain trusted and stable between canonicalization and file creation; this check is not a defense against concurrent directory or reparse-point replacement. If writing or syncing fails, the error identifies the destination: a partial new file may remain and is not automatically deleted. The reported output hash is for the intended in-memory buffer; the command does not read the written file back.

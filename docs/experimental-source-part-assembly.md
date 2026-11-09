# Experimental source-part assembly inspection

This headless inspection reads a caller-supplied CreatureData payload, checks its SHA-256 and assembles the part models that its rows reference using the existing PROP/model resolver and stored absolute transforms. It requires every referenced part to resolve and rejects incomplete or non-finite input. It displays untextured Bevy geometry; it does not regenerate a baked GMDL, source skin, original editor output, or playable creature. Camera framing is computed separately for each payload, so screenshots cannot by themselves establish a geometric difference.

Set `RUSTSPORE_PACKAGES` to the additional package files or directories used to resolve part resources, and provide the source package explicitly. The screenshot parent must exist under this checkout's ignored `target` directory, and the PNG must be new. Keep extracted game data private.

```powershell
$env:RUSTSPORE_PACKAGES = 'C:\path\to\Spore\Data'
tools\build.bat run -p spore-bevy -- --experimental --assembly-preview 'C:\path\to\Spore_Content.package' 'target\edited.creaturedata' '<64-hex-payload-sha256>' --shot 'target\assembly-preview.png'
```

The payload is read into memory and the hash is checked over those bytes. The package lookup uses the existing package-set resolution policy, which is not proof of original mount precedence. The caller must keep the package and trusted output directory stable while the command runs. An ignored, explicit-input test named `edited_assembly_moves_only_the_selected_part` compares the full assembly vertex arrays for a row-zero translation edit; its required `RUSTSPORE_ASSEMBLY_TEST_*` variables are listed in the test. The result is a geometry check of this inspection path, not a native editor or runtime validation.

The preview exclusively creates the destination only after payload and assembly validation; an existing file or dangling link is rejected. It retains the open handle through PNG encoding, flush and sync, and reports success only after those operations finish. A rendering timeout or capture/write/sync error exits with failure and names the destination because an empty or partial file may remain. The trusted `target` directory and its parent path must remain stable while the command runs; exclusive creation does not defend against concurrent directory or reparse-point replacement.

The preview checks wall-clock deadlines during application updates. If application updates themselves stall, the in-process timeout cannot fire. If the Bevy runner returns before a capture is acknowledged, the command reports failure and identifies the reserved output as possibly partial.

# Candidate kind-0 controlled native draw

This records one controlled offscreen diagnostic of the explicit `DAE8CE17` RW4 candidate. Its installed source and shader-selection evidence are in [the RW4 shader note](cell-kind0-rw4-shader-evidence-v1.md). This is not an observed retail actor, pose, camera, resource choice, or frame. The original installation remained read only.

The retained run `target/source-cell/cell-kind0-native-fixture-v1.log` reports exit 0 in adjacent `cell-kind0-native-fixture-v1-hashes.json`, no stderr, and a D3D9 adapter identified as NVIDIA GeForce RTX 3060 Laptop GPU, vendor `10DE`. The compiler reports its loaded module as `C:\WINDOWS\SYSTEM32\d3dx9_27.dll`, entrypoint `main`, flags `12` hexadecimal. The two source texture uploads each report 21,872 compressed bytes and eight mip levels. Input hashes before and after the run match for all eleven recorded paths; an independent 64-bit Python audit also matched each current file to the recorded hash. Key inputs are:

| Input | SHA-256 |
| --- | --- |
| Base `SporeApp.exe` | `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a` |
| `PatchData.package` | `cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff` |
| `Spore_Graphics.package` | `e01bb86d8ddf02c56d52394b4e1db7181c1e1ebe1878bf6fb72c4ee523fcedd1` |
| Loaded System32 D3DX9 compiler | `f9dd288c9895973f8db1856d172779041c6dee173ad1ef53b1727fc85cb6b75f` |
| Fixture executable | `25274ef965b665cb65b80fda61376c2204ab08a8ba1e22d8fa8fe69d53b6fe16` |

The full before/after record also includes the fixture, D3D9/D3DX wrapper, shader-generator and material parser sources. This proves continuity for those named files; it does not independently establish all transitive build inputs or binary-to-source provenance.

The selected fixture recipes were vertex `17 19 15 03 10 31 01` and pixel `05 09 3B 2A`. Their HLSL was assembled from original source fragments and templates and compiled for this diagnostic. The pixel recipe was not present in the selected cached pixel table, so there is no original cached program comparator for it; generated source is not a claim of original bytecode identity. See [pixel generation evidence](shader-pixel-codegen-native-v1.md).

The source and compiled program artifacts are retained under `target/source-cell/cell-kind0-native-fixture-v1-artifacts/`:

| Artifact | SHA-256 |
| --- | --- |
| `vertex.hlsl` | `73a8695679834cb45f35c813f43fb48fbe4570d47023a4c33f11f67dd43117da` |
| `vertex.vs` | `cf9219346800298569ea69816280de460a07db937660807d2c0c3a4cf3b46e0d` |
| `pixel.hlsl` | `38fdc205f51f7594aded79150b892865d5a9c5045c4c0f8deabc0896b067d3b5` |
| `pixel.ps` | `43ee500b0714d546e0abb42bde46acd0914de49e36846b3c2b300a00679abff0` |

All four SHA-256 values and the log's four FNV values were independently checked against the saved files. The log records vertex uniform callbacks `0022`, `0003`, `0004`, `0008` and `0006` at starts `0`, `1`, `2`, `194` and `197`, with row counts `1`, `1`, `192`, `3` and `4`; pixel callback `020C` uses start 0 and one row.

The draw explicitly supplies selector `24E=0`, sampler availability for stages 0 and 1, the stored `3`, `4`, and `210` data, all other shader data absent, a 64-matrix identity palette, white material/tint, a bounds-fit camera, culling disabled, sRGB disabled, and D3D defaults for unspecified states. The saved stage references and their interpreted runtime fixup are conditional. In particular the stored SkinBones start 1/count 8 does not by itself explain the active source index 0; identity rows covering it are fixture values, not a verified original palette. The log records `retail_fidelity=false`.

The saved `target/source-cell/cell-kind0-native-fixture-v1.png` is a valid 768 x 768 RGBA PNG, SHA-256 `f0a65c4fa6b9556ee753e9808032919958ce3790833f60f0fdcf0007c2e3ed55`. Independent decoding found 166,269 pixels different from the clear color, 111,534 distinct RGBA colors, and an inclusive non-background bounding box `(154,154)` to `(614,614)`. The image shows a green textured circular object on a dark field. These observations establish nonuniform output from the controlled run; they do not establish correct retail appearance, pose, lighting, shader selection, state overrides, provider precedence, animation, or gameplay.

## Corrected fixture log: v2

The v2 rerun at `target/source-cell/cell-kind0-native-fixture-v2.log` replaces the earlier ambiguous `first_27_rows_active=true` marker with `palette_rows_0_to_26=identity_fixture`. It preserves the explicit conditional interpretation above: these supplied identity rows do not establish the native palette. The adjacent `cell-kind0-native-fixture-v2-hashes.json` records exit 0 and identical before/after hashes for all eleven named inputs. An independent 64-bit Python audit matched those hashes to the current files and checked every output hash. The original EXE, both packages, compiler, wrappers, generators and material parser hashes match their v1 recorded values; the fixture source and executable changed. Their v2 SHA-256 values are respectively `0afa8ac7f0395f74ed03e0a217a3412cba4c0f980f8e4e43b1e94e680916f1f6` and `9fd4aeebc66b3ea8dab9cafe01952d831185aaf677b4b378c7a0339613df592f`. The v1 source and executable hashes above are historical snapshots and no longer match the current paths after this rebuild.

All four generated shader artifacts in `target/source-cell/cell-kind0-native-fixture-v2-artifacts/` are byte-identical to their v1 counterparts and retain the four hashes above. The v2 PNG is also byte-identical to v1, SHA-256 `f0a65c4fa6b9556ee753e9808032919958ce3790833f60f0fdcf0007c2e3ed55`; independent decoding again gives 768 x 768 RGBA, 166,269 changed pixels and 111,534 distinct RGBA colors. The v2 log SHA-256 is `4ec0c7b8585026217dfc548b4eab54c0c89241f301cc7a40747e30e69898bd6d`. Both runs are controlled diagnostics and retain the same runtime and fidelity limitations.

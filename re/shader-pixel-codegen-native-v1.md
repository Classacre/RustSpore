# Original pixel-fragment generation: cached-program comparison

The bounded pixel source generator in `crates/spore-core/src/shader_pixel_codegen.rs` follows the pinned base executable's `6FAE60` and its fragment declaration, structure, output and placeholder helpers. It reads original templates from the user's executable and fragment text from an explicitly selected installed package. This is a controlled generation and compiler comparison, not proof of a runtime shader selection or rendered frame.

The coordinating run compiled profile-3 source with installed D3DX9 using the `generic_skinning` builder's common flags `12` hexadecimal. The verifier compared generated uniform metadata, register starts and uniform dependency flags, plus executable instruction tokens after excluding D3D COMMENT tokens. This is not byte-for-byte equality of entire shader blobs. Independent inspection of the saved TSV rows confirms every comparison field passed:

| Exact source package | Cached comparisons | Saved report SHA-256 |
| --- | ---: | --- |
| `Data/Spore_Graphics.package` | 403 / 403 | `bf9edb084021b8366b8f9bd7d479f9e83b37d76e96ea5de995fe021a7f71b4af` |
| `Data/PatchData.package` | 477 / 477 | `b003bd374e86cbc6a9aeaae714dadad5c962f76930517709b5fc06b7534ed386` |

The respective directories are `target/source-shaders/pixel-codegen-all-v1/` and `target/source-shaders/pixel-codegen-patch-all-v1/`, with adjacent `.log` and `-hashes.json` files. Both logs record zero mismatches, and both hash records record runner exit 0. Both use native keys `00000003:0469A3F7:40212002` for the fragment library and `00000003:0469A3F7:40212004` for the shader pack. Retained payload SHA-256 values are:

| Package | Fragment library | Shader pack |
| --- | --- | --- |
| Graphics | `0bd3957c95ddf701bac2a5ec1e3fd5e9343594dc8911eb7fab42eaeff3e8cbca` | `7eb8054e5b0d1d7c0f6f555c6a74fbc75c01e2f4c3f1e8251f57d051cb814ff9` |
| PatchData | `aaed3767f67dcae934b65f1ea2884f76c22213ae8b166fe6f8be19df16afba16` | `ab5cc3cf86340ec1f7489cda7f801e8bc2d969e6e8a3babd8b1f34734a506409` |

Each `-hashes.json` records matching before/after hashes for seven named inputs: the original executable, selected package, `System32/d3dx9_27.dll`, verifier executable, pixel and shared vertex generator sources, and verifier source. The original executable hash is `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`; the recorded DLL hash `a70d571cd675c97c9eeb4a234dba1d667ffb54ec3bb14defb36b3e2f605ae257` was collected by a redirected 32-bit PowerShell process and does not identify the x64 verifier's loaded compiler; its actual compiler identity requires a new pinned run. The verifier executable is `6aafedf9cd9eb41a172dd89fed0f21ef79409093b68059132dcdd150000325af`. Package hashes are `e01bb86d8ddf02c56d52394b4e1db7181c1e1ebe1878bf6fb72c4ee523fcedd1` for Graphics and `cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff` for PatchData. These snapshots do not cover every transitive build input or independently prove the binary-to-source build chain. The original build flags for every cached program are not individually established; treating `12` as a common comparison flag is an explicit assumption.

The separate PatchData candidate recipe `05 09 3B 2A` produced a 589-byte HLSL source and 296-byte pixel program in `target/source-shaders/pixel-codegen-candidate-v1/`. Their SHA-256 values are `38fdc205f51f7594aded79150b892865d5a9c5045c4c0f8deabc0896b067d3b5` and `43ee500b0714d546e0abb42bde46acd0914de49e36846b3c2b300a00679abff0`. Its log reports zero cached comparisons and intentionally exits with an error because no cached comparator exists. Compilation alone does not establish instruction equivalence or that the original runtime selects this recipe.

These results cover only the two profile-3 package inventories and chosen compiler/flags. They do not establish source-to-runtime selection, draw-time uniforms, GPU execution, actual lighting or visual fidelity. Original generated HLSL and compiled program bytes remain in ignored `target/`.

## Corrected x64 compiler pin and v2 reruns

The v2 reruns are retained in `target/source-shaders/pixel-codegen-graphics-all-v2/` and `pixel-codegen-patch-all-v2/`, with matching named `.log` files. Both verifier runs identify their actual loaded module as `C:\WINDOWS\SYSTEM32\d3dx9_27.dll` and report `process_bits=64`, `architecture=x86_64`. The combined `target/source-shaders/pixel-codegen-all-v2-hashes.json` records matching pre/post hashes collected by 64-bit Python, both exit codes zero, and the loaded x64 compiler file SHA-256 `f9dd288c9895973f8db1856d172779041c6dee173ad1ef53b1727fc85cb6b75f`. The earlier `a70d...` observation above belongs to redirected x86 PowerShell and is not the compiler identity for the earlier x64 run.

The v2 Graphics report again records 403 / 403 comparisons, and the PatchData report 477 / 477; every row reports matching uniform metadata, register starts, dependency flags and executable tokens with COMMENT tokens excluded. Their TSV SHA-256 values are respectively `bf9edb084021b8366b8f9bd7d479f9e83b37d76e96ea5de995fe021a7f71b4af` and `b003bd374e86cbc6a9aeaae714dadad5c962f76930517709b5fc06b7534ed386`, identical to the v1 reports. The coordinating agent reports an independent recomputation of all 880 token comparisons. The v2 verifier executable SHA-256 is `24dce47bb975c0a6898663b3bc663dd539e45ce05f307c69bbdb99c410030965`. The combined snapshot includes both packages, executable, compiler, verifier, generator sources and `native_d3dx.rs`, but does not establish every transitive build input.

The common `12` compiler flags remain an assumption for the full cached tables, and matching instruction tokens excluding comments is not full byte identity or proof of runtime recipe selection. The earlier uncached `05 09 3B 2A` run was not given a cached comparator by these reruns.

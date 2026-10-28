# Screen-effect material selection: static trace

This supplements `cell-screen-effect-render-trace-v1.md`. Addresses refer to the immutable base `SporeApp.exe` SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. These are bounded static x86 decodes, not evidence of an active effect or a loaded shader.

## Named material entries

Both initialization routines `776DF0` and `777110` allocate ten pointer slots at object +10C and pass count 10, the hash array at `1632040`, and that output array to virtual slot +2C of global `15FE8C8` (obtained by `67D890`). Initializer `12E2960..12E2A3D` calls `932B30` on the following strings and fills the hash array in this order:

| Index | Name |
|---:|---|
| 0 | ScreenAdditive |
| 1 | ScreenBlend |
| 2 | ScreenTint |
| 3 | ScreenBrighten |
| 4 | ScreenSkyBox |
| 5 | ScreenBackground |
| 6 | ScreenAdditiveTexture |
| 7 | ScreenBlendTexture |
| 8 | ScreenTintTexture |
| 9 | ScreenBrightenTexture |

On one startup path `7A7928` calls factory `7136E0`, whose constructor `713520` installs vtable `140C8B0`; `7A7961` publishes the resulting pointer through `67DA90`. That vtable's +2C slot is `712750`, which iterates the keys, calls `712480` with each key, and writes the returned entry pointer into the caller's output array at `7127C4`. If entry byte +0 is zero it writes the key at entry +18 and, when different, copies manager +1B4 to entry +4. Consequently the name identifies a requested table entry, not necessarily the actual state or shader held at +4.

In the first path `7769D4..7769EB` uses the 16-bit selector directly as an array index, and reads entry +4 as the object supplied to `11EE360`. If descriptor mode 5 was accepted and this path executes, the requested entry is **ScreenBackground** (index 5). The candidate's separate key lookup supplies the texture-like object, and does not change this material index. Its resolved resource and the selected state object remain unverified.

In the fallback, `777450` uses jump table `777558` (`00 00 00 00 02 02 01 01 01 01`); mode 5 reaches `7774A2`, sets selector +128 to 1, and sets enabled byte +13C. At `776D54..776D6A`, the sentinel key selects index 1 **ScreenBlend**; a non-sentinel key selects index 7 **ScreenBlendTexture**. The special key and non-sentinel lookup have further branches and can fail or redirect. These statements apply only if that fallback path reaches selection.

## Binding and camera guard

`11EE360` copies selected object +4 to global `16F95A8`, conditionally processes flags at +8/+10/+14 and packed state data via `11EDB60`, `11EE0D0`, and `11EE160`, and accumulates state masks. Those instructions do not independently identify a shader, its technique, or blending values.

`11F1030` marks global state dirty, compares the selected slot's cached pointer, and when changed calls device vtable +104 with slot and either the object's +8 pointer or null. Existing `native-float-render-context.md` traces `16F99D0` to the output of `IDirect3D9::CreateDevice`; for that interface +104 corresponds to `SetTexture`. The main path calls it for slot 0 only if its selected texture pointer is non-null; the fallback calls it even for null. Those differences are visible at `776CBF..776CCE` and `776DBA..776DC6`.

`7C4950` first returns false if the object at receiver +170 has null +40. Otherwise it calculates the absolute difference between ratios of its +80/+84 16-bit fields and +5C/+60 floats; only when the comparison and global byte `153D988` and receiver byte +16C gates all pass does it call `7C44F0`. It then calls `11F3CD0` on +170 and returns false if that returns zero; on success it calls `7C3D80`, updates an associated +158 record from +150/+154, calls `777990` with `201`, and returns true. This is a conditional setup guard; no assertion is made about numerical behavior for zero divisors or unordered comparisons.

The mapping establishes candidate entry names, not loaded technique, actual blend/sampler values, resource-provider precedence, execution, or visual equivalence. No installed game files were modified.

## Additional entry population paths

`7129F0` looks up a caller-supplied key via `712480`, stores a caller count in entry byte +0, and copies the caller's array into entry +4 onward at `712A4D..712A5F`. Separately, `712B10` constructs a key from manager +200, type `2F4E681B`, and global `1536F64`; it queries the global provider and requires a non-null result stored at manager +204. It then queries the separately supplied provider with manager +200, type `0469A3F7`, and global `1536F60`. The resulting stream supplies keys and entry counts at `712D0B..712D5F`. For each item, helper `11E26D0` returns a tagged value; `712E53..712E65` stores its payload into entry +4 onward when the tag equals `2000B`. The original resource identifiers, provider precedence, tagged payload semantics, and state values have not been resolved. The presence of these alternate population paths means table name alone cannot establish the actual bound state.

## Installed candidate resources and program provenance

The initialized PE data words at `1536F60` and `1536F64` are `40212000` and `40212001`. On the identified manager implementation, vtable slot +1C (`712110`) reads two configuration values with keys `1A34E253` and `E1408308` through `6A2110`; it stores the second at manager +200 if the second is nonnegative and the first exceeds it, otherwise stores the first. Thus a particular profile is not established by this static trace. The pair of constructed resource keys in `712B10` corresponds to the header-zero material links and companion compiled states described in `material-links.md`.

The initializer calls `932B30` with mode 1, whose `932B82..932BB3` path lowercases characters and multiplies by `01000193` before XOR. Applying it with the supplied seed `811C9DC5` gives `ScreenBackground = 1F624775`. All twelve installed material-link records recorded in `target/source-shaders/material-links-comparison-v3.json` include this hash and one associated state. Within `PatchData` and `Spore_EP1_Data` the state is section index 280 and shader ID `70000091`; within `Spore_Graphics` it is section index 275 and shader ID `7000008C`. Every record carries one texture key, `[4D272BC5, 00000000]`. These are the library's stored defaults, not proof of the texture finally bound: the screen-effect path may override stage 0 with its independently resolved texture pointer.

The same-package profile-matched shader packs contain a render-type-zero `ScreenEffectsTextureShader` record with those respective IDs and no recorded uniform entries. Pack keys have type `0469A3F7`, separate index group `0001FFFF`, and packed instance `4021200400000000` through `4021200400000003`. Numeric shader IDs cannot be treated globally: `7000008C` is `FilterDepthBlendShader` in PatchData, while `7000008C` is `ScreenEffectsTextureShader` in Graphics. Profile-specific original vertex and pixel bytecode extracted from the package payloads is under ignored `target/screen-material-evidence-v1/`; its `manifest.json` records source package, full pack key, shader record offset, profile, sizes, and SHA-256. The extraction script `target/screen_material_extract_tmp.py` checks source package payloads against the independently retained pack files and verifies shader ID and name before extraction. These files provide reproducible candidates, not a claim that any particular package or profile wins runtime lookup.

The independently parsed state records have the same values across the twelve candidates: render-state pairs `(7,0), (14,0), (15,0), (24,127), (25,8), (27,0)`, and stage-zero sampler pairs `(1,3), (2,3), (5,2), (6,2)`. `11EE0D0` forwards state/value pairs via the created D3D9 device slot +E4 (`SetRenderState`), indicating disabled depth test, depth writes, alpha test and alpha blending, plus alpha reference 127 and function 8. `11EE160` forwards the stage sampler list through slot +114 (`SetSamplerState`), indicating U and V clamp and linear magnification/minification. Its conditional path at `11EE2A5..11EE30C` can replace filter value 2 with 3, so the stored values do not by themselves establish the effective runtime filter. The initial texture placeholder and other state may also be changed by intervening calls. No shader execution or visual acceptance follows from this inventory.

# Cell source asset chain

Scope: the user's installed `D:/Games/SPORE Collection`, inspected 2026-10-22.
The asset loader is `crates/spore-bevy/src/source_cell.rs`. Extracted game images
and raw inventory evidence remain in ignored `target/source-cell/`.

## Measured coverage

- `PatchData.package` contains 223 Cell definitions, all exactly 796 bytes.
- 222 definitions resolve their first little-endian u32 to a `0x4B9EF6DC`
  structure. One unused definition (`65FE5A08`) names structure zero.
- All 169 patched structures consume exactly `28 + 40 * entry_count` bytes.
  Entry kind is word 1 and reference ID is word 4 of each 10-word row.
- 90 Cell definitions contain a kind-3 baked-creation reference.
- 65 of those definitions have installed model props, LOD0 gmdl geometry,
  individual painted atlases, and source thumbnails. These reuse 28 distinct
  creations. All 65 available chains pass the Rust installed-corpus test.
- The other 25 references, covering 17 distinct low IDs, have no art resource
  with that low instance under any prefix/type across all 82 package indexes
  (158,720 records). Some IDs have only their `.cell` and structure resources.
  This is wider than the loader's explicit four-base-package lookup scope.
- Other structure kinds (0, 1, 4, 5, 6) remain unsupported in this loader.
  Their presence is preserved rather than replaced with procedural artwork.

The 25 unavailable baked references and the unused zero reference remain
reported as unresolved. This is not complete Cell-stage content coverage.

## Exact example: Jawhead

The table uses `group:type:instance`, matching the inspection JSON. The binary's
existing `ResourceKey` display uses `type:group:instance`.

| Role | Package | Exact key |
|---|---|---|
| Cell definition | Data/PatchData.package | `0001FFFF:DFAD9F51:000000007D00E384` |
| Structure | Data/PatchData.package | `0001FFFF:4B9EF6DC:00000000285A339B` |
| Baked model prop | Data/PatchData.package | `0001FFFF:00B1B104:40616200780A9EAF` |
| LOD0 gmdl | Data/Spore_Content.package | `0001FFFF:00E6BCE5:40617100780A9EAF` |
| Own painted atlas | Data/Spore_Content.package | `0001FFFF:2F4E681C:40612900780A9EAF` |
| Source thumbnail | Data/Spore_Content.package | `0001FFFF:2F7D0004:40616200780A9EAF` |
| Source rig | Data/Spore_Content.package | `0001FFFF:0F43029A:40616200780A9EAF` |
| Body editor data | Data/Spore_Content.package | `00010000:1A99B06B:40616200780A9EAF` |

The Cell definition's first word is `285A339B`. Its structure's kind-3 row names
`780A9EAF`. Model prop ident `00F9EFBB` explicitly names LOD0. The source mesh
has 11,894 vertices. The primary atlas is 512x512 DXT5. The adjacent
`40612901780A9EAF` is 256x256 raw data and is not used as an albedo fallback.

The executable contains literal `cell_jawhead`, whose lowercase FNV-1 hash is
`7D00E384`; the definition also contains `PLACEHOLDER_Jawhead`. The embedded
developer label is not presented as a recovered player-facing name.

Other measured named definitions and their baked low IDs:

| Executable literal | Cell ID | Baked low ID |
|---|---|---|
| cell_squid | `4BC16582` | `6E9FA321` |
| cell_poker | `7712E0BB` | `6ED040F4` |
| cell_ducky | `9B100A94` | `F5447AEB` |
| cell_Megamouth | `4BC9336F` | `33E71830` |
| cell_Eggkid | `7ADE24E1` | `F7840E98` |
| cell_EggkidSmall | `B5064646` | `F7840E98` |
| cell_StabberAlpha | `EFE530A1` | `A0AE3D5C` |
| cell_puffish | `2DFC83CF` | `DD6F6214` |

## Geometry and material limits

The faithful asset path loads the baked source gmdl directly and requires its
own source atlas. It does not merge standalone part meshes or generate paint.
For Jawhead, the existing standalone assembly resolves 20/20 parts but differs
substantially in bounds from the baked gmdl; successful part resolution alone
does not establish faithful assembled geometry.

The camera now uses the source thumbnail preset's proven orbit orientation,
perspective parameters and clipping settings; see `re/cell-camera.md` for its
native selector, matrix evidence and exact-square aspect branch. Model bounds
fitting, lighting, StandardMaterial shading, UI and background remain inspection
controls. Source shader interpretation, secondary maps, effects, deformation,
animation, water/background composition and full retail camera framing remain
open. Rendering the original mesh and paint does not complete those gates.

`[VALIDATED]` Two offscreen 1280x800 captures, Jawhead (`7D00E384`) and
StabberAlpha (`EFE530A1`), rendered the source geometry, own painted atlas, shipped
thumbnail and inspection annotations using the RTX 3060 Laptop Vulkan adapter.
The images and logs are `target/source-cell/{jawhead,stabber}-render-v1.*`.
Both were visually checked for missing geometry, paint and UI overlap; this is an
asset-loading check, not a retail image-equivalence result. The focused source
suite passes 5/5 including all 65 installed baked-model rows.

## Behavior data limits

`0xDFAD9F51` is Cell definition data, not a string table. All observed developer
labels terminate by byte `0x38`, while a structured numeric region begins at
`0xA4` and continues through the end. Its field semantics are being traced in the
native executable; values are not assigned invented speed/health names.

The legacy `spore_core::string_table` API only exposes an embedded label and the
first word. Its old whole-format-completeness claim has been withdrawn.

The separate PROP primitive endian correction makes source camera values read
correctly (including exact `1.0` and `-90.0` records); see `re/prop-numeric.md`.
Their orbit orientation and projection roles are now traced in `re/cell-camera.md`.
Cell gameplay numeric fields and the exact native developer-name extent remain
open, separately from the camera PROP data.

## Reproduction

```powershell
.\tools\build.bat test -p spore-bevy source_cell::tests -- --nocapture --include-ignored
target\debug\rustspore.exe --source-cell --audit
target\debug\rustspore.exe --source-cell --cell 7D00E384 --shot target/source-cell/jawhead-new.png
```

`--install <dir>` or `RUSTSPORE_INSTALL` selects another local install. Source
previews always render offscreen and refuse an existing screenshot path.
Missing geometry or a missing painted atlas is an error.

Ignored evidence includes `cell-model-inventory.json`, `structure-inventory.json`,
`prop-chain-inventory.json`, `missing-baked-all-packages.json`, raw numeric words,
exact source thumbnails, and decoded source atlas images. No game art is bundled
with the repository.

# Authored creature models and inspection inputs

`[MEASURED]` Saved creation model references are available at the source
CreatureData instance's own PROP. `AssetSource::authored_creature_model` follows
the requested explicit LOD property; the creature inspection target now loads
that saved GMDL and its sampler-zero source map. The older rigid part assembly
remains available through `AssetSource::creature` for diagnostics.

This closes one asset-selection defect in the generic inspection viewer. Bevy's
lighting/material, base texture mip, camera normalization and static saved
geometry do not reproduce the original creature renderer or animation.

## Original loader evidence

`[MEASURED]` Addresses refer to the immutable installed `SporeBin/SporeApp.exe`,
SHA256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The expansion executable is a different target. Evidence and decompilation stay
under ignored `target/rea/20261022-source-runtime-v3/`.

- Model-world virtual `0C` resolves through main vtable `140D740` to `74D3B0`.
  It asks the property manager for the supplied original instance/namespace,
  retains that PROP on the model and starts `74C810`.
- `74C810` loads model options and invokes `74A7F0` on its synchronous route;
  its queue path is separate. `74A7F0` reads `00F9EFBB + i`, for i zero through
  three. The first absent property ends this LOD loop.
- Each successful key uses its first and third words for model identity.
  A third word of `FFFFFFFF` inherits the loaded PROP object's namespace at
  `+10`. The bounded inspection helper requires an explicit namespace and
  reports this inherited case as unresolved.
- `754A90` constructs a GMDL resource key and tries `754850`; if that fails,
  it tries the same identity as RW4. It does not rewrite a `62` namespace byte
  to `71`, and it does not use the authored key's middle word as a dispatch.
- `6A0CF0` is the typed key-getter wrapper (type `20`). Its recovered C argument
  types are unreliable, so the observed LOD call/control flow and key loads
  carry the identity claim rather than those inferred C signatures.

The inspection helper deliberately supports explicit GMDL references, matching
all measured direct LOD0 references. It rejects an absent exact GMDL rather than
claiming the full native GMDL/RW4 loader, inherited PROP context or retail package
precedence has been implemented.

| Retained v3 query | Evidence ID |
| --- | --- |
| 403, model creation `74D3B0` | `ev_ba33d2938560f4c79cc360dac055d6a5f22f0bb0050325e7d319fab0f9ef070c` |
| 408, transform/options `74B660` | `ev_46e7386caefa273735d9028848c43435ecd2092a106ec8927837434dd330707b` |
| 415, model load route `74C810` | `ev_6a4f3d2133ff829def6b9bfd5d0772a4651b8eca33ce3b57c6089e0779bf7ecd` |
| 425, ordered LOD getter `74A7F0` | `ev_57a7d7cd9de19f3b79f0c8380aab1dc16ab4149928382bd738aeb7612dcd6144` |
| 433, GMDL then RW4 lookup `754A90` | `ev_d31c69dafa21af6f708b112c4312def0287ac6d0aa2b76a9e8a344250873e3cc` |
| 434, key getter wrapper `6A0CF0` | `ev_6083975d54f29aed607c808897a552b8181db62e9dc3879e14f388c173b01362` |

## Complete direct-reference audit

`[MEASURED]` The current 82 installed package indexes contain 1,365 distinct
source CreatureData instances. At those exact instances there are 1,065
complete named PROP copies describing 704 source creatures. Their 3,770 direct
LOD references all resolve to installed model resources; all 1,065 copied
LOD0 references resolve to GMDLs. No conflicting LOD0 identity was found between
copies. The other 661 source instances have no same-instance local model PROP.

`[MEASURED]` Independent Rust/Python readers agree on every named selected
property and key in all 1,065 PROP copies. The normalized digest SHA256 is
`6d6d48f459636539a0574dd481b0935ace7e4d8cf9d32ee77ff8b7c212860f3c`.
The retained comparison is `target/animation/creature-model-links-compared-v1.json`.
The canonical feed includes the source payload length, complete declared
property count, selected count, each selected property id/type/specifier and
every raw key word, preserving order. No namespace substitution selects these
model identities.

For source `40626200067C5390`, authored LOD0 is GMDL `40627100067C5390`;
LOD1 is `40627101067C5390`, while LOD2/3 are `40626202067C5390` and
`40626203067C5390`. Source `406B62000660EACA` names a LOD0 GMDL with the same
instance as the source creation. These references demonstrate why one namespace
rewrite cannot implement the authored model chain.

The previous count report had 637 namespace-selected candidate pairs. Direct
PROP keys now corroborate those 637 associations and provide another 67 source
creatures in family `406B`. All 637 prior first-group counts still match the
non-class-one source prefix after the proven native clamp in
[anim-palette-native.md](anim-palette-native.md). This does not turn source row
ordinals into the finalized runtime rig or evaluate its poses.

`[MEASURED]` A separate fresh-payload census now covers all 704 direct LOD0
models, including those 67 additional references. Every first source group is
an absent-resource group with a nonnegative signed-byte count. For all 704,
applying the native first-group clamp to the serialized source row count equals
the non-class-one prefix. The census retains 21,753 serialized source rows and
17,752 prefix rows, complete 128-byte header framing, contiguous class-one tail
checks and SHA256s of every source/model payload. No row is deleted from the
reader. This is a count constraint, not observed finalized runtime matrices.

The saved census is `target/animation/creature-direct-prefix-v1.json`; its
normalized digest SHA256 is
`c82f426f248ab7d47387577ed10123d13c378b4808ad352c7e7daf423327b3b0`.
It builds on the complete independently compared direct reference report and
the separately verified native clamp operation, with no namespace guess:

```powershell
python tools/creature_direct_prefix_probe.py "D:/Games/SPORE Collection" --links target/animation/creature-model-links-compared-v1.json --out target/animation/creature-direct-prefix-v1.json --digest-out target/animation/creature-direct-prefix-v1.tsv
```

## Bounded viewer behavior and verification

`[MEASURED]` `authored_creature_asset(instance, lod)` requires a complete named
PROP, contiguous scalar LOD keys up to the requested slot, the exact referenced
GMDL, complete geometry/material framing and an explicit sampler-zero binding.
All merged meshes must share that map and supported address/filter state;
differing maps produce an explicit separate-draw limitation. Missing or disabled
texture state is an error. There is no lower-LOD, rigid-part, generated-paint or
matching-low-half raster fallback in this path.

The map comes from GMDL shader field `20D` and its last sampler-zero record,
decoded by the independently established `shader_binding::texture_bindings`
reader. The saved pointer word is not dereferenced. The lookup prefers the
actual bound raster identity, with a same-identity RW4 carrier also supported.
The generic viewer preserves explicit addressing and min/mag filtering and
restricts its inspection image to the decoded base mip. Offscreen capture waits
for that exact image handle to reach the GPU.

Six routing/state tests cover different saved LOD namespaces, missing preceding
slots, duplicate/inexact PROP framing, unestablished namespace inheritance,
source sampler selection, state mismatches, and synthetic package misses.
The archive fixture proves that a lower available LOD cannot replace a missing
exact model, and that the actual resolved DBPF group is retained.

`[MEASURED]` An independent Python read of three selected source creations
recovers saved geometry counts of 6,967/11,070, 2,201/1,854 and 2,119/2,602
vertices/triangles, with source maps 512x512, 512x512 and 64x64 respectively.
It records the exact creation, PROP, model and raster keys, eight saved sampler
bytes and base-level RGBA hash. The expected normalized SHA256 is
`bc6690331f3932890fbb1b011becbc88b82453007bc38e68074ba8f49d43fdd3`.
The Rust example agrees exactly with all three independent rows, retained in
`target/animation/authored-creature-asset-compared-v1.json`. GPU capture and
retail scene/pose comparison remain separate validation steps.

```powershell
tools\build.bat test -p spore-bevy --bin rustspore authored_tests
tools\build.bat run -p spore-bevy --example authored_creature_asset_verify -- "D:/Games/SPORE Collection" target/animation/authored-creature-asset-rust-v1.tsv
python tools/authored_creature_asset_probe.py "D:/Games/SPORE Collection" --out target/animation/authored-creature-asset-compared-v1.json --digest-out target/animation/authored-creature-asset-python-v2.tsv --rust-digest target/animation/authored-creature-asset-rust-v1.tsv
```

`[UNKNOWN]` Original creature shaders, complete per-material native draws,
evaluated source poses, finalized generated rig construction, source camera and
gameplay scene behavior remain open. Loading the saved mesh and its real map
does not complete any of those gates.

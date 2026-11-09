# Source UFO model inspection candidate

The [Spore ModAPI group list](https://www.davoonline.com/sporemodder/emd4600/SporeModAPI/namespace_group_i_ds.html)
labels scope `40656200` UfoModels. That classification is a discovery lead;
it does not establish selection or use in an actual Space-stage session.

In explicitly selected `Data/Spore_Content.package` (SHA-256
`0b849bd79d9c13dec5737159655bf7e2b7763cc2b79949c81370fa0490256e8c`),
PROP `00B1B104:4065620024A7C08A` (decoded SHA-256
`76b5076b61cfbe70a8c3e86c28e96c5c022d0c809f53fe0145a046c8d09598d7`)
explicitly names LOD0 GMDL `00E6BCE5:4065620024A7C08A` (decoded SHA-256
`9d646ce8d85aa447739d586762a2a6d31cd84eb2f9eb5b41587a31dd9c1e1af6`).
Both saved material records name sampler-zero raster
`2F4E681C:4065290024A7C08A`, present in that package (decoded SHA-256
`c423f2573823fc139a46ca382fb2e9a92599f3c4e3585fe594b6026f0f0d7ca8`).
Keys here are type plus scope-and-instance inspection spellings. The different
PatchData PROP copy (decoded SHA-256
`b9887ab953e1fe8db444c9e532d26cfb3031838e49a94a68f2e719cbc53de1c1`)
names the same LOD0. This does not decide retail package precedence.

The two material records have settings `07 01 02 02 00 52 14 00`, omitting
the availability update. As described in
[the Civilization inspection note](civilization-source-model-inspection-v1.md),
the incoming availability at an actual draw is unresolved. The strict viewer
therefore renders the geometry untextured and displays the reason rather than
assuming a source texture is enabled.

```powershell
target\debug\rustspore.exe --shot 'D:\Games\SPORE Collection\Data\Spore_Content.package' 'target\space-ufo-source-candidate-v1.png' 0 '' 'prop=4065620024A7C08A'
```

The hidden capture shows 17,073 vertices and 14,292 triangles. Its SHA-256 is
`823523ed37febd7ee2182ad3d164980f780513eeba83d4583044c08d220b0375`.
This is a static Bevy geometry inspection with an inferred classification, not
the original shader, animation, selection, draw state or Space gameplay.

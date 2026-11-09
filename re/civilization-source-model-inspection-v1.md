# Building and vehicle model inspection candidates

An explicit PROP inspection of `Data/Spore_Content.package` (SHA-256
`0b849bd79d9c13dec5737159655bf7e2b7763cc2b79949c81370fa0490256e8c`)
found PROP `00B1B104:4064620062A5E270` (decoded SHA-256
`2fc46ab379926fd253888eea74ee07e23ccf229b3aed50e34edfb49a438c1a01`).
Its LOD0 property explicitly names GMDL `00E6BCE5:4064620062A5E270`
(decoded SHA-256 `b0005e8f89fefb7bca74c30ffca47a062d56abe5227ffa18196ac2f3316041a7`).
Both material records name sampler-zero raster `2F4E681C:4064290062A5E270`
in the same package (decoded SHA-256
`c8fd9c85502fef10359ef0d9e6c8161981e509ce545e99af31ff8bd891c8b324`).
These are type:scope-and-instance inspection spellings; index metadata labels
are not substituted for canonical scope. A PatchData PROP copy has different
decoded bytes (`ff5676839ed12ef76767cdc3c8e0973cff87d4aec461bfb0be167f607ec27376`)
but the same LOD0 key. The viewer checks its primary package first, followed by
attached packages; this inspection order is not established retail precedence.

The [Spore ModAPI group list](https://www.davoonline.com/sporemodder/emd4600/SporeModAPI/namespace_group_i_ds.html)
labels `40646200` VehicleModels and `40636200` BuildingModels. This supports
candidate classification, not selection or use in a Civilization session.
Another Content PROP `4063620006539715` explicitly names GMDL of the same
scoped instance. Neither result identifies runtime creation selection.

The PROP viewer now inspects a GMDL's saved shared sampler-zero map using the
same strict material checks as the authored creature inspection path. A missing
or incompatible map leaves geometry untextured with a visible reason. For the
vehicle above, both material settings are `07 01 02 02 00 52 14 00`: the
availability update flag is absent. A scan found the same settings in 449
building/vehicle LOD0 GMDLs with readable common sampler bindings. These bytes
do not establish the incoming availability state. The captured inspection
therefore remains untextured, with 16,819 vertices and 14,300 triangles:

```powershell
target\debug\rustspore.exe --shot 'D:\Games\SPORE Collection\Data\Spore_Content.package' 'target\stage-vehicle-textured-candidate-v1.png' 0 '' 'prop=4064620062A5E270'
```

The capture SHA-256 is `9933e2f385a00ce0102849a9db044b47c43a6b8fcd0e116c4a02689f2801dda1`.
The existing focused shared-sampler tests passed 2/2. This is a Bevy material
inspection, not an original shader, animation, draw state or gameplay result.

Retained native dossier `target/rea/20261022-anim-rig-finalize-v2/254-analyze_function.json`
shows selector condition 10 checks both pointer array `016F6C58` and availability
array `016F95B4`. Dossier 298 shows `011EDB60` can copy 17 input bytes into
the latter when a state flag is set. A separate bounded PE decode finds another
copy at `011F14C0..011F14DE` from `016FB3AC` when flag `8000` is set.
At `011F0C16`, `xor ecx,ecx` precedes four dword writes at
`011F0C18..011F0C2A` and a byte write at `011F0C96`, zeroing all 17
availability bytes when executed. An apparent routine at `011F0620` follows
padding and a raw direct-call candidate at `011F8267` targets it before
allocation-like calls; these boundaries and reachability are not independently
confirmed. The copy sources, intervening state changes and ordering before this
candidate's draw remain unresolved. The source sampler state must
be established before enabling its texture; a file binding alone is insufficient.

The retained `011EE360` dossier (q851, `target/rea/20261028-source-runtime-v6`)
shows the packed bytes come from a caller-supplied state object, whose flags at
offset `+8` gate `011EDB60`; no specific selected state object has been linked to
this vehicle. In the original PE, `011F11B0` conditionally copies 17 bytes from
`016FB3AC` to `016F95B4`. Direct-call candidates `011F19DE` and `011F1FEF`
pass masks derived from global state; their source bytes and draw ordering have
not been resolved. A raw scan for the absolute address `016FB3AC` found the
copy's read but does not exclude indirect writers. `006FD370` binds the texture
pointer through `011F1030`, but only updates the availability byte when bit 8
of the source settings is present. The candidate's bit is absent. Even though
the initialization path clears availability, later caller-supplied state can
replace it, so neither zero nor one is established for this draw.

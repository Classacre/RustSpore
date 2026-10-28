# Cell ANIM candidate bridge

This is a caller-selected source-data association, not evidence of live actor
selection, resource-manager precedence, controller binding, or playback. The
example reports its inputs as unauthenticated and uses a bounded metadata-only
ANIM projection; it does not validate all clip semantics or sample a pose.

The coordinating agent compiled and ran the metadata-only version against the
inputs below. It joined row 64 and record 1689 to the selected ANIM, matched
the supplied comparison bytes and fixture channels, and matched supplied reply
case 30. The subsequently added focused tests have not yet been run.

From the repository root in PowerShell, create an explicit comparison file:

```powershell
python -c "import sys;sys.path.insert(0,'tools');from gmdl_skin_probe import indexes;from dbpf_extract import payload;from pathlib import Path;p=Path(r'D:\Games\SPORE Collection\Data\PatchData.package');Path('target/cell-mov-cil-reference.bin').write_bytes(payload(p,indexes(p)[4845]))"
```

This comparison file comes from the same selected package entry. Equality is
not independent corroboration. Check the following SHA-256 values separately;
the example does not authenticate hashes or evidence pins:

| Input | SHA-256 |
|---|---|
| `target/source-cell/cell-anim-source-crosslink-v3/table.bin` | `ac4aa6836350b90495f7cd5e20fb38e64986d89472d33f7b1ba8e5e5dfffcfda` |
| decompressed selected AnimList | `df812c5d070e2c3efe6985198e3499f4f74212c9bb901bc0041a3a4bc2e64868` |
| comparison ANIM | `837a340a2b4b08e5c69b075abfa01a9a9d1f18195c446cb2cfbe3a06aa90a802` |
| `target/animation/ungrouped-binding-builder-inputs-v1/input.bin` | `d9d009821bd97d0217af832b3c90d6d5012a53d4ac80451f31418a9b61696319` |
| `target/animation/ungrouped-binding-builder-native-run-v2/native-v1/corpus-output.bin` | `5b5d790761cb63e30ce4b953f2062b23a483bf1eb5fbac46b0d689d9d6eff4d9` |

Run the example:

```powershell
cargo run -p spore-bevy --example cell_anim_candidate_bridge_v1 -- target/source-cell/cell-anim-source-crosslink-v3/table.bin 'D:\Games\SPORE Collection\Data\PatchData.package' 12808 'D:\Games\SPORE Collection\Data\PatchData.package' 4845 64 1689 0 0 target/cell-mov-cil-reference.bin target/animation/ungrouped-binding-builder-inputs-v1/input.bin 30 target/animation/ungrouped-binding-builder-native-run-v2/native-v1/corpus-output.bin
```

The selected table row has state `12` and request `AAAA0060`; list ordinal
12808 identifies a v10 list and ANIM ordinal 4845 has type `EE17C6AD`, group
`0001FFFF`, instance `87CB0177`. Case 30 is a retained prepared-body fixture;
the example checks its channel words and descriptors against the selected
payload and compares its computed output with the supplied reply. It does not
verify the fixture's identity or native provenance. Package files must remain
unchanged between header inspection and the package reader reopening them.

The metadata projection accepts 1–14 channels and 1–24 descriptors per
channel, with at most 100,000 cumulative keyframe rows and 16 MiB cumulative
keyframe spans. It checks span bounds, descriptor strides and widths, and
frame ordering without constructing decoded keyframes. The example also caps
selected stored/decompressed resources at 16 MiB, fixture input at 64 MiB,
and package indexes at 64 MiB and two million entries. These are example
policies, not original-engine limits or proof of complete ANIM validity.

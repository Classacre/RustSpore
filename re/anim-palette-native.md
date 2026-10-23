# Native generated prefix and palette gathering

`[MEASURED]` `crates/spore-core/src/anim_palette.rs` gathers a native matrix
palette from explicit runtime group records, a supplied generated table and
already evaluated SkinsInK buffers. It implements the first-group clamp that
was missing from the earlier producer/gatherer trace. It does not construct
the runtime rig or evaluate original animation curves.

## Actual consumer chain

The source executable SHA256 is
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Native addresses and disassembly refer to `SporeBin/SporeApp.exe`; the expansion
executable has a different digest and must not substitute for this target.

`[MEASURED]` Native constructor stores at `753859..75387A` install main vtable
`140D740`. Destructor stores at `7502BD..7502DE` corroborate that table, with
secondary interface tables at object `+4`, `+8`, `+C` and `+10`. The exact main
table positions are:

| Virtual offset | Table address | Function |
| --- | --- | --- |
| `00` | `140D740` | `D37B30`, AddRef increments object+14 |
| `04` | `140D744` | `750480`, Release |
| `0C` | `140D74C` | `74D3B0`, model creation |
| `98` | `140D7D8` | `744AA0`, generated bone table forwarding |
| `9C` | `140D7DC` | `744AE0`, separate callback setter |
| `A0` | `140D7E0` | `744B30`, separate group-transform setter |

The community SDK's
[IModelWorld declaration](https://github.com/Spore-Community/Spore-ModAPI/blob/master/Spore%20ModAPI/Spore/Graphics/IModelWorld.h)
names virtual `98` as `SetBoneTable`. That declaration supplied navigation;
the original instructions and table establish the implemented consumer.
No SDK implementation code was copied.

`[MEASURED]` Animation bridge `9C6E40` supplies `(model, count, pointer)` at
virtual `98`. `744AA0` obtains the internal model by subtracting eight from
the model interface pointer, visits up to four animation LOD slots beginning
at internal `B4`, and forwards `(count, pointer)` to `73B7C0`. The forwarding
loop stops at the first null animation LOD slot. This is independent from the
mapping between source rows and the pose table produced by the bridge.

`[MEASURED]` `73B7C0` stores a first-group bound before the gatherer runs:

- With no runtime groups, the supplied count is stored unchanged at object+C.
- With groups, group zero's count byte at `group+1` is read as signed i8; the
  smaller of that value and the supplied signed count is stored at object+C.
- The table pointer is stored at object+10, including a null pointer.
- The 64-bit revision at object+90/+94 increments with carry.

Thus a generated table with 21 source-backed rows and a first group of 19
can supply only 19 generated palette rows. Rows beyond the bound must not
overwrite the following external group's evaluated matrices. A null pointer
retains the count update but selects complete group fallbacks in the gatherer.

| Retained evidence | Evidence ID |
| --- | --- |
| v2 query 198, bridge `9C6E40` | `ev_e9714492af5399fa9af83e66f3135211c46f185f81b973d36a91c9f3adf51ecd` |
| v3 query 378, complete 192-byte table read | `ev_5f9c143c41063645d1982dfd2e23b8f5eb1a6d6eed341bf5986ccb5be9a61e0b` |
| v3 query 377, prefix setter `73B7C0` | `ev_af0bd1a976e01ca77e932aebd60169050a9ec5c50efc0cf6988dfeceecaa87f8` |
| v2 query 183, gatherer `740530` | `ev_099e9af5757cb0ea25853cd5fda70e68bbacee4b331749c23de6fc8d72233a32` |
| v2 query 137, group initializer `742D90` | `ev_bb8d3a677169291f4fd8f517ba3f1676fb1627dca1a8b8ab2230c22153625401` |

Ghidra query 376 leaves `744AA0` undefined. Its complete original instruction
range `744AA0..744ADF` is retained from Microsoft SDK dumpbin 14.51.36260.0,
without substituting another procedure's pseudocode. Constructor stores,
the complete forwarding function, prefix setter and source identity are
indexed with file hashes in
`target/animation/model-world-prefix-native-evidence-v1.json`. No retail process
was executed by this static trace.

## Bounded assembly operation

`[MEASURED]` The helper takes contiguous runtime group offsets and counts in
their raw word/byte storage. Native consumers sign-extend both. The bounded
Rust API rejects negative signed values, noncontiguous groups, counts outside
the native signed argument, and short buffers that would be read. These are
explicit defensive errors, not assertions about retail error handling.

`[MEASURED]` A caller explicitly supplies the native nonempty matrix-storage
condition at object+64..68. Ineligible storage or no groups produces no
palette. Eligible assembly copies the generated prefix, then completes each
group with affine identity matrices or the exact evaluated SkinsInK buffer.
For an external first group partially covered by the generated prefix, its
evaluated suffix begins at the same skipped ordinal. A completely covered
group does not read or evaluate a fallback buffer.

`[MEASURED]` Each output is three affine rows of four floats. All twelve
components are copied without arithmetic, retaining signed zero, infinities
and NaN payload bits. Identity is the measured row layout. The operation is
stateless: revision/callback scheduling remains the caller's responsibility.

## Source constraints and independent fixtures

`[MEASURED]` The existing complete GMDL report contains 11,041 models. Among
them, 5,595 have an absent external resource in their first source part; all
of their low-byte counts are in 0..74 with the signed bit clear. Another
5,431 have no source parts and fifteen start with an external resource.
The native initializer supplies its empty-part fallback group separately;
external first-group counts require the actual SkinsInK binding rather than
an inferred source count.

`[MEASURED]` All 637 previously described candidate source/GMDL associations
start with an absent-resource group. Applying the now proven native clamp to
the candidate source row count produces the non-class-one prefix count in all
637. This explains the earlier count difference without deleting parsed rows.
The original namespace selection was a candidate and left 728 unmatched.
The later [direct PROP audit](creature-model-links-native.md) corroborates all
637 associations through authored LOD0 keys and finds another 67 source
creatures. That later evidence establishes these local model references, while
finalized runtime block and palette construction remain separate gates.

`[MEASURED]` A fresh source/model payload census in that later audit extends the
first-group count constraint to all 704 directly linked creatures: 21,753 parsed
source rows, 17,752 non-class-one prefix rows, all first groups absent-resource
with nonnegative signed counts, and all 704 clamped counts equal to the prefix.
It preserves every original row and does not infer the finalized pose table.
The census is `target/animation/creature-direct-prefix-v1.json`, normalized SHA256
`c82f426f248ab7d47387577ed10123d13c378b4808ad352c7e7daf423327b3b0`.

The inventory is `target/animation/creature-body-prefix-candidates-v2.json`.
It retains the hashes of both complete source reports and every candidate pair.

`[MEASURED]` `tools/anim_palette_probe.py` independently chooses each destination
matrix from the generated, identity or evaluated source. It tests all 128
nonnegative signed first-group counts, requests below/at/above each clamp,
null/present table pointers, both fallback types in both groups, and eligible/
ineligible storage. Synthetic components include negative zero and NaN payloads.
The Rust helper uses prefix and group-range copies rather than per-destination
selection.

There are **18,288 cases / 600,440 assembled matrices**, with no independent
Python failures. The normalized expected digest SHA256 is
`9fe64cd8b3a0f68ae40b67b2699da964d053f7382dcd32a66909ccb816850a28`.
All five focused Rust tests pass, and the Rust verifier agrees exactly with
every independent case. The saved comparison is
`target/animation/anim-palette-compared-v1.json`.
The canonical feed is `runtime-palette-v1\0`, seven input fixture words,
bound/copy/output counts, then every matrix component's unchanged u32 bits.
All words are LE u32, and each case uses FNV-1a-64. These are explicit synthetic
poses, not captured original creature animation output.

```powershell
python tools/anim_palette_probe.py --fixtures target/animation/anim-palette-fixtures-v1.tsv --digest-out target/animation/anim-palette-python-v1.txt --out target/animation/anim-palette-python-v1.json --source-inventory target/animation/creature-body-prefix-candidates-v2.json
tools\build.bat test -p spore-core anim_palette::tests
tools\build.bat run -p spore-bevy --example anim_palette_verify -- target/animation/anim-palette-fixtures-v1.tsv target/animation/anim-palette-rust-v1.txt
python tools/anim_palette_probe.py --fixtures target/animation/anim-palette-fixtures-v1.tsv --digest-out target/animation/anim-palette-python-v1.txt --out target/animation/anim-palette-compared-v1.json --source-inventory target/animation/creature-body-prefix-candidates-v2.json --rust-digest target/animation/anim-palette-rust-v1.txt
```

`[UNKNOWN]` Full source resource resolution, finalized runtime body generation,
authored curve evaluation, evaluated external poses and retail creature output
remain separate acceptance gates. Palette assembly does not close those gates.

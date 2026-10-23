# Source material libraries and compiled state framing

This describes installed source data, not completed rendering fidelity. The
typed implementation is `spore-core::material_links`. Original package payloads,
HLSL, native analysis output and generated reports remain under ignored `target/`.
No game assets or translated native implementation are included here.

## Resource identity and complete population

The 82-package installed collection contains twelve header-zero SMT libraries:
four profiles each in `PatchData`, `Spore_Graphics`, and `Spore_EP1_Data`.

| Role | Type | Packed instance high word |
| --- | --- | --- |
| Material links | `0469A3F7` | `40212000` |
| Companion compiled states | `2F4E681B` | `40212001` |
| HLSL fragment library | `0469A3F7` | `40212002` |
| Shader pack/builders | `0469A3F7` | `40212004` |

The low word is profile 0 through 3. A companion is selected by exact package,
type and packed instance; requiring the DBPF index's separate `group_id` field
to match would wrongly exclude the Graphics libraries. This documents the
repository's existing wide-index representation, not a global renaming of keys.
Shader IDs must retain package/profile provenance: the same named shader can
have a different numeric ID between Graphics and PatchData.

The full source population has 5,120 material records, 5,132 compiled states and
6,170 texture-key references. Each PatchData/EP1 profile has 429 materials and
430 states; each Graphics profile has 422 materials and 423 states.

## SMT header zero

All words in this table are big-endian:

| Field | Storage |
| --- | --- |
| Header | `u32`, measured value zero |
| Material ID | `u32`; `FFFFFFFF` terminates the whole table |
| Consecutive state count | `u8` |
| Texture count | `u8` |
| Texture keys | count times `(u32 instance, u32 logical group)` |

Every material consumes its declared consecutive companion RW4 sections. The
first byte is a count, not a state index. Exactly one material in each library
consumes two states; there is no unexplained default state to skip. The sum
consumes every companion section, and every section is `0x2000B`.

The independent endian test found 29 exact installed resource keys among 99
unique texture keys; the same 29 have low-half candidates. Seventy have no
installed resource of any type with either their full key or low half. Their
runtime/builtin role is unresolved. All tested little-endian key permutations
had zero hits; the data supports big-endian instance/group storage.

## CompiledState storage

RW4 state sections are aligned to sixteen bytes. The first seven little-endian
words occupy 28 bytes:

| Offset | Field |
| --- | --- |
| `00` | Complete instanced section size |
| `04` | Primitive type |
| `08` | Soft dirty flags |
| `0C` | Soft delta flags |
| `10` | Hard dirty flags |
| `14` | Hard delta flags |
| `18` | Shader ID |

The four bytes at `1C` are retained alignment padding. Source files include
debug fills, pointer-looking values and old text fragments there; they are not
usable GPU bindings. The aligned variable payload begins at `20`.

`CompiledState::bindings()` currently accepts the source-observed soft extent
mask `0x108034` and zero hard-delta extents. Unsupported combinations are
reported explicitly; their original payload bytes remain available.

The measured variable regions are:

| Soft dirty bit | Extent |
| --- | --- |
| `100000` | `12 * (u16(region + 12) + 2)` bytes |
| `10` | 16 bytes |
| `20` | 12 bytes |
| `8000` | 17 bytes, deliberately leaving following words unaligned |

Bit `4` is present in all observed source states but contributes no bytes in
this framing. The vertex-declaration/prefix bytes remain opaque where their
individual field meanings are not established.

When hard-dirty bit `20000` is present, state groups follow: a group ID, a list
of `(state, value)` words ending with state `FFFFFFFF`, then another group ID;
group ID `FFFFFFFF` terminates the groups. Observed group IDs are only 0 and 5.
The parser preserves these IDs without inventing a render-domain interpretation.

The final texture region starts with an observed `FFFFFFFF` word. Each stage
contains its stage ID, raster reference, a flags word and optional terminated
state/value list A, then another flags word and optional list B. Stage ID
`FFFFFFFF` terminates the region. The parser names the lists A/B until their
native domains are corroborated.

All 5,132 source states parse to their exact ends, with 8,052 state groups and
6,918 texture-stage records. Stage IDs span 0 through 7. Raster-reference words
are exclusively `00400000` (6,170 occurrences) and `FFFFFFFF` (748). For each
of all 5,120 material records, its key-vector length exactly equals its number
of `00400000` placeholders across its consecutive state range. The typed
`texture_slots` API validates this cardinality and exposes state/stage slots;
it does not yet assert key-to-slot ordering or turn saved words into resources.

## Actual generated Cell material path

Jawhead GMDL `40617100780A9EAF` in `Spore_Content` has two meshes with material
ID `D7BE35F9`. Its shader-data `0x20D` lists atlas `40612900780A9EAF`, secondary
map `40612901780A9EAF` and global texture `00000000552A5077`. Its `0x210`
payload has twenty bytes: first LE word `26445C04`, then floats
`20, 0.95, 1, 0.28`.

Neither this mesh material ID nor the first `0x210` word appears as a predefined
SMT material key. Exact installed native literal anchors locate `D7BE35F9`
stores in functions `46A100` and `46B010`: both populate a generated editor mesh
object, store the ID at offset `AC`, then pass it through the export path.
`468610` serializes that field through export schema item `15`. This establishes
generated source data, not a runtime shader selection rule.

Across all 17,730 `0x210` payloads in the complete GMDL population, the upper
24 bits of that first word are always `26445C`; its low byte distribution is:

| First byte | Payloads |
| --- | ---: |
| `01` | 9,187 |
| `02` | 4,769 |
| `04` | 150 |
| `05` | 1,614 |
| `07` | 2,010 |

Source fragment metadata identifies uniform `0x210` as one `float4 materialParams`
(16 bytes). Native reader `767790` selects the raw shader-data byte width by the
actual ID `0x210`; the leading word is not a shader identifier.

`[MEASURED]` Native registration `77E540` binds ID `210` to callback `778F50`.
That callback takes XYZ from payload offsets 4, 8 and 12. It compares bytes 1,
2 and 3 against `5C`, `44` and `26`; only an exact marker match loads W from
16. Otherwise W is exactly 1.0, confirmed by source constant `14861D0`.
It uploads one float4 to the supplied register in either the vertex or pixel
stage. The first-byte selector mode and this marker have separate consumers.
Thus those three bytes are a meaningful compatibility marker, correcting the
earlier padding hypothesis. Jawhead uploads `[20, 0.95, 1, 0.28]`.

`spore_core::shader_binding::MaterialParams` implements the measured conversion,
preserving float bits. It bounds both the legacy 16-byte and marked 20-byte
branches. Source bytes, SDK disassembly and REA byte reads are retained under
`target/rea/20261022-source-runtime-v3/` (queries 35, 41, 42 and
`material210-sdk.asm`). This callback lacks a recovered Ghidra function body;
the branch evidence is bounded SDK disassembly, not decompiler fallback output.

`[MEASURED]` All 17,730 source `210` payloads from 11,041 GMDL resources use the
four-component marker. Rust constant bits agree with independent Python for every
material slot, retaining duplicate payloads and resource/package identities.
Sorted LF record SHA-256 is
`5802fa1ed12a2034770754d824b31a708318e20e13a8fdf3459d90be9a36c0ee`.
Evidence: `target/source-shaders/material-params-comparison-v1.json`.
The legacy branch is verified with synthetic marker/truncation cases; it is not
claimed present in this install.

## GMDL texture sampler bindings

`[MEASURED]` Builder callback `6FD370` reads pointer `16F7DDC`, which is shader
data **20D**, using base `16F75A8 + 20D * 4`. It walks a count followed by
16-byte live records. Native reader `767790` loads at most six records, resolves
each saved raster instance and replaces its saved pointer word. The file form
has 24 bytes per record, including that eight-byte raster instance.

The source texture record contains a sampler index, saved pointer word, eight
settings bytes and raster instance. In the settings, byte 0 gates updates:

| Flag | Update from the source record |
|---|---|
| 1 | Signed byte 1 sets D3D address U, V and W |
| 2 | Signed byte 2 sets both MAG and MIN filters |
| 4 | Signed byte 3 sets MIP filter |
| 8 | Byte 4 sets the sampler availability flag used by selector condition 10 |

An absent flag retains state; it does not imply a default. Bytes 5-7 and the
saved pointer word are retained without dereferencing or invented roles.
Native `6FD2D0` corroborates the cached D3D sampler-state calls.

`[MEASURED]` Rust and independent Python agree on all 42,446 texture records
in the 11,041 GMDL resources, including keys, sampler indices, enabled updates
and signed-byte interpretation. Sorted LF SHA-256 is
`9f5e0fea1e1c7a0f18568176c219b97c28d680fec4afd49e2c73468d255b88d2`.
See `target/source-shaders/material-texture-bindings-comparison-v1.json`.

Both Jawhead materials bind source units 0, 1 and 2 with flags `0F`, address
value 3 (clamp), filters 2 (linear) and availability 1. The third texture,
`00000000552A5077`, exists as a 64 x 64 DXT5 RW4 texture in Spore_Graphics;
it is not an absent procedural substitute. Metadata and independent decoded
RGBA hash are retained in `552A5077-lookup-metadata.json` under the same target
folder. The source preview now derives its skin sampler from these records,
rejecting unestablished or incompatible state instead of selecting a default.
`[MEASURED]` The original 3D renderer now uploads all three texture units and
complete source mip chains. Each standalone RASTER level has its own length
prefix; independent checks cover 56,936 levels in 5,931 resources. All 28
resolved primary Cell creations pass controlled RTX draws with original
bytecode. Full retail state and dynamic material overrides remain separate
gates; see `shader-execution.md`.

Native MaterialLinks orchestration `712B10` independently corroborates the
header-zero count/key grammar and consecutive `0x2000B` section association.
Runtime dynamic-material lookup and final texture injection remain open. The
renderer must not choose a similarly named shader merely to make this model
draw.

## Reproduction and evidence

Use the repository build helper and coordinate Cargo with other agents:

```powershell
tools/build.bat test -p spore-core material_links::tests -- --nocapture
tools/build.bat build -p spore-bevy --example material_links_verify
target/debug/examples/material_links_verify.exe 'D:/Games/SPORE Collection' target/source-shaders/new-material-digests.tsv
python tools/material_link_probe.py 'D:/Games/SPORE Collection' --out target/source-shaders/new-material-comparison.json --rust target/source-shaders/new-material-digests.tsv
python tools/compiled_state_probe.py target/source-shaders --out target/source-shaders/new-binding-comparison.json --rust target/source-shaders/new-material-digests.tsv

tools/build.bat test -p spore-core shader_binding::tests
tools/build.bat build -p spore-bevy --example material_params_verify
target/debug/examples/material_params_verify.exe 'D:/Games/SPORE Collection' target/source-shaders/new-material-params.tsv
python tools/material_params_probe.py target/animation/gmdl-skin-probe-complete.json --rust target/source-shaders/new-material-params.tsv --out target/source-shaders/new-parameter-comparison.json
```

Eleven focused synthetic tests pass, including every truncated binding prefix,
unsupported flags, unaligned lists, absent raster slots, count mismatches and
reserved sentinel rejection. The final Rust/Python pass matches all twelve
per-resource raw and parsed-binding FNV digests with zero mismatches. Both SMT
and compiled-state serialization are byte exact; the binding-region roundtrip
also reproduces every original payload byte.

Reports: `target/source-shaders/material-links-comparison-v3.json`,
`compiled-state-comparison-v3.json`, `rust-material-links-v3.tsv`, and
`gmdl-210-word-constraints.json`. Native evidence
is in `target/rea/20261022-anim-rig-finalize-v2/`, notably queries 205/207
(variable-state extent constraints), 214/215 (exact material-ID anchors),
227 (source geometry/skin initializer), 232 (export field consumer), 221
(shader-data reader), and 262 (MaterialLinks orchestration).

The [community SDK declarations and Materials source](https://github.com/Spore-Community/Spore-ModAPI/blob/master/Spore%20ModAPI/SourceCode/Graphics/Materials.cpp)
were consulted for names and cross-checks after independent source framing.
Their implementation was not copied. Native conclusions above use the installed
artifact evidence, not public version-specific address assumptions.

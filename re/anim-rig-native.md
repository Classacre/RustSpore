# Native ANIM capability and creature rig bridge

2026-10-22. D4 remains incomplete. This investigation establishes source
capability dictionaries, exact per-part capability/morph ownership, and the
native path from creature rows to animation body blocks. It does not establish
the final body/limb retargeting or produce faithful creature animation.

## Target and retained evidence

[MEASURED] Static analysis used the user's installed `SporeBin/SporeApp.exe`,
x86 PE, SHA256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
REA 6.5.0 ran through its pinned local npm MCP stdio server because the app's
deferred runtime did not expose the needed native queries. Ghidra 12.1.4 used
the focused profile already used for paint investigation: Stack, x86 Constant
Reference Analyzer, and Decompiler Switch Analysis disabled. Analysis completed
in 606 seconds. No original-game runtime was launched. The target was immutable;
no EA code, binary, or payload has been added to the repository.

Profile digest:
`07956626c4c75f6e693f1731c6ed414b61cf009d055c0d72055d224a32d58a49`.
Evidence, pseudocode, raw instruction observations, the MCP tool inventory and
Ghidra log stay under ignored `target/rea/20261022-anim-rig-focused-v1/`.
`evidence-bundle-initial.json` contains 54 Evidence records (31,832,831 bytes)
through numbered query 58. Five later native function queries remain as
individual complete Evidence JSON results 60–64. An absent tool name caused
the first helper to close its session via `finally`; the original executable
hash remained unchanged and its Ghidra process exited. This workflow error did
not remove the already exported bundle or individual query results. A second
focused session under `20261022-anim-rig-finalize-v2/` continues finalization
queries; its helper tolerates absent names and saves a snapshot on closure.

[MEASURED] The second session completed the same focused profile in 770 seconds.
Its stage-2 evidence bundle includes the native GMDL trailer and record-generation
queries described below. A stage-4 bundle additionally retains the actual external
bone-record consumer. The stage-6 export retains queries through 230, including
the generated-body matrix feed and the GMDL material reader. The session is shared through one request arbiter for
bounded animation, source-UI and property-reader questions; no second Ghidra
analysis or original-game process is required.

[UNKNOWN] Original symbols/PDB types were unavailable. Numeric entry points
below are observed procedures, not recovered original names. Indirect calls
are not fully resolved by the decompiler. Two observed vtable targets,
`0x9FC460` and `0x9FDFA0`, had no recovered Ghidra function definitions;
instruction observations at the latter address still decode. This is a
function-recovery limitation, not evidence that the code is absent. The large
editor writer `0x4C8110` was inspected in bounded row/header/range regions,
not completely reviewed.

## PartCapabilities dictionaries

[MEASURED] `Anim/partscaps/deformdesc` anchors the capability/deformation system.
`0x9FC160` packs up to four literal characters into a zero-initialized u32;
these are capability fourcc values, not bone hashes. `0x9FC1C0` hashes names
through the lowercase FNV-1 routine and resolves them through a name map.
`0x9FEB50` dispatches binary/text dictionaries; binary reader `0x9FE6E0` and
writer `0xA08540` corroborate the schema implemented independently in
`part_capabilities.rs` and `part_capabilities_probe.py`.

| Source package | Resource key | Version | Bytes | Names / remaps / aggregates / deform groups |
| --- | --- | ---: | ---: | --- |
| PatchData | `7C19AA7A:00010000:000082D0142D3338` | 4 | 4,961 | 88 / 168 / 8 / 20 |
| Spore_Game | `7C19AA7A:0001FFFF:00006F14F8CCFE08` | 3 | 3,475 | 60 / 112 / 8 / 20 |
| Spore_EP1_Data | `7C19AA7A:0001FFFF:000082EA82BC2330` | 4 | 4,961 | 88 / 168 / 8 / 20 |

Each dictionary contains 60 deformation entries. The two v4 payloads are byte
identical; v4 priority bits are `40000000` (2.0). All three deformation tables
are identical. Source SHA256: v4
`c83c0db7dbf029bf0905388476ca2ecb800b8786e0434a0814e84db0f62722c5`;
v3 `5bf3afe7ccef107fdce37bd1f30a615710b283109b09c8a9378f48bdd84f2aaa`.

[MEASURED] Scalar words and name lengths are big endian:

1. Literal `pctp`, version u32, then priority bits u32 only for v4.
2. Name count; each name is length, exact name bytes, capability u32.
3. Remap count; each entry is key, capability, index (three u32).
4. Aggregate count; each entry is capability, member count, member IDs.
5. Deform group count; each group is capability, channel count, then entries
   containing channel ID and three further raw words.

All counts are bounded against remaining bytes. Unknown names/bitfields,
duplicate entries, priority bits and raw parameter words are retained. Parsing
rejects unsupported versions and trailing data. Five malformed/schema tests
passed. Rust opened all 82 package indexes and decoded all three dictionaries,
with no problems. Python's wide-index helper opened 11 and explicitly excluded
71 compact indexes; all three dictionaries occur in the supported set.

[MEASURED] Rust and independent Python canonical digests agree for all three:
v4 `0D57B37B8AB7C917`, v3 `C2C417E93AE63DBA`. Sorted normalized three-line digest
file SHA256 is
`c9ec0c62b0088e8843613638ee4b37b0ab8ad49246b7d2772585127ba425bcea`.
Commands:

```powershell
python tools/part_capabilities_probe.py --out target/animation/part-capabilities.json --digest-out target/animation/part-capabilities-python-digest.txt
tools\build.bat run -p spore-bevy --example part_capabilities_verify -- --digest target/animation/part-capabilities-rust-digest.txt "D:/Games/SPORE Collection"
```

## ANIM descriptors and selectors

[MEASURED] `0x99D010` fixes up an ANIM at load time. Header+324 is channel count,
+328 the channel pointer table; CHAN+220/+224 locate component descriptors.
The loader uses a nine-slot template at `0x14477D0`: INFO, POS, ROT and six
deformation slots. It finds the relevant PCTP layout, matches component ID/type
and rewrites descriptor slot metadata. Unmatched components lose enable bit
`0x10`. Bits 0/1 in the last PCTP parameter word propagate to descriptor bits
`0x80`/`0x40`. Their downstream numerical roles remain unknown.

[MEASURED] A bounded implementation now preserves this once-per-clip template
and prepares source descriptors with an explicit dictionary/fallback context.
See [anim-deform-binding.md](anim-deform-binding.md) for exact selection masks,
flags, slot behavior, independent corpus results and remaining native boundary.

[MEASURED] All 27 hashed nonstandard ANIM tags occur in these PCTP tables;
21 also occur in creature morph IDs. The other six nonstandard ANIM tags are
literal numeric IDs 1..6, each present 21,572 times, with low type flags `3`
and no enable bit `0x10`. A base-game walk example has these six padded slots;
its patched form has only INFO/POS/ROT. The loader/template evidence supports
their role as unused padding, rather than six newly identified morph types.

[MEASURED] CHAN contains two 16-byte selectors:

| Offset | Meaning established by native consumers |
| ---: | --- |
| 136 | Separate channel flags |
| 140 / 144 / 148 / 152 | Primary selector: flags, capability, alternate capability, raw fourth word |
| 156 / 160 / 164 / 168 | Secondary selector, same four-word layout |

`0x99C480` invokes the selector path and handles mirrored comparison.
`0x9B2150` selects **prepared body-block indices**, not RW4 bone IDs. It filters
by per-block capability/number (`0x9B2E60`), modes, spatial bands, class flags,
extrema, and hierarchy expansion (`0x9B1B90`). `0x9B0090` mirrors the relevant
side masks. Root modes, explicit-index mode and capability mode coexist;
names are not the selection algorithm. `0x9FC3E0` selects the PCTP deformation
layout using the primary/alternate capability and selector mode.

[MEASURED] The X spatial mask includes direct side modes `4`/`8` and relative
band modes `20000`/`20004`/`20008`. The latter use the min/max X of matching
blocks, a 0.525 span factor and hundredth-unit quantization; the center band
uses a 0.1-unit tolerance. Other spatial axes, extrema and hierarchy flags
also exist. These measured constraints do not justify applying only a side
test and treating every selector as fully supported.

## Exact creature per-part array ownership

[MEASURED] Native consumer `0x46ADF0`, writer `0x4C8110`, and direct rig loader
`0x9CBFA0` establish the row layout. Previous `row` and `count` u32 labels at
+12/+16 were incorrect. They remain available as raw compatibility fields.

| Row offset | Width | Established field |
| ---: | ---: | --- |
| 0 | u16 | Row ordinal |
| 2 | u16 | Generated joint/node reference; full semantics not established |
| 4 | u16 | Parent row index, `FFFF` for root |
| 6 | u16 | Reciprocal symmetric row index, `FFFF` for none |
| 12 | u16 | Start in flat capability array |
| 14 | u16 | Start in flat morph array |
| 16 | u8 | Capability count for this placed part |
| 17 | u8 | Morph count for this placed part |
| 18 | u16 | Uninterpreted; zero in all measured rows |

[MEASURED] All 1,365 source creatures (1,022 base content + 343 expansion)
contain 46,418 rows, 81,052 capabilities and 57,917 morphs. Every capability
and morph range is in bounds, starts at the preceding range's end, and ends
at its global array length. There are no gaps or overlaps, including empty
ranges. Capability count is not uniformly one. Raw capability IDs must retain
all four bytes, including zero padding; matching via a lossy text string is
unnecessary and unsafe for unknown encodings.

[MEASURED] All parent indices are in range or `FFFF`; all parent chains end,
with exactly one root per creature. All 23,242 non-`FFFF` symmetry references
are in range and reciprocal. These meanings are now native-corroborated,
superseding the earlier structural inference. They do not imply that stored
absolute part transforms should be composed through the parent tree again.

`CreatureData::part_view`, `part_capabilities` and `part_morphs` expose checked
per-part membership. `part_view` also returns the complete original row.
`CreatureCapability::packed_id()` is lossless. Parsing validates the measured
partition invariant; checked accessors remain safe after public-array edits.

[MEASURED] The full Rust range verifier opened 82/82 package indexes and parsed
1,365/1,365 creatures with zero problems. All 1,365 canonical typed/range
digests exactly match the independent Python calculation. Normalized digest
file SHA256:
`c0909c88b63e897ecb5ea5789268b1c680bc68f018d36a7d461179db0a85e0b5`.
The 21-part candidate's digest is `7A070959A1062B3F`.

```powershell
python tools/anim_rig_mapping_probe.py --from-report target/animation/anim-rig-mapping.json --refresh-creatures --out target/animation/anim-rig-mapping-ranges.json --bone-inventory target/animation/rw4-bones.tsv --creature-digest-out target/animation/creature-rig-python-digest.txt
tools\build.bat run -p spore-bevy --example creature_rig_verify -- --digest target/animation/creature-rig-rust-digest.txt "D:/Games/SPORE Collection"
```

## Direct preparation path and remaining deformation boundary

[MEASURED] `0xA0B110` obtains creaturedata then calls `0x9CBFA0` to construct a
definition with 0x468-byte prepared body blocks. `0x9C5130` then constructs the
runtime animation instance. `0x9CBFA0` copies the exact part capability and
morph ranges and maps parent indices into the prepared block ordering. Source
row zero is not always the root: 341/1,365 resources have a non-`FFFF` parent
in row zero, and the native path reorders the nominated root before selection.
Further preparation can insert blocks and rebuild the hierarchy. This ordering
must be retained separately from source row ordinals and RW4 bone indices.

[MEASURED] A selector's position at prepared block+0x10C is the transformed
center of its scaled source bounds, not the stored part translation. The
loader reads scale at source row+44, bounds at +20/+32, rotation at +48 and
translation at +84. It derives local offsets and half extents, clamps
nonpositive extents to 0.1, and derives an effects origin using source+96.
The matrix-to-quaternion helper `0x9A4DF0` uses an xyzw quaternion; its caller
transposes the stored 3x3 before conversion. The complete reference-space
contract with authored ANIM POS/ROT remains unverified.

## Bounded initial source candidate implementation

[MEASURED] `anim_binding.rs` preserves source row/part/prop identity, checked
capability and morph ranges, parent/symmetry links, raw rows, scale, bounds,
stored rotation, translation and effects-origin values. It computes the
initial loader's xyzw orientation, selector center, local origins and clamped
half extents. PCTP aggregate expansion is bounded at 32 capability IDs,
retains original records, and combines duplicate numbers by maximum. The
observed aggregate consumer is `0x9FC210`; malformed cycles terminate safely.

`SourceRig::candidate_parts` supports capability selection, X direct/relative
side bands and six coordinate extrema at the **initial source-block stage**.
Root, hierarchy, class, Y/Z spatial bands, alternate/reserved words and unknown
flags return typed `UnsupportedSelector`. Every successful candidate result
carries `NativeFinalizationRequired`; `require_finalized()` always returns
that typed error. Source row numbers are never presented as runtime block or
RW4 bone IDs. The existing static mesh assembly convention was not changed.

[MEASURED] Rust opened all 82 indexes: initial preparation accepts 1,364/1,365
creatures, 46,395 source parts. The one unsupported resource is
`Spore_Content`, `0F43029A:0001FFFF:40666200A5EDCF7D`, whose row 7 exceeds the
supported 0.01 row-norm-squared tolerance. Its row norms are 1.006525, 1.009536
and 1.006447, with pairwise dot products below 0.000001; this is an explicit
unsupported approximation, not a claim of source corruption. That resource
still passes the creature container and range parser. All 281 core tests passed, including five new range/view tests
and five bounded source-binding tests.

[MEASURED] Independent Python uses direct transposed-matrix geometry; Rust uses
the native matrix-to-quaternion input convention. Across 46,395 supported rows,
maximum absolute differences are 0.001845684 for selector positions,
0.000000798 for half extents, and 0.003567137 for body/effects local origins.
These are bounded cross-implementation checks, not equality with a captured
native runtime. Matrix and quaternion forms need not be bitwise equal for
approximate source matrices and float evaluation; no renderer convention is
inferred from these differences.

[MEASURED] All 1,364 initial-source selector digests match Python exactly for
all ten channels of the explicit patched `BP1_cycle_walk` resource. Normalized
digest-file SHA256:
`5d037a759b457159b8059672785588bcf24570dc377e347a45dae4ad8711d1da`.
The 21-part candidate selector digest is `A600B4A03F5229E7`: head/mouthMorphs
select row 10; eyes 11/12; R/L hand 18/17; R/L foot 16/15; ears select none.
Creature frame/root selectors are explicitly unsupported. Final runtime
bindings remain zero.

```powershell
tools\build.bat run -p spore-bevy --example anim_binding_verify -- --blocks target/animation/anim-binding-rust-blocks.tsv --selection-digest target/animation/anim-binding-rust-digest.txt "D:/Games/SPORE Collection"
python tools/anim_binding_probe.py --out target/animation/anim-binding-python-compared.json --survey target/animation/anim-rig-mapping-ranges.json --rust-blocks target/animation/anim-binding-rust-blocks.tsv --selection-digest target/animation/anim-binding-python-digest.txt --rust-selection-digest target/animation/anim-binding-rust-digest.txt
```

## Static geometry convention comparison

[MEASURED] Row+44 is nonunit on 26,851/46,418 source parts; all measured values
are positive, range 0.2..7. The 21-part candidate has nonunit spine, mouth and
eye scales. This motivated a direct geometry comparison, without changing the
renderer. `tools/creature_geometry_probe.py` reads actual part meshes and
shipped merged GMDLs and compares vertex nearest-neighbor distances in the
source coordinate system; no camera, centering, rescaling or alignment is used.

[MEASURED] Of 1,022 base creatures, 178 have the complete geometry this probe
requires. It excludes 661 without the merged prop and 183 whose first selected
model is absent. Its wide-index helper excludes 71 compact indexes, so this is
not a full-index negative inventory. All four variants use the same 178 cases.
Distances below are mean nearest-vertex distance divided by the corresponding
merged-model bounding-box diagonal; table entries are medians over creatures.

| Part convention | Assembly to merged | Merged to assembly |
| --- | ---: | ---: |
| Stored matrix, no source scale (current baseline) | 1.7358% | 3.6215% |
| Stored matrix, source scale | 1.7395% | 3.4614% |
| Transposed matrix, no source scale | 1.3736% | 2.4914% |
| Transposed matrix, source scale | 1.2812% | 2.4393% |

[MEASURED] Transposed+scaled improves the forward distance over the current
baseline in 163/178 cases. Candidate `40626200067C5390` improves forward
1.5316% -> 0.8678%, reverse 3.3433% -> 2.1603%. A control against the existing
Rust `--creature-overlap` CLI on the earlier four-part `406B62000660EACA` agrees
with the Python baseline: Rust prints 4.6% forward / 4.7% reverse; Python
calculates 4.62357% / 4.65099%. Transposing that case gives 1.36392% / 2.19459%
without scale and 1.39410% / 2.21993% with scale.

[INFERRED] These results support transposed rotation and investigation of scale
for source assembly. Neutral part meshes still omit authored shape/morph and
body deformation, so the lowest distance is not proof of exact reconstruction.
The shipped merged GMDL is the stronger existing reference for the source
creature's complete static geometry. The renderer was not changed in this pass.
Reports remain in `target/animation/creature-geometry-conventions.json` and
`creature-overlap-rust-baseline.log`.

```powershell
python tools/creature_geometry_probe.py --out target/animation/creature-geometry-conventions.json
```

[MEASURED] The debug `.blocks` writer path `0x464D00 -> 0x46CE50 -> 0x46E9B0`
distinguishes jointPosition, cal3DOrigin, effectsOrigin, position, orientation,
scale, capabilities and deformation channels. It generates joint names such
as `joint_b%d_n%d`, with additional bone/connection/end-node classes when
generated skeleton state is present. This is further evidence that selecting
a source part is not yet selecting a single part-model bone.

[UNKNOWN] No renderer applies these recovered body blocks to the assembled
source rig yet. Generated body/limb joints, inserted-block rules, capability
aggregate semantics, authored curves, morph evaluation, inverse-bind/retarget
spaces and mesh deformation need independent acceptance. Generic ANIM
sampling remains explicitly generic, and RW4 clips with direct bone IDs remain
a separate supported path.

## Native finalization constraints

[MEASURED] `0x9C3150` does more than sort the source rows. Its helper `0x9C2FE0`
places each block's direct children contiguously and records direct-child and
descendant extents. Source indices therefore cannot be substituted for finalized
runtime indices. All 341 creatures whose row-zero parent is not `FFFF` point to
the actual source root, whose source class byte is 4; this is a measured source
graph property, not a general rule for generated runtime blocks.

[MEASURED] Native helper `0x9B5DA0` walks a bounded ancestor chain, adds `limb`
capability number 1, and can retain `spin` with number 0 while rebuilding masks.
The capability-presence and nonzero-number masks differ. Finalization can
therefore change the results of a selector even when the original source
capability ranges were read correctly.

[MEASURED] Two observed preparation-policy globals have on-disk defaults
`0x166CFE4 = 0` and `0x1551A84 = 1`. Entry point `0xE4F3B0` saves these settings
and overrides them to 1 and 0, respectively; later sites restore saved values.
The finalizer's root-selection policy depends on these settings. The relevant
runtime context of a particular cached assembled GMDL is not yet established,
so the source preparation API retains `NativeFinalizationRequired`.

## Assembled GMDL external bone records

[MEASURED] Reader `0x7683E0` independently corroborates the trailer schema in
`gmdl_skin.rs`: part count, two 56-byte transforms, one word, three key words,
presence word, and, when present, a count followed by count times 160 bytes.
The key's type word `FFFFFFFF` denotes absence. Runtime part storage has stride
`0x7C`, with transforms at +0/+0x38, the separate word at +0x70, a resolved
resource at +0x74 and optional bone-buffer object at +0x78. The bone-buffer
constructor `0x6FFDA0` stores its array pointer at +8 and count at +12.

[MEASURED] Record copier `0x6FFC90` copies all fields except word indices
3, 18, 19 and 39 into a previously zeroed destination. Word 14 is copied as four
individual bytes. The skipped words are padding candidates; all original words
remain preserved by the parser. Non-finite interpretations of arbitrary raw
words are not sufficient reason to reject the complete record.

[MEASURED] Generator `0x7750C0` fills the record from an input array with stride
48 and another with stride 176. The first array is initialized with identity
quaternion, zero translation and unit scale by `0x76F9A0`. Its translation is
stored at output +0, quaternion at +0x10 and three scale components at +0x20.
The second array supplies triples at output +0x2C, +0x3C and +0x90, a byte at
+0x38, and composed matrix storage at +0x50. These additional triples and the
byte are retained without assigning unsupported deformation roles.

[MEASURED] Independent inspection of the complete 11,041-GMDL report finds
46,576 external records; all 46,576 quaternions at +0x10 have norm squared within
0.0001 of one. Matrix storage is all zero on 23,333 records, including all 11,384
first-bone records. Byte +0x38 is 0 or 1, with 9,835 ones; every first-bone record
has zero there. This is reported in ignored
`target/animation/gmdl-bone-field-invariants-full.json`, using the already measured
GMDL population rather than another package-payload pass.

[UNKNOWN] The 16-word matrix storage is not established as a homogeneous 4x4 or
inverse bind. Its padded fourth components are not consistently an affine last
row. The source vertex shader's `SkinBones[192]` declaration independently
confirms three float4 registers per palette bone, but it does not identify the
body palette missing from these external-part records. A body-count formula
`2 * body_source_rows - 1` fits only 55/955 measured low-half creature joins and
must not drive skeleton construction. Source-to-runtime hierarchy, body palette
generation and animation application remain open. Native record-order constraints
below apply to the exact SkinsInK-linked skeleton, rather than a skeleton selected
only by equal bone count.

[MEASURED] Runtime procedure `0x742D90` iterates the same part records. For an
absent external resource, it reads the low byte of +0x70 as a palette count and
advances a cumulative palette offset. For a present resource, it passes the
resource, optional bone buffer and both transforms to `0x741AF0`. That procedure
creates a binding for each referenced RW4 `0x7000C` SkinsInK section. The measured
schema is consistent with the community SDK's
[SkinsInK field declaration](https://github.com/Spore-Community/Spore-ModAPI/blob/master/Spore%20ModAPI/Spore/RenderWare/SkinsInK.h);
only the data declaration was consulted, and no SDK implementation was copied.

[MEASURED] Setter `0x7965A0` resolves the bone buffer through `0x6FFD60` and
configures its array/count for the RW4 skin-evaluation callback `0x798C80`.
With an external buffer present, that callback selects `0x797970`. The latter
advances the external record by 160 bytes for each sampled bone ordinal. It
combines the sampled quaternion with the record quaternion in sampled-times-
record order, multiplies scale components by +0x20, and adds +0 translation.
These describe measured input adjustments; they are not a complete independent
implementation of the hierarchy/space contract.

[MEASURED] The binding follows the SkinsInK section's skeleton reference and
uses that skeleton's declared count. During evaluation, external record ordinal
`i` is consumed with sampled bone channel `i` and skeleton bone ID `i`; this loop
does not resolve record hashes or reorder records by a guessed name. A successful
join to an arbitrary same-count skeleton does not establish the correct binding.

| Record offset | Measured consumer use |
| --- | --- |
| +0x00, 3 floats | added to sampled translation |
| +0x10, 4 floats | quaternion composed after sampled quaternion |
| +0x20, 3 floats | multiplies sampled scale componentwise |
| +0x2C, 3 floats | translation multipliers when byte +0x38 is zero |
| +0x38, byte | selects the additional matrix-adjustment path |
| +0x3C, 3 floats | translation added in the enabled matrix path |
| +0x50, 16 words | matrix storage read only in the enabled path |
| +0x90, 3 floats | scale factors read in the enabled path |

[MEASURED] All 23,333 all-zero matrix storages in the complete report
have byte +0x38 equal to zero. All twelve basis/translation components read from
enabled matrices are finite. A parser must not require an invertible matrix
when that path is disabled. The consumer outputs twelve floats per bone to
SkinMatrixBuffer; its parent-stack and scale-compensation behavior still need
a separate mathematical specification and source-pose acceptance before use.

## Generated body palette feed

[MEASURED] Palette gatherer `0x740530` has two distinct inputs. It copies a
caller-supplied prefix of twelve-float matrices from the model animation object's
+0x10 pointer, bounded by its +0x0C count and the palette's total count. Remaining
groups either receive identity matrices for an absent external resource, or
receive the referenced SkinsInK binding's evaluated SkinMatrixBuffer. Therefore
the external GMDL records do not supply the generated body's complete rest pose.

[MEASURED] The actual model-world consumer now supplies the missing first-group
bound: vtable `140D740` at virtual `98` selects `744AA0`, which forwards to
`73B7C0`. That setter clamps the generated count to the first runtime group's
signed-byte count before `740530` runs. The native chain and bounded palette
assembly with explicit evaluated inputs are in
[anim-palette-native.md](anim-palette-native.md).

[MEASURED] The animation-to-model bridge `0x9C6E40` reads each finalized body
block's +0x21C palette index, checks it against the definition's +0x37C bound,
and writes a twelve-float matrix at that palette ordinal. Its inputs include
the runtime animation block's position at +0x10 and xyzw orientation at +0x1C;
the translation is divided by the creature instance scale. It then supplies
the matrix table to the model-world interface at virtual offset +0x98. This
establishes the producer/consumer connection. The constructor-scoped index
identity and bounded pose arithmetic recovered below close specific parts of
that connection; complete animation application remains open.

[MEASURED] The optional inserted-block helper `0x9CBAB0` constructs blocks named
`fake_foot_b%d`, gives them the `foot` capability, and assigns +0x21C equal to
`FFFFFFFF`. These blocks participate in animation selection but have no direct
palette row. The observed insertion is not the general generated body/connection
bone construction rule. Finalization still must keep source identity, runtime
block order and palette identity separate.

[MEASURED] The generated-mesh export initializer `0x46E3C0` reads four source
skin-index bytes and multiplies each by three before storing the generated
vertex attributes. This independently corroborates the raw register-offset
interpretation already established from the vertex corpus and source shader
declaration. It does not recover the body hierarchy.

### Bounded source-class prefix relation

[MEASURED] Across all 1,365 parsed source creatures, rows whose class byte at
row+0B is not one form a contiguous prefix; all class-one rows are at the end.
The class byte is retained as the high byte of `CreaturePart::kind`.

[MEASURED] A candidate join replacing the creature instance's namespace byte
62 with 71 finds exactly 637 source/GMDL pairs in the existing complete GMDL
report. For all 637, the GMDL's absent-resource body count equals the number
of non-class-one source rows. The total source row count only agrees for
411/637. For candidate `40626200067C5390`, the source has 21 rows and its last
two have class one; candidate GMDL `40627100067C5390` has body count 19.
This improves on the previously rejected `2 * body_source_rows - 1` hypothesis.

[INFERRED] This relation identifies a useful prefix boundary for further native
investigation. The namespace substitution is a candidate association, not a
recovered resource-resolution rule, and 728 source creatures have no such
candidate in the report. It must not drive a skeleton or runtime palette index.
Native loader `0x9CBFA0` initially assigns block+21C from the source ordinal.
The later constructor-scoped proof below establishes that identity through
finalization; source class one is not a loader exclusion.

[MEASURED] The later [direct PROP/model audit](creature-model-links-native.md)
corroborates these 637 associations through explicit authored LOD0 keys and finds
67 additional source creatures that the namespace guess missed. The creature
inspection target now follows those authored references and each GMDL's real
sampler-zero map. Native model loading, runtime rig construction and rendering
retain the distinct scope and open gates described in that report.

[MEASURED] The bounded child-contiguous ordering stage is now implemented for
explicitly supplied post-reroot runtime parents, root and capability masks in
`anim_order.rs`. Its sibling swaps, permutation and child/descendant ranges are
specified in [anim-block-order.md](anim-block-order.md). This operation does not
infer those runtime inputs from source rows or establish the palette mapping.

The reproducible report is
`target/animation/creature-body-prefix-candidates-v1.json`, including input report
SHA256 values and every candidate pair. No additional source payload was read:

```powershell
python tools/creature_body_palette_probe.py --creatures target/animation/anim-rig-mapping-ranges.json --models target/animation/gmdl-skin-probe-complete.json --out target/animation/creature-body-prefix-candidates-v1.json
```

### Constructor-scoped palette identity through finalization

[MEASURED] Native `0x9CBFA0` constructs a block for every CreatureData row,
including class one. It retains source row ordinal `i` at block+21C
(`0x9CC1B1`), separately from the initial runtime ordinal at +1F8
(`0x9CC1AB`). Its root nomination `R` is the first row's parent if not `FFFF`,
otherwise zero. Source rows initially occupy runtime slots `i+1` for `i<R`,
zero for `i=R`, and `i` for `i>R`. This permutation changes no palette ID.
Source row+0A's low byte is a model-group index, copied to block+33C with zero
represented as `FFFFFFFF`. Row+0B's class byte is separate from that group.
The class-five parent-anchor adjustment does not change block+21C.

[MEASURED] Source-pinned Capstone operand discovery over `0x9B0000..0x9D0000`
found the following direct +21C stores. Discovery by a broad linear scan alone
does not prove code boundaries. Each listed candidate was checked in its
bounded constructor/copy/loader routine; the finalizer's full body and vector
copy/swap callees were then followed independently.

| Store | Bounded routine | Effect on block+21C |
| --- | --- | --- |
| `9B30A1` | default constructor `9B2F90` | sets `FFFFFFFF` |
| `9B3428` | copy assignment `9B32A0` | copies source+21C unchanged, loaded at `9B3422` |
| `9B3DCC` | copy construction `9B3C50` | copies source+21C unchanged, loaded at `9B3DC6` |
| `9CBB9B` | optional foot insertion `9CBAB0` | sets `FFFFFFFF` on newly appended blocks |
| `9CC1B1` | CreatureData constructor `9CBFA0` | stores the source row ordinal |
| `9CD213` | alternate text-loader block constructor `9CD180` | stores an externally parsed ID |

[MEASURED] Optional foot insertion begins at the old block count, extends it
by one or two, and constructs only those appended rows. Their +1F8 values are
new runtime ordinals; +21C is `FFFFFFFF`. The insertion driver `0x9CBE70`
uses spatial/attachment tests before invoking it. It does not add a generic
connection bone for each source part. Existing block copies preserve IDs.

[MEASURED] Finalizer `0x9C3150` first stores the runtime block count at
definition+37C (`0x9C31E6`), then removes only trailing rows whose +21C is
`FFFFFFFF` from that palette bound (`0x9C31FF`, `0x9C3208`). Canonical
CreatureData rows followed by optional fake feet therefore leave all `N`
source palette rows, including class one. Its rest-matrix producer reads +21C
at `0x9C33C4` before rerooting and runtime ordering.

[MEASURED] The finalizer's permutation copies each complete block through
`0x9B32A0` at `0x9C441D`. The write at `0x9C4429` is destination+1F8, not
+21C: ESI points 120 bytes into the destination. The old block+1FC temporarily
stores its new runtime ordinal; subsequent writes remap destination parents
and symmetry IDs. The permutation and rerooting do not renumber +21C.
Array swap `0x9C0D40` either exchanges buffer pointers directly or uses
`0x9C0AC0`/`0x9C0C10`; those paths reach block-copy loops `0x9BE530` and
`0x9B4120`, then the same ID-preserving constructors/assignment above.

[MEASURED] This proof covers `0x9C3150`'s complete measured body ranges
`9C3150..9C3D79`, `9C3D80..9C43E8`, `9C43F0..9C4688`, and
`9C4690..9C48B8` (inclusive). The successful wrapper `0x9C48C0` additionally
requires a finalized count at most 255. Constructor `0x9CD180` is an explicit
exception to source-ordinal identity: text loader `0x9CD250` reads a file with
`group blocks`, `numBlocks`, `block` and `cal3DRootBlockIndex` commands, and
uses externally parsed IDs. A general BodyDefinition or imported text body
must not be assigned CreatureData source ordinals without its construction
provenance.

[MEASURED] A fresh immutable-source census independently decoded all 1,365
CreatureData resources with their full native 128-byte headers: 46,418 rows,
including 7,839 class-one rows. Every class-one set is a contiguous tail and
every such row has a nonzero model-group byte. 23,130 nonclass-one rows have
zero group. All nominated roots have parent `FFFF`; 341 nominations are
nonzero. Counts range from 2 to 100; no resource exceeds the wrapper's bound.
This is constructor-input evidence, separate from the 704 authored PROP/LOD
associations and their generated-prefix counts. Report
`target/animation/rig-source-classes-native-inputs-v1.json` has SHA256
`55aefc099af839934c9855bc374941baed88b64a0a25357fd16d34681d1f676b`.

[MEASURED] Within this canonical constructor context, source palette ID `i`
survives finalized runtime order and is the index read by `0x9C6E40`. Together
with the separate authored model and first-group count proofs, this identifies
which source rows occupy the generated-body prefix. It does not recover the
evaluated runtime pose from ANIM channels, external SkinsInK poses, all reroot
inputs, or authored gameplay behavior.

### Bounded rest and evaluated-pose matrix arithmetic

[MEASURED] `anim_body_pose.rs` implements two explicitly supplied operations:
rest preparation from the pre-reroot block's center (+10C), xyzw orientation
(+118) and retained palette ID; then the bridge from finalized runtime position
(runtime block+10), orientation (+1C), retained palette ID and creature scale
to model matrices. It does not synthesize a hierarchy, select channels or
normalize quaternions. The separate first-group count clamp remains in
`anim_palette.rs`.

[MEASURED] Both measured regions use scalar SSE binary32 arithmetic. The
intervening x87 loads/stores only copy already rounded floats. The implementation
keeps each multiply/add/subtract in the measured order and uses no fused
multiply-add. The independent instruction oracle uses binary32 round to
nearest, ties to even, with gradual underflow; the original process's MXCSR
configuration has not been measured by this static investigation.

[MEASURED] Quaternion-to-basis helper `0x9A4580..0x9A46B1` uses xyzw directly.
Each operation below rounds separately to binary32; products are retained
before the expressions use them:

```text
yz=z*y; zz=z*z; xy=y*x; xx=x*x; xz=z*x
wx=w*x; yy=y*y; wy=w*y; wz=w*z
Q = [1-(zz+yy)*2, (xy-wz)*2, (wy+xz)*2,
     (wz+xy)*2, 1-(zz+xx)*2, (yz-wx)*2,
     (xz-wy)*2, (wx+yz)*2, 1-(yy+xx)*2]
```

[MEASURED] Rest producer `0x9C33C4..0x9C374C` stores the basis of conjugate
`[-x,-y,-z,w]`, then three translations, at definition+3F8 + 48*paletteID.
It computes translation with the expanded quaternion expression below, before
adding the original center. Multiplying a pre-rounded basis by the center
does not preserve these binary32 results. `neg` is a sign-bit flip, including
for zero; all parentheses and left-to-right additions are significant.

```text
xxn=neg(x*x); yyn=neg(y*y); zzn=neg(z*z)
xy=y*x; xz=z*x; yz=z*y
xwn=neg(x*w); ywn=neg(y*w); zwn=neg(z*w)
t0=neg(((((zzn+yyn)*px + (xz+ywn)*pz) + (xy-zwn)*py)*2) + px)
t1=neg(((((xy+zwn)*px + (yz-xwn)*pz) + (zzn+xxn)*py)*2) + py)
t2=neg(((((xz-ywn)*px + (yyn+xxn)*pz) + (yz+xwn)*py)*2) + pz)
```

[MEASURED] The rest intermediate is nine consecutive row-major basis floats
followed by three translation floats. This is distinct from the final output's
three rows of four floats. Runtime `0x9C6E40` uses poses at stride 2BC, aligned
by finalized runtime ordinal, then writes each result by that definition
block's retained +21C. It computes `s = f32(1 / creatureScale)` once with
DIVSS at `0x9C722C`, uses `position * s` for three live translations, and calls
the unchanged quaternion basis helper at `0x9C72D2`.

[MEASURED] Let `A[0..8]` be the rest basis and `A[9..11]` its translation;
`P` is the analogous live basis followed by scaled live translation. Product
`0x9C7AD0..0x9C7DF8` writes affine rows `B[12]` in the following source order.
Its first basis row adds z, x, y; other basis rows add x, y, z. Translation
accumulation also differs by row and must remain explicit:

```text
B0 =(A6*P2 + P0*A0) + P1*A3
B1 =(A7*P2 + P0*A1) + P1*A4
B2 =(A8*P2 + P0*A2) + P1*A5
B3 =((A11*P2 + P0*A9) + P1*A10) + P9
B4 =(P3*A0 + P4*A3) + P5*A6
B5 =(P3*A1 + P4*A4) + P5*A7
B6 =(P3*A2 + P4*A5) + P5*A8
B7 =P10 + ((P3*A9 + P5*A11) + P4*A10)
B8 =(P6*A0 + P7*A3) + P8*A6
B9 =(P6*A1 + P7*A4) + P8*A7
B10=(P6*A2 + P7*A5) + P8*A8
B11=((P6*A9 + P8*A11) + P7*A10) + P11
```

[MEASURED] The Rust boundary requires complete, unique coverage of the explicit
palette count, up to 255; FFFFFFFF rows are skipped before pose fields are read.
It rejects other out-of-range IDs, duplicate or missing IDs, nonfinite inputs
or results, and zero/nonfinite scale or reciprocal. Those are defensive API
errors, not claims about native diagnostics: native out-of-bound IDs are skipped
and repeated IDs can overwrite table storage. Finite nonunit quaternions and
negative finite scale retain the literal measured arithmetic.

[MEASURED] `tools/anim_body_pose_probe.py` decodes only three straight-line
regions from the SHA256-pinned original executable and interprets their scalar
instructions with exact integer binary32 arithmetic. Generated inputs contain
only synthetic values: independently permuted IDs, sentinel rows with unread
nonfinite fields, signed zeros, nonunit quaternions, scale reciprocal boundaries,
and maximum-size palettes. The initial oracle inventory contains 297 cases,
13,134 rest matrices and 13,134 output matrices, plus 132 skipped sentinel
rows. All six focused Rust tests pass, and the Rust verifier matches every
oracle record exactly, with zero problems. The matched digest SHA256 is
`f79a32def722045b99d9657889b923fd28ace1e48670193becaf5d9ec33ce975`.
The completed comparison is `target/animation/anim-body-pose-compared-v1.json`.

```powershell
python tools/anim_body_pose_probe.py --fixtures target/animation/anim-body-pose-fixtures-v1.tsv --digest-out target/animation/anim-body-pose-python-v1.txt --out target/animation/anim-body-pose-reference-v1.json
tools\build.bat test -p spore-core anim_body_pose
tools\build.bat run -p spore-bevy --example anim_body_pose_verify -- target/animation/anim-body-pose-fixtures-v1.tsv target/animation/anim-body-pose-rust-v1.txt
python tools/anim_body_pose_probe.py --fixtures target/animation/anim-body-pose-fixtures-v1.tsv --digest-out target/animation/anim-body-pose-python-v1.txt --out target/animation/anim-body-pose-compared-v1.json --rust-digest target/animation/anim-body-pose-rust-v1.txt
```

[MEASURED] Local native evidence remains in ignored `target/animation/`:
`rig-21c-direct-operands-v1.json`, `finalizer-memory-writes-capstone-v1.txt`,
`rig-finalizer-permutation-capstone-v1.asm`,
`rig-finalizer-post-copy-capstone-v1.asm`, `rig-vector-swap-capstone-v1.asm`,
`rig-vector-copy-ctor-capstone-v1.asm`, `rig-vector-assign-capstone-v1.asm`,
`rig-vector-copy-elements-capstone-v1.asm`,
`rig-vector-assign-elements-capstone-v1.asm`,
`rig-optional-foot-driver-9cbe70-sdk.asm`,
`rig-finalize-wrapper-9c48c0-sdk.asm`,
`rig-finalizer-rest-store-capstone-v1.asm`,
`rig-quaternion-basis-capstone-v1.asm`, and
`rig-model-feed-complete-capstone-v1.asm`. These are static native evidence;
no original process was launched by this operand/matrix investigation.

### Exact canonical source-to-rest intake

[MEASURED] `anim_body_source.rs` is a separate strict geometry intake for the
canonical `0x9CBFA0` source-row path. It reads each complete original 140-byte
row and returns only the rest center at block+10C and xyzw orientation at +118,
with its explicit source ordinal as palette ID. Class-one rows are included.
It leaves `SourceRig`'s existing approximate geometry/selection diagnostics
unchanged and does not construct a finalized hierarchy or runtime pose.

[MEASURED] Source geometry region `0x9CC249..0x9CC5A8` reads bounds at source
+14/+20, source scale at +2C, stored rotation at +30 and translation at +54
(all offsets hexadecimal). Six bound/scale products commit to binary32 first.
Each local center component is `(scaledMax + scaledMin) * 0.5`, in that order.
The loop at `0x9CC340..0x9CC35F` transposes the stored matrix before calling
`0x9A4DF0`: `M[i,j] = sourceRotation[j,i]`.

[MEASURED] Matrix-to-quaternion `0x9A4DF0..0x9A4F6B` has mixed arithmetic.
The trace is the SSE binary32 expression `(M00 + M11) + M22`. For a positive
trace, it loads that committed trace into x87, adds one, and takes FSQRT.
FST commits the root to binary32 for a separate SSE division `0.5 / storedRoot`;
the original x87 root remains live, multiplies by 0.5, then commits the w
component. Each x87 arithmetic operation uses the caller's explicit context.
Numerator subtractions and multiplication by the factor use SSE binary32:

```text
r = x87.sqrt(x87.add(load(trace), load(1)))
factor = f32(0.5 / x87.store_f32(r))
q = [(M21-M12)*factor, (M02-M20)*factor, (M10-M01)*factor,
     x87.store_f32(x87.mul(r, load(0.5)))]
```

[MEASURED] For a nonpositive trace, the helper chooses i=0, replacing it with
one only if M11>M00, then with two only if M22 exceeds the selected diagonal.
Equal diagonals therefore retain the earlier choice. It sets `j=(i+1)%3`,
`k=(i+2)%3`, and performs the following retained x87 sequence before the SSE
component arithmetic. No orthogonality tolerance, normalization, quaternion
sign canonicalization or alternate approximation appears in this routine:

```text
others = x87.add(load(Mkk), load(Mjj))
r = x87.sqrt(x87.add(x87.sub(load(Mii), others), load(1)))
factor = f32(0.5 / x87.store_f32(r))
qi = x87.store_f32(x87.mul(r, load(0.5)))
qj = (Mji+Mij)*factor
qk = (Mki+Mik)*factor
qw = (Mkj-Mjk)*factor
```

[MEASURED] The loader then expands the resulting quaternion to rotate the local
scaled-bounds center. It does not multiply the center by a separately rounded
3x3 matrix. Every operation below is binary32; each product is retained before
the expressions use it, and the source translation is added last:

```text
xxn=neg(x*x); yyn=neg(y*y); zzn=neg(z*z)
xy=y*x; xz=z*x; yz=z*y; wx=x*w; wy=y*w; wz=z*w
rx=(((xz+wy)*cz + (xy-wz)*cy) + (zzn+yyn)*cx)*2
ry=(((zzn+xxn)*cy + (yz-wx)*cz) + (xy+wz)*cx)*2
rz=(((yyn+xxn)*cz + (yz+wx)*cy) + (xz-wy)*cx)*2
restCenter = [(rx+cx)+translationX, (ry+cy)+translationY,
              (rz+cz)+translationZ]
```

[MEASURED] `source_rest_block` checks its full raw row and finite geometry;
`source_rest_blocks` additionally checks the typed/raw row-count agreement and
the canonical 255-row bound. Public edits to decoded typed part fields do not
replace original raw source bytes in this API. These defensive checks do not
claim corresponding native diagnostics. Body/effects local origins, extent
clamping, capability aggregation, root/parent construction and finalization
are outside this returned seven-float rest-input boundary.

[MEASURED] The independent `tools/anim_body_source_probe.py` opens source
packages afresh through the independent full-index DBPF/header reader. Its
bounded interpreter executes the actual geometry instructions, including the
real CALL/RET through `0x9A4DF0`, and retains x87 values as exact rationals with
integer square-root/rounding. It covers all 82 package indexes and all 1,365
CreatureData resources, 46,418 rows, under explicit x87 contexts 007F, 027F
and 037F (24/53/64 significand bits, nearest-even). There are 4,095
resource-context digests and 139,254 evaluated rows, with no failures or
excluded packages. Per context, the helper takes positive trace on 32,095
rows, dominant X on 3,657, Y on 3,559 and Z on 7,107.
This denominator covers CreatureData creation records, including flora; it is
not a verified count of Creature-stage species or completed game behaviors.

[MEASURED] The prior approximate `SourceRig` exception
`0F43029A:0001FFFF:40666200A5EDCF7D` is included in this full oracle inventory;
its imperfect source rotation does not trigger a native matrix-quality filter.
This is not a native runtime capture or proof of a retail control word/MXCSR.
All four focused strict-intake Rust tests pass. The fresh Rust installed-source
verifier opened all 82 indexes and matched all 4,095 resource-context records,
139,254 evaluated rows, exactly, with zero problems. The matched digest SHA256 is
`73a338fa053368d4113ac29a1acb2c6cf3084f6accc23bf90a0138beee296a4a`.
The completed comparison is `target/animation/anim-body-source-compared-v1.json`.
It retains the independent source-instruction report hash and compares that
expected digest against fresh Rust package reads, including every raw source
row in the digest. These conformance results are under the declared arithmetic
contexts; they do not establish the retail thread's later context.

```powershell
python tools/anim_body_source_probe.py --digest-out target/animation/anim-body-source-python-v1.txt --out target/animation/anim-body-source-reference-v1.json
tools\build.bat test -p spore-core anim_body_source
tools\build.bat run -p spore-bevy --example anim_body_source_verify -- "D:/Games/SPORE Collection" target/animation/anim-body-source-rust-v1.txt
python tools/anim_body_source_probe.py --digest-out target/animation/anim-body-source-python-v1.txt --out target/animation/anim-body-source-compared-v1.json --rust-digest target/animation/anim-body-source-rust-v1.txt --reference target/animation/anim-body-source-reference-v1.json
```

[MEASURED] `target/animation/anim-body-source-reference-v1.json` retains every
source payload hash and both instruction-region hashes. Source-pinned local
disassembly is retained in `rig-matrix-quaternion-source-v1.asm` and
`rig-loader-source-geometry-v1.asm` under the same ignored directory.
The additional `rig-finalizer-before-rest-store-capstone-v1.asm` records the
finalizer's earlier branches. No center/orientation rewriting precedes its
rest producer in the measured finalizer path; root/reroot/order handling is
later and remains distinct from source palette identity.

## Key evidence index

| Query / procedure | Evidence ID |
| --- | --- |
| Capability string trace | `ev_e7a2186bdcc9d38dd64e850b5c24d026a5160a2af5ded893c407016dea001eec` |
| Fourcc packer `0x9FC160` | `ev_084537c299be0af2ee3c3de6bc7c9692f4884dd60b40b4e03f3b6f81cde0ccae` |
| Binary dictionary reader `0x9FE6E0` | `ev_76d041ee8f4ef642a1349bafbc7e99038ace7556e358a56399f5630b502c432c` |
| Dictionary dispatcher `0x9FEB50` | `ev_5c1dd0b4cac64da6412625a0b21979767d9d21ba21b5a09428b1e4019787c65e` |
| Binary dictionary writer `0xA08540` | `ev_36a90acd43b39e98f72efa06c8d57080928c84d5f51ec719d26c652a5189ef98` |
| Deform layout `0x9FC3E0` | `ev_aaaff3b93d7aa54514372587f29ce5762bdd53e77233b30b4d69c64f1183a6e1` |
| ANIM fixup `0x99D010` | `ev_ebc40d26197544c28e8935bf950b72adff67410b8d96cf2983998d8512b5ae1a` |
| Block selector `0x9B2150` | `ev_0afdcbc4029eebdc2aeb06b471fe0cf1a3e633ad88435126286b6cbda55efe14` |
| Capability filter `0x9B2E60` | `ev_ec03e4d52e32d0061927f6be16c98092e2c66cb0444e54a02ac81d8a5446b5d7` |
| Selector mirror `0x9B0090` | `ev_2490bc225145c3141c7e304f52c700f5262edf3e3916804c778f4551a7351633` |
| Per-part capability reader `0x46ADF0` | `ev_36d923584b89418d79bd189c8faeafd453c14bc9506bcab73939639d35cb8e22` |
| Creature writer `0x4C8110` (bounded regions) | `ev_64331b2396dbdea4b083f63ae0c4d266baadd94caa854b47bdd33ea207462444` |
| Debug body definition `0x46CE50` | `ev_f598599335bcf2613fc42daf5e0881daa579a6f8f88c25b3b5fd5f57564ee369` |
| Direct body-block loader `0x9CBFA0` | `ev_0789da66e7c2b2f79cb1e09c7665a7ab598558b8968857cb0bc80cdee48647ab` |
| Matrix/quaternion helper `0x9A4DF0` | `ev_0af88a0fb2038af33131c939735c349979d0dcbe1c25040b37de3248e892b775` |
| Finalizer capability relabel `0x9B5DA0` | `ev_e2aa9f86a82dbacd71fbeaa0d315c0fbcd75f5e3f3e88554449e12d965bf580a` |
| Child-contiguous ordering `0x9C2FE0` | `ev_f85f10655fbea984c3a099ff2f4dcdbbe1f5590dab3d109e0373eac3a4ca8397` |
| GMDL trailing-part reader `0x7683E0` | `ev_67599cb0016b48912cfe0e3ea5d1dd1928aea771619dbe21d7d828467890af56` |
| External record constructor `0x6FFDA0` | `ev_0ca52d6e8743867f427726b58b3ec58b2bef9ac723ae52f8d93babb275319280` |
| External record copier `0x6FFC90` | `ev_e7f83b71e0821f7e4f6033b19afb2ff7e5e85b9706b550dcfa4684e6448c4da2` |
| External record generator `0x7750C0` | `ev_6db287944ecf5143914315ff836746d562ccc3de3b693a5b0dd83158ad810c43` |
| Runtime part/count consumer `0x742D90` | `ev_bb8d3a677169291f4fd8f517ba3f1676fb1627dca1a8b8ab2230c22153625401` |
| External-record skin evaluator `0x797970` | `ev_8662a8359d8d37e9824722fa504ea39737ed35a6d3eb5be35d17a1437d0feec1` |
| Skin-evaluation callback `0x798C80` | `ev_99ef4e9bcc3697a4c65e24609e44c2374295747cd8a2c8db5db55e8fee8ef73a` |
| Model palette gatherer `0x740530` | `ev_099e9af5757cb0ea25853cd5fda70e68bbacee4b331749c23de6fc8d72233a32` |
| Animation-to-model matrix feed `0x9C6E40` | `ev_e9714492af5399fa9af83e66f3135211c46f185f81b973d36a91c9f3adf51ecd` |
| Optional foot-block insertion `0x9CBAB0` | `ev_bd862b6c5c6224aac9348833a70a6f916735415ab82b16fe01b94699d82a5d0a` |
| Finalizer `0x9C3150` | `ev_3dbb86207589fa36be3c6f46d0a0dc248ab8efac58dcfc8924f8ad57b7ddee12` |
| ID-preserving block assignment `0x9B32A0` | `ev_080b3803aca72e0b9a090721f4243ffd7eaa78f478d9bc552575ceaa53e49e24` |
| Alternate text-loader block construction `0x9CD180` | `ev_a11a7a047f46d85dd08a3319eee9f49ab6563c0eaf5a9477930cf3b29e15a201` |
| Alternate text-file loader `0x9CD250` | `ev_6304d8ba08bdd30fef1d6df2c0359ef5529213a269ec1f09c8d61ee4bd051304` |
| Generated mesh export initializer `0x46E3C0` | `ev_3f4e7f3f56625b5a34da5fe94c4893fd5c9d6cb5d4dc5fc336ea8a11feff1c60` |

# ANIM to assembled-creature mapping constraints

2026-10-22. D4 remains open: the channel values are readable, but no supported
binding yet turns them into deformation of an assembled creature. This pass
narrows that missing link using original source-package measurements. No target
was selected by guessing a name or by assigning a procedural stand-in.

## Scope and reproducibility

[MEASURED] The Python survey attempted all 82 installed package indexes. Its
existing wide-index helper accepted 11 and explicitly reported 71 unsupported
compact indexes. Those 11 contain all 5,022 ANIM resources and 1,365 creature
resources previously counted by Rust's complete index reader. The full Rust
ANIM run covered 82/82 packages; the independent RW4 inventory covers 13,498 RW4
resources across those 82 packages. The Python pass is not an independent
validation of the 71 compact indexes.

[MEASURED] Survey totals:

| Data | Count |
| --- | ---: |
| ANIM resources / channels / distinct channel-name strings | 5,022 / 48,700 / 487 |
| ANIM component descriptors / distinct tags | 301,684 / 36 |
| Creature saves / part rows | 1,365 / 46,418 |
| Creature capability records | 81,052 |
| Creature morph records / distinct IDs | 57,917 / 76 |
| Referenced distinct part-prop instances | 654 |
| Part-prop records, including package variants | 1,274 |
| Part-prop records whose robust walk stopped | 2, both instance `40686000C75C961B` |
| Direct LOD0 model instances | 631 |
| Direct LOD0 models present in this install | 566 |
| RW4 bone inventory rows / distinct bone IDs | 26,061 / 2,742 |
| Bone rows / distinct IDs for the referenced part models | 5,015 / 745 |

All 654 part-prop instances resolve to at least one prop resource. The 631
direct LOD0 IDs are RW4 models. The 65 absent model IDs are source-content gaps,
not parser exceptions. The property walker consumes 1,272/1,274 records exactly;
the two exceptions remain in the report. Parent-prop inheritance has not been
fully decoded here, so direct-property negatives do not prove absence through
inheritance.

```powershell
python tools/anim_rig_mapping_probe.py --out target/animation/anim-rig-mapping.json --bone-inventory target/animation/rw4-bones.tsv
```

The TSV is emitted by `rw4_animation_verify --bone-inventory`, using its normal
full-install package arguments. It includes package, resource key, skeleton
section/ID/flags, bone index/ID/parent/flags. To recompute joins from the retained
survey without decoding assets again:

```powershell
python tools/anim_rig_mapping_probe.py --from-report target/animation/anim-rig-mapping.json --bone-inventory target/animation/rw4-bones.tsv --out target/animation/anim-rig-mapping.json
```

Complete reports stay under ignored `target/animation/`. Metadata measurement
does not establish that a clip ran in the original game.

## Capability selectors are not direct bone IDs

[MEASURED] CHAN+144 contains 30 distinct nonzero printable capability codes.
Twenty-six of those exact strings also occur in creature capability records,
covering 41,133 ANIM channels. Examples:

| CHAN+144 code | ANIM channels | Creature capability records |
| --- | ---: | ---: |
| `grsp` | 10,835 | 1,280 |
| `foot` | 9,540 | 2,411 |
| `mout` | 8,777 | 972 |
| `eye` | 4,795 | 2,117 |
| `ear` | 4,712 | 624 |

The unmatched strings are `skrs` (303 channels), `psft` (72), `shot` (11), and
`gest` (27). Their absence is bounded to these 1,365 creature saves. Root and
other structural channels can have zero in this selector field. CHAN+160 can
hold a second code, including `mout`, `grsp`, `eye`, `foot`, `spin`, and `root`.

[MEASURED] Names do not uniquely specify a target. `head` and `MouthMorphs` both
use `mout`, while their component sets differ. Hand and foot side flags cannot
be treated as one universal bit convention: in `BP1_cycle_walk`, hand selectors
use R=`8`, L=`4` at +140, while foot selectors use R=`20004`, L=`20000` in hex.
In other clips foot selectors use `6004`, `6008`, `8004`, and `8008`. Native
follow-up now establishes capability, spatial, extremum and hierarchy branches
for these selectors; see [native evidence](anim-rig-native.md). The complete
resolver is not yet implemented.

[MEASURED] Across all 48,700 channels and all 19 selector words per channel,
there are **zero exact numeric matches** against the 2,742 distinct RW4 bone IDs.
This rules out a direct raw-word-to-bone-ID join for this corpus.

[MEASURED] Lowercase FNV-1 hashes of 14 observed channel-name variants match
some global RW4 bone IDs. Lowercase FNV-1a matches none. Restricting the same
test to referenced part models leaves seven name variants: `root`, `Root`,
`Weapon`, `Eye`, `eye`, `shoulder`, and `lookat`. In particular, the common
`head`/`Head`, `L foot`, `R foot`, `L hand`, and `R hand` hashes do not identify
bones in this part-model subset.

[MEASURED] `root`/`Root` both hash to `7BD05F71`; that ID exists in 123 referenced
model instances. Counting qualified candidates across placed parts and all
discovered model variants gives no root candidate for 482 creatures, one for
478, and more than one for 405. Repeated package copies of the same bone record
are collapsed. These are candidate counts, not a chosen package-priority or
binding policy. Name hashing alone does not provide a complete or unambiguous
assembled-creature rig.

[INFERRED] Consecutive part-rig IDs matching lowercase FNV-1 of `joint1`,
`joint2`, and subsequent numbered names explain many common part bone IDs.
For example, candidate names `joint1`..`joint3` hash to `D0BE09E0`, `D0BE09E3`,
and `D0BE09E2`. These names are hash probes, not recovered plaintext labels or
proof that the corresponding joints are the ANIM foot/hand targets.

## Component IDs join morph IDs

[MEASURED] Twenty-one of the 36 ANIM component tag IDs exactly occur in the
creature morph-ID array. Examples include `89E06A31` (11,729 ANIM descriptors),
`75D4C8CD` (5,490), and `47F0B3DC` (5,502). The complete ID/count intersection is
in `join_summary.component_tag_creature_morph_matches`.

[MEASURED] Only one ANIM component tag matches any global RW4 bone ID:
`70E47545`, present in 137 descriptors, also the lowercase FNV-1 candidate
`scale`. A matching hash does not establish that this scalar/morph channel is
a skeletal transform.

[INFERRED] The component/morph intersection supports shared parameter
identities between authored animation and creature morph metadata. It supplies
an investigation anchor for typed morph evaluation, but not its numerical
meaning, applicability, curve policy, or target vertex deformation.

## Creature-row topology

[MEASURED] For all 46,418 rows, the u16 at +0 equals the row ordinal. Native
writer and loader confirm u16+4 is a parent index: all are valid or `FFFF`,
with no self-links and terminating chains. Every one of the 1,365 creatures
has exactly one `FFFF` root.

[MEASURED] Every one of the 23,242 non-`FFFF` values at u16+6 names an in-range
row which points back at the original row through the same field. No tested
mirror reference is one-sided.

[MEASURED] Native writer/loader corroborate parent and symmetric-peer semantics
for fields retained under compatibility names `flags_a` and `flags_b`. Checked
accessors expose `parent_index` and `symmetric_index`. Static part transforms
remain absolute, as measured by the existing assembly comparison. Parentage
does not justify composing those stored transforms a second time.

[MEASURED] Native readers/writers also resolve per-part capability and morph
membership: +12/+14 are u16 starts in the two flat arrays; +16/+17 are their
u8 counts; +18 is an unknown word, zero in all 46,418 rows. Both arrays form
exact contiguous partitions for all 1,365 creatures, including zero-length
ranges: 81,052 capability and 57,917 morph records. Previous u32 `row`/`count`
labels at these offsets were incorrect; the parser keeps those raw fields and
adds precise fields and safe per-part views. Global capability presence can
now be narrowed to particular placed parts without guessing prop names.

[MEASURED] u16+2 differs from the row ordinal and progresses in steps of one
or two; its relationship to per-part joint ranges remains unestablished. The
high byte of u16+10 has five observed class values. These fields, selector
flags and generated body/limb joint ranges still need their complete runtime
semantics before mapping channels through to whole-creature deformation.

## Clips suitable for the next binding experiment

[MEASURED] Filename filtering found 820 resource/package variants across 448
names containing walk/run/locom/gait/sprint/idle. Of these, 818 have finite
recognised POS/ROT values. The exceptions are Spore_Game's
`epic_idle_combat_std` (`3F376548`, group `00010000`) and `idle_combat_strd`
(`A25EC4C2`, group `0001FFFF`). The candidate list is discovery scope, not an
assertion that all 820 are locomotion cycles.

| PatchData resource key | Name | Frames | Measured useful property |
| --- | --- | ---: | --- |
| `EE17C6AD:0001FFFF:00000000DAC89265` | `BP1_cycle_walk` | 209 | Root, head, both hands and both feet have changing finite POS/ROT; their first/last positions match |
| `EE17C6AD:0001FFFF:00000000A4CAE1B5` | `BP1_cycle_run` | 61 | Same six transform channels change, including both feet; some endpoint values differ |
| `EE17C6AD:00010000:00000000155611E8` | `mig_walk_01` | 62 | Root/head/hands change, but each foot has one static key at frame 0 |
| `EE17C6AD:0001FFFF:0000000052BB7FEA` | `walk_neutral` | 54 | Four foot channels each have one static key at frame 0; upper-body/graspers change |

[INFERRED] The latter two are useful negative controls: a walking filename does
not imply a stored lower-body gait. A separate gait/retargeting path is likely
required. `BP1_cycle_walk` is the strongest stored-foot-motion experiment, once
the resolver establishes actual targets. Its matching position endpoints do
not establish continuity of unsupported components or authored curves.

[MEASURED] There are 51 `Spore_Content` creatures which have foot/grsp/mout/eye
capability records, all direct LOD0 models present, and a matching shipped skin
raster set. The smallest is `0F43029A:0001FFFF:40626200067C5390`, with 21 part
rows, two foot records, two grsp, one mout, two eye, eight limb, and two spin.
This is a suitable assembly test candidate after binding is established; the
qualifications do not prove that this creature uses either selected gait clip.

[MEASURED] Exact source membership on this 21-part candidate:

| Capability | Source part rows | Parent rows | Symmetric peer rows |
| --- | --- | --- | --- |
| `mout` | 10 | 0 | none |
| `eye` | 11, 12 | 0, 0 | 12, 11 |
| `foot` | 15, 16 | 3, 5 | 16, 15 |
| `grsp` | 17, 18 | 7, 9 | 18, 17 |

Its Python canonical typed/range digest is `7A070959A1062B3F`. Native selectors
use prepared block positions derived from transformed bounds, rather than
just these source rows' translations. This table establishes membership; it
does not claim a completed retargeting or bone deformation.

## Remaining native questions

[UNKNOWN] Generated joint layout, complete selector/preparation behavior,
reference-space conversion, authored curves, and deformation of the assembled
body remain unresolved. The exact capability-to-part association and the
native selector branches are now established in
[anim-rig-native.md](anim-rig-native.md). The focused native follow-up uses strings
`Anim/partscaps/deformdesc`, `Anim/gait/untagged`, `BDeformBoneBaseJoint`,
`DeformBoneEndJoint`, and `jointPosition %f %f %f`; findings belong in
`anim-rig-native.md` with their own Evidence IDs and scope.

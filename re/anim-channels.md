# ANIM channel rows and generic sampling

2026-10-22 continuation. This records original measurements of locally installed
SPORE package bytes. It does not contain EA assets or decompiled code.

## Verified layout

[MEASURED] `anim_verify` opened all 82 local packages and found 5,022 resources of
type `EE17C6AD`. Every resource parsed with the layout below. An independent
Python reader examined all 5,022 resources in the four animation-bearing
packages: PatchData (2,223), Spore_Game (2,091), Spore_EP1_Data (683), and
BoosterPack_01 (25). The Python reader is restricted to its existing wide DBPF
index format; it is not the evidence for opening the other 78 packages.

Offsets in this table are absolute until the CHAN section.

| Offset | Measured interpretation |
| --- | --- |
| 284 | float frame duration |
| 288 | float duration in frames |
| 296 | raw word, previously called `last_frame`; not a reliable key bound |
| 304 | raw word, previously misidentified as the channel count |
| 324 | channel count |
| 328 | channel-table pointer, 512 throughout this corpus |
| table + 4*i | absolute pointer to a CHAN section |
| CHAN + 8..136 | bounded channel name |
| CHAN + 136..212 | 19 selector/flag words, retained raw |
| CHAN + 212 | keyframe row count |
| CHAN + 216 | absolute row-data pointer |
| CHAN + 220 | component-descriptor count |
| CHAN + 224 | absolute component-descriptor pointer |

[MEASURED] Each descriptor occupies 32 bytes:
`flags:u32, tag:[u8;4], offset:u32, stride:u32, slot:u32, reserved:[u32;3]`.
The flags precede the tag. All descriptors within a channel share a stride, and
each component offset addresses a field inside an interleaved row. The frequent
200-byte stride is a byte count, not a key count. Channels with additional
components can have larger strides.

[MEASURED] INFO starts with an integer frame index followed by four words. POS
starts with three floats. ROT starts with four quaternion components in xyzw
order. INFO/POS/ROT commonly start at row offsets 0/20/100, but the parser reads
the descriptors instead of hardcoding these offsets. POS and ROT can be absent.
The source rows contain further component bytes, which remain available without
discarding unknown data.

[MEASURED] The old `last_frame` interpretation fails, for example, on instance
`A931EF6B`: the header word is 72, duration is 586, and actual keys span 0..585.
Call `AnimBone::frame_range()` for actual channel key bounds.

## Extraction and sampling coverage

[MEASURED] The Rust corpus run produced:

| Quantity | Count |
| --- | ---: |
| Parsed animations | 5,022 / 5,022 |
| Channels | 48,700 |
| Keyframe rows | 878,791 |
| Position keys | 731,075 |
| Rotation keys | 731,383 |
| Empty channels | 238 |
| Unsupported component descriptors retained | 173,204 |
| Nonfinite source position keys | 4 |
| Source rotations outside norm-squared tolerance 0.01 | 1,050 |
| Generic sample errors at keys and adjacent midpoints | 8 |

[MEASURED] All 48,462 nonempty channels have nondecreasing integer frame indices.
Finite positions number 731,071; every extracted quaternion is finite. The four
nonfinite positions remain represented in parsed source data, and sampling
returns a typed error when it touches those values. Eight exact-key/midpoint
sample attempts encountered these source anomalies. Parse success therefore
does not mean that every source value is a usable runtime transform.

[MEASURED] The API preserves selectors, descriptor flags/tags/slots/reserved
words, INFO words, and complete raw interleaved rows. `is_supported()` explicitly
distinguishes the three recognised tags from other component IDs. Empty channels
sample to `None`. Missing position or rotation components remain `None`.

[INFERRED] POS float3 and ROT quaternion interpretation is supported by component
tags, numeric distributions, coherent key sequences, and quaternion norms. These
are raw local/channel values. Their relationship to a particular runtime bone
or model requires independent selector and bind-pose evidence.

[MEASURED] Sampling offers the caller `InterpolationPolicy::Step` or
`InterpolationPolicy::LinearSlerp`. The latter linearly interpolates positions
and performs normalised quaternion slerp over the shortest arc, with normalised
linear interpolation for near-equal quaternions. The sample frame is clamped to
the actual channel key range. Duplicate frame indices select the final key at
that index. Nonfinite sample times and invalid position/quaternion values return
`AnimSampleError` rather than producing invalid poses.

[UNKNOWN] The retained selector words have not been mapped to model skeletons.
Authored curve controls, hashed component semantics, morph retargeting,
inverse-bind conventions, and faithful SPORE animation evaluation remain open.
The generic interpolation API does not claim to evaluate those authored curves
or to supply faithful playback.

## Reproduce

Run these commands from the workspace, keeping outputs under ignored `target/`:

```powershell
.\tools\build.bat test -p spore-core
.\tools\build.bat run -p spore-bevy --example anim_verify -- --digest target/anim-rust-digest.txt "D:/Games/SPORE Collection"
python tools/anim_channel_probe.py --out target/anim-channel-probe.json --digest-out target/anim-python-digest.txt
Compare-Object (Get-Content target/anim-rust-digest.txt) (Get-Content target/anim-python-digest.txt)
```

[MEASURED] The core test run passed 169 tests, including 16 ANIM tests. ANIM tests
cover actual row counts and values, unknown components, real key bounds,
malformed descriptor ranges, truncated prefixes, frame ordering, interpolation,
shortest-arc quaternion normalisation, and explicit invalid-value errors.

The per-resource digests use FNV-1a-64, initially `CBF29CE484222325`, multiplying
by `100000001B3` after XOR of each byte. All integers are little endian u32, and
float payloads use their original u32 bits. The feed is:

1. Channel count.
2. For each channel in table order: UTF-8 name length and bytes, row count,
   19 selector words, descriptor count, each complete 32-byte descriptor.
3. For that channel: decoded key count, then each key's frame and four INFO
   words; position-presence word and three float words if present; rotation
   presence word and four float words if present.
4. For that channel: raw-row byte length and complete raw rows.

Digest lines identify package filename and full resource key. This checks both
decoded values and preservation of unsupported/curve data; it is not a proof of
native runtime semantics.

[MEASURED] The independent Python and Rust runs produced 5,022 digest lines each,
with zero differing lines. This agreement covers the full typed key corpus and
all retained raw rows, including unsupported components and nonfinite float bits.

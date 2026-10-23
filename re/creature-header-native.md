# Native CreatureData reader and complete header

2026-10-23. This corrects the earlier 68-byte retained header and chunk-repeat
assumption. It does not establish generated body palette indices or playback.

## Original reader path

`[MEASURED]` The installed executable SHA256 is
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Native `4BC760` requests resource type `0F43029A`; its typed interface is
`03E1C247`. The resource factory constructor at `4BD770` installs vtable
`13EFF38`. Its virtual `+24` callback is `4BDCC0`, which obtains CreatureData
through `4C1C60` and invokes the actual serialized reader `4C0BB0`.
`4BDDA0` is the following factory hook, invoking writer `4C0EF0`; it is not
the read callback. This distinction is established by the constructor's table
address and the observed stream read/write operations.

| Native v3 query | Evidence ID | Established contract |
| --- | --- | --- |
| 318, factory table assignment | `ev_296f984fc864d0c66d1a7647af2ef0a7cf2c47f2df20c437587928d6a87ff937` | Actual vtable base, not an adjacent guessed address |
| 320, `4BDCC0` | `ev_ea2630a9bf65ea890245bc2b2447668680cec05b216c54c41e16e329bd3595ee` | CreatureData read dispatch |
| 321, `4C0BB0` | `ev_99c49b60eeb1906b92761ae427c1109275ac80a6cf0cc48d3607c48146691ffd` | Header extent, counts and direct array reads |
| 331, `4C0A20` | `ev_b971baf2e25074d0a0ceef6a8bad2e2f406b4dd144d6c2d62befba2f7f7cc9e5` | Exact native chunk/escape/repeat behavior |
| 327, `4C0820` | `ev_67f88d21ea4f0908f21466f10a27f377e2e6390b8fdf2d96befea4b5277f9727` | Matching header writer and repeat state |

Complete native Evidence remains under ignored
`target/rea/20261022-source-runtime-v3/`.

`[MEASURED]` The reader accepts versions 9 and 10, checks part/capability/morph
counts against 256/4,096/1,024, and decodes 128 bytes into object `+18` before
reading the arrays. The decompilation also contains a 68-byte version-before-8
branch, which cannot be reached through this version-9/10 guard. The writer
emits version 10 and the same 128-byte header.

`[MEASURED]` Chunk sizes are LE u16 in 1..2048. A literal updates the previous
decoded byte; `FE 00` emits FE and updates that state. A nonzero repeat count
copies the previous byte without changing it. The state starts at zero and
persists across chunks. A dangling escape inside a chunk and a repeat extending
beyond the requested output are failures. The encoded chunk is read in full
before decoding, so the raw framing is retained even if output completes
before the chunk's last encoded byte.

`[MEASURED]` The row reader copies every declared `0x8C` source row directly.
It does not filter class-one rows at this boundary. The candidate body-prefix
relation in `anim-rig-native.md` must therefore not be turned into a rule that
deletes those rows from parsed CreatureData.

## Implementation and independent corpus

`[MEASURED]` `creature_data.rs` now retains all 128 decoded header bytes,
preserves repeat state across chunks, and rejects the two malformed native RLE
cases. Public data fields remain available. Native count limits apply before
header reads; the fourth version-10 array keeps the previous defensive cap.
The palette stays at its measured offset within the full header.

`[MEASURED]` The independent Python reader completed 1,365 resources, comprising
1,022 version-9 base creatures and 343 version-10 expansion creatures. It uses
the original package payloads and the native 128-byte decoder directly, without
the earlier 68-byte framing helper. Its wide-index reader excludes 71 compact
indexes; this is not evidence that Python opened all 82 indexes.

| Quantity | Independent result |
| --- | ---: |
| Complete decoded headers | 1,365 / 1,365 |
| Decoded header bytes | 174,720 |
| Previously discarded header bytes | 81,900 |
| Nonzero bytes in that discarded tail | 0 |
| Headers using multiple chunks | 0 |
| Array framing / native count errors | 0 |

All source headers fit in one encoded chunk. The prior 68-byte reader consumed
that whole chunk and thereby retained correct following-array offsets while
discarding 60 decoded bytes. Existing installed images should consequently
remain unchanged by full-header retention. Synthetic tests separately exercise
nonzero tail bytes, literal FE, cross-chunk repeat state, malformed RLE,
truncated prefixes and native count boundaries.

The normalized independent header digest SHA256 is
`cc2034f2f47396d9571eaa4618dc4f33ea24075b84e167bbe84db26791d098df`.
The fresh Rust verifier agrees on all 1,365 header digest records, retaining
174,720 decoded bytes. The existing 1,365 per-part rig digest records are
unchanged. All eighteen focused CreatureData tests pass; the remaining
animation-binding fixture now uses the same public 128-byte header constant.
Comparison output is `target/animation/creature-header-compared-v1.json`.
The canonical feed is `creature-header-v1\0`, version, encoded byte count and
bytes, decoded byte count and bytes, following-array offset, and total consumed
bytes. Integers are LE u32; the digest is FNV-1a-64. Package filename and the
complete resource key identify each digest line.

```powershell
tools\build.bat test -p spore-core creature_data::tests
tools\build.bat run -p spore-bevy --example creature_rig_verify -- --candidate NONE --digest target/animation/creature-rig-rust-header-v1.txt --header-digest target/animation/creature-header-rust-v1.txt "D:/Games/SPORE Collection"
python tools/creature_header_probe.py --out target/animation/creature-header-compared-v1.json --digest-out target/animation/creature-header-python-v1.txt --rust-digest target/animation/creature-header-rust-v1.txt
```

`[UNKNOWN]` The additional zero bytes do not establish new field meanings.
Generated body order, runtime palette association, authored curves and faithful
creature animation remain open.

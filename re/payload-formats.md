# Payload format notes

Working notes on the resource payload types, as of 2026-10-16. Grades: `[MEASURED]`,
`[INFERRED]`, `[UNKNOWN]`.

The container and the compression codec are solved and validated
(`docs/dbpf-format.md`). This file is about what is *inside* the payloads, which is
mostly still open. It is deliberately blunt about that.

## Summary

| Type | Name | Status |
|---|---|---|
| `0x2F7D0004` | PNG | **decoded.** Standard PNG. 13,174 extracted, all CRCs valid. |
| `0x00B1B104` | prop | **not decoded.** See below. |
| `0x2F4E681B` | RW4 | not decoded. Dispatch strategy identified, see `docs/engine-architecture.md` section 3. |
| `0x01C135DA` | gmsh mesh | not decoded. |
| `0xEE17C6AD` | anim | not decoded. Bound to Cal3D, so the format is public. |
| `0x5CAA5E28` | blockdata | not decoded. |

## prop, type 0x00B1B104

The most common type in `Spore_Game.package`: 7,030 of 11,529 resources.

Status: **partially decoded.** 1,612 of 7,030 resources (22.93%) parse exactly, meaning
the walk ends precisely on the last byte.

### Confirmed

`[MEASURED]` The layout is publicly documented (SimsWiki, via `docs/prior-art.md`
section 4.1) and the documentation matches the bytes:

- `u32` item count, **big-endian**
- per item: `u32` identifier (BE), `u16` type (BE), `u16` specifier (BE)
- specifier `0x009C` means an array, preceded by an array number and array size, both
  `u32` big-endian
- strings are a big-endian `u32` length followed by that many bytes, UTF-16BE for type
  `0x13`

`[MEASURED]` The documented type ids appear in real data, which is independent
confirmation of the record header:

| Type | Name | Occurrences in `Spore_Game.package` |
|---|---|---|
| `0x000d` | float | 3,829 |
| `0x0020` | key | 3,533 |
| `0x000a` | uint32 | 1,691 |
| `0x0013` | string (UTF-16) | 1,475 |
| `0x0001` | bool | 754 |
| `0x0009` | int32 | observed |
| `0x0034`, `0x0033`, `0x0031`, `0x0012` | colorRGBA, vector4, vector3, string8 | observed |

`[INFERRED]` A useful sanity check on the earlier work: the first attempt read
`00 13 00 0d` as a constant u32 `0x0D001300`. It is in fact `type = 0x0013` plus
`specifier = 0x000D`, which is exactly where the documentation puts them. The earlier
reading was one field-width away from correct.

### Padding, measured

`[MEASURED]` The documentation warns that padding varies per type and per
single-versus-array context, and that a single `key` carries an extra u32. Measuring the
effect on the exact-consumption rate over 7,030 resources:

| Configuration | Resources consuming exactly |
|---|---|
| no padding | 459 (6.53%) |
| `key` +4 | 1,322 (18.81%) |
| `key`, `vector3` +4 | 1,328 (18.89%) |
| all compound types +4 | **1,612 (22.93%)** |

So `key` padding alone accounts for most of the gain, and the wider compound set is
confirmed by the same measurement.

### `texts` (type `0x22`), layout derived exactly

`[MEASURED]` Solved by arithmetic rather than by guessing, using props that contain
exactly one record of this type. Three such props have a 540-byte blob, so the record
occupies 540 - 4 = **536** bytes. The first field inside the value is **520**, and
`536 - 520 = 16`, which fixes the header at 12 bytes following the count:

```
u32 count, big-endian
per entry:
    u32 buffer_size in BYTES, big-endian      (520 = 0x208 in 1,784 of 1,793 samples)
    u32 unknown
    u32 unknown
    buffer_size bytes of UTF-16LE text, NUL padded
```

So the value length is `4 + count * (12 + buffer_size)`. `count` is 1 in 1,784 of the
1,793 reachable records.

### Mixed endianness, and a corrected error

`[MEASURED]` Strings carry a **big-endian** length but **little-endian** UTF-16 characters:
`00 00 00 07` followed by `44 00 65 00 74 00 ...` is `"Details"`, not CJK. Reading the
characters as big-endian produces `一愀洀攀` for `"Name"`, which is how this was caught.
The public notes warn that endianness is mixed; this is the concrete instance.

### `key` is 16 bytes, measured

`[MEASURED]` Grid-testing the two suspect types over all 7,030 resources:

| `key` size | Exact |
|---|---|
| 12 bytes | 458 (6.51%) |
| **16 bytes** | **1,612 (22.93%)** |

So a `key` occupies 16 bytes, not the 12 that three u32 would suggest. Whether the fourth
u32 is padding or a real field is `[UNKNOWN]`. The array case makes almost no difference
(1,612 versus 1,610), so the extra word is a single-value concern.

The `texts` header size (8, 12 or 16) and the `key` array size do not move the rate.

### Blockers

`[MEASURED]` With everything above applied, the failures are still dominated by:

| Preceding type | Failures |
|---|---|
| `key` (`0x20`) | 1,220 |
| `type 0x00` | 2,000+ |

`[INFERRED]` Since `key` is now correctly sized and still precedes the largest group of
failures, the error is probably in how a `key` **value** is followed, or in a type that
appears immediately after a key. `type 0x00` is not in the documented type list and the
bytes around it are often zeros, which suggests the walk has run past the real end of the
records rather than that `0x00` is a real type.

`[INFERRED]` The remaining work is a search over a small space, and the exact-consumption
rate is the metric. It was not completed here.

### The earlier failed attempt, and why it is kept

### The earlier failed attempt, and why it is kept

`tools/prop_names.py` scanned for length prefixes as **little-endian** and produced
truncated fragments such as `Def` for `Defense`. The documentation explains it: the
fields are big-endian. The tool is retained because its failure is the evidence that
identified the error.

## Note on method

This file exists because the alternative was to ship `tools/prop_names.py` as though it
worked. The tool is kept, because its failure mode is the evidence that the hypothesis is
wrong, and because it will be the validation harness once the structure is known. But its
output is not a result.

That is the same pattern as the corrections already recorded elsewhere in this project: the
bit-31 compression flag in `docs/dbpf-format.md` section 4.2, and the shared-code claim in
`re/binary-analysis.md` section 6. Measuring beat guessing in both cases, and it did here
too.

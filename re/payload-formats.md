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

### What is established

`[MEASURED]` Prop payloads contain **length-prefixed UTF-16LE strings**. Several were
read cleanly, where the length word and the character count agree exactly:

| Bytes | Length word | String | Character count |
|---|---|---|---|
| `00 00 00 07` | 7 | `Details` | 7 |
| `00 00 00 0a` | 10 | `Domes_gear` | 10 |
| `00 00 00 16` | 22 | `Group_mkmidlegSkinny_0` | 22 |
| `00 00 00 03` | 3 | `EMP` | 3 |
| `00 00 00 07` | 7 | `I-Field` | 7 |

`[MEASURED]` In several props the leading region reads as: a u32 count, then per record a
u32, a u32 that is constant within the file (`0x0D001300` in every sample), a u32 name
length, and then the UTF-16LE name. Two adjacent constants, `0x00B2CCCA` and `0x00B2CCCB`,
recur as the leading word of records in different files.

`[MEASURED]` Many prop resources are 4 bytes and contain a single zero u32. Whatever the
structure is, it has a trivially empty form.

### Why it is still open

`[MEASURED]` The value encoding that follows each name is unknown, so a reader cannot
advance from one record to the next. Without that, the only available strategy is to scan
for the length-prefix pattern, and the scan is demonstrably unreliable.

The failure is measurable and worth recording, because it is what exposed the problem.
`tools/prop_names.py` scans for the pattern and, with no identifier filter, returns mostly
localisation text (`Welcome (PLACEHOLDER)`, `Your creature is an herbivore!`). Adding a
filter for identifier-shaped names removes the prose but then returns **truncated
fragments**: `Def`, `Too`, `Milita`, `Soci`, `Cuten`, `Influ`, `Conv`. Those are prefixes of
`Defense`, `Tool`, `Military`, `Social`, `Cute`, `Influence`, `Conversation`.

`[INFERRED]` A scan that returns a word prefix is landing mid-string and reading a byte
that happens to look like a short length. That means the assumed record header is wrong, so
the pattern is being matched at wrong offsets. The extracted "vocabulary" is therefore not
trustworthy and is not presented as a result.

`[MEASURED]` Coverage is also poor: only 25 of 11,953 prop resources across three packages
yielded any identifier-shaped name at all.

### What would settle it

1. A prop with uniform record types, where the value stride can be read off by inspection.
2. Decompilation of the prop reader, which needs a working analysis toolchain. Ghidra could
   not finish a 20 MB binary inside its deadline.
3. Public documentation of the prop format. `docs/prior-art.md` section 4.1 records what
   public sources say, which is a description of intent rather than a byte layout.

Until one of those lands, the prop format stays an open item, and the honest statement is
that the payload contains readable strings but its structure is not known.

## Note on method

This file exists because the alternative was to ship `tools/prop_names.py` as though it
worked. The tool is kept, because its failure mode is the evidence that the hypothesis is
wrong, and because it will be the validation harness once the structure is known. But its
output is not a result.

That is the same pattern as the corrections already recorded elsewhere in this project: the
bit-31 compression flag in `docs/dbpf-format.md` section 4.2, and the shared-code claim in
`re/binary-analysis.md` section 6. Measuring beat guessing in both cases, and it did here
too.

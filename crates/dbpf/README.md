# `dbpf`

Reader for the DBPF ("Database Packed File") container format used by SPORE's
`.package` files, plus the RefPack codec those packages use for compressed payloads.

This crate is written against real shipped packages. It makes no assumptions about parts
of the format that have not been measured: where the layout is unknown, the code returns an
explicit error instead of guessing. Every layout decision is documented with its evidence
in `docs/dbpf-format.md`.

## Implemented

- Opening a package, parsing its header, and reading its resource index
  (`DbpfFile::open`, `DbpfHeader`, `DbpfFile::entries`).
- Payload extraction (`DbpfFile::resource_data`), which returns raw bytes or decompresses
  them with the bundled RefPack implementation (`refpack::decompress`).
- The `ResourceKey` triple (`type_id`, `group_id`, `instance_id`) with a `Display`
  implementation formatted as `TYPE:GROUP:INSTANCE` in uppercase hex
  (`00001234:0000ABCD:0000000000000001`).

## Validation status

The RefPack codec was validated against real game data: 800 coded resources across two
packages each decompressed to exactly the size declared by the DBPF index, using only the
size declared inside the stream. See `tools/validate_refpack.py`.

**This crate has never been compiled.** No C linker is installed on the development
machine, so `cargo build` cannot link. The Rust here is a direct port of working Python
(`tools/refpack.py`) and its unit tests are written but unrun. Treat it as unverified until
a toolchain with MSVC Build Tools is available.

## Header layout

The header is 96 bytes (`0x60`). Offsets are from the start of the file and all fields are
little endian.

| Offset | Size | Field | Status |
| --- | --- | --- | --- |
| `0x00` | 4 | `magic` | Measured. Always ASCII `DBPF`. |
| `0x04` | 4 | `version_0x04` | Measured as `3` everywhere. The name is inherited convention; public sources disagree, see `docs/dbpf-format.md` section 2.1. |
| `0x08` | 4 | `version_0x08` | Measured as `0` everywhere. |
| `0x24` | 4 | `index_entry_count` | Measured. Confirmed structurally against the parsed index. |
| `0x28` | 4 | `unknown_0x28` | Unidentified. Zero in every package inspected. |
| `0x2c` | 4 | `index_size` | Measured. `index_offset + index_size == file size` on three packages. |
| `0x3c` | 4 | `unknown_0x3c` | Unidentified. `3` in every package inspected. |
| `0x40` | 4 | `index_offset` | Measured. See `index_size`. |

## Index layout

```
u32 prefix            (4 bytes, measured as 4 everywhere)
N x 28-byte records   (N = index_entry_count)
u32 trailer           (4 bytes, 0x0001FFFF or 0x00010000)
```

Each 28-byte record is seven little-endian u32 fields, in order: `group_id`, `type_id`,
`instance_hi`, `instance_lo`, `offset`, `stored_size`, `decompressed_size`. The resource
key is `(type_id, group_id, (instance_hi << 32) | instance_lo)`.

Bit 31 of `stored_size` is set on 100% of records in every package that parses, so it
carries no per-resource information and is masked off. The raw-versus-coded rule is
`stored_size == decompressed_size` means raw.

## Not implemented

- The index layout for three packages. `Spore_Audio1.package`,
  `Spore_EP1_Locale_01.package` and `Spore_EP1_Locale_02.package` do not fit the 28-byte
  record and surface as `DbpfError::UnsupportedIndexLayout`. See `docs/dbpf-format.md`
  section 6. The reader refuses to guess rather than silently mis-parsing them.
- Decoders for the payload types themselves (prop, RW4, gmsh, anim). Decompression is
  implemented; interpreting the decompressed bytes is not.

## Reference measurements

The real files these values were read from. They are not in the repository and are not
required by the test suite.

| File | Size (bytes) | `index_entry_count` |
| --- | --- | --- |
| `Data/Spore_Game.package` | 20,210,108 | 11,529 |
| `Data/PatchData.package` | 80,428,851 | 25,535 |

## Tests

`cargo test -p dbpf` builds synthetic packages in memory, including one with a full valid
index, so the tests do not depend on any game file being present.

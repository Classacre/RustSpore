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

The record shape is **not** constant across retail packages. Three variants were measured
in one install, and all 13 packages fit one of them exactly:

| Packages | Prefix | Record | Trailer | Record fields |
| --- | --- | --- | --- | --- |
| 10 of 13 | 4 | 28 | 4 | `group, type, instance_hi, instance_lo, offset, stored_size, decompressed_size` |
| `Spore_Audio1.package` | 12 | 24 | 0 | `type, instance, offset, stored_size, decompressed_size, group` |
| both EP1 locale packages | 16 | 20 | 0 | `hash, offset, stored_size, decompressed_size, group` |

A measured invariant holds across all 13: `prefix + record_size + trailer == 36`.

`index::detect` chooses the shape using two constraints. First, exact fit:
`prefix + count * record_size + trailer == index_size`. That narrows the candidates but
does not decide between them, because for one real package two shapes both fit. Second,
contiguity: payloads are packed, so when records are sorted by offset each resource should
end where the next begins. The best-scoring shape wins. On the real packages the correct
shape scores 100% and the wrong one far lower.

Bit 31 of `stored_size` is set on 100% of records in every package, so it carries no
per-resource information and is masked off. The raw-versus-coded rule is
`stored_size == decompressed_size` means raw.

## Validation status

Two independent checks pass against real data.

- **Codec**: 800 coded resources across two packages decompressed to exactly the size
  declared by the DBPF index, using only the size declared inside the stream.
- **Whole pipeline**: 13,174 PNG resources decoded across three packages, and every chunk
  CRC of every image verified. 100.0% valid, 214 MB decoded. A CRC match cannot be
  satisfied by producing the right number of wrong bytes, so this validates the header
  parse, index location, record field order, raw-versus-coded rule and the codec
  byte-exactly.

**This crate has never been compiled.** No C linker is installed on the development
machine, so `cargo build` cannot link. The Rust here is a direct port of working Python
(`tools/refpack.py`, `tools/dbpf_variants.py`) and its unit tests are written but unrun.
Treat it as unverified until a toolchain with MSVC Build Tools is available.

## Not implemented

- Decoders for the payload types themselves (prop, RW4, gmsh, anim). Decompression is
  implemented; interpreting the decompressed bytes is not. See `docs/roadmap.md` phase 1
  and `docs/engine-architecture.md` for the leads on RW4 and Cal3D.

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

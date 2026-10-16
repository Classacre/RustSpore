# RustSpore

A clean-room reimplementation of the 2008 EA/Maxis game *SPORE*, written from scratch in
Rust on top of the [Bevy](https://bevyengine.org) engine.

RustSpore contains no EA code, no EA assets, and no EA data. It reads game data that you
supply yourself from a legitimate installation of the original game. Nothing proprietary
is redistributed by this repository.

## Status

The container format is reverse engineered and the compression codec is validated against
real game data. The game itself is not started; see `docs/roadmap.md` for scope and stages.

| Crate | Purpose | State |
| --- | --- | --- |
| `crates/dbpf` | DBPF `.package` container: header, resource index, payload extraction, RefPack codec | implemented, **never compiled** (see below) |
| `crates/spore-core` | Engine-agnostic core types (game stages, resource ids) | scaffold |
| `crates/spore-bevy` | The `rustspore` binary: opens a package, reports its contents, runs a Bevy scene | scaffold |

Verified against the shipped game data:

- The DBPF header, index block and 28-byte resource record, established by measurement.
  `index_offset + index_size == file size` holds exactly on three packages, and a
  structural search found 11,529 consecutive valid records matching the header count at a
  ratio of 1.0000.
- **136,081 resource records** parse across 10 of 13 archives. The three that do not are
  recorded as an open gap rather than papered over.
- The RefPack codec, validated on **800 coded resources across two packages**, each
  decompressing to exactly the size declared by the DBPF index, using only the size
  declared inside the stream. Two independent sources of truth agree.

## Building

```sh
cargo build --workspace
cargo test -p dbpf
cargo run -p spore-bevy
# Point it at a package to dump its header, index and resource type histogram:
cargo run -p spore-bevy -- "C:/path/to/Data/Spore_Game.package"
```

### Known blocker: no C linker on this machine

The workspace **has not been compiled**. There is no Visual Studio or Build Tools
installation, so MSVC's `vcruntime.lib` and `msvcrt.lib` are absent, and `link.exe` on
`PATH` resolves to PortableGit's GNU `link`, which is a different program. Rust's bundled
`rust-lld` combined with the Windows SDK import libraries gets as far as the MSVC CRT
symbols, where it fails on `_CxxThrowException`, `__CxxFrameHandler3`, `__chkstk`,
`_tls_index`, `_tls_used`, `memcpy`, `memset` and others.

Installing Visual Studio Build Tools with the "Desktop development with C++" workload is
the unblocking step. Until then the Rust code here should be treated as unverified.

## Documentation

Read these before changing a parser.

| Document | Contents |
| --- | --- |
| `docs/dbpf-format.md` | The container format, every claim graded `[MEASURED]` / `[INFERRED]` / `[UNKNOWN]` |
| `re/binary-analysis.md` | Executable analysis: sections, imports, Rich header, mitigations, and its limits |
| `docs/roadmap.md` | Honest scope in measured numbers, staged plan, legal position |
| `docs/prior-art.md` | What public sources say about these formats, and where they conflict |
| `docs/resource-types.md` | Resource type IDs with sources and confidence |

Analysis scripts that produced the above live in `tools/`. They are reproducible and are
the evidence behind the format documents.

## Game data

Extracted game data is expected under `assets/extracted/`, which is gitignored, along with
any `*.package` file. Supply your own.

## Licence

Dual licensed under MIT or Apache-2.0, at your option.

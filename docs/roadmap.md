# RustSpore roadmap and scope

## 1. What this project is

A from-scratch reimplementation of SPORE (Maxis/EA, 2008) in Rust on Bevy. The
reimplementation reads assets from a copy of the game the user owns. It does not ship any
EA asset, and the repository excludes `*.package` files and anything under
`assets/extracted/`.

The first milestone is not "a game". It is a toolchain that can read the shipped data
correctly, because without that there is nothing to build a game on. That milestone is
now partly met.

## 2. Scope, in measured numbers

Worth stating plainly, because the numbers set the shape of the work.

| Quantity | Value | Source |
|---|---|---|
| Asset archives in this install | 13 | `docs/dbpf-format.md` |
| Total archive size | about 4.9 GB | file sizes |
| Resource records in the 10 parsable archives | **136,081** | `tools/dbpf_fields.py` |
| Archives whose index layout is still unknown | 3 | `docs/dbpf-format.md` section 6 |
| Engine executable size | 20,447,744 bytes (base), 24,888,320 (EP1) | `re/binary-analysis.md` |
| Executable code section | 16,560,128 bytes | section table |
| Imported DLLs / imported functions | 20 / 495 | import table |
| Game stages to implement | 5 | Cell, Creature, Tribe, Civilization, Space |
| Distinct resource types seen so far | 6 dominant, dozens total | type histograms |

For calibration: a full SPORE recreation is a many-person, multi-year effort. The
executable alone is 16.5 MB of compiled code with a custom engine, a procedural creature
system, a galaxy-scale simulation and five distinct gameplay modes. Any plan that implies
otherwise is not honest.

What is achievable is a sequence of independently useful milestones, each of which can be
verified against the real game data.

## 3. Current state

### Done and verified

- **DBPF container reader, complete.** Header fields at 0x24, 0x2C and 0x40 established
  by measurement, with `index_offset + index_size == file size` confirmed exactly on
  three packages. **All 13 of 13 packages now parse**, using three index record variants
  that differ in record size and field order. A measured invariant holds across every
  package: `prefix + record_size + trailer == 36`. The three packages that were
  previously unparsed are now resolved (see the item below).
- **Every package accounted for.** The 3 packages whose index could not be located
  (`Spore_Audio1`, `Spore_EP1_Locale_01`, `Spore_EP1_Locale_02`) are solved. Record shape
  is chosen by exact fit plus storage contiguity, and the decisive evidence is
  `Spore_Audio1.package` chaining 17,650 of 17,650 consecutive payloads with zero gaps.
  `docs/dbpf-format.md` section 6.
- **Engine architecture mapped.** RTTI is absent, so the class names were recovered from
  string literals instead: 344 simulation classes, 340 ArgScript names, 332 console
  commands, and a full middleware inventory including RenderWare 4, Havok 3.1.0, Cal3D,
  RenderWare Audio, EAPd and EASTL. `docs/engine-architecture.md`.
- **RefPack codec.** Validated on **800 coded resources across two packages**, each
  decompressing to exactly the size declared by the DBPF index, using only the size
  declared inside the stream. Two independent sources of truth agree. Python reference in
  `tools/refpack.py`, Rust port in `crates/dbpf/src/refpack.rs`.
- **Raw versus coded rule.** `stored_size == decompressed_size` means raw. Measured split
  in `Spore_Game.package`: 2,215 raw, 9,314 coded.
- **Binary fingerprinting.** Sections, imports, Rich header, mitigations, and the
  corrected finding that the two executables' `.text` sections are 93.96% different
  despite near-identical sizes.
- **Rust workspace.** Three crates: `dbpf`, `spore-core`, `spore-bevy`.

### Blocked

**The Rust workspace cannot be compiled or tested on this machine.** There is no C
linker installed:

- `link.exe` on `PATH` resolves to PortableGit's GNU `link`, which is a different program.
- No Visual Studio or Build Tools installation exists, so MSVC's `vcruntime.lib` and
  `msvcrt.lib` are absent.
- Rust's bundled `rust-lld` gets much further (it is present, and the Windows SDK import
  libraries at `Windows Kits\10\Lib\10.0.28000.0` supply `kernel32.lib` and `ucrt.lib`),
  but linking still fails on MSVC CRT symbols that only ship with MSVC:
  `_CxxThrowException`, `__CxxFrameHandler3`, `__chkstk`, `_tls_index`, `_tls_used`,
  `memcpy`, `memset` and friends.

**Remediation:** install Visual Studio Build Tools with the "Desktop development with C++"
workload. That is the single unblocking step. Until then, the Rust code in this repository
has never been compiled, and should be treated as unverified.

**Also blocked: deep binary analysis.** The reverse-engineering MCP backend runs Ghidra
headless with a 330-second startup deadline, which auto-analysis of a 20 MB MSVC binary
exceeded on two attempts. No decompilation, call graph or RTTI recovery was obtained. RTTI
recovery is the highest-value missing item, because MSVC class-name strings would expose
the engine's internal architecture directly.

## 4. Staged plan

Each phase is independently useful and verifiable against real data.

### Phase 1: payload decoders

The container is solved; the payloads are not. Dominant types seen so far, with what
stands between each and being readable:

| Type | Name | Work required |
|---|---|---|
| 0x2F7D0004 | PNG | none, standard format, already recognisable after decompression |
| 0x00B1B104 | prop | decode the property tree; public sources describe it, needs measurement |
| 0x2F4E681B | RW4 | RenderWare 4 asset container; the largest single job in this phase |
| 0x01C135DA | gmsh | mesh format: vertex buffers, index buffers, submesh splits |
| 0xEE17C6AD | anim | animation curves and skeletal binding |
| 0x5CAA5E28 | blockdata | unknown structure |

Deliverable: a `spore-formats` crate with a decoder per type, each with a round-trip test
against a real resource.

Two of these are now much better specified than they were, see
`docs/engine-architecture.md`:

- **RW4 (`0x2F4E681B`)** is not a mystery container. The binaries name RenderWare's own
  object type constants (`RWGOBJECTTYPE_SHADER`, `RWCOBJECTTYPE_CLUSTEREDMESH` and so on),
  so the decoding strategy is to locate the RW object type id in the payload and dispatch
  on it, rather than guessing at a byte layout.
- **Animation (`0xEE17C6AD`)** is bound to Cal3D, a third-party character animation
  library, evidenced by the `Cal3DBoneName` and `Cal3DOrigin` script parameters. That
  turns an open-ended problem into a known format with public documentation.

`[MEASURED]` Note also that `Spore_Content.package` carries two adjacent RW4 type ids,
`0x2F4E681B` (1,131 resources) and `0x2F4E681C` (2,954). The pair needs explaining before
either decoder is written.

### Phase 2: asset browser in Bevy

A Bevy app that opens a package, walks the index, decodes textures and meshes, and renders
them. This is the first visibly useful milestone and it validates phases 0 and 1 end to
end.

### Phase 3: creature data

The creature editor's data model. This is SPORE's signature feature and the hardest single
subsystem: procedural bodies built from parts, with a rig generated from the part layout.

### Phase 4: one gameplay stage

Cell is the smallest stage and the right target: a 2D plane, eat and grow, a handful of
part types. Shipping one complete stage proves the engine loop, the input path, the
renderer and the asset pipeline together.

### Phase 5 onward

Creature stage, Tribe, Civilization, Space, then the editors. Space is effectively a
second project: galaxy generation, star systems, planets, terraforming, missions and the
trading economy.

## 5. Legal and licensing position

- **Assets.** Everything in the `.package` archives is EA's copyright. This project reads
  them; it must never redistribute them. `.gitignore` already excludes `*.package` and
  `assets/extracted/`.
- **Third-party tool code.** `docs/prior-art.md` section 6.2 catalogues the licences. The
  short version: SporeModder FX and S3PI are GPL-3.0 and cannot be copied into a
  permissively licensed project. Several tools have no licence at all, which defaults to
  all rights reserved. `chieltbest/dbpf` is a Rust DBPF implementation and therefore the
  most tempting to reuse, but its licence is an unresolved multi-licence `REUSE` setup and
  must be checked before any copy.
- **What this project did instead.** Every layout in `docs/dbpf-format.md` was derived by
  measuring the shipped data. Public sources were consulted to cross-check and to flag
  conflicts, which is why the version-field conflict in section 2.1 is recorded rather
  than smoothed over. This keeps the format work clean-room.
- **This is not legal advice.** Reverse engineering for interoperability and modding has a
  substantial community history around SPORE and the wider EA back catalogue, but the
  position varies by jurisdiction and no lawyer has reviewed this project.

## 6. How to work on this

```bash
# Read the evidence before changing a parser.
docs/dbpf-format.md      # container format, evidence graded
re/binary-analysis.md    # executable analysis and its limits
docs/prior-art.md        # what public sources say, and where they conflict
docs/resource-types.md   # resource type IDs with sources

# Re-run the analysis that produced the format.
PY=C:/Users/nieuw/.workbuddy-ai/binaries/python/envs/default/Scripts/python.exe
$PY tools/dbpf_fields.py
$PY tools/validate_refpack.py "<a real package>" 20
```

Rules that this project has already found it needs:

1. **Grade every claim.** `[MEASURED]`, `[INFERRED]`, `[UNKNOWN]`. Several conclusions in
   this repository were corrected by measurement after being stated too confidently,
   including the compression flag in `docs/dbpf-format.md` section 4.2 and the shared-code
   claim in `re/binary-analysis.md` section 6.
2. **Report the denominator.** "800 resources validated" only means something alongside
   "sampled from 9,314 coded resources in one of 13 packages".
3. **A gap is a result.** The three unparsed packages and the missing RTTI are recorded as
   gaps, not omitted.
4. **Do not assume one global layout.** `Spore_Audio2.package` fits the 28-byte record and
   `Spore_Audio1.package` does not, in the same install.

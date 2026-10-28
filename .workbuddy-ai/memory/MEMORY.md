# RustSpore: durable project notes

Latest continuation: `docs/continuation-2026-10-27.md` and the dated
`2026-10-27.md` project log. The user resumed work on2026-10-27 and requested
GPT-6.1 Sol/lower reasoning for delegated work. The full original-fidelity
roadmap is incomplete; the prior pause handback remains historical context.
Main Cell and authored creature captures now execute original programs; the latest
full workspace all-targets suite V21 passes611 test executions (599 primary unit tests,
twelve repeated example tests) with two explicit installed-corpus ignores. All704
explicit-linked creations now pass hidden RTX original-program captures. Native UI
batching, ANIM descriptor preparation and typed Cell definitions/branches now
have independent full-corpus or GPU checks; final game/editor gates stay open.
Optional foot construction now matches1,212 native/Python/Rust projections and
8,190 fresh source digests. Cell runtime reader/writers pass129,331 native/Rust
cases including4,012 positive finite-API errors. Simple font phantom sets and
selector-0/1 scaling pass independent source-instruction/Rust comparisons.
Full Cell curves pass84,544 native cases (288 positive input errors); data2
movement multiplier passes37,669 (4,552 positive finite-API errors).
Finalizer fields pass4,590 native/Python/Rust replies, seven tests and all4,095
fresh-source reconstruction contexts. FPGM setup66,626 rows and CALL/LOOPCALL
65,699 rows now pass Rust/reference checks with five focused tests each.
Scalar contributions now match52,911 native/Rust cases; ordinary root binding
matches15,113 native/Python/Rust contexts. A cohesive25-case/five-channel path
matches sampling, count-only reset, accumulation and modifier read with supplied
frames/bindings/actor context. Font0 PREP913 rows and glyph0 hinting249 rows match
source/Rust; retained native scalar state agrees. Glyph0 source raster matches
all49 native pixels and Rust raster now matches1,389 source/root/Rust rows.
Fourvisiblefont0characters also match1,357 source/root/Rust rows and16 retained
nativeobservations. Clock1,560 native/Rust and40case/fivechannel composition
through96freshcurves now pass. FreshrootROT matches3,702 rows/215clips,two
excluded. Source-onlysimplefont0census389glyphs/1,556nativeobservations matches;
generic font coverage is now recorded below. E/F/H/L/T bounded Rust matches2,573 rows/20
retained native observations. Root conversion/accumulation now matches22,212
native/Rust source-paired cases,42 protocols and14 further Rust/native protocol
cases. Sparse carrier v3 fixes earlier pre-input mappings and a retained v2
WinError193 PE adjacency failure, with original bytes unchanged. Loop arithmetic
matches2,256 native/Rust cases,15 protocols and21 typed errors under explicit
state/CRT policy. Zero-child staging/final commit now matches66,636 native/Rust
cases and576 owned alternate/threshold contexts. Actual source metadata now
drives32 clock/sampler/scalar cases across four admitted channels; constructor
metadata independently matches2,304 cases. D/P/U adds2,291 corrected source/Rust
rows/12 retained native observations; two inherited diagnostic axes were fixed
from original frame/callback proof without numeric Rust changes. Controller
setter183B passes536 native/Rust cases,14 protocols and15 typed guards with34
unchanged pins. FreshAid1049B now passes288 native/Rust cases,11 protocols and12
typed guards with59 unchanged pins through the new sparse carrier; original
pre-input mapping failure stays retained. Generic bounded font0 Rust coverage
now passes360/389 simple glyphs,164,948 numeric rows/1,440 retained native observations,
plus326 owned contour rows. Correct original-prefix replay preserves all338 numeric
groups; accepted general-vector/intersection successors now compare complete
frame vectors and callbacks.29 simple/183
composite/six empty glyphs retain exclusions. Original rotated-box publication
passes15,864 native/Rust rows; combined production bounds/nonroot selection passes
161,284 queries over1,322 genuine bodies/all1,644 retained source instances,
73 protocols and119 unchanged native pins. Full ungrouped constructor now passes
1,085 prepared cases/27,142 records including160 ownership fallbacks;57 protocols,
216 native pins and9,765 typed Rust guard executions pass. Actual actor lifetime,
allocation and nonzero-child pose admission stay open. Activation hybrid passes300 records/2,700 snapshots,23 rejection protocols
and6,300 typed guard executions with143 unchanged native pins; source channel
counts remain actual and all four sample/event masks are zero. Nonzero sampling
and events remain open. Integer vector primitives pass2,135 source/root/Rust rows;
the07 and0F VM paths now have actual full-program checks. F20 and full-program87
remain open. Actual rig/live
binding, wider hint/raster and all stage/editor gates remain open. Current
2026-10-27 checkpoint supersedes the old pause handback and records retained
failures, bounded acceptance and next steps.

## Conventions

**Grade every claim in analysis docs.** Use `[MEASURED]` (observed in bytes),
`[INFERRED]` (reasoned, state from what), `[UNKNOWN]`. Never present a guess as a
measurement. This convention earned its place immediately: two early hypotheses
(the bit-31 compression flag, the shared-code claim) were stated too confidently and
then falsified by measurement.

**Report the denominator.** "800 resources validated" is meaningless without "sampled
from 9,314 coded resources in one of 13 packages".

**Record gaps as results.** The three unparsed packages and the missing RTTI are
documented, not omitted.

**Use the canonical flag-driven index for identity.** Original reader8DAFE0 proves
native instance/type/scope keys and current-record compression metadata. Earlier
size-fit/contiguity work preserved payload spans but assigned incorrect group/trailer
labels. All82 packages /158,720 keys and every payload now agree independently;
the native renderer, AssetSource, PackageSet and SourceCellLibrary use this reader.
`RecordShape`/`ResourceKey` and older CLI/corpus callers are compatibility views.

**Exact-fit arithmetic is not sufficient to pick a layout.** For
`Spore_EP1_Locale_02` two shapes both satisfy the exact-fit equation. The arbiter is
storage contiguity (`offset_i + size_i == offset_{i+1}`), because payloads are packed.
Use a control package whose layout is already known.

**RTTI is absent from the SPORE binaries.** Do not plan work around recovering class
names from TypeDescriptors; there are only 2 in the whole 20 MB binary. Engine class
names live in string literals instead, and `tools/pe_engine_map.py` extracts them.

**Never claim a build or test passed without running it.** This note used to say the
Rust workspace had never compiled. When it was finally run, the build was green and two
real `dbpf` bugs surfaced at once (a wrong end-of-stream test and a decoder that accepted
short output). A brace-balance check is not a substitute for cargo, and neither is a build
that passes while the tests have never executed.

**A render is not a measurement.** I once compared an assembled creature against the
game's merged model by diffing two screenshots and got IoU 0.178, then wrote up "a per-part
scale is missing". The viewer normalises each mesh by its own radius, so the pixel metric
was measuring framing. Comparing vertices directly gave a median 2.1% nearest-neighbour
distance: the shapes matched. Any comparison that depends on how something was displayed is
not a measurement of the thing.

**Cross-check with digests, not with "it decoded".** Three separate wrong readings in this
project produced plausible output: a RefPack decoder that accepted short output, an RW4
`BaseResource` offset used as absolute (still yields valid-looking block data), and a DXT3
decoder that swapped endpoints. Each was caught only by comparing a per-resource FNV digest
against an independent implementation. When two implementations disagree, one of them is
wrong; a diff is the cheapest arbiter there is.

**A field kept "for information" is untested.** The creaturedata RLE decompressor was wrong
from the start: it emitted the chunk's first byte for every literal and never advanced, so
the buffer was one byte repeated. Nothing read it, so the format docs said "100% decoded"
while it was garbage. It surfaced only when the paint palette was read out of it. Either
read a field somewhere or delete it.

**Shell loops over generated lists must strip CR.** A Python heredoc on Windows writes
`\r\n`, and `u64::from_str_radix("406262008933CC09\r", 16)` fails, so the binary fell back
to instance 0 and a 366-creature survey reported "all 366 have no mesh". It was an artifact
of the loop. Pipe lists through `tr -d '\r'`, and grep the output for
`no creature 0000000000000000` before believing a zero.

**Parsed is not modelled, and the docs must say which.** A parser that consumes the right
bytes has not necessarily produced usable data. Track the difference explicitly: gmdl was
"100% exact" while its vertex data was still unread, and gmsh attribute sections were
opaque blobs long after the mesh "parsed". Say "consumes exactly" when that is all that is
known.

## Layout

- `docs/` user-facing analysis: `dbpf-format.md` (the container spec), `roadmap.md`
  (scope), `prior-art.md` (public sources and their conflicts), `resource-types.md`.
- `re/` reverse-engineering output: `binary-analysis.md` plus raw dumps in `re/data/`.
- `tools/` the Python analysis scripts that produced the above. They are the evidence
  behind the format docs, so keep them working and keep them in sync with the docs.
- `crates/` Rust workspace: `dbpf` (container + RefPack + `PackageIndex`, the index-only
  reader used for cross-package lookup), `spore-core` (parsers, `mesh.rs` conversion to
  renderable geometry, `skin.rs` which paints a creature's UVs from its palette),
  `spore-bevy` (`assets.rs` registry with prop resolution and `PackageSet`, `viewer.rs`
  Bevy app).

## Hard constraints

- Never redistribute EA assets. `.gitignore` excludes `*.package` and
  `assets/extracted/`. The project reads user-supplied data only.
- GPL-3.0 modding tools (SporeModder FX, S3PI) must not be copied into this
  permissively licensed project. `chieltbest/dbpf` has an unresolved REUSE licence.
- Do not use em dashes or en dashes in prose.

## Environment

The workspace builds and its tests pass. Visual Studio 18 Community with MSVC 14.51.36231
and Windows SDK 10.0.26100 / 10.0.28000 are installed. Always build through
`tools\build.bat`, never bare `cargo`: it locates the toolset with `vswhere`, sets `PATH`,
`LIB` and `INCLUDE`, and avoids `vcvars64.bat` (which needs `reg.exe`). Cold build about
3 minutes; the latest full `test --workspace` run is 514 passed with two corpus ignores.

`rustspore --verify <package>` is the real-data test for the codec: it decompresses every
coded resource and compares against the index's declared size, exiting non-zero on any
failure. Current result across the install: 82 of 82 packages open, 101,312 of 101,312
coded resources exact. Use `--report` for the header dump without a window.

The persistent REA 6.5.0 SDK session now runs Ghidra 12.1.4 successfully.
Its focused profile skips Stack, x86 Constant Reference Analyzer and Decompiler
Switch Analysis; their missing facts remain unknown. Preserve the live helper
and accumulated Evidence rather than repeating setup or expensive auto-analysis.

**Bevy 0.18 API traps** (cost an hour to find): `Indices` lives in `bevy::mesh`, not
`bevy::render::mesh` (that module is public but its contents are not re-exported);
`PrimitiveTopology` is in `bevy::render::render_resource`; `RenderAssetUsages` is in
`bevy::asset`; `AmbientLight` is a component that requires a camera, not a resource. And
`spore_core::Vec3` is not `bevy::prelude::Vec3`.

**Format facts that cost time and must not be rediscovered:**
- A prop `key`'s three words are little-endian in an otherwise big-endian format, and they
  are `(instance low, type, instance high)`, not `(group, type, instance)`.
- An RW4 `BaseResource` offset is relative to `p_buffer_data`; every other object's is
  absolute.
- DXT3 always uses four-colour mode, but the endpoints must keep their order. Swapping them
  to satisfy `c0 > c1` shifts every colour.
- Use each material's recovered sampler state, not a global UV heuristic.
  Source `20D` records conditionally update address/filter state; Jawhead and
  Stabber explicitly use clamp/linear. See `re/material-links.md`.
- Some property names can be cross-checked through executable literals and
  public interface registries. Do not guess names for other hashes; measure
  their actual readers and roles.
- A key whose type is `0x00000000` means "unspecified": match the instance alone, or it will
  not resolve.
- `PatchData.package` is a patch overlay: every prop in `Spore_Content` has a copy there, so
  resource lookup must decide which copy wins rather than assuming the first found.
- A `.creaturedata` rig row is 0x8C bytes: rotation 3x3 at 48, translation at 84, part prop
  instance split high-at-132 / low-at-136. Transforms are absolute, not hierarchical.
- A creature's skin is per creature, not per part, and no prop names it: it is a RASTER
  whose instance shares the creature's **low** half and differs in the high half. 366 of
  1,022 creatures in `Spore_Content` ship one; the rest need the paint system. Do not expect
  texture inheritance to close that gap.
- A creature part's own RW4 container holds no texture. The skin atlas is the only one.
- A creature's paint palette is nine floats at offset 32 of the decompressed header buffer:
  three RGB triples. 656 of 1,022 creatures have no skin and only this.

**A creature's palette is a colour set, not a flat palette.** The nine floats at offset 32
give the three authored colours, but the shipped skins are hand-painted: over 40 creatures
with both, only a median 14% of a skin's opaque pixels lie within 0.08 RGB of a palette
colour. `spore_core::skin` paints the back, belly and a marking mask from those three
colours; it produces a base colouring, not a reconstruction, and the docs say so. Its
rasteriser is validated by containment: the shipped alpha mask lies inside the generated
one, IoU equalling the shipped area fraction to two decimals on 5 of 5 creatures.

**Verify rendering headlessly.** `rustspore --shot <package> <out.png> [index]` renders a
few frames and writes a PNG, so the pipeline can be checked from a shell. Combined with
`--mesh-selftest`, `--raster-digest` and `--gmsh-digest`, every layer has a real-data check
that does not need eyes.

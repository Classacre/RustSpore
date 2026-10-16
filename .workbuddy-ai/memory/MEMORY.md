# RustSpore: durable project notes

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

**Do not assume one global layout.** All 13 packages satisfy
`prefix + record_size + trailer == 36`, but the record size varies: 28, 24 and 20 bytes,
with different field orders. `Spore_Audio1` and `Spore_Audio2` differ, in one install.

**Exact-fit arithmetic is not sufficient to pick a layout.** For
`Spore_EP1_Locale_02` two shapes both satisfy the exact-fit equation. The arbiter is
storage contiguity (`offset_i + size_i == offset_{i+1}`), because payloads are packed.
Use a control package whose layout is already known.

**RTTI is absent from the SPORE binaries.** Do not plan work around recovering class
names from TypeDescriptors; there are only 2 in the whole 20 MB binary. Engine class
names live in string literals instead, and `tools/pe_engine_map.py` extracts them.

**Never claim a build or test passed without running it.** The Rust workspace here has
never compiled, and the README says so. A brace-balance check caught one real error, but
it is not a substitute for cargo.

## Layout

- `docs/` user-facing analysis: `dbpf-format.md` (the container spec), `roadmap.md`
  (scope), `prior-art.md` (public sources and their conflicts), `resource-types.md`.
- `re/` reverse-engineering output: `binary-analysis.md` plus raw dumps in `re/data/`.
- `tools/` the Python analysis scripts that produced the above. They are the evidence
  behind the format docs, so keep them working and keep them in sync with the docs.
- `crates/` Rust workspace: `dbpf` (container + RefPack), `spore-core`, `spore-bevy`.

## Hard constraints

- Never redistribute EA assets. `.gitignore` excludes `*.package` and
  `assets/extracted/`. The project reads user-supplied data only.
- GPL-3.0 modding tools (SporeModder FX, S3PI) must not be copied into this
  permissively licensed project. `chieltbest/dbpf` has an unresolved REUSE licence.
- Do not use em dashes or en dashes in prose.

## Environment

No C linker on this machine (no MSVC Build Tools). Rust cannot link. Details and the
exact failing symbols are in `docs/roadmap.md` section 3.

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

**Do not assume one global layout.** `Spore_Audio2.package` fits the 28-byte index
record and `Spore_Audio1.package` does not, in the same install.

**Never claim a build or test passed without running it.** The Rust workspace here has
never compiled, and the README says so.

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

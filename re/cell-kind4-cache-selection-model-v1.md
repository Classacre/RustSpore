# Bounded Cell kind-4 cache and selection model

`spore_core::cell_kind4_cache_selection_v1` is registered source-guided
groundwork. It requires a caller-admitted structure-handle key, stable resolved
four-row snapshot, cache, RNG state, control word and type-0 third word. It is
not wired into live Cell gameplay and does not resolve the list, prove a
post-fixup identity or select a retail package. The static instruction trace
and its boundaries are in `cell-kind4-cache-selection-static-v1.md`.

The independently reviewed source trace follows `E65640`'s cache lookup:
a hit returns the existing triple without invoking selection or directly
consuming RNG; on a miss the source obtains a slot before calling `E5C0F0`.
For the admitted four type-0 rows with weight 1.0, the latter calls `935D40`
once per row and replaces the selected index only when the weight is strictly
greater than the sampled product. Four integer totals and the corresponding
PC53/PC64 products in this bounded domain are exact. This does not generalize
to arbitrary weights, type-1 rows, zero-state RDTSC, unknown cache lifecycle or
the original runtime floating-point state.

## Model evidence

The full all-target workspace run at
`target/workspace-tests-20261030-resume-v63.log` reports **833 passed, 0
failed, 4 ignored** across 173 result summaries. Its SHA-256 is
`ec7dc9f3882f0fd2f7f6d4af0b6ade3c75722f471117c5af5de6f1205fc3f270`.
The ignored PatchData fixture described below was run separately and passed;
it is not included in that workspace run's passed count.

Eight focused Rust tests passed, including the fourth-row equality threshold,
cache-hit behavior, failure staging and cache capacity. The registered model
also matched an independent integer reference for **539 synthetic cases**.
The reference pins the original executable and selected instruction spans as
static input; it does not execute those instructions.

| Retained evidence | SHA-256 |
| --- | --- |
| `target/cell-kind4-reservoir-production-v1/input.bin` | `c36d307d7328e23f6f91ff5867ff54a73f5566738e97298b5a1102b597edb7b2` |
| `target/cell-kind4-reservoir-production-v1/rust.bin` | `2a59c5b46202cca6106c2b7ec38a77c044a4a43324e2934b13a65a6e8e0d2727` |

The corpus includes a strict equality witness for the fourth draw, threshold
neighbors, PC53/PC64 with four rounding modes, miss-then-hit cases and an
initial hit. The adapter compares the model's exposed outcome, returned
triple, final RNG state, selected index and draw count. Per-row intermediate
decisions in `expected.jsonl` are **reference-only traces**; the production
API does not expose them and they were not independently observed in Rust.
The manifest pins the production model, adapter, reference and dependencies;
the verifier does not authenticate an arbitrary reply's origin.

The comparison protocol rejects unsupported zero-state and zero-numerator
misses before calling the model, so that its reference can leave the cache
unchanged. This is an explicit safe protocol restriction: source insertion
occurs before selection. The model's `Pending` and rejection variants are
bounded policy representations; the original failure contents, recovery and
lifetime are not established.

The separately run ignored PatchData fixture also passed after checking the
canonical list key `9A391385:F9C3D770:00000000` and decoded SHA-256
`29b0f195b26bf2480586518e385933284819ff360a991ea706a939717a5d5f8a`.
It explicitly admits the parsed disk rows as the runtime snapshot for this
fixture and supplies an empty cache, synthetic structure handle `0x42`, seed 1,
control word `027F` and third word 7. Under those admissions the model selects
row index 1, consumes four draws and returns RNG state `D369BF11`. This does
not establish the actual post-fixup rows, cache, handles, seed or original
selection. Package extraction assumes no concurrent mutation or replacement;
before/after package hashes cannot exclude a transient replacement.

This comparison is model-to-model evidence. No original-game instructions,
actual runtime cache or actor were observed. The supplied row words and
handles are fixture values, not proof of runtime fixup or resource selection.

## Reproduction

From the repository root, use a fresh output directory and the matching
installed executable for the read-only source hash and span checks:

```powershell
tools\build.bat test -p spore-core cell_kind4_cache_selection_v1::tests
python -I -S tools/cell_kind4_reservoir_reference_v1.py prepare target/cell-kind4-reservoir-repro
tools\build.bat run -p spore-core --example cell_kind4_reservoir_compare_v1 -- target/cell-kind4-reservoir-repro/input.bin target/cell-kind4-reservoir-repro/rust.bin
python -I -S tools/cell_kind4_reservoir_reference_v1.py verify target/cell-kind4-reservoir-repro target/cell-kind4-reservoir-repro/rust.bin
tools\build.bat test -p spore-bevy cell_start_reference_fixup_v1::tests::explicit_patchdata_candidate_uses_canonical_keys -- --ignored
```

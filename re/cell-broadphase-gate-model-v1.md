# Cell broadphase gate model v1

`spore_core::cell_broadphase_gate_v1::evaluate` models the bounded `E7F327..E7F381` strict predicate described in [the static trace](cell-broadphase-gate-static-v1.md). Seven focused tests passed, covering threshold equality and adjacent rounded values, distance accumulation order, cancellation and signed zero, subnormal behavior, checked nonfinite and overflow errors, and restoration of MXCSR on success and arithmetic error.

The inputs are two caller-supplied geometry outputs. Under explicit MXCSR `1F80`, the model preserves finite subnormals and signed zeros and evaluates the separately rounded operations in source order. It rejects nonfinite inputs and intermediates without output; this checked policy does not reproduce original exceptions, unordered comparisons, partial effects or status flags. The result means only that the strict numeric predicate passes. It does not establish original runtime floating-point state, actor identities, pool lookups, cache provenance, pair traversal, a live pair, contact or event.

## Synthetic comparison

The independent exact-rational binary32 oracle and Rust adapter matched 527 synthetic cases bit-for-bit: 522 successful results, three nonfinite intermediate errors and two nonfinite input errors. Directed cases include strict thresholds, nonassociative accumulation, signed zeros, subnormals, adjacent overflow boundaries and negative scalar values. The ignored `target/cell-broadphase-gate-model-v1` manifest pins the reference, arithmetic oracle, model, geometry types, SSE helper and adapter. Input SHA-256: `aae64f081be73ad3c5ba97601219bbecba332867820597c0d520dd6e146244e9`; expected and Rust reply SHA-256: `306e3a194608f7352933f2706f52254f1d03b16ace8add29b48fc1da66abd768`. The existing source pins and hashes were checked; compiled reply provenance is not authenticated. This is model agreement, not original instruction execution.

## Separately run installed-byte fixture

The ignored `explicit_patchdata_candidate_uses_canonical_keys` test passed separately using the explicitly selected, hash-bound PatchData package (SHA-256 `cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff`). The existing chain and its admissions are documented in [the geometry model note](cell-actor-geometry-projection-model-v1.md). Its actor and cache values are synthetic; the resulting center `(10,20,32)` and scalar `6` are compared with a synthetic second output. Distance squared 144 equals squared scalar sum 144 and skips; changing the second center from `(22,20,32)` to `(21,20,32)` yields distance squared 121 and passes. These numerical checks establish no second actor, source pair, contact or event. The test assumes the package is stable throughout; hashes before and after cannot rule out transient replacement because the reader reopens the path.

## Reproduction

From the repository root on x86/x86_64 with SSE2, use a fresh target directory:

```powershell
tools/build.bat test -p spore-core cell_broadphase_gate_v1
python -I -S tools/cell_broadphase_gate_reference_v1.py prepare target/cell-broadphase-gate-NEW
tools/build.bat run -p spore-core --example cell_broadphase_gate_compare_v1 -- target/cell-broadphase-gate-NEW/input.bin target/cell-broadphase-gate-NEW/rust.bin
python -I -S tools/cell_broadphase_gate_reference_v1.py verify target/cell-broadphase-gate-NEW target/cell-broadphase-gate-NEW/rust.bin
tools/build.bat test -p spore-bevy explicit_patchdata_candidate_uses_canonical_keys -- --ignored --nocapture
```

The ignored fixture requires the pinned local package; `RUSTSPORE_TEST_INSTALL` selects its install root. No original source-byte execution has been performed for this predicate.

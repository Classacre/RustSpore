# Cell sweep-pair model: scope and synthetic comparison

`spore_core::cell_source_sweep_pairs_v1` is registered in the core library. It is a bounded reconstruction of the finite, unique-endpoint portion of the source collection sweep and reverse iterator described in [the Cell integration trace](cell-source-health-integration-proposal-v1.md). It is not wired into the Cell gameplay update and does not construct actor geometry, track persistent endpoint updates, or model the source collection's free list and lifecycle.

Given explicit current slot IDs, raw tags, 2D endpoint values and pair capacity, the model sorts each axis, sweeps axis zero, rejects equal nonzero tags, checks axis-one overlap, uses swap-last active removal, stores each accepted pair with signed IDs in canonical order, suppresses pairs after the buffer reaches capacity, and exposes stored and reverse dispatch orders. Its suppression count is a Rust diagnostic, not a recovered native field.

The API rejects nonfinite or subnormal endpoints, ties, invalid intervals, more than 4,096 slots or pair capacity above 4,096, and more than 1,024 simultaneous active slots. These are explicit safety restrictions; they are not claims that the source checks those cases. Source equal-endpoint ordering, MXCSR behavior, merge scratch bounds, mutation during iteration and history-dependent lifecycle remain unproved. A rebuilt set of supplied endpoints is not evidence of the original collection's update history or of a live gameplay contact.

## Evidence

Six focused module tests passed. An independent static review compared the model to the source control flow. Separately, a synthetic Python reference and Rust adapter matched all 264 cases with zero mismatches, including swap-last ordering, signed and duplicate IDs, tag exclusions, capacity suppression, reverse iteration, 64 active slots and 256 seeded generated cases. This comparison's domain was unique finite normal-or-zero endpoints, strict intervals, at most 64 slots and capacity at most 128; it does not exercise the full API limits.

The ignored evidence directory `target/cell-sweep-model-v3/` contains `input.bin`, `expected.bin`, `rust.bin` and `manifest.json`. Input SHA-256: `2b98c16a56df9b14d2bda009190968a9ecc9b18cc5e325b3637c5dd7fc8d2b97`. Matched reply SHA-256: `79f991de5c5eec18fa8b2c5085a706f1ad62bd026e3dfd385c130d8f77e3d4dc`. The retained manifest pins the generator, model and adapter. A read-only rerun of the Python verifier confirmed the hashes and zero mismatches; the verifier reports that reply provenance is not authenticated. This is model-to-model comparison and static review, not original source execution, real actor-state observation, or live Cell integration.

To reproduce from the repository root, choose a new output directory under `target`:

```powershell
python -I -S tools/cell_source_sweep_pairs_reference_v1.py prepare target/cell-sweep-model-repro
tools\build.bat run -p spore-core --example cell_source_sweep_pairs_compare_v1 -- target/cell-sweep-model-repro/input.bin target/cell-sweep-model-repro/rust.bin
python -I -S tools/cell_source_sweep_pairs_reference_v1.py verify target/cell-sweep-model-repro target/cell-sweep-model-repro/rust.bin
```

A source-bound native comparison, persistent endpoint/lifetime model and actual actor-to-contact-to-event integration remain separate gates. No Cargo, GPU or native executable was run while writing this note.

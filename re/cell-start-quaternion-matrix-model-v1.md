# Cell start quaternion matrix model v1

`spore_core::cell_start_quaternion_matrix_v1::matrix_words` is the registered bounded model of the nine physically ordered binary32 words produced by source helper `59C000`. The source trace and exact individual operation order are in `cell-start-quaternion-matrix-static-v1.md`. Independent static review compared all nine outputs and operation order to the source instructions and found no defect on the declared domain. Six focused unit tests passed. The model does not normalize its four input words or establish a row/column convention, coordinate basis, actor, or live pose.

The model uses the explicit scoped MXCSR `1F80` policy: nearest-even, masked exceptions, gradual underflow and DAZ disabled. It retains finite input and intermediate subnormals and signed zeros, and rejects nonfinite inputs or intermediates as a checked safety policy. These restrictions and early error returns do not model original exception flags or native fault and partial-write behavior. The model returns an owned array and does not model source buffer aliasing. It makes no claim that the original game used this MXCSR state.

## Independent comparison

The independent Python exact-rational binary32 reference was compared with the Rust adapter on 530 synthetic cases under MXCSR `1F80`; all result/status words matched bit-for-bit. The retained ignored directory is `target/cell-quaternion-matrix-boundary-v2`. Input SHA-256 is `755932462f1109b2428d3a150ba2b4109dd386f0a24abf70cfb94671cc5b4b48`; Rust reply SHA-256 is `1aec36ee11b948fafce689bf351857dd65fc6fcf030730ef40b525fb57bbcc54`. The manifest records 526 successful cases, two nonfinite-input rejections and two nonfinite-intermediate rejections, including directed signed-zero, cancellation, rounding, subnormal and overflow boundary witnesses. It pins the reference, model, SSE helper and adapter source; it does not authenticate compiled reply provenance or execute source instructions. Agreement covers this finite synthetic corpus and checked errors, not the original runtime.

The separately passed ignored `explicit_patchdata_candidate_uses_canonical_keys` fixture extracts hash-bound explicit PatchData records and feeds its **synthetic** orientation result into this matrix model. It checks finite outputs and selected relationships among the resulting words. It does not independently compare those matrix words with original instructions or establish a real actor, retail resource selection, source RNG continuity or original FP control state. Fixture inputs, package hashes and the unproved RNG continuity admission are documented in `cell-start-orientation-model-v1.md`.

No original source-byte host execution has been performed for this matrix model. An independently reviewed guarded fixture using source bytes, explicit MXCSR and safe nonaliasing memory would be a separate validation gate. No semantic matrix layout, actual actor state, gameplay or original FP environment follows from the current evidence.

## Reproduction

From the repository root, on an x86/x86_64 host with SSE2, and with a fresh target directory:

```powershell
tools/build.bat test -p spore-core cell_start_quaternion_matrix_v1
python -I -S tools/cell_start_quaternion_matrix_reference_v1.py prepare target/cell-quaternion-matrix-NEW
tools/build.bat run -p spore-core --example cell_start_quaternion_matrix_compare_v1 -- target/cell-quaternion-matrix-NEW/input.bin target/cell-quaternion-matrix-NEW/rust.bin
python -I -S tools/cell_start_quaternion_matrix_reference_v1.py verify target/cell-quaternion-matrix-NEW target/cell-quaternion-matrix-NEW/rust.bin
tools/build.bat test -p spore-bevy explicit_patchdata_candidate_uses_canonical_keys -- --ignored --nocapture
```

The final test requires the explicitly pinned local PatchData package, with install root selected by `RUSTSPORE_TEST_INSTALL` or its default documented in the orientation note. The note author did not run Cargo, GPU, native fixtures or original game code while writing this note.

The reported full all-target workspace run `target/workspace-tests-20261030-resume-v65.log` passed 843 tests, failed 0 and ignored 4; its SHA-256 is `200119fe76f1db4969f7f0e4d1ce370ab9224cf263ad8fb5982c194d6a3f6ec9`. The ignored hash-bound asset fixture passed separately and is not included in that count.

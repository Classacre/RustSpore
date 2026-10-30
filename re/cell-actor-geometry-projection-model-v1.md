# Cell actor geometry projection model v1

`spore_core::cell_actor_geometry_projection_v1::project` is a registered bounded projection of source `E6D7F0`, described in `cell-actor-geometry-consumer-static-v1.md`. Independent static review compared both branches, the nine physical matrix indices, scalar and ordered SSE operations with the source instructions and found no defect on the declared domain. Eight focused unit tests passed. This is a projection over explicit values, not a live actor or cache implementation.

For first actor word +FC zero, the checked model returns the finite supplied position and scalar directly, ignoring cache and matrix. For nonzero +FC it requires an explicit cache snapshot whose first key word matches, multiplies the cache scalar by actor scalar, optionally rotates the supplied cache center when flag bit 02 is set, then scales and offsets it in source operation order. The caller admits a stable, appropriate snapshot; matching the first word does not prove a real cache hit, full-triple validity, population, provider, lifetime or resource identity. No row/column convention or collision result is inferred.

The SSE path uses explicit MXCSR `1F80` (nearest-even, masked exceptions, gradual underflow, DAZ off), preserving finite subnormals and signed zeros. It rejects nonfinite inputs and intermediate results and returns no partial output. These are safe-model policies, not native error/exception or partial-write semantics. The model does not establish original runtime floating-point state or buffer aliasing behavior.

## Independent comparison

The reported independent Python exact-rational binary32 reference and Rust adapter matched all 530 synthetic cases bit-for-bit under MXCSR `1F80`. The ignored directory `target/cell-actor-geometry-boundary-v2` contains the fixture. Its input SHA-256 is `79a637ef5833df262e854d6430c1e0fdbf597f38c9ff64ddf0913a3b2d6be7ea`; Rust reply SHA-256 is `40640897f7918a4349be1eb66724dbdc2c51fe20b9f28563e2c44afc7d6c99d5`. The manifest records 524 successful cases, one missing-cache rejection, one wrong-key rejection, two nonfinite-input rejections and two nonfinite-intermediate rejections. Directed cases include physical index order, cancellation, signed zeros, subnormal rounding and overflow boundaries. Reference, binary32 oracle, model, SSE helper and adapter source are pinned; the reply's compiled provenance is not authenticated. This is model agreement on synthetic inputs and checked policies, not original source execution or runtime cache or actor observation.

## Separately run installed-byte fixture

The ignored hash-bound `explicit_patchdata_candidate_uses_canonical_keys` test passed separately with explicit PatchData globals, CELL and list bytes. The existing synthetic size, orientation and selection chain is described in `cell-start-orientation-model-v1.md`; it explicitly assumes no intervening RNG mutation, which remains unproved. The chain uses the selected first word `E219E5D3` as a symbolic cache key and supplies a synthetic actor position `(10,20,30)`, scalar `2`, rotation flag `02`, the synthetic orientation matrix, and an admitted cache center `(0,0,1)` and scalar `3`. Its checked result is center `(10,20,32)` and scalar `6`; wrong-key, missing-cache and zero-+FC behavior are also checked. This simple axis-aligned fixture checks handoff and selected relationships, not general matrix or projection parity. It does not prove a native handle, cache, actor, resource winner, source RNG continuity or runtime FP state.

## Reproduction

From the repository root, on an x86/x86_64 host with SSE2 and using a fresh target directory:

```powershell
tools/build.bat test -p spore-core cell_actor_geometry_projection_v1
python -I -S tools/cell_actor_geometry_reference_v1.py prepare target/cell-actor-geometry-NEW
tools/build.bat run -p spore-core --example cell_actor_geometry_compare_v1 -- target/cell-actor-geometry-NEW/input.bin target/cell-actor-geometry-NEW/rust.bin
python -I -S tools/cell_actor_geometry_reference_v1.py verify target/cell-actor-geometry-NEW target/cell-actor-geometry-NEW/rust.bin
tools/build.bat test -p spore-bevy explicit_patchdata_candidate_uses_canonical_keys -- --ignored --nocapture
```

The ignored fixture requires the pinned local package; `RUSTSPORE_TEST_INSTALL` selects the install root. Its package stability caveat and exact hashes appear in the orientation note. No original source-byte execution has been performed for this projection. The note author did not run Cargo, GPU, native fixtures or original game code while writing this note.

The reported full all-target workspace run `target/workspace-tests-20261030-resume-v66.log` passed 851 tests, failed 0 and ignored 4; its SHA-256 is `4c1eed39bc324d86ab643bb6dfb33fe58765b6fe019979f3cd49e38f081cef48`. The SHA-bound PatchData fixture described above passed separately and is not included in that count. Its actor fields, cache snapshot and key association are explicit synthetic admissions, not observed runtime state.

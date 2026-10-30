# Cell start equal-size transition model v1

`crates/spore-core/src/cell_start_equal_size_v1.rs` is a registered, bounded model of the pre-allocation size transition in the `E74A50` constructor's `E751C0` argument-5 = -1 / argument-7 = +0 branch. The constructor and call evidence is recorded in `re/cell-start-actor-constructor-static-v1.md`. Independent static review reported no source-order defect on the declared domain. Six module unit tests passed in the reported run; those tests cover admission, RNG advancement, rounding and rejection boundaries. This does not establish a complete actor.

The caller admits key `CBCD1287:DFAD9F51:00000000`; raw globals startCell and the definition's structure ID must both equal `CBCD1287`. The structure-ID check is an additional candidate restriction, not identity proof. The API does not authenticate bytes, package provenance, fixup or live selection. It admits identical positive normal binary32 size endpoints, positive normal supplied global `+514c` and argument 6, a nonzero RNG state and nonzero sampled fraction, masked x87 exceptions and PC53 or PC64. It advances the RNG once even for equal endpoints, then evaluates `(sample * (1 / global_514c)) * argument_6` in source order with the modeled x87 control state and stores binary32. It rejects unsupported or non-normal results. The equal-endpoint restriction avoids modeling the general sampler's binary64 intermediate stores, which the current x87 API does not provide. Orientation, cache, allocation, side effects, FP flags and gameplay are outside this slice.

Errors are a checked-model safety policy: this pure interface returns no advanced RNG state on an error, whereas native execution may already have advanced the RNG before a later exceptional result or fault. Exceptional native floating-point behavior and original FP control state are not established.

## Independent comparison

The separate Python exact-rational reference was compared with the Rust adapter on 276 rows: 216 successful and 60 rejected. The checked fixture directory is `target/cell-start-size-fixture-boundary-v1` (ignored build artifacts). The case stream SHA-256 is `b581fbd3ab421b8d501b4bc71bcfaf4b29652fe0f808eb5df31e1a576054313d`; Rust reply SHA-256 is `e443b1a7cd0fa8b8810a6b25d7e77b0a1d06f1186797f5e42a239d72b8ee0b86`. Verification reports all 276 matching, while explicitly reporting reply provenance unauthenticated and no source execution. This is agreement of implementations on the admitted fixture domain, not execution of the original game or proof of native FP-state parity. The manifest describes fixture preparation and its `rust_compared: false` field is not a record of the later verification.

The rational comparison reconstructs synthetic definitions and globals from the case rows, including the candidate endpoint subset; it does not pass the complete installed records through the Rust adapter. It must be distinguished from the full-byte test below.

## Installed-byte handoff

The ignored `explicit_patchdata_candidate_uses_canonical_keys` test passed using explicit PatchData and complete extracted globals and CELL bytes through parsing, checked fixup and the size API. It pins package SHA-256 `cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff`, globals SHA-256 `e8c4b80348fd8f3a1821533c87110315ba584f66f081d46305ff61ab6cbf8c72`, and CELL SHA-256 `f12d98c0f59af6d4d58174016f1e04d709647480310103341193df9b0fa76542`. With explicitly synthetic global scale `1.0`, argument 6 `1.0`, RNG seed `1` and control word `027F`, sampled and scale bits were both `3E8CCCCD`, with next RNG state `278DDE6D`. The package must remain stable throughout the test; before/after hashing cannot rule out transient replacement while its index reader reopens the path. The symbolic descriptor and explicit package provenance are fixture admissions. This test does not prove retail overlay selection, actual native fixup, a live actor or original runtime inputs.

## Reproduction

From the repository root, with the matching installed package and a fresh fixture output directory:

```powershell
python -I -S tools/cell_start_equal_size_rational_v1.py prepare 'D:/Games/SPORE Collection/Data/PatchData.package' target/cell-start-size-fixture-NEW
tools/build.bat run -p spore-core --example cell_start_equal_size_compare_v1 -- target/cell-start-size-fixture-NEW/cases.tsv target/cell-start-size-fixture-NEW/rust.tsv
python -I -S tools/cell_start_equal_size_rational_v1.py verify target/cell-start-size-fixture-NEW target/cell-start-size-fixture-NEW/rust.tsv
tools/build.bat test -p spore-core cell_start_equal_size_v1
tools/build.bat test -p spore-bevy explicit_patchdata_candidate_uses_canonical_keys -- --ignored --nocapture
```

The ignored test uses `RUSTSPORE_TEST_INSTALL` as the install root when set, otherwise `D:\Games\SPORE Collection`. The unique function-name filter selects the ignored test. These are reproduction commands, not runs performed while writing this note.

The reported full all-target workspace run `target/workspace-tests-20261030-resume-v62.log` had 824 passed, 0 failed and 4 ignored; its SHA-256 is `a0e420b48678755461167fd84fa22c1690a98bf67354aaa0b577fc9a1f791b04`. The hash-bound installed-byte test was run separately because it is ignored by default. Independent review of the 276-row comparison also identified true midpoint and directed-overflow cases; the adapter still uses synthetic records with the source-derived endpoint, not the full installed bytes.

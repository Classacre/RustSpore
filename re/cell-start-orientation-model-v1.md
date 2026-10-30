# Bounded Cell start null-orientation model

`spore_core::cell_start_null_orientation_v1` models the conditional quaternion value slice `E74B98..E74C0D`; the instruction trace is in `cell-start-null-orientation-static-v1.md`. Independent static review reported no source-order defect within its declared domain. Four focused unit tests passed, covering one RNG draw and retained angle, signed-zero axes, rejected inputs and control words, and the angle bound. These tests do not constitute original source-byte execution.

The caller admits definition byte `+AC != 1`, a null orientation pointer, successful prior allocation and an explicit post-size RNG state. The function requires a nonzero RNG state and nonzero returned fraction, a positive normal binary32 angle global, and finite normal or signed-zero binary32 axes. It admits masked x87 exceptions and PC53 or PC64 with any supported rounding mode; the sole SSE policy is MXCSR `1F80` (nearest-even, masked exceptions, gradual underflow, DAZ off). A computed absolute angle over four is rejected by the bounded transcendental helper; nonfinite outputs are rejected. The returned state, retained angle bytes, binary32 sine/cosine and four quaternion words are fixture values, not authenticated actor state.

The source-order model advances the RNG once, multiplies its retained fraction by the supplied angle global and then 0.5, evaluates host `FSIN`, commits sine to binary32, evaluates host `FCOS` on the original retained angle, performs three separate SSE `MULSS` operations on the stored sine and axis words, then commits cosine. `FSIN` and `FCOS` use the host processor's approximations and check C2; they do not provide platform-independent trigonometric bits. The helpers restore the x87 environment and the SSE scope restores MXCSR, so native sticky flags and exception paths are outside this model. This pure API returns no advanced RNG state on error even when native execution could already have advanced it. This is a checked safety policy, not native error behavior.

## Explicit installed-byte fixture

The separately run ignored test `explicit_patchdata_candidate_uses_canonical_keys` passed using hash-bound explicit PatchData globals, CELL and kind-4 list bytes. Package SHA-256 is `cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff`; decoded globals, CELL and list SHA-256 values are respectively `e8c4b80348fd8f3a1821533c87110315ba584f66f081d46305ff61ab6cbf8c72`, `f12d98c0f59af6d4d58174016f1e04d709647480310103341193df9b0fa76542`, and `29b0f195b26bf2480586518e385933284819ff360a991ea706a939717a5d5f8a`. The package must remain stable throughout; before/after hashing cannot exclude transient replacement between path reopenings.

The test supplies synthetic size global and argument 6 of `1.0`, seed `1`, CW `027F`, orientation global bits `40C90FDB`, axis bits `[00000000, 00000000, 3F800000]`, MXCSR policy `1F80`, structure-handle key `42`, third word `7` and fresh cache. Size advances to `278DDE6D`; the orientation advances to `FB533A69`. Under the explicit unproved assumption that intervening calls do not mutate RNG, four selection draws produce row index 2, word `E219E5D3`, and final state `492C37F9`. Parsed disk list rows are admitted as a runtime snapshot for this fixture, not proven fixed-up runtime rows. The test checks quaternion component relationships and finite output; it does not independently authenticate the host's sine/cosine bits against original instructions.

The caller-supplied descriptor, package, cache and handles are fixture admissions. The test does not establish retail provider selection, actual fixup, live RNG or floating-point control state, matrix output, a constructed actor or gameplay. Resolver callbacks and insertion effects make the no-intervening-RNG assumption unproved; see `cell-constructor-orientation-selection-rng-gap-v1.md`. The proposed guarded native-host source-byte comparison remains incomplete and has not been executed for this slice.

## Reproduction

From the repository root, on an x86/x86_64 host with SSE2:

```powershell
tools/build.bat test -p spore-core cell_start_null_orientation_v1
tools/build.bat test -p spore-bevy explicit_patchdata_candidate_uses_canonical_keys -- --ignored --nocapture
```

The ignored test uses `RUSTSPORE_TEST_INSTALL` when set, otherwise `D:\Games\SPORE Collection`, and requires the pinned explicit PatchData snapshot. The note author did not run Cargo, GPU, native fixtures or original game code while writing this note.

The reported full all-target workspace run `target/workspace-tests-20261030-resume-v64.log` passed 837 tests, failed 0 and ignored 4; its SHA-256 is `daf43dc75f86a3f24bf23926988110a8f0042f504b90f05b44875d6c1d5e4016`. The hash-bound installed-byte fixture passed separately and is not included in that count.

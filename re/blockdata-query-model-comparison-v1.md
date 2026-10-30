# Blockdata query model comparison

2026-10-30. `spore_core::blockdata_query_v1::query` evaluates a checked
`NativeSamples` view using the source-guided dimension, corner and interpolation
sequence described in `blockdata-query-interpolation-static-v1.md`. It returns
the raw-buffer result; the lookup wrapper's component-zero validity test and
default replacement are separate. Exact payload extent, checked sizes and
finite-domain errors are Rust safety policies, not native error semantics.
The module is integrated into `spore-core`, not a gameplay caller.

## Validation

The retained full workspace test log
`target/workspace-tests-20261030-resume-v57.log` has SHA-256
`71d2e3bf104454c2475174377aa9a544f5a635fa7b7bac112a9233bc689afff1`.
Its 169 result summaries total **790 passed, 0 failed, 2 ignored**, including
nine `blockdata_query_v1` unit tests. Those tests include bounds, coordinate
policy, overflow and a rounding-sensitive recursion-order example.

The Rust production adapter and separately written Python binary32 model also
agree on every output byte for both retained bounded sets:

| Set | Cases | Input SHA-256 | Rust reply SHA-256 |
| --- | ---: | --- | --- |
| Synthetic finite 2×2 grids | 128 | `cd50811dabda22aa863943b8adde38ad7a74edc5c6b0465f8c632858ec89e4a4` | `f14ad1f7b424f5e03c33e731d7249a2a15da2e9107afbbf57dc22f7a0aeebfbb` |
| Retained package records | 8,446 | `698cef44aab58fe372ef7e5f818ec784518482104d7be71b5ddbf4dd95793aff` | `cdb6f8be5fff36140160800a9b5bb25bfd69ce1070476a56ea7f0a2e7789d414` |

For the synthetic set, case zero distinguishes both reversed recursion and a
reassociated blend in binary32. The retained evidence is under
`target/blockdata-query-production-reference-v4/`, including `input.bin`,
`expected.bin`, `rust.bin` and a source-pinning `manifest.json`.

The second set derives from 3,098 selected package **records**, all with finite
header and scale words. It includes 424 zero-dimension header cases and three
coordinate sets for each of 2,674 nonzero-dimension records. No record was
excluded for a nonfinite header/scale and no selected case for a nonfinite or
overflowing result. The retained evidence is `target/blockdata-query-real-v4/`;
its manifest pins the generator, reference, adapter, production query and
blockdata module, the source corpus and its manifest. `case-map.json` maps each
query back to its source stream ordinal and coordinate policy.

The retained source scan excludes `Spore_Audio1.package` because its index
layout was unsupported; its number of matching records is unknown. Package
records include duplicates: a read-only SHA-256 census found 1,432 distinct
payload byte digests among the 3,098 records. The source stream does not retain
each resource key or prove runtime override selection.

These are independent implementation comparisons over selected finite data,
not original-game execution. They do not establish arbitrary exceptional-float
or floating-point control-state equivalence, real runtime identity, collision,
terrain semantics or gameplay behavior.

## Reproduction

From the repository root, use fresh output directories. The retained original
package-stream input and its manifest must be present for the real-data run.

```powershell
python -I -S tools/blockdata_query_compare_v1.py prepare target/blockdata-query-production-repro
tools\build.bat run -p spore-core --example blockdata_query_compare_v1 -- target/blockdata-query-production-repro/input.bin target/blockdata-query-production-repro/rust.bin
python -I -S tools/blockdata_query_compare_v1.py verify target/blockdata-query-production-repro target/blockdata-query-production-repro/rust.bin

python -I -S tools/blockdata_query_real_corpus_v1.py prepare target/blockdata-corpus-v1/input.bin target/blockdata-corpus-v1/manifest.json target/blockdata-query-real-repro
tools\build.bat run -p spore-core --example blockdata_query_compare_v1 -- target/blockdata-query-real-repro/input.bin target/blockdata-query-real-repro/rust.bin
python -I -S tools/blockdata_query_real_corpus_v1.py verify target/blockdata-query-real-repro target/blockdata-query-real-repro/rust.bin
```

The verifier checks current source pins, corpus hashes, output framing and all
reply bytes. It does not authenticate who produced a supplied reply; the
retained build/run provenance remains a separate part of the evidence.

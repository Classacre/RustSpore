# Manual Cell selector context

This is a controlled source-table fixture for the bounded `E6CC30` selector projection, not an observation of a live actor. The 131-row table was read from virtual address `01484A20` in the base executable with SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. Its 4,716 bytes have SHA-256 `ac4aa6836350b90495f7cd5e20fb38e64986d89472d33f7b1ba8e5e5dfffcfda`. The table bytes remain in ignored `target/`.

The Rust fixture calls `select_snapshot` with manually supplied state 0, actor mode 0, an empty part-code list, null context flags, and both predicates true for every row. In that context it admits source-table row indexes 0, 50 and 101 in order, each with weight 5, yielding sum 15. Three manually supplied binary32 draws produce these request keys:

| Draw | Request key |
| --- | --- |
| 0 | `AAAA0015` |
| 5.5 | `AAAA0049` |
| 10.5 | `AAAA0097` |

The context, predicates and draws are not derived from `A1C46DF2` or any actual actor. No live RNG behavior, installed clip selection, binding, pose or playback follows from the outputs. The table and selector evidence are discussed in [the actor animation request trace](cell-actor-animation-request-trace-v1.md).

The coordinating agent reported this command after building the local example:

```powershell
python tools/cell_manual_selector_context_v1.py 'D:\Games\SPORE Collection\SporeBin\SporeApp.exe' --example target/debug/examples/cell_manual_selector_context_v1.exe --example-sha256 a1d4e5a7890b2af4e6144b8dc79b20e55669d63c28c8be3f96dff6fa1f29e8b8 --out target/cell-manual-selector-context-v1
```

The retained ignored `target/cell-manual-selector-context-v1/report.json` has SHA-256 `70dec83540236cd41761553965d6d26b2cb070ecef81d9ed7e56aeb1a25cebc5`. It reports a zero exit, empty stderr, matching expected and actual output SHA-256 `3b28338adf5b967eedcd0e0350ca1e32ac7792c752c646f4ff87707cb3d22f52`, and matching before/after source executable, extracted table and example hashes. The report does not bind the example binary to a build command or source tree. The script and example source at documentation time hash to `6f47bed92ac45152aa96bfaea7f67b2e4ed5bf95c68d37118b333c095800ccc8` and `b85670214066f15e8d4c2e69834f55a528d274cb0d2486f03bc06fd490f945a4`, respectively; these current hashes are not independently bound to that binary. A repeat requires a new target output directory and the pinned trusted executable and example. Hashes before and after do not exclude transient replacement.

# Source-bound capability join v1

`crates/spore-bevy/examples/cell_capability_source_join_v1.rs` is an opt-in, install-dependent composition of a selected source creature with retained classification, geometry-prefix, and scalar fixture records. It reads an explicitly supplied package and fixture directory, checks pinned evidence and payload hashes, constructs ten ordered part rows, and compares the modeled results against the retained prefix and scalar references.

## Recorded verification

The following commands passed for this workspace:

```powershell
tools\build.bat test -p spore-bevy --example cell_capability_source_join_v1
tools\build.bat run -p spore-bevy --example cell_capability_source_join_v1 -- 'D:\Games\SPORE Collection\Data\Spore_Content.package' target/source-cell
```

The test command passed three unit tests. The example exited successfully and reported ten ordered rows with matching retained prefix and scalar references. It also reported that classification historical input immutability is unverified. To reproduce, substitute the installed `Spore_Content.package` path and the directory containing the retained fixtures if they differ. The example requires x86 or x86-64 with SSE2, the pinned executable and fixture files, and the exact package described below.

## Source and evidence boundary

The selected package is SHA-256 `0b849bd79d9c13dec5737159655bf7e2b7763cc2b79949c81370fa0490256e8c`. The uniquely selected resource has instance `A1C46DF2`, type `0F43029A`, scope `40616200`, decoded length 1,889 bytes and SHA-256 `6852040fa9a9376ed9025f4823e05024a22685a147e00e64d1b0f536b33e8cd8`. The example also checks the installed `SporeApp.exe` against SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.

The fixture checks bind current retained request and reply files, source payloads, historical reports, host binaries and manifests to the hashes in the example. Those hashes identify retained evidence and historical assertions; they do not independently prove how the historical runs were performed. The classification runner lacks an equivalent retained historical input immutability record, so its provenance remains unverified. The geometry and scalar historical pin and immutability records are retained assertions, not proof against transient replacement. See [retained host evidence](cell-capability-retained-host-evidence-v1.md) for the detailed inventory and [source-bound join plan](cell-capability-source-bound-join-plan-v1.md) for the admissions and comparison sequence.

The package and fixture files must remain stable and must not be concurrently modified or replaced during the run. Before-and-after hashes cannot detect transient replacement when a path is reopened. Globals, curve, floating-point context, origin and other fixture inputs are explicit admissions; they do not establish runtime selection or state.

This verification uses separately retained host references. It performs no new native execution and does not establish a live actor, a live provider or resource-manager winner, or a contiguous original invocation. The comparison supports only the stated source-bound fixture composition.

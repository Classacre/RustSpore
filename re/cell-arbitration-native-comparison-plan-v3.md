# E59B40 local comparison, version 3

This version retains the finite-case and native-host scope described in the [original plan](cell-arbitration-native-comparison-plan.md), with the hash checks and containment limitations explained in the [version 2 plan](cell-arbitration-native-comparison-plan-v2.md). It replaces direct `rustc` compilation with the public `spore-core` Cargo example `crates/spore-core/examples/cell_arbitration_oracle.rs`. The x64 example communicates with the x86 native host through hexadecimal text and compares binary32 bit patterns as integers. Previous versions remain untouched.

After static source and linked-binary review, the local v3 build and comparison
completed: all 314 cases matched on the twelve final output words. The result is
`target/cell-arbitration-oracle-review-v3/results.json`, SHA-256
`702d0cb939cca24a2eda00a3213d3af3eafd306c738798fb614e7a0eb3742bae`.
The limits below still apply, and this is not a live-game acceptance result.

Review the v3 scripts, generated host, example and manifest, then separately review the executable hashes and build records before any execution. The following uses values independently obtained and recorded by the operator; paths point to actual tools and pins to their independently inspected SHA-256 values:

```powershell
python tools/cell_arbitration_native_prepare_v3.py 'D:/Games/SPORE Collection/SporeBin/SporeApp.exe' --out target/cell-arbitration-oracle-v3-new
Get-FileHash -Algorithm SHA256 target/cell-arbitration-oracle-v3-new/manifest.json
Get-FileHash -Algorithm SHA256 $cargoPath,$rustcPath,$clPath,$x86LinkPath
python tools/cell_arbitration_cargo_build_v3.py --prep target/cell-arbitration-oracle-v3-new --prep-manifest-sha256 $prepHash --source-exe 'D:/Games/SPORE Collection/SporeBin/SporeApp.exe' --cargo $cargoPath --cargo-sha256 $cargoHash --rustc $rustcPath --rustc-sha256 $rustcHash
```

The Cargo step invokes exactly `tools/build.bat build --locked -p spore-core --example cell_arbitration_oracle --target x86_64-pc-windows-msvc --target-dir <prep>/cargo-target`. It records the command, output, Cargo and rustc executable hashes and versions, selected source identities, output path and binary digest in `<prep>/cargo-build-manifest.json`. Use actual toolchain binaries rather than proxies when possible. The target directory must be new. The existing prepared static-only manifest is `target/cell-arbitration-oracle-review-v3/manifest.json`; if any pinned source changes, prepare a new directory.

After reviewing the Cargo receipt and adapter, supply their independent hashes to the native builder from an x86 MSVC environment. The builder compiles only the native host; it does not invoke Rust tools:

```powershell
Get-FileHash -Algorithm SHA256 target/cell-arbitration-oracle-v3-new/cargo-build-manifest.json
Get-FileHash -Algorithm SHA256 target/cell-arbitration-oracle-v3-new/cargo-target/x86_64-pc-windows-msvc/debug/examples/cell_arbitration_oracle.exe
python tools/cell_arbitration_native_build_v3.py --prep target/cell-arbitration-oracle-v3-new --prep-manifest-sha256 $prepHash --source-exe 'D:/Games/SPORE Collection/SporeBin/SporeApp.exe' --cl $clPath --cl-sha256 $clHash --link $x86LinkPath --link-sha256 $x86LinkHash --adapter target/cell-arbitration-oracle-v3-new/cargo-target/x86_64-pc-windows-msvc/debug/examples/cell_arbitration_oracle.exe --adapter-sha256 $adapterHash --cargo-manifest target/cell-arbitration-oracle-v3-new/cargo-build-manifest.json --cargo-manifest-sha256 $cargoManifestHash
Get-FileHash -Algorithm SHA256 target/cell-arbitration-oracle-v3-new/build-manifest.json
```

After inspecting the linked native host and the build record, run with its reviewed digest:

```powershell
python tools/cell_arbitration_native_compare_v3.py --prep target/cell-arbitration-oracle-v3-new --build-manifest-sha256 $buildHash --out target/cell-arbitration-oracle-v3-new/results.json
```

Preparation pins the binary, generated host, example, arbitration module, complete `spore-core/src` Rust file tree, workspace/crate manifests, lockfile, build.bat and all v3 scripts. The Cargo build checks the inputs and Cargo/rustc binary pins before and after, and emits a receipt. The native builder and runner check that receipt, adapter hash, inputs and compiler binaries before and after their work. Results bind the receipts and binary hashes. This is not a hermetic build: Cargo configuration, rustc sysroot, loaded DLLs, compiler helpers, environment, and MSVC/linker and SDK selected by build.bat are unpinned. Captured build output identifies the selected MSVC and SDK when the batch file succeeds; a pin of `cargo.exe` or `rustc.exe` alone may describe a proxy, not the ultimately executed toolchain. Before/after checks do not detect changes that are restored in between. Script execution trusts all selected binaries and build scripts.

Timeouts kill the direct process only and do not contain descendants. They may survive an interrupted build or comparison. There is no process sandbox or Job Object. A failed build can leave partial files; use a fresh target directory. A passing comparison covers final output for the finite, non-aliasing inputs only, not the independent write-path flags, geometry, runtime reachability, or unmasked FP exceptions.

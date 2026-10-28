# E59B40 hash-bound local comparison, version 2

Version 2 keeps the same scope and cases described in [the version 1 plan](cell-arbitration-native-comparison-plan.md), but uses separate scripts and does not change the version 1 files. Neither build nor native execution has been performed. The x86 native host and x64 Rust adapter communicate only by raw hexadecimal words in text; the adapter performs integer bit comparisons and does not require an i686 Rust target.

`tools/cell_arbitration_native_prepare_v2.py` checks the original PE SHA-256, generates the same host and adapter under `target/`, and records hashes of the generated sources, Rust source, and all three v2 scripts. Review those sources and the manifest, and retain the independently read SHA-256 of the manifest. The builder requires that exact manifest digest plus operator-supplied SHA-256 pins for x86 `cl.exe`, x86 `link.exe`, `rustc.exe`, and x64 `link.exe`; it checks those and all prepared inputs before and after building, sets explicit linkers, captures compiler version output, commands, tool paths, and hashes, and records output executable hashes in `build-manifest.json`. Use the actual toolchain `rustc.exe`, not an unpinned rustup proxy. Building is opt-in and runs compiler executables and their subprocesses.

The comparison runner requires the independently reviewed build-manifest hash. Before and after the run, it checks that manifest, prep manifest, original PE, Rust source, generator/builder/runner sources, generated sources, compiler and linker executables, built executables, and Python executable still match their recorded hashes. Results record manifest and executable identities, compiler paths and hashes, and cases. A mismatch prevents a pass. These endpoint checks are not protection against a file being modified and restored during execution. Independent confirmation of the output hashes and inspection of linked binaries remain necessary before running.

The `PATH`, `INCLUDE`, `LIB`, `LIBPATH`, and x64 Rust `LIB` values are represented by hashes, without exposing their contents. The selected compiler executables are pinned, but their loaded DLLs, compiler helpers, SDK headers/libraries, Rust sysroot and other runtime dependencies are not. The recorded environment hashes do not attest file contents. Python's interpreter binary is pinned at build and run, but Python libraries and OS components are not. Thus this is a reproducibility aid, not a fully hermetic build or security sandbox. The scripts trust the selected tools. Timeouts kill the direct child only: descendants may survive, and there is no Windows Job Object or process-tree containment. An interrupted or failed build can leave partial files in the ignored directory; prepare a new directory instead of assuming those files are valid.

From a configured x86 MSVC developer environment, capture the x64 `LIB` string separately from a matching x64 developer environment. Supply absolute paths and review these hashes before use; use the relevant installed toolchain paths rather than assuming the examples exist:

```powershell
python tools/cell_arbitration_native_prepare_v2.py 'D:/Games/SPORE Collection/SporeBin/SporeApp.exe' --out target/cell-arbitration-oracle-review-v2-new
Get-FileHash -Algorithm SHA256 target/cell-arbitration-oracle-review-v2-new/manifest.json
Get-FileHash -Algorithm SHA256 $clPath,$x86LinkPath,$rustcPath,$x64LinkPath
python tools/cell_arbitration_native_build_v2.py --prep target/cell-arbitration-oracle-review-v2-new --prep-manifest-sha256 $reviewedPrepHash --source-exe 'D:/Games/SPORE Collection/SporeBin/SporeApp.exe' --cl $clPath --cl-sha256 $clHash --link $x86LinkPath --link-sha256 $x86LinkHash --rustc $rustcPath --rustc-sha256 $rustcHash --rust-link $x64LinkPath --rust-link-sha256 $x64LinkHash --rust-lib $x64Lib
Get-FileHash -Algorithm SHA256 target/cell-arbitration-oracle-review-v2-new/build-manifest.json
```

The builder writes intermediate objects and executables in the preparation directory. After inspecting build logs and linked binaries and recording the reviewed build manifest digest:

```powershell
python tools/cell_arbitration_native_compare_v2.py --prep target/cell-arbitration-oracle-review-v2-new --build-manifest-sha256 $reviewedBuildHash --out target/cell-arbitration-oracle-review-v2-new/results.json
```

The existing static-only preparation is `target/cell-arbitration-oracle-review-v2-final`; an earlier `review-v2` preparation was superseded when scripts changed and must fail hash verification. A direct-child timeout is reported as a failure and terminates the case loop. Other exceptions, including preflight or postcheck failures, abort without a success result. This bounded result tests final output only; it does not independently prove either write-path flag, aliasing, unmasked FP exceptions, candidate production, or live game behavior.

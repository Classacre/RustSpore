# Nonzero Cell request RNG: prepared native comparison

The original executable is read only, and its SHA-256 must be `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. All extracted bytes, generated C, binaries and result streams belong under ignored `target/`. The Python preparer requires the existing local Capstone runtime at `target/rea/capstone-runtime`.

After independent source and linked-host inspection, the v2 local comparison
matched all 10,368 controlled cases on updated state and numeric binary64
value. The report `target/cell-request-rng-native-result-v2/report.json` has
SHA-256 `dca3294970e62137261d0aaa859fa2ad168c89a7ffe0121292c588af6b9a2491`.
The comparison excludes zero sign, x87 status and the other boundaries below.

The copied interval is `[935D40,935D76)`. Capstone statically verifies a complete decode ending in RET, no calls, only two direct internal branches, and precisely two absolute memory operands. Code is mapped at its original address so all relative branches and absolute constant pointers retain their source addresses. Constants at `0143F52C` and `01471B2C` are verified respectively as binary32 2^-32 and 0.5. ECX points to a nonzero local state word. That bypasses the RDTSC branch, and the odd multiplier preserves nonzero state. The host saves the calling thread's x87 control word, loads the explicitly supplied masked PC53/PC64 control word, calls the routine, stores ST(0) directly to binary64, and restores the control word. It does not establish the game's active control word or compare status flags. Fixed address reservation must succeed; failure is explicit.

The corpus exercises all four rounding modes at PC53 and PC64, signed boundaries and output endpoints using inverse-derived seeds, 64 deterministic pseudo-random seeds, and 16 successive recurrence states per seed. Comparison checks exact updated state and numeric binary64 value. Both zero signs compare equal; status, sign-sensitive downstream operations, PC24, zero seeds, live game state and broader behavior are excluded. No binary32 conversion occurs in the output path.

Prepare, in a fresh output directory (this step only reads the source and generates C):

```powershell
python tools/cell_request_rng_native_prepare_v1.py 'D:/Games/SPORE Collection/SporeBin/SporeApp.exe' --out target/cell-request-rng-native-prep-v2
```

Review the generated manifest, C, and linked source before native work. In an x86 MSVC developer shell, build with x86 `cl`, and build the public model adapter with the installed Rust toolchain. Example, from the repository root:

```powershell
cl /nologo /W4 /O2 /TC target/cell-request-rng-native-prep-v2/host.c /Fe:target/cell-request-rng-native-prep-v2/host.exe /Fo:target/cell-request-rng-native-prep-v2/host.obj
$env:CARGO_TARGET_DIR = 'D:/AI/RustSpore/target/cell-request-rng-native-prep-v2/cargo-target'
.\tools\build.bat build -p spore-core --example cell_request_rng_native_compare_v1
python tools/cell_request_rng_native_run_v1.py --host target/cell-request-rng-native-prep-v2/host.exe --adapter target/cell-request-rng-native-prep-v2/cargo-target/debug/examples/cell_request_rng_native_compare_v1.exe --out target/cell-request-rng-native-result-v2
```

The run script records request and both output streams before checking each state and value. A completed report is evidence only for this finite corpus and supplied policies; differing hashes can be valid when the only difference is the sign of zero. Review compiler/toolchain identity and binaries as appropriate to the intended trust boundary.

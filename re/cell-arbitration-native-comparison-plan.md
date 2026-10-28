# E59B40 local comparison preparation

This is a preparation, not a native result. No compiler or native program was run. Static evidence is Ghidra q493 (`ev_ca9d95f9fe0f313cabe5c27eec324c49da02888e50e70e66447be17def314e8e`) for base SporeApp.exe SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. Function range is inclusive `0xe59b40..0xe59c0b`. Its conditional branches target locations within that range, it contains no calls, and its external absolute data access is `0x016b4be4` to a pointer whose `+0x411c` word is read on the selector-eight path. It reads nine stack arguments, preserves EBX/EBP/ESI/EDI, ends in plain RET (caller removes arguments), and leaves the output pointer in EAX. There is no x87 return instruction. This is instruction-level inference, not an observed ABI contract for every caller.

The generated C uses x86 Windows cdecl, raw 32-bit word argument for the stack-resident float, distinct point/normal/output/root buffers, and maps the global at its original address. Relocating the full function is intended to preserve internal relative branches while leaving the absolute global operand valid. It uses MXCSR `0x1f80` (all FP exceptions masked, DAZ and FTZ disabled), restores the caller's original MXCSR, and emits post-call flags. It checks EAX, canaries around output, original input buffers, root, and the global pointer. It does not sandbox the process or prove absence of every stray write. A mapping collision fails closed. Host and adapter must be independently reviewed, then linked executables inspected before any run. Compiler output is an unverified hypothesis until reviewed.

The Python comparison makes 314 cases: signed selector boundaries, all pairings of finite values, signed zeros, infinities, subnormals and quiet/signaling NaNs, plus selector-eight player override with matching and nonmatching IDs. It compares the twelve raw final output words. A native double write and one write have the same final output under these non-aliasing inputs, so this does **not** test the Rust `ArbitrationWrites` flags independently. Native point/normal and output may alias; this host deliberately excludes aliasing, which is outside the Rust model. MXCSR exception flags are recorded but not compared to Rust, and masked exceptions do not validate unmasked exceptions or exception traps. It does not test geometry, candidate generation, game reachability, concurrency, or live gameplay.

The generated host and all extracted bytes stay under ignored `target/`. To prepare a fresh directory:

```powershell
python tools/cell_arbitration_native_prepare.py 'D:/Games/SPORE Collection/SporeBin/SporeApp.exe' --out target/cell-arbitration-oracle-review-v2
```

Only after reviewing source and toolchain, from a configured x86 MSVC developer shell, an example build is:

```powershell
cl /nologo /W4 /O2 /arch:SSE2 /Fe:target/cell-arbitration-oracle-review-v2/host.exe target/cell-arbitration-oracle-review-v2/host.c /link /DYNAMICBASE:NO /BASE:0x00400000
rustc --target i686-pc-windows-msvc -O -o target/cell-arbitration-oracle-review-v2/adapter.exe target/cell-arbitration-oracle-review-v2/adapter.rs
```

The compiler may emit intermediates in the shell working directory; run `cl` from within the target directory or direct `/Fo` there. Pin and inspect actual tools and generated executables first. If the Rust i686 target or x86 MSVC environment is missing, stop and report that rather than installing implicitly. Following that review, one local run is:

```powershell
python tools/cell_arbitration_native_compare.py --native target/cell-arbitration-oracle-review-v2/host.exe --rust target/cell-arbitration-oracle-review-v2/adapter.exe --out target/cell-arbitration-oracle-review-v2/results.json
```

The comparison launches one isolated process per case per executable; it returns nonzero on mismatches or child failures and saves detailed inputs, results and MXCSR counts. A pass is finite-case agreement at the stated boundary, not general equivalence. A timeout aborts the run and needs investigation.

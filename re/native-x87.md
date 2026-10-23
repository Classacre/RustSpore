# Explicit x87 arithmetic contexts

`[MEASURED]` Native Cell steering uses retained x87 intermediate values between
binary32 commits. Replacing every intermediate with `f32` loses precision;
replacing it with `f64` also cannot represent every 64-bit-significand result.
Source procedure `E55E40`, for example, stores a square root as binary32 while
retaining the wider value for its comparison. This arithmetic boundary matters
before reconstructing the complete swimming integrator.

`[MEASURED]` `spore-core::native_x87` supplies finite 80-bit storage values,
exact binary32 loads, add/subtract/multiply/divide/square root, retained-value
comparison and binary32 stores. The caller explicitly supplies precision
24/53/64 and one of four rounding directions. A store leaves the retained
value available. Negative zero and subnormal inputs survive exact loads.
Nonfinite inputs/results and unavailable architectures return explicit errors.
The current hardware implementation supports x86 and x86-64.

Each operation saves and restores the caller's x87 environment, balances its
register stack, masks local arithmetic exceptions and leaves MXCSR unchanged.
All eight x87 registers are declared clobbered according to the
[Rust inline-assembly contract](https://doc.rust-lang.org/reference/inline-assembly.html#rules-for-inline-assembly).
The first focused precision test exposed an output-register alias before
environment restoration in the comparison block. Restoration now precedes
SETcc outputs; the corrected implementation passes the independent corpus.

`[MEASURED]` Three focused tests cover retained versus committed comparisons,
sampled exact binary32 round trips and caller control-word restoration.
An independent Python oracle uses exact rational arithmetic and integer
square-root rounding, without using native floating arithmetic for results.
All **2,880 cases**, across three precisions and four rounding modes, agree
on every 80-bit result, binary32 commit and comparison. Cases include
cancellation, rounding ties, extreme inputs and explicit nonfinite store
errors. The fixture SHA256 is
`08430aa2a99e95dbf6ca5484d3bb30c7ececd9496951270b1c6e87d7ee23f5ef`.

```powershell
python tools/native_x87_probe.py --fixtures target/math/native-x87-fixtures-v1.tsv --out target/math/native-x87-python-v1.json
tools\build.bat run -p spore-core --example native_x87_verify -- target/math/native-x87-fixtures-v1.tsv target/math/native-x87-rust-v1.tsv
python tools/native_x87_probe.py --fixtures target/math/native-x87-fixtures-v1.tsv --out target/math/native-x87-compared-v1.json --rust target/math/native-x87-rust-v1.tsv
```

Outputs are newly created; retain existing evidence and choose fresh paths for
a repeated run.

`[UNKNOWN]` The original game's active x87 control word, its creation-time
changes and full mixed SSE/x87 execution context remain unestablished. The
hidden renderer explicitly requests D3DCREATE_FPU_PRESERVE; this does not prove
retail device flags. The helper establishes arithmetic under supplied contexts,
not the retail choice or completion of source steering/gameplay.

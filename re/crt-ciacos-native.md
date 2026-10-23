# Installed `_CIacos` and the native quaternion curve

This note concerns the installed x86 CRT leaf reached by the quaternion
interpolator at `0099C9A0`. It records static evidence, without loading the
DLL, executing original code, or substituting another library's `acos`.
The retail call's floating-point environment remains an explicit input.

## Source identity and the call link

`[VERIFIED]` The installed files
`D:/Games/SPORE Collection/SporeBin/msvcr90.dll` and
`D:/Games/SPORE Collection/SporebinEP1/msvcr90.dll` are byte-identical:

- File version `9.00.21022.8`, 655,872 bytes, x86 PE32.
- SHA-256
  `ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185`.
- Preferred image base `78520000`.
- Export `_CIacos`: ordinal 73, RVA `0007A6E0`, preferred VA `7859A6E0`.
- Export ordinary `acos`: ordinal 1208, RVA `0007A6A0`.

All CRT addresses below are preferred VAs and need the module relocation
delta in a live process. They are not assumed live addresses.

Native REA query 488 (`0099C9A0`) is pinned to base `SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Its call at `0099CA88` targets import thunk `011E06B8`. Static source bytes
at that thunk are `JMP [013CC48C]`; `011E06B8` is not the IAT slot itself.
The imported name is `MSVCR90.dll::_CIacos`.
The caller follows this call with hardware `FSIN` and keeps the angle on
the x87 stack. Its dot product has already been committed to binary32,
clamped and made nonnegative. Its close-to-one branch bypasses `_CIacos`.
These observations constrain the input; they do not establish the
callee's internal result precision.

## Dispatcher and default enablement

`[VERIFIED]` `_CIacos` at `7859A6E0` selects the SSE2 intrinsic wrapper
`785A35E0` only when all three conditions hold:

1. Global `785BD7B0` is nonzero.
2. `(MXCSR & 00001F80) == 00001F80`.
3. `(x87_CW & 007F) == 007F`.

The gates examine exception masks, including the low reserved CW bit.
They do **not** examine the x87 precision or rounding fields, MXCSR
rounding, FTZ or DAZ. Selecting x87 PC24 therefore does not by itself
select the legacy path. The finite SSE2 body does not install its own
MXCSR rounding mode.

The remaining path is `7859A71B` and the shared legacy core `7859A738`.
Ordinary `acos` has the same state gate, but its input is already a stack
double and it enters the corresponding ordinary wrappers.

The file/BSS initializer for `785BD7B0` is zero. CRT initializer array
`78521290..785212A8` is walked by `_initterm_e` (`78542211`) from
`785422BC`. Array entry `785212A0` names `7859A653`, which runs
`785A3563` and stores its result to the enable global. Entry `785212A4`
names `785A35CA`, which runs the same detector and stores capability
global `785BD7B4`.

The detector checks CPUID availability and EDX bit 26 from CPUID leaf 1,
then calls `785A350E`. The latter executes an SSE2 `MOVAPD` inside a
structured exception guard; access violation or illegal instruction
produces a false result. The installed default is consequently to enable
this path when that CPU/OS probe succeeds. This is a recovered startup
policy, not an observation of a running game.

Export `_set_SSE2_enable` at `7859A66C` writes
`(argument != 0) & capability` to the enable global. The base game does
not directly import that export. An indirect runtime override has not
been ruled out by an executed startup trace.

## Finite SSE2 path: binary64 input and result

`[VERIFIED]` Intrinsic wrapper `785A35E0..785A35F7` pops ST0 to a qword
with `FSTP`, loads that qword into XMM0 and calls `785A35FE`. This commits
the input to binary64 using the caller's x87 rounding state; it does not
retain an extended input in the arithmetic kernel. A binary32 input is
exactly representable in this store.

The complete kernel is `785A35FE..785A3B33`. Within finite `[-1, 1]`, all
return blocks store the scalar SSE2 result to a qword and reload that
qword into ST0. The result is therefore binary64 widened back to the x87
format. These are not x87 `FPATAN` results left at extended precision.
The wrapper preserves net x87 stack height by consuming the input and
loading one result.

Let `q = (input_binary64_bits >> 44) & 7FFFF`. Its integer comparisons
divide the finite domain as follows. Sign is handled inside the blocks.

| Absolute input range | Body | Result load |
| --- | --- | --- |
| `[0, 2^-60)` | `785A3AFD`, sum of split pi/2 constants | `785A3B1A` |
| `[2^-60, 1/16)` | `785A3871`, small-argument polynomial | `785A38F9` |
| `[1/16, 0.865234375)` | `785A3650`, indexed table and correction polynomial | `785A371E` |
| `[0.865234375, 0.99951171875)` | `785A3734`, square-root/table reduction and correction | `785A3859` |
| `[0.99951171875, 1)` | `785A390F`, endpoint square-root/polynomial | `785A39FB` |
| Exact `1` or `-1` | `785A3ABD`, sign-selected split zero/pi constants | `785A3AF5` |

Each listed load is `FLD qword` immediately after the result qword store.
The finite bodies use scalar/packed SSE2 double operations, including
`SQRTSD` and `DIVSD`. None uses a generic imported math function. The
domain/NaN error path through `785A8FA1` is outside this bounded finite
contract.

The main table block computes an index using
`((q & FFFF) & ~3) - FB00`. It reads one qword from
`78535770 + index*2` and a qword pair from `78534870 + index*4`.
The square-root/table block uses a different bit-derived index from the
square-root result: `2*(word3(result_bits << 2) - FEC0)` with native
32-bit wrapping arithmetic. Negative table offsets in that block are
valid; extracting only data beginning at the positive main-table base
would omit consumed entries.

The retained conservative constant window is preferred VA
`78534400..78536050` (end exclusive), 7,248 bytes, SHA-256
`76210f1fef0a723d62dbcf92c1a7ce9ce1697b99f6a14530692fa8e3871f07df`.
It includes the indexed prefix, split constants, coefficients and bit
masks. Original bytes remain ignored under `target/source-ui/`.
The static report identifies each scalar/mask by exact 64-bit pattern;
NaN-looking masks must be treated as bit patterns, not floating values.

Under nearest rounding, tiny inputs return the binary64 pi/2 value, and
the exact endpoints return zero and binary64 pi respectively. Different
MXCSR rounding can affect the split-constant addition even though the
state gate still admits that environment. The bit-perfect Rust result
must therefore bind the MXCSR state explicitly or verify its retail
value, rather than infer nearest from SSE selection.

## Finite legacy x87 path and conditional binary64 commit

`[VERIFIED]` At `7859A71E`, `FST qword [ESP]` takes a binary64 input
snapshot **without popping ST0**. `785A3DF8` classifies the snapshot's
exponent. The core continues to operate on the original ST0; it does not
reload this snapshot for ordinary finite arguments. The snapshot itself
can set a sticky precision flag for an extended-only input.

Core `7859A738` saves the caller's CW. If it is not exactly `027F`, helper
`785A3D85` loads temporary CW `(saved_CW & 0300) | 007F`. This preserves
the caller's PC bits, masks exceptions and selects nearest rounding. It
does **not** force PC53. Exact saved CW `027F` is already the desired
nearest/masked PC53 case and does not take the adjustment call.

For `|x| < 1` the instruction sequence constructs `1+x`, `1-x`, their
product, hardware `FSQRT`, then hardware `FPATAN` with `x` as the other
operand. The operation order is thus
`atan2(sqrt((1+x)*(1-x)), x)`, with x87 arithmetic and the selected PC,
rather than a software `acos` implementation. This formula is a
description of the observed stack operations, not permission to replace
their rounding order with a generic formula.

For snapshots exactly equal to `-1` or `+1`, the original input is popped
and `FLDPI` or `FLDZ` supplies the result. Endpoint classification uses
the binary64 snapshot; for an arbitrary extended input, rounding that
snapshot to an endpoint can affect selection. The actual source caller's
binary32 input removes this ambiguity.

Global `785BC614` chooses between a direct return and the normal
precision-return dispatcher. Its file initializer is zero; all 21 direct
literal references in the installed DLL are `CMP [global],0` reads. No
direct writer was found. The global has not been assigned an unsupported
symbol name or assumed to represent a documented CRT setting.

| State after finite core | Return behavior |
| --- | --- |
| `785BC614 != 0` | `785A3E0E`: restore saved CW if needed; retain ST0 result |
| Global zero, saved CW exactly `027F` | `785A3E1B`: retain ST0 result |
| Global zero, other saved CW, original precision mask clear | Restore original CW; retain ST0 result |
| Global zero, other saved CW, original precision mask set, sticky PE clear | Restore original CW; retain ST0 result |
| Global zero, other saved CW, original precision mask set, sticky PE set | `785A3D27`: binary64 store, exception-policy helper, binary64 reload, restore original CW |

`785A3E1B` tests the sticky status flag and does not clear earlier
exceptions. The return decision can therefore depend on prior arithmetic
even when the current endpoint operation is exact. This condition is
specific to the observed `_CIacos` return path; adjacent CRT routines use
other return dispatchers and must not be generalized from it.

The code-8 precision route is closed statically:

- `785A3D27` saves ST0 to qword `[EBP-8]`, then calls `785A951E`.
- Code 8 selects exception flag `10`, the precision flag in that helper's
  internal representation.
- `785A6D2E` sees the original CW precision mask set, generates masked PE
  through `785A692E(20)`, clears its handled flag, and returns success.
  It does not alter the saved result qword.
- `785A951E` restores the saved CW through `785A6900`. For code 8 it
  bypasses user `matherr`; its errno helper `785A6F15(8)` does nothing.
- `785A3D27` reloads the saved qword into ST0 and restores the original CW
  again if needed. The qword commit remains visible to the caller.

The finite code-8 route does not require evaluating a user exception
handler. An unmasked exception or out-of-domain argument can follow
different policies, and is not covered by this finite, masked route.

## Reproducibility and remaining gate

`[VERIFIED]` Static extracts were produced by MSVC SDK `dumpbin` version
`14.51.36260.0` using explicit preferred-VA ranges. Complete relevant
function scopes and hashes are recorded in
`target/source-ui/crt-ciacos-static-v1.json`. The initial short entry
extract and wider support extracts may include boundary bytes belonging
to neighboring code; the report's half-open scopes identify the bodies
used above.

Ignored helper `crt_pe_probe.py` reads PE sections and virtual zero-fill
without mapping or loading the DLL. `crt_ci_acos_static_probe.py` checks
the two installed copies, export-entry/store encodings and initializer
slots; it writes the original constant window and a finite branch audit.
The audit covers both signs and both mantissa tails of every high-20-bit
interval within finite `[-1,1]`: 1,047,554 rows. It validates the integer
branch partition; it is not a floating-point output oracle.

Relevant ignored SDK extracts are `crt-ciacos-{entry,sse-kernel,
x87-helpers,exception,fp-policy}-sdk.asm`, `crt-sse-{policy,cpu-check,
os-check}-sdk.asm` and `crt-init-table-sdk.asm`. Original DLL code and
tables have not been committed as source assets.

`[UNVERIFIED RETAIL STATE]` The original DLL's initialization/dispatch and
the retail process have not been executed. The actual loaded DLL identity,
enable global, entry x87 CW/sticky status, and complete entry MXCSR have
not been captured at the retail quaternion call. Those values select a
measured path. They must not be silently supplied by a generic Rust math
default. The separate reviewed finite-kernel fixture below does not
establish this retail state.

## Bounded Rust implementation and prepared fixture

`[VERIFIED UNDER EXPLICIT SSE2 POLICY]` Module `native_crt_acos`
implements only finite nonnegative input under explicit policy
`Sse2Mxcsr1f80`. It returns the committed binary64 result, accepts signed
zero, and rejects other negative inputs, nonfinite inputs and values above
one. Selecting the separate named `LegacyX87` policy returns an explicit
unsupported-policy error. No retail dispatch policy is inferred.

`ConstantWindow` borrows the exact-length external window. The caller must
verify its DLL or window digest; the dependency-free core checks every
address and does not embed original coefficient/table bytes. The arithmetic
uses hardware `ADDSD`, `SUBSD`, `MULSD`, `DIVSD` and `SQRTSD` under a scoped
MXCSR `1F80`, with each intermediate committed at binary64. Packed lanes
are independent operations; bit operations preserve the observed lane
shuffles and source operation order. Original scratch upper lanes are
not read into output before a defining operation. Tests cover domain and
data bounds plus caller-MXCSR restoration on success and failure; they
have not been run by the implementing worker.

`tools/native_crt_acos_probe.py prepare` reads the pinned DLL and prepares
a no-CRT x86 host under ignored `target/`. It assembles and links the host
but contains no action that executes it. The current fresh preparation
is `target/source-ui/crt-acos-native-v1/manifest.json`:

- Host SHA-256
  `6df1877f3683b8375b59825baa5793d2d8f382f148fd635e91772ff968558ecd`.
- Generated assembly SHA-256
  `864d4302007368780cd9b9362affd0bb90b227156e2f9a25f1de6ddbc689de90`.
- 62,624 inputs, including every high-bit table interval, mantissa tails,
  source-like binary32 dot samples, near-one samples and signed zero.
- Source code allowlist `785A35E0..785A35F8`,
  `785A35FE..785A3A38`, `785A3ABD..785A3B22` (end exclusive).
  Other code remains `INT3`; no original exception or legacy code is copied.
- The only source call is wrapper-to-kernel. All direct branches land on
  allowlisted decoded instruction boundaries. The endpoint's omitted
  fallthrough is unreachable after the host's finite-unit-interval guard.
- Constant reads have a checked direct-address/index-range audit. A
  rational outward bound encloses the square-root table's index interval;
  it includes negative offsets before the main table base.
- Initial writable pages become code `RX` and constants `R` before calls.
  No original DLL is loaded. The owned host's only imports are eight
  allowlisted Kernel32 functions for standard I/O, allocation/protection,
  instruction-cache flush and process exit.
- Every input is evaluated twice, with zero and all-one scratch XMM lanes;
  mismatch, changed control bits, nonzero final x87 TOP, or damaged reply
  canaries causes failure.

The manifest contains complete source and owned-host disassembly, branch
targets, import slots, data ranges and hashes. It marks `host_executed`
false for its preparation step. Root reviewed the complete fresh audit
before authorizing the separate hidden run documented below.
Example `native_crt_acos_verify` reads the external window and input TSV,
evaluates only Rust, and writes result bits. The probe's `compare` action
compares already-produced Rust/native outputs and does not execute a host.

The bounded conformance comparison is closed below. The retail
floating-environment gate, legacy path and full quaternion expression
remain separate work; the finite SSE2 result alone does not establish
completed native quaternion fidelity.

## Reviewed native-only conformance run

`[VERIFIED NATIVE REFERENCE]` Root reviewed all copied spans, endpoint
fallthrough, indexed/direct memory bounds, owned host code/imports,
RX/R protections, input guards, partial EOF and reply/scratch checks.
`tools/native_crt_acos_run.py` then executed only the exact reviewed host
SHA through hidden binary pipes (`CREATE_NO_WINDOW`, show-window zero).
It rechecked executable, assembly, constant/input and original DLL hashes
before and after. It did not load the original DLL or launch the retail
game and used no GUI.

The main fresh stream contains 62,624 cases and 125,248 original wrapper
calls (zero/all-one scratch states per case). All six branches executed:
37 tiny, 43,555 small, 9,457 table, 1,272 square-root/table, 8,302 near-one,
and one exact-one case. All 62,624 scratch-output equality checks and
500,992 compiled canary DWORD comparisons passed; exit was zero and
stderr empty. The expected 32 canary bytes have SHA-256
`fc8b64001c5fdd0f2f40fb67dae4a865a2c5bd17836676d6d5b58b7917e33717`.
Canary bytes are checked internally rather than transmitted as replies;
successful exit is the evidence that every equality comparison passed.

All output CW fields were `027F` and all x87 TOP fields zero. MXCSR was
`1FA0` for 62,623 cases and `1F80` solely for input `+1`, whose result
was zero. x87 SW was `0000` for 62,606 cases and `0002` for exactly the
18 binary64-subnormal input rows. That denormal flag originates from the
fixture's x87 input load; it does not establish the retail caller state.

Retained `target/source-ui/crt-acos-native-v1-run/` identities:

| Evidence | SHA-256 |
| --- | --- |
| Complete 1,001,984-byte native reply stream | `52ae7b90295d70678c63740dff124e3d8a398403440c8614e0ce663081db10d3` |
| Binary64 result bits only | `6ee635b5eee6232602ef35e4f5a971c77b48e160d9f16c0b72e1bead2088921a` |
| Context bytes only | `1a5dcfe3aa8a562a1c8bb25514e5696de967b570f39b5dd507ccc02dc7f08b6a` |
| Input/branch/result TSV | `99081a5ed5960b9eb0a5de4bed1bc5f3b4cf3ad048b901e0061fc2f9030065f7` |
| Protocol-check report | `0add789dbfa25843ba90a069b162cbf1c92411dd633258a2c281ba572e229fd1` |

All 26 protocol checks passed, including 25 explicit rejections:
one- through seven-byte partial records, complete records followed by
partial input, above-one input, nonfinite/signed-domain inputs and invalid
records before/after a complete record. Complete prefixes returned the
exact smoke reply before the deliberate failure. This includes two smoke
calls and 16 original calls from valid protocol prefixes; total original
calls across all 28 hidden processes were 125,266. `inventory.json` retains
main-run evidence and `context-checks.json` retains the independent
row/context and total-call accounting.

Root then ran the independent Rust example over all 62,624 inputs. The
probe's separate `compare` action verified every input/branch/result row
byte for byte: both native expectations and root's `rust.tsv` have
SHA-256 `99081a5ed5960b9eb0a5de4bed1bc5f3b4cf3ad048b901e0061fc2f9030065f7`.
The retained `comparison.json` records `exact_match: true`. This closes
the bounded nonnegative SSE2/MXCSR1F80 arithmetic comparison. It does not
select the retail CRT dispatcher, establish a retail floating environment,
implement the legacy path, or validate the larger quaternion operation.

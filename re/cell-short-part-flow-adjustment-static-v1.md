# Cell short-part flow adjustment: native input and operation boundary

Static x86 inspection of `D:\Games\SPORE Collection\SporeBin\SporeApp.exe`, SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. This extends [the position branch trace](cell-movement-accumulator-position-static-v1.md) and [the component lifecycle trace](cell-player-movement-component-lifecycle-static-v1.md). It neither observes a live player nor proves current field values, call reachability, an original frame, or a playable update.

## Admission and sampler

At `E7C0B0..E7C118`, after copying old actor `+4C/+50/+54` into `+84/+88/+8C`, the path reaches `E7C213` with actor `+248` nonzero and **signed** `+250 <= 1`; definition `+B4` must be nonzero. The initial copy occurs before these gates. `E7C224..E7C22F` sets actor `+98` and `+1CC` to binary32 positive zero. The retained nonzero handle then enters `E7C23F..E7C251`, passing the original actor-position address `+4C`, environment pointer `[016B4BE4]+4108`, and a disjoint three-word output to `E4E220`. These claims assume valid pointers and nonaliasing of the actor, environment and local output. Counts <= 1 include negative values at the instruction gate; this is not a claim that a negative count is a valid constructed actor state.

`E4E220` calls `E4DB80/E4DFD0`, then reads weighted entries from environment `+1C210`; `cell_flow_field::sample_flow_field` already implements the finite 20-by-15 sampling arithmetic with explicit point XY, runtime frame origin `016B4C68/6C`, scale `015A8B9C/BA0`, and row-major vectors. Its declared SSE nearest/gradual-underflow policy is an admission, not a live control-word measurement. Field construction, producer ordering, array identity, runtime stability, and the current frame remain external inputs. The native call produces three words, but this particular consumer loads only sampled X and Y: it does **not** use sampled Z in the following adjustment. The sampler may still read and compute Z, so its validity and any exceptions cannot be omitted from an exact whole-call model.

## Arithmetic after the sample

`E7C256..E7C2A4` loads the original scalar as binary32 into x87, negates it, loads `FLDL2E`, multiplies, duplicates and rounds with `FRNDINT`, subtracts the rounded integer, then uses `F2XM1`, `FLD1/FADDP`, `FSCALE`, and the x87 stack pop sequence. It loads one, executes `FSUBRP`, and commits the factor with `FSTP` binary32. In real-number notation the intended shape is `1 - exp(-scalar)`; this is **not** a bit-exact replacement for the recorded x87 instructions. x87 precision, rounding/control state, transcendental implementation and exceptional values affect this trace. The commit at `E7C2A4` writes to the caller's original actor-argument stack slot after the actor pointer has been retained in ESI. An exact harness must account for this stack write, stack depth/status and applicable exception policy.

The SSE scalar sequence immediately around that x87 calculation is:

1. After the explicit positive-zero store, load actor `+98` and XOR with the source sign mask at `013EB8B0` (four words `80000000`), producing negative zero in the finite ordinary case. Form `sample_x - actor_90` and `sample_y - actor_94` with each sampled word as `SUBSS` destination.
2. Multiply these X/Y differences and the sign-flipped Z by the committed factor, each as `MULSS` destination. At `E7C2B6`, X adds old `+90` into the product; at `E7C2C2..CE`, Y adds its product into old `+94`; at `E7C2CE..D6`, Z adds its product into current `+98`. Store X, Y, Z to `+90/+94/+98` at `E7C2DA..EA`.
3. At `E7C2F2..E7C350`, independently multiply actor `+1C4/+1C8/+1CC` by the same original scalar and add the three products to the just-written accumulator, using each accumulator word as the `ADDSS` destination. `+1CC` was explicitly zeroed, but signed zero, negative scalar and nonfinite intermediates still matter.
4. `E7C358..E7C3A7` multiplies each updated accumulator by the scalar. X and Y add old position into the product; Z stores its product alone. It writes all three position words, ORs word `+48` with `0004h`, and increments the word at `+4A` modulo 65536 (`E7C3AA..B3`). Later callbacks and the separately gated `+1C4` reset are outside this step.

A checked source-backed step therefore needs a valid explicit branch admission, the old position/accumulator/vector and two words, scalar, sampled X/Y (or all sampler inputs and complete sampler behavior), explicit SSE and x87 control/exception policies, and finite/intermediate bounds. Inputs and operations discarded by the final value can still affect exceptions or observable writes. A supplied sample or synthetic field supports a controlled fixture only; actor initialization and source field/producer provenance are separate. Broader conformance tests should exercise the original-instruction x87 sequence and exact SSE operands rather than compare against an algebraically equivalent library exponential.

## Bounded implementation and a fixed instruction comparison

`cell_movement_short_part_flow_v1` implements the selected short-part branch
with a caller-supplied completed sample and explicit FP controls. It is limited
to x86-64, returns only checked finite results, and does not expose native FP
sticky flags, partial writes, the overwritten stack argument, sampler behavior
or callbacks. The implementation uses an isolated x87 stack and restores an
FXSAVE64 image. Abridged tags are reconstructed, and on some older AMD64 CPUs
the saved image can omit x87 instruction/data pointers and opcode when ES is
clear; those diagnostic fields cannot be promised unchanged. Native parity
also requires sufficient stack capacity in the original call. See the
[AMD64 architecture manual](https://docs.amd.com/api/khub/documents/sfvvekC9mDflu6vd3R0NXA/content)
for the processor-dependent FXSAVE/FXRSTOR behavior.

One reviewed hidden 32-bit host ran the original `E7C256..E7C3B3` bytes on a
fixed synthetic actor and post-sampler inputs, with x87 CW `027F` and MXCSR
control `1F80`. Its receipt is
`target/flow-short-native-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `cb1d54892fbefd60d2516ba10db918b1a2e07877e4d52ed4eb794c0accb61fd2`.
`cell_short_flow_compare_v1` checks that exact receipt and matches 12 actor
float words and the flag/counter words against the Rust model. The host supplied
previous position and vector Z before the copied instructions; these checks do
not validate their earlier native writes. It also skipped the sampler and branch
gates. The reference used an AMD Ryzen 9 5900HS as reported by the local process,
and x87 transcendental results may be hardware-dependent. Source and executable
hashes were checked at endpoints, not continuously; the host mappings had no
guard pages and complete compiler/build provenance was not established. This
single fixture is not a live player update or a general parity claim.

## Reproduction

`target/flow-short-branch-review-20261105.txt` retains exact bytes and Capstone 5 x86-32 instructions for `E4E220..E4E3E5` and `E7C0B0..E7C3B3` from the hash-checked PE. Reproduce with Python `pefile` and `capstone`, mapping virtual addresses as `pe.get_data(start - 0x400000, end - start)` and decoding instruction-aligned starts; end addresses are exclusive. The sign-mask bytes can be read at `013EB8B0`. No Cargo, GUI, original game execution, or native conformance host was used.

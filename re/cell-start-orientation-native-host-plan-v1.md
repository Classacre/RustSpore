# Guarded original-byte orientation comparison plan

2026-10-30. This is a **source-only design**, not a prepared, linked,
reviewed or executed host. It concerns the conditional null-orientation
arithmetic in `cell-start-null-orientation-static-v1.md`, not actor allocation,
matrix creation, live RNG initialization, or a complete Cell constructor.
Do not execute a host until its source, actual linked image, exact runner and
guard/ABI behavior receive independent safety and instruction review.

## Exact executable boundary

Target base executable `D:\Games\SPORE Collection\SporeBin\SporeApp.exe`,
SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Fresh read-only Capstone decoding gives:

| Span, end exclusive | Bytes | SHA-256 | Role |
| --- | ---: | --- | --- |
| `00E74B98..00E74C10` | 120 | `307b498427dd42946221a6104705e505985c31bb66e812e6dd8f935d51f2d4f4` | Original RNG call and complete four-word orientation write |
| `00E74BA2..00E74C10` | 110 | `2ddfea385522dd7b04f650c26cd958346368d5f111a220384597c3622f11614a` | Arithmetic alone, entry with RNG result already in ST0 |
| `00935D40..00935D76` | 54 | `35d81765a9410a57d5516506680395d783bffbc0d2ff6cc20e1b74aa1211afce` | Original RNG, including its excluded zero-state RDTSC branch |

The common shorthand `E74BA2..E74C0D` must be treated as inclusive: the
three-byte `MOV [EAX+0C],ECX` begins at `E74C0D`. An exclusive exit at
`E74C0D` omits the fourth output. Place an owned exit/trap at `E74C10`,
outside the retained source span, and prove its exact entry and placement.

Use the first span for the stronger proposed gate: its original relative call
at `E74B9D` resolves to original `00935D40` when both are mapped at their
original preferred virtual addresses. Do not copy it elsewhere without
relocation proof, and do not replace that call with a host RNG while calling
the result original RNG execution. The arithmetic-only span is a separate
weaker projection with a supplied ST0 and no RNG claim.

The `935D40` initial branch reads the writable state word at `01602760`;
reject seed zero before source entry, making `RDTSC` unreachable. Include
`935D40`'s exact original bytes and both of its absolute binary32 inputs:
`0143F52C = 0000802F` (2^-32) and `01471B2C = 0000003F` (0.5), in hex byte
order. Confirm both absolute addresses, decoded transfers, constant bytes and
source PE hash in the preparer and again in a linked-image audit. Preserve the
original RNG's `push ecx` scratch and return-address stack behavior.

`E74BA2` reads writable runtime global `016B4DB0`; it is zero-fill in the
image, not an extractable initialized source float. For the first fixture,
explicitly initialize it from the observed source initializer input at
`015A8BB8`, bytes `DB0FC940`; record that this is an admission, not evidence
of the global at a live draw. Supply axis words at `015A8C40..015A8C4B`
explicitly: installed initial bytes `00000000 00000000 0000803F`. Their
live values are also not established. The actor pointer is not derived from
the installed resource or native allocator in this host.

## Entry ABI and memory design

Enter `E74B98` with an empty x87 stack, `EBP` pointing to a bounded synthetic
actor buffer, `ESP=S` pointing into a bounded stack with adequate call depth,
DF clear, and a supplied nonzero RNG word. The source sets ECX to the fixed
RNG address, calls the original RNG, and returns with exactly one x87 value;
subsequent arithmetic ends with an empty x87 stack. Preserve host ESP, EBP,
EBX, ESI and EDI independently, and check actual output registers/stack rather
than assuming a normal function ABI at this partial entry. `LEA EAX,[EBP+14]`
at `E74BA8` addresses the synthetic actor. Source writes actor bytes
`+14..+23` and stack scratch `S+18..S+27`; the original RNG's CALL and PUSH
also touch the stack immediately below S. Check exact byte masks and both
canaries, not only the four returned words.

Reserve all original source and absolute-data addresses explicitly, reject
any unavailable/colliding page instead of relocating silently, and verify
all regions lie in the 32-bit child process. Map owned code separately and
copy source code byte-for-byte at the original VAs. Set code pages RX after
construction and flush the instruction cache; make fixed constants/axis pages
read-only after initialization and only RNG/global-angle/actor/stack buffers
writable. Never leave a page writable and executable. Place no-access guard
pages around actor and stack allocations, and protect unrelated memory. The
runtime angle, RNG word and any writable controls need precise independent
shadows, bounded access and guard checks. Because several addresses share
pages, audit real page granularity, page overlap and permissions; do not assert
that a subrange of a shared writable page is independently read-only.

Use a deterministic record protocol in a hidden, separately spawned 32-bit
child with a bounded timeout, exact reads/writes, size caps, one attempt per
case and retained exit/status/stderr. Reject malformed input, zero RNG and
unsupported control words before entering source bytes. Preserve and restore
the incoming host x87 and SSE environment, including x87 registers/status and
MXCSR, using an independently audited aligned save/restore mechanism. Supply
masked exceptions, explicit PC53/PC64 and all four x87 rounding modes, and a
specified MXCSR (including rounding, DAZ and FTZ). Check x87 TOP, control,
stack depth and allowed status bits, and check MXCSR controls rather than
assuming defaults. Status flags are observable separately; do not claim the
fixture recreates unknown retail flags or hardware behavior.

## Transcendental observation and comparisons

The full unchanged run records pre/post RNG word, final four actor words,
stack/actor write masks, FP control and guard status. Bound the supplied angle
and RNG to a finite domain in which `FSIN`/`FCOS` are intended to operate;
with the stated positive fixed angle and admitted RNG this is approximately
0..pi, but verify the actual retained x87 value rather than relying only on a
rounded f32 angle. `FSIN` and `FCOS` are hardware transcendental operations;
do not claim bit identity with a generic math library or across CPUs.

C2 must be observed **immediately after each transcendental instruction**;
one final status word cannot independently establish both. A reviewable
approach is separate checkpoint runs from identical admitted inputs: one
unchanged prefix ending immediately after `FSIN` (`E74BB5` exclusive), and
one ending immediately after `FCOS` (`E74BC4` exclusive), each with a
separately owned exit and correctly preserved stack/x87 state. The owned
checkpoint can record status before further x87 instructions; it must be
clearly labeled as instrumentation and cannot be substituted for the full
unchanged output run. Audit exact prefix bytes and exit behavior separately,
including source bytes displaced by an exit trampoline. Do not silently patch
an instruction inside the full-output span. The full run's final scratch
`S+18` is overwritten by the x-component multiply, so it is not the original
stored sine; any sine-store claim needs its own exact checkpoint.

Compare native outputs against an independently prepared model only after
review: one draw with known state, x87 multiply/FSIN/store and retained-angle
FCOS/store, then the three ordered `MULSS` outputs. Include signed-zero axis
cases and different controlled x87/MXCSR rounding policies only when each has
an explicit domain and oracle; report unsupported environments and divergence
rather than silently normalizing. A Python transcendental calculation may be
useful as a sanity bound but is not a bit oracle for original x87 instructions.
Test exact protocol failures, guards and repeated poison patterns before any
corpus execution. Original size sampling, pool allocation, branch reachability,
the following 3×3 matrix and live actor behavior are separate gates.

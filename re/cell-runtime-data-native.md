# Cell runtime animation data

The complete native reader `E833B0` and aggregate writers `99C170/99C210`
now pass original-byte comparison. Full-channel kind3 curve sampling also
passes original-byte comparison across all authored `data1..data4` components.
These are ANIM components, not a measured PROP association. Their caller blend
contribution and live actor/pose binding remain separate gates.

All addresses below refer to `SporeApp.exe`, SHA256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
REA dossiers are retained in `target/rea/20261022-source-runtime-v3/`.
Original bytes, source payloads, generated hosts and raw results stay ignored
under `target/`; the installed game remains immutable.

## Reader and consumers

`E51760` (query848) gates on actor words `248` and `FC`, resolves the handle
through manager `[16B4BE8]+168` and `B71D00`, then returns object word `24`.
The identity/construction of that returned external object and its `17C`
pointer remains open.

`E833B0` (query849) follows argument `+17C`, pointee `+2E4`, then the first
pose's aggregate at `+A0`. The count is signed; nonpositive counts read no
rows. Rows begin at `+A4`, each20 bytes, in original order.

| Row offset | Measured field |
| --- | --- |
| `0` | ID, first match wins |
| `4` | Raw producer kind, overwritten on each accepted write |
| `8` | Accumulated scalar/first numerator |
| `C` | Second pair accumulator; scalar writer preserves it |
| `10` | Accumulated weight/reader denominator |

Found reads execute `FLD32 numerator`, `FLD32 denominator`, `FADD64` with
exact binary64 bytes `3a8c30e28e79453e` at `1447BB8`, then retained x87
division and one final `FSTP32`. The epsilon is not prematurely converted to
binary32. Missing IDs write positive zero and return `AL=0`; found returns
`AL=1`. Incidental upper EAX bits are not a canonical Boolean.

| Consumer | ID | Source use |
| --- | --- | --- |
| `E56440` | `98C942B6` (`data1`) | Flow input; ignores AL, so missing positive zero enters its original math |
| `E563F0` | `98C942B5` (`data2`) | Outside actor18C states30/12/14/13; missing object/value returns one |
| `E560E0` | `98C942B4` (`data3`) | Actor1B0 states12/13/14/30; `MULSS100`, store actor1C0; missing data uses Globals108 |
| `E561E0` | `98C942B4` | Same state family plus object54 gate; `MULSS20`, store actor1C0; distinct Globals104/`E50200(2)` branches |
| Bounded callers `E6091F/E60B18/E60BB8` | `98C942B3` (`data4`) | Containing procedures remain undefined; no gameplay meaning inferred |

Queries365/367/385/386/850 retain these consumer branches. Their arithmetic
does not establish the live producer's clock, blend weights or root pose.

## Original sampled-value producers

`99C170` (query869) and `99C210` (query870) receive the count-header pointer
in ECX. Both scan the first matching ID. Existing IDs update even at capacity;
new IDs append only while unsigned count is below8. Full capacity rejects a
new ID with AL0 and no writes. Caller counts outside0..8 are an unsupported
producer domain in the bounded Rust snapshot API.

Appends copy raw binary32 bits with `MOVSS`, including signaling NaNs.
Updates execute incoming-left `ADDSS(new,old)` for each accumulated field.
The scalar writer leaves rowC unchanged on both paths; the pair writer also
accumulates C. Both overwrite raw kind4. No generic sanitization, alternate
NaN operand order or fabricated initial second value is added.

Complete `99D190` (query873, four source body ranges) supplies the actual
sampled-animation path:

1. It selects a sampled channel wrapper at `param3[5]+binding[0D]*1F0`.
   Wrapper0 points at CHAN; its descriptors are CHAN+E0/countDC.
2. The destination pose is `[owner+2E4]+binding[0C]*2BC`; the aggregate
   passed to the writers is that pose+A0.
3. Selected kind3 descriptors provide their raw tag at4 and prepared slot10.
   Sampled scalar/weight are wrapper`CC/D0+40*(slot-3)`.
4. Scalar flags40clear execute `MULSS(sampleWeight,callerBlend)` and then
   `MULSS(sampleValue,scaledWeight)`, passing raw kind `(flags>>7)&1` to
   `99C170`. All shipped data1..4 descriptors use this path.

The angular flags40 path computes a sine/cosine pair and calls `99C210`.
The pair writer is conformant, but this caller's FSIN/FCOS preparation is not
claimed by that writer gate. Also, the caller's outer `COMISS/JNC` blend gate
must not be replaced by a guessed ordered `blend>0` rule for NaN inputs.

`9A1590` (query421) calls `99F6A0` with output at sampled wrapper+4. This
four-byte displacement closes the source-slot-to-wrapper offsets: data1
`F4/F8`, data2 `11C/120`, data3 `144/148`, data4 `16C/170`.

## Full authored source census

The canonical index identity is `(low,type,scope)`, recovered from the native
archive reader; historical group/combined-instance labels are superseded.
The census reads all82 package indexes/158,720 resources, including all5,022
ANIM resources, 48,700 channels and173,204 kind3 descriptors. The three source
PCTP copies have identical deformation layout SHA256
`2f353855091d6c835a3a95c22e8def218bfe3d5cf614eaf74d13597bad3cb221`.

| Tag | Components | Channels | Clips | Source rows | Prepared slot |
| --- | ---: | ---: | ---: | ---: | ---: |
| data1 | 797 | 797 | 785 | 16,624 | 4 |
| data2 | 797 | 797 | 785 | 16,584 | 5 |
| data3 | 489 | 489 | 485 | 10,906 | 6 |
| data4 | 3 | 3 | 3 | 13 | 7 |

The union is2,086 components in818 clips, with44,127 rows. Every component
matches the source root PCTP layout and retains prepared flags93/B3.
All2,086 primary selectors satisfy literal `flags&100003==1`; their capability,
alternate and reserved selector words are zero. Primary flags1/5 and channel
flags0/1/10 remain retained. Secondary selectors are `(2,0,0,0)`.
This proves authored root-mode selection, not that runtime binding0C is zero.

Accepted census artifacts are
`target/source-cell/runtime-data-source-census-v2/report.json`,
`prepared-data-components.tsv`, `selectors-v1.json` and `caller-proof-v1.json`.
The earlier v1 endpoint diagnostic read the two source words at4/8; v2
corrects those diagnostics to0/4. The prior census totals/slots/flags were
unchanged, and no native comparison used those superseded diagnostics.

Exact INFO20/custom40 rows are retained in
`target/source-cell/runtime-data-source-curves-v1/`.
`source-INFO20-custom40-rows.bin` SHA256 is
`44bcedd0fddb19ab681e89841581c11a81351097f3e0542b38c77f55cb5c416e`;
`components.tsv` SHA256 is
`d33ca484bad7fdc7d8c23c4d63064f1613f01afe5a54d3cf09b3ce9c28662837`.
Custom rows store scalar/weight at0/4, scalar slope/control block at8 and
weight slope/control block at18hex. All six float fields in all source rows
are finite. Remaining row bytes stay opaque.

## Accepted reader/writer comparison

`cell_runtime_data` and `cell_runtime_data_verify` pass129,331 original-byte
requests. Of these,88,254 inject each of the44,127 source rows into an explicit
blend1 scalar append and reader lookup at CW027F/MXCSR1F80; they do not sample
an actual clip. The remaining41,077 cases cover all12 masked x87 PC/RC policies,
counts/order/duplicates/full capacity, signed nonpositive reader counts,
finite extremes, raw infinities/NaNs and synthetic snapshots.

Rust matches all172 modeled reply bytes in125,319 requests: AL, committed
reader result and the complete164-byte aggregate including untouched bytes.
The remaining4,012 cases positively check the finite reader API's typed
errors:3,840 reached nonfinite operands and172 nonfinite final FST32 results.
Their raw native outcomes remain retained. Raw writer nonfinite inputs and
results are compared exactly; they are not exclusions. Every modeled/error
call also preserves the caller's complete MXCSR under three hostile incoming
states. Eight native context bytes are independently checked. All18 protocol
checks pass, and all source/host/input pins remain unchanged.

The finite/error policy does not replace a game result with a fallback. The
native reader can produce nonfinite output; this finite checked API reports
its reached unsupported arithmetic explicitly.

Accepted comparison:
`target/source-cell/runtime-data-fixture-v1/rust-comparison-v1.json`.
Rust/native modeled projections share SHA256
`2a28d79022ca2d38723c97551d8207e2bf7a09a67b5775ebcd86d5c48129e5b6`;
full180-byte native replies have SHA256
`7ebd7f29993c8d86f3c46102bea627817a642f54dce1c14c0455e3e52fae7292`.
Input SHA256 is
`d69b88d734011030d8ab1a1686d0c796cab70a7a57d3db855c4589b81360e0e6`.

The reviewed host contains only428 original bytes in three complete functions
and the one eight-byte epsilon. It has no original calls or replacement
callbacks. Root independently reproduced/compiled/audited it and performed
hidden file-backed execution. Source functions execute in RX pages, data in
R pages, and six guarded buffers supply all pointer-chain state. Stack,
callee-saved registers, DF, CW/TOP/MXCSR, immutability and permitted writes
are checked before output serialization. No game initializer, retail process,
GUI or network operation ran. Host EXE SHA256 is
`0fe85f1c282f5f977613c35c8c41e9242c478cc9949d83e1d1bfd3ff8d020294`;
assembly SHA256 is
`40f7acb855b0a9a0de70c9d77dcd8188767089127836d7cf0b60b4663a293c82`.
Tools are `cell_runtime_data_native_probe.py` and
`cell_runtime_data_native_run.py`; the runner defaults to preparation only.

## Pose ownership and remaining gates

`9C5550` (query867) initializes the animation owner's pose-vector fields
`2E4/2E8/2EC`. `9C5130` (query872) binds the source definition, allocates one
700-byte/2BC pose per468-byte body-definition block at definition384, and
initializes poseA0/144 counts to zero. Source factory9ABB20 allocates a0x1930-byte
object with the literal label `Anim/bam/new_cid`, then calls these constructors.
`9CB2F0/9CA920` (queries861/862) bind separate AC-byte animation cache entries.
The retained matrix-feed9C6E40 reads the same measured700-byte pose layout.

The external E51760 object24→17C chain must still be tied to this owner class
and live runtime construction. Source root selectors must still be bound to
actual pose indices. Active clip selection, event dispatch, clock, blend
weights, pose reset/order and the complete actor/controller integrator are
not established by the aggregate kernel comparison.

## Accepted full-channel custom curve comparison

Exported `cell_runtime_curves` now matches the complete native9A14C0 upper
selection,99C6C0 neighbors and99F6A0 kind3 scalar/weight branch. The two outputs
use asymmetric SSE/x87 auto tangents and left-first endpoint products.
Flags20clear clamps both outputs; endpoint copies preserve40 bytes and bypass
interpolation enable/clamp. Event arguments are explicitly null; this gate
does not dispatch animation events.

The full source schedule contains50,367 cases for all2,086 components in818
clips, independently reparsed from the installed packages and prepared with
all three original PCTP dictionaries. Every request preserves the full original
channel, including1,016 components/33,221 rows with more than16 keys. The
largest source channel has133 keys; the reviewed host bounds requests at160.
The schedule covers before/after, representable first/last frames and every
one of42,041 distinct-span midpoints. No source duplicate/nonmonotonic span or
unrepresentable midpoint is excluded. Twelve exact-time witnesses retain
the six single-row channels whose FFFFFFFF uint time cannot be represented by
binary32; their before/after native upper results are checked in24 cases.

The accepted84,544-request cohort includes those50,367 source requests and
34,177 synthetic cases across all12 masked x87 PC/RC policies. Rust matches
all468 modeled reply bytes in84,256 cases: complete output storage, endpoint
payload/untouched bytes, AL/exact-time result, native upper and all four
neighbors. The288 reached nonfinite-input cases positively match the finite
API's `NonFiniteInput` errors; their raw native outcomes remain retained.
Unused nonfinite payloads are compared rather than silently excluded. Every
modeled/error call restores all bits of three hostile incoming MXCSR states.
Eight native FP metadata bytes are checked separately, all23 protocol checks
pass, and all32 native-run pins remain unchanged.

Accepted comparison:
`target/source-cell/runtime-curves-native-v1/rust-comparison-v1.json`.
Rust/native468-byte projections share SHA256
`afd603bacfd6447e9903f0592829d6b83fe335e3a464619cefe8f7d34a77ef6a`;
raw476-byte native replies have SHA256
`69d4c943387093e466be1d429ee32ab0ccb4fc825fcd7bd28062863e5db7f6d7`.
The814,327,808-byte fixed request stream has SHA256
`abf62621d25227958e06e6331aab7024d49438bb2284ad4aa60cd4bbee1ebbe6`.
The runner references this immutable file directly, streams output and retains
every failure; only empty pre-input allocation failures can retry.

The unchanged reviewed v3 packet is
`target/source-cell/native-runtime-curves-host-v3/`. Root independently
reproduced/compiled it in `native-runtime-curves-host-root-reproduction-v1/`,
then executed the hidden bounded host. Its EXE SHA256 is
`8a33668111028b95f389607fd714eba65ca6e094f3588db488ee65a100af1c40`;
ASM SHA256 is
`c21135e6b7cedc4a182b032c753afc634be1a2e781d4a3da403bcdd2305dd013`.
Original source spans/constants, copied RX/R pages, event-unreachable branch,
copy-provider lengths, stack/register/FP state, all seven guarded buffers and
unwritten output bytes were reviewed before execution. No retail process or
GUI ran. Earlier16-row and supplied-upper packets remain separate, prepared
and unexecuted; the accepted source comparison uses complete channels and
native frame-to-upper selection.

The next caller gate is99D190's original scalar contribution:
`MULSS(weight, explicitBlend)`, then `MULSS(value, weightedWeight)`, followed by
the already measured99C170 accumulation. Its descriptor mask/order and outer
blend gate must be preserved before composing it with the accepted sampler.
Actual selected clips, advancing clock, runtime blend policy and pose binding
remain explicit inputs until their own source chains are recovered.

All explicit kernel floating contexts remain distinct from the original
entry/thread policy. See [the rendering-context evidence](native-float-render-context.md)
before connecting these helpers to live game state.

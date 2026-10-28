# Source root binding to visible pose

## Accepted coordinate and accumulation boundary

[MEASURED] Original coordinate conversion and zero-state POS/ROT accumulation
now match Rust on22,212 cases:3,702 genuine source sample pairs from215 clips,
three explicit reference contexts and both retained binding passes. All156
modeled bytes agree. Native42 protocol checks,14 further Rust/native protocol
cases across12 x87 policies, three incoming MXCSR states and33 typed-error
checks pass. All41 execution pins remain unchanged. The public API is
`spore_core::anim_root_pose::evaluate`; this accepts an explicit state record.

Projection SHA256:
`f23ed6aa20da3bd15d2cbc32cd1695453ec173db487786c0bc01c23f826018ac`.
Acceptance:
`target/animation/root-pose-sparse-carrier-native-run-v3/rust-comparison-v1.json`.
The root record/reference, actor route and positive global blend remain supplied.
This gate does not execute staged normalization, child constraints, final pose
commit or a visible animated creature.

[MEASURED] The first runtime-mapping fixture exhausted eight empty pre-input
allocation exits. Diagnostics preserved those failures. A sparse primary PE
then failed process creation with WinError193; no source ran in that attempt.
Independent review identified nonadjacent sections. A new v3 carrier adds five
uninitialized, zero-raw, no-RWX gap sections, preserving every source/owned
instruction and absolute VA. It passes independent source and linked reviews,
loader smoke, all native protocols and the corpus on single attempts. Microsoft
requires adjacent section VAs in its [PE section-table specification](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#section-table-section-headers).
Rejected versions remain retained. Proprietary bytes stay in ignored `target/`.

## Remaining bridge

[MEASURED] The accepted constructor is a record boundary. The static first-root
retention report covers217 of818 data-bearing clips,512 data descriptors and434
two-pass records. Every one of the818 clips also has enabled non-root channels.
The count of entirely modeled root-only clip builders iszero.

[MEASURED] Within the217 subset,217 enabled primary selectors use ordinary root
mode1, and1,644 enabled primary selectors use capability mode0. Those1,644
channels cannot be deleted from a full builder test against real generated
bodies: their capability selections can produce records and affect claims. A
body containing only a synthetic ROOT block would change that test's meaning.
The smallest honest full-builder gate must retain their complete descriptors,
order and metadata and run native9B2150 against an explicit finalized body.
Its mode0 capability/class/spatial filters and callees remain outside current
constructor acceptance. No full9A1E00 host is proposed as already admitted.

[MEASURED] A more useful bounded next gate is the original root-channel consumer:
accepted POS/ROT samples, an admitted root record, native coordinate conversion,
native weighted pose accumulation, staged-pose evaluation and final palette
assembly. Source evidence is q873 and20261027-source-runtime-v4 q02/18/19/20/22.
Complete source bytes/disassembly remain in ignored `target/`.

| Boundary | Exact source | Supported facts and remaining work |
| --- | --- | --- |
| Record association | 99D3E7..99D43E | Binding byte0D selects wrapper stride1F0; byte0C selects pose stride2BC; FF secondary becomes explicit -1. Root0 is distinct from retained palette ID. |
| Coordinate conversion | 9B0CA0..9B1B81 | Full3,810-byte operation. Explicit wrapper samples, actor/root reference, finalized rest block, scale and mirror input; outputs translation/quaternion. |
| Root translation scaling | 9B0B90..9B0C90 | POS71 scales by definition+364/368/36C and owner scale; B1 uses size434 or374 after the original threshold test;31 takes the unchanged path. |
| Quaternion composition | 99BD80,235 bytes | Reached by source ROT32 after raw quaternion/mirror copy; preserve original multiplication order and signed-zero behavior. Boundary is in q18 callee metadata. |
| Position accumulation | 99D7E5..99D918 | Wrapper POS weight times blend controls admission. Ordinary sums/weight land at pose+58/+64; alternate sums/weight at+7C/+88. Preserve instruction order. |
| Rotation accumulation | 99DA94..99DBCF | Original hemisphere-aware add/sub helper99C8D0 and FLD32/FSTP32 copy5992B0; sums/weight at+68/+78 or+8C/+9C. |
| Staged pose evaluation | 9F9C10..9F9D2F | Called from9AA184 before final commit. Dispatches9F4730 or cached9EAC30, then9F3460 and9F3BC0. Source entries are retained for the uncached chain; runtime/model acceptance remains open. |
| Final pose commit | 9BD370..9BD502 | Copies staged+190..1A8 to final+10..28, computes velocity and blends+3C translation. It does not normalize weighted accumulators. |
| Palette | accepted9C6E40 | Consumes final position+10/quaternion+1C and retained source palette IDs. This must follow staged pose evaluation and commit. |

[MEASURED] All217 first channels have CHAN+88 equal0 (160 clips) or10hex (57).
POS flags are71 (190), B1 (24),31 (3); ROT flags are32 (217). Thus bit0-clear
channels bypass9B0530/9B05E0 relative-secondary branches in9B0CA0. Their position
path reaches9B0B90; their rotation path reaches99BD80. This makes a coherent
source-driven transform/accumulation fixture feasible without implementing the
whole binding builder. Keep the full accepted record mask and run its data
components through the accepted scalar contribution consumer as well.

[MEASURED] The accepted conversion host copies5,727 unchanged original bytes
across six spans and proves nine excluded calls unreachable in its domain.
The joined cohort preserves complete low24 binding masks and actual mirror
bits. Two of217 clips, keys52A61C9A/13E70F2B, retain four excluded frames due
to the existing sampler host's2^24 INFO/frame limit. The217-source admission
is static retention, not a live clip list.

[MEASURED] The optional9B8D50 pass is not the missing accumulator normalizer.
q22 shows a global/body/vector-gated constraint-record loop ending in9D6510.
Likewise,9BD160 resets accumulators and9A9C4F..9AA13C only supplies conditional
rest defaults for zero-weight blocks. None may be substituted for the unresolved
staged evaluator. Exact9E82E0/9E8930/99C810/9E7C80/9BD370 source bodies now
total5,901 bytes. A genuinely zero-child definition makes9E7C80 return without
writes and9E8930 skip child processing, while positive root ROT still executes
the full mixed x87/SSE99C810 normalization. The separate native/Rust gate now
matches66,636 source-derived cases (22,212 accumulators x three explicit
dt/history/skip contexts), all728 projected bytes,35 protocols and12 additional
x87-policy cases. A separate576-case owned arithmetic supplement covers alternate
merges, quaternion hemisphere dots and dt-threshold neighbors. These admit only
one root with definition childcount0. Real generated bodies with nonzero child
counts, full9F4730 traversal and constraints remain open. Evidence:
`target/animation/root-zero-child-stage-native-run-v1/rust-comparison-v1.json`
and `target/animation/root-zero-child-stage-arithmetic-native-run-v1/rust-comparison-v1.json`.

[UNKNOWN] Live clip/cache admission, actual actor association, controller clock,
whole-body constraints, all non-root channel selection and complete visible
animation remain separate acceptance gates. No static record retention or
sample/aggregate agreement closes them.

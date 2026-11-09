# Source-derived nonroot fixture and static host plan

2026-11-04. This is a preparation packet, not a native execution or an
animation result. Reproduce in an isolated Python environment from the workspace:

```powershell
python -I -S tools/anim_nonroot_source_fixture_v1.py
```

The destination `target/animation/nonroot-source-fixture-v1/` must be new. The
script never starts a host, compiler, Cargo, GPU or desktop. It pins and reads
the existing builder input/reply and POS evidence, validates a unique canonical
ANIM resource and its SHA-256, and saves the selected raw material and a
pending rotation sampler request. Proprietary data remain in ignored `target/`.

## Exact admitted source and retained result

The selected prepared builder case is 0: finalized-body cohort index 603,
CreatureData key `0F43029A:40626200:065CB828`, decoded body SHA-256
`4f2f83ea051011f97f7a39fc805bf830f0c258475736de6852dd54ae83bc247e`,
explicit cross-product clip `71D0A3D5:EE17C6AD:00000000` in PatchData,
ANIM SHA-256 `9daf9fe5bce32a039ff4a4de2ae70f9090b053b570c17da7d9141d61f83ee10a`.
The paired original-builder reply row 5 is `[00000001,00000000,0000000E,
20FF0813]`: pass bit 0, mask E, selected pose ordinal 19, wrapper/channel 8,
secondary FF, row flags 20. The channel's primary word has bit 0 clear.
Descriptor 1 is POS 71 and descriptor 2 ROT 32; both ordinal bits occur in
that binding mask. The payload's full selected channel descriptor bytes and
CHAN+88..D0 words match the prepared input exactly.

The prepared source body fields for ordinal 0 and ordinal 19 are saved as
separate 324-byte records. They are projected records, not original 0x468-byte
Definition objects; only the producer's selected fields are established.
The saved builder input and reply hashes are in the manifest, together with
all output hashes. The installed package is hashed before and after reading;
matching hashes do not rule out a transient replacement. The retained decoded
ANIM bytes themselves match the independently supplied cohort hash. This
fixture does not establish package mount precedence or an actor association.

For source frame `00000000`, original-host POS case 8019 is retained from the
previously checked POS corpus: position `[00000000,00000000,00000000]`,
weight `3F800000`, explicit CW 027F. The script checks the saved reply against
the corpus table and validates their hashes against the POS inventory and
comparison, but does not rerun the comparison or authenticate provenance
beyond those saved artifacts. The ROT source request is 1,952 bytes, hash
`ba2b6e3bb6b801dee699d2eec99f14c0031befba00829feb6888c58871f91c55`.
It uses exact frame-0 source rows and descriptor, with full neighbors
`[7,0,1,2]` and local window source indices `[0,1,2,7,8]`. It has no ROT reply.
A reviewed rotation-host run and independently verified output are required
before the input can be called a paired sampled POS/ROT fixture.

Blend 1.0, zero accumulator, unit scale, identity reference orientation and
CW 027F/MXCSR 1F80 are proposed explicit test inputs, not observed actor
state. No frame time or live clip selection follows from frame zero.

## Static plan; not authorization to execute

1. Independently inspect packet construction, upstream hash/provenance, the
   selected record including all low-24 bits, and the rotation request's full
   source-window mapping. Review the existing original ROT host and the exact
   request/runner before a separately authorized single execution and comparison.
2. Materialize distinct selected-node and CID-root Definition buffers using only
   admitted projected field offsets, plus guards and explicit values for every
   other read. Definition +154 is admitted for this prepared selected block as
   0x4, as traced below. Prove each call/branch and the owner/scale/reference
   pointers; do not generalize this value to another body or live actor.
3. Adapt the reviewed root conversion carrier as a *new* nonroot-specific
   host: preserve and separately pin original spans 9B0CA0..1B81,
   9B0B90..0C90, 99BD80..BE6A, 99D74F..DBFD, 99C8D0..C992 and
   5992B0..92CE. Its old root snapshot and omitted-call assumptions do not
   carry over automatically. Verify all branch destinations, global predicates,
   data constants, pointer aliasing and exact linked bytes. Starting directly
   at 99D74F is a partial accumulator slice, not the complete 99D190 binding
   traversal; prior masks, any selected branch and effective blend must be
   separately established. Position/rotation inputs must be paired native
   sampler replies at the same source channel and frame.
4. Save input, whole mutable buffers, write masks and reference-context words;
   guard all buffers and stack, audit inaccessible paths, run positive and
   malformed protocols, and compare with an independent Rust implementation.
   Record floating-point state and source/host/input hashes before and after.
   Original instructions must not be run until source, linked host and runner
   receive independent review. An accepted single-row zero-prior gate still
   excludes repeated rows, alternate paths, nonzero-child staging, final commit,
   actual actor/clip association, scheduling and visible animation.

## Follow-up: selected +154 and ROT host preflight

The prepared 324-byte body record begins with capability presence and nonzero
masks, followed by 32 IDs and 32 number bytes; the following 39-field list
is separate. `anim_body_topology.rs` identifies the two masks with Definition
+150 and +154. Selected ordinal 19 has masks `00000004/00000004` and slots
foot=1, cute=2, cspd=1; the independent documented `9B2E20` category map
assigns foot bit 4 and no category bit to the other IDs. The nonzero mask
therefore reproduces +154 = 4. The root ordinal 0 is distinct and has both
masks 9. At `99D46C`, TEST +154 against 0x14 is nonzero for this selected
prepared block; the further wrapper-bit branch and effective blend must still
be established for the host.

The 20 body records in this builder case were compared byte-for-byte to the
upstream `nonroot-selector-Z-inputs-v1-complete/input.bin` group at
`body-index.json` entry 603. Its 6,480-byte records hash matches that entry,
`5e087bc47d9e47f0358ace51f2543e5b9ee54e46b577ba19951e96546ef48d68`.
That entry identifies source CreatureData and a native finalizer projection;
its actor association is explicitly false. The accepted capability and
finalizer scopes are in `anim-body-capabilities-native.md` and
`anim-body-finalizer-fields-native.md`. This admits the recorded prepared
snapshot, not a runtime actor's value or the rest of the Definition allocation.

The retained ROT host and assembly hashes match the existing runner's pins:
`972bf058d3cf7711b2e9f0ea5ff63c03d5e4e3f1a7a6dafa7fa17653e9d27f18`
and `b914ed8a7ff6cd972b60cfc3733628489895ec9a2e538a79988547f27f8e84c6`.
The original source executable and CRT hashes match the host manifest. An
independent invocation of the existing static linked-byte `audit_host` matches
that manifest; this is not a fresh peer review of the runner. The request's
header is frame +0, count 5, duration 177, flags 8, CW 027F, blend 1,
enabled 1, reserved 0; five keys are `[0,72,78,157,176]`. All host-consumed
floating fields are finite, padding is zero, and the request satisfies the
visible count/order/duration and policy guards. Full-window and local-neighbor
mapping were checked by the fixture builder. The prior ROT corpus report
compares 30,240 cases including 3,072 source-window samples, but not this
new request specifically.

The existing `anim_rotation_native_run.py` runner hard-pins a 30,240-row
corpus and is not a single-request runner. `anim_root_pose_curve_native_run.py`
similarly hard-pins its own 3,702-row input. Therefore neither should be invoked
as though it admits this packet. A new minimal runner must first pin the host,
manifest, executable and CRT, all packet inputs, and itself; check linked code,
request guards and window mapping; perform exactly one hidden bounded process
invocation with no retries, save exit/stderr/output and immutable before/after
hashes, run structural inspection and an independent numeric comparison.
Source, linked executable, request and exact runner require independent review
before original bytes are executed. This note does not perform such execution.

## One-shot runner prepared for review

`tools/anim_nonroot_rotation_run_v1.py` is prepared but has not been executed.
It requires an explicit `--reviewed-runner-sha256` equal to its own file hash,
checks fixed host, helper, packet and provenance pins, reruns static source,
CRT and linked-host audits, and checks the exact request and source-window
mapping before creating a new output directory beneath `target/animation`.
After these checks it launches the host once, hidden, with a 60-second timeout;
there is no retry path. It records request, reply, status, structural inspection,
pre/post execution pins and any failure. Numeric Rust comparison remains a
separate gate; structural agreement is not numeric acceptance. The runner SHA-256
at preparation is `076ff8a98d3bf1df5d5dc5da533bf45118938c896e283ed28fe49bc1007d4165`.

An exact runner-hash argument enforces identity, not proof of independent
review. Source, linked host, packet and runner must be independently reviewed
before invocation; any change requires a fresh hash and review. `ast.parse`
confirmed Python syntax without executing the runner. No output directory or
original execution was created by this preparation.

## One-shot rotation sample comparison

The coordinating agent reviewed and ran the one-shot host once; retained
`target/animation/nonroot-rotation-native-run-v1/` records exit 0, a finite
reply and unchanged pinned files. The host reply SHA-256 is
`165fd1eee514a1e9d2a9fe601e006769667a5f3cc378eb701faeeb9c135625ce`.
The independently implemented Rust `anim_rotation_curve_verify` example ran
on the exact retained request with verified constants SHA-256
`76210f1fef0a723d62dbcf92c1a7ce9ce1697b99f6a14530692fa8e3871f07df`.
The strict `anim_rotation_compare.py` reports 1/1 interpolated source-window
sample exact, zero exclusions and no problems. Both 48-byte comparison
projections hash to
`88368b92f8813bc21beaef9eb950d6c1fede2f33fbda9400cfec7ca5a18d6c1f`;
comparison report SHA-256
`5d7b02ec19b612863d45369ba5887f1e96fd6afeebd5bcd829edee977b279173`.
The source sample xyzw and weight bits are `19F50000 00000000 00000000
3F800000 3F800000`. The POS and ROT samples now share the selected source
channel and frame, but no nonroot coordinate conversion, accumulation,
staging, actor association or visible pose was executed or established.

## Rejected nonroot host proposal

`tools/anim_nonroot_pose_host_prepare_v1.py` prepared an assembly-only proposal
under `target/animation/nonroot-pose-host-packet-v1/`; it has not been compiled,
linked or executed. Independent review rejected it: its omitted ESP+2C
producer makes it take root-cache branches for nonroot pose 19. It must not be
compiled, linked or executed. Host assembly SHA-256
`39725180a9bce4681bff7a7032a0a584843768a446ad020337d80c6837ddda63`,
exact 144-byte input SHA-256
`bb4cda5f23963d3588d84bde08113559dbf1ba9b836d4be2672784f800d00a17`,
manifest SHA-256
`79ab931e0d2b74036f43949faeaa65cc74a3198fb6a7e4eb078f943e1ca33107`.
The proposal is generated from the accepted sparse-root carrier with exact
one-time source edits: separate root-reference and selected-node quaternions,
selected +154=4, pose ordinal 19's bitmap byte 4 and bits 40/80, and an exact
input guard. Effective blend is supplied as 1 at the partial 99D74F entry;
that is a conditional assumption, not execution of the preceding 99D190 gate.
The owner scale, dimensions and reference inputs, zero prior state, and two
zero globals remain explicit fixture values. No complete 99D190, actor or
live-playback claim follows.

The manifest pins each copied source span and data constant. The copied
spans are `9B0CA0..9B1B81`, `9B0B90..9B0C90`, `99BD80..99BE6A`,
`99D74F..99DBFD`, `99C8D0..99C992`, and `5992B0..5992CE`; exit at 99DBFE
is the owned continuation stub. The proposal retains INT3 exclusions for
`99BE70`, `9A48B0`, `9A4DF0`, `9B0150`, `9B0530`, `9B05E0`, `9B0AC0`,
`9B75B0`, and `9CDA70`; the manifest records exact hashes and data extents.
These exclusions must be independently re-proved for this selected nonroot
case, not inherited merely from the root host. Review its assembly, input,
materialized root/selected fields, bitmap, branch gates, stack/shadow write
ranges and failure paths before compilation; any linked carrier and exact
runner need separate review. Do not execute this proposal on its preparation
status.

## Rejection and source-root reference preparation

Independent review identified a blocking defect in the earlier proposal:
`FrameBuffer+82C`, the original partial entry's ESP+2C, was zero. Source
`99D33F..343` zero-extends the binding row's byte +C into EBX and stores it
at ESP+2C; `99D409..411` uses that same value to choose the 0x2BC pose.
For the retained row the value is 19. With nonzero selector and ESP+10 zero,
`99D765..76E` skips the POS root-cache update, and `99D9B5..D9` skips its
ROT counterpart. The source producer at `99D21C..22F` establishes ESP+10
as true only when binding object +10 bit 0 is set and bit 1 clear; the two
later pushes shift its original ESP+8 location. The prepared builder output
has binding+10=0, conditional on that binding remaining unchanged until use.
Do not build or run the rejected proposal.

Source `99D337..3DF` instead normalizes root cache on the first reached
nonroot row, through `99CB70`, then copies its position/quaternion as the
reference to `9B0CA0`. Raw bytes of `99CB70..99CC73` show positive POS
weight causes SSE reciprocal and position scaling; positive ROT weight
causes x87 norm/sqrt followed by SSE reciprocal and quaternion scaling;
otherwise it supplies identity quaternion. This is a static trace, not a
new comparison of that full helper. The binding output's pass-0 root row 0
precedes nonroot row 2; row 1 is pass 1 and is excluded when pass 0 is
selected. Earlier reached bindings and the initial root cache must be modeled
before assigning a later row's reference.

The earlier native root sample fixture has a matching source clip, frame and
root pass (row 12), but its explicit identity rest context does not match this
body. `tools/anim_nonroot_root_reference_prepare_v1.py` now prepares the same
source POS/ROT samples with body 603's source-projected root position and
quaternion, using root position/quaternion as the initial reference and literal
root +154 mask 9. Its unexecuted input SHA-256 is
`bd1d0f0180699b4443e709515d76d5e6d3181e72df2b2bf96b601a9b52187ad9`.
The exact initial-unit scale/dimensions, blend and pass remain fixture choices;
mount precedence, stable binding state and live actor are not established.
The existing root host rejects mask 9 and its old outputs cannot be reused for
this body. A new reviewed host must admit the precise input, compare its root
cache, and separately cover 99CB70 and intervening root/nonroot record order
before its reference is accepted for the nonroot partial host. The existing
root-carrier CLI also pins the old generated ASM and cannot build the modified
nonroot proposal; a new reproducible builder and its linked audit are required.

## Source-body root plus 99CB70 proposal

`tools/anim_nonroot_root_host_prepare_v1.py` creates an uncompiled proposal in
`target/animation/nonroot-root-host-packet-v1/`, not an admitted executable.
Its 144-byte input SHA-256 is
`bd1d0f0180699b4443e709515d76d5e6d3181e72df2b2bf96b601a9b52187ad9`,
assembly SHA-256
`1fe8759037111c48064b9746d377068c2b95e4af0fd9ffd7fefab8fe026aa4b4`,
and preparation manifest SHA-256
`cb2d1593941e42a96b4a3af2276cea0c8ee1ce4c9fe672e536265c10fd343acc`.
It supplies literal root +154=9, accepts only the exact input, captures the
nine-word root cache before calling original 99CB70 and retains that cache
again after normalization. Both 36-byte snapshots are covered by its proposed
two-poison consistency check. The additional 260 original bytes at
`99CB70..99CC73` hash to
`eaa0c6adc09305fbf84286ff1c72d8f5cf0a2f7fbe96dc4cb7ffb89be8e58832`;
static decoding found no calls, three internal branches, and absolute constant
`014861D0` only. The previous conversion spans, nine excluded calls and owned
99DBFE exit stub remain explicit in its manifest. Output expands to 212 bytes;
its field layout and static disassembly are recorded there.

The proposal models only pass-0 root row 0 with zero prior cache, before the
first nonroot row, with unit blend, scale and dimensions, and binding+10
remaining zero. It does not execute complete 99D190 or observe an actor.
Independent review must check the source branch domain, stack/calling
convention, buffer and write masks, altered output/poison comparisons and every
retained exclusion. The prior sparse-carrier CLI cannot build this changed
assembly: a separate reproducible builder must include the extra span and then
receive independent linked-byte, section, guard and runner review. No host was
compiled or original instruction executed in this preparation.

## Revised root host guard proposal

An independent review rejected the earlier guard boundaries. The revised
uncompiled `tools/anim_nonroot_root_host_prepare_v2.py` produces
`target/animation/nonroot-root-host-packet-v2-final/`, assembly SHA-256
`5cba9971b2290a7a17002740fe952a9f500f541603e111ba6a7ee04aa8351a23`.
It compares all owner words against the shadow except bit 0 of the dword at
+1B8, thereby also checking bytes +1B9..+1BB. After conversion it checks the
frame from +800 through +33FF before writing partial-entry locals. It then
snapshots those owned locals and checks the entire frame after accumulation,
except +500..+7FF stack scratch, +854..857 and +878..87B source temporaries,
+8C0..8EF source call temporaries, and +A28..A4B root cache. These are
proposed guards, not linked-host or runtime evidence. The deepest-stack bound
and all allowed scratch writes need independent instruction review.

The manifest records a conditional reason for each excluded original call:
POS 71 excludes its 0x200 and relative 0x100 branches; channel bit 0 is
clear; both guarded global switches and wrapper+10 bit 0 are explicitly zero.
Root mask 9 does not select the B1-specific scale branch for POS71. These
conditions still require independent control-flow verification on exact bytes.
A separate sparse builder mapping the 99CB70 span and a linked-code/guard and
runner audit are missing. No compilation or execution has occurred.

The first v2 preparation failed after writing a partial packet due to an
incorrect `SourceImage.path` reference; the intermediate `v2-fixed` packet
placed a frame shadow before the intended setup. Both are retained as rejected
under `target/`; only `v2-final` is proposed for review.

## Conditional instruction and stack audit

The read-only `tools/anim_nonroot_root_static_audit_v1.py` checks pinned source
bytes and selected branch instructions and writes
`target/animation/nonroot-root-static-audit-v1/report.json`, SHA-256
`d50576bb94907de0c4766d9251c5ff746a5f6ef73ca107a449834f18c5d965a8`.
For the fixed host values, 9B0D38/3D excludes the first 99BE70 call because
POS71 lacks bit 200; 9B0DD4..DDE excludes the second because 71 & 120 is 20,
not 100; 9B1ACE..AD4 excludes the third on bit 200. Channel word 0 makes
9B1290..92 and 9B1856..58 bypass the relative POS/ROT branches and their
helper calls. Zero global 1551A74 skips 99D821, and wrapper+10 bit0 zero
skips 99D949 and 99D997. POS71 selects 9B0B90's 40 path, independent of
+154=9. These are conditional source-instruction conclusions; not a complete
automated symbolic execution or independent review of partial-entry setup.

On those admitted paths, the deepest traced stack point is `FrameBuffer+6D8`:
from +800, 52 bytes of owned arguments, a four-byte call return, 200-byte
9B0CA0 frame, four saved registers, five 9B0B90 arguments and its return
sum to 296 bytes. The audit verifies no calls or stack adjustments in admitted
leaf helpers 9B0B90, 99BD80, 99C8D0 and 5992B0; 99CB70 uses 20 bytes plus
its return. The comparison assumes the traced branches, correct caller
register setup, and balanced call-site operations. It does not independently
prove the linked host, runtime addresses, unrestricted inputs, or absence of
other effects. The current +500 scratch lower bound is therefore conservative;
independent review is required before tightening it or executing a carrier.

## Bitmap revision and remaining proof

The uncompiled v3 host proposal adds an exact `OwnerBuffer+1B8` dword-equals-1
check, while retaining the remaining-bit comparison against the zeroed shadow.
Its assembly SHA-256 is
`1a0cab067aac13861009debea8946aeee366da256bb77fd9b3c93f60d1faadb3`.
The +500..+7FF frame scratch exception was not narrowed: even if the
conditional +6D8 stack calculation holds, bytes +500..+6D7 (472 bytes) remain
unchecked. Complete reachable CFG, register/stack provenance and write-set
proofs remain open, so this proposal is not admitted for compilation or native
execution.

To support that review, `tools/anim_nonroot_root_cfg_inventory_v1.py` records
all linear control transfers, explicit memory writes and stack operations in
the seven pinned source spans under `target/animation/nonroot-root-cfg-inventory-v1/`.
Its report SHA-256 is
`c70c8d552483a00e59a7252e49c41fc57daeb12bf467f904717700603693298b`.
The inventory contains 119 control transfers, 211 explicit memory writes and
173 stack operations; it includes unreachable instructions, does not trace
registers or aliases, and is not a reachable-CFG or full write-set proof.

## Fixed-input walkthrough and indeterminate points

For the fixed input, the proposed conversion call has CHAN+88 word 0, POS
flags 71, ROT flags 32, mirror 0, explicit owner scale/dimensions 1, separate
source-projected root Definition and identity sample quaternion. Source
9B0D3D and 9B0DDE skip their branches for POS 71; 9B10FA skips the ROT60
reference transformation; 9B1292 and 9B1858 enter the bit0-clear paths;
9B0C84 sees channel bit 10 clear. The corresponding conditional stack depth
is bounded as above if the traced source call-site balancing and pointer
bindings hold.

At the partial 99D74F entry, ESP+2C and ESP+10 are both zero by host setup,
so this *root* case takes the root cache path. ESP+28=1E and ESP+38=4 include
POS and ROT descriptor bits; wrapper weights and explicit effective blend are
1; global 1551A74 and wrapper+10 are zero. The host supplies ESP+11 as zero;
full source computes that branch condition from Definition+214 and a parent
Definition's +150. The retained projection does not contain +214, so the
source parent/null context must be established independently before claiming
this accumulator routing is source-equivalent.

The first remaining data-dependent control condition requiring a complete
floating-point/register proof is the zero-cache quaternion dot and COMISS/JBE
at 99C939 in 99C8D0. The supplied root cache starts at zero and the fixed
inputs are finite, but this walkthrough has not proven every intermediate
quaternion operand and floating environment through 9B0CA0. Both helper
branches are leaf and stack-neutral; their exact output writes still matter.
99CB70's x87 square root, finite/zero classification and branch at 99CC0D
likewise require a byte-accurate arithmetic proof or a separately reviewed
native comparison. These are precise open proof points, not evidence that a
branch is faulty. The current audit does not construct a complete reachable
CFG, prove all pointer ranges, or close the 472-byte guard blind area. No
host was compiled or source executed.

## Definition parent pointer and accumulation branch

The retained source finalizer dossier
`target/rea/20261022-anim-rig-focused-v1/60-analyze_function.json` has SHA-256
`058073c8626ccd3c8027b9e7fdecec189825fc96a98ab5f1605ec998d02ec75a`.
At `9C4690..46B9`, after the block ordering step, 9C3150 iterates Definition
blocks at stride 0x468. For ordinal 0 it writes null at +214; otherwise it
computes +214 as the block vector base plus the block's +1FC parent ordinal
times 0x468. It then sets +218 from the first-child ordinal when child count
is nonzero. This is a static finalizer result, not proof of later object
lifetime or absence of mutation.

Body603's previously hash-checked 20-row prepared projection has runtime
ordinal 0, parent word FFFFFFFF and masks +150/+154 = 9/9. Thus its finalized
root's +214 is null under this source finalizer path. The caller
`99D71E..740` then sets the root host's Frame+11 to zero, conditional on
stable finalizer state and the same admitted object being supplied.

The selected nonroot ordinal 19 instead has parent ordinal 18. Parent 18 has
presence +150 = 2 and nonzero +154 = 2; its sole stored capability is limb
ID `626D696C`, number 1. Its 324-byte prepared projection hashes to
`d10126c8596279fef7210b77bfc9e3bdc5ba4ddc57016cb2db30ed4bda2ccb1e`.
Under the same finalized-array and unchanged-state conditions, ordinal19's
+214 points to ordinal18. At `99D720..740`, the parent's presence bit2 set
and bit8 clear produce Frame+11=1. The POS71 descriptor lacks bit 800 and
ROT32 lacks bit 100, so 99D85C..868 and 99DAC0..DACC choose the alternate
POS and ROT accumulators. A future nonroot host that supplies Frame+11=0 or
ordinary accumulation for this selected record is therefore not source-aligned.

The 20 records were compared to the body-index[603] upstream packet, whose
6,480-byte projection hash is documented above. These conditional facts do
not establish a mounted resource, live owner, active clip, post-finalizer
mutation policy, or safe pointer lifetime at the actual runtime call.

## Alternate accumulators into nonzero-child staging

In this prepared body, root Definition 0 has first child 1/count 4;
Definition 18 has first child 19/count 1; Definition 19 is a leaf with
count 0 (its stored next child ordinal is 20). Under the admitted parent
context and a reached positive-weight row, 99D190 writes the nonroot POS
sum/weight to node +7C..+88 and ROT quaternion sum/weight to +8C..+9C.
The values also depend on prior rows, blend and converted samples; a single
row does not establish the full node snapshot.

At 9F4781, 9F4730 first calls 9E82E0 for its supplied root node. That helper
merges the root's alternate and ordinary sums and stages its own position and
quaternion. The independently compared `anim_root_zero_child_stage` covers
that explicitly zero-child root domain only; it is not an implementation of
the body603 nonroot stage. Following the root call, 9F47D2..5668 visits nodes
in descending ordinal order. For each, 9F4809..4872 tests the ordinary
weights +64/+78 and alternate weights +88/+9C, and 9F4814..4918 combines
that node's active state with its child flags via 9E6890. In this prepared
topology node 19 is visited before parent 18, which will inspect the child
flag when its child range is traversed. Numeric state, child flag and ordering
must still be admitted at the actual call; source structure alone does not
prove the execution or a resulting pose.

After the reverse pass, 9F5681 invokes 9E8930, whose 9E89C6 calls 9E7C80.
The later 9F57E0 path also calls 9E7C80 during nonroot traversal. The source
9F4EC9..4FDC can assign node+238 modes 0, 1 or 2 depending on flags, body
masks and earlier branch conditions. The existing native/Rust mode-2 gate
uses actual geometry but explicitly synthetic stage state; it does not prove
which mode this selected node takes or supply its real stage. No staged pose,
commit, palette output, live actor or rendered motion is claimed.

## Reduced one-row flag analysis

Consider only an explicitly constructed reduced fixture: pass 0 processes row 5
for node 19, all prior node accumulators start zero, both sampled weights and
effective blend are 1, and no other row contributes to any node. This is not
the prepared builder's full pass and not an actor snapshot. In its full pass,
row 0 is root, rows 2–4 visit other nodes, and rows 6–7 visit nodes 16 and 5;
pass-1 rows are separate. All effects and ordering must be evaluated before
claiming a full-body result.

Node19 is the first of the 20 reverse-loop nodes. Source 9F47C7 sets XMM3 to
zero before that loop. The called 9E6D30 preserves EDX and XMM3, so on this
first iteration DL remains ordinal 19. Its exact 142 bytes
`9E6D30..9E6DBD` hash to
`6706f02362cfc10f87a74ea084550e29e0377bb85ec700916d2f8227439fbbaf`.
It also writes node+238=0 at 9E6D7B during this evaluator reset; that transient
write does not prove the separate fresh constructor initialized +238.

Conditional on source-correct alternate +88/+9C weights of 1 after the reduced
row, the zero ordinary weights and positive alternate weights make
9F4814..4895 initialize node19 flags `10 00 FF 13`; child count zero means
9E6890 is not called for it, and 9F4918 stores those four bytes. Parent18 has
one child, node19. If parent18 has zero own weights *and* XMM3 is zero at its
weight comparisons, its initial `04 00 FF FF` merges child `10 00 FF 13` via
9E6890's destination-lowbits-4 case into `02 00 00 13`, using child ordinal 0
within its one-element range. XMM3 is cleared at 9F5636, but the selected
node's complete route to that instruction has not been proved; consequently
the parent bytes are conditional and not an acceptance result.

The subsequent node+238 mode cannot yet be determined. 9F4EC9..4FDC gates
mode writes on generated flags, Definition masks and earlier branches; the
route to that point and EAX/EDX Definition provenance need full tracing.
The existing 64-case mode-2 test used synthetic stage state and is not proof
that node19 reaches mode 2. No host was executed or source pose established.

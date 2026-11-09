# Nonroot pose flags and weight source trace

2026-11-04. Static examination of the installed `SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`,
and retained `target/rea/20261027-source-runtime-v4/24-analyze_function.json`
and `71-analyze_function.json`. No native execution, actor observation or
new Rust implementation was performed. Addresses are x86 virtual addresses.

## Traversal and flag aggregation

At `9F4730`, the node count is derived from CID +2E4/+2E8 with stride 0x2BC.
After `9E82E0` and `9E6D30`, the first node loop starts at count minus one
(`9F47D2..47FF`); `9F5655..5668` decrements the byte offset by 0x2BC and
ordinal by one. Thus this pass visits node ordinals in descending order. For
an individual node, `9F4814..4823` starts four temporary bytes as
`04 00 FF FF`. `9F4809..4872` tests POS ordinary weight +64, ROT ordinary
weight +78, POS alternate +88, ROT alternate +9C against zero; `9F4879..4895`
sets the first byte to 0x10 when any test is above zero. COMISS edge behavior
is part of the original code and no arbitrary-NaN behavior is admitted here.

When Definition +208 is nonzero, `9F48AC..490D` takes first child ordinal from
Definition +204, visits that many contiguous 0x2BC node records in ascending
order, and invokes `9E6890` with each child's +1AC four-byte word and its
zero-based ordinal within this child range. The result becomes this node's
+1AC dword at `9F4918`. Descending ordinal traversal does not by itself prove
the serialized hierarchy places every child after its parent; that is a
separate admission to establish for a selected finalized body.

## 9E6890 original jump cases

The retained Ghidra function body excludes its switch bodies. Raw PE reads
recover the first table at `9E6974`: `[9E68F4, 9E68CD, 9E6928, 9E68B9,
9E6958]` for destination low three bits 0 through 4. Tables at `9E6988`,
`9E6998`, `9E69A8` respectively contain `[9E68ED,9E68E3,9E68ED,9E68E3]`,
`[9E6920,9E6914,9E6920,9E6914]`, and the same last four addresses. These
are direct raw table values and case instructions, not a validated Rust model.

Let destination bytes be D0..D3, child bytes C0..C3, and ordinal be the
32-bit second argument. A child with `(C0 & 7) == 4` leaves D unchanged;
destination low bits greater than 4 also leave it unchanged. Otherwise:

* D low bits 4: set D0 to `(C0 & 1) | 2`, D2 to low byte of ordinal,
  D3 to C3; D1 remains unchanged.
* D low bits 0: set D0 to `(D0 & 0xF9) | 1`, D2 to FF and D3 to 1.
  For child low bits 0 or 2 set D3 to 2; for 1 or 3 set D3 to C3+1
  modulo 256. Child low bits 5..7 leave the initial D3 of 1.
* D low bits 2: set D3 to 1; if the full ordinal equals zero-extended D2,
  set D0 to `(D0 & 0xFB) | 3`, otherwise set D0 to `(D0 & 0xF9) | 1`
  and D2 to FF. Apply the same child-low-bit D3 rule as destination 0.
* D low bits 3: if ordinal differs from zero-extended D2, set D0 to
  `(D0 & 0xF9) | 1` and D2 to FF; then follow the destination-1 rule.
* D low bits 1: child low bits 0 or 2 increment D3 modulo 256; child low bits
  1 or 3 add C3 modulo 256. Child low bits 5..7 leave D3 unchanged.

Cases 0/2/3 retain the non-low flag bits according to the masks above;
case 4 replaces D0, and D1 is not written. Ordinal comparison is 32-bit,
not a truncated-byte comparison. All cases can be checked at `9E68B9..696D`.
The callee's `RET 8` removes the child word and ordinal arguments.

## Source-data path and remaining gaps

The existing body finalizer and `anim_cid_node_init_v1` can supply bounded,
source-derived definition fields, ordinal links and node snapshots; the
ungrouped binding builder supplies ordered records under its admitted domain.
ANIM readers/samplers supply actual channel samples. Binding byte +C selects
pose and +D selects sampled wrapper. In `99D190`, POS and ROT sample weights
are multiplied by the supplied blend and accumulated into ordinary +64/+78
or alternate +88/+9C weights with corresponding sums. The accepted
`anim_root_pose::evaluate` covers a restricted first-root, zero-initialized
case only; `cell_runtime_root_data` rejects reached nonroot poses and handles
scalar components. Neither produces general nonroot POS/ROT accumulator state.

Before wiring source data to this flag merge, establish selected channel's
nonroot `9B0CA0` conversion domain (including relative secondary selectors,
body references, mirroring and scale), full ordered 99D190 accumulation and
reset/default state, source traversal topology and the actual clock, blend,
actor and resource identities. Then compare a bounded original-byte host and
independent Rust implementation, including active-child and skipped-child cases,
flags and mode writes. `9F4EC9..4FDC` mode production and `9E7C80` use are
additional gates; raw +238 must not be treated as initialized. Even a passing
fixture would remain conditional on supplied actor and clip association, and
would not alone demonstrate final pose commit, palette delivery or live motion.

## Concrete bit-zero candidate and ordered accumulation

A read-only inspection of the 1,085 prepared builder cases and their paired
native replies found 24,960 output rows with nonzero pose byte +C. Of these,
24,892 reference a channel whose CHAN+88 bit zero is clear. The descriptor
pairs POS 71/ROT 32, B1/32 and 31/32 occur on 5,882, 486 and 278 such rows,
respectively. Requiring both descriptor ordinal bits to be set in the actual
output row's low-24 mask reduces these counts to 5,702, 476 and 276. These are
row counts across the prepared five-body/217-clip cross product, including
both binding passes, not distinct clips, independent actor observations, or a
claim that the actual sampled frame has positive weights. Example from the
retained packet: supplied body index 603, clip index 0, output row 5, pose 19,
wrapper 8, CHAN+88 zero, binding mask 0xE, POS 71 and ROT 32. Labels are packet
indices; refer to its manifest for source identity. The `FF` secondary index
and prepared builder/owner assumptions remain those of
[the constructor gate](anim-ungrouped-binding-builder-native.md).

Inputs and replies were read without modification. Input SHA-256 is
`d9d009821bd97d0217af832b3c90d6d5012a53d4ac80451f31418a9b61696319`;
reply SHA-256 is
`5b5d790761cb63e30ce4b953f2062b23a483bf1eb5fbac46b0d689d9d6eff4d9`.
Counts used header body/channel counts, 324-byte body records, 80-byte channel
headers, 32-byte descriptors, and 114,292-byte paired replies. Reply record
count is word +0C and 16-byte records begin +2C. Descriptor flags and tags
are words +0 and +4. A row's pose/wrapper are bytes +C/+D. No separately
persisted census script or independent review was used, so reproduce before
using these counts as an admission gate.

`9B0CA0` begins with the selected node Definition's +10C position multiplied
by CID +70 and selected Definition quaternion +118. It also independently
reads the Definition reached by CID +8 to form a root-reference quaternion
from the supplied reference orientation; the two Definitions must not be
collapsed for a nonroot fixture. POS flag 0x200 invokes an offset rotation;
`flags & 0x120 == 0x100` changes the positional reference. The selected pair
31/71/B1, 32 avoids these two POS branches, and CHAN+88 bit0 clear avoids the
relative-secondary `9B0530`/`9B05E0` branches. This is a candidate reduction,
not evidence that the accepted root-only calculation is already valid for an
arbitrary nonroot context. Component masks, descriptor ownership, weight,
selected node/root fields, mirror and scale must all be retained.

At `99D3E7..43E`, row +D chooses wrapper stride 0x1F0, row +C selects node
stride 0x2BC, and secondary FF maps to -1. `99D46A..48E` can modify effective
blend if Definition +154 has bits 0x14 and wrapper +10 bit 1 is clear. POS
weight at wrapper +24 and ROT weight at +78 are individually multiplied by
that effective blend and pass the original positive COMISS gate. POS flags
0x400/0x800 and ROT flags 0x80/0x100, together with the path condition at
stack byte +11, select ordinary or alternate destination. POS writes ordinary
+58/+5C/+60/+64 or alternate +7C/+80/+84/+88; ROT uses the original
hemisphere helper `99C8D0` and writes +68/+78 or +8C/+9C. Source ADDSS operand
order differs among vector components and ordinary/alternate paths; do not
substitute vector reductions or assume a zero prior state for multiple rows.

A useful first original-byte fixture should take one explicitly chosen row
from this paired cohort, retain the full row mask and ordinal, recover the
matching original ANIM payload and sample both descriptors, and materialize
separate selected and root Definition records from the matching body. It must
also set and disclose actor/reference orientation, scale, blend, prior node
accumulator, global gate values, and source floating-point environment. First
validate single-row zero-prior conversion/accumulation for both mirror passes;
then compare ordered repeated rows and alternate branches separately. A reviewed
host must establish every reachable helper and preserve guards and source bytes.
The existing root host cannot be relabeled a nonroot oracle merely by changing
its record index. Use native results and an independently implemented model as
separate gates before any staged nonzero-child or rendered-motion claim.

The source dossiers used for this section have SHA-256
`c43b6e036940830dcb7748a89f7e9070a4bf1c3426032cd55d8459a7b26d78d7`
(`18-analyze_function.json`) and
`cc30b97c308ad525108b23e5d19b22de3fdfe4a8fe5f90749d82276e4c4c8e37`
(`873-analyze_function.json`).

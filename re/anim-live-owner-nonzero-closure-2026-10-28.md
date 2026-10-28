# Live owner and nonzero-node closure, 2026-10-28

Source-only preparation; no new native execution or actor admission.

The continuation handoff incorrectly described a 0x700-byte node. Exact saved
q21 assembly at 009C5307, 009C53C9 and 009C53E7 uses IMUL 0x2BC (700 decimal).
Definition blocks separately use 0x468. This correction comes from original
instructions, not from a convenient buffer size. The complete q21 assembly is
retained under ignored target/animation/live-binding-outer-boundary-closure-v2/.

Full 009C5130 creates the missing nonzero-node relationship graph: each node+0
points to its Definition block, +4 to CID, +8 to its parent's node (root null),
and +C to its first child when Definition+208 is nonzero. Definition+1FC and
+204 are indices into the same 0x2BC allocation. Absolute quaternion is copied
from Definition+118..124. Position comes from Definition+10C..114 multiplied
by CID+70; this initializer sets scale to 1.0 before the loop. It also writes
rest/history/default fields and calls later helpers. A loop-only projection
cannot be represented as the complete 009C5130 behavior.

An owned bounded projection draft is retained at
`target/animation/live-binding-outer-boundary-closure-v2/node-init-projection-v1.py`.
It emits only identified writes and retains position FMUL operands without
inventing IEEE results. It requires complete Definition records, validates
indices and distinguishes byte/word writes. It has no native comparison and
is not exported in the public API. It is intended to supply real hierarchy
links to later child staging once allocation, initialization and arithmetic
are admitted, replacing the explicitly supplied zero-child topology boundary.

The actual fresh map cannot be null. q89 passes ECX=CID+17A0 into 009C1520;
q90 reads CID+17A4/+17A8, hence map+4/+8. The 009C1520 wrapper's inline storage
starts at map+7C (CID+181C), and it invokes 009C07E0 on that map before a
possible A1B590 reserve. Exact 009C07E0 is the immediate missing source closure;
A1B590, 920DB0, 920E10 and 920E90 are secondary constructor dependencies absent
from the compact saved-query index. A fabricated null bucket array would make
q90's unconditional scan invalid and cannot stand in for source allocation.

Fresh owner production remains 009ABB20 -> 009C5550 -> 009C5130 -> manager insert
009AAD30, followed by cache 009A4430 and frame binding 009A3140. The earlier
7,140-record prepared builder capacity is not fresh cache allocation: genuine
fresh cache construction starts at zero and needs actual 009A0980 growth.
Actual owner references at +14, clip retained reference and wrapper reference
must be preserved across cache hit, replacement and teardown. This source
closure is a prerequisite for functional creature motion, not another selector
coverage claim. All 43 upstream body exclusions remain API limits.

## Fresh prime/map evidence

Fresh v6 q02 closes full 009C07E0 (174 bytes). Queries03..06 close 920E10,
920E90 and the two unsigned binary searches98B600/BA7F10. Exact source table
bytes are retained under the ignored closure directory and pinned in
`map-prime-producer-v3.json`. The actual wrapper pushes 0x11 into920E10;
upper_bound(primes,17)[-1] returns17. Thus the actual fresh CID takes the
allocated map branch, not the small-argument static array at0154EF28. The
latter array contains [0,FFFFFFFF] but does not establish actual fresh buckets.

For the fresh zero element count and load factor1,920E90 returns prime2 and
stores integer2 at map+18. Every arithmetic operation in this specific case
is exactly representable. Actual initial bucket count/pointer/sentinel still
need920E30 andA1B520, called by9C07E0. The source-only report therefore retains
those fields as unknown. No constructor or native acceptance follows from
this prime-table calculation.

## Actual 17-bucket layout

Fresh q07/08 close920E30 (94 bytes) andA1B520 (98 bytes). Lower_bound(17)
returns17; with loadfactor1 the initial threshold is17. A1B520's count17 path
uses map+30 rather than allocating, zeroes68 bytes and appends FFFFFFFF.
The wrapper descriptor passed into9C07E0 carries map+34 in its last word;
9C07E0 copies that word to map+30. Actual buckets therefore start at CID+17D4,
contain17 zero words, and have sentinelFFFFFFFF at CID+1818. This is distinct
from the 192-byte inline free-list pool beginning CID+181C. Subsequent920E90
stores threshold2, but2 <= existing17 avoidsA1B590 reserve.

Consequently fresh9CB5B0 scans17 zero bucket words, findsFFFFFFFF and compares
equal to its end sentinel. Its nonempty/free-list body executes zero times.
An owned relative72-byte bucket projection and pinned report are retained as
`fresh-buckets-relative72.bin` and `actual-fresh-map-layout-v4.json` under the
ignored closure directory. They describe the source-produced layout and must
not replace original constructor execution in a native acceptance host. Full
constructor closure still needs920DB0's actual free-list initialization.

The retained v1 draft treated quaternion transfers as raw bits; exact q21
shows FLD/FSTP, which requires explicit NaN/rounding treatment. It is rejected
for arbitrary IEEE inputs. New `node-init-projection-v2.py` emits only integer
and default writes and retains quaternion FLD/FSTP operands, position MULSS
operands and following history FLD/FSTP operations explicitly. Neither draft
is native or API acceptance. Position multiplication is SSE MULSS, not x87.

## Guarded nonzero-node native preparation

The new source-only native packet is frozen at
`target/animation/cid-node-init-native-packet-v1/pins.json`. The host under
`cid-node-init-native-host-v1` executes only original445-byte loop9C52F4..54B1.
Its separately prepared actual-body cohort contains1,322 bodies/46,256 nodes,
including23,981 nodes with actual nonzero Definition+208. All43 upstream body
exclusions remain. Accepted finalized fields are materialized into0x468-byte
blocks; unknown Definition bytes remain A5 and are not claimed original bytes.

The host supplies the source factory scale1, explicit CW027F/MXCSR1F80, a bounded
255-node allocation and source One14861D0. It checks all700 node bytes against
the exact source write mask, guards buffers and source inputs, and canonicalizes
only reply pointer words to actual ordinals. Raw native state retains pointers.
Quaternion FLD/FSTP, position MULSS and history FLD/FSTP retain their source order.
A source-reference projection preserves finite bits under this scale1 domain;
the pending Rust model executes the actual scoped arithmetic operations and
restores incoming floating state. The pending verifier compares every700-byte
row against native and reference under three incoming CSR states.

Prebuild source/mask/control audits pass. No compilation, original native
execution or Cargo run has been performed by this worker. Full9C5130, reference
lifetimes, allocation, constraints and live actor animation remain separate.
The next functional consumer is original nonzero-child staging fed by these
actual node parent/first-child links, not an authored zero-child substitute.

Fresh q09 closes920DB0's92-byte initializer: sixteen12-byte records span
CID+181C..18DB, aligned4; each record links to the next and the last points0.
Map headCID+17BC pointsCID+181C. The fresh17-bucket map constructor's source
callee closure is now established without heap allocation in its actual path;
the remaining full CID constructor/reset helpers are a separate closure.

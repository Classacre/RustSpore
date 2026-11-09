# E69B00 selector-10 continuation

`E69B00` is an actor-state path called by `E7E050`, not E7E160 event dispatch.
It performs checked first-actor lookup `B71CC0`; null or first +112 nonzero
returns false. For source raw type 2 it chooses state 12. If first +18C is
0,1,2,3,18,19,20,21,22,24,25,26,27,29,40,48,50 or 51, it writes 12 there
and calls `E57890` with the first actor and selected word; otherwise it skips
that write/call. The checked fixture models this prefix with fresh admitted
actor bytes, retaining a pending external call or second lookup. Fixture pool
identity and lifetime checks are extra safety policies.

## Controlled state-table comparison

The independently reviewed host compared `E69B2D..E69B70` (68 bytes,
SHA-256 `be3f0d235b36619f5c007d98e20aba4ec99d779aba2f342841a4dd57894fe205`)
and source table `E69C4C` (52 bytes, SHA-256
`e9e28fe03c147ee01b04a293b18eed8c5a382fd91c2354df9ba79beeaf97343e`)
against the owned model with type=2 and states 0..53 plus FFFFFFFF. All 55
final +18C words and E57890 call counts matched. The [receipt](../target/cell-e69b-state-result-v1/receipt.json)
SHA-256 is `9333ec7d4bbfeb4b93dee5515d0f7edd6f7c2e3f9bf448b2d8367e2605960bf3`.
Model/native raw stdout SHA-256 values are
`9de1f8b660324fab7acc2768572071cc5b65813d3a27ac0ffa1968d0ea33dd04` and
`481e7b97a777802e0d03bc44b2136655e21c23714d58588fe7e84a8ff5e4d9a6`.
Both exited zero and emitted no stderr. The host SHA-256 was
`3fa15d8eda9aca83dee517ac62259751e1b6967b931e7b51410e88c03f7ec3a5`,
runner SHA-256 `fba0dcc7331184cab4ed8132d77cff14c72f4d49b8bb2517a093e145d270084a`,
and model source hash `b7ef5c78e7eeebd8ca55e1bb21f8a7224172dbe469b2809593586247b5c1da21`.
All rows had x87 control 027F, status 0000 and MXCSR 1F80; the runner checked
x87 control/TOP, MXCSR control, linked nonpatch bytes, relocated table and
stub operands/targets, and pre/post pins.

The host uses a guarded synthetic actor and fixed type=2. Two absolute table
operands point to host copies; E57890 is a guarded stub that verifies actor and
selected arguments and the +18C write, increments a count, and has no effects.
The slice begins after first actor lookup/+112 and stops before second actor
lookup. It does not execute actual E57890, callback effects, actor allocation,
event creation or dispatch. Prior event and gate comparisons remain tied to
their separately recorded source hashes.

## Following the callback boundary

After `E57890` returns (or when it was skipped), `E69B71..85` re-reads the
second actor handle from the call stack and performs another `B71CC0` lookup.
Null takes `E69C38..41`, returning AL=1 without the next actor-byte stores.
On non-null, `E69B8F..9F` stores byte 1, in order, to retained first +178,
resolved second +179 and resolved second +17F. Then `E69BA5` calls `4098A0`
for a local object; further `E68070` and audio/timer paths remain pending.
A real E57890 could mutate either actor, the handle slot, or lifetime before
that second lookup; carrying either object or input through it requires a
fresh explicit admission. The non-null path writes through the retained first
pointer without re-resolving it, so a safe fixture's stale/foreign rejection
is a policy rather than source behavior. Distinctness/nonaliasing must also be
admitted for any model that reports independent actor snapshots.


The owned continuation requires either explicit completion of pending E57890
or an admission that it was skipped, a freshly supplied second stack handle,
and an explicit null or checked second lookup result. A null result models
the true return with no actor stores. On a non-null result the fixture checks
the retained first and resolved second identities, requires distinct actor
snapshots as an extra safety rule, and returns the ordered three byte values
with `4098A0` pending. It neither invokes nor assumes effects of the callback,
lookup, constructor or subsequent actor/audio/timer path. Focused tests cover
callback admission, null, fresh handle mismatch, foreign/stale identities and
the pending writes. The post-lookup slice comparison below checks its bounded stores. The preceding 55-row receipt
is tied to its earlier model source hash and does not validate this addition.


## Controlled second-result/store comparison

The reviewed host executed `E69B85..E69BA4` (32 bytes, SHA-256
`29b3ab8bf80c0278a54cd8d95376e6a9ff9348e47283f2c1f727cc185a3614e0`)
with an admitted synthetic lookup result and compared the three actor-byte
results and branch marker. All four cases matched: absent `(00,00)` ->
`(00,00,00,1)`; absent `(A5,5A)` -> `(A5,5A,5A,1)`; present `(A5,5A)` ->
`(01,01,01,0)`; present `(FF,FF)` -> `(01,01,01,0)`. The input pair is
initial first +178 and second +179/+17F; output lists those three bytes and
the synthetic missing marker. Both processes exited zero with empty stderr.

The [receipt](../target/cell-e69b-second-result-v1/receipt.json) SHA-256 is
`625837b00be8931ea6aed160e3f05f36288d00f21bf6b23ac5a4cfeef1a9662d`.
Model/native stdout SHA-256 values are
`b23304918931746730dafe0da2f07013a8bad24511d7e457b2098c1c537e3750` and
`e73fed7cf88c47a117fab5b93feb6eba8199461e07609df9e5cea276bd02d836`.
The host SHA-256 is `6a966869ecbc89472b85320b20bea6163349024541577bcd9c931d9bed0f8f2f`,
runner SHA-256 `7ded0a378c916d12d70f3eb6db52f2eea8f2c620b9bf5948a3348b6a0e4a5e01`,
and tested event model source SHA-256
`5073225d25a8e2f5acbe4937d62fc75ad8b55fb587eade62caef9b4acd253b9c`.
All rows recorded x87 control 027F, status 0000 and MXCSR 1F80. Linked
nonpatch bytes, branch target, synthetic exits, controls and pre/post pins
were verified.

The host redirects the null-result JE to a synthetic return and supplies
guarded, distinct actor buffers, EAX as the admitted lookup result and BL=1.
It does not execute B71CC0, E57890, the next 4098A0 constructor, dispatch,
or any real actor lifetime or callback effects. This test and the earlier
55-state comparison have different pinned source hashes.

## Following 4098A0

The non-null path calls `4098A0` with a local object pointer. The constructor
zeroes its first two words, copies three global words to +04/+08/+0C, copies
a global word to +10, and calls `41CAE0` to copy nine global words to
+14..+34 in three three-word groups. It returns that local object pointer.
These are separate ordered reads of global data and may require stable-source
and nonaliasing admissions; no identity matrix or other values are assumed.
The next call `E68070(first, selected word, local object)` can overwrite the
local and has resource/cache and indirect effects, so a copied post-call
snapshot requires fresh admission. The E7E050 caller's event object is not
passed as an argument to this E69B00 continuation.


## Controlled 4098A0 local-constructor comparison

The reviewed v2 host ran complete source functions `4098A0..40991E`
(127 bytes, SHA-256 `b37177c54324bed1456ba6ff14f4ab9a43153859066a325ec5032d67e850b54f`)
and `41CAE0..41CBB7` (216 bytes, SHA-256
`7a78f4ce62efbe8d6de3a972cc52ee6409f80016c17959ae2730c65e879b0393`).
All four full 56-byte model and native local objects matched. The
[receipt](../target/cell-e69b-local-result-v1/receipt.json) SHA-256 is
`e3dacffe0e50529cc9cf33a78f3c2f4f3841abb219488bd79b1e67990de51951`;
model and native stdout hashes are
`e14b4c821eed22e568c5f560ce781e0d3b521efd3687ee139ed9561db8e49800` and
`dbd6ef9475827099e5e508dddf6f1fd51757336661e48461f4a52b7b8d0778eb`.
Both processes exited zero with empty stderr.

Inputs are 13 hexadecimal words: vector[3], scalar, matrix[9]. The four rows
are `00000000..0000000C` in sequence; `80000000 7FC00001 00000003..0000000D`;
13 words of `FFFFFFFF`; and `3F800000 00000000 00000000 7F800000` followed
by `00000001..00000009`. These are copied bit patterns, not assertions about
source global values. Host SHA-256 is
`a64e0ac239c3041715a2bc35f8f6ba584dcb7b600af982201b4a9a28c2cb9072`;
runner SHA-256 is
`6fa8a91d6c5920ef6c7760e35cc575089296787048c5a55d2c35ebedae772b8f`.
Model source hash was
`0e910781ab4f31ef3cbb80753bd7857065cff9507640880306e7a44ec0ac18d2`.
The runner verified complete instruction coverage, constructor `ret`, copy
helper `ret 4` (`C2 04 00`), linked source bytes except five relocated data
operands and one call operand, target addresses, pre/post pins, and pointer
and stack checks. All rows recorded x87 control 027F, status 0000 and MXCSR
1F80; control/TOP checks passed. Stable, disjoint synthetic global and output
buffers were admitted; actual source global values, aliases, E68070, actor
lifetime, callbacks and event dispatch were not tested.

The original v1 preparation was not executed. Independent review caught its
truncated `41CAE0` slice, which ended with the bare `C2` opcode and omitted
the `04 00` operand. Its preparation directory remains retained as evidence;
the reviewed v2 host includes the complete source instruction and was the only
one run. Earlier state and second-lookup receipts remain tied to their own
recorded source hashes.


## E68070 next boundary (static only)

`E69BAA..E69BB1` passes retained first actor, the selected +0C word, and the
new local object to `E68070` in that order. `E68070` treats the selected word
`FFFFFFFF` as the direct fallback: it calls `537EE0` with destination local
and source first actor +48 (`E6807A..96`). Otherwise it reads first +248 and,
only when nonzero, first +FC. If either is zero it takes the same fallback
(`E68097..AD`, `E680C7..DA`). When both are nonzero it uses the actor +248
word in an unchecked `B71D00` lookup in global +168 and reads resolved +24
without a local null test; zero +24 also takes the fallback (`E680AE..C5`).
A nonzero +24 leads to pending `E84180(resolved +24, selected, local)` before
re-reading actor +FC/+100/+104 and invoking `E67A50` (`E680DB..E68103`).
These reads cross external calls and cannot be filled from an earlier snapshot
without an explicit stability admission. Later code indexes the returned
cache at `selected*72+40`, with no observed range check here, and performs
ordered binary32 operations before changing local fields and flags; it must
not be reduced to an assumed position copy.

`537EE0` is also not an unconditional copy. It first compares destination and
source flags, calls `41DBD0` for vector comparison when flags agree, compares
the +10 words as floats, and, if flag mask `0x0002` is set, calls `41DC30` for matrix
comparison. The observed decision (`537EE0..537F53`) can lead to incrementing
destination +02 word and copying source flags, vector, float and matrix
(`537F53..537FAA`), or return without those stores (`537FAC..B4`). Its x87
float copy and SSE comparisons require an explicit FP environment and ordered
comparison semantics; callbacks, aliases, mutation, helper results, and
pointer lifetimes remain unmodeled. The `FFFFFFFF` fallback reads actor +48
without needing +248/+FC; a fixture must not demand snapshots for skipped
reads or infer that cache +24 has been resolved.

A bounded next model could expose the selector/actor-field gate as pending
`537EE0`, pending unchecked cache lookup, or pending `E84180` after an
explicit fresh cache +24 admission. It should preserve separately admitted
actor and local-object identities, model neither fallback copy nor arithmetic
without their helper outputs and FP context, and revalidate actor/cache
identity across external calls. This is static tracing only; no source or
fixture implementation for this E68070 gate has been added here.


## Fallback comparison and E84180 trace

The `537EE0` helpers implement ordered numeric equality, not raw-bit
equality. `41DBD0` checks three vector words in order using `UCOMISS`,
`LAHF`, `TEST AH,44`, `JP`; it returns 1 for the first unequal or unordered
comparison and 0 only if all three compare ordered-equal. Thus positive and
negative zero compare equal; NaNs take the unequal/unordered branch.
`41DC30` inverts the result of `422D90`, which invokes `422E00` in order on
the three matrix rows; each `422E00` returns 1 only if all three binary32
values compare ordered-equal. `537EE0` first compares flags, then vector,
then +10 with the same ordered comparison, and checks matrix only when flag
mask `0002` is set. Any mismatch increments destination +02 as wrapping
16-bit, copies source flags, vector, x87-loaded/stored +10, and all nine
matrix words; an all-equal path leaves the destination unchanged. A model
must preserve short-circuit order, byte aliasing and pointer lifetimes,
MXCSR/x87 exception masks and status, and x87 float-load/store behavior for
special values. In particular, a bit-for-bit copy of signaling NaNs at +10
is not established. A useful first implementation could restrict inputs to
finite values and admit disjoint stable snapshots plus masked, specified FP
controls, retaining exception/status or rejecting unmodeled inputs.

`E84180` follows the nonzero cache +24 branch and dereferences argument 1
at +17C, then its +00 object and +384/+388 range, computing a signed entry
count. It walks records with stride 0x468, comparing each +21C to the
selected word while advancing a companion pointer by 0x2BC
(`E84186..E841EA`). This is an unchecked pointer traversal, not a validated
lookup. If the count is nonpositive or no match is found,
`E841EC..E84238` writes nine current global words from `016B5534` to local
+14..+34, current global scalar at `014861D0` to +10, three words from
`016B54E4..EC` to +04..+0C, then zeroes the two words at +00 and +02.
These are fresh reads after entering E84180, not the earlier 4098A0 defaults.
A matching entry reaches `E84239` and a substantially longer SSE-dependent
path, whose effects are not yet modeled. No-match can be represented only
after an explicit admission of the completed traversal and stable, disjoint
current globals; the model must stop before `E67A50` and subsequent indexed
cache access. Callback/resource/cache identity, array bounds, default-global
mutation and traversal lifetime remain independent requirements.

A realistic native comparison should isolate the fallback helper or the
no-match store slice with guarded buffers, fixed finite inputs, explicit
MXCSR/x87 controls and sticky-status capture, and exact linked-source bytes.
Cover equal and changed flags, each vector/scalar/matrix difference, signed
zero, revision wrap and short-circuited matrix reads; model NaN/alias cases
only after their exception and mutation behavior is independently bounded.
For E84180 no-match, compare all 56 local bytes against distinct admitted
global snapshots and retain raw results, with a guarded stub or a separately
hosted store slice; this does not validate the list traversal, match path,
E67A50 or live callbacks.


## Controlled finite 537EE0 comparison

The reviewed host ran complete original `537EE0..537FB6`, `41DBD0..41DC27`,
`41DC30..41DC4E`, `422D90..422DFB` and `422E00..422E57` with six relocated
internal helper calls and guarded disjoint 56-byte objects. All 10 modeled
records matched all 56 native output bytes. The input corpus hash was
`fc998452a66757ed37955afe90e8e91f2d6e7aefdefcd54f84034d9df85417e0`.
The corpus covers exact equality; signed-zero vector equality; differing
flags; changed first and third vector components; changed scalar; changed
first and last matrix components under flag mask 2; an ignored matrix change
under flags 0; and revision `FFFF` wrapping on a vector change. All numeric
words are finite normal values or signed zero.

The [receipt](../target/cell-e680-fallback-result-v1/receipt.json) SHA-256 is
`5f1e53b448a5ca7227ba8bd6d09aa67b740552eb79fec3cb70290c2c1c4ea430`.
Model/native stdout SHA-256 values are
`e6562d2d69b3ed5a6a11d5bcf49387b74a843c9a40d3793253cc4bcdb4f95a8f` and
`1c303c1cd98a786a262f6509791954d9af6c567bcaebb9158ef05fef21c47c97`;
both processes exited zero and emitted no stderr. Host SHA-256 is
`7f7d2b2694840b4dc0c9f7050735809bdfec028d2e24ddcf8ea703610b02d02c`,
runner SHA-256 `bfa0035abef2148002b3130f84b3fe3776c8da16c25c8f9bff13a592c1f7e432`,
and event model source SHA-256
`72520ee3c2bc0cc3f55f20250f396e5b9e3351c9eaed63469df71a8c44819fca`.
The runner checked complete function instruction coverage, `ret 4` and
plain `ret` tails, linked nonpatch bytes and all six call targets, pre/post
pins, and x87/MXCSR controls. Each row recorded x87 027F, status 0000 and
MXCSR 1F80. This comparison excludes NaNs, infinities, subnormals, sticky
input flags, unmasked exceptions, buffer aliasing, concurrent mutations,
E68070 routing, live actor/cache lookup and gameplay dispatch. The checked
fixture's actor identity and stable-buffer admissions are additional policies,
not behaviors proven by the native helper.

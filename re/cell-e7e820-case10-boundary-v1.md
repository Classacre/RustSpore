# E7E820 selector-10 event boundary

Static x86 trace of installed PE SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
`E7E820` reads selection +00 and +04 and resolves each, in order, via actor
pool `B71D00` (`E7E82F..E7E851`), with no observed null check before the
case code dereferences actors. It reads selector +08 and dispatches through a
21-entry table; selector 10 enters `E7EF1F`, which calls `E7E050` with first
actor, selection +0C, literal 2 and the second actor in ESI. A false return
continues to pending `E67170` using record fields and the caller float
(`E7EF38..E7EF6D`); true returns without this fallback.

`E7E050` checks second +179 first; nonzero returns AL=1 without creating an
event (`E7E051..62`, `E7E155..5A`). Otherwise second +111 == 1, second +17B
nonzero, or first +178 nonzero each return AL=0, in that order. Only after
these gates does `E572E0(first, second)` run; nonzero returns AL=0. The new
owned-fixture case-10 model rechecks the explicitly supplied selected actor
handles and takes caller-supplied fresh byte snapshots, stops at pending
`E572E0`, and represents the false-return fallback as pending. Checked-pool
rejections and stable snapshots are extra fixture safety admissions; no live
pool dereference, relation callback, or fallback is executed.

On the relation-zero continuation, `E7E0AE..CF` re-resolves the first actor ID
and calls `E6D260`; its x87 return and an external constant feed subsequent
floating-point values. `E7E102..111` calls `E51D40`. `E51D40` invokes event
pool `B71C50`, then `B71D00`, dereferencing the result without a local null
check (`E51D40..6F`): no graceful allocation/lookup failure is established.
It stores selector at +24, first and second actor IDs +28/+2C, raw literal 2
at +38, second actor +108 at +3C, copies other provided words, vectors and
floating values, and initializes +04 and byte +08 to zero. It returns the
allocated handle, not the resolved pointer. Zero vector and quaternion pointer
arguments select global default objects, whose current contents are read at
construction; their values must not be invented. `E7E050` next calls
`E52730` with handle and callback `E7DBB0`, `E52750` with handle, index zero
and a float, and `E69B00` with further inputs. The first two each freshly
resolve the event handle without an observed local null check; they write the
callback to +04 and the float to +0C respectively (`E52730..46`,
`E52750..6F`). Their lookup lifetime and the remaining call effects remain
boundaries. Later `E7E160` gates a callback on current record
+08, +04 and an argument byte, then reloads fields; neither callback nor event
lifetime is assumed stable across the handoff. Case 9 and selectors other than
10 are unsupported by this bounded continuation.

The earlier seven-row native coordinator comparison is tied to its earlier
coordinator source hash in `cell-controlled-pair-coordinator-v1.md`. The
case-10 gate comparison below uses a later source hash and is a separate test.
Neither establishes a live gameplay binding.


## Controlled original-instruction gate comparison

The reviewed, guarded host ran the 71-byte `E7E050..E7E096` slice, SHA-256
`ba4cd95e8ff5272fd51d096672ca2dbdc70695d850ddfb8192ca6c689971be0b`,
against the owned fixture oracle. All eight rows matched both outcome and
relation-call count. The [receipt](../target/cell-case10-gate-result-v1/receipt.json)
SHA-256 is `3cb008b61ba3c6224e3373a65c615fa1dd25075cb929d603d9ff22038e000610`;
the preserved model and native stdout hashes are respectively
`b1522a9cb063d1fcca55992053a8569825678feab63f6e5881d51a2595966be4`
and `685751f32901458323acff03a9a43074e1323b529a1084291e953e80342c2919`.
Both processes exited zero and produced no stderr.

The corpus tuple is `(second +179, second +111, second +17B, first +178,
scripted relation return byte)`; outcomes are `(return boundary, call count)`.
`2` is a synthetic marker for stopping at the unexecuted relation-zero
continuation, not a native E7E050 return value.

```text
(  1,   1, 1, 1, 1) -> (1, 0)
(  0,   1, 1, 1, 1) -> (0, 0)
(  0,   2, 1, 1, 1) -> (0, 0)
(  0,   2, 0, 1, 1) -> (0, 0)
(  0,   2, 0, 0, 1) -> (0, 1)
(  0,   2, 0, 0, 0) -> (2, 1)
(255,   0, 0, 0, 0) -> (1, 0)
(  0, 255, 0, 0, 0) -> (2, 1)
```

The host uses guarded synthetic actor buffers and a stub that checks actor
argument order, counts calls, and returns the scripted relation byte. The
near conditional branch at `E7E062` is redirected to a synthetic epilogue
for the true boundary, and the call at `E7E08B` is redirected to the stub.
The host stops before the original relation-zero continuation; no original
relation, event creation, allocation, ownership, cleanup, or callbacks run.
The linked-image verification checked all other fragment bytes, both targets,
and synthetic epilogue bytes. Pre/post pins matched; each row recorded x87
control `027F`, status `0000`, and MXCSR `1F80`. The runner checked x87
control and TOP plus MXCSR control while recording status and sticky bits.
Host SHA-256 is `1f5ff69e5fc4c762b0a9e54fe7f0f6c6cd10905473e5470239ae9bf94106d619`;
runner SHA-256 is `7ed0f6d27e6d677126d102df6888eb0460fc84756a853294ef2473f321b044c4`.
The model executable was
`84f97fa0c29f9c7e951470b21169070e8e12e7bcee81caad7d78ef0eb17f6d6e`,
and its coordinator source was
`8eebdd69a709f1c97cd31d1fd6db2379118c7b4768df3c963d6cc12adf5187a8`.
This finite synthetic corpus does not prove actor resolution or source callback
effects, and the additional checked fixture lifetime policies remain separate
from the native fragment.


## Owned construction inspection

`cell_source_case10_event_v1.rs` composes only the relation-zero pending
boundary with explicitly admitted post-`E572E0`/first-actor-lookup/`E6D260`
effects, fresh checked fixture actor identities, second definition word,
duration bits, and global default vector/quaternion snapshots. The default
vector is read twice by the source; this model admits a stable vector across
both reads. It separately admits a synthetic event allocation handle and
matching resolved-object token before applying the `E51D40` case-10 field
writes to caller-supplied record bytes. Bytes not written by the traced
constructor remain unchanged. Failed allocation and invalid native lookups
are not modeled as graceful source outcomes.

The constructed event has selector `1` at +24, actor words at +28/+2C,
selection +0C at +30, `FFFFFFFF` at +34, literal `2` at +38, the fresh
second actor +108 word at +3C, admitted duration bits at +1C/+20, and zero
at +40/+44. The two admitted default-vector reads populate +48..+5C, and
the quaternion populates +60..+6C. +70, byte +74, +78, +04 and byte +08
are set to zero, preserving unaddressed bytes. For each subsequent setter,
the caller must separately admit that the same event handle freshly resolves
to the same synthetic object: callback +04 becomes `E7DBB0`, then a fresh
caller-supplied float word is written to +0C. The model stops at pending
`E69B00`; it does not execute a callback or dispatch an event. Non-aliasing,
actor and event identity stability, and lookup validity are additional
inspection admissions rather than native guarantees.

The focused tests check creation fields, untouched bytes, setter order,
foreign/stale actor rejection, and mismatched event-object admissions. The field-store comparison below covers only its pinned construction stores.
The earlier gate and coordinator receipts apply only to their pinned source hashes.


## Controlled event field-store comparison

The reviewed host compared four complete 124-byte records from the owned
construction model against five unmodified source instruction slices:
`E51D6F..E51DC7`, `E51DD1..E51DE1`, `E51DED..E51DFD`, `E51E09..E51E31`,
and `E51E33..E51E3B`. All four records matched, including bytes outside the
stores. The [receipt](../target/cell-case10-event-stores-result-v1/receipt.json)
SHA-256 is `735cf6d44fac081ada2d897ae063e5d36d430f5437f03a6f19a4892e1da7e367`;
the model and native raw stdout hashes are respectively
`61f6b27c7e57422246f87abd21196aeeda62b1f8aaf53f972ca8df2d4c6dd707` and
`41e5c136b91366baf9f507558b4f4455c03da44f8ea58313ea54961ede3d88f2`.
Both processes exited zero with empty stderr. The model executable hash was
`bc9777fd950aaf08b2571cb71344b25121e5c3a0751cb1272510ef9559fd7e87`,
with event model source hash
`e82e93404226a07efa83722cf7f181ce0e735ddf10378b0bcaf39e98abce2884`.

Each row below lists hexadecimal `(selection +0C, second definition,
duration bits; vector[3]; quaternion[4]; initial record fill byte)`. Actor
words are synthetic `00010000` and `00020001` in all rows.

```text
00000025 00001234 3f800000; 0000000a 0000000b 0000000c; 00000014 00000015 00000016 00000017; a5
ffffffff 00000000 80000000; 00000000 ffffffff 7fc00001; 00000000 00000001 80000000 7f800000; 5a
00000000 deadbeef 00000000; 11223344 55667788 99aabbcc; 00000001 00000002 00000003 00000004; cc
87654321 abcdef01 7f7fffff; 00000004 00000005 00000006; 00000007 00000008 00000009 0000000a; ff
```

The runner verified all 20 linked slice copies against the pinned installed
PE and checked all recorded pins before and after execution. All rows recorded
x87 control `027F`, status `0000`, and MXCSR `1F80`; the runner checked x87
control and TOP, and MXCSR control, while recording status and sticky flags.
Host SHA-256 was
`560a22bed46665006d3d1b7d0c898329dd8c1d1e839ca979d434ccf3b8552166`;
runner SHA-256 was
`8507e645a2762a5df1f1d0e0796acc00c708a26271814f8052df76b19eb6aee1`.

This is a sliced store comparison. It supplies guarded synthetic record,
vector and quaternion buffers, source-equivalent stack values, and stable
default snapshots; it skips the null-default selection branches and original
`pop edi`. It does not execute native allocation, actor or event lookup,
callback/float setters, `E69B00`, callbacks, dispatch, or ownership/lifetime
transitions. Matching these four records does not validate those admissions,
the earlier relation outcome, or live gameplay behavior.

# E673A0 continuation and event boundaries: next bounded slice

Read-only Capstone decode of installed `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
No original instruction, callback or live game was executed in this trace.
The current fixture has modeled the B-definition-6 E59B40 output only; it
stops before E673A0 and its actors are owned synthetic identities.

## Fresh source call inputs

After the E59B40 call returns, `E6A548..E6A590` retains EDI=A and ESI=B
actor pointers and EBP=point pointer. It reads the *current* binary32 A/B
fields +90/+94/+98, not the +4C/+50/+54 feature-projection positions. In x87
instruction order, with arithmetic rounded under the admitted x87 environment:

```text
x = A90 - B90; y = A94 - B94; z = A98 - B98
sum = (z*z + y*y) + x*x
distance = FSTP_m32(FSQRT(sum))
```

The source interleaves reads of global root `016B4BE4`, its +5190 pointer and
that object's +74 word, B's first-dword ID and A's first-dword ID between FP
operations. At `E6A596` it passes A ID, B ID, retained EBP point, committed
distance and +74 word to E673A0. Consequently the next safe fixture slice is
an explicitly resumed post-arbitration state with independently supplied fresh
+90 fields, IDs, current +5190/+74 and point bytes while retaining the admitted
actor/point identities. It can compute the x87 distance under an explicit
control/stack policy and return a **pending E673A0 call**. It must not equate
this distance to the earlier SSE feature-center distance. Model errors on
nonfinite values, overflow, stale/foreign pointers and unsupported FP state are
safety policies. A meaningful independent test corpus should distinguish the
x87 operation order and binary32 commit, positive/negative zero, subnormal,
rounding boundary, and stale/foreign post-arbitration handles. A separately
reviewed original-byte host is required for native comparison.

## E673A0 gates and record mutation

`E673A4..E6740A` checks point X, Y, Z in that order, each lower then upper
bound, against globals `016B4C68..7C`. With masked exceptions, `COMISS/JB`
rejects out-of-bound or unordered comparisons; equality passes. It then calls
constructor `7B2350`, resolves globals through `E4CCA0`, and returns after
holder cleanup if resolved float +6C is ordered strictly greater than the
supplied distance. The +6C and +70 snapshots are provider-backed and can change;
the calls, pointer validity and cleanup are not implied by disk payload bytes.
The trace and exact subsequent stores are in
[cell E673A0 side effects](cell-feeding-e673a0-producer-side-effect-v1.md).

On the surviving route, source unsigned-orders the full IDs but forms its
lookup key from their low 16-bit halves. Different generations can therefore
collide; a matching key is not evidence of a matching full actor pair. It uses
E616C0 with a live comparator/configuration and calls E65810; vector insertion,
allocation, callbacks and movement require separate lifetime admissions. The
returned address is entry+4. On an admitted existing hit it updates entry +10,
then rereads the point and performs another E65810 call before writing entry
+14/+18/+1C. Entry +0C, IDs and +20 remain unchanged. The second lookup result
need not be the same allocation or entry without an explicit stable-container
and nonaliasing admission. On a newly allocated entry it writes ordered full IDs,
creation and current words, point, +20 flag (distance strictly above +70 under
ordered finite comparison) and zero handles at +24/+28. These are candidate
record fields, not a complete native vector implementation or contact event.

A meaningful subsequent state model could operate on an **owned** record with
an explicitly admitted existing-hit/new-slot decision and no external mutation,
returning separate pending effects around the two E65810 calls and holder
cleanup. Directed tests should cover full-ID collisions sharing the packed key,
unchanged first timestamp/flag on an existing hit, fresh point reread, new flag
at below/equal/above threshold, exact entry-vs-returned-pointer offsets, and
stale/reallocated handles. No map lookup or pointer stability should be claimed
as native until E616C0/E65810, allocation failure and their callbacks are
independently established. E673A0 mutates before E59CD0 and later E6A450
branches; it cannot be replaced by a frozen precomputed final selector.

## Distinct later routes

The E673A0 collection is traversed by conditional E64E90. Entry +0C must equal
the current +5190/+74 word and +20 must equal 1 for the impact category path;
that path resolves categories and can issue an audio command. Other records can
be updated or released/compacted based on signed comparison with `(current-5)`.
Those calls can change handles, storage and state; no particular cue or sound
follows from a new modeled record. See [contact event dispatch](cell-contact-event-dispatch-static-v1.md).

The selected E6A450/E59B40 output also follows a separate E7F170 -> E7E820
path after both ordered actor passes. E7E820 can construct an event record;
E7E160 may call its callback, then writes +08 and rereads selector/fields.
E7DBB0 re-resolves two actor IDs, and E7D8B0 can change current player/state.
The callback may mutate or invalidate the record, actor and provider state.
Revalidate the correct pool/epoch and fresh fields at each source read; a
pre-callback record or earlier output cannot substitute. Full gates are in
[feature producer and callback boundary](cell-feature-producer-callback-integration-v1.md)
and [feeding event dispatch](cell-feeding-event-dispatch-v1.md).

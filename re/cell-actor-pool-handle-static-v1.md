# Cell actor pool: handle, allocation and release trace

Static x86 trace of installed `D:\Games\SPORE Collection\SporeBin\SporeApp.exe`, SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. Addresses are virtual addresses. These are source-instruction observations, not original runtime execution or proof that all Cell call sites satisfy the required memory/lifetime preconditions. The actor and event pools use this same routine family, but are separate objects.

## Pool storage and initialization

`B71B70` allocates `stride * capacity` bytes and writes the following fields at its ECX pool pointer:

| Offset | Static behavior |
|---|---|
| +00 | buffer pointer |
| +04 | free-list head, initialized `FFFFFFFF` |
| +08 | allocation counter, initialized as global `0156C380 << 8`; then the global is incremented |
| +0C | live count, initialized zero |
| +10 | high-water slot count, initialized zero |
| +14 | stride in bytes |
| +18 | capacity in slots |

`E80D00..E80D13` passes stride `398` hex and capacity `1000` hex for the actor pool at Cell global +1C. `E80D30..E80D40` initializes the separate event pool at +54 with stride `7C` hex and capacity `200` hex. These constants are call-site observations, not portable pool format maxima. The 32-bit multiplication and allocation behavior are not a safe validation of arbitrary inputs.

## Allocate, resolve, release

`B71C50` compares live +0C with high-water +10. On equality it returns zero if live equals capacity; otherwise it allocates the slot at index high-water and increments high-water. On inequality it takes the index in free-head +04, reads that slot's first word as the new free-head, and reuses the slot. On success it increments the 32-bit counter and live count, forms the handle as `(counter << 16) | index` using 32-bit x86 operations, zeroes the entire slot by the supplied stride, writes the handle as the first word, and returns it. This behavior assumes internally consistent counts, head and slot allocation; malformed state is not validated. A zero return is normally treated as allocation failure, but encoding can itself generate zero, as explained below.

`B71CC0` returns null for handle zero, extracts `index = handle & 0xFFFF`, and checks `index > high_water` using signed `JG` before reading the first word at `buffer + stride*index`. If that word equals the full handle it returns the slot pointer; otherwise null. The native bound is **greater than**, not greater than or equal: `index == high_water` reaches a read despite referring beyond the initialized range, and when high-water equals capacity that address may be beyond the allocation. This observation must not be reproduced in a safe accessor. `B71D00` simply computes `buffer + stride*(handle & 0xFFFF)` and performs no zero, range, generation, occupancy, or equality checks; call sites using it rely on separate preconditions.

`B71D50` similarly extracts only the low 16 bits, writes the old free-list head to that slot's first word, decrements live count, and sets free-head to that index. It does not check the handle's high word or occupancy, and does not run an object destructor. Double-free, stale-handle release, out-of-range access, invalid count and broken head can corrupt native state. A source-faithful higher-level caller needs to perform any separately observed teardown before release; this routine alone does not establish the complete actor or event lifecycle.

The iterator at `B71C10` scans indices below high-water and treats a slot as live when the low 16 bits of its first word equal that index. `B71D20` calls this scan, returns the slot pointer, advances the cursor, and returns null at exhaustion. This inference relies on valid pool invariants: free-list links and corrupt/double-freed slots can make the test ambiguous. Iteration under mutation and reentrancy remains unproved.

## Generation, reuse and safe boundary

Only the counter's low 16 bits survive in the handle's upper 16 bits; the full +08 counter wraps as a 32-bit word. After 65,536 further successful allocations its encoded generation repeats, so reusing the same slot can make an old raw handle equal a new one. If the encoded generation is zero and the selected index is zero, the formed handle is zero, conflicting with the null/failure convention. The initial pool seed is derived from the process-global word, but the per-pool shift is not a guarantee of globally unique handles across pools or indefinite time.

`B71C00` sets live and high-water to zero and free-head to -1; it does not reset the +08 counter or clear the underlying buffer. `B71BD0` frees the backing pointer and writes `0000BAAD` to the pointer field, subject to its branch checks. Neither operation establishes the destruction of objects previously held in slots. A safe implementation should explicitly define reset/destroy ordering, reject access to destroyed storage, and treat any remaining handles as invalid even if raw words later coincide.

For a bounded source-mode implementation:

1. Model separate actor/event pools with validated capacity and stride, checked allocation sizes, explicit occupied slots, live count, high-water and a free-list whose membership has been validated. Keep opaque pool identity separate from the 32-bit raw handle.
2. Lookup must require a nonzero handle, `index < high_water`, `index < capacity`, occupied state, and exact full handle equality before exposing a slot. Never use the unchecked `B71D00` projection as a public accessor. Release must apply the same validation and reject stale, duplicate or out-of-range releases before changing counts or the free list.
3. Use the observed allocation/free-list order and handle construction only while an explicit non-aliasing generation policy holds. On zero-handle or generation reuse risk, return a typed exhaustion/ambiguity error instead of silently issuing a conflicting handle; a separate wider epoch can protect internal references but is a safety addition, not native 32-bit behavior. Reset and reinitialization must invalidate references even when words match.
4. Specify update/iteration mutation rules and staged teardown at the actual call boundary. Do not infer that allocating a slot alone creates a valid actor, or that freeing a slot performs the event callback or actor cleanup.

Acceptance should compare source-byte-bound native host behavior for ordinary creation, freed-slot LIFO reuse, count/high-water and iterator order with an independently reviewed bounded model, including stale and full-capacity cases. The invalid-access, zero-handle and wrap policies should be tested as Rust safety restrictions, not as claims that the executable handles those cases safely. A separate actor creation/contact/event integration test must verify real resource and player identity; this pool trace alone does not provide it.

# Source player pool binding: bounded implementation gate

This is a static synthesis, not an observed player. A fresh Capstone decode of
installed `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`
confirmed the following instruction sites. It did not execute original code.

* `E80D00..E80D13` initializes Cell global +1C with stride 0x398 and capacity
  0x1000. The next distinct pools at +38 and +54 use respectively stride
  0x304/capacity 0x200 and stride 0x7C/capacity 0x200. They cannot share raw
  handle identity merely because the generic pool format matches.
* `E74AF8..E74B14` allocates from +1C, preserves the returned word, then
  immediately uses `B71D00` to project it. `E74A50` later returns that same
  word on the documented normal branch. `E751C0..E751DE` publishes it at
  +411C and projects it again. Neither projection validates the word.
* `B71CC0` rejects zero and checks full slot word equality, but admits a read
  at index equal to high-water. `B71D00` only computes a low-16-bit slot
  address. The checked Rust pool intentionally requires occupied state and
  index strictly below high-water, and excludes generation ambiguity.

`E751C0` supplies ten constructor arguments as recorded in
[cell constructor](cell-start-actor-constructor-static-v1.md): global +40FC,
position pointer, zero, resolved globals +34 reference handle, -1, caller scale,
zero, 1, null, zero. The handle is a post-fixup definition reference, not the
raw `startCell` ID. The constructor requires the valid resolved definition,
RNG and FP context, and other helper results before an actor can be initialized.
It creates a leaf from the constructor triple before the player caller may
replace that triple; no unconditional leaf repair is demonstrated.

A useful next bounded fixture could allocate an **owned** checked +1C pool,
assign an explicit pool epoch, and construct a staged actor containing separately
admitted definition identity, constructor triple and transform. It should reject
allocation/lookup errors, carry the raw handle with pool identity, run the
explicit saved-state decision, and retain both pre-override leaf geometry and
post-override triple until an independently admitted update repairs the leaf.
A scripted sequence should test separate pools with colliding raw words,
allocation failure, release/reuse, wrong epoch and saved override. It must stop at
unknown callbacks and not expose the staged actor to gameplay. The current
`CheckedActorPool` already models ordering and safe local access, but does not
provide cross-pool/epoch ownership, actor initialization, or native lifecycle;
its existing Python comparison is synthetic, not original-instruction validation.

Remaining source gates are descriptor fixup/provider winner and lifetime,
actual constructor effects and RNG, saved state and manager callbacks, cache
association and mutable lifetime, tree/collection registration and update order,
and checked actor validity at each later contact/event access. Establishing a
handle-shaped word or matching source asset alone does not satisfy these gates.
Detailed baseline: [pool trace](cell-actor-pool-handle-static-v1.md),
[definition handle](cell-actor-pool-definition-handle-static-v1.md),
[player lifecycle](cell-current-player-handle-lifecycle-static-v1.md), and
[pool model](cell-actor-pool-checked-model-v1.md).

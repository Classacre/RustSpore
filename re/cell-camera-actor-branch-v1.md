# Conditional actor-to-camera interpolation

The reviewed original `E60A76..E60B0D` slice and original `B71CC0` lookup
matched four fixed cases on a non-switched private desktop. The cases use a
synthetic stable two-slot pool, supplied state +E0 handle and scalar 0.5; they
cover zero, valid, stale generation, and an index equal to high-water with an
actually allocated readable slot. The host stops before the following
`E833B0` provider. Native exit was zero, and the desktop closed after reaping.
Receipt `target/cell-camera-actor-branch-result-v2/receipt.json` SHA-256
`1b599ab3faaaf35b96f5f5da4cb42d9e021b40801e027208043249a638e1c15c`.
This does not establish a live actor, scalar provider, later camera input or
gameplay view. The original build-only v1 assembly failure is preserved.

`cell_camera_actor_branch_v1` is a separate checked projection of that lookup
and its ordered SSE interpolation. It requires explicit stable disjoint slots,
MXCSR 1F80 and finite normal/zero inputs and intermediates when a lookup
succeeds. It rejects an unreadable slot, including the source's permitted
equal-high-water read. On zero, out-of-range or stale handles it leaves the
supplied vector unchanged and the next provider pending; unused float inputs
are not evaluated. The comparison example checks all four retained pointer,
vector, state and FP rows under the synthetic pool and scalar admissions.

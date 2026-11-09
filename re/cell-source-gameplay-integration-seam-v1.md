# Source Cell gameplay integration seam

The experimental Bevy Cell application currently stores a prototype
`CellWorld` in `Game` and calls `world.advance(dt, input)` in
`crates/spore-bevy/src/cell_game.rs:946`. Its `CellWorld::step` in
`crates/spore-core/src/cell.rs:503` runs prototype movement, separation,
bite and food rules on prototype player/enemy/food values. Source art selection
in that application does not create a source actor or establish its handle.
Neither prototype entity ID, art asset ID, mass/health, nor order in a vector
is evidence of any native actor, source player handle, definition handle or
event identity.

A concrete integration seam is a separate, opt-in source Cell simulation
resource and fixed-step system adjacent to the existing Bevy update. It should
own one source actor registry, source player/current-selection handle, source
ordered-pair manager, event registry, and their epochs. Its inputs should be
explicit owned snapshots and verified resource bindings, with unsupported
stages represented as pending. Its output can initially be an inspectable
ordered transition/diagnostic stream, or a labeled controlled fixture frame;
it must not write prototype health, create feeding events from prototype
collisions, or present a scripted selector as a live native result. This can
be exercised end-to-end with synthetic inputs through the actual scheduling
seam, including stale/foreign/reused handles and callback-induced invalidation,
without pretending that controlled inputs are retail gameplay. The existing
`cell_source_sweep_fixture_v1` registry and the two-actor player fixture each
own independent checked pools; their matching raw words cannot join them.
A real source-mode implementation must share a single registry or use an
explicit validated ownership/epoch mapping rather than reinterpreting words.

The first source-backed frame that could advance beyond controlled pending
boundaries needs these independently established inputs and lifecycle rules:

1. Native resource mounting, overlay precedence and fixups for the selected
   CELL/creation and E4CCA0 +34 resource-handle result; an observed or
   reconstructed E750F0/E74A50 creation route and its assigned global +411C
   player handle. The CLI art ID and source fixture's checked handle do not
   prove that binding. Actor pool generations, release/reuse and stable object
   lifetimes must be represented across each source callback.
2. Current actor geometry inputs, source transforms, definition/cache and
   feature rows, source sweep registration/order/capacity and the frame order
   of mutation/iteration. The independently admitted sweeps and feature
   projections are not a source pair producer until these fields and ordering
   are owned by the same update. Source-selected feature identity and both
   E73E10 passes, E6A450/E6A2B0 candidates and E59B40 effects must be supplied
   by modeled producers, not the existing scripted fixture selectors.
3. Fresh actor +178/+179/+17B/+111/+112/+18C and E572E0 state, the retained
   pointer and fresh second lookup after E57890, and E68070's source-local
   snapshots, +248/+FC/cache +24, traversal inputs, current globals and FP
   context. The finite 537EE0 helper comparison validates only its stated
   disjoint input corpus; it does not establish these live values or calls.
4. Source event-pool allocation/re-resolution, handle ownership and callback
   timing, the intervening E69B00 actor continuation and its E68070 effects,
   E7E160 scheduling, E7DBB0/E7D8B0 side effects, then fresh event fields,
   current player and definition resolution before health/feeding updates.
   No assumed fixed player identity can cross a callback that can change it.

The nearest concrete implementation step is an opt-in, one-frame Bevy harness
that owns one registry and accepts explicitly labeled fixture snapshots,
then runs only the existing source-ordered registryâ†’sweepâ†’checked-resolve
handoff and reports the first pending producer. It must not accept source art
selection as player identity. This tests the scheduling and lifetime seam;
it does not resolve the native startup, selectors, or event scheduler and
therefore cannot by itself be called source gameplay parity. Promotion to a
real source simulation requires implementing and independently validating the
missing producers and fresh-input/lifetime rules above, not adding aliases
between prototype and fixture state.


The harness's `Update` system is one-shot: the first scheduled frame resolves
the explicitly selected fixture player, rebuilds the modeled sweep, resolves
each pair through the same checked registry, and records either rejected 3D
broadphase, unsupported input, or pending E7F170. It reports buffer suppression
and explicitly lists unmodeled actor creation, persistent sweep lifecycle,
selector/contact producers, callbacks and event dispatch. The default CLI
provides two synthetic actors and an explicit player index; it accepts no
source-art or native identity parameters. Subsequent app updates do not rerun
the frame. The focused tests exercise that schedule, capacity, strict gate,
invalid geometry and stale/foreign/reused player identities. The modeled sweep
rebuild is not a native persistent registration lifecycle, and reporting a
pending E7F170 is not selector, contact or event production.

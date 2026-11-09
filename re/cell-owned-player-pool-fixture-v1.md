# Owned source-player pool inspection fixture

The opt-in `--source-player` inspection now supplies its staged constructor
triple, independently controlled saved-state decision, pre-override projected
geometry, and eligible rows borrowed from its retained capability to a short-lived
owned checked pool. The fixture checks the admitted snapshot first word again,
allocates and resolves a pool-scoped handle, reports the retained inputs, releases
it, and verifies that the old handle no longer resolves. It deliberately does not
repair or update the pre-override geometry after a triple change. The current
inspection supplies an empty saved state, so a differing post-override triple is
exercised only synthetically.

The pool ID, strict checked lookup, stale rejection, generation limit, capacity
of two, zero counter and short lifetime are explicit fixture safety choices, not
native behavior or actual player state. The row snapshot and geometry use the
controlled package/provider, RNG, transform, cache, and no-concurrent-writer
admissions of the staged inspection. The snapshot checks only the first cache
key word, consistent with the observed cache comparison; it does not establish
full-triple validity. The fixture does not run E829D0 callbacks, source actor
initialization, actual leaf registration, contacts, or gameplay. The prior leaf
and identity boundaries are detailed in [the pool gate](cell-player-pool-binding-gate-v1.md).

# Controlled player triple decision

The source-player inspection now models the copy-only portion of the player
start path using independently supplied constructor and saved triples and mode.
The installed `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`
was independently decoded with pefile and Capstone at `E751E3..E7527C`.
Mode 1 copies all three saved words to the actor even when the first is zero,
and bypasses E829D0. Other modes copy a nonzero-first saved triple to the actor;
otherwise they copy the entire actor triple into saved state. Both non-1 paths
then pass the saved triple to E829D0. The model returns that call as pending;
it does not execute it or assume its downstream effects. It assumes valid,
stable, disjoint actor and saved objects with no concurrent mutation.

The inspection supplies mode 0 and zero saved words as controlled inputs; this
is not evidence of the actual +5190 object or its state. Its feature snapshot
is a separate controlled inspection, not a continuation of E829D0. Object
creation and current-player publication are described in
[cell player producers](cell-player-actor-pick-producers-static-v1.md); the
actual saved state, resource provider, actor handle lifetime and E829D0 manager
side effects remain unobserved. No gameplay identity follows from this model.

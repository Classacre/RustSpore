# Cell feeding event dispatch: bounded static trace

This is a static trace of the installed x86 executable, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
It does not establish runtime reachability, timings, complete feeding behavior,
or live RustSpore integration. Binary-derived disassembly and detailed dossiers
remain under ignored `target/`.

## Actor pair to event record

`E7F290` iterates a source collection, resolves each pair of handles, compares
squared position distance with squared summed radii, and calls `E7F170` on the
passing branch. `E7F170` requires different actor IDs, resolves both actors'
`+108` handles as definitions, and tests each definition's `+B4` for nonzero.
It calls `E73E10` twice with reversed actor and definition order using one
shared output, then invokes `E7E820` if the output selector is neither zero nor
`0x16`. The complete pair-generation and update scheduling are not established.
Evidence: `ev_534da495e379f635f0b9a830b402ba3df103ff32f2b954c3893292e4ffd971c3`,
`ev_83f69f5a29c4cec6c87bb95778fb9eaf71c85a6b9db159650907df4489b423f7`.

`E73E10` routes according to whether each actor has nonzero values at both
`+248` and `+FC`, calling one of `E6ADE0`, `E73660`, `E73930`, or `E73BF0`.
The selector-producing rules within those functions remain to be recovered;
Ghidra reports terminal-call flow limitations at these sites. Evidence:
`ev_df8c3ea3f2c32c70f7808861c80960c2bfeade9437ec0f9696261043209e109c`.

`E7E820` resolves the two IDs in its input record. Case 10 calls `E7E050`
with the first resolved actor, input `+0C`, literal **2**, and the second
resolved actor in ESI. Subject to its gates, `E7E050` calls `E51D40` with
selector 1, the first actor's ID, raw word 2, and the second actor's `+108`
definition handle. Case 9 directly constructs a selector-1 event with raw
word **3** and the second actor's `+108` handle, subject to its own gates.
When the second actor's `+179` byte is nonzero, `E7E050` can return 1 without
constructing an event; that return value alone is not proof of construction.
`E51D40` allocates and resolves an event record, stores selector at `+24`,
actor ID at `+28`, raw word at `+38`, definition handle at `+3C`, initializes
callback `+4` and byte `+8` to zero, and returns the allocation handle. The
case 10 caller then sets the callback and a float at `+0C`; the relevant case 9
path does the same. Evidence: `ev_cc68060d3e89e03734f5f334ab2836533acbeac53c2bd1d13a0b453ec65d904f`,
`ev_6ab17ba4cba508f1e1ab9af85d5a5ef08f552a1bb6ec8d2c4571dac37c3aa4a9`,
`ev_25502ffdd4b4a51e1e4b01ef413eb1755d339482ed20dd02a58a39ef4003d5db`,
`ev_6712ddffacdca6d580e1529da634827b8512684b146ef0808d91fec70b8bd493`,
`ev_8620d3c5c39b7b346142d477d339679f1229515f31423449dfdf835323cb831c`.
The correspondence between a constructed record and a later dispatch remains
conditional on its lifetime and intervening changes.

## Callback and feeding

In `E7E160`, before dispatch, a callback is called only if record byte `+08`
is zero, pointer `+04` is nonzero, and the low byte of the third argument is
zero. After that call, the procedure sets byte `+08` to one, then dispatches
based on the current `+24`. The observed callback at `E7DBB0` resolves actor
IDs `+28` and `+2C`; if both resolve it passes the objects and raw word `+38`
to `E7D8B0`. If either does not resolve it returns without that call. The caller
ignores the callback's return value. The callback itself has no direct writes
to `+38` or `+3C`; downstream aliasing or mutations remain unproven.
Evidence: `ev_a66d55179cd6b40f2611da8f3a031053ff7b39284a7b155d1d236a67d268ff58`
and the Ghidra-confirmed bounded callback decode under
`target/source-cell/feeding-callback-routes-linear-v1.json`.

`E7D8B0` has consequential branches. If its second actor is the current player
and save byte `+E8` is nonzero, it changes the global player ID to the first
actor and adds 150 to save progress `+1C`, followed by further calls and writes.
Other paths change actor state, may increment save `+6C`, and invoke other
callbacks or allocate another record. Therefore an earlier snapshot cannot
stand in for state after this callback. Evidence:
`ev_9806cf588f4a06ac5c20dbe257f96ebead6aa9ef3255b964a0c460c1733fe7fb`.
`E50830`, called on one branch, returns true for null definition; otherwise,
when definition flag bit 0 is set, raw word 2 or 4 yields false and 3 yields
true; other cases compare definition `+B4` to 3. Evidence:
`ev_91d98ecd7b0cdf14186b463c1cda182b29f5b1f37b92d188c8f6d1e8d1dced27`.

For selector 1, `E7E160` forwards the current record `+28`, `+38`, `+3C`,
byte `+08`, and its float argument to `E7D830`. That procedure resolves the
actor ID and requires nonzero gate byte before forwarding to `E7D790`.
`E7D790` calls `E7D0A0` only when the resolved actor ID equals the current
player ID; it passes the raw word as first stack argument, the definition handle
in EAX, and the float as second stack argument. `E7D0A0` compares EAX to the
special global at `016B4BF4`: equality skips definition lookup; otherwise it
calls `E4CAA0`/`E823D0`. Valid handle, object lifetime, callback effects and
fresh snapshots are necessary admissions. In particular, a null resource
acquisition must not be assumed safe: the observed resolver has a zero-base
read on its absent-object path. Evidence:
`ev_c63759f9f8a705570a360df74f48fe24f726d322ef5a6c5647474398aac692fe`,
`ev_a087b3ecd9eeb1cc0636f12c2f76929366b2e7863a6ce2b60c854a7052ca2291`,
`ev_5e6ce788c28246e92f4ffd658ea8d119e88ab6ddafef3d9da8c4cf556d470baf`;
the `E7D790` dossier is `target/rea/20261028-source-runtime-v6/125-analyze_function.json`.

Two other direct calls, at `E816F2` and `E8172D`, pass that special global,
raw word 3, and float zero. These take the equality branch and cannot establish
ordinary definition selection. Their surrounding bounded linear decode does not
by itself establish function boundaries or reachability.

The next source targets are the four `E73E10` branch callees for selector
production and actor-role conditions, and the event scheduler/callers of
`E7E160` for record lifetime, timing, and intervening writes. Zero-food cleanup,
post-feeding growth and transition effects also remain separate. The current
prototype's float health, mass-based progress and abstract food do not provide
these source states or admissions.

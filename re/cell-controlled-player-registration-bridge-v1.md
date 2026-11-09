# Controlled player registration bridge: static source trace

Read-only Capstone decode of installed `SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The instructions below provide a conditional way to replace the installed
frame fixture's independently supplied *player* tag and 2D bounds with a
source-ordered computation from its already admitted staged geometry. They do
not identify an actual player or prove that the live registration occurred.
No source model or game execution was performed in this trace.

## Identity and registration order

On the conditional E750F0 route, `E7516C..E751C0` supplies ten E74A50 stack
arguments. In particular, argument 1 is a snapshot of Cell global +40FC at
`E751B3..BF`, argument 5 is `FFFFFFFF`, argument 10 is **zero**, and argument
4 is the runtime resource reference loaded from E4CCA0 result +34. The last is
not the raw CELL key. `E74AF8..E74B14` allocates an actor-pool handle, saves it
and resolves it unchecked; `E74E67..E74E94` sets actor +FC/+100/+104 from
E656C0's selected triple. A local fixture handle can represent this modeled
allocation but is not the source handle.

`E74F5B` calls E6D7F0 on the newly initialized actor and stores its center and
scalar into locals. `E74F60..E74F90` passes those values through E86960 before
registration. E86960/E868F0 allocate and link an object and copy values from
the passed center pointer (`E86960..91`, `E868F0..55`); allocation failure,
aliasing, callbacks/other writers, and any effects on the geometry/manager
state must be separately admitted. At `E74F93..9D`, the constructor compares
its original argument 1 with the **current** global +40FC. Only equality
enters the registration arm; inequality sets actor +364 to FFFFFFFF. The
source does not establish that +40FC cannot change during intervening calls.

On the equality arm `E74FA3` reads argument 10; because this caller supplied
zero, it branches directly to `E74FCD` with ECX/tag **0**. The alternative
argument-10 FFFFFFFF branch checks definition +B4 and sometimes chooses -5 or
the saved actor handle; it is not the admitted E750F0 caller's zero-tag path.
No CELL kind, command-line art ID, or fabricated player ID is needed to derive
that zero tag under the stated route and unchanged-manager admission.

## Source-ordered bounds

`E74FCD..E7501D` is 81 original bytes, SHA-256
`46562e916d7ec57d46b974da4dc39653adfeb2c2a0059916f2f1fc2e71577ac6`.
With initial ESP S at E74FCD, center x/y at `[S+30]`/`[S+34]`, and scalar at
`[S+80]`, it executes binary32 SSE operations in this order:

1. `x_max = x + scalar` (`E74FE0`), store at `[S+28]`.
2. `y_max = y + scalar` (`E75000`), store at `[S+2C]`.
3. `x_min = x - scalar` (`E75004`), store at `[S+18]`.
4. `y_min = y - scalar` (`E75008`), store at `[S+1C]`.

The addresses account for the source's three intervening pushes. It then
passes the min and max vectors, saved actor handle and tag to `BBB7F0`
(`E7501E..26`) and stores the result in actor +364. This arithmetic precedes
post-return saved-player triple handling at `E751F1..E7527F`. Thus the current
`StagedPlayer.projected` pre-override geometry is the appropriate *controlled*
input to a modeled registration-bound construction, provided its admitted
E6D7F0/cache snapshot and E86960 nonmutation/stability remain in force. It
remains based on the controlled seed, `[100,100,0]`, sampled matrix/scale and
modeled cache, not observed live actor geometry.

## Minimal safely testable change after the suite

Add a separate source-ordered registration-prefix model accepting the exact
center/scalar words, original/current +40FC snapshots, argument-10 tag, and
explicit post-E86960 stable/disjoint admission. For the explicitly modeled
player call use tag 0 and equal manager snapshots; otherwise return skipped or
pending rather than silently asserting either. Under scoped MXCSR 1F80,
perform the four binary32 operations above in order, preserve bits and reject
nonfinite/subnormal or tied/invalid bounds through the existing sweep
admissions. Feed the resulting tag/bounds and the same registry-owned actor
handle to the installed one-frame fixture. Keep a caller-supplied bounds mode
as a separately labeled fixture if desired, and preserve a separately
specified peer: this bridge only removes the player's independently invented
bounds and tag. It does not implement E86960, BBB7F0's persistent collection,
actor startup identity, or post-override cache updates.

A focused native comparison can safely isolate the 81-byte bounds slice with
synthetic guarded stack/actor, relocated-or-skipped subsequent calls, explicit
MXCSR control, all four raw result words, positive normal values, signed zero,
cancellation and adjacent rounding. It must independently check complete
instruction boundaries, original nonpatch bytes, stack/push layout and the
failure receipt before execution. A separate branch/argument test should cover
unchanged versus changed global +40FC, direct argument-10 zero, and the -1
fallback without pretending the global is stable. Tests should also reject
zero-width, endpoint ties, stale fixture handles and unsupported FP input.
Compare with original instructions before describing the derived bounds as
independently validated. The proposed change is an inspection bridge, not live
source player identity or native gameplay parity.


## Controlled original-instruction comparison

The independently reviewed host compared the combined gate, tag and bounds
slice `E74F93..E7501D` (139 bytes, SHA-256
`1dfb287d6f32cb234db45c1077978173465b5b8115aecf428d8cdc3beed22aa8`)
against the bounded registration-prefix model. All eight cases matched the
skip/registration decision, tag and four result words. The
[receipt](../target/cell-registration-prefix-result-v1/receipt.json) SHA-256
is `01cc262750753310f629c9565a2fc89c672decd037a723483f789d7945638748`.
Model and native stdout SHA-256 values are
`a3c7edf9057477cc2cf62b70d2ae11552dbbc3875e64a0e8079e3d30c9bbdd6f` and
`6d908fa1138e92f779d42dfbaf1187ef994ad5e1ce70a705ee7200c6e256708e`;
both exited zero with empty stderr. The model source SHA-256 is
`3e274119577a50896b40a3a1922f122f43079fde6a75ef38b23f3d7a5fe1636a`;
the host and runner hashes are
`98bd81a95b58b5a1e41f3a28a167b51cfda8a7476491cac9fa05e64c18e9315f` and
`b0455177682a035626c2932069432544f6fe50dd8d0c0d6ffdc358fcd8a89789`.

Corpus tuples below are hexadecimal `(original manager, current manager,
argument 10, definition +B4, actor word, x, y, scalar)`.

```text
00000009 00000009 00000000 00000002 00010000 42c80000 c0000000 3f000000
00000009 0000000a 00000000 00000002 00010000 40000000 40400000 3f800000
00000009 00000009 ffffffff 00000005 00020001 40800000 40a00000 3f800000
00000009 00000009 ffffffff 00000007 00020001 40800000 40a00000 3f800000
00000009 00000009 ffffffff 00000002 00020001 40800000 40a00000 3f800000
00000009 00000009 00000016 00000002 00020001 4b800000 40a00000 3f800000
00000009 00000009 00000000 00000002 00020001 80000000 00000000 00000000
00000009 00000009 00000000 00000002 00020001 3f800000 bf800000 3f800000
```

The host redirects only the manager-mismatch near jump to a synthetic exit;
the registration arm captures the pushed tag, unwinds its three pushes and
returns without calling `BBB7F0`. Guarded synthetic manager, definition, actor
and stack data are admitted disjoint and stable. Linked nonpatch bytes, full
source instruction boundaries, branch target, synthetic exits, stack/guards
and pre/post pins were checked. All rows recorded x87 control `027F` and
status `0000`. MXCSR was `1F80` except the rounding case's `1FA0`: its sticky
precision bit was recorded, while controls (masked sticky bits) matched
`1F80`. This finite corpus does not execute E86960, actor allocation or
lookup, persistent registration, callbacks, real source identity or gameplay.
The model rejects nonfinite results and restores incoming MXCSR; it does not
claim the native routine's general exception or sticky-state behavior.


## Controlled installed-frame integration

The installed one-frame diagnostic now accepts `--source-registration` only
with `--admit-registration-stability`. This admits equal original/current
manager words and stable, disjoint post-E86960 geometry, uses the controlled
E750F0 argument-10 zero path, computes bounds from the staged pre-override
geometry, then passes them to the same synthetic registry/sweep. It does not
run E86960 or BBB7F0, establish an actor handle, or claim native registration.
Caller-supplied `--tag`/`--bounds` remains a separate mutually exclusive mode;
peer inputs remain independently supplied. Existing sweep checks reject
invalid intervals, unsupported endpoints and endpoint ties before registry
creation. The model installs/restores MXCSR 1F80, rejects nonfinite input or
results, and does not expose source sticky-state side effects.

Focused core tests passed 2/2; frame tests passed 8/8 with two installed checks
ignored by default. The exact-install derived-mode test was then explicitly
run with seed 123, progress 0 and stable-file/registration admissions: it
passed with controlled tag 0 and bounds approximately
`[99.4547,99.24996]..[100.964966,100.76023]`. This confirms the modeled frame
handoff against pinned installation data, not the live player or manager.

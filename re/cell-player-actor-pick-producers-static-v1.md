# Cell player actor and initial pick geometry: static boundary

Trace against installed `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
No original code or live game was executed. The explicit PatchData
definition, structure and kind-4 list candidate and its four type-zero
creation IDs are pinned in
[cell-start-actor-constructor-static-v1.md](cell-start-actor-constructor-static-v1.md)
and [cell-kind4-cache-selection-static-v1.md](cell-kind4-cache-selection-static-v1.md).
These source records do not establish active reference fixup or actor identity.

## Position and scalar

The traced caller `E803D3..E803D8` passes image vector `015A8D3C`
`(100,100,0)` and an SSE-computed binary32 scalar to `E750F0`. At
`E8036F` XMM0 is loaded from `14719E4` (2.0); `E803C5..E803CE` multiplies
the live Cell global `+514C` by that value. This resolves the earlier note's
uncertainty about that multiplier; it does not make the global itself a
constant. At `E802EF..E8033A`, the code chooses a 28-byte progression-table
row from the signed input word saved on its stack, uses the special row for
1000, reads that row +10, multiplies by image constant `1471B2C` (0.5),
and stores the result at global `+514C`. The source table and input range
must be admitted; do not extrapolate its unbounded native scan.

`E750F0` resolves the global descriptor object and passes its +34 reference
handle as the definition input to `E74A50`. It passes size-index `-1`,
argument 7 zero and argument 8 byte one. The constructor resolves the
definition, and on the zero size-argument branch calls `4DF9A0` with the
definition's two float words at +304/+308; even equal endpoints do not
establish that the RNG call is skipped. It computes an x87 value using the
sampled result, a 1.0 factor for index -1 divided by live global +514C,
and the supplied `+514C * 2.0` scalar, then stores binary32 to actor +B4.
Do not simplify away the operations: x87 context, zero, overflow and
intermediate rounding are material. `E74D4A..E74D5D` copies +B4 to actor
+58 only when argument 8's byte is nonzero; it is one on this caller.

`E74B22..E74B34` copies the supplied position into actor +08/+0C/+10,
and `E74C10..E74C2E` copies those words into current +4C/+50/+54.
Depending on definition +AC, supplied orientation pointer, and RNG, it
constructs a quaternion and calls `59C000` to store the nine-word matrix
at +5C; actor +48 bit 02 is set. Thus exact orientation and float controls
are required when projecting nonzero-cache geometry. The constructor writes
actor +112 zero and +358 from the argument whose player-call value is
`FFFFFFFF`; the selection filter's progression fallback is then relevant
if that value remains unchanged. These are constructor-route values only.

## Triple, cache, and leaf timing

`E74E67` calls `E656C0` with the resolved definition reference and copies its
result to actor +FC/+100/+104. On the explicit kind-4 cache-miss path, the
structure and list handles must resolve and `E5C0F0` makes its ordered
weighted selection with actual RNG/FP state. Cache hits, other kinds, default
and type-1 paths are separate; the chosen triple is not determined by the
disk list alone. The current tree owner is the constructor's first argument.

Before returning, `E74F5B` calls `E6D7F0` using the constructor-selected
triple, current position, scale, matrix and the `E67A50` cache result.
`E86960` allocates and queues a leaf with that geometry and actor handle,
storing its pointer at +360. For nonzero +FC the geometry needs the
correct matching cache +1238..+1244; the explicit source resource
composition models are not a live provider or cache binding.

After constructor return, `E750F0` records its handle at global +411C
and can replace the actor triple with saved words from global +5190
(`E751F1..E75235`). It then calls `E67A50` on the post-override triple
(`E7527F..E752A3`). The initial leaf was already populated using the
pre-override triple: no unconditional repair or identity equivalence follows.
If `E7C8F0` reaches `E7BFE0` and that result is true, `E7CB2D` calls
`E7C0B0`; all decoded branches inside that routine reach `E7C3B3`, which
recomputes actor geometry and invokes `E870B0`. The latter overwrites leaf
geometry and conditionally removes/requeues it when parent containment fails.
Whether this update runs after override and before selection is unobserved.

## Integration gate

Actual application selection needs the live/admitted player and candidate
actor pool, definition and reference-fixup provider, saved-state branch,
cache state and lifetime, tree owner/leaf lifecycle, current cursor camera
and event routing. Current prototype `CellWorld` and its artwork supply none
of those source identities. An explicitly selected installed definition and
creation candidate can support a controlled analysis with all other values
supplied, but cannot establish original player, initial tree or input behavior.
The query excludes the current player, so obtaining that one source actor
alone also does not populate the candidate set.

## RNG initialization and application implications

The candidate-specific equal-size, null-orientation, quaternion and
kind-4 selection modules can consume the pinned installed records with
explicit source-state admissions; none currently establishes the actual
selected live player or source RNG history. `12C17B0` pushes `FFFFFFFF`,
sets ECX to global RNG word `01602760`, and calls `935D20`. That helper
uses `RDTSC`'s low word for input `FFFFFFFF` or zero and otherwise stores
the supplied word (`935D20..935D31`). The image pointer at `13D89B4`
contains `12C17B0`. Normal CRT startup `11E0F90 -> 11E0CD0` passes
`13CC80C..13EB098` to `MSVCR90!_initterm` (`11E0D7E..11E0D88`); the
pointer lies inside this initializer range, before the later `F48910` call
at `11E0E0B`. This is static startup evidence conditional on that CRT
initialization branch succeeding, not a measured seed.

A bounded raw `.text` scan found 50 candidate direct calls to `935D20`;
only `12C17B7` had the literal `mov ecx,01602760` in the preceding 50
bytes. The same scan found 368 raw direct-call candidates to `935D40` and
588 literal references to `01602760`; byte matching is not a decoded call
graph, and register aliases, indirect calls, alternate stores, initializer
callbacks and asynchronous mutation are not excluded. In particular, the
RNG state at constructor entry and at the later kind-4 selection cannot be
reconstructed from the executable image or installed resources.

A fresh source-oriented application may explicitly choose and retain its
own nonzero seed, then execute the bounded source generator under admitted
FP policy; this is an application-controlled run, not the historical retail
seed. Original zero-state behavior also invokes `RDTSC` and is outside the
current checked generator. Its constructor must still account for any RNG
changes through allocation and resolver callbacks between the size,
orientation and selection draws. Exact live equivalence requires an observed
entry state or a complete, controlled initialization and call history;
calling `RDTSC` in another process cannot recover the original seed.

The first two constructor draws admit a narrower conditional ordering than
the later selection. On the explicit player path, `4DF9A0 -> 4BA080` makes
the size draw. Before the null-orientation `935D40` call at `E74B9D`,
`E74A50` calls pool allocator `B71C50` and projection `B71D00`. The latter
has no calls; `B71C50`'s only call is `11E052E`, the `MSVCR90!memset`
import, to clear the selected pool slot. With successful allocation, valid
nonaliasing pool memory and no concurrent/global mutation, the orientation
draw follows the size draw in the source recurrence. This does not cover the
later `E4F830`/`E656C0` reference callbacks and cache insertion before the
kind-4 selection draw; their effects remain explicit gates.

The enclosing `E7FD30` performs cleanup before `E803D8` calls `E750F0`: it
iterates existing pools, removes leaves, can release actor resources, and
contains several indirect calls (`E7FFF3`, `E8000A`, `E800B5`, `E800C6`,
`E8012B`, `E8013D`, `E80150`) as well as descriptor resolution `E4CCA0`.
A complete linear decode `E7FD30..E803DC` has no direct call to `935D40`,
but these transitive/indirect paths and earlier initialization remain
unbounded. This cannot establish the first Cell-stage state from the CRT
seed or prove a particular number of intervening draws.

## Fresh-state source branch and side effects

The word `016B4DC0` lies beyond the file-backed raw bytes in the PE `.data`
virtual extent, hence is initially zero-filled by image mapping. This is an
initial image fact, not its value at a later Cell transition. `E63CC0`
allocates an `E61390` object, installs it at `016B4DC0`, and passes its
input to `E62750`, which copies source state +0C/+10/+14/+18 and further
fields. `E81B64` can also assign `016B4DC0` from current Cell global +5190.
At `E80D75..E80DF4`, an existing `016B4DC0` is reference-assigned to +5190;
otherwise `E61470` creates a new object and assigns it to +5190. The
nonzero global is cleared/released after that transfer. Later writes,
callbacks, object lifetime and all indirect assignments are not exhaustively
proved by this bounded trace.

In the new-object branch, `E61390` initializes +10/+14/+18 to zero.
`E61470` resolves CellGlobals and writes its +00 `game_mode` to object
+0C; the explicitly pinned PatchData globals have mode zero. For mode 0
or 2, `E614B0` invokes `E82950` and copies its triple to +10/+14/+18.
`E82950` calls `B3D3A0` (global `0167FAC0`) then `B1FD90`, which returns
object +74. If that field is null it returns a zero triple; otherwise it
copies words from the returned object's +68C/+690/+694. A bounded literal
search found the getter but not the initialization or mutation path for
`0167FAC0` or its object +74. The null case is therefore a precise explicit
fresh-app assumption, not a proven condition of retail new game.

If the new object has mode zero and its triple is zero at `E750F0`,
`E75237..E7524F` copies the constructor-selected triple to object
+10/+14/+18 rather than overriding the actor. `E75252..E75277` then invokes
`E829D0`: it obtains object `[015D1C24]`, copies the triple to its
+A4/+A8/+AC (`4DF9E9..4DFA02`), calls `67C680` virtual +58 when first
word is nonzero, and sends a constructed `03795725` message through
`67D7E0` virtual +14. An application can initialize its own empty resume
state and implement its own explicit triple storage and event sink, but the
original external service callbacks and their effects are not established.
Such a controlled fresh run may proceed with recorded assumptions rather
than calling a fabricated retail provider; it must expose the unimplemented
notifications and avoid claiming replay-equivalent source state.

## Triple notification services and a controlled empty branch

`67C680` reads global `015FDC74`. On the initialization sequence at
`5D71EC..5D71F8`, `55C550` constructs and installs an object; `559030`
sets its primary vtable `13F4460`, where slot +58 is `55F930`. This identity
is conditional on that initialization and no replacement. `55F930` calls
`563110`, which searches the +14..+18 sequence in 0x50-byte strides for the
full three-word key. A missing row yields `FFFFFFFF` at `5631EB` and
`55F930` returns false without its index/timestamp update. With a match and
flag 1 it consults an index, conditionally calls `5618A0`, sets byte +1D0,
and stores a `92E040`-produced two-word value at row +48/+4C. `5618A0`
itself consults the service from `67D850` and processes further state; its
transitive effects are not bounded here. The caller ignores the boolean.

`67D7E0` reads `015FE890`. The initialization path `7E8B7D..7E8BB4`
constructs via `884AD0` and installs this object; its primary vtable
`1428550` has +14 = `885320`, +18 = `885800`, +20 = `883BC0`. The +14
handler has a direct-target path when supplied a nonnull target; otherwise
it walks wildcard and ID-specific subscriber buckets and calls collected
listeners. At `FDD110..FDD151`, another object conditionally registers
its +4 interface for ID `03795725` through service +20. Constructor
`FDCDC0` assigns that interface vtable `1494AC0`, whose +4 is `FE3290`.
There the event case `FE3597` requires nonnull `1021930` and a `B5B5E0`
value other than `01654C10`; it extracts message fields, calls `4DFC20`
on the object from `401090`, forwards the result to `C33A60`, and sends
`0490D429` through service +18. `C33A60` accesses further service and
object state. This identifies one conditional recipient, not a complete
listener census or measured delivery; its transitive influence on Cell
cache or actor state is unproved.

The smallest runnable controlled new-app slice can explicitly initialize
an owned empty `55F930`-style record set and owned empty event subscriber
registry, preserve the selected triple, record the source message payload,
and execute the traced empty-record / zero-recipient branches. These are
real owned application states, not a substituted retail service or an
assumption that retail has no subscribers. A nonempty or unknown collection,
listener, callback result or side effect must remain unresolved until modeled.
This bounded path can advance to source cache derivation only in the new app's
controlled context; it does not establish original provider precedence,
callbacks, RNG state or runtime actor identity.

## Opt-in staged application inspection

`rustspore --source-player --install <dir> --seed <nonzero decimal>` now
reads the explicit installed PatchData globals, definition, structure and
list, and checks the selected `Spore_Content` CreatureData candidate. It
checks executable, package and individual decoded-resource hashes before
and after the read. The path must be trusted/stable throughout: before/after
hashes cannot exclude transient replacement during reopen. It does not use
ignored research artifacts.

The mode models the source progress-0 branch with explicit PC64/nearest
x87 control `027F`, MXCSR `1F80`, source initializer angle/axis, fresh owned
selection cache and controlled seed. It admits disk list rows as the runtime
snapshot and assumes intervening resolver/allocator operations do not mutate
the RNG; prior actor allocation is only assumed, not performed. The modeled
`3D97A8E4` triple and separately authenticated `0F43029A` creation candidate
are printed as distinct identities. No reference callbacks, full constructor,
retail cache/provider, actor publication or selection input is executed.
The command stops with `UNRESOLVED` geometry/property/cache providers and
exit code 2; this is staged inspection, not a playable player or a completed
gameplay integration. The null-orientation transcendental calculation is
host-dependent and remains outside original-instruction comparison.

Reproduction on the pinned installation:

```powershell
tools\build.bat build -p spore-bevy --bin rustspore
target\debug\rustspore.exe --source-player --install 'D:\Games\SPORE Collection' --seed 1
```

Two independent invocations of the same build produced identical stdout and
stderr, each ending at the expected unresolved status with exit 2. The build
SHA-256 is `83a0458aec3fc072e4f91b5af8a7993b450717d1fb5352f6818c267a56199a5d`;
run records and raw output are in
`target/source-cell/controlled-source-player-run-v2`. The focused
`cell_source_player_v1` test passed 1/1 through the wrapper. The earlier v1
run used pre-review diagnostics and remains historical, not the final CLI
contract.

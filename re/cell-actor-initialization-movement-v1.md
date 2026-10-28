# Cell actor initialization, movement state and AnimList priority

This is a static trace of the base executable SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
It does not show a live actor, an admitted selector row, a loaded animation,
or a bound or playing clip.

## Initialization

[MEASURED] `E6D950` resolves a Cell-side actor and obtains a handle through
`B71C50`. At `E6DA81` it stores the handle to actor +248, resolves it through
`B71D00`, and calls `E5CCE0`. The latter's initialization path associates an
object at its +24 through `E4EE10`; the request interface on that object is
traced in `cell-actor-animation-request-trace-v1.md`. After initialization,
`E6D950` resolves the actor again and calls `E6D260` with context 0, requested
state 0 and previous state from actor +1B0. `E6D260` only proceeds if actor
+FC and +248 are nonzero; it sends a state-derived key through `E4F190` and
updates actor +1B0 and +1B8. This is a source call path, not evidence that
those gates passed or the resulting virtual call used a particular vtable.
REA dossier 800.

## Movement and refresh

[MEASURED] `E6EF30` has a state-dependent movement branch. For its supplied
mode 0, `E56960` compares the distance between actor positions +4C..54 and
+08..10: it clears actor +368 below distance 1 and, when distance is at least
3 and +368 is at most 1.5, increments +368 by the supplied delta and returns
true. The caller uses that Boolean to write state `30` or `12` respectively
at actor +18C. Other mode values can write states `14`, `13`, `32` or `33`.
These are hexadecimal state values. Under the branch that calls `E557A0`, a
false result can reset existing states `12`, `13`, `14`, or `30` to 0.
Input routing, floating-point context, movement helper effects and all
other branches must be preserved before interpreting these as complete
interactive controls. REA dossiers 803–805.

[MEASURED] `E7C8F0` contains calls to `E6D3A0` at `E7CA5E` and `E7CA91`.
`E6D3A0` compares the desired state at actor +18C to the current state at
+1B0. A difference calls `E6D260` with the desired state. When they match,
it decrements +1B8 by supplied delta and, once the resulting duration is at
most zero, calls `E6D260` using the state at +1B4. The conditions for reaching
each call and other requests in `E6D3A0` remain material. REA dossiers 785, 796.

## Duplicate AnimList keys

[MEASURED] `A0E100` initializes the temporary record's priority at +14 to
positive zero. The version-7 path through `A0E8C0` leaves this default in
place; version 10 reads its priority from the record. For an existing request
key, `A0EE1F..A0EE29` compares incoming priority to existing priority and
skips replacement on less-than or unordered; an ordered equal or greater
value replaces the existing record. Equal priorities therefore depend on
enumeration order. REA dossiers 677 and 806.

[MEASURED] A read-only scan of the four indexed installed AnimLists found
requests `AAAA0015`, `AAAA0021`, `AAAA0060`, `AAAA0091` and `AAAA0092` in
PatchData version 10 and Spore_Game version 7. Their respective paths and
animation instance words match between those two lists. PatchData's five
version-10 priorities are 1.0; the version-7 records use the 0.0 default.
The keys did not occur in the other indexed version-7 and version-10 lists.
If both records are actually enumerated and parsed, and no additional source
competes, the 1.0 record wins regardless of their relative order. This does
not establish mounted-resource enumeration, ANIM package precedence, or a
specific resource being loaded or bound. Index coverage is limited as noted
in `cell-animlist-resource-selection-v1.md`.

The next bounded bridge can accept explicit state, actor/part and predicate
outcomes, and a supplied committed draw, then join an admitted request to a
caller-selected AnimList record and ANIM instance. Such a bridge would be a
supplied-context demonstration. Authentic selection additionally requires
the actual actor and world inputs, selector RNG and timing, mounted resource
resolution, binding and controller evidence.

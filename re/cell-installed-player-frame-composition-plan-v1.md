# Installed staged player to the one-frame source fixture: bounded plan

The controlled one-frame harness in `cell_source_frame_fixture_v1.rs` owns one
`Registry` and admits `ActorSnapshot`s, an explicit player index and pool seed.
It currently constructs synthetic snapshots; it does not create source actors.
Its registry's scoped handle is generated locally and is not interchangeable
with native pool handles or the independently owned player fixture's handles.

`cell_source_player_v1::inspect` can safely supply a *controlled staged input*
under its existing explicit seed/progress and file-stability admissions. It
hash-checks the installed executable and PatchData/Content packages, reads the
fixed CELL key `CBCD1287:DFAD9F51:00000000` and pinned supporting resources,
selects one of a bounded set of pinned creations, classifies its rows, computes
capability data, and produces `StagedPlayer` with modeled triple, matrix, scale,
part rows and projected geometry. Its projection uses an explicitly synthesized
actor at `[100,100,0]`, controlled flags/matrix and a modeled capability cache;
it is not a fresh E67A50 cache or observed source actor. It does not execute
E829D0 or publish a native actor. The installed package paths must be trusted
and remain stable; before/after hashes do not prove atomic file stability.

A bounded composition is feasible without inventing a runtime actor identity:

1. Add an opt-in, separately named installed-data frame fixture accepting the
   install directory, nonzero seed and bounded progress. Invoke `inspect` once
   and retain its staged values as one explicitly selected fixture snapshot.
   Transfer the data into the *one existing frame registry*, allocating its own
   checked handle there; do not copy raw handles across the two existing pools.
2. Expose the hash-verified CELL resource bytes and exact package/key/digest
   already read by `inspect`, or pass an owned typed provenance record returned
   with them. Preserve the CELL definition key separately from the selected
   creation triple and creation hash; neither is the synthetic registry handle
   or a native resource handle. Preserve the feature rows/first-word binding if
   the fixture claims to retain them, using owned data rather than a borrowed
   temporary. An independent later package re-read needs its own exact key,
   hash and stable-file admission; do not silently mix two file versions.
3. Use `StagedPlayer.projected` only as its labeled modeled **pre-override**
   geometry, with its synthetic position and controlled cache noted in output.
   For an empty saved triple, the existing triple decision retains the selected
   triple, but its E829D0 call remains pending. A nonempty saved-player override
   can change the actor triple and invalidate the first-word/cache join; reject
   or stop pending unless its fresh matching cache is independently admitted.
4. Supply registration/tag, exact sweep min/max and any additional actor
   snapshots explicitly as fixture inputs. A source-scalar-derived 2D box is
   a further arithmetic/registration admission and must be separately
   validated; the current sweep rejects tied, subnormal or invalid endpoints.
   A single installed staged actor is enough to exercise allocation and player
   selection; a second synthetic actor can exercise pair traversal only if its
   source-independent geometry and identities are unmistakably labeled. Do not
   require an overlapping pair or fabricate a rival to make the report pass.
5. Schedule one frame, re-resolve both generated pair handles in the same pool,
   report broadphase outcomes, suppressed pairs and pending E7F170, with the
   staged seed/progress/triple, resource provenance, projected geometry and
   admissions. No feature selector, feeding, callback, event or prototype state
   should be advanced by this composition.

Tests should cover a pure synthetic adapter with exact triple/geometry transfer,
foreign and stale handles, invalid bounds and unsupported ties, empty versus
nonempty saved override, and non-reexecution of the scheduled frame. An ignored
installed-data test can use the existing exact-hash inspection with a specified
seed and verify CELL/creation provenance and the precise projected words passed
to the frame; source files must be stable under that test's admission. A bad
package hash or inconsistent first-word must reject before registry publication.
A headless CLI test should confirm the one-frame output and explicit pending
labels. None of these tests establishes native player selection, registration,
or runtime behavior.

Actual source gameplay remains blocked on native mount/fixup and E4CCA0 +34
identity, E750F0 creation/current +411C player assignment, post-override actor
and E67A50 cache lifetime, source transform/update inputs, source persistent
registration and endpoint ties/lifecycle, real rival producers, source feature
selection and both ordered contact passes, callback/handle re-resolution and
event scheduling. The controlled source/fixture registry and its player index
cannot close those gaps. See `cell-source-gameplay-integration-seam-v1.md`.


## Implemented controlled path

`--source-installed-frame-fixture` invokes the staged inspection once and
transfers its owned CELL package path, exact key, payload hash and bytes,
selected creation triple/hash, retained feature rows and modeled pre-override
projection into one synthetic registry actor. The command requires an explicit
nonzero seed, progress, tag, caller-supplied 2D bounds and
`--admit-stable-files`; it does not derive the bounds or introduce a rival.
The empty saved triple is the only admitted path. Nonempty overrides, wrong
cache first-word, mismatched CELL digest and invalid bounds are rejected before
registry publication. E829D0 remains pending, as do native actor/cache identity,
persistent registration, contact selectors, callbacks and event dispatch.
The staged actor position `[100,100,0]` and its modeled capability cache are
explicit synthetic projection inputs; the registry handle is synthetic.

The ordinary unit tests check exact owned transfer, override/cache/digest
rejection, bounds and one scheduled frame. The ignored exact-install test was
run with seed 123 and progress 0: it passed and checked the same-read CELL
provenance, selected triple/rows and exact projected words in the registry.
This result relies on explicitly admitted stable local files and is not a
retail actor, provider, player-selection or gameplay observation.


## Optional controlled peer

The installed-frame command also accepts `--peer` followed by three actor
position floats, three projected-center floats, scalar, unsigned tag, and
four 2D bound floats. These are independent caller admissions: the peer center
is not inferred from position or the retained CELL, and the caller supplies
the bounds rather than having the fixture claim source registration. Position
and geometry reject nonfinite/subnormal values, and scalar must be positive.
Whole-frame sweep endpoint checks reject invalid intervals and ties before
registry creation.

The peer clones the *same* CELL bytes/key/digest and selected creation/rows
from the one controlled inspection, but receives a second synthetic handle in
the same registry. Output labels each actor's modeled-pre-override versus
caller-supplied geometry; it does not identify a second installed creature or
retail actor. With overlapping admitted bounds and a passing 3D gate the
scheduled diagnostic reports the pair in dispatch order and stops at pending
E7F170. It performs no source contact selection, callback or event dispatch.

# Cell feature geometry to producer and callback: integration boundary

Static trace of pinned `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
This synthesizes the decoded `E6B151..E6B19A` handoff, a fresh linear decode
of `E6A450..E6A547`, and the existing evidence linked below. It observes no
live actor, source cache, callback or gameplay. The experimental owned fixture's
second feature is supplied synthetic data, not a registered second source actor.

## Geometry and actor inputs

The feature-to-feature route is conditional on both actors having nonzero
+248 and +FC. The two E67A50 results supply ordered 72-byte rows; row class 1
is skipped and the original zero-based loop index is preserved. Once pair
geometry reaches `E6B151`, the caller passes E6A450 fourteen stack words. After
its entry `push ecx`, those words are at `[esp+08]` through `[esp+3C]`:

| Ordinal | Value and identity requirement |
|---|---|
| 1 | Shared selection-output pointer; valid and stable, with current contents |
| 2, 3 | Point and normal pointers, respectively, to current frame locals; their bytes and lifetimes are material |
| 4 | Binary32 depth, loaded/stored through x87 by the prefix |
| 5, 9 | A and B actor pointers; first dword read as each actor's handle, not a resource ID |
| 6, 10 | Resolved A and B definition +B4 words, not the actor +108 handle itself |
| 7, 13 | A and B feature class words from the respective cache rows |
| 8, 14 | A and B feature row indices, retaining skipped positions |
| 11 | B definition +B0 word |
| 12 | E57260 relation computed earlier, with its fallback and actor +358 dependencies |

E7F170 first rejects equal actor ID words, resolves both +108 definition handles
and requires both resolved +B4 words nonzero. It calls the feature selector
with A/B and then B/A, sharing one output; only the final selector other than
0 or 0x16 is forwarded to E7E820. Pool identity, slot lifetime and validity,
reference-fixup/provider identity, current-player +411C, the actual row/cache
association, two source transforms, and any changed fields after a callback
must all be separately admitted. Matching an installed key, a modeled feature
row, or a synthetic checked-pool handle does not supply these identities.

For the small-distance geometry path, the point and normal locals are not
written before this call. Their current six words can be inherited from stack
or prior iteration and must be explicitly admitted at the actual call site.
For the nondegenerate branch they are written by the geometry continuation;
that bounded comparison does not establish their source runtime lifetime.
See [geometry continuation](cell-feature-contact-geometry-continuation-v1.md).

## First testable producer boundary

`E6A450` tests feature A then feature B for code 9 and exits on either. If B
+ B4 is 7, it submits one pending E59B40 candidate with selector 19 when the
current A ID equals +411C, else selector 0, then returns. This precedes the A
+ B4 == 7 exit. If B + B4 is 6, it submits selector 18 for current-player A,
else 1, and continues. A bounded fixture can therefore join separately
admitted geometry, two pool-scoped *fixture* actors, row classes and original
indices, explicit definition words and current-player fixture ID into the
existing `cell_feeding_producer_prefix_v1` model. For B=7 it may expose only a
pending E59B40 call; for B=6 it must suspend after the pending call and cannot
claim continuation. Class-9 and A=7 cases test ordered early exits. Separate
synthetic tests should cover reversed actors and point/normal/index preservation,
foreign/stale handles, wrong cache key, and unavailable small-branch locals.
An admitted current selection and current-player value can be passed to the
separately tested E59B40 model only as a further explicit fixture step; it does
not prove native candidate generation. Existing prefix tests are synthetic;
its original-byte host is not prepared for execution or independently run.
See [prefix model](cell-feeding-producer-prefix-model-v1.md) and
[comparison plan](cell-feeding-prefix-native-comparison-plan-v1.md).

After the B=6 submission, source retains actor and geometry pointers in
nonvolatile registers, then rereads actor +90/+94/+98, IDs and current
`+5190 -> +74` and calls E673A0. E673A0 can mutate a world container before
E59CD0 and later decisions. A correct continuation requires ordered suspension,
fresh values at the read sites, original pointer/lifetime and alias admission,
FP state and callback effects. A precomputed frozen record is insufficient.
See [E673A0 side effects](cell-feeding-e673a0-producer-side-effect-v1.md) and
[ordered producer](cell-feeding-candidate-producer-v1.md).

## Event and callback boundary

E7E820 re-resolves the selected actor IDs; selected cases 9/10 can construct a
selector-1 record with actor ID, raw code and B's +108 definition handle, but
the relevant gates and successful allocation are required. E7E160 invokes a
record callback only when +08 is zero, +04 nonzero, and argument 3's low byte
zero; it sets +08 after the callback and subsequently reads the *current*
selector and record fields. The observed E7DBB0 callback resolves both +28 and
+2C actor IDs before E7D8B0, which can alter player identity and progress.
Selector-1 dispatch later resolves actor and definition handles again. A single
pre-callback snapshot, arbitration win, or callback return does not prove an
event or feeding; pool reuse and intervening mutations require revalidation.
Full conditions and evidence IDs are in
[feeding event dispatch](cell-feeding-event-dispatch-v1.md).

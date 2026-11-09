# Controlled E7F170 pair coordinator

The opt-in owned source-player inspection now runs a separate scripted
coordinator on two checked fixture actors. It exposes ordered A then B
resolution admissions, checks their distinct reference words and synthetic
resolved-object identities, and waits until both have resolved before applying
the two +B4 zero gates. Resolver and holder cleanup calls are not executed.

On the surviving branch it initializes the first six shared-output words to
zero and keeps the six caller-supplied point/normal words opaque. The caller
supplies the effects of two selector passes in A/B then B/A order. Both passes
retain the same checked actor and definition identities; the source callbacks
and changing object contents are not modeled. After the second supplied effect,
selector zero or 0x16 leads to pending cleanup; another selector produces a
pending E7E820 input, with cleanup still outstanding. No event is allocated.
The finite E7E820 float is supplied up front with an explicit admission that
the source caller's float slot remains unchanged through both scripted passes;
the fixture does not model x87 exception status or alias mutation of that slot.
Errors on stale, foreign, equal or mismatched fixture identities and phase/order
violations are safety policies, not native behavior.

The inspection's definitions, selected output and no-change reverse pass are
explicit synthetic scripts. They are not outputs from its geometry or the
source selector and cannot establish a live contact, actual provider, callback,
event or gameplay behavior. In particular this coordinator does not connect
its scripted result to E7E820, feeding, audio, or the prototype's actor state.
The original trace and missing bindings are in
[cell actor pair event binding gate](cell-actor-pair-event-binding-gate-v1.md).
The bounded original-instruction comparison below tests coordinator control flow
under an explicitly scripted host; it does not validate those missing bindings.


## Controlled original-instruction comparison

On 2026-11-08 the reviewed host compared the contiguous original
`E7F170..E7F28B` fragment (284 bytes; SHA-256
`f69a63d2cfddb13fc1aa2454e1b6edd6cadc2c12bbacebfead9ce278969642ed`)
with the owned coordinator oracle. The installed PE was SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
All seven cases matched their 43 observed words. The [receipt](../target/cell-pair-coordinator-result-v1/receipt.json)
SHA-256 is `f3da77870597477262ce971bfd00cffdb0c624b0ec8e1238847fc1ed1dd5fe8b`;
raw model and native stdout are preserved alongside it, SHA-256 respectively
`7b6a01b99c46caec294c0863ebe2f84ddbee64a3ad260fc7fdb6bd4bf54a0817`
and `c014b1dbec87dadc217f63bf2ea0cc9b9aa4046507819f26818c29bdc8f1aa87`.
Both processes exited zero and produced no stderr.

The fixed tuples below are `(equal actor ID, A +B4, B +B4, first selector,
second selector, float bits)`. Each actor reference, resolved definition,
and output buffer is synthetic; the float is negative zero in every row.

```text
(1, 2, 2, 9, 10, 80000000)  equal ID
(0, 0, 2, 9, 10, 80000000)  A +B4 zero
(0, 2, 0, 9, 10, 80000000)  B +B4 zero
(0, 2, 2, 9,  0, 80000000)  selector zero suppresses event
(0, 2, 2, 9, 22, 80000000)  selector 0x16 suppresses event
(0, 2, 2, 9, 10, 80000000)  event stub reached
(0, 2, 2, 0, 19, 80000000)  event stub reached
```

The comparison includes resolver/selector order and counts, cleanup and event
counts, the 12-word shared output at each selector and event, and event float.
The six opaque tail words are host-seeded `A5A5A5A5`; snapshots not reached
retain `CCCCCCCC`. The linked PE check confirmed all non-relocated bytes and
13 original call/jump opcodes, with the 13 rel32 operands resolving to the
reviewed stubs. All pre/post pins matched. Each row recorded x87 control
`027F`, status `0000`, and MXCSR `1F80`; the runner checks x87 control and
TOP and MXCSR control while retaining status/sticky bits in the receipt.
The host and runner hashes are respectively
`b7d07db8199697511bf77b2ebb3ae5ad4aeeb9619d0891eb547b81b8fdb6f5af` and
`bd5ca737a55d2c48204858e5eea2e31f8426f233c76f604ee479bf2c6ef90031`.
The oracle executable was
`43e6bfe4885c217ccf24eda8952e4189fa004bc51b59e70650df40b67891a62e`;
its pinned coordinator and pool-fixture source hashes are listed in the receipt.

This comparison executes the original coordinator fragment, but its external
calls are replaced. Holder-init validates the expected synthetic holder and
zeros it; resolver validates the supplied reference/holder and returns a
synthetic definition; selector validates the actor/definition order, records
the shared 12 words, and writes only the scripted selector; event records
the shared words and float; cleanup validates the holder and counts calls.
These stubs do not execute native provider resolution, selector effects,
event allocation or dispatch, holder cleanup, or gameplay. They assume the
scripted identity, non-aliasing, lifetime, and stable float inputs; the seven
rows cannot establish behavior for arbitrary selector writes, uninitialized
source stack contents, live actor/cache identities, callback mutation, or
concurrent state.

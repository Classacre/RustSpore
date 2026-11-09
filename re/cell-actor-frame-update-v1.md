# Actor frame registration update boundary

Source `E6DB40..E6DBC9` receives an actor in EAX, checks actor +364 against
FFFFFFFF and skips if equal. Otherwise it invokes E6D7F0 with the actor in ESI,
a center output in EDI and a scalar output on the stack. It forms max X/Y by
SSE `ADDSS` and min X/Y by `SUBSS`, then passes the global Cell +4104 manager,
the actor +364 slot word, min and max pointers to BBBD10 (which jumps to
BBB880). The slot word is not validated in this caller apart from the sentinel.
The source registration path E74F5B..E75036 creates that slot conditionally;
actor cleanup E7812A..E7814B removes it when non-sentinel. The distinct
E6E085..E6E0FE update follows E86960 and needs separate timing/freshness proof.

The actual missing join is stable source actor/pool identity and ownership of
its +364 slot in the same Cell +4104 manager across calls, plus freshness and
lifetime of E6D7F0's actor fields and optional cache result. Installed package
bytes and the controlled registry do not establish any of those live facts.
The actor may also be removed or the slot reused; no cross-frame ownership is
implied merely by a matching numeric word.

A build-only host in `target/cell-actor-frame-update-prep-v2` isolates the whole
E6DB40 caller with synthetic guarded actor/Cell and explicit geometry and
update-recording stubs. It tests sentinel skip, three supplied slot words,
finite exact center/scalar inputs and output bounds. It patches only two calls
and the global reference, and statically checks all other source bytes and
instruction boundaries. E6D7F0 data, pointer freshness, manager state and
BBBD10 effects are admitted stubs; E6E085, live actor identity and gameplay are
outside the comparison. The v1 build-only attempt failed assembly because an
operand lacked explicit width and is preserved; neither host was executed as
of preparation. A hidden non-switched private desktop, stack/FP and guard
checks and failure streams are part of the review gate.

The independently reviewed v2 host was subsequently executed: all four
controlled rows matched the byte model; native exit was zero, controls passed
and the private desktop closed after reaping. Receipt
`target/cell-actor-frame-update-result-v1/receipt.json` SHA-256
`71fef1ede8d4c0a68c9aa84a82c0f6a8a7240202641f8879b54fe6b5395b0fd5`.
GeometryStub supplies fixture center/scalar and records its call; UpdateStub
checks the synthetic manager and captures slot and bounds without executing
BBBD10. This evidence validates neither E6D7F0 nor BBBD10, actor identity,
cache freshness, slot ownership, the E6E085 timing or live gameplay.

The separate checked Rust caller API was compared directly with the retained
native output, without rerunning native code; all four fixed rows matched.
Receipt `target/cell-actor-frame-update-rust-result-v1/receipt.json` SHA-256
`eda17e513c5080a02c01245deb8056595e9af86d78ca052ae21c92f315a19c12`;
runner SHA-256
`ead2ea43c31c13026ea320cd88126786d97fd07ceec7c1eb67b36201a9619c49`.
It treats FFFFFFFF as an early skip; other slot and manager words are opaque
fixture tokens with stable ownership admitted. It reuses the finite source-
ordered SSE bounds calculation under MXCSR 1F80 and stops at a pending update.
No actor lookup, actual E6D7F0, native manager lifetime or BBBD10 execution is
established by this Rust comparison.

The installed-frame diagnostic has a separate opt-in
`--multiframe-update --admit-actor-manager-stability` path requiring `--peer`.
It resolves both distinct synthetic handles in the same owned fixture registry,
uses their admitted geometry snapshots through the checked E6DB40 caller model,
and sends the resulting bounds through sorted-old and pending-new update models
and the multiframe merge. The peer keeps the same installed definition
provenance and has caller-supplied geometry; it is not a second observed source
creature. The fixed manager token and actor/manager stability are explicit
fixture admissions. Existing one-frame behavior remains the default. Focused
synthetic tests and one explicitly run installed-data test passed. The full
composition has not been checked against a native live actor or lifecycle.

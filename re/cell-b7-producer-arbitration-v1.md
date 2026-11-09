# Early B7 producer and arbitration comparison preparation

A first E6ADE0 candidate can proceed into E6A450 only with explicit stable
actor/definition words and geometry pointers. The useful early B7 branch reads
actor A's first word and the current-player word; it submits selector 19 if
equal, otherwise 0, through E59B40 and returns. Feature code 9 exits first;
B7 precedes A7. B6 submits then continues toward E673A0 and is excluded. Even a
winning early submission is not the final E7F170 selector: later candidate
iterations and the reversed pass may change it.

The build-only host under `target/cell-b7-producer-arbitration-prep-v1`
contains complete E6A450 and E59B40 source bodies. The two early B7 calls
route through a counting thunk to copied source E59B40; the other 40 direct
calls route to a nonreturning trap. The two relevant global operands point to
a synthetic Cell root. Fixed cases cover player/nonplayer writes, depth-based
no-write, feature-9 exit, and B7/A7 precedence. Full output, count, stack,
nonvolatile registers and floating controls are compared to a Python model.
The source has other absolute references that could be read before a trap on
unadmitted branches: fixed B7/feature-9 preflight and independent control-flow
review are essential. Synthetic disjoint point/normal, actor words, current
player and lifetime are admissions. The host was not executed as of
preparation. It does not call E6ADE0, E6A2B0, callbacks or source actors, and
cannot establish feeding or final selector behavior.

The independently reviewed host subsequently ran: all five controlled rows
matched; receipt `target/cell-b7-producer-arbitration-result-v1/receipt.json`
SHA-256 `526e8aaa51d01d49d12bfa977c3ea8e817adf63793f66d3ddf3f250d4bf1f1bf`.
A separate Rust adapter matched the retained native rows without rerunning the
host; receipt `target/cell-b7-selection-rust-result-v1/receipt.json` SHA-256
`6afe4c885ce5cbaf94fd4330d21c403a75764965c87de8ece0147420cab6f5f3`, runner
SHA-256 `313b5dea0dbf52ba04fe95dc911c32cfa2d85f292a2767374bc04b82c99b9691`.
The reusable B7 step accepts the bounded first-candidate result and an
explicit stable synthetic token-to-actor-word/current-player binding, applies
only the early producer and arbitration, and labels the result intermediate.
It refuses opaque small-branch geometry before any submission and does not
execute the B6 continuation, E6A2B0, the reverse actor pass, callbacks or
actual actor/cache resolution. A separate model test composes first candidate
with this step; the two native comparisons remain separately bounded fixtures.

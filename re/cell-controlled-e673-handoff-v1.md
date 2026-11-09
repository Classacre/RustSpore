# Controlled post-arbitration E673A0 handoff

The opt-in owned fixture can now resume its B-definition-6 path after its
modeled E59B40 write and return a pending E673A0 call. The one-use continuation
contains the two retained pool-scoped fixture handles and a unique synthetic
point identity assigned when the fixture produces the prefix. Resumption requires those identities to match fresh supplied
snapshots and both actors to remain valid in the same pool; stale, foreign or
mismatched identities are rejected. The point identity is an owned fixture
label, not a native address or proof of retail lifetime. A fresh point's three
words, A/B +90/+94/+98 words, current +5190/+74 word and x87 control are
supplied explicitly after arbitration. No fixture transform or earlier point
is implicitly reused by the resume API.

The distance model follows `E6A548..E6A590`: A-B for x/y/z, then `(z*z +
y*y) + x*x` under an explicit 027F x87 context, FSQRT and binary32 store. Its
checked operations reject nonfinite inputs/results and restore the host x87
environment; these are fixture policies, not claims about source exceptions or
sticky flags. The model assumes stable, disjoint values and an admitted empty
source x87 stack, and does not reproduce interleaved source reads or external
mutations. A later six-case original-instruction comparison is described below.
The earlier SSE feature distance and geometry are distinct quantities.

Inspection supplies a separate synthetic B=6 route, uses the two controlled
fixture transform positions as explicit fresh +90 values, a synthetic fresh
point `(101,100,0)`, current word zero and control 027F. If the preceding
feature geometry does not reach that route, no call is manufactured. The result
is a pending E673A0 record only: it executes no bounds checks, provider lookup,
container mutation, E59CD0, event, callback or gameplay. The source boundary
and remaining lifetime obligations are in
[cell E673A0 continuation](cell-e673a0-continuation-boundary-v1.md).

## Reviewed fragment comparison

The reviewed host ran `E6A548..E6A592` with one relocated pointer to synthetic
root/state data. All six distance words matched the pinned Rust model; source
actor ID/current-word reads and retained fixture point identity checks also
matched. No E673A0 call was executed. The input and output words were:

| Case | A position; B position | Distance bits |
|---|---|---|
| 0 | (3,4,0); (0,0,0) | `40A00000` |
| 1 | (-0,0,0); (0,0,0) | `00000000` |
| 2 | (`00000001`,0,0); (0,0,0) | `00000001` |
| 3 | (1,1,1); (0,0,0) | `3FDDB3D7` |
| 4 | (16777216,1,1); (0,0,0) | `4B800000` |
| 5 | (-3,2,7); (1,-1,7) | `40A00000` |

Each captured A/B ID was `00010000/00020001`, current word `12345678`, and
the retained point identity check returned 1. The six captured x87
control/status words were `0000027F 0000027F 0002027F 0020027F 0020027F
0000027F`; x87 control and TOP checks passed. MXCSR was `1F80` throughout.
Sticky status was recorded but not compared.

The compared model source SHA-256 is
`64e3bde3fce207c317b1e88d50172ebb4a17c0430f6c8cb068e2f048bfea1fce`.
Receipt SHA-256
`755866a3a64caea60ee89aef3cb099bb362981246c6d9a1fc1cf4424dd1f3f5b`;
native stdout SHA-256
`caa5df4ca85338519196a90fb5c4d12edc3dc7f1a87fa691501c253702558b4f`;
model stdout SHA-256
`f660470c105837edd5ff963546352b1247939e2d28dc5fd4073d79d3758dc172`.
The retained run is under `target/cell-e673-call-run-v1/`; host and corpus
pins are in [the preparation note](cell-e673-call-comparison-preparation-v1.md).
This covers six controlled call-input cases and the named checks, not real
pointer identity, live actor/provider state, E673A0 effects, exception-status
parity, callbacks, gameplay or atomic file stability.

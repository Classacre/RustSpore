# Class-5 segment descriptors and sampler boundary

Static instruction trace of pinned `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
On the `413D70` flagged-row path and with an initially empty builder, each
leading class-5 row produces a node. Starting with the second, the builder
also appends a 0x10 segment through `4FD360`: prior node ordinal, current node
ordinal, binary32 `3E99999A` (0.3), and the previous row's **sign-extended**
+02 word OR `80000000`. `414041..41407D` shows the argument construction and
updates the previous node and row word. The append requires successful vector
allocation and does not identify either ordinal as a palette entry.

`creature_skin_class5_segments_v1` implements that bounded descriptor prefix
from explicitly admitted frozen rows. It returns no interior lanes. Two focused
unit tests cover the all-row flag gate, termination at the first non-class-5
row, descriptor order, a negative signed +02 word, and short prefixes. These
are source-derived tests, not native execution. A separate source inspector
checks the payload SHA-256 and exact parse and requires
`--assume-frozen-post-mutation`.

At `4FDA40`, the complete builder's node vector is processed before its
segment vector. For each segment, `4FDB20..4FDBD0` passes the query receiver,
segment +08 spacing, two referenced 0x18 node positions and their +0C/+10
scalars, and segment +0C key to `4FD3C0`. The latter computes the separation
vector and its length, normalizes it using `453970`, and consumes the query
receiver's +0C value. The instruction span `4FD476..4FD55F` performs divisions
of that receiver word by each node's +0C, invokes MSVCR90 `_CIpow` with the
pinned 0.25 word, stores the result as binary32, invokes `_CIsqrt`, and
combines the result with the corresponding node +10 and the binary32
`358637BD` word. Subsequent branches compare the resulting slope magnitude
against the pinned binary64 `1e-6`; the general branch also invokes MSVCR90
`_CIlog` and `_CIpow`, while both branches can append interior lanes through
`4F9100`. These steps involve x87 stack inputs, binary32 spill points, and
SSE arithmetic; replacing the imported library operations with a convenient
host power/log implementation is not yet validated. Empty/degenerate and
nonfinite conditions also need separate admission.

Class-3 construction is a different upstream branch. `413D70` scans the
contiguous class-3 rows after the prefix and only calls `412890` for rows with
+08 mask 1. Parent +04 and earlier-row state determine inputs. In `412890`,
the object's virtual +58 provider is called with row +88/+84; a null result
skips its node/segment construction. A nonnull result leads to property lookups
and flag-dependent fallback, table, or interpolation paths. Thus class-3
rows alone do not establish its geometry, property responses or builder output.
The 0473 snapshot has no contiguous class-3 rows after its six class-5 rows.
Neither its five class-5 descriptors nor their node endpoints establish
sampled lanes, full bounds or the original grid.

The next exact comparison should bind a frozen builder snapshot containing
node records, descriptors and query +0C, load the pinned MSVCR90 helpers in an
isolated host, and compare sampled append count, order, full lane words and
bounds against a checked implementation for both slope branches and boundary
cases. The CRT binary identity, calling convention, floating-point controls,
input domains and append capacity must be explicit. This is distinct from a
real-row-to-live-provider or complete-creature claim.

Reproduce by mapping and decoding `413F46..41407D`, `4FD360..4FD3B5`,
`4FDA40..4FDBDF`, `4FD3C0..4FDA35`, and `412890..413C4C`; the retained
`target/ghidra-creature-entry-v1/skin-lane-upstream-v1.log` and
`skin-query-keys-source-v1.log` pseudocode provide supporting context. Original
instructions control the specific claims above.


The retained 0473 payload inspection produced five descriptors, linking nodes
0 through 5 consecutively with keys `80000001` through `80000005`; its report
`target/creature-skin-class5-native-20261108/segment-source-inspection.json`
has SHA-256 `ace761417fbadec149af759a6f235de16f89e96198f767d210b0b838fa4fe3f2`.
This is model output on admitted retained bytes, not an observed native builder.
The focused command `tools\build.bat test -p spore-core
creature_skin_class5_segments_v1 --lib` passed two tests.

## Restricted zero-threshold sampler

`creature_skin_segment_zero_threshold_v1` models only a positive, finite,
nondegenerate segment with equal positive endpoint radii, both endpoint
coefficients 1, caller-admitted query +0C exactly +0, MXCSR 1F80 and x87
control 027F. Those inputs force the low-slope path, provided the source CRT
returns +0 for `pow(+0,0.25)` and exactly 1 for `sqrt(1)`; the model uses
hardware FSQRT for positive finite separation under the same control. Those CRT
identities and the matching behavior of FSQRT remain explicit admissions, not
a completed native comparison. The caller supplies the source-consistent
spacing, endpoints and key. It returns only sampled interior lanes, never
endpoint lanes, complete bounds or a grid, and has finite/work checks absent
from the source. This restricted threshold is synthetic and has not been
established for the retained 0473 creature.

The installed `SporeBin/msvcr90.dll` SHA-256
`ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185`
exports `_CIpow`, `_CIsqrt`, `_CIlog` and `ceil`, matching the pinned PE's IAT.
Static decoding of its `_CIsqrt` positive ordinary path shows FSQRT; the
routine also examines and may adjust the x87 control word. A safe next native
host needs pinned PE/CRT and fixture bytes, system-selected executable storage
with source-relative placements and independently audited code spans, an
explicit checked IAT binding to that pinned CRT, a preallocated bounded query
receiver, controlled x87/MXCSR, output and stack canaries, hidden execution,
timeout and preserved failure receipts. Its proposed source and exact command
must be reviewed before execution. It should compare interior count, order,
all lane words and bounds across zero/nonzero slopes and boundary cases; it
must not assert native runtime identity or real-source threshold from those
synthetic inputs. No sampler host has been prepared or executed here.

A reviewed isolated diagnostic host subsequently ran the original sampler and
supporting spans for four fixed **synthetic** zero-threshold equal-radius
cases. An independent retained-output comparison matched sample counts
`1, 3, 0, 4` and every emitted center, cutoff, scale, coefficient and key word
against the pinned Rust comparator. The receipt is
`target/creature-skin-segment-independent-compare-20261108/result.json`,
SHA-256 `b2892c83036160d6498dfed3477006252ad07cc127e577fc22a70cbede95a541`.
This is a four-case comparison, not a proof across the model domain; bounds and
floating-point sticky flags were not compared.

The initial host exited 7 before output. A separately pinned diagnostic found
that the absolute-path CRT load failed, and a further diagnostic captured
Windows error 1114. A separately reviewed diagnostic then created and activated
an activation context from the pinned game's embedded manifest and completed;
the game executable was read for its manifest and was not executed. The
successful diagnostic receipt has SHA-256
`2c705845b5fd9d926ae0a3b2fd8868d4db1c32094b4049ddb53484986114cf6a`.
The original failure and diagnostic receipts/streams remain under the adjacent
`target/creature-skin-segment-*-20261108` directories. CRT path bytes and
manifest were pinned before and after execution, but the loaded module's
identity was not independently verified. Output canaries were present but no
guard pages; neither native live runtime, original creature threshold, complete
grid nor source row lifecycle follows from this result.

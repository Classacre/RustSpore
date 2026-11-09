# Controlled first-feature projection

A static model of the first-side `E6AE6C..E6AF2F` feature transform now runs
inside the opt-in owned player-pool inspection on each retained non-class-1 row.
The executable decoded for this model has SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The separately controlled six-case original-instruction comparison below
matched; this is not a contact test or a live actor observation.

For row center x/y/z, actor matrix m0..m8 at +5C..+7C, and bit 02 of +48 set,
the instructions produce separately rounded scalar SSE results in this grouping:

```text
x' = (m3*y + m6*z) + m0*x
y' = (m4*y + m7*z) + m1*x
z' = (m5*y + m8*z) + m2*x
```

Without bit 02 the center is unchanged at this point. Each coordinate is then
multiplied by actor +58 and added to +4C/+50/+54; row radius is multiplied by
+58. Unlike the existing aggregate E6D7F0 projection, the feature Y grouping
starts with `m4*y + m7*z`. The implementation uses separately rounded SSE
operations under scoped MXCSR 1F80 and rejects nonfinite consumed inputs and
results. It does not reproduce exception flags or native partial writes; its
ordering of independent arithmetic is not an x86 instruction trace. The model
admits stable disjoint snapshots and returns an error on unsupported values.

The opt-in inspection explicitly supplies position `(100,100,0)`, its modeled
scale and matrix, and flag 02. The borrowed rows and key association are the
controlled capability and post-override admission, not an observed E67A50
cache. The printed per-row centers/radii are thus candidate fixture values.
Actual source contact also needs both actors' post-override triples, +248 gate,
current transforms, cache and pool lifetimes, the second-side feature loop,
pair geometry including degenerate branches, source FP environment, ordered
E6A450/E59B40 calls and intervening effects. No overlap, contact acceptance,
source event, or gameplay result follows from this projection.

## Original-instruction comparison

The reviewed x86 host ran the pinned 200-byte `E6AE6C..E6AF33` fragment once
per supplied case and all six resulting x/y/z/radius words matched the Rust
model. Both programs exited zero. Inputs were disjoint synthetic actor/row
buffers; the host reset MXCSR to 1F80 per case and checked memory guards.
Recorded MXCSR values were `1F80, 1F80, 1F80, 1F80, 1FB2, 1F80`;
control bits passed after masking sticky flags, which were not compared.

| Case | Controlled input | Result x, y, z, radius (hex binary32) |
|---|---|---|
| 0 | flag clear; center (3,4,5), radius 2, position (10,20,30), scale 2 | `41800000 41E00000 42200000 40800000` |
| 1 | flag 02; same values and identity matrix | `41800000 41E00000 42200000 40800000` |
| 2 | flag 02; center (1,2,3), radius 2, zero position, scale 1, physical matrix words (1,2,3,4,5,6,7,8,9) | `41F00000 42100000 42280000 40000000` |
| 3 | flag 02; center (3,1e20,-1e20), radius 1, zero position, scale 1; m1=m4=m7=1, other words zero | `00000000 40400000 00000000 3F800000` |
| 4 | flag clear; center bits `00000001 80000001 80000000`, radius 1, zero position, scale 0.5 | `00000000 00000000 00000000 3F000000` |
| 5 | flag clear; center, radius and position all negative zero, scale 1 | `80000000 80000000 80000000 80000000` |

Retained `target/cell-feature-fragment-run-v1/receipt.json` SHA-256
`50f91155eca2a1bb9634b36b8563ad0059d098bebd7747f3628014713807b664`;
native stdout SHA-256
`9fd8b182dbd46ea71dc948db7a3c86763b9a08d32e483bc29c1912ac8e2c8ba9`;
model stdout SHA-256
`97c4cc784ac721045e068621bd2b88d098be249b1bc2dccea1415247e55412b5`.
The fragment, host, cases, source and runner pins and preparation limits are in
[the preparation note](cell-feature-fragment-comparison-preparation-v1.md).
This checks only these six finite inputs and four output words. It does not
establish native exception status parity, complete instruction-path behavior,
stable-file admission against transient replacement, or actual runtime state.

The first- and second-side projection arithmetic has since been extracted to
`spore-core::cell_feature_projection_v1`; the existing controlled Bevy feature
fixture delegates to it with its same supplied row and transform. A separate
comparison checked the reusable core adapter against the two retained native
streams: six first-side and six second-side projection tuples matched, without
rerunning source instructions. The second stream comparison includes only its
four center/radius words, not the separate pair-prefix calculation. Receipt
`target/cell-feature-projection-core-result-v1/receipt.json` SHA-256
`31e52fce03f737e278a0d15e6da7622c950292f830d7aa5773fb6fafc4dfd746`;
runner SHA-256
`d79ce12a321e528f765e201ad1cc0787562607d501794367281e37bd71889210`.
The original receipts remain bound to their earlier separately pinned Bevy
source revisions. The core model preserves both original operand-order
variants, finite admissions and scoped MXCSR 1F80; neither comparison proves
cache identity, native actor ownership, feature traversal, contact or gameplay.

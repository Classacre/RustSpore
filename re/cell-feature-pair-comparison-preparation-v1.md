# Second-feature pair fragment comparison preparation

The preparation step built, but did not execute, a synthetic x86 host containing source bytes
`E6AF72..E6B08B` from the pinned executable. The original 282-byte fragment
SHA-256 is `e1c99e797b887d15fe3b2cc6c6d9f93684c85554a062d31bb83318d84a250cde`.
The subsequent reviewed run is described in [the pair-prefix note](cell-controlled-feature-pair-prefix-v1.md).
The only changed fragment bytes are the four-byte absolute threshold address
at offset 0x101; it points to the host's guarded `37800000` value. The internal
conditional branch retains its original relative displacement. Static PE
inspection found one matching prefix, the unchanged suffix and an appended RET;
the linked threshold address `00403008` resolves to the expected bits.

Six finite cases cover zero distance, immediately below, exactly at and
immediately above the squared-distance threshold, asymmetric rotated geometry,
and subnormal behavior. The wrapper supplies first center/radius as synthetic
inputs, second actor and row bytes, resets MXCSR 1F80, and records second
center/radius, differences, squared distance, radius sum, EFLAGS and MXCSR.
The runner compares the nine numerical words and derives the small-distance
branch from captured CF/ZF; it masks sticky flags when checking MXCSR controls.
It preserves stdout/stderr and writes an exclusive success or failure receipt.

Review the generated x86 wrapper and linked host, scratch offsets (the CALL
changes ESP by four), bounds and guards, patched operand, corpus, branch flag
interpretation, imports and runner before any execution. Pins and build records
are in `target/cell-feature-pair-prep-v2/manifest.json`. Host SHA-256 is
`e7fdda118e28103ed1950df69d5a50a738e8993af8a4abcc551ce0f4c2da98c5`;
runner SHA-256 is
`0a1cb7131850a02de4f5644ea1c48ccfb2f7465d83ad4d71f9c8a8d965750846`.
The earlier v1 preparation remains preserved; v2 adds threshold guards.

After review the proposed invocation is:

```powershell
python tools/cell_feature_pair_run_v1.py 0a1cb7131850a02de4f5644ea1c48ccfb2f7465d83ad4d71f9c8a8d965750846
```

A pass would be limited to this instruction slice, six inputs, numerical words
and branch condition under the declared controls. It would not establish live
actor identity, source cache, contact point, event, full FP exception parity or
gameplay. Hash checks cannot exclude transient replacement.

# E673 record-phase original-instruction comparison preparation

The preparation step built, but did not execute, a synthetic x86 host with four unmodified, independently
called source slices from the pinned executable. This is not a contiguous
E673A0 invocation and executes no provider, E65810, container allocator or
callback. The slices and SHA-256 values are:

The subsequent reviewed run is recorded in [the record-phase note](cell-controlled-e673-record-phases-v1.md).

| Slice | Source range (inclusive) | SHA-256 |
|---|---|---|
| Signed lower bound, including RET | `E616C0..E6170C` | `233dbf636987d15096f54508905e9569f4bd48990ef6063a40dfebd9c673e1f9` |
| Existing +10 stamp | `E674DD..E674DF` | `eb5237e9a61ef8c9bb4bdfefa8f1396104a56dd75c52cb2fb154888bcb01084a` |
| Existing point stores | `E674FE..E6750C` | `325a2d2cc8273841f710514f209a19baa41f981f2ee87622d94771aeb34d2684` |
| New-entry stores | `E67524..E6756D` | `fe80d46b69fdf42b835139dfbc5ccfd3120aa9de05cd2944f83ce75ba823199b` |

Six signed-key lookups use an explicit sorted three-entry vector and separate
hit/miss checks. The existing-entry case admits the same stable owned entry at
two independently modeled lookup boundaries and supplies fresh point words
between source store fragments. Four new-slot cases include thresholds below,
equal and above distance plus signed-zero equality; full IDs include a pair
with colliding low 16-bit keys. Entry memory starts with sentinel bytes so the
comparison includes preserved padding and the source's byte-only impact write.
The host captures complete 0x2C-byte records and per-new-case MXCSR, uses
canaries, and tests control bits with sticky flags masked. It does not establish
native allocation success, comparator-state behavior beyond the copied lookup,
mutation between calls, pointer lifetime or actual actor identity.

Review before execution: `tools/cell_e673_record_prepare_v1.py`,
`tools/cell_e673_record_run_v1.py`,
`crates/spore-bevy/examples/cell_e673_record_model_v1.rs`, and the generated
`target/cell-e673-record-prep-v2/` assembly, PE, case bytes, fragments and
manifest. Static PE inspection matched the copied fragments at 00401662,
004016AF, 004016B3 and 004016C3; the first has its source RET and the others
have explicit wrapper RETs. Review lookup ABI/end pointer, source entry+4
versus entry offsets, new-fragment CALL-adjusted stack offsets, internal branch
targets, canaries, byte-only flag and adapter comparison independently.

Host SHA-256:
`d3b137c28d243d1edae8b5fde1d876ac5ec502ba7b96f3dfc402931db69a189f`.
Runner SHA-256:
`599365f8602d6d3293176f6fd5e08113c610e78e3926d464d830321c210c66be`.
The v1 build remains retained; v2 adds input guards. After review, the proposed
invocation is:

```powershell
python tools/cell_e673_record_run_v1.py 599365f8602d6d3293176f6fd5e08113c610e78e3926d464d830321c210c66be
```

The runner makes an exclusive run directory, records raw stdout/stderr and a
success or failure receipt, checks pins before/after, original/linked bytes,
imports, output extents and MXCSR controls. Hash checks cannot exclude a
transient file replacement. A match would apply only to the enumerated isolated
synthetic slices and fields, not bounds/provider gates, a complete source call,
container mutations, callbacks, live contact or gameplay.

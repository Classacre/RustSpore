# Checked row-to-edge remap construction

Static trace of installed `SporeApp.exe`, SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. This extends [the conditional intermediate identity](creature-skin-intermediate-row-identity-static-v1.md) and [row-edge construction](creature-skin-row-edge-construction-static-v1.md). It does not establish selected package, live execution or palette identity.

At `4CC11C..4CC177`, `4CC060` derives edge count from its receiver +54 vector and a temporary endpoint-to-row vector length from the last row's **signed** +02 plus one, and initializes that vector with `FFFFFFFF`. At `4CC17C..4CC21B`, each 0x8C row is visited in order. Signed +02 exactly `-1` skips the row entirely. Otherwise, if +08 mask 2 is set, native writes the row ordinal at index `link-1`, then writes it at index `link`; later rows overwrite earlier entries. No source bounds checks protect those stores.

At `4CC251..4CC2CD`, the output remap contains one entry per row ordinal: start from that ordinal and repeatedly follow signed +04 parent while the current row's +0B byte equals 1, then store the first non-class-1 ordinal. `4CC2CF..4CC36B` handles edge ordinals starting at the row count: use that edge's second word as an index into the endpoint-to-row vector, dereference the resulting row, start at its signed +04 parent, and perform the same class-1 walk before storing the result. The output vector is sized from the edge count; malformed lengths, indices, `FFFFFFFF` sentinel, parent chains and cycles are not guarded by this native slice. The vector then reaches `52B700` as the optional argument-5 map on this caller route. This is a row-derived map, not proof of selected CreatureData provenance or finalized palette identity.

`creature_generated_skin_remap_v1` models these two ordered snapshot passes and composes them with the existing checked row-to-edge and edge-to-membership slices. It requires explicit stable post-mutation rows and successful prior preparation/vector operations. It bounds row, endpoint and edge counts, accepts only signed `-1` as the skipped link, retains unconsumed `FFFFFFFF` slots, rejects invalid/unmapped extra-edge accesses and invalid parent chains, and terminates cyclic walks with an error. Its checked transactional behavior is stricter than the native partial writes/unchecked loops. It does not implement any row mutation, +40 coordinates, native allocator, actual query, source cache/provider, live actor, palette or mesh.

The source inspection example accepts `--include-remap` after its explicit
`--assume-frozen-post-mutation` flag. On the retained 10-row payload SHA-256
`dbbb55830f7f75eb5c45afd470a8bd0716cb08134e26f62a82674e28cd697137`, it
produced an 11-entry endpoint map and a 10-entry edge remap `[0,1,2,3,4,5,6,7,8,9]`.
The translation-edited payload SHA-256
`84b8f9c0448dc2828a0803f73264171f5a8d06d0684b90243de9b3e0ffc1eb99`
produced the same maps. This is a property of these supplied bytes under the
admitted conditions, not proof that the game selects them or that those ordinal
values are palette identities. The reports under `target/source-cell/` are
`skin-remap-source-inspection-original-20261105-v1.json` (SHA-256
`d7188021341ebba5e4d8e465742e467aea5545f189a7a2dbb7f0f2d322f59e7c`)
and `skin-remap-source-inspection-translation-20261105-v1.json` (SHA-256
`5d6e4ded87644e162f48f792bcc5a44cff8c5fb22865809850883975b9957a45`).

The same admitted inspection accepted all 1,365 retained pinned source files:
46,418 rows and 50,221 edge remap entries. Of those entries, 11,642 differed
numerically from their edge ordinal, across 917 files; added edges contribute
to that count. No endpoint-to-row slots were left unassigned in this supplied
snapshot set. This confirms that an identity remap is not generally valid for
these model inputs; it does not establish the native runtime or palette identity.
The retained report `target/source-cell/skin-remap-source-census-20261105-v1.json`
has SHA-256 `1d95f9896075db7ee41c37b220422cd1044daae024a7ced091437bd38cc1d5f6`;
its inspector SHA-256 was
`d78a3d6bef36718a94c39e053eab98cbff07a0819e8d38312095750f4ebd4d4c`.
The per-file pins manifest and source-stability limits are described in the
[edge census](creature-skin-row-edge-construction-static-v1.md).

An independently reviewed x86 host executed source ranges
`4CC17C..4CC220` and `4CC251..4CC370` on three fixed synthetic fixtures.
The Rust comparator matched all 22 endpoint-map and edge-remap words, including
overwrite order, a skipped `-1`, a class-1 parent walk and an additional edge.
The host substituted initialized, owned vectors for native allocation and
resize, and preflighted the fixed fixture indices and walks. It used canaries,
not guard pages; it did not execute source mutation or establish runtime data.
The receipt is
`target/creature-skin-row-remap-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `9340a005826551b0d1256312d9a5d1dddacb631fdc4ab3ac10dfd7ed5597e7a0`.
Hashes do not establish continuous file stability or complete build provenance.

Reproduce the static trace by hash-checking the PE and decoding `4CC11C..4CC370` plus its caller `4CC491..4CC4B9`, mapping addresses through image base `00400000` and PE sections. Raw focused disassembly is retained at `target/ghidra-creature-entry-v1/skin-remap-disasm-v1.txt`; supporting read-only/noanalysis Ghidra pseudocode is `producer-build-trace-v1.log` in that directory. Tests cover overwrite order, skipped and unconsumed sentinel, flagged extra, cycles, invalid parent/link and composition. The separate fixed-fixture host is the only native-instruction execution reported here.

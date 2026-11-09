# Creature skin: query records to accumulation

Static PE/x86 trace of `D:\Games\SPORE Collection\SporeBin\SporeApp.exe`, SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. This extends [the query/ranking note](creature-generated-skin-query-ranking-static-v1.md). It does not identify the runtime object, prove a source part owns a vertex, or establish the generated skin bytes' palette binding.

## Record handoff and shared inputs

For an admitted finite ordered input, `4F8AE0` supplies the lane's **full dword** at +70 and its computed positive binary32 weight as a two-word record to `475DE0` (`4F8C0F..32`). On `475DE0`'s available-capacity path `475DE9..3A`, the helper advances the output vector end by eight bytes and copies both dwords in that order. Its grow path calls `478390` and remains a separate admission. `4F9C30` walks its returned source blocks in order, each of which contributes up to four such records in lane order. At `52B93A..52B995`, `52B700` walks that output vector at stride eight and copies the two words into local key and weight; no conversion to a byte happens at this handoff.

Here `52B700` argument 2 is the +40 vector from its `4CC060` caller (0x34-byte entries), argument 3 is the +54 vector (eight-byte entries), argument 5 is an optional vector-of-dword mapping pointer, and argument 7 is the +34 geometry object. Its weight-vector size is derived from the argument-3 vector's byte extent divided by eight; `52C740` prepares it with image zero `01485E24 = 00000000` before processing the query records. The complete allocation behavior and any concurrent alias/mutation must be admitted. In the inspected `4CC060` caller, a local mapping vector is passed at `4CC491..4CC4B4`. Its retained Ghidra pseudocode builds values by following 0x8C-byte row +04 parent words while the +0B class byte equals 1, and has a separate path for additional entries. This is not proof that the queried key, result index, source row ordinal and finalized animation palette index are identical. Native behavior on invalid parent chains and cycles, lifetime and actual source data identity remain unresolved.

The subsequent [remap trace](creature-skin-row-edge-remap-static-v1.md) now
models the row and extra-edge mapping passes with checked cycle and bounds
handling; three fixed synthetic original-instruction fixtures matched its map
words. Loader identity, intervening mutation, invalid native accesses and the
finalized palette relationship remain unresolved.

## High-bit-set key

For each record in vector order, `52B99B..B9B8` tests bit `80000000`. If set, it masks the bit and uses the remaining unsigned word to select one argument-2 +40 entry at 0x34-byte stride. It reads that entry's +0C/+10 vector range and computes its dword count (`52B9BE..B9F5`). At `52B9FB..BA1D`, the native sequence places the count in a 64-bit integer with a zero high word, `FILD`s it, executes `FDIVR` with the binary32 query weight, and commits the quotient with `FSTP32`. This happens **before** the member loop, including for count zero. The x87 precision/control, masked exceptions, stack capacity and exceptional/sticky behavior matter; it is not a single SSE division.

At `52BA23..BAC4`, each member dword is read in order. When argument 5 is nonnull, its backing array is indexed by that member and replaces the member value. The resulting value is checked as **unsigned < 64**; values at least 64 are skipped. Accepted values index the accumulation vector, and native executes `ADDSS old_accumulator, quotient` before storing the result. Duplicate members or remap collisions therefore contribute repeatedly in source order; no deduplication follows. The `<64` check does not itself establish that the actual accumulation vector has 64 entries, nor does it bound the pre-remap read. A safe fixture must independently validate the selected +40 entry, member extents, map indices and accumulation-vector lengths, as well as excluding zero count and unsupported floating results. The finite positive weights supplied by the bounded `4F8AE0` fixture are a stricter admission than the raw accumulator itself.

## High-bit-clear key

At `52BACE..BB93`, a clear-high-bit key indexes the argument-3 +54 array at eight-byte stride; its two dwords select two argument-2 +40 entries. The first three words of each 0x34-byte entry are used as endpoint coordinates. `52BB9B..BC0B` calls `41D9B0` twice to form endpoint1 minus endpoint0 and position minus endpoint0. `52BC51..BCC9` computes the dot product of those vectors and squared endpoint difference in X/Y/Z SSE order. `52BCD1..BD0C` divides the former by the latter, then applies source `MAXSS` with zero and `MINSS` with one. For finite, nonzero-denominator admitted inputs this is a [0,1] projection; native unordered/zero-denominator behavior follows the actual instruction operands and is outside that simplified description.

At `52BD24..BD7C`, a projection strictly above 0.5 selects endpoint 1 and computes `1 - projection`; otherwise it selects endpoint 0 and keeps projection. The resulting number is multiplied by image 2.0. At `52BD84..BDEC`, if `(query_weight * factor) > 0` in the finite ordered path, the original **edge key** is optionally remapped and indexes the accumulator directly: source recomputes the product and uses `ADDSS product, old_value`. There is **no `<64` check on this direct edge contribution**. A caller-supplied mapping must be long enough for the raw key and its result must fit the actual accumulator.

For the selected endpoint, `52BDF0..BE59` computes `((1 - factor) * query_weight) / member_count` with the documented x87 `FLD1/FSUB/FMUL/FILD/FDIVP/FSTP32` sequence. If the committed share is ordered positive, `52BE74..BF29` visits endpoint +0C members, optionally remaps each, skips remapped unsigned values >=64, and adds the share to each accepted accumulator word with `ADDSS old, share`. The division happens before the positivity gate and before the loop, so zero count is not a no-op under native exception rules. Mapping extents and actual accumulator length remain independent requirements.

After all records, a positive argument-10 correction value enters additional row-derived work (`52BF37..52C1A5`), potentially replacing accumulated values before ranking; see the query/ranking note. A proposed no-correction fixture must explicitly admit a finite nonpositive value. A positive argument-9 smoothing iteration count admits later `52B270` passes after the initial record writer; a complete unsmoothed fixture must also admit a nonpositive iteration count. Neither admission establishes the active native values.

## Practical next boundary

The bounded high-bit model consumes explicit ordered `(key,weight)` records,
member lists, optional remap and accumulator state with finite, bounds and
nonempty-list checks. A reviewed relocated host matched all 12 Rust output
words across three synthetic cases covering duplicates, remap collisions,
the `>=64` skip and x87 division by three. Its receipt is
`target/creature-skin-high-accum-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `313699fc3c228145db76e37d48072d886955a676dee0298d067399b239c620fc`.

A separate checked clear-high-bit model covers the endpoint projection,
direct edge contribution and selected member list. A reviewed relocated host
matched all 20 Rust output words across five fixed synthetic cases; the two
original subtract-helper calls were retained and six constant addresses were
patched. Its receipt is
`target/creature-skin-edge-accum-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `93ee442109458f9522a44d73df3b701da01f6ba437c387f0c33a144a6643d12a`.
Both hosts used owned supplied data, appended exits and no guard pages; endpoint
hashes do not establish continuous path stability or complete build provenance.
Neither result establishes the actual query/geometry, row/palette identity,
correction, smoothing or live editor.

An explicitly admitted mixed fixture replays both models in supplied query
order before ranking and writing a record, with a bounded admission proxy. A
reviewed host ran two unrolled synthetic query orders using the original branch
and helper instruction slices; all eight output words matched Rust, including a
one-bit difference caused by reordering. The host supplied the outer dispatch,
patched six constant addresses and did not run ranking. Its receipt is
`target/creature-skin-mixed-accum-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `b84fff5c45162fca7485304b35961b07bd33778dac9936996233fcc574b95df7`.
This is not original-runtime or source-geometry validation, and the host lacked
guard pages. Endpoint hashes do not establish continuous path stability.

## Reproduce

SHA-check the PE, map image VAs through sections, and inspect x86-32 `4F8C0F..4F8C32`, `475DE0..475E53`, `52B921..52BF37`, `52BF37..52C1A5`, and `4CC491..4CC4B9`. The producer's pseudocode is retained in `target/ghidra-creature-entry-v1/skin-producer-52b700-v1.log`; its address recovery and helper semantics must be checked against instructions, as done for the branch statements above. No original process ran in this trace.

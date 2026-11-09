# Generated-skin sampler threshold provenance

Static trace of `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
This does not claim the retained 0473 bytes were selected in a live run.

`4F8C40` writes its argument unchanged to query receiver +0C and writes its
sign-bit-xored word to +10 (`4F8C47..4F8C63`). `4F8C70` calls it during
construction. Two direct constructor call sites were found in executable
section E8 byte scanning, and instruction decoding confirms both supply the
same nonzero binary32 word `3C23D70A` (approximately 0.01):

* `418BDA..418BE7` loads `13EB2B0`, calls `4F8C70` after successful
  allocation, and `418BF8..418C13` binds its result to the newly created
  intermediate at `[outer+24]+38`.
* `4CB1F5..4CB249` checks the intermediate receiver's +38. Only when null,
  and allocation succeeds, it loads `13F0590`, calls `4F8C70`, and installs
  its result at that +38. An already nonnull +38 is retained rather than
  reinitialized by this slice. `4CB150` later performs property-dependent
  work and dispatches an indirect call; its `param_2` is not identified as
  the query object's +0C setter by this evidence.

`418A80` later passes its intermediate into `414400`; that calls `413D70`
with the intermediate argument. Under the previously documented same-object,
valid-allocation and lifetime conditions, `413D70` resets its +38 query and
passes it to `4FDA40`, which passes it to `4FD3C0`. The reset `4FC9E0` writes
lane count, bounds and other fields but not +0C. The inspected `4F9100`
append and `4FDA40` spans do not write that word either; `4FD3C0` reads
`[receiver+0C]` at `4FD479`, `4FD4E6`, `4FD735`, and `4FD907`. This supports
0.01 reaching the sampler **if** the freshly initialized query is the same
object and no intervening operation changes the word. It is not a complete
alias/indirect-dispatch write audit. The byte scan found the two listed direct
`4F8C70` calls and `4F8CF2`'s direct `4F8C40` call, but raw E8 scanning is
not proof that no indirect call, alternate constructor or alias store exists.

The source-audit receipt
`target/creature-skin-threshold-audit-20261108/result.json`, SHA-256
`5dc84db016477e3286b8866abfd7781e60bb3b64cb9676bd018083274ea5cfb2`,
records both constant bytes and those raw direct-call hits. Read-only Ghidra
support is `target/ghidra-creature-entry-v1/skin-threshold-4cb150-v1.log` and
`skin-identity-lifecycle-v1.log`; instruction addresses above control claims.
The 0473 payload remains only a pinned decoded row snapshot. It does not
establish provider choice, post-mutation identity, either constructor path,
query-object continuity, or the actual threshold. The zero-threshold sampler
comparison is therefore a **synthetic component check**, not a source-runtime
reconstruction or a basis to bind +0 to 0473.

A next source-informed experiment can construct builder nodes and the five
0473 class-5 segment descriptors from the admitted frozen bytes, supply 0.01
under an explicit *fresh-query/no-mutation* assumption, and execute the
reviewed isolated original sampler host with pinned MSVCR90 and a properly
activated context. It can record complete sampled lane words and bounds, but
would still be a conditional model input, not a runtime observation. A Rust
nonzero-threshold implementation is not yet supported by the zero-only model:
`4FD3C0` uses `_CIpow`, `_CIsqrt`, and for unequal-radius general-slope cases
`_CIlog` and `_CIpow`, interspersed with x87 and binary32 spills. A comparison
must bind and verify the actual loaded CRT module or independently reproduce
its numerical behavior; ordinary host `pow` or `log` is not established as
bit-equivalent. The retained 0473 direct-node radii are not all equal, so the
zero-threshold equal-radius implementation cannot represent its segments even
if the threshold were independently admitted as zero.


A reviewed isolated native reference host subsequently consumed the six
retained 0473 class-5 row records (row-byte SHA-256
`500aa927c1ac32e07b8382eaddc571e073421179ccd803a82f9f6e0cf40a6693`),
computed their direct nodes using the original helper and append spans, and
called the original segment sampler on the five adjacent descriptors with
spacing bits `3E99999A` and the conditionally admitted fresh-query threshold
bits `3C23D70A`. It used x87 027F, MXCSR 1F80 and a synthetic preallocated
receiver. Cumulative lane counts were `[6,7,8,9,10,11]`: each of the five
segments appended one lane. The first six lane words matched the separately
pinned earlier node-prefix receipt; segment keys matched their descriptors.
The runner recorded all 11 lane words and native bounds, but there is **no
nonzero-threshold Rust sampler comparison** yet.

The reference receipt is
`target/creature-skin-segment-source-native-20261108/result.json`, SHA-256
`91214c9b67b9b237c2042dc33aec9644b7cb794147149e678e19e81c53268e47`;
the raw stdout SHA-256 is
`9d5ec85678135ec0a337e52373a1f0b4b7c97018a6787b725edb949b15e5460b`.
`GetModuleFileNameA` on the loaded CRT handle returned the pinned SporeBin
`msvcr90.dll` path, and that file's hash was checked as
`ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185`.
The mapped image bytes were not independently checked. This requires the
frozen-post-mutation row admission and the fresh-query/no-intervening-threshold-
mutation admission. It does not establish that 0473 was live-selected, that
other rows/segments or a complete grid were constructed, or that the palette
or mesh was produced. The host used output canaries, not guard pages, and did
not compare sticky flags; hashes do not prove continuous stability.

The next implementation step is a checked nonzero-threshold sampler that
preserves the documented x87/binary32 spill sequence and the pinned CRT's
`_CIpow`, `_CIsqrt`, `_CIlog`, and `ceil` semantics, or explicitly admits
caller-supplied native math results at a narrowly defined boundary. Compare
all five sampled lane words and final bounds with this retained native output,
then exercise both slope branches, cutoff boundaries and different fixed
inputs independently. A Windows CRT binding must separately establish the
loaded module and calling convention; ordinary host math is not yet shown to
be bit-equivalent.

A bounded Rust **general-slope** sampler is now available as
`creature_skin_segment_crt_admitted_v1`. It accepts equal unit endpoint
coefficients, finite positive radii/spacing and a threshold strictly between
zero and one, x87 027F and scoped MXCSR 1F80. Its `CrtMath` provider returns
binary32 results at each original power/log spill; correctness beyond checked
bounds requires the caller to admit that provider's operation-accurate CRT
semantics. The low-slope branch, exceptional/native side effects and full
builder are outside this model. The implementation derives sample count,
positions, lane values and keys from its inputs and arithmetic; the retained
native output is not an input to the sampler.

The standalone `creature_skin_nonzero_source_compare_v1` experiment uses Rust
host `f64::powf`/`ln` as its explicitly experimental provider and the retained
0473 frozen-row/0.01 assumptions. Compared independently with the pinned native
reference, its cumulative counts `[6,7,8,9,10,11]`, all 11 lane records, and
lower/upper bound words matched. The comparison receipt
`target/creature-skin-nonzero-model-compare-20261108/result.json` has SHA-256
`7641ce5e86d38dbfcadb1756d96dd23db5de9b44e1cdbea93af10af838a01300`.
This agreement covers five segments from one retained snapshot only. It does
not establish general host math/MSVCR90 equivalence, the 0473 runtime row or
threshold, the original complete lane array, grid, palette, or mesh. The native
reference's loaded CRT path and file hash were checked, but mapped module bytes
and sticky flags were not compared; endpoint hashes do not establish continuous
stability or full build provenance.

# Frozen-row construction of generated skin edges

Static trace of `SporeApp.exe`, SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`, extending [the conditional row-identity path](creature-skin-intermediate-row-identity-static-v1.md). This note separates an edge-writing slice from all earlier preparation and live execution.

At `4CB4A8..4CB564` native counts the leading class-5 byte (+0B), then contiguous class-3 rows. Only if the class-5 count is positive does it count class-3 rows with +08 word mask 2 as extra edges. At `4CB5A1..4CB5C4` the endpoint count is the **signed** +02 word of the final row plus one, and edge count is row count plus that extra count. An empty row array returns false. Prior to the edge writes the source rebuilds a +40 endpoint array via row mapping and coordinates; its validity, floating operations and vector allocation are distinct admissions.

The +54 vector is erased and resized. `4CE1A0` supplies zero-pair values to the grow helper `4CF700` and the erase helper resets the end; under successful vector operations the new edge words begin zero. Source reads can therefore select future/unwritten edge slots, yielding zero words. This matters to exact sequential behavior and is not permission to read outside vector bounds.

* `4CB9AB..4CB9F2`: for each leading class-5 row `i`, pair `[i, i+1]`.
* `4CBAC7..4CBB61`: each following class-3 row's second is signed row +02. With flag 2, first is signed +02 minus one; otherwise it is signed +02 from the row indexed by signed parent +04.
* `4CBC36..4CBD1E`: for remaining rows, second is signed +02. If there are no class-5 rows, first follows the same flag/parent-row rule. Otherwise signed parent +04 indexes the **edge array**; first is that edge's first word when parent is less than `class5_count/2`, else its second. The source processes these rows in order, and in-bounds unwritten slots still contain zero.
* `4CBDE8..4CBE83`: if class-5 count is positive, each flagged class-3 row gets an appended edge in row order: first is the second word of the edge indexed by that row's signed parent +04; second is signed row +02 minus one. This extra branch also uses that signed parent to read the 0x8C row array for its +68 transform and translation copies at `4CBE87..4CBEA1` and `4CBF00..4CBF1A`; the bounded model therefore requires parent within the row array even where an additional edge slot exists. The resulting base and appended edge ordinals feed membership construction at `4CBF96..4CC004`.

`471D90` is conditionally called on the resource object before this construction in `415760`. Its `471E34/471E45` calls prepare temporary vectors, and `47205B` calls `471560` with resource/auxiliary inputs. The latter invokes virtual property methods and other helpers; a complete side-effect/alias audit was not established here. Therefore raw bytes just read from a package cannot automatically be called the later rows. The bounded model requires a stable **post-mutation snapshot**, or an independently justified no-mutation admission, plus successful earlier mapping/coordinate setup. Dynamic provider, caching, callbacks and lifetimes remain separate.

`creature_generated_skin_edges_v1` accepts explicit 0x8C snapshots and implements only the zero-initialized edge writes in the above order. It rejects empty/oversized arrays, negative or out-of-range accessed parents, invalid endpoint words, and allocation failure, returning no partial result. These checks are stricter than source and do not implement +40 coordinates, actual loader/provider choice, transforms, query construction, palette, or mesh. The prior preparation admission is essential: this slice cannot establish that original control flow reaches it or that the earlier row map and +40 storage were valid.

A separately reviewed x86 host executed the original `4CB4A8..4CB5CA`
counting prelude and `4CB9AB..4CBF96` edge-writing slices on four fixed
synthetic row sets. The Rust comparator matched all 28 edge words, including
flagged extras, future zero-initialized reads, no-spine paths, self-edges and
parallel edges. The host substituted the intervening row-map and coordinate
preparation, vector erase/resize/zero-fill, and orchestration; it retained the
source's +68 writes but did not compare them as geometry. It had canaries and
no guard pages. The checked receipt is
`target/creature-skin-row-edge-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `838741684932b3c141d887727314c764af56e4f6eddb5b65fc27e20bbd3f8d5c`.
Hashes do not prove continuous file stability or complete build provenance.

## Explicit source inspection

`creature_skin_source_edge_inspect_v1` composes the edge model with the
`creature_generated_skin_membership_v1` endpoint-list builder. It reads one
bounded decoded CreatureData file, verifies a caller-supplied SHA-256 over the
bytes actually parsed, requires an exact parse and an explicit
`--assume-frozen-post-mutation` admission, and writes a JSON report to stdout.
The flag is an assumption about the supplied rows; it is not evidence of what
`471D90` did. The report also states the required earlier native preparation
and vector-success assumptions and the unresolved runtime identity and palette.

Using the retained installed payload SHA-256
`dbbb55830f7f75eb5c45afd470a8bd0716cb08134e26f62a82674e28cd697137`, the
inspection admitted 10 rows, 11 endpoints and 10 edge pairs, with 20 ordered
membership entries. A separately edited payload SHA-256
`84b8f9c0448dc2828a0803f73264171f5a8d06d0684b90243de9b3e0ffc1eb99` produced
the same edges and membership. That edit changed translation bytes, while this
slice reads the class, flag, parent and link fields; equality does not validate
geometry or the native runtime. The retained reports are
`target/source-cell/skin-edge-source-inspection-original-20261105-v1.json`
(SHA-256 `2f3c9b987058a15af120af408476b8f5c5bbc0e5ef1d1bd9b5870e5f199d4be6`)
and `target/source-cell/skin-edge-source-inspection-translation-20261105-v1.json`
(SHA-256 `bf5a32d50daf8f827ab9bfc83d5544898d7de0b2265046d888a74bdd30536638`).
A wrong hash and a missing admission were each rejected with no stdout.

The same inspection was run on the retained, individually pinned 1,365-file
source snapshot set under `target/source-cell/capability-source-composition-v1/`.
All 1,365 were accepted: 46,418 rows produced 52,728 endpoint slots, 50,221
edge pairs and 100,442 membership entries. The highest admitted file contained
100 rows and 106 edges. `tools/creature_skin_source_census_v1.py` checks each
retained file against its existing manifest before and after the inspection,
and records errors and aggregate identities; the report is
`target/source-cell/skin-edge-source-census-20261105-v1.json`, SHA-256
`37e3bb00bfe3c6c5602727fdcf7f8a74a35782f28520d018cddba52bdc970b82`.
The retained pins manifest SHA-256 is
`276dc612a046a261d68f00693fdc969bc437c264de52c167330b4a3486537129`;
the edge-only inspector used in that run had SHA-256
`329db18cadad479fa92a70cab43ef105f7faccca99d882cbd9de2554f64ec13b`.
These counts are a model acceptance census over the snapshots, not original
runtime observations or evidence that `471D90` leaves every row unchanged.

Rebuild with `tools\build.bat build -p spore-bevy --example
creature_skin_source_edge_inspect_v1`, then pass the decoded file path, its
SHA-256, and `--assume-frozen-post-mutation`. The inspected original is retained
under ignored `target/source-cell/capability-source-composition-v1/sources/0473.creature`;
no source asset is redistributed by the example.

Reproduce the static trace by SHA-checking the PE, mapping VAs through image base `00400000` and section RVAs, and decoding `4CB400..4CC006`, `4CE1A0..4CE220`, `527780..52781F`, `471D90..4720A5`, and `471560..471D90`. Retained read-only Ghidra 12.1.4 `-noanalysis` pseudocode is in `target/ghidra-creature-entry-v1/producer-build-trace-v1.log`, `skin-edge-initialization-v1.log`, `skin-row-copy-boundary-v1.log`, and `skin-mutator-path-v1.log`; direct edge disassembly is `skin-edge-construction-disasm-v1.txt` in the same directory. Pseudocode has stack and unreachable-block warnings. The separately described fixed-fixture host is the only native-instruction execution reported here.

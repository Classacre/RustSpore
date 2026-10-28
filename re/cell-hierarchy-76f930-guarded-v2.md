# Guarded 76F930 comparison, v2

This is a controlled comparison of the original `76F930` hierarchy check and a Rust adapter. It does not establish live Cell resource selection, retargeting, or an animated pose. The earlier eight-case comparison and static source audit are in [the controlled v1 note](cell-hierarchy-76f930-controlled-v1.md); the source call and lookup limits are in [the binding lookup follow-up](cell-rw4-binding-lookup-followup-v1.md).

## Recorded results

The retained `target/cell-hierarchy-76f930-corpus-v2/report.json` and `results.json` record **150 matches across 75 deterministic synthetic cases and two placements**, seed `0x76f930`. The corpus includes empty and unmapped cases, forests, ancestor mismatches, duplicate mapped ordinals, forward B parent references, a 256-node chain, and 54 seeded random cases. The report and per-case records have SHA-256 `b1dfa4f15bbee7a785596fa416ef519f67a2586db7937bfc0cc4a8a496d4f86e` and `66ac11e46b3cd4a3b87ee156bce09fc1debbf0fbccfaefd9ca7fdcf75a4d5645` respectively. Each result includes an input hash and each process's boolean.

Separately, `target/cell-hierarchy-source-compare-v1/report.json` records matching result **1** from both executables at each of the two placements for the selected source-derived case. Its SHA-256 is `d9d1e51c03fc934952a131749bc79a96ade55936b2b74642cc133caa48e9a670`. Both reports record unchanged executable hashes before and after comparison; the source case report also records unchanged fixture hashes. This is a comparison on explicit inputs, not proof of production selection.

## Source input and scope

The supplied assignment A=clip, B=model is a caller-selected hypothesis. The native keys in `Spore_Graphics.package`, in instance:type:scope order, are A `60BDD177:2F4E681B:40606000`, binding section 17 to skeleton section 40, and B `DAE8CE17:2F4E681B:00000000`, binding section 10 to skeleton section 0. The extractor retains the legacy wide-index lookup: its apparent group values `0001FFFF` and `00010000` are preceding metadata words, and its 16-digit apparent instances concatenate scope and instance. See [the native archive index evidence](archive-index-native.md) for the corrected layout. Both selected skeletons have ID `379CC22E`, nine identically ordered IDs, and identical parent arrays. The source-style last-match ID map is identity. The fixture extractor verifies package and payload hashes and emits normalized integers only; it does not prove these resources were paired by the original runtime. Its target-only manifest retains the package, payload, IDs, parents, map, and fixture hashes.

The earlier pose check reports **93 bitwise differing inverse-bind float words**, maximum absolute difference `3.8583126`, between the two candidate bindings. A hierarchy-check result of 1 does not resolve those differences or authorize a pose transfer.

The guarded native host reads one bounded H930 case per process and validates counts, map values, A parent ordering, and acyclic B parents. It places each input array and each descriptor in a separate read-only middle page between inaccessible pages, aligned to the left or right end in separate runs. This detects accesses crossing the guarded edge for that placement and prevents writes to the readable page. It does not detect every in-page out-of-bounds read, and is not an invalid-pointer or arbitrary-input test. The native adapter's 256-element cap and Rust candidate's admitted domain are harness/API constraints, not demonstrated source-engine limits. The synthetic corpus and two source-derived placements are finite checks; full native runtime scheduling, resource association and pose behavior remain open.

## Reproduction with existing pinned binaries

From the repository root, with the existing v2 host and Rust adapter and newly named output directories, the following commands run the controlled native host and Rust adapter. The comparison scripts reject mismatched executable and fixture hashes and preserve before/after hashes. These commands were not rerun while writing this note.

```powershell
$h = 'e31f3627390f2d08df153529d1590f4e824c670e29069cde562f9e551c870876'
$r = '335cbd7a94b22cb84dd750e1fc45a55ca9387dd1328cdfbeddb62c5d718acc9c'
python tools/cell_hierarchy_76f930_corpus_v2.py --host target/cell-hierarchy-76f930-v2/host.exe --host-sha256 $h --rust target/debug/examples/cell_hierarchy_76f930_adapter_v2.exe --rust-sha256 $r --out target/cell-hierarchy-76f930-corpus-v2-repeat
python tools/cell_hierarchy_source_case_v1.py 'D:\Games\SPORE Collection\Data\Spore_Graphics.package' --out target/cell-hierarchy-source-case-v1-repeat
python tools/cell_hierarchy_source_compare_v1.py --host target/cell-hierarchy-76f930-v2/host.exe --host-sha256 $h --rust target/debug/examples/cell_hierarchy_76f930_adapter_v2.exe --rust-sha256 $r --left target/cell-hierarchy-source-case-v1-repeat/side-0.h930 --left-sha256 a9dc2dfd0745da60928fd0255bf4c61b6e8d59f6dbf71d1fcb3428b69b43dbb4 --right target/cell-hierarchy-source-case-v1-repeat/side-1.h930 --right-sha256 4062eae1f139a8ae30e38a5515d3f13dd841a2dacd767a5303347db55f6770db --out target/cell-hierarchy-source-compare-v1-repeat
```

The extractor requires exact package SHA-256 `e01bb86d8ddf02c56d52394b4e1db7181c1e1ebe1878bf6fb72c4ee523fcedd1` before and after reading. It reopens the package during processing, so the caller must supply a stable trusted file; hashing does not close a replacement race. The prepared source function is 108 bytes with SHA-256 `81fc0c1de942b4c378cf5b24662f2b5b7104e577ca6c98108e8ba46a403bc0f6`, from executable SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. Host preparation manifest `built=false` and `executed=false` state preparation-time status, not the subsequent comparison status.

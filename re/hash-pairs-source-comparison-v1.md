# Hash-pair parser source comparison

The completed bounded comparison covers all 2,590 `02D5C9AF` records in the retained 82-package native-index census, across four selected installed packages. It checked each selected native key, offset, stored and decoded lengths, and `FFFF` compression tag against the live package index; RefPack output was capped and checked against the declared size. An independent Python byte-level projection and the Rust `hash_pairs` batch example produced identical complete framed output for all 2,590 cases. This compares a disk parser projection, not a native runtime implementation, registration, provider choice, or actor selection.

The retained comparison receipt is ignored `target/hash-pairs-source-compare-v1/report.json`, SHA-256 `c647bb4516ba28b9753e4a19f719caa80283472b3aa6fb0bb24457b45ce65251`. Its Rust binary SHA-256 before and after is `43ce9c0584afd8025e43f0bd266ca3068fb975045ee67baeb2f9b97f3c7a3c32`; Python projection and Rust output SHA-256 both are `6810fe31583f976d76296d986f4f53bcbefe229ef81dcfb3502a9e936d0e4274`, and input batch SHA-256 is `270d4df7a803169a85aff5b10d3b8de71a3294924acc8783917ec09cf8c05839`. The census SHA-256 before and after is `3de1933446c61546f8afcf44e22c1010a5405baf353d0d5c9259ea6071f4e7c6`; all four package hashes agree before and after and are recorded in that receipt. The receipt itself does not bind a build command or source-file hashes to the binary; the observed binary and comparison are the supported evidence.

Every compared payload had exactly two framed sections: 65,069 records in the first and 46,453 in the second. A separate bounded read-only scan, retained at ignored `target/cell-kind4-marker-census-v1/report.json` with SHA-256 `39d1829ac601e032976c7bd4407aaae9b22cca854222c2969b7d8ad2ddc24da7`, independently reports 2,590 successes, zero failures and stable package and census hashes. It counts first-section middle words `02E1A75D` 38,014 times and `02E1A7FF` 27,055 times. The second section is nonempty in 2,160 payloads and has 400 distinct middle words. Their interpretation as native type markers remains unverified; section framing and word preservation alone do not establish their semantics.

The Rust parser now preserves each record's `marker` word, all extra sections and the explicit section count. Adding the public `HashPair.marker`, `HashPairs.extra_sections`, and `HashPairs.section_count` fields is a source API break for downstream struct literals and exhaustive destructuring. Existing consumers need to initialize or account for these fields. The actual corpus contains two sections only; additional sections are supported by the parser but this corpus comparison does not validate such source data.

The coordinating agent reported the following build and comparison commands. The receipt does not itself record or independently authenticate the command line:

```powershell
tools\build.bat build -p spore-core --example hash_pairs_batch_compare_v1
python tools/hash_pairs_source_compare_v1.py 'D:\Games\SPORE Collection' --census target/native-archive-index-census-v1/records.tsv --example target/debug/examples/hash_pairs_batch_compare_v1.exe --example-sha256 43ce9c0584afd8025e43f0bd266ca3068fb975045ee67baeb2f9b97f3c7a3c32 --out target/hash-pairs-source-compare-v1
```

A repeat comparison requires a new output directory.

The original ignored marker census was produced by `python target/cell-kind4-marker-census-prep-v1.py`; its local script SHA-256 is `d476aa1d19bccad449b677d2ac8a90e9361a40d6cb5dd3313b8c29459935c282`. The coordinating agent subsequently ran the derived reusable tool with the following command:

```powershell
python tools/hash_pairs_marker_census_v1.py 'D:\Games\SPORE Collection' --census target/native-archive-index-census-v1/records.tsv --out target/hash-pairs-marker-census-v2
```

Its ignored `target/hash-pairs-marker-census-v2/report.json` is byte-identical to the original report, with SHA-256 `39d1829ac601e032976c7bd4407aaae9b22cca854222c2969b7d8ad2ddc24da7`. A further run requires a new output directory.

The scan is bounded to a stable trusted installation; its framing checks are not a complete RefPack conformance proof. Source payloads, batch bytes and generated outputs are not committed.

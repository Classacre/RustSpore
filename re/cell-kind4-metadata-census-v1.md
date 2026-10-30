# Kind-4 metadata census

A bounded disk scan selected 2,590 `02D5C9AF` records in four installed packages from the retained 82-package native index census. The completed v2 scan verified each selected record's current native key, physical span and compression tag against its package index, then decoded its bounded payload and checked version-4 big-endian two-section framing. It reports 2,590 records, 42 matching records and no failures. This is a disk census, not evidence of registration or runtime selection.

The retained census SHA-256 is `3de1933446c61546f8afcf44e22c1010a5405baf353d0d5c9259ea6071f4e7c6`, unchanged before and after the v2 scan. The four selected package SHA-256 values were recorded before and after and matched:

- `Data/Spore_Content.package`: `0b849bd79d9c13dec5737159655bf7e2b7763cc2b79949c81370fa0490256e8c`
- `DataEP1/Spore_EP1_Content_01.package`: `d5377275dab759e68de822682cc2ee1787dc542e558f5eb43522290bdb3b0699`
- `DataEP1/Spore_EP1_Content_02.package`: `87f8a4b7965b8e1038599d10a469036a33869c7096b6c4975716097a0616ad42`
- `DataEP1/Spore_EP1_Data.package`: `5acecfc2a1e3bdd43b5d46dfefcfe79e16511c5e79edd17dc733d0415af7bd71`

Exactly 42 distinct records contain field `02DD90AF` with marker `02E1A75D` and value `3D97A8E4`, all in section zero of `Data/Spore_Content.package` with native scope `40616200`. Their native keys, selected fields and payload hashes are retained under ignored `target/cell-kind4-metadata-scan-v2/matches.json`, SHA-256 `4fe634a732eb1d7fe8bd115de59ff37abe39873650be1843354d822ed0e7a13f`. The v2 `report.json` SHA-256 is `18a611f2d79436a18b8e2cda10d8424543f3ad8625f13252798048e406374bbb`. Their package, key and payload hashes agree with the prior v1 scan. None of the 42 contains field `DDDD0000` or `DDDD0001` in either decoded section.

An independent join of the 42 exact native low-instance and scope pairs against the retained census finds 84 `00B1B104` records and 42 each of `2F7D0004`, `1A99B06B`, `0F43029A`, `030BDEE3` and `02D5C9AF`; it finds zero `00E6BCE5` and zero `3D97A8E4` records at those exact pairs. This corrects the earlier note's imprecise same-ID `00E6BCE5` statement. The 82-package join uses the retained census, not freshly checked indexes for all 82 packages, and does not establish sidecar associations or precedence.

These observations identify disk metadata candidates for the predicate in [the manager trace](cell-kind4-query-manager-v1.md). They do not establish active registration, range values, selection, fixup or an actor. Missing literal range fields could be generated or supplied elsewhere; their origin remains open. The decoder's bounded framing checks do not establish strict RefPack conformance. The v2 scan assumes a stable trusted installation; before/after hashes record stability during that run and are not a live provider lookup. No native code, Cargo or GPU was run for this scan.

# E219 source group and matrix census

The explicitly selected `E219E5D3:00E6BCE5:40617100` GMDL was extracted from the hash-bound `Spore_Content.package` snapshot. The independently reviewed extractor and externally pinned single run are retained in `target/e219-source-groups-prep-v2`, `target/e219-source-groups-result-v2/receipt.json`, and `target/e219-source-groups-execution-v2`. The first preparation was not executed: review found that two section fields needed section-local extent checks. Revision 2 added those checks and the complete absent-part key check. Four synthetic tests passed.

The model SHA-256 is `95ba0f5e5e71761ccf0f2ef919cce9bcc8d40303be4b1b7a9fa954a7c7cb8ff4`. Its five ordered part records consist of an absent external reference with field 10, followed by four explicitly referenced RW4 records with 5, 7, 7, and 7 bone rows. The source tail also stores the pair `(0, 36)`. The receipt binds each part byte range and each full decoded RW4 payload. The following section indices are serialized references, not a measurement of relocated runtime objects:

| RW4 instance | 7000C binding | 7000F matrices | 70002 skeleton | 70003 bones | matrices |
|---|---:|---:|---:|---:|---:|
| 6713CE5C | 8 | 7 | 10 | 6 | 5 |
| A1C47665 | 7 | 6 | 9 | 5 | 7 |
| 35916FD9 | 9 | 8 | 6 | 7 | 7 |
| 7A6C3F3F | 9 | 8 | 6 | 7 | 7 |

All 26 serialized 3-by-4 matrix records are identity. The five-record buffer SHA-256 is `30faf68cb04f89e92509de924ead1d2f2431e4cace145803f24095cb24bb89fb`; each seven-record buffer SHA-256 is `b231275e583d06209366486c3c75cc433c388db5c60ba0d7b55b77cc3a12e54e`. The exact separate files, model, resources, section references and hashes are recorded in the receipt. These matrices can be supplied as a controlled initial-state diagnostic, but the data do not establish that they remain unchanged when the source runtime renders.

Original `742D90` constructs the local group from the absent external part and calls `741AF0` for external parts. With successful fixup, one binding per referenced part, compatible source flags, preserved ordering, and no intervening updates admitted, the conditional groups are `(FF,10,0)`, `(0,5,10)`, `(1,7,15)`, `(2,7,22)`, `(3,7,29)`. This source-derived route explains a possible 36-slot composition; it does not establish actual registration, live actor identity, provider order, timing, or source-evaluated pose. The previously retained direct-draw mesh has 2,555 vertices and can address slot 35, so the first ten-slot group alone is insufficient for that mesh. Do not clamp or pad the indices.

The source package is an explicit stable-snapshot admission: pre/post hashes are non-atomic. RefPack input-tail consumption and equivalence to the native decoder are not established by this extraction. No native source host or live gameplay state was run for this census.

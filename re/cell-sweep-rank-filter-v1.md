# Bounded sweep rank and tag filter

After the controlled fresh append snapshot, `cell_source_sweep_pairs_v1`
sorts finite unique endpoints and derives each slot's four 16-bit positions
`[x-min,y-min,x-max,y-max]`. Its pair pass now calls the separate bounded
`BBA9C0` rank/tag predicate, rather than repeating the equivalent y-coordinate
comparison. The predicate rejects equal nonzero raw tags, skips the current
sweep axis, and compares the other axis's endpoint words as signed 16-bit
values. The checked API admits excluded axis 0 or 1; the integrated x sweep
uses 0. The existing sweep's finite, normal-or-zero, unique-endpoint, capacity
and non-mutating snapshot admissions remain unchanged.

Source `BBAD00` calls `BBA7C0` on both endpoint axes before calling `BBA9C0`;
that sort/merge has a CRT sort call and history-sensitive merge. The model's
fresh rank reconstruction is an explicit bounded interpretation for its unique
endpoint domain, not execution or independent validation of BBA7C0, persistent
source ordering, endpoint ties, concurrent mutation or live actor identity.
The source filter receives slot pointers after other operations; a real join
requires their validity and fresh post-sort contents. The controlled one-frame
fixture keeps the actor lookup in one checked registry and still stops at
pending E7F170 for a passing pair.

A separate pinned host is prepared under `target/cell-sweep-filter-prep-v1`
for independent review. It contains the complete unpatched 94-byte source
`BBA9C0..BBAA1D`, SHA-256
`0fa3e802e72ca2535cd4a72a19d3d2fd6031560ee2c7d99f96cbc77fd427ee3d`, and
uses guarded synthetic disjoint slot records. Its eight fixed inputs cover tag rejection, both excluded axes,
overlap/separation, signed link words and a boundary-equality probe.
The host cannot validate the preceding sort, slot mutation/lifetime or actual
pair emission. The six preexisting sweep unit tests and the direct rank-filter
test passed; source comparison is a distinct gate.


## Controlled original-instruction comparison

The reviewed host ran the complete unpatched function and all eight cases
matched the model's result byte. The [receipt](../target/cell-sweep-filter-result-v1/receipt.json)
SHA-256 is `5b597c55666d27123753d6c6014411243ea296d03a51298552d8176c2128c9ac`.
Model and native stdout hashes are
`b96b9ac6f7821deaab08e701906eed0ed36209f977c345b2cfcc92b075f5d3f3` and
`27b4a8d039c45382b585991c7d045c3ed3f8ad4f08ac896537598adf61b26b83`;
both processes exited zero with empty stderr. The filter model source SHA-256
was `869f1550116811e464d334b5d24b08fd36a820b1e4e3566ca1277fecf6f3bf70`
and the integrated sweep source was
`7ecc7898fb6812769f2a2ee65e98f3ea4b489b34ae11b7a79c87131ebcb51e36`.
Host SHA-256 was
`faf97623160d803d086d2fa27cd320c73173dc3d952188d29cfabc59c8b85b31`,
and runner SHA-256
`c2bf804585028b7fdc8a66bdfe3b3673494d55f9a0fedcad557386aa97f1ff9a`.

Each case is `(tag A, tag B; A links; B links; excluded axis) -> result`:

```text
0 0; [0,1,4,4]; [1,3,5,5]; 0 -> 1
0 0; [0,1,4,2]; [1,3,5,5]; 0 -> 0
0 0; [0,5,4,7]; [1,1,5,3]; 0 -> 0
7 7; [0,1,4,4]; [1,3,5,5]; 0 -> 0
0 0; [0,1,4,4]; [1,3,5,5]; 0 -> 1
9 0; [0,1,2,4]; [3,1,5,3]; 1 -> 0
0 0; [0,1,2,4]; [0,8000,2,8001]; 0 -> 0
0 0; [0,1,2,3]; [0,3,2,5]; 0 -> 1
```

Link words in the seventh row are hexadecimal; source treats them as signed
16-bit values. The last equal-rank boundary probes the predicate alone, not
an admitted integrated sweep, which rejects equal endpoint values. All rows
recorded x87 control `027F`, status `0000` and MXCSR `1F80`; linked bytes,
complete RET, controls, guard/stack checks and pre/post pins passed. The
comparison does not validate `BBA7C0`, endpoint sorting/merge, persistent
collection history, slot lifetime or real pair emission.

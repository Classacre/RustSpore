# GMDL descriptor range transfer into a graphics object

This is static executable evidence, not a live Jawhead observation. Addresses
refer to `SporeBin/SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
REA dossiers are under `target/rea/20261028-source-runtime-v6/`.

[MEASURED] At `7546C6..7546F7`, `754660` allocates a 0x100-byte graphics
object and calls constructor `740CB0`. At `754715..754722`, the resource type
is compared to `00E6BCE5`; on equality, the resource pointer at the caller
object's `+20` is passed to `742B10` with the new object as `ECX` (dossier
`710`). This is a conditional type-specific path, not proof it ran for Jawhead.

[MEASURED] In `742B10`, `742C35..742C5D` compares the source object's `+E0`
and `+E4` pointers. When unequal, it passes that half-open range to `740AE0`,
with the destination object's `+1C` vector as `ECX` and its current begin
pointer as insertion position (dossier `692`). This source layout matches the
GMDL vector populated by `7690A0`; `754660` supplies the GMDL resource type
on the cited branch. `740AE0` computes the range length in eight-byte units,
allocates or reuses vector storage, and calls element-transfer helpers (dossier
`701`). Helpers `6F3DE0`, `74B260`, and `838F30` copy both four-byte words
per eight-byte element; the last copies backwards (dossiers `715`–`717`).
These observations identify a downstream transfer of the descriptor range, not
yet the interpretation of either word.

[MEASURED] In `768730`, `768E80..768EE9` computes the number of pairs from
`+E0/+E4`, caps the count against a value on the stack, calls the stream
interface for that count, then calls its `+38` method separately with four
bytes from each half of each eight-byte pair (dossier `681`). This is a
stream-oriented access; it supplies no evidence of palette register assignment.

[UNKNOWN] The destination vector's later readers and any effect on mesh draw,
shader constants, or blend indices remain unestablished. Neither this transfer
nor the asset's compatible bounds proves an association between source mesh
order and descriptor order. Indirect calls and other unexamined code may have
additional consumers.

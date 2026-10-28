# GMDL palette descriptor use in a rendering-state path

This is static executable evidence, not a live Jawhead draw or GPU upload.
Addresses refer to `SporeBin/SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
REA dossiers below are under `target/rea/20261028-source-runtime-v6/`. The
related transfer into the graphics object's `+1C` vector is documented in
`re/cell-gmdl-descriptor-transfer-v1.md`.

[MEASURED] `73FD30` conditionally loops over the pointer vector at its object's
`+08/+0C`, passing each item and its zero-based loop index to `73EDF0` on the
same object (`73FD74..73FD9C`; dossier `756`). The same index selects entries
in the object's `+30` and `+44` pointer vectors inside `73EDF0` (dossier `751`).
In the type-specific transfer described in the companion note, `742B10` also
inserts the GMDL `+18/+1C` pointer range into graphics object `+08/+0C`;
`741010` transfers four-byte elements with memory copies (dossiers `692`,
`758`). The source reader `7690A0` creates one draw object for each source mesh
record in read order: it validates the record's two indices, attaches the
selected vertex and index buffers, and appends the object to GMDL `+18`
(`769580..769648`, dossier `659`). The vector starts empty in the newly
constructed graphics object (`713`). The synchronous loader `769A80` first
calls `7000D0` on that same GMDL,
which releases existing `+18` entries and resets `+1C` to `+18`
(`7000EE..700131`, dossier `765`). Its intervening `763670` reads into stack
locals and does not directly modify the model before `7690A0` runs
(`769AB2..769AE5`, dossiers `766` and `767`). Thus, on this loader and the direct `73FD30`
loop on a newly constructed graphics object, ordinal `i` is source mesh
record `i`. The separate caller `769A40` has not been traced to initialization.
This conclusion does not apply automatically to alternate render paths or
objects subsequently modified elsewhere.

[MEASURED] In one conditional path within `73EDF0`, if the object's `+1C/+20`
vector is nonempty, `73EEF8` reads the indexed eight-byte pair's first word.
`73EEFB..73EF03` multiplies it by 48 (`0x30`) and adds it to the selected base
pointer, storing the result at `0162F880`. `73EF09..73EF10` stores the pair's
second word at `0162F874`. The base comes from `object+F0` then `+10` when its
count matches a value in the rendering context; otherwise it selects static
address `0162C870`. If the pair vector is empty, the code instead stores that
context value at `0162F874` without this pair offset (dossier `751`). The
nonempty test does not itself prove an index bounds check.

[MEASURED] `73EF23..73EF2A` passes the address `0162F870` to `777990` with
state index 4 and a nonzero third argument. `777990` records that pointer in a
state table, marks the corresponding state bit dirty and sets a further dirty
flag (dossier `748`). This is evidence of state preparation, not by itself
proof of a particular GPU upload or shader register. In the earlier CPU path,
`740530` assembles 48-byte matrix records, but the base pointer selected here
has not independently been tied to that buffer in every case.

[MEASURED] After preparing state, `73EDF0` passes the selected item as `ECX`
to `11F94C0` (`73EFFD..73F001`). The latter binds vertex/index data and
conditionally invokes a device virtual method for an indexed draw (dossier
`764`). The GMDL reader sets that item's vertex-buffer pointer, index-buffer
pointer, and index-count field using `11F9420`, `11F9490`, and `11F9460`
(dossiers `761`–`763`). This direct path is separate from the conditional
`72D520` geometry-group construction discussed in the earlier trace.

[INFERRED] For Jawhead's measured two source mesh records and two descriptors,
the direct path selects descriptor `(0, 64)` for source mesh 0 and `(64, 19)`
for source mesh 1, if the model was successfully loaded through `769A80` and
the direct path runs on its newly constructed, unmodified graphics object.
Their shared vertex buffer does not collapse the two draw items in this path. The first word chooses the base offset in 48-byte units and the
second is passed in state data; interpreting it as a shader register count
still requires the ultimate state consumer.

[UNKNOWN] The ultimate state consumer, actual register assignment, and any
adjustments to vertex index bytes remain unresolved. No live Jawhead draw or
GPU upload has been observed; other rendering paths can group geometry.

The bounded public direct-draw resolver takes an explicit assertion of the
fresh, unmodified path and checks only vertices referenced by each draw. For
Jawhead, its output matches the independent local census for all 5,947
referenced vertices and 10,779 active lanes, including descriptors `(0,64)`
and `(64,19)`. Three focused tests pass. The comparison report is
`target/source-cell/gmdl-direct-draw-census-comparison-v1.json`, SHA-256
`f5993133a28fa1518ace0111fabfa8b2997dfed3c16c7d44a9b9988d8550102d`.
This is a conditional compatibility check, not proof the path ran or an
animated draw was rendered.

A position-only diagnostic applies explicitly supplied synthetic matrices to
separate per-draw vertex-buffer copies. On the original Jawhead asset, its
identity and per-matrix translation cases matched an independent scalar
calculation for 35,682 position components; maximum absolute errors were
`5.97e-8` and `1.82e-6`, respectively. Other vertex bytes were unchanged. The
report `target/source-cell/gmdl-position-diagnostic-source-v1/comparison.json`
has SHA-256 `5cd94cf294e3adb1dbad2105826ac5487cc90d32695e8f8a59aa6db555dc8661`.
This is synthetic, position-only deformation; no actor pose, original clip,
native arithmetic equivalence, or corrected normal/tangent lighting is claimed.

The opt-in `--diagnostic-skin-translation` path was also captured offscreen on
the NVIDIA RTX 3060 using the checked original non-skinning shader recipe. Its
PNG SHA-256 is `66cbac4985278988f20f2648404ed94334dcdfb24cca8895b2872fd8f8c2026a`;
the normal capture remained byte-identical to the retained baseline, SHA-256
`b06a039f1c3d8ff838a699ba991cc480a9cb973ac66059a51d94d074f96ebfad`.
The images differ at 72,559 pixels. The synthetic translations intentionally
separate the mesh sections and leave normals and tangents unchanged; this
checks plumbing and does not demonstrate native animation or a retail scene.

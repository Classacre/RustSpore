# A parallel preparation of the creature mesh intermediate

The pinned installed `SporeApp.exe` has SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. This follows [the generated-mesh corner join](creature-generated-mesh-corner-join-static-v1.md), and separates a newly traced preparation step from the still-unresolved upstream surface and skin producers. It is static evidence, not an observed edit, model regeneration or game run.

## The same inputs in another native path

`462F30` receives an object in its second argument whose `+34` pointer and `+7C/+80` vector are the same offsets and record shapes used by export initializer `46E3C0`. On its valid-input path, it copies or prepares data from the `+34` object and then builds temporary arrays from the 20-byte records at `+7C`. This is a second consumer/preparer of the supplied intermediate. It does **not** establish that the two calls received the same object in any session or that either object was generated from a particular CreatureData payload.

The concrete skin-array preparation at `463397..46351C` computes record count `(end-start)/20`, allocates two destination vectors, then for each source record in order:

* `463459..4634DE`: zero-extends bytes `+0..+3` into four successive 16-bit values.
* `4634E3..463515`: copies the four raw 32-bit words at `+4..+13` into a separate 16-byte destination record using integer moves.

This path's widened indices are **not multiplied by three**. That differs from `46E776..46E7C5`, which selects the 20-byte record through a corner's position index and emits each byte times three. The two formats must not be silently interchanged. A checked, nonaliasing slice transformation accepting `N` explicit 20-byte records and producing `N` four-u16 tuples plus `N` four-word tuples is an implementable next step; it should preserve all 16 payload bytes exactly, bound `N` and lengths, and test ordering, values 0/255 and unusual float bit patterns. It would model this preparation only, without assigning part ownership or promising a generated GMDL. A later join to corner indices requires an explicitly admitted common intermediate and bounds; the source alone does not supply that identity.

## Parallel geometry attribute preparation

`4630EE..46310A` passes the vector at `+34` object's `+30` to `4742E0`, then `46310F..4631A5` walks the resulting eight-byte records. It leaves their first word alone and replaces the second float with `1.0f - old` using scalar SSE; the PE constant at `014861D0` is binary32 `3F800000`. The copied-vector helper's successful contract and floating controls are additional admissions, and this does not identify the attribute's higher-level meaning solely from its shape. `4631A7..463329` uses `(end-start)/24` for the `+34` object's `+44/+48` records and forms two separate 12-byte vectors by integer-copying the first and second triples in order. The actual array contents and the producer of these attributes remain unknown. The representation conversions can be inspected or separately implemented with an explicit SSE policy; they do not deform or regenerate a mesh.

## What this does not close

The 20-byte records, source positions, eight-/24-byte attributes and corner/index arrays are **inputs** to both inspected paths. Their producer, the attachment and deformation inputs derived from an edited CreatureData row, transform spaces, body surface construction and any source-row-to-influence mapping are not established. In particular, neither this parallel preparation nor a single-index match proves that only a selected portion of a baked mesh can be changed safely. The next producer trace must identify the writer of the export object's `+7C/+80` and `+34` arrays, demonstrate its input object identity, and follow any virtual or indirect calls rather than infer ownership from the output shape.

## Reproduction

Retained fresh read-only Ghidra 12.1.4 output is `target/ghidra-creature-entry-v1/geometry-upstream-ghidra-v2.log`; `CreatureGeometryUpstreamV1.java` and the project are in the adjacent `scripts` and `project` directories. The selected `462F30` body was disassembled/decompiled in a `-noanalysis -readOnly` project; the pseudocode contains stack-recovery warnings and is not source. Independently verify the instruction-aligned PE ranges `4630EE..46332E` and `463397..46351D` with x86-32 Capstone and check the above hash and constant. The earlier exporter dossier and its limits are cited in the linked note. No original instruction, game process, GUI or Cargo command was executed.

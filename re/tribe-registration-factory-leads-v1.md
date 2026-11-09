# Tribe registration and factory leads (2026-10-28)

Scope: static Ghidra analysis of installed SporeApp.exe SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Retained provider outputs are under ignored `target/rea/20261028-source-runtime-v6/`.
The accompanying `tools/tribe_registration_evidence_audit_v1.py` verifies the
executable digest and bounded registration observations against dossiers 119 and 122.

At `0xb25bda` / `0xb25bdf` the initializer pushes the name
`cCreatureTribeMember` and word `0xef3da0d7`, then calls `0xb23af0`.
Dossier 362 shows that helper copying the name and passing data to generic
mapping routines; it does not establish a constructor. The only Ghidra xref
to the name is this push (361). A complete byte-pattern search (366) found
that word at its immediate operand and at `0xc24540`; the latter is not an
instruction boundary and has no containing analyzed procedure (369–370).
It is not established as an executable reference. Name registration is not
proof the object is instantiated or active in Tribe stage.

The larger registry at `0xb245d0` associates the `cTribeFoodMat` registration
key `0x629bafe` with a returned slot populated with `0xb1eee0`; for `cTribeTool`
the key is `0x18c8f0c` and slot value `0xb1f060` (119). These are candidate
factory targets, not established constructors. Ghidra has no function at
either address (367–368), though each target begins with a decoded `PUSH 0`
(371–372). A bounded disassembly, entry and boundary validation, and tracing
of any called allocations and constructors are necessary before claiming a
construction path or order or food behavior. No gameplay constants or rules
are admitted from the experimental Tribe simulation.

Dossier 315 shows a decompiler candidate for a `TribeGame` transition label,
but its analyzed function body skips `0xb5da0b–0xb5da59`, where the switch
block would fall. It is therefore a lead rather than a verified entry path.

A bounded, Ghidra-confirmed linear decode
(`effects-tribe-factories-linear-v1.json`) provides the next static leads.
At `0xb1eee0`, the decoded sequence pushes `0x110`, calls `0xf472b0`,
tests EAX and, when nonzero, moves it to ECX and jumps to `0xc9bc50`;
zero returns zero. The analogous `0xb1f060` sequence pushes `0x250` and
jumps to `0xc9d430` on the nonzero branch. These resemble allocation and
constructor paths, but function boundaries, actual runtime reachability,
allocator semantics, and destination behavior remain unverified.

Destination dossier 379 (`0xc9bc50`) initializes a byte at object offset
`+0x108` to 1, words at `+0x10c` to 0 and `+0x88` to 2, and stores two
results from calls to `0x932b30` at `+0xc4` and `+0xcc`. The call inputs
are respectively `trg_sacrifice_altar1` and `TribalTools`, each with
`0x811c9dc5` and `1` as the other arguments. The meaning of those words,
the helper, resources and subsequent consumers are not established here.
The link to FoodMat remains conditional on the candidate factory entry and
slot assignment above; runtime invocation has not been observed.

Destination dossier 380 (`0xc9d430`) initializes several pointer-like values
and clears words at offsets `+0x1ec` through `+0x1fc`, and `+0x240` through
`+0x248`; it has no referenced strings. The relationship to the Tool
registration is likewise conditional. These field writes do not establish an
order, food transfer, equip action or tool behavior.

Dossier 392 shows `0x932b30` uses a multiply-then-XOR byte loop and, for
flag 1, calls `tolower` per byte. Restricted to the two NUL-free ASCII inputs
above, the auditor models that loop with 32-bit wrapping: the computed values
are `0x1e42e0c6` and `0xf37c24e6`, respectively. These are calculated values,
not an observed runtime or evidence of what consumes them. Locale-dependent
non-ASCII behavior is deliberately outside the model.

The first candidate FoodMat pointer table includes `0xc9bd90` (read-only PE
pointer inventory; dossier 403). This function conditionally clears nonzero
words at object offsets `+0x10c` and `+0x2c`, calling each former pointer's
virtual slot `+4`, then calls `0xc8b220`. This is an observed cleanup-like
sequence, with its association still conditional on the table and factory
leads; it does not identify the objects or establish food behavior. Several
other queried table targets have no Ghidra-defined function (400–402, 404).

A byte search for `0xf37c24e6` (398) found a match in analyzed function
`0xc8de50` (405). Its decompilation passes that immediate as the second
argument to an indirect call in branches where its first parameter is 0 or 1.
This corroborates a consumer of the same word but does not establish a link
between the function and FoodMat, a resource lookup, or gameplay behavior.

## Candidate TribalTools property records

Dossier 405 (`C8DE50`) shows selector 0 and 1 supplying IDs `A1001C58` and `BCB8A199`, respectively, along with the constructor-derived `F37C24E6`, to an indirect virtual `+0C` call. The retained native index census lists matching `00B1B104:F37C24E6:A1001C58` and `00B1B104:F37C24E6:BCB8A199` in both explicit PatchData and Spore_Graphics archives. A local exploratory decode of the explicit PatchData pair gives UTF-16 property labels `Totem Pole Part - Combat` and `Totem Pole Part - Social`; this is a key-word correspondence and not yet proof that the indirect service resolves these records or that either is selected in play. A reviewed, hash-bound standalone intake is prepared at `target/tribe-totem-intake-prep-v3/inspect.py`; its independently pinned execution is pending. RefPack consumes 162 of 163 stored bytes in the exploration, leaving `FC`; original decoder acceptance remains unresolved. These records supply a concrete source-data inspection seam, not validated combat/social rules or live scene placement.

The externally pinned v3 intake failed its label assertion. The decoded string has 24 counted UTF-16 units and is immediately followed by the next property header at byte 64; v3 incorrectly required a NUL terminator there. Its failure output is retained under `target/tribe-totem-intake-result-v3/`. A fresh v4 checks the next header instead and requires independent review and a separate run.

The v4 intake also failed its combined assertion. Independent diagnostics on both records verified the counted UTF-16 label bytes are little-endian, and verified the next property header at byte 64; the v4 script incorrectly decoded the labels big-endian. Its output is retained under `target/tribe-totem-intake-result-v4/`. A fresh v5 corrects the byte order and validates both records before emitting its report; review and execution are separate.

The reviewed, externally pinned v5 intake completed `matched_static` for both records; `target/tribe-totem-intake-result-v5/receipt.json` SHA-256 `2f015e4c09bc12786be8d8b066f3de1200ead009debe1f4211d3b5341e85abda`, stdout SHA-256 `50791337a059a6b08b871bf6e827a90fc61aa0096cbae3c80555c2199a9c77de`, empty stderr. The corrected labels are UTF-16 LE; each record decodes to 200 bytes from 163 stored, with 162 consumed and `FC` remaining. This checks an explicit stable package, not virtual-provider identity, runtime selection or gameplay behavior. The independently preserved v3 and v4 failures above are not successful comparisons.

A bounded decode of the same pinned executable refines the caller boundary: retained dossier 405 lists incoming calls at `CDB045` and `CDB121` from `CDAF40`. The shown path requires a non-null `C90690` result and a nonempty array of 16-byte records, then tests the last record's `+0C` byte for zero and sets it to one. Record `+04 == 2` leads to `C8DE50` selector 1; `+04 == 1` leads to selector 0. The direct selector-to-key choice is source evidence, but the record's identity and meaning, actual provider behind virtual `+0C`, runtime reachability, and downstream gameplay effects remain unresolved. This is a static path, not a live selection claim.

A bounded original-code host executed the source branch cut `CDAFEF` and complete `C8DE50` for supplied record values 1, 2 and 3, with a synthetic provider returning null, stable supplied vector and guarded preallocated append slots. Its independently reviewed and externally pinned run matched the three scripted expectations, exited 0, and reaped the child before closing its private desktop; `target/tribe-selector-append-result-v3/receipt.json` SHA-256 `955b1a00d03b7f57af14c0734f7ca1f42263465e5fb5fa14ba6ad2038f13e90a`. The linked source slices were statically audited; the runner did not independently disassemble the complete wrapper or callback and separately pinned source/binary do not prove build provenance. It does not execute earlier source gates, provider-success behavior, dynamic growth or live gameplay. Preparation v1 had an assembler symbol failure; v2 omitted a per-row vector reset and was not executed; both remain preserved.

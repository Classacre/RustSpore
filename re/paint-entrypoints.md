# Creature paint entry points, 2026-10-22

## Target and coverage

`[MEASURED]` Static analysis of the base-game `SporeBin/SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Ghidra 12.1.4 completed the configured analysis in **676 seconds**, according to its log.
Seven selected functions returned pseudocode and assembly. The executable's SHA-256 was
unchanged afterward. No game runtime was launched.

`[MEASURED]` Profile digest:
`07956626c4c75f6e693f1731c6ed414b61cf009d055c0d72055d224a32d58a49`.
Disabled analyzers: `Stack`, `x86 Constant Reference Analyzer`,
`Decompiler Switch Analysis`. Automatic local-variable recovery, propagated constant
references and switch/jump-table recovery therefore have reduced coverage. These results
are specific to this executable and profile; addresses must not be transferred to EP1.

Complete local Evidence is retained under ignored
`target/rea/20261022-paint-focused-v2/`. `evidence-bundle.json` exports **10 records,
586,810 bytes**. The JSON contains proprietary recovered representations and stays out of
the repository. This document records facts and investigation leads, not implementation
code.

## Observed path

`[MEASURED]` The defined strings `ArgScript/SPSkinPaintFloodEffect` at `0x13f2ae4` and
`ArgScript/SPSkinPaintFloodEffectBlock` at `0x13f2aa4` both reference the 192-byte function
at `0x535900`. Its recovered representation passes `SPSkinPaintFlood` and two constructed
objects to separate virtual calls. **Both indirect destinations remain unresolved.**
Calling this a registration routine is `[INFERRED]` from that structure.

`[MEASURED]` The block's constructor-like routine `0x5359c0` installs pointer table
`0x13f2b08` and stores constant `0x02c1642d` at object offset `0x34`. The constant's role is
`[UNKNOWN]`.

`[MEASURED]` Complete reads recovered six pointers at `0x13f2acc` and eight pointers at
`0x13f2b08`. Their installation as object-first-word pointers supports interpreting them
as virtual-method tables `[INFERRED]`; individual method roles require separate analysis.

`[MEASURED]` Table entry `0x535ea0` forwards through two helpers, passing constant `0x27`
to one of them. The same constant appears in block handler `0x535a60`, which allocates
`0x5c` bytes labelled `Skinner` and chooses between `0x5356e0` and `0x535d10` based on a
helper-produced count-like value. Argument-driven default/reference selection is
`[INFERRED]`. The handler's final virtual call remains unresolved.

`[MEASURED]` `0x5356e0` initializes an object with table `0x13f2aa0`; `0x535d10` installs
the same table and copies tail fields from another object. The default tail includes
zero values, one byte set to 1, `0x3f800000` and `0xffffffff`. Their field semantics are
`[UNKNOWN]`; the float-shaped bit pattern does not establish a strength or colour field.

`[MEASURED]` Function inventory within this investigation:

| Address | Body bytes | Evidence file | Interpretation |
|---|---:|---|---|
| `0x535900` | 192 | `04-analyze_function.json` | registration-like; 2 unresolved calls |
| `0x5359c0` | 159 | `05-analyze_function.json` | block constructor-like |
| `0x535480` | 44 | `07-analyze_function.json` | destructor-like; conditional deallocation |
| `0x535ea0` | 53 | `09-analyze_function.json` | forwarding wrapper |
| `0x535a60` | 474 | `10-analyze_function.json` | block handler; 1 unresolved call |
| `0x5356e0` | 158 | `11-analyze_function.json` | default constructor-like |
| `0x535d10` | 173 | `12-analyze_function.json` | copy-like constructor |

Role labels are `[INFERRED]` from the observed instructions and pseudocode. These are
Ghidra representations, not original source, and resolved call edges are not a complete
call graph.

## Remaining questions

`[UNKNOWN]` Paint-layer syntax, palette correspondence, blending, projection,
rasterization and the algorithm that produces shipped skins remain unestablished.
Nothing here validates or replaces `spore_core::skin`'s approximate palette colouring.

Potential follow-ups are the observed constructor helpers `0x5357c0` and `0x535e40`,
and the forwarding helper `0xa6fa70`. Their implementations were not inspected in this
pass. Keep a single REA session alive for successive queries: after startup, the selected
function dossiers completed in fractions of a second.

## Evidence identifiers

- String trace: `ev_7c3f762158203091eba9490008570b1a83faf0b47191d6f5aeabcdb69208b1a7`.
- `0x535900`: `ev_f1d0ebec4299ed4af03172c7bd3dbe665df3c8ac616e7ddf13df7eaddb608292`.
- `0x5359c0`: `ev_db78e2a218c1d6f7025c10abf6a56ef42f7773a9dc69094c0c720f8896388202`.
- Effect table read: `ev_0968909d6e51ba5945da4b8054d06e2994afdcdfaa141d3ab81e6e761409c456`.
- `0x535480`: `ev_c91ed6bdebec9f30ae105e16824337dc939d21514fa8a87b5a549fbe9c791ae8`.
- Block table read: `ev_6e2bebaf065c494b1e9bc0ae808e535954b1aaa18c3522fa9fdb9d973dc5ce16`.
- `0x535ea0`: `ev_7edc7614d4a435a457f1bd5c02588ffc274d6795f30a4b392a37ecd4fb13af6f`.
- `0x535a60`: `ev_df17d6eba948537723f66cf7ffe98bf860e20e3f8d4bd44c8238a488c6c40d13`.
- `0x5356e0`: `ev_4c716b577a4f40e6ba04cf9cfde39d66d496220d69f43f29801c50e4055151e5`.
- `0x535d10`: `ev_24cd44645c995127ac4eee6c4bfcefc61e1478783bca693621aeb4505f4ab7a9`.

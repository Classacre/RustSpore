# Space and galaxy entry leads, 2026-10-28

Shipped base `SporeBin/SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
REA/Ghidra q304 finds `GalaxyGameEntry` at `0146334C`, referenced from
`00B5D860` and `00DDDD60`. Dossier q315 shows `00B5D860` distinguishing
`CellGame`, `CreatureGame`, `TribeGame`, `CivilisationGame`, `SpaceGame`,
and `GalaxyGameEntry` strings in a transition-like branch. On comparison
with `02CCD1D2`, it passes `GalaxyGameEntry` to `00454E80`, followed by
`00401090` and `004DF9E0`. The missing original types and uncertain decompiler
stack mean exact arguments and transition semantics need additional evidence.
Dossier q316 shows `00DDDD60` registering an allocated object via virtual call
with `02CCD1D2` and `GalaxyGameEntry`; this does not establish object behavior.

REA q305 finds `SP_Simulator/cSimulatorSpaceGame/COM` at `01462AB4`, with a
reference at `00B5C0A8` outside a recognized function. q317 decodes that
instruction as a push of the string address; its enclosing operation remains
unknown. These are static entry and ownership leads, not demonstrated gameplay,
world state, or save serialization. In PatchData, a PROP instance hash matching
`Planet_Space` exists, but REA q306 finds no direct corresponding byte constant
in the analyzed base executable. No runtime usage or data meaning follows.

A bounded linear decode at `00B5C080` through `00B5C0D1` (stored under
ignored `target/source-space/space-com-entry-linear-v1.json`) shows an apparent
separate sequence starting at `00B5C0A0`: push `0xF0` and the simulator COM
string, call `00F472B0`, then on nonzero call `01006B70` with the result in
ECX; if that result is nonzero return it plus eight, otherwise return zero. The
sequence is consistent with a factory-like path, but entry boundary, reachable
caller, and target meanings are not established. The previous sequence ends
in `ret 8` and `int3` padding; do not combine them as one function by proximity.
Dossier q322 shows `004DF9E0` copying three argument words into object offsets
`+A4`, `+A8`, `+AC` before service calls, without sufficient evidence to name
their meanings or identify them as save state.

Dossier q326 shows `01006B70` assigning several vtable pointers and object
fields, calling an allocator-like routine with size `0x108` and label
`Simulator`, calling `00FE6C10` after success, and assigning global
`016DD0DC`. This supports a constructor-like interpretation. q327 reports a
data reference from `01463008` to `00B5C0A0`; q324 reports an adjacent data
reference from `01463004` to `00B5C080`. q328 reports no incoming references
to `01463004`. A possible factory table is not proof of registration or
reachability, and these observations do not define a galaxy data format.

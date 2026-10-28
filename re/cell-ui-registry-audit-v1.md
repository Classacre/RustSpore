# Cell UI registry mapping audit v1

The executable identified in `cell-ui-candidate-audit-v1.md` contains a concrete
registration path linking the requested type `0510A95B` to serialized types
`250FE9A2`, `050F4731`, and `EFBDA3FF`. This narrows the earlier type mismatch;
it does not establish which layout or package is displayed.

* During the guarded initialization function `815C10`, `816092..8160BC` constructs an object whose vtable
  is `1419314`, obtains a manager through `67D7F0`, and calls manager slot
  `44` with enabled flag 1 and that object. The vtable matches the resource
  manager at `1437740` documented in `archive-index-native.md`.
* Slot `44` is `8E1440`: its enabled branch invokes the object at slot `10`,
  calls slot `2C` to enumerate its supported types, and inserts the object into
  the registry keyed by those types. Decompiler output has uncertain local
  variables; the slot calls and overall registry insertion are observable.
* Object slot `10` is `80FC60`. It calls manager slot `24` (`8E1320`) with
  `0510A95B` and the ordered array `[250FE9A2, 050F4731, EFBDA3FF]`.
  The callee builds mappings from the provided array. Object slot `2C` is
  `80FCB0`, which enumerates the same three types in reverse order; slot `30`
  is `80FCE0`, which accepts any one of the three with `0510A95B` as the
  second argument. The latter two entries are bounded linear instruction
  observations, not Ghidra-recognized function boundaries.
* `8DF310`, manager slot `48`, looks up candidates by first type and tests
  slot `30` against the second. `8DE8A0` uses the type mapping and this
  compatibility check while attempting provider loads. This supports a SPUI
  alternate-type path for the caller's `0510A95B` request, conditional on
  initialization, provider success and runtime state. It does not prove that
  this path succeeds in the Cell screen or exclude the other candidate types.

The relevant raw dossiers are in ignored
`target/rea/20261028-source-runtime-v6/{62,63,65,69,72,74,75}-analyze_function.json`;
the executable is SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The six control IDs and candidate payload hashes are recorded in the earlier
Cell UI audit. A file hash match and the supported type mapping are evidence
for candidates, not proof of active layout selection.

The database list uses descending signed priority and first successful query,
as documented in `archive-index-native.md`. That evidence does not establish
which installed product databases are registered or their effective order in
this run. Both different SPUI payloads and both different PNG payloads remain
candidates; no package winner or rendered HUD is asserted.


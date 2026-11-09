# Cell indexed-input to player target: static direct route

Static x86 PE inspection of `D:\Games\SPORE Collection\SporeBin\SporeApp.exe`, SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. This extends [the player target bridge](cell-player-target-input-static-v1.md) and [modified input trace](cell-input-modifier-pick-static-v1.md). The requested `cell-player-target-bridge-v1.md` was not present; those are the existing relevant notes. No native code or live input ran. All branches below require the named objects, pointers, callbacks and normal returns to be valid and their identities stable at the relevant operations.

## Input-state producer and frame caller

The `Game_Cell` table `01486008` has +24 = `E81920`, +28 = `E50E70`, and +3C = `E809B0`. The registration and event-routing conditions are in the earlier player-input note; this table alone is not proof of live dispatch. `E81920..E8195F` calls `E82930` with its two incoming words; only if the returned low byte AL is zero does it pass those words to `6974E0` with ECX = `[016B4BEC]`. `6974E0..69750B` accepts an unsigned first word at most `FFh`, then sets bit `(code & 31)` in dword `(code >> 5)` of that state and calls `697450`. `E50E70..E50E87` calls `697510` on the same global state; that helper applies the same bound and clears that indexed bit. The second argument has no effect in the inspected bit operations. A different `697540` path handles the mouse-associated code range and must not be substituted for these bit setters. Input message translation, active receiver, code naming/layout and timing remain conditional; numerical code matches alone are not observed key presses.

`E809B0` forwards into `E806E0`; one conditional path calls `E5B610` at `E8086B`. `E5B610..E5B648` requires `[016B4BE4]+5158` zero and resolves the handle at `+411C` through pool `+1C` via `B71CC0`; on successful resolution it calls `E5B160` with the incoming binary32 scalar. `E5B160` separately repeats the pool resolution. The two lookups must identify valid intended actors, with relevant globals and object lifetimes stable; this establishes a possible scheduled call chain, not a live player or observed frame cadence.

## Indexed state to actor target

`E5B185..E5B3D5` starts a local vector with the three words at global addresses `016B4C08/0C/10`. Four conditional bit groups modify it in sequence. Each group also clears state `+C4` and sets byte `[[016B4BE4]+5190]+E0` to 1 when reached. The inspected image constants at `015A8C34..3C` and `015A8C28..30` are `(0,1,0)` and `(1,0,0)` respectively. The first group adds the former, the second subtracts it, the third subtracts the latter, and the fourth adds it. Each group tests alternatives joined by branch logic:

| Group | Indexed state bits tested |
|---|---|
| Add `(0,1,0)` | `57h` if `E844A0` returns AL zero, otherwise `5Ah`; or `26h`; or `68h` |
| Subtract `(0,1,0)` | `53h`, `28h`, or `62h` |
| Subtract `(1,0,0)` | `41h` if `E844A0` returns AL zero, otherwise `51h`; or `25h`; or `64h` |
| Add `(1,0,0)` | `44h`, `27h`, or `66h` |

`E844A0` is actually called separately in each group; only its returned low byte AL chooses the first and third indices. The second and fourth calls' results do not select an index in these instruction windows, but their side effects are not excluded. In the first and fourth groups the X `ADDSS` destination is the basis constant and the source is the previous X value; the Y and Z additions use the previous component as destination. This operand order matters for exceptional values and zero-sign behavior. The branch conditions, bitfield producer and numeric codes are static facts; equating these codes with a particular physical keyboard or locale requires the event-code mapping and runtime dispatch. The vector's initial global words and `E844A0` provider result are runtime inputs, not fixed source-state defaults.

At `E5B3D5..E5B3FF`, SSE separately computes `z*z`, `y*y`, adds those, computes `x*x`, then adds it to that sum; it takes the later path only on an ordered comparison greater than image float `01485C9C = 1.52587890625e-5`. That path loads the rounded binary32 sum into x87, adds image float `013EC4B0` (approximately `1e-8`), computes square root and reciprocal, and stores it to binary32. It separately multiplies the three local components by that committed reciprocal. `E5B428..E5B459` passes the original incoming scalar, the normalized vector, and state vector at `+CC/+D0/+D4` to `E5AF20`; the returned three words replace that state vector. The complete `E5AF20` branch and helper arithmetic matter and are not replaced here by an assumed interpolation formula.

`E5B476..E5B4D7` reloads state `+CC/+D0/+D4`, multiplies each by image float `013EECD0 = 20.0`, adds the corresponding resolved actor `+4C/+50/+54` position word, stores the three resulting target words at actor `+08/+0C/+10`, clears actor byte `+04`, and sets state byte `+C8` to 1. The separate selected-object route `E6F620` can also write these actor target fields from a resolved `+C4` handle; no ordering or precedence between the two routes is asserted. If the magnitude branch is not taken and state `+C8` is exactly 1, `E5B4E4..E5B517` clears it, sets actor byte `+04` to 1, and copies the global baseline vector into state `+CC/+D0/+D4`; it does not perform the above target stores on that local path. Unordered/exceptional comparisons, mutable globals, control words and indirect side effects require separate treatment.

This identifies an actionable source path from caller-admitted indexed state to a player actor target, not proof of current runtime key state or a complete playable controller. A faithful implementation still needs the callback routing and bit lifecycle, both actor-pool resolutions and source actor identity, baseline/global state, incoming scalar and exact `E5AF20` behavior and FP policy, plus ordering with `E6F620`, movement integration and animation. A supplied synthetic bitfield can test the conditional arithmetic but cannot establish the native input producer or live scheduling.

The smallest independently testable first domain is the finite local-vector assembly and branch selection with explicit baseline words, an indexed-state snapshot, and four separately admitted `E844A0` results, under declared masked-exception SSE policy. It can report the four reached side-effect requests without applying them to a real state. A subsequent exact target fixture must also model the committed x87 normalization and `E5AF20` or admit its returned vector explicitly; it must not present an admitted result as an executed native helper. Include cancellation, opposing/simultaneous bits, both layout-result branches, threshold neighbors, signed zero and invalid/nonfinite transactional rejections. Source-side callbacks and side effects can mutate state between the four lookups; a frozen snapshot is a stricter controlled-fixture admission.

`cell_indexed_input_vector_v1` implements the frozen finite vector assembly.
A reviewed host executed the original `E5B185..E5B3D5` span on one synthetic
bit snapshot with four controlled callback stubs. The three returned vector
words and aggregate C4/E0 effects matched the Rust model; the final memory
cannot distinguish each individual store. The receipt is
`target/cell-indexed-input-native-host-prep-20261105-v2/reviewed-fixed-result-v1.json`,
SHA-256 `eea707b4e6d581d05e0bd5d26eebb10654ecd9f980c571442394bdff9f28c26d`.
The separate `cell_input_player_target_fixture_v1` composes vector assembly,
normalization and the helper's early branches into an explicitly synthetic
target update; its default entry point rejects the long path transactionally.
An optional entry point accepts supplied committed CRT words and a period for
that branch without executing the real CRT. One
reviewed host then executed the original update, checked pool lookup and early
helper against a synthetic pool/actor and four fixed callback stubs. The target,
direction and modeled flags agreed with the Rust composition. The host receipt
is `target/cell-player-target-integration-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `dc20425ac472b8389592b8fd38d135fc0af5b6bff8c9d79df6771fe597e48be6`.
The Rust fixture is given a resolved actor; the native host performs the
checked synthetic pool lookup. This covers one active early-return case.
Neither fixture establishes live callbacks, keyboard events, player ownership
or gameplay. The host lacked guard pages, and endpoint hashes do not prove
continuous input stability or complete compiler/build provenance.

## Reproduction

Hash the executable, then decode PE-mapped x86-32 ranges `E81920..E81965`, `E50E70..E50E87`, `6974E0..69753E`, `E806E0..E80870`, `E5B610..E5B650`, `E5B160..E5B522`, `E5AF20..E5B155`, and `E6F620..E6F6CC` using `pefile.get_data(start - ImageBase, end - start)` and Capstone from instruction-aligned starts. Read the table words at `01486008+24/+28/+3C` and image constants above. A bounded `.text` scan for `E8 rel32` finds `E8086B` to `E5B610` and `E5B648` to `E5B160`; this encoding scan is not a complete caller census. No Cargo, GUI, game execution or live capture was used.

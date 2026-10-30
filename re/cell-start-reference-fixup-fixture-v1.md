# Cell start-reference fixture

`crates/spore-bevy/src/cell_start_reference_fixup_v1.rs` is a private,
unwired fixture for the `globals.startCell` type-3 reference path. It is not
a Cell update system and does not prove a live player or a resource-manager
winner. Source addresses and the canonical package observations are detailed
in `cell-player-start-definition-static-v1.md` and
`cell-start-reference-source-bound-test-plan-v1.md`.

The fixture accepts caller-supplied resource keys, byte slices, provenance
labels and a symbolic descriptor identity. It checks the scope-zero globals
and Cell keys, parses their exact structural extents, extracts the globals
start ID and interns the pair of ID and descriptor identity. Handles carry a
process-local table identity as well as an index; a different table cannot
resolve them. Descriptor tokens and provenance labels are assertions by the
caller, not captured native pointers or authenticated manager decisions.
Parsing the Cell bytes establishes their extent, not semantic validity or
native reference fixups.

The fixture deliberately checks both supplied resources before mutating the
table. Original `E4AAD0` instead interns a nonzero field during fixup before
later resource acquisition; this includes `FFFFFFFF`, whose ordinary lookup is
then rejected by `E82200` and may enter fallback. The fixture's atomic errors,
capacity, identity checks and sentinel rejection are bounded safety policies,
not reproduced native failure order. It does not implement the original list,
allocator, field-pointer write, provider/cache state, fallback, actor or pool
lifetime.

## Validation

The full all-target workspace run retained at
`target/workspace-tests-20261030-resume-v61.log` reports **818 passed, 0
failed, 4 ignored** across 171 result summaries. Its SHA-256 is
`df087116fb068f612d37c1eb02370ade4400530ae1ba2e4d27ba7a12af5864b1`.
The ignored installed-data test described below was run separately and passed;
it is not included among that workspace run's passed tests.

Seven synthetic unit tests passed. They cover descriptor/ID interning,
scope and layout errors, supplied/unavailable choices, sentinel handling,
table-bound handles and capacity. The separately ignored, opt-in installed
test also passed for an explicitly selected PatchData copy. It checks the
package hash before extraction and again afterward, verifies unique canonical
scope-zero keys and both decoded payload hashes, then supplies the validated
bytes to the symbolic fixture.

| Input | SHA-256 |
| --- | --- |
| `Data/PatchData.package` | `cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff` |
| decoded globals `A426730B:2A3CE5B7:00000000` | `e8c4b80348fd8f3a1821533c87110315ba584f66f081d46305ff61ab6cbf8c72` |
| decoded Cell `CBCD1287:DFAD9F51:00000000` | `f12d98c0f59af6d4d58174016f1e04d709647480310103341193df9b0fa76542` |

The package index and payload reads reopen the path. The test therefore
requires the selected package to remain stable and not be concurrently
modified or replaced; equal before/after hashes alone cannot exclude a
transient replacement. Payload hashes bind the bytes handed to the fixture.
This is an explicit package fixture, not evidence of the original manager's
mounted winner, native fixup execution, actor creation or gameplay reachability.
No original-game program was executed for this gate.
The same hash-bound test now also passes the original record bytes to the
restricted size/RNG model with explicitly synthetic runtime inputs; see
[cell-start-equal-size-model-v1.md](cell-start-equal-size-model-v1.md).

## Reproduction

From the repository root, the ignored test defaults to
`D:\Games\SPORE Collection`; set `RUSTSPORE_TEST_INSTALL` to a different
install only if it contains the exact hash-bound package snapshot.

```powershell
tools\build.bat test -p spore-bevy cell_start_reference_fixup_v1::tests
tools\build.bat test -p spore-bevy cell_start_reference_fixup_v1::tests::explicit_patchdata_candidate_uses_canonical_keys -- --ignored
```

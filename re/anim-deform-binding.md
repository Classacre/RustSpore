# Native ANIM deformation descriptor preparation

2026-10-23. This is a bounded load-time binding step. It does not close the
generated creature rig, authored curve or mesh deformation milestones.

## Native contract

`[MEASURED]` Original `SporeBin/SporeApp.exe`, SHA256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`,
selects the deformation layout through `0x9FC3E0` and rewrites descriptor metadata
through `0x99D010`. Retained REA/Ghidra queries are in ignored
`target/rea/20261022-anim-rig-focused-v1/`:

| Query | Evidence ID | Established use |
| --- | --- | --- |
| 34, `0x9FC3E0` | `ev_aaaff3b93d7aa54514372587f29ce5762bdd53e77233b30b4d69c64f1183a6e1` | Capability/root/alternate layout lookup |
| 35, `0x99D010` | `ev_ebc40d26197544c28e8935bf950b72adff67410b8d96cf2983998d8512b5ae1a` | Persistent template and descriptor flags/slots |
| 40, data `0x14477D0` | `ev_c64a65e08804625bc7f34e519574b119b63d5243d731198f20f745bd058b3e40` | Nine-slot template types and initial IDs |
| Native v3 query 280, `0x9A9730` | `ev_61442c234e879a027710982ddf3ab43f61bed6224adebd9426c1fa71c100cfce` | Initial empty fallback allocation at object +0x24 |
| Native v3 query 291, `0xA0F20F` | `ev_d72837b96f2162df85b233a1362dd33342e96c6c367430bd1ec05a78c0fefc6e` | Direct call to the initializer at the SDK-bounded callsite |

`[MEASURED]` Lookup first masks the primary selector flags with `100003`.
Mode zero uses the primary capability; mode one uses the `root` fourcc. Other
values immediately select the fallback. For the first two modes, masks
`flags & 801C00` equal to `1800`, `1C00` or `800400` replace that capability
with the alternate. A zero or missing key also selects the fallback.
Channel labels do not participate in this lookup.

`[MEASURED]` The loader initializes one template per clip, with INFO/POS/ROT
in slots 0/1/2 and six deformation slots 3..8. It replaces only the current
layout's deformation IDs when entering each channel. Later slots retain their
previous IDs. Each descriptor selects the first template entry having the same
low four type bits, a nonzero template ID, and either the same ID or a descriptor
ID of `FFFFFFFF`. Unmatched descriptors receive slot `FFFFFFFF` and lose enable
bit `10`. A match alone does not set the enable bit.

`[MEASURED]` For a matching type-three component, bits 0 and 1 of the current
PCTP entry's final parameter word replace descriptor bits `80` and `40`,
respectively. Other flags, offsets, strides, tags and reserved words stay intact.
The downstream numerical meanings of those two bits remain unestablished.

`[MEASURED]` Native initializer `0x9A9730` stores a zero-length descriptor vector
at its object +0x24. Bounded Microsoft SDK disassembly of the original executable
at `0xA0F204` shows the caller passing object `166D038` before its call at
`0xA0F20F`; therefore the initialized pointer is exactly `166D05C`. The initial
fallback is empty. The SDK output remains in ignored
`target/animation/anim-globals-init-alt-sdk.asm`, with cleanup's same-object
call in `anim-globals-init-call-sdk.asm`. These callsite observations have SDK
provenance, separate from Ghidra's initializer Evidence.

## Implementation and corpus boundary

`[MEASURED]` `spore_core::anim_deform::prepare` returns prepared descriptors
without editing the parser's source data. `prepare_native_initial` supplies
the measured empty initial fallback; `prepare` accepts an explicit replacement.
The implementation preserves the once-per-clip
template lifetime. It rejects duplicate capability layouts, more than six
deformation entries, and a stale template match that would read past the current
layout's parameter array. `PreparedChannel::deform_for_component` exposes the
exact selected PCTP entry; it does not treat its slot as a runtime block or bone.

`[MEASURED]` The independent Python reader completed all 5,022 ANIM resources
in the four animation-bearing packages, using the explicit PatchData dictionary
and measured native initial empty fallback. Its existing wide-index reader excludes 71 compact package
indexes. A separate Rust verifier opens the full package population; Python's
coverage must not be described as successful opening of all 82 indexes.

| Initial-native-context quantity | Python result |
| --- | ---: |
| Resources / channels | 5,022 / 48,700 |
| Component descriptors | 301,684 |
| Matched INFO/POS/ROT descriptors | 128,480 |
| Matched deformation descriptors | 43,772 |
| Unmatched descriptors | 129,432 |
| Rewritten flags / slots | 34,950 / 172,576 |
| Enabled prepared descriptors | 108,293 |
| Channels selecting the initial fallback | 2,318 |
| Stale parameter reads / other failures | 0 / 0 |

The source dictionary SHA256 is
`c83c0db7dbf029bf0905388476ca2ecb800b8786e0434a0814e84db0f62722c5`.
`[MEASURED]` The Rust verifier opened all 82 package indexes and prepared all
5,022 resources without an error. Its counts and all 5,022 normalized resource
digests exactly match the independent Python reader. Five focused Rust tests
pass, covering native selector masks, fallback input, low-type/tag/wildcard
matching, flag replacement, preserved source descriptors, stale slots and
bounded/ambiguous layouts. The normalized digest file SHA256 is
`03f1433d51634ba1be3a253c04509ce7918d7e69e93b6407f7a8654bc37c619c`.
This is independent implementation agreement work, not a comparison with a
recorded original-game animation runtime.

Reproduce from the workspace, keeping all output under ignored `target/`:

```powershell
tools\build.bat test -p spore-core anim_deform::tests
tools\build.bat run -p spore-bevy --example anim_deform_verify -- --native-initial --digest target/animation/anim-deform-rust-v1.txt "D:/Games/SPORE Collection"
python tools/anim_deform_probe.py --native-initial --out target/animation/anim-deform-compared-native-initial-v1.json --digest-out target/animation/anim-deform-python-native-initial-v1.txt --rust-digest target/animation/anim-deform-rust-v1.txt
```

`[UNKNOWN]` Original curve evaluation, later runtime context, generated
body-block order, palette identities, per-part retargeting and complete source
mesh deformation remain open. This API supplies a measured descriptor binding
input to that work; it does not claim faithful playback.

# Candidate kind-0 palette: stored initialization and upload boundary

This is a pinned source-data audit and conditional static trace, not a live
actor, an observed pose, a fixed-up pointer, a native device call or proof
that candidate `FD24574B` is the actual player. The actor and provider gaps
are described in [the player ownership trace](cell-animation-owner-palette-draw-static-v1.md).

The explicit `Spore_Graphics.package` resource `DAE8CE17:2F4E681B:00000000`
has SHA-256 `bbd99a741d41a3606a310f242e14e5f85dc7b49af454972ff02124b16e318592`.
Its section 10 (`7000C`, SkinsInK) is 20 bytes, SHA-256
`f817cbee6d55b2f3c1e873f4eb896017ce34d4e249d85648dcaac2fa0cfb70fa`.
Its last three little-endian words are section indices `9,0,8`, referring to
section 9 (`7000F`, SkinMatrixBuffer), section 0 (`70002`, Skeleton), and
section 8 (`70003`, AnimationSkin). Their section SHA-256 hashes are,
respectively, `d67e79fcc84ddd7ee7bf9cb49ac5b96a3d03de32fd3b4707517095f27b19ccbb`,
`ff063ca1b090b2657e3abfd25290092f0e2bd56c0b6148fb4aa7aab04e1c3a67`, and
`183579054af266398c9aa8f1f7f9d17ebbf453e0df0548ed4a59109ecf1a7e3d`.
The first two binding words are retained without interpreting their runtime
pointer meaning. The schema and reference-type checks are in
[`rw4-animation.md`](rw4-animation.md).

Section 9's header stores count 9 and a pointer to its 432-byte matrix payload.
The payload SHA-256 is
`a5d772a0ebe904854a7b443fbf00884f2766ed037c2bd6a52b584ea0ca8a4ad0`.
It contains nine identical matrices, each twelve binary32 words in the order
`[1,0,0,0; 0,1,0,0; 0,0,1,0]`. This is source-stored initialization; identity
alone does not prove the runtime matrix convention or that the buffer remains
unchanged. The corresponding 454 stored vertices have active UBYTE4 register
offsets `0,3,...,24`; the first three UBYTE4N weight bytes sum to 255 and the
fourth is zero. Nine consecutive three-row matrices cover those offsets in the
explicit generated-shader fixture. A read-only, unique-key audit checked package
SHA-256 `e01bb86d8ddf02c56d52394b4e1db7181c1e1ebe1878bf6fb72c4ee523fcedd1`
at both endpoints and preserved receipt
`target/source-cell/kind0-palette-init-audit-v1/receipt.json`, SHA-256
`b8b53fefea215c869310b9d775bbf0079357928d6993db9e8ca3d5e3f70e82cd`.
Trusted stable paths were assumed; endpoint hashes do not exclude transient
replacement. The first 9 identity matrices used by the earlier 64-matrix test
are value-equal to the stored bytes, but its construction was synthetic and is
not evidence that the stored section was uploaded.

The conditional source route is narrower than a direct section-to-shader join.
As documented in [the binding lookup trace](cell-rw4-binding-lookup-followup-v1.md),
`76F820` looks for type `7000C`, while `7755F0` and `773DE0` conditionally map
and evaluate bindings; candidate ownership and exact runtime objects remain
unresolved. Separately, a conditional `73EDF0` draw path prepares state 4 from
a selected palette and descriptor; `777990` publishes the state pointer, and
`777B60`'s mode-zero branch derives its upload arguments. Under the explicit
device-interface and lifetime admissions in
[the upload trace](cell-animation-palette-device-upload-static-v1.md), the
receiver slot is `IDirect3DDevice9::SetVertexShaderConstantF`. Nothing here
establishes that this kind-0 RW4 follows the GMDL-specific path, that the stored
buffer is selected, or that the call is made successfully at draw time.

The saved state-4 record starts with words `1,8,0` and two further stored
words; runtime pointer fixup and later writers remain unresolved. If a caller
explicitly admits mode zero, uses those first three words as the active state,
provides the nine source identity matrices as stable storage, and supplies
fixture register base 2, the decoded arithmetic requests 24 vector rows
starting at register 5 from byte offset 48. That spans matrices 1–8 and leaves
registers 2–4, corresponding to matrix 0 in the test shader, unaccounted for.
Those admissions are not observed runtime values; filling matrix 0 from an
assumed palette would conceal a real boundary.

The smallest independent native comparison should host only pinned original
instructions `777B60..777BA7` on the admitted non-null, mode-zero branch, with
guarded stable storage, a synthetic device vtable callback recording register,
pointer, row count and copied bytes, and no live game or GPU. Compare the
source-stored nine matrices using `(start,count,base)=(1,8,2)` against the
checked arithmetic model, plus `(0,9,2)` as an explicit control; reject short
or mutated buffers before entering the unchecked native slice. Record the
unwritten first three registers separately, retain raw callback bytes and failure
receipts, and pin executable slice, host, source data and model. A successful
comparison would validate the selected upload arithmetic and byte transfer only;
it would not validate RW4 fixup, actor/palette selection, mutation ordering,
device lifetime or live animation.

The first reviewed x86 host exited with code 11 because its requested fixed
code allocation at `00770000` was unavailable. It did not enter the source
instructions. The runner preserved failure receipt
`target/source-cell/kind0-state4-host-result-v1/receipt.json` SHA-256
`897028cd11064330e1307860a5c40056088e3fd70e5b388d075c07a07af6b29c`.
A relocated-code host requires a separate review: the internal relative
branches must remain inside each copied slice and the unadmitted upload exits
must terminate at traps. Fixed global mappings, callback admission and all
other memory checks remain explicit.

The separately reviewed relocated v2 host matched both guarded cases against
the independent model and exited 0. It executed the pinned original setter and
mode-zero upload slices with the nine source-stored identity matrices, a
caller-supplied pointer and a synthetic receiver. For `(start,count,base)`
`(1,8,2)`, the callback received first register 5, 24 rows and byte offset 48;
the `(0,9,2)` control received first register 2, 27 rows and byte offset 0.
Receipt `target/source-cell/kind0-state4-host-result-v2/receipt.json` SHA-256
`3a711befe94f0ac32587e10b9db3cbaf7c07814ccecc52a6631a1254ce7ff222`.
This validates only those guarded instructions and callback bytes; in the
source-shaped first case registers c2–c4 remain outside the upload. Actual
caller base, native pointer fixup, source palette selection and live state
are not established. The failed fixed-address v1 receipt remains preserved.

## Caller-base lead

A bounded Capstone decode of the same pinned executable gives a conditional
route to the consumer's first argument. `77E573..77E589` supplies callback
`777B60` to registrar `11F4020` for state ID 4. The registrar stores its fifth
argument in the 32-byte callback table at `016FB604 + 32 * ID`; this makes
slot 4 `016FB684`. A separate initialization sequence at `11F445F` also writes
`11FA120` to that slot, so initialization order and later mutation cannot be
assumed. Both consumers have a mode-zero upload path, but registration alone
does not establish the active callback at a draw.

On the gated dispatcher path `6F9C50..6F9CE3`, the object read from
`0161A758` supplies an entry count at +28, descriptor pointers starting at
+2C and four-byte values starting at +AC. For an admitted entry, the code
reads the descriptor's word at +10 as the callback table ID and word at +14
as another callback argument; it pushes the corresponding +AC array value as
the first callback argument at `6F9CC2..6F9CD1`. If that entry identifies ID
4, the callback slot still contains `777B60`, and all dispatch gates pass,
the +AC value is the consumer's base-register input. The separate dispatch
at `6E4EE0..6E4EF7` takes that first argument from a word in its supplied
12-byte record. These are conditional dataflow observations, not a value
observed for the candidate.

The controlled generated-shader fixture assigned its `SkinBones` declaration
base `c2` after `materialColor` and `skinWeights`; its retained log records
ID 4, start 2 and 192 rows. This makes base 2 appropriate to that selected
fixture recipe, not proof that the live dispatcher entry, program, callback
slot or register array is the same. A next guarded comparison could isolate
`11F4020` and the bounded `6F9C50` admitted-entry dispatch with explicitly
constructed callback table, descriptor and register arrays; record the actual
callback arguments for multiple base values before joining any selected
program. It must separately model the competing table writer and reject
unsupported gates and mutable snapshots. No such native dispatch comparison or
live c2–c4 producer has yet been observed.

## Distinct fresh direct-draw object

The direct-draw state object at `0162F870` is distinct from the saved RW4
state-4 record beginning `(1,8,0)`. In the pinned PE it lies at offset
`122870` in `.data` (section base `0150D000`, virtual size `212764`, raw
size `0C4C00`), wholly in the zero-filled virtual tail. Fresh image loading
therefore supplies zero bytes there before runtime writes. A bounded scan for
the five exact dword addresses found code references to base `0162F870` at
`73EF24`, +4 at `73EF12/73EF1D`, and +10 at `73EEEA/73EF05`; it found no
exact references to +8 or +0C. This scan cannot see adjusted pointers, bulk
writes, indirect writes or dynamic mutation.

On the conditional direct-draw path, `73EDF0` writes the selected count at
+4 and palette pointer at +10, and passes the object's address to `777990`.
If fresh zero initialization has not been changed by any such other writer,
its start and mode remain zero. Under that explicit admission, a count-nine
source palette and fixture base c2 would cover matrices 0–8, unlike the
separate saved record's conditional `(1,8,0)` example. No evidence yet binds
this candidate RW4 to that GMDL-oriented draw path, establishes count nine at
the call, or proves the object remains fresh. Zero-filled initialization must
not be presented as a current runtime value.
The exact decoded bytes and instruction ranges are retained in
`target/source-cell/kind0-state4-dispatch-static-v1.json`, SHA-256
`117fcfb265c27e2c7db59effb49c0d0cedb066ed086361bd8c1e539c7aa196c7`.

The generation-path register cursor has a further conditional static route.
`6FAE60..6FAEA5` initializes stack slot +3C to zero; at `6FAF4B..6FAF5B`
its address is passed to `6FAD00`. For an accepted, not-already-present
descriptor and available vector capacity, `6FAD98..6FADF2` appends its
descriptor pointer and the current cursor to parallel global vectors. After a
separate formatting call, `6FAE18..6FAE1F` reads the descriptor word at +14
and adds it to that cursor. At `6FB33D..6FB360`, the latter vector is copied
into the selected object's +AC array. Allocation, duplicate-detection,
fragment selection, formatting, vector lengths and later mutation are separate
admissions. The bounded instruction bytes are retained in
`target/source-cell/kind0-register-cursor-static-v1.json` SHA-256
`68f7bf7ab9529c8c5db286e992488a314c81aafbf060e71d1dade456dc81bf3a`.

An independent Python extraction of the pinned PatchData profile-3 fragment
library, using the earlier explicitly admitted recipe `17,19,15,03,10,31,01`
and declaration-string deduplication, yields ID/count/start tuples
`(22,1,0), (3,1,1), (4,192,2), (8,3,194), (6,4,197)` in source order.
Receipt `target/source-cell/kind0-uniform-cursor-fixture-v1/receipt.json`
SHA-256 `67504494d2dde55fb13a6706805cae87dc12a1129e9faa96078e809098d1fd8a`.
The recipe and absence/presence of other data remain explicit fixture choices;
this extraction does not independently execute the original selector or
deduplication path. A guarded original-instruction comparison of the cursor
increment can test these stored counts and starts, but would not by itself
prove that this recipe, dispatch object or register values were used live.

A separately reviewed x86 host executed the original nine-byte cursor slice
`6FAE18..6FAE20` with those five source-extracted descriptor counts. Its
independent model matched all five increments, including `1 -> 2` for
skinWeights and `2 -> 194` for SkinBones; it exited 0. Receipt
`target/source-cell/kind0-register-cursor-host-result-v1/receipt.json`
SHA-256 `15f479be8d205c99eb65667fc99b5a06ade16bbc50bc10bcf3eec5a4eaa371f3`.
The host explicitly converted big-endian source counts into supplied runtime
descriptor fields and used a guarded trampoline for the original stack
adjustment. It did not execute native fragment selection, descriptor decoding,
deduplication or append; those admissions remain separate.

A separate reviewed x86 host executed original declaration lookup
`6F9CF0..6F9D53` against the five source-extracted declaration strings, using
caller-owned vector construction. Five fresh lookups, five repeated lookups
and one near-miss matched the independent model; exit was 0. Receipt
`target/source-cell/kind0-uniform-dedup-host-result-v1/receipt.json` SHA-256
`a01700cbb5d4abb2f94d5419d24859b7cfc57619cf8dfb30dad3d4970b3ec7bf`.
The original lookup compares declaration strings, not the whole metadata;
the host does not execute the selector, native vector append, allocation or
live dispatch. The source fixture's order is still an explicit admission.

A separately reviewed relocated x86 host then executed original selector
`6FC780..6FC96D` against the 38 source-extracted PatchData profile-3
`generic_skinning` render-type-zero vertex records. It supplied feature mask
`0008C045`, saved data IDs 3, 4 and 210, sampler availability for slots 0/1,
no other data, and explicit `24E` values 0 and 1. Both cases matched the
independent source-record model and emitted `17 19 15 03 10 31 01`; exit was
0. Unsupported conditions were trapped, including the only branch containing
an external call. The original code was relocated with its jump-table operand
changed to a host-owned table; source table targets and other bytes were
checked. Receipt `target/source-cell/kind0-selector-host-result-v1/receipt.json`
SHA-256 `52fb54aa3ca466cded14283254d7c3471b8642e8eae406bb752bf34ce0db42e9`.
The host converted source selector records into supplied runtime layouts; it
did not execute the original decoder. These two explicit contexts do not
establish provider precedence, all possible data, current 24E, final shader
selection, c2–c4 contents or live state.

## Conditional model provider convergence

The explicit PatchData and Spore_Graphics `DAE8CE17:00B1B104:00000000`
PROP payloads differ (SHA-256
`583134ef726cadbefead546947a142d49b54d77a50ba9fcdf4d2ab8ce5480e7b` and
`15ebf08981b6b45ec8ce98d899135126281a30c4ecde4caaacbd496c8e82b9e8`),
but their exact `00F9EFBB` LOD0 keys both name
`DAE8CE17:2F4E681B:00000000`. The retained 82-package census
`target/native-archive-index-census-v1/records.tsv` SHA-256
`3de1933446c61546f8afcf44e22c1010a5405baf353d0d5c9259ea6071f4e7c6`
contains one matching RW4, in `Data/Spore_Graphics.package`, and no matching
`00E6BCE5` GMDL. Read-only audit
`target/source-cell/kind0-model-provider-audit-v1/receipt.json` SHA-256
`6219cb4fe567c40e69ed08d3d5af3bf39619ba8c4cbd54df637e9a074b0e2907`
records both PROP payloads, parsed keys and before/after package/census pins.

The conditional model-world route `74D3B0 -> 74C810 -> 74A7F0 -> 754A90`
tries the same identity as GMDL before RW4, per
[the model-loader trace](creature-model-links-native.md). If the selected PROP
is one of these two, the complete mounted resource set exactly matches the
retained census, its files and ordering are stable, no cache or alternate
provider supplies a GMDL, and both lookups run successfully as described, the
disk census is consistent with falling through to the listed RW4. It does not
establish the actual mounted set, provider/cache behavior, actor choice or draw
binding. Different entire PROP payloads must not be treated as interchangeable
beyond the measured LOD0 reference.

## Conditional RW4 attachment and selector check

The pinned candidate RW4 has twenty serialized section tags, including one
`0002001A` at index 15 and one `00080003` at index 6, but no `000200AF`.
The section audit receipt
`target/source-cell/kind0-rw4-type-audit-v1/receipt.json` SHA-256
`8cfe379ce50d1840fe5489a19b1e5c2ff11ce1b443f2457d7db1d035ecb33f47`
pins the candidate payload and package and records the ordered table. A
reviewed relocated x86 host executed the original `757BC0..757CB3` with an
explicitly supplied collection and synthetic enumeration, initialization and
destruction callbacks. Candidate, index-7 positive and empty cases matched the
independent model, with 20, 8 and 0 visits respectively; native exit was 0.
The private non-switched desktop closed successfully. Receipt
`target/source-cell/kind0-rw4-200af-host-result-v2/receipt.json` SHA-256
`cfc18443f82ce0f9702f4bfaeb0b3038cb14930f5dfbe7192ac19ff5d76dbcaf`;
reviewed runner SHA-256
`aefb8a213df033fd9a2020028f275c107868ab2f241d65c7de8539b0a7bb10b2`.
The host checks callback stack bounds and preserves raw output and failure
evidence. The positive control changes one supplied tag; no original native
collection, resource fixup or provider was exercised.

In the conditional `754850` path, a clear caller flag bit 0 permits the
`757BC0` check; false reaches direct `741A20` attachment. That function
reference-updates object `+C4`, calls `7411F0` with the same resource, then
calls `7403A0` only if the resulting vector endpoints at `+08` and `+0C`
differ. `7411F0` enumerates `resource+18`; for tag `2001A`, it appends the
first word at the enumerated payload pointer to the `+08` vector, asks the
`67D890`-supplied manager through vtable offsets `+40` and `+28` using payload
`+08` and the resource, and appends the returned value to the `+30` vector.
When that vector is nonempty, `7403A0` enumerates again and, for tag `80003`,
constructs a `762020` object from the full enumerated entry plus the resource,
reference-updates owner `+CC`, and clears owner `+D0`. Constructor `762020`
copies 56 entry bytes into its object and retains the supplied resource at
`+44` via its first virtual method.

This establishes a conditional path through both candidate tags, not the
contents returned by the native accessor. A source-bound attachment comparison
must first validate the complete 56-byte enumerated entry and its payload
pointer/fixups, provide bounded manager callbacks and preallocated vectors,
and account for reference ownership and the constructor's resource lifetime.
The selector host only supplied the first tag word and cannot establish these
requirements. Actual provider/cache selection, native collection identity,
actor ownership, material/pose binding and live state remain unobserved.

### Accessor framing and section hooks

`11E26D0` indexes a 24-byte in-memory record from collection `+30` after
checking index against `+24`. The `+14` word is looked up in a runtime registry;
the first word of a found registry record becomes output `+00`, otherwise zero.
The in-memory section-record `+00` becomes output `+04`; helpers `11EB390`
and `11EB580` fill output `+08..+17` and `+18..+37`, respectively. Those
helpers walk backwards across immediately preceding `10030..1003F` rows,
putting each first word and each pair of `+08/+0C` words into indexed slots;
their default empty size/alignment pairs are `(0,1)`. Thus the complete output
is 56 bytes, and the tag is registry-dependent rather than a direct copy of
the serialized type. For the two selected candidate rows, the preceding row
types are `80005` and `2000B`, respectively, so this backwards walk stops
without consuming an auxiliary row, subject to an unchanged loaded table.

The serialized row 6 is `[260,0,6A00,10,11,80003]`, and row 15 is
`[732C,0,C,4,B,2001A]` (hexadecimal words). Their raw contents at those
offsets hash to `6612a1dd8da17946ee5e7bcec741f0b7150891eb67fbfb7acbee397594be0dd5`
and `6a0c3bfa4010140e684133cab4a0b95a905ce59e9a3d617eab6170e1f2d99722`.
The latter consists of words `13, 1, 14` in decimal. These are on-disk bytes,
not live pointers. Under the separately described `11E3A60` relocation branch,
with this contiguous image at nonzero base and old base zero, the two row
first words become `image+260` and `image+732C`, while row `+04` retains the
old offset. Each accessor output `+04` would then expose the corresponding
rebased address, provided no subsequent mutation and the same collection.

Section contents need additional hooks. Registration `11FA030` obtains
registry type `2001A` and assigns hook `11F9FE0`. That hook resolves the first
payload word and each of the following count-selected words through `11E20B0`,
writing the returned pointers back in place; the count itself is retained.
`11E20B0` divides an input index into high 10 and low 22 bits and reads the
first word of the corresponding 24-byte table entry through the supplied
collection's pointer chain (`11E4400`); it does not itself validate indices.
For the serialized count-one row, a correctly constructed table mapping would
therefore turn 13 and 14 into two pointer words, but the actual mapping and
its lifetime still require the native loader/provider. A bounded original
hook-and-lookup comparison can admit a private writable 12-byte source copy,
explicit stable table storage with bounded entries 13/14, and controlled
addresses, including a count-zero control; it must not claim actual provider
identity or concurrent lifetime.

A reviewed x86 fixture subsequently ran original `11F9FE0`, `11E20B0` and
`11E4400` with the exact 12 serialized bytes and a synthetic stable lookup
chain: first index 13 and the count-selected index 14 mapped to two supplied
addresses. A separate count-zero control changed only the count and left the
second index untouched. Both matched, exit 0, with snapshots and canaries
unchanged; private desktop creation and closure succeeded. Receipt
`target/source-cell/kind0-2001a-hook-result-v2/receipt.json` SHA-256
`9f9b522a197f1114b0b2811609228c2d4b82ca7941ad35ce99704e46fea88172`.
The fixture checked all three relocated call destinations and does not execute
registry dispatch or load a real collection. It has no protected guard pages
and its supplied table identities and lifetime are admissions.

Under the additional admission that lookup page zero is the relocated
candidate's own 24-byte section table, entries 13 and 14 project to
`image+7100` and `image+7130` respectively. Their serialized types are
`20009` and `2000B`, and their raw section SHA-256 values are
`5786233aea546230a3f3b07bb6e21cffeba8b783065b75c96d7c77e62dfe6cb5`
and `8eb2b22037454e4a337b535c711dac725dacf8ddf51a92ce6b04e69f7c638d89`.
This is a conditional table join, not evidence that either post-hook object
has valid native contents or that a live manager chose it. `7411F0`'s
manager consumes the resolved second pointer via vtable `+40` and its result
via `+28`; the actual manager, those return values and owner lifetime remain
unobserved.

The separate `80003` registration at `75E46E` installs hook `12054F0`, which
calls `1206EC0`, adjusts four payload words at `+34..+40` relative to the
supplied loader context's `+7C`, invokes `1206F30`, and writes a global pointer
at `+20`. Its complete in-memory state is therefore not established by the
outer table rebasing alone. The hook order, context origin and pointed-to
buffers require separate bounds and lifetime validation before an attachment
host can treat that object as loaded. The `80003` hook has not been executed
in this trace. More specifically, `11E3A60` initially copies the image old-base
word to context `+7C` but overwrites it with `98h` at `11E3BAA` before the
subsequent `11E50A0` call; other context writes may occur before the `80003`
hook. `1206F30` additionally rebases the first word of the object reached by
the already-adjusted `+3C` pointer, using that pointer minus context `+7C`;
it calls `12055B0`, which returns success without accessing arguments. The
precise context `+7C` at this hook and the validity of that nested pointer are
not yet established. A next comparison requires an explicitly admitted
context value, verified in-bounds nested address and writable isolated copy,
not a claim that the native loader has produced that state.

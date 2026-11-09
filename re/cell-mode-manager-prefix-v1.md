# Conditional B1E280 mode-manager admission

E828F0 pushes `01654C01`, obtains the manager from B3D3C0 (which returns the
pointer stored at `0167FACC`) and calls B1E280. Separately, D3BD69..D3BD7D
registers the same ID with the `Game_Creature` string on its conditional
registration path, as noted in `creature-scene-source-binding-gap-v1.md`.
This supports the ID association, not live manager identity or mode completion.

B1E280 substitutes manager +24 when the supplied request is FFFFFFFE and
compares the result with manager +20. Equal values return AL=1 without the
changed-path calls. Otherwise the source obtains a service state byte, invokes
a no-op setter with zero, passes old/new to B1DF30, forwards its return to a
service +28 virtual method, writes manager +28 byte 1, and calls the setter
again. The source writes the saved byte into the low byte of the original
request stack slot; the final setter receives the original upper 24 request
bits combined with that byte. The separately inspected DF5080 implementation is
`ret 4`; the fixture captures the full argument and does not invent effects.
Manager +20 is not written in this function.

After independent review, the complete original B1E280 with guarded scripted
service/selector/virtual stubs matched six equality, change and sentinel
cases; receipt `target/cell-b1e280-manager-result-v1/receipt.json` SHA-256
`aaf249ae8e02b2b3ced78dcbab8da683ca7a86cf368642950872eece6c3c5628`, raw
stdout SHA-256 `49ec8e0d0c6cd038da020ba07d72940aa26d77c43c92e6c069443cf77aab49be`.
The runner SHA-256 is `7b452e0b3f59fef9f6d25eca56b2f4d42cf2991f923ab39bde622a8667de8e65`.
This covers only synthetic stable inputs, controlled callbacks and the source
function return; actual selector behavior, service effects, actor identity and
completed stage transition remain unobserved. Guards are not page guards.

The checked Rust projection requires stable manager/service snapshots and
normal scripted callback returns, and explicitly preserves the request slot's
upper bits in the final setter word. Two focused tests passed. The comparator
matched all six retained native rows and FP controls without native rerun;
`target/cell-b1e280-manager-rust-result-v1/receipt.json` SHA-256
`6c03a07cfe5f91c4122158962dc3bb1590136339f424c85bc4326472bdd1b319`.
Callback call-count words in this comparator describe inert fixture stubs.

## Direct B1DF30 transition names

The old mode is B1DF30's first cdecl argument, the requested mode the second.
For old `01654C06`, requested `01654C01` selects the original literal
`CreatureGameFromLevelEditor` through table B1E1F0. Requested `01654C08`
selects `LoadGame` before the old-mode test. The generic Cell-to-Creature path
instead builds `CreatureGame` plus `FromCellGame` through two string helpers,
then passes the result to service +38 and conditionally frees storage; it is
not covered by the direct-name fixture.

After independent review the guarded source direct-name host matched six
editor and load-game cases, without dereferencing literal pointers in its inert
service stub. `target/cell-b1df30-direct-result-v1/receipt.json` SHA-256
`0a837773ff049dbae0f23cc0676f6aa41b7b35ef07c583c9f5f8e1961437cce4`.
The checked Rust projection matched the six retained native rows and controls;
`target/cell-b1df30-direct-rust-result-v1/receipt.json` SHA-256
`914fc4d859d69fbcaf82f6e634b28c7e7daf7db7cc939898d6c2cd7bcb54f02a`.
The service callback and live transition effects remain pending.

After separate independent review, the complete original B1DF30 source matched
one controlled Cell→Creature generic path with five relocated jump tables.
The two string helpers scripted descriptor writes into a bounded synthetic
buffer, the service callback returned an opaque token, and the conditional
free was trapped; original helper, allocator and service effects were not run.
The fixture checks source literal inputs `CreatureGame` and `FromCellGame`;
its linked stubs were independently inspected for the combined bytes. Receipt
`target/cell-b1df30-generic-result-v1/receipt.json` SHA-256
`d94e01eb6ac96d5fe685100aeac0b05a9166d92d14300c8a398b4a441a6fbdf8`, raw
stdout SHA-256 `695257fff225d0bfa6a308ed26d2a6b95e483ce3c9f83081abe8bf1aac23cff6`.
The result does not validate actual native string construction or mode change.

The separately checked generic projection requires the fixed Cell→Creature
inputs, a nonzero synthetic buffer token, capacity at least 25 and explicit
successful scripted descriptor/helper and service-return admissions. It
identifies source literal arguments and pending service/cleanup, without
executing either. One focused test passed; its comparator matched the retained
scripted source row and FP controls without rerun. Receipt
`target/cell-b1df30-generic-rust-result-v1/receipt.json` SHA-256
`5bdb2b655435caaebdaffa1f99e0382929504ed4983fb43508ef9c140a3562fb`.

## Candidate service lookup and action gate

The global getter 67D840 returns `[015FE8B4]`; 67DA50 stores a caller-supplied
pointer there. Conditional startup path 7E6CCC obtains 7E54F0's allocated
0x174 object, constructed by 7E4F50, and publishes it via that setter after a
virtual call. The constructor installs vtable 01414150: its +38 slot is
7E2D40, which copies the supplied name to object+164, performs a lookup through
the object+F8 container and returns node+10 or FFFFFFFF at the sentinel. The
setter can accept another object, so this association is conditional and does
not prove live singleton identity, ownership or map contents.

The same candidate vtable's +28 slot is 7E3850. Its source compares current
service+88 with the supplied token and skips the changed body on equality.
Otherwise, conditional notifications and the old/old-new/new lookups can
invoke callbacks; source may append the previous token to history and writes
new token to +88 near the end. 7E2450 searches +B8 and 7E2DB0 searches a
pair in +D8, using 0x50-stride records and further callback dispatch. These
paths require live container and lifetime evidence before any completion claim.

The reviewed bounded source gate host matched six synthetic equal/different
cases, trapping before both continuations and omitting original SEH setup.
`target/cell-service-action-gate-result-v1/receipt.json` SHA-256
`d695036b8b2ab69fb3434ac5013b14c7bf0ff7ec8e41886917920d9d7697e07c`, stdout
SHA-256 `3e48cfe5aeba565bc39093bd5b2403c27236df21ddbfe4d6140efa09ce12d81f`.
The host does not execute callbacks or writes, and does not establish a live
service identity or transition outcome.


The reviewed bounded 7E3934..7E3991 host matched three scripted cases:
missing/found name-map node and found after the first callback changes +88.
The original re-reads +88 for the pair callback and writes +8C=FFFFFFFF before
that call. The guarded map, old/pair/found callbacks and their effects are
scripted; notification, history and final +88 assignment are outside the cut.
Native receipt `target/cell-service-changed-prefix-result-v2/receipt.json`
SHA-256 `6f5a372d1c20df0635e4b308833adb54b2dc793de4814956a9d8429b2301c9b7`.
The peer-reviewed checked Rust model exposes the +8C value before the pair
callback; its retained comparison admits the inert later stubs preserve it.
Entry inequality is an extra fixture restriction because earlier calls could
have changed +88 after the equality gate. One focused test and all three
retained rows matched; Rust receipt
`target/cell-service-changed-prefix-rust-result-v1/receipt.json` SHA-256
`1024ca1a66c49d2d53788a4353b01a0593452c44d925e548d6b2b3f409d0dd09`.

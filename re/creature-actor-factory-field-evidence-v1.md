# Named Creature actors: bounded factory and field-store evidence

Source: immutable `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
All addresses below are image virtual addresses. REA dossiers are under
`target/rea/20261028-source-runtime-v6/` and remain local ignored evidence.

The native registry function `B245D0` (q119) associates `cCreatureAnimal` ID
`018EB45E` with factory pointer `B1EC40`, and `cCreatureCitizen` ID `018EB4B7`
with `B1EC70`. Read-only direct decoding of the original PE `.text` gives:

```text
B1EC48 push 0145ECB8    B1EC4D push 000016A0
B1EC52 call 00F472B0    B1EC57 add esp,18
B1EC5A test eax,eax     B1EC5C je B1EC65
B1EC5E mov ecx,eax      B1EC60 jmp 00C06870
B1EC65 xor eax,eax      B1EC67 ret

B1EC78 push 0145ECD4    B1EC7D push 00001038
B1EC82 call 00F472B0    B1EC87 add esp,18
B1EC8A test eax,eax     B1EC8C je B1EC95
B1EC8E mov ecx,eax      B1EC90 jmp 00C26FA0
B1EC95 xor eax,eax      B1EC97 ret
```

Each factory also pushes four zero arguments immediately before the shown
string argument. REA q309/q310 identify those strings as
`Simulator/cCreatureAnimal` and `Simulator/cCreatureCitizen`. The attempted
REA procedure analyses q307/q308 did not define those factory labels as
functions; the instructions above are direct PE disassembly, not REA function
dossiers. The null allocation branch does not call a constructor.

Constructors `C06870` (q269) and `C26FA0` (q270) call shared constructor
`C1F940` (q80), then install their respective primary vtables `146AB98` and
`146B818`. That base constructor writes the result of its `Combat` /
`StandardCreatureHealth` lookup, converted from x87 to binary32, to receiver
`+5D4` and `+5E0`. This supports static construction of the two named actor
classes, not a proof that a live game instance exists or that either offset is
current or maximum health.

The animal table at byte slot `+78` contains `C222B0` (q276); the citizen table
contains `C24890` (q291), which directly calls `C222B0`. The common method
uses an embedded receiver at whole-object `+5A8`, then calls `CBCAA0` (q292)
to store a computed word at subobject `+38` (whole-object `+5E0`). The animal
table slot `+8C` contains `C080F0` (q277), which writes positive-zero bits at
`+5E0` during a larger operation. No event name or actual dynamic dispatch is
established. `C03F70` (q311), the animal primary `+7C` producer used indirectly
by `C05C40` (q295), contains multiple branches and runtime dependencies. An
attempted analysis of citizen producer `C24F20` (q312) did not define a
function. These do not justify implementing numeric gameplay behavior.

`creature_actor_field_words_source_v1.rs` models only the observed individual
stores, accepts externally evaluated binary32 words, and excludes inferred event
logic. It is unexported and its inline tests are unrun. The scope is not a
complete actor implementation, an observed runtime trace, or proof of active
resource lookup.

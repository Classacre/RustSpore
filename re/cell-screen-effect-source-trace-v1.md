# Cell candidate screen component: native static trace

This trace concerns the x86 `SporeApp.exe` whose SHA-256 is
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Addresses are specific to that build. It follows static analysis, not observed
execution or rendering. The candidate data and loader qualifications are in
`cell-skybox-effdir-candidate-v1.md`.

The directory reader at `A6C090` checks version 4, reads component sections,
and dispatches each supported version through a function pointer indexed by
component type in a table at `1676E9C` with stride `20` hex. For type 9 the
reader slot is `1676FBC`. Registration at `ABCE70` populates that slot with
`ABCE20` and sets the supported version range to 1–1. The directory reader
skips sections whose handler or version range does not match. Evidence:
`ev_6396d6c31aa3bfeee1b9a16039f8b37173f16838465ac3080564ffceb7c29934`,
`ev_478d33b111bf923634eea018f7934d7f44a96bacead1317eb4249ca02f2cfa1e`,
`ev_5b378f6aa92bc2a182f236298d18247549752eb242ceb053589fd481bab31342`.

Bounded linear decoding from the registered entry `ABCE20` shows an allocation
of `E8` hex bytes, initialization by `ABBB60`, and a read through `ABC380`.
REA confirms the constructor and reader: the reader puts the first byte at
component offset `08`, a masked flags word at `0C`, and a two-word ID at `60`.
Those positions correspond to the independently audited mode, flags, and
texture ID fields. The candidate's values are 5, 1, and `00000000:BD08BD8A`;
this correspondence does not prove selection of the candidate at runtime.
Evidence: `ev_558606088e891a2c2b66dddc6e005f7fe0ee1afddaa99afae5769023487c8d95`,
`ev_1f1a77786e555aeff55c45c4b8be440d946c4f581f645b36ebd8699492e8c24b`.

The adjacent registered factory entry `ABCDF0`, bounded-decoded from its
entry, allocates `4C` hex bytes and calls `ABCC20`. The constructor stores its
argument at runtime-object offset `0C` and installs vtable `145A34C`.
Evidence: `ev_883f94d086beb8d37366b33562aad97bf592baac546d30a87894d65568963610`.
REA analysis of vtable method `ABC5A0` shows it assembling a descriptor
from the pointed-to component's byte `08`, word `68`, ID words `60` and `64`,
and a pointer to `6C`, then passing it to an indirect method at vtable offset
`64` of the object held at runtime-object offset `14` (evidence
`ev_8660ae594e15551d708623c12d7c2ca893bde118f48d34afb2b1e5f159ee6fbb`).
Bounded decoding of `ABC500` shows it repeats that
descriptor path under a separate state transition. The analyzed vtable method
`ABC710` interpolates component values and calls indirect methods at offsets
`68` and `6C` on the same object. Evidence for the latter:
`ev_92edcd2d4280dd08b414dc0dae0c1e27d8b05cd4a450f3bec91019f2fbfcce09`.
REA analysis of `ABC470` shows the object at `14` comes from an indirect call
at offset `20` on a supplied object; its concrete target is unresolved. Evidence:
`ev_143fe245f8c9936a8b16d6f707f69fba0ece40456f1ff38086f63b9159f0c18d`.

Two generic factory paths identify that supplied object more precisely. At
`A71480`, the factory is selected from `1676E98 + type * 20` hex, after an
indirect call at offset `70` on the object at context offset `14` supplies a
factory argument. For a non-null factory result, instructions `A714F3–A714FF`
pass the context pointer itself as the first argument to the result's vtable
slot `0`, followed by the context's `14` field and a fourth caller argument.
For type 9 this slot is `ABC470`. The parallel path at `A70F60`, instructions
`A710C6–A710D4`, passes the same context and context field to the runtime
object's slot `0`. Evidence:
`ev_e184519fe7926a7c2e4136043d7a9bf99ebe58f909e1394169c22dbca1ed18fd`,
`ev_96b84dc6c5c82cc06dff17bca78e39aa9a9067530895d5d8cfb0d9c6a4000ac9`.
`A71990` calls the latter path using the same context; `A71FA0` and `A71FF0`
reach it through `A71F20`. The manager iterations at `A6D088` and `A6E1D5`
pass the value at list-node offset `4` to these methods. This is a static
path through manager-held objects, not proof that the candidate is active.
Evidence: `ev_a633f53762786e3ac31ac53944e0417e26cc80fdc1df0c26765f332906c5ab0e`,
`ev_365c60c51166f8d147444c6893458d037cf0846266eacb284d730431285d8abe`,
`ev_0ca309022960d5d69791caa742ca63091926750ac4b05737349a1f47936b5031`.
One concrete context implementation is now identified. Constructor `A705C0`
installs primary vtable `1456AC0` at offset `0` and initializes context offset
`1C` to zero. Its vtable includes `A72040`, which can reach `A71FA0` and then
the factory path while retaining the context pointer. The primary vtable slot
at offset `20` points to `5B4BD0`, whose entire decoded body returns the value
at context offset `1C`. The adjacent slot at offset `1C` points to `A70D30`,
which replaces that field and makes indirect calls on the old and new values.
Thus, for this implementation, `ABC470` stores the context's `1C` field as its
downstream receiver; the context's distinct `14` field is not that getter.
Evidence: `ev_f0db3883c73c12b9b8aa71d4c4fe7b7b73ae0e021d12153d0016eb124a6f6e05`,
`ev_9571cd2fba97ea33ffea0ebcf5ebdd549b7d9a60c0175e504b1e5ab6bcb9f104`,
`ev_956070aac7af2472974f92c3891322a8812a684cacd2270c1462a85e7831d3c3`,
`ev_bb0d93d438907cea5d43e96244d7a356405bd3e23b9d260ede00a789291aeb83`.
Manager paths `A6DCE0` and `A6DFB0` allocate this context and call its
virtual slot `1C` with the manager's field at offset `40`, thereby supplying
that field as the receiver for these constructions. Evidence:
`ev_a1643af9a80d3c2d7c9f2f33dcfd81934b38eb261b83809b354307b62149915d`,
`ev_ec4ad76b329331f74865598ec1ed3da374de2e24a2dda3457fd06c0b13d09669`.
Manager method `A6DF50` updates offset `40`; when an existing context is
present it also passes the new value to that context's slot `1C`. Bounded PE
decoding places this method in slot `18` of manager vtable `1456688`, installed
by `A6E220`. Evidence:
`ev_b3130724faff470c77529e197972938fc45482bfe4528595c4b8e93b64577fca`.
One startup path supplies a concrete receiver. Bounded decoding at `7A7B0E`
allocates `388` hex bytes, calls constructor `6EDBE0`, stores its returned
pointer at owner offset `38`, and publishes that pointer through `67DB40` to
global `15FE8F4`. At `7EAABA–7EAAF1`, `67D920` reads that global and the code
passes it to the manager's slot `18`. Constructor `6EDBE0` installs primary
vtable `140B188`, whose slot `64` points to `6E7BB0`. These instructions
identify a concrete static startup chain, not the value of the global in a
particular run. Evidence:
`ev_0acd0b5a12970dbd52a5bb561464fa972fbdc247e14112b5e50d8541249827b1`,
`ev_f2c988483fb2c1272c1c902346721bd32c60c554f0999754c6d0a72fd0f89d54`.

For a descriptor mode other than `0A`, `6E7BB0` first calls `777270` if its
receiver's field `360` is nonzero, then calls `777450` through field `364` if
the first path is absent or returns false. Both receive the mode, the two ID
words from descriptor offsets `08` and `0C`, and an output pointer. For mode
5 and a non-special ID, `777270` invokes `776FE0` with those words; on success
it records them and returns true. The fallback `777450` also invokes
`776FE0` for non-special IDs. This is a conditional call order, not proof of
which provider is initialized or of a successful lookup. Evidence:
`ev_3f2faf9953d911f6db73ae04ee1a9b8e3de7d773f28c95d8227fc43cee0e54af`,
`ev_b5d222a8902cc0d787909dcc59afe5a085246cfe0964604f51763d352aa03937`.
`776FE0` obtains a separate global object via `67D880` and calls its virtual
slot `20` with the ID words and zero; a non-null result is required for this
lookup to report success. Evidence:
`ev_274108cc3c89411de1ebd16143047209aa3036a7f453f9e05ea4ab1a35a90f91`.
Bounded decoding traces the global to `15FE8C4`, assigned in startup from the
object returned by factory `7ABF90`. That factory allocates `148` hex bytes
and calls `7ABD60`, which installs vtable `140FAB8`. Its slot `20` is `7A8E50`;
that method constructs three words `[ID word 1, 2F4E681C, ID word 2]` and
passes them with the third argument to its own slot `1C` (`7AC730`). The
screen reader calls `93A470` for the eight-byte ID with endian conversion;
`8DD640` reverses the eight bytes. Consequently, the audited candidate's
big-endian `00000000:BD08BD8A` becomes component words `BD08BD8A` at `60`
and `00000000` at `64`, yielding request words
`[BD08BD8A, 2F4E681C, 00000000]` along this path. Evidence:
`ev_e857f89feb79df4d0d9b0e2944f888f3a793863f2c152f90b8ef46db468efeea`,
`ev_6e4f4277b60106644496312731fa8485dfd9b977068aa966dbadce64f9bfbdb3`.
These
are static paths; other implementations and runtime selection remain unresolved.
At `7AC730`, the cache lookup zeroes the type word; on a cache miss with flag
bit `2` clear, an external provider is queried with type `2F4E681C`. Its two
load branches `7AC3D0` and `7AC2D0` also set that type before invoking the
provider. The separately inventoried matching installed record has type
`2F4E681B`, so its instance match does not prove that this request resolves
to it. `8DE8A0` contains a type-keyed alternative-type lookup and iterates
substituted type words when present. Separately, `76A0D0` has an explicit
conversion branch for a requested `2F4E681C` and source key `2F4E681B`;
`765B00` also tests that pair. Startup `7A77E0` constructs the converter with
vtable `140E02C` and registers it with the global resource provider through
slot `44` (`8E1440`). The converter's slot `2C` enumerates eight supported
types, including both `2F4E681B` and `2F4E681C`; slot `30` accepts the
source-`B`, request-`C` pair. This establishes registration and compatibility
on this startup path, but not that the separate alternative-type list contains
`B`, or that the conversion is used for this key.
Evidence: `ev_b9fba3e58bc9c5a45d7bd6561358435d87e0078b7f28ea1bfebca0adb3ca595d`,
`ev_e9f242c6982928f27f65522a0757a925615a69142402d65debbd11283795227c`,
`ev_fba70731dcf0d38940c418235cee1cd84e29154c3f9e588c91154bd6cae0463b`,
`ev_0242f273bf966eb7373e12f4adda41467f792b258e45796a89f2171670fa02fa`,
`ev_e6f294709c1db55b6494fe11fdc56341ecfe11cf7472a417f0789763d7644e52`,
`ev_2ab8559c95fb0065ea6418adf7c263b1ecfc44ef9039f635d7e6f3ce3f1c9afe`,
`ev_05e68cc50152e318444dc90835ec64580b2f806d3263fe171219c3b55d67c60c`,
`ev_f855a2b15a4397461c7a230d3a608813b2693219fb361a561c0d11391379f183`.

The receiver's original class name, full provider lookup, and draw or shader
implementation are not yet identified. The mode
byte's rendering semantics are therefore unknown. No provider precedence,
successful texture load, active Cell selection, or visual result follows from
this static trace. Bounded linear decoding identifies instructions at known
entry points but is not complete control-flow or runtime evidence. REA evidence
is retained under ignored `target/rea/20261028-source-runtime-v6/`; no game
payload or native code is included in this note.

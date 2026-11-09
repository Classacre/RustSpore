# Cell player target acquisition: bounded static bridge

Target: installed `D:\Games\SPORE Collection\SporeBin\SporeApp.exe`,
SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`,
verified in this pass. Addresses are virtual addresses in the base image.
The instructions below were inspected with Microsoft SDK `dumpbin` 14.51.36260.0.
No native code or game process ran. The REA Ghidra provider was unavailable
because `GHIDRA_INSTALL_DIR` was not configured; this is direct static
disassembly, not new REA Evidence. The selected code windows are not a
complete census of all input handlers or writes to these fields.

## Selected-object to player target

`E6F620` reads a word at `[[16B4BEC]+C4]` and calls `B71CC0` with the
pool at `[16B4BE4]+1C`. If resolution succeeds, it copies the resolved object's
three words at `+4C/+50/+54` to the player actor at `+08/+0C/+10` and writes
zero to player byte `+04` (`E6F641..E6F656`). On failure it clears state
`+C4` (`E6F65C..E6F662`). It then calls `E681E0` and passes the result of
`E67A50(actor+FC/+100/+104)` to `E6EF30`: cache `+1258`, literal float 1,
cache `+1254` and literal final zero (`E6F66C..E6F6C2`). The exact source
argument and movement boundaries are documented in `cell-movement.md` and
`cell-capability-cache.md`. These instructions establish a conditional
selected-object target bridge, not how an arbitrary user action chooses a
point or how a live cache is bound.

`E6C8C0` first passes its four arguments to `697540`. That routine sets a
byte indexed by the first argument relative to the object and stores the
second and third arguments in object `+2C/+30`; when object `+34` is zero it
also stores the first, fourth, second and third arguments at `+34/+40/+38/+3C`
respectively (`697540..69757C`). Their event semantics are not proven by
these stores. When the low two bits of the fourth argument are clear,
`E6C8C0` calls `E6C7E0`. It conditionally stores its nonzero result at state
`+C4`: if the value loaded from the `E4CCA0` result at `+D4` equals 2, the
first argument must be `3E8`; if it equals 1, the first argument must be
`3E8` or `3EA` (`E6C944..E6C991`). Otherwise this branch clears `+C4`.
When either low bit is set, execution takes a different branch beginning at
`E6C9C2`; its complete semantics have not been recovered here. Numeric
codes correspond to the source's named mouse-button table as detailed below;
the upstream event dispatch is not yet fully traced.

`E6C7E0` obtains an object through `67D830`, invokes its virtual `+58` entry,
passes the result to `7C42A0`, then calls `E87270` with a 512-entry output
buffer and a value from global `[16B4BE4]+40FC`. For each returned word it
calls `B71D00` on the pool at `[16B4BE4]+1C`, excludes the actor whose first
word equals global `+411C` and actors with byte `+112` nonzero, then tests
`E620D0` or `E6B560`. The first accepted actor's first word is returned;
otherwise it returns zero (`E6C7E0..E6C8B4`). The virtual provider, geometric
query, filters, object lifetime and ordering must be traced before interpreting
this as a particular mouse or selection operation.

## Practical next bridge and remaining gates

The next bounded implementation candidate is a conditional, supplied-context
bridge from an admitted `+C4` handle and a resolved actor's three position
words to the player's three target words and byte `+04`; it should preserve
the failure clear and source operation order. It requires an independently
verified actor-pool handle lookup and a source-backed owner for both state
and player before use in gameplay. Trace `E6C8C0`'s callers and registration,
the virtual `+58` provider, `7C42A0`, `E87270`, both filters and the
alternate modifier branch to identify actual controls and selection semantics.
The existing application maps W/S and A/D to prototype thrust/turn and does
not supply these source actor fields, selected handle or cache; assigning them
from those axes would be an unsupported invention. Original animation state
also needs the live actor, floating-point policy and subsequent movement and
request paths described in `cell-actor-initialization-movement-v1.md`.

## Follow-up: button names and sampled cursor path

The executable's contiguous `.data` name/value pairs identify `3E8` as
`mouse_button_left` at `154CEB0/154CEB4`, `3E9` as `mouse_button_middle` at
`154CEB8/154CEBC`, and `3EA` as `mouse_button_right` at
`154CEC0/154CEC4`. Thus the conditional `+C4` writer above accepts left for
`+D4 == 2`, or left/right for `+D4 == 1`, subject to the other gates. The
source table establishes these names, but this pass did not prove the full
OS-event-to-handler dispatch or characterize the configuration value `+D4`.

The concrete registration path is conditional but explicit: `E61500` installs
primary vtable `1486008` on the newly allocated object and passes it together
with the literal name `Game_Cell` to its caller object's virtual `+20`
(`E6151B..E61547`). Primary slot `+2C` is `E6C8C0`. `B5C797` calls this
registration using the manager returned by `67D830`. This is evidence of a
registered Cell callback and its labeled button codes; reaching the virtual
slot from a particular live OS event still needs the dispatch trace.

The `67D830` manager's virtual `+58` returns its `+30` field under the
constructor/vtable chain documented in `cell-gameplay-camera-static-v1.md`;
the actual selected camera and initialization remain conditional. In
`E6C7E0`, `7C42A0` takes two output buffers and calls `67D870`, then the
returned object's virtual `+1C`. On the `7A4910` initialization path,
`75FA70` allocates the object, `75F840` installs vtable `140DC98`, and the
object is published through `67DA80` to singleton `15FE8C0`, read by
`67D870`. This vtable's `+1C` is `C86FB0`, which returns object `+5C`.
`7C42A0` converts the returned structure's `+08/+0C` integers (object
`+64/+68`) to floats, and passes them and the two output buffers to
`7C40D0` (`7C42A3..7C42D5`). The latter applies viewport-related words,
camera fields and normalization to produce two three-word outputs; this
supports a cursor-position-derived camera query, not proof of a specific
runtime camera, ordering or complete geometric hit behavior.

In the same constructed object, virtual `+38` is `75DE60`. It calls
`8D2610`, whose indirect import at `13CC5E4` is `USER32!GetCursorPos`, and
gets four words from another provider. If the two computed dimensions are
positive, it subtracts the provider origin from the cursor coordinates,
scales by globals `171344C/1713450` using signed integer multiplication and
division, and stores the results at object `+64/+68` (`75DE60..75DED9`).
This ties the cached query coordinates to an OS cursor sample under those
conditions. The scheduling of this virtual update relative to the Cell
callback, the provider rectangle identity and actual runtime vtable have not
been demonstrated. No retail process or input was observed.

## Windows message to queued event and conditional Cell handoff

The message switch at `849706..849730` indexes a byte table at `849AD4`
by message minus `120`, then a target table at `849A98`. The inspected
table entries route `201` to `849759`, `203` to `84977D`, `204` to
`8497E9` and `206` to `84980D`. The first two branches call `848B70`
with respective extra values zero and one; the latter two similarly call
`848BF0`. Those wrappers pass event codes `3E8` and `3EA` respectively,
and byte one, into `848AF0`. The `202` and `205` entries route to
`848C30` and `848CB0`, which send the same respective codes with byte zero.
The switch also forwards two sign-extended 16-bit halves of the message
parameter. This source routing matches the Windows mouse down/up/double-click
message numbers, but no live delivery was observed.

`848AF0` constructs a record with type `5`, code at record `+0C`, the
button byte at `+10`, and sends envelope ID `1EE1001` to `847FE0`.
If the envelope ID matches and the receiving object's `+60` is nonzero,
`847FE0` forwards the record to that object's virtual `+30`; otherwise it
uses the other `+5C` or `+64` paths. Separately, `814C80` handles envelope
ID `1EE1001`, uses type-5 records to update left/middle/right bytes at
`+70/+71/+72`, and, if its `+4C` is nonzero, calls `812BC0` and forwards
the translated record via `+4C` virtual `+34`. For type 5, `812C8C`
outputs type 6 when the button byte is nonzero and 7 when zero, copies the
button code to output `+18`, scales the two coordinate words into floats at
`+0C/+10`, and forms flags at `+14`. The constructor at `813FF0` installs
the secondary vtable `1419348` at object `+214`; its `+04` entry is
`814C80`. At `815F4F..815F78`, the same `+214` interface is registered
with the object returned by `67D7E0` for envelope ID `1EE1001`. The identity
of `847FE0`'s actual receiver and whether that delivery reaches the registered
listener are still conditional, especially when its `+60` override is set.

Under the constructor path at `815C65..815C88`, that `+4C` receiver can be
the object allocated by `958E00`, whose primary table `1441318` supplies
`+34 = 957E50` and `+18 = 95AAB0`. `957E50` forwards the translated record
to `95AAB0`, which copies seven words into a queued record. In the inspected
drain path `95ABFB..95ACCE`, an untargeted record reaches `959890`; its
type dispatch and hit/target path have additional conditions and virtual
calls. `959A98..959AD7` copies the event and invokes its owner's virtual
`+10`, which for vtable `1441318` is `95AA90`, forwarding into `959750`.
`959750` can pass to `958B00` and `958A30`, which first checks an ordered
listener list and, if the relevant supplied gate is clear and no listener
consumes the event, invokes the target object's primary virtual `+20`.

On the root-object construction path `F482DD..F48327`, `813FF0` builds the
object stored at owner `+28`, and `F48326..F48327` supplies the manager
returned by `67D830` at `F481F5` to `813070`, which stores it at root
`+268`. `815CFA` passes the root secondary interface at `+04` to the
constructed queue receiver's virtual `+08`, and `95B190` retains the
corresponding primary object at receiver `+04`. The queue's hit/target path
must actually select that root and pass the listener/consumption gates.
Under those conditions, `958A30` invokes root primary `+20 = 813CB0`, which
calls `950AA0`. Event type 6 selects table entry `950B3B`; that passes code
`+18`, two float coordinates `+0C/+10`, and flags `+14` to primary virtual
`+3C = 813200`. At `813243..813279`, a nonzero root `+268` invokes that
manager's virtual `+2C`, then the returned object's virtual `+2C` with those
four arguments. For manager table `14130C0`, its `+2C = 7D7FE0` returns the
entry selected by index `+28` from its 24-byte registration records. If that
entry is the `Game_Cell` object registered by `E61500`, its virtual `+2C`
is `E6C8C0`. The callback can therefore reach the selected-object path under
these explicitly supplied conditions.

This is a conditional static chain, not observed gameplay. The actual message
receiver, registration/selection timing, hit target, listener interception,
active Game_Cell entry, live camera and actor/cache identities, and frame
ordering remain unverified. `813200` has an early `+28D` gate and fallback
consumer as well. A release is type 7 and takes a different root virtual
slot; it must not be treated as a press. These gaps preclude connecting the
prototype's keyboard axes to source animation or claiming an unconditional
mouse-to-movement implementation.

For an application integration, the minimum next evidence is: identify the
actual `847FE0` receiver and active `Game_Cell` registration; establish which
target and listeners receive a translated event and its frame order; recover
the active camera/viewport, `E87270` query and acceptance predicates with an
admitted actor pool; and bind the resulting handle, player actor and source
capability cache to the original integrator and animation state with an
established floating-point policy. A standalone conditional handle-to-target
copy can be modeled with supplied state before those steps, but would not by
itself make application controls source-backed.

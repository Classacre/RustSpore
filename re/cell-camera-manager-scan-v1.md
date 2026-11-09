# Cell camera manager setter: bounded static search

This is a static byte search of the installed `SporeBin/SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
No game process was run. It extends the conditional manager and vtable evidence
in `cell-gameplay-camera-static-v1.md`; it does not identify the active Cell camera.

`tools/cell_camera_manager_scan_v1.py` checks the executable hash and PE32 header,
then searches the raw `.text` section (VA `00401000`, 16,560,128 raw bytes):

- 421 raw `E8 rel32` byte positions resolve to manager getter `67D830`.
- No raw `E8 rel32` byte position resolves directly to setter `7D8180`.
- For each getter candidate, the following 300-byte window contains no raw
  `FF /2` call or `FF /2`-like register-base encoding with displacement `54`
  among the simple `FF 50..57 54` and `FF 90..97 54 00 00 00` encodings.

Reproduce with:

```powershell
python tools/cell_camera_manager_scan_v1.py 'D:/Games/SPORE Collection/SporeBin/SporeApp.exe'
```

These are raw byte candidates, not a disassembly census: bytes can be embedded
in other instructions, and the window can cross branches or function boundaries.
The pattern excludes other encodings (including SIB and register-loaded call
targets), tail jumps, other entry points, code outside `.text`, and use of a
manager pointer retained beyond the window. This bounded negative search therefore
cannot establish that a setter call is absent. In particular, it does not tell
which resource or class was selected, whether selection happened, or the actor
anchor, live viewport or frame timing.

The next useful static step is a provider-backed data-flow search across callers
of `67D830`: trace the returned pointer through stores, parameters and wrapper
calls and resolve indirect vtable accesses, including calls outside the local
window. Independently trace `7D8180` vtable slot `01413114` references and the
`7C5A70` service-object setter path. Runtime observation of an actual Cell-stage
selection and camera frame remains a separate acceptance gate.

## Follow-up: concrete alias and vtable words

Static executable bytes establish an additional alias route. The branch at
`7E5E77..7E5EA7` compares a name with the null-terminated ASCII literal
`GetAppPluginServices` at `0141439C`. On the equal branch, `7E5EC8` calls
`67D830` and `7E5ECD` writes the returned pointer into the caller-supplied
output structure at `+0C`. Adjacent fields receive other singleton results.
This exposes a manager pointer through that service-table path conditionally;
it does not show that an installed plugin or Cell code consumes the field.
A local getter-followed-by-call search cannot exclude later calls through this
alias, so the required dataflow boundary is wider than direct getter callers.

The file-backed word at `01413114` is `007D8180`, the manager primary vtable
`014130C0` slot `+54`. The service primary vtable slot `014106A0` is `007C5A70`,
which is `01410688 + 18`. Searching literal little-endian words across file-backed
PE sections found the setter address only at that manager vtable slot, and the
service setter address only at its service vtable slot. Literal addresses of
those two slot locations did not occur in file-backed sections. These are raw
literal searches, not exhaustive xrefs or proof that indirect calls do not exist.
Manager constructor `7D8AF0` clears `+30` at `7D8B45`; its setter writes `+30`
at `7D8184` and tail-dispatches through the service's `+18` slot at `7D8190`.

A follow-up decoded the first 150 bytes after each of the 421 raw getter-call
candidates, looking for `mov` instructions mentioning displacement `+54`.
This is a scout, not a CFG or complete dataflow analysis. Representative virtual
`+54` candidates do not establish manager dispatch: `EF0F0C..EF0F13` operates
on the result of `67C5E0`, while `EFD111..EFD116` follows a return and separate
entry `EFD0E0`, not the preceding manager getter. Likewise `FFDB29` is in the
separate entry at `FFDB10`. The getter at `7E5EC8` is the material surviving
alias found by this bounded inspection. The precise remaining barrier is tracking
consumers of the `GetAppPluginServices` output structure, in addition to direct
and stored manager pointers, through their virtual dispatches to identify a
concrete setter argument and its creation/resource provenance. Neither the active
Cell camera key nor its class follows from the observations above.

## Service-table consumer search and installed modules

The code at `7E5E30` receives a name in its first stack argument and compares it
with the literal at `0141439C`. The match path writes the singleton table into
the caller's second stack argument, including manager `+0C` as above. An
independent raw literal search of file-backed PE sections finds the function
address `007E5E30` once, at immediate operand `7E8E5A`: the instruction at
`7E8E59` pushes that callback, followed at `7E8E5E..7E8E60` by an indirect
call on `EDI`. This supports callback handoff, not identification of the
recipient's type or later callback invocation. The same raw search found no
`E8 rel32` candidate directly targeting `7E5E30`. It cannot exclude computed
references, dynamic registration or code outside the file-backed image.

A recursive filesystem inventory of `.exe` and `.dll` under the installed
`D:/Games/SPORE Collection` found the base executable, the EP1 executable,
`steam_api.dll` and `msvcr90.dll` in each bin directory, an EP1 GOG DLL, and
installer/redist executables. A raw ASCII byte search for
`GetAppPluginServices` over those files found it only in the two `SporeApp.exe`
files: the base file above and EP1 SHA-256
`922af7c7908af2614b9caca2ffb1ec2353442022fd124866105724acaa24c26e`.
This inventory did not find a separate installed DLL containing that literal.
It does not rule out dynamically constructed names, embedded modules, alternate
file extensions, external files, or calls using the registered callback without
repeating the literal. EP1 was searched for the name only; its offsets and
behavior were not equated with the base executable.

No consumer of the returned `+0C` field, concrete manager `+54` caller,
setter argument, camera key, or class was established. The next supported
trace is to identify the object and method receiving callback `7E5E30` at
`7E8E60`, follow its registration and invocations, and recover the returned
service table's consumers through function or module dataflow. The module search
above narrows the obvious installed DLL route but does not settle it. Runtime
selection and camera state remain unobserved.

## Callback registration and shared-library dispatch

The receiver can be resolved under the constructor path. Owner construction at
`7E74B0` calls `936560` on owner `+D0`, which writes vtable `0143F630` at that
embedded object. At `7E8E48..7E8E60` the call is made on that same embedded
object's vtable `+04`; file-backed slot `0143F634` contains `009369C0`.
Conditional on the constructed vtable remaining installed, `9369C0` checks
that its pointer array `+08..+0C` is empty. On that branch it allocates an
entry, passes the two supplied words to constructor `9368F0`, and appends the
entry to the array. The constructor calls `936870`, which admits a nonzero
first word and inserts an eight-byte pair in entry array `+1C..+20`; this
registration supplies callback `007E5E30` and companion `0`. An existing
nonempty registry takes a different branch, so static code alone does not
prove this callback was registered in a particular run.

The registered entry's vtable is `0143F658`; slot `+2C` is `936380`. That
wrapper calls `9360B0`. For a supplied entry `9360B0` iterates its `+1C..+20`
eight-byte pairs, calls the first word as a function at `9360F3` with the
second word and other forwarded arguments, and stops at a true result. This
is a concrete conditional invocation path for the registered callback, not
an observation that it was called or that a returned service table was used.

`936250` also shows how the dispatcher reaches a loaded module: it calls the
import `KERNEL32!LoadLibraryW` (`013CC270`), then `GetProcAddress` (`013CC0BC`)
with literal `EASharedLibraryStartupFunction` at `0143F610`. When lookup
succeeds, it invokes that export at `9362D2`, passing `9360B0` and a word
obtained through virtual `+24` on global `0166A928`. Successful callback
registration sets `0166A928` to the embedded registry via `936080` called at
`7E8E67`. The same routine later uses `KERNEL32!FreeLibrary` (`013CC0E4`).
These are conditional static call paths; neither the loaded path nor the export's
behavior, registry entry selected at dispatch, service-table consumer, or
actual call ordering was observed. The earlier installed module literal scan
cannot exclude a DLL that uses the passed dispatcher without containing the
`GetAppPluginServices` string.

The next trace is the source of the library path consumed by `936250` and the
module/export invoked on the Cell route; inspect its call to the supplied
`9360B0` and subsequent table `+0C` use if an installed module can be tied to
that path. No active Cell camera key, class, setter argument, or runtime state
is established by this chain.

## Library path and installed startup exports

Under the `0143F630` registry vtable, slot `+10` is `936AC0`. It compares
caller-supplied UTF-16 path text with each entry's virtual `+34` result; if no
match is found it allocates a `0143F658` entry and forwards the supplied path
and a second argument to `9365C0`. In `9365C0`, a non-null first argument is
passed to `KERNEL32!LoadLibraryW` (`9365E5..9365EC`); on a successful handle,
`GetProcAddress` tests `EASharedLibraryStartupFunction`, then the handle is
freed. On successful validation the supplied UTF-16 path is copied into the
entry at `+0C` (`936610..936620`). The later `936250` load reads this saved
entry field. This establishes the parameter-to-load route, but not a concrete
caller-provided path, or whether a particular DLL passes validation at runtime.

A read-only PE export-directory enumeration of every `.exe` and `.dll` in the
recursive installation inventory listed above found no named export
`EASharedLibraryStartupFunction`. The inspection covered the files physically
present under that installation and made no assumptions about external paths,
other file extensions, or modules added after the inventory. An absent named
export in that bounded set does not prove no shared library can be loaded.
There is consequently no installed export body from this inventory to trace to
the service table, and no observed Cell camera setter or resource key.

One separate in-image registration is visible: `5D6A80..5D6A9C` obtains global
registry `936070`, calls registry virtual `+24` (under the recovered vtable,
`936240`, returning its first entry), and calls that entry's virtual `+24`
(`936870`) to register callback `005CB7C0` with companion zero. The nearby
`5D60B7..5D60D1` calls entry virtual `+28` (`936340`) with that callback.
The callback's name-comparison paths at `5CB7C0` do not establish that it uses
the camera service table; neither static registration sequence proves runtime
execution or a connection to Cell. Other indirect registration and library-path
callers remain unresolved.

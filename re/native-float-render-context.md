# Original floating-point setup and renderer ordering

[MEASURED] This is a bounded static trace of the immutable base executable,
SHA256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
It identifies actual policy writes and ordered call sites. It does not capture
the control state at an animation/Cell entry on every runtime thread, and it
does not change the separately declared arithmetic contexts of the Rust/native
conformance tests.

## Startup and explicit x87 setters

[MEASURED] The earlier [startup trace](cell-movement.md) establishes the original
CRT's PC53 request through PE entry `11E0F90`, startup `11E0CD0`, initializer
`11E0EAF` and helper `11E130A`. That helper calls
`_controlfp_s(NULL,10000,30000)` through thunk `11E14E6`. It precedes later
game/render initialization and does not establish the later frame context.
The retained report is
`target/source-cell/native-default-precision-startup-v2.json`.

[MEASURED] Original helper `689080..6890B4` changes only the x87 precision bits:
it saves CW, clears `0300`, then maps argument zero to PC24, one to PC53 and
two to PC64. Other arguments also leave the cleared PC24 selection. Helper
`6890C0..689105` changes only rounding bits `0C00`: zero nearest, one down,
two up and three toward zero. Other non-three values leave cleared nearest
after its comparisons. Both load the modified word with FLDCW and preserve
unrelated incoming CW fields.

[MEASURED] A source-byte direct-call/reference census finds three call pairs
to these two setters. All three pairs, `7E5B12/7E5B19`, `7E6CBD/7E6CC4` and
`7E9C53/7E9C5A`, pass zero for PC24 and nearest. These direct policies are
stronger than treating the CRT startup request as a universal default.

[MEASURED] Complete initializer `7E64E0..7E6DE1` obtains the renderer at
`7A64B0` and calls its +08 start method. A failed return reports
`Could not start the renderer` and exits. On the successful path, it clears
the two time fields at +B8/+BC and executes `689080(0)` and `6890C0(0)`
at `7E6CBD/7E6CC4`. This is an explicit PC24/nearest request after renderer
startup, independent of assumptions about Direct3D's precision behavior.

[MEASURED] The later pair at `7E9C53/7E9C5A` is inside the complete update
routine `7E9AE0..7EA012`, after clearing a pending array and before updating
time and dispatching subsystem callbacks. It is conditional: the +174/+178
array bounds must differ, and an earlier +16C branch can skip that block.
It is not an unconditional per-frame precision reset. Later virtual subsystem
calls and entry state on other runtime threads remain separate evidence.

## Actual Direct3D9 creation

[MEASURED] The source PE imports `d3d9.dll!Direct3DCreate9` at IAT `13CC790`.
Thunk `120B910` jumps to that IAT. The source byte census finds its one direct
call at `11F826E` inside `11F8260`, passing SDK version `20`. The returned
IDirect3D9 pointer is stored at `16F99C8`.

[MEASURED] Renderer initializer `75E750` calls `11F8260` at `75E795` and sets
its +0C initialized byte only after success. Within `11F8260`'s successful
path, before returning, `11F8397` stores MXCSR, `11F83BC` ORs `8000`, and
`11F83C6` reloads MXCSR. This explicitly sets flush-to-zero (FTZ), preserving
the other incoming MXCSR bits. It does not establish their initial values or
their state after all later external calls.

[MEASURED] Device creator `11F8430..11F866F` uses that exact IDirect3D9
pointer and calls its vtable+40 at `11F84FE`. The seven pushed arguments match
the installed Microsoft SDK's IDirect3D9::CreateDevice declaration, including
the output device pointer `16F99D0`, presentation parameters `16F9B9C` and
the behavior flags in EAX.

[MEASURED] The complete flags decision starts from zero or `04` and ORs one
of `20`, `40`, `50` or `80`, based on the explicit vertex-processing settings
and device capability bits. The possible values are exactly
`20,24,40,44,50,54,80,84`. The installed SDK declares `02` as
D3DCREATE_FPU_PRESERVE, `04` as MULTITHREADED, `20` as software vertex
processing, `40` as hardware vertex processing, `10` as PUREDEVICE and `80`
as mixed vertex processing. No possible source branch includes FPU_PRESERVE.
The SDK header path and SHA256 are retained in the local policy report.

[INFERRED FROM THE DOCUMENTED API CONTRACT] Microsoft specifies single-precision,
nearest rounding for Direct3D initialization on the calling thread when
FPU_PRESERVE is omitted. Combining that contract with the measured flags
identifies an expected PC24/nearest effect of this creation call. It does not
by itself prove the control word at a later actor entry, all worker threads,
or the exact MXCSR after the driver/runtime returns.
[Microsoft D3DCREATE documentation](https://learn.microsoft.com/en-us/windows/win32/direct3d9/d3dcreate),
[Microsoft calling-thread floating-point guidance](https://learn.microsoft.com/en-us/windows/win32/dxtecharts/top-issues-for-windows-titles).

## The surrounding helper restores exception masks only

[MEASURED] Initial device wrapper `75EF50` calls `689140` at `75F237`, calls
`11F8430` at `75F253`, then passes the saved value to `689110` at `75F261`.
This is not a complete control-word save/restore. Query helper `689140` returns
`(~CW) & 003F`, the enabled-exception selection. Restore helper `689110`
computes the equivalent of `(currentCW & FFC0) | ((~selection) & 003F)` and
loads it. The precision and rounding bits currently left by Direct3D survive
this exception-mask restoration.

[MEASURED] The renderer's recreate function `75E820` has a destroy/recreate
branch (`75E9F7 -> 11F7CC0`, `75EA11 -> 11F8430`) and a separate reset branch.
The reset branch brackets `11F7D80` at `75EB0C` with the same exception query
at `75EB00` and exception restore at `75EB12`. Native reset helper `11F7B90`
calls the actual device `16F99D0` vtable+40 at `11F7C3B`. Its later callbacks
remain additional control-flow context; this trace does not assume Reset has
the same documented precision effects as CreateDevice.

## Second explicit SSE policy site

[MEASURED] A low-memory linear executable-section census found only two
STMXCSR/LDMXCSR pairs: `5403C8/5403E2` and `11F8397/11F83C6`. The scan is
candidate discovery, not independent proof of every instruction/function
boundary. The actual bounded instruction paths confirm the renderer pair
above and the second sequence below. The same census contains many scoped
x87 conversion-control changes; those cannot be treated as a single global
precision policy.

[MEASURED] The second sequence computes
`(incomingMXCSR & FFFFFFC0) | 8000 | global15DC058` before LDMXCSR. It clears
the six sticky exception-status bits, enables FTZ, and preserves other bits
unless the OR mask adds a bit. Its complete owning function is
`5402E0..54044A`: after retaining its two explicit resource inputs and storing
the supplied rate/inverse rate at object+14/+18, it sets MXCSR, then calls
`53E7C0`, `53EEE0` and `53E8D0`. It does not save/restore the incoming MXCSR
around this sequence. The path's specific caller/world/thread binding remains
outside this bounded setup trace.

[MEASURED] Global `15DC058` has image value zero. Source CPU probe
`4F87E0..4F8858` first writes zero at `4F87ED`, tests CPUID SSE/SSE2/FXSAVE
capabilities, stores FXSAVE state at `15DC390`, then checks MXCSR_MASK bit
`40` at `15DC3AC`. Only if supported, it writes `40` at `4F8847`. The raw
absolute-reference census finds those two writes and the read at `5403D7`.
Thus the second SSE setup can also enable denormals-are-zero (DAZ), gated by
the CPU's reported support. This does not establish all entry MXCSR bits.

## Conformance scope and evidence

[MEASURED] The strict source-rest intake and body-pose operations in
[anim-rig-native.md](anim-rig-native.md) retain their declared SSE binary32
nearest/gradual-underflow test context. The x87 context is explicit. Their
matching corpus/host results do not automatically become retail FTZ/DAZ
conformance results. No default whole-game CW, MXCSR or retail dispatch is
selected by these static setup observations.

[MEASURED] Local evidence is retained under ignored `target/animation/`:

- `rig-float-render-policy-v1.json`: source/SDK pins, exact regions, call order,
  flags and remaining unknowns.
- `rig-float-render-imports-v1.json` and
  `rig-render-creation-call-candidates-v1.json`: PE import/thunk/call discovery.
- `rig-d3d-device-create-and-following-v1.asm`,
  `rig-d3d-device-reset-native-v1.asm`, `rig-d3d-reset-cleanup-v1.asm` and
  `rig-renderer-create-recreate-callers-v1.asm`: bounded static bodies.
- `rig-x87-state-helpers-689080-v1.asm`: the four exact control helpers.
- `rig-float-environment-instruction-census-v1.json`: linear executable-section
  candidate inventory, with its limitations.
- `rig-direct-float-policy-setter-references-v1.json` and
  `rig-other-mxcsr-global-ref-*.asm`: later source policy leads/producers.

[MEASURED] REA v3 query529 identifies renderer initializer `75E750` with
Evidence `ev_535c770b9e772b6e2f84b025bf17b5d8bfc4f2ff764ff6e8e0a51da2f4dd0b2a`;
query530 identifies initial device wrapper `75EF50` with Evidence
`ev_4e75563a25520529b5d46b2b68d7f5d8dc4f195dcc0335007ab527213dee4c4e`.
Their caller inventories are empty because relevant virtual flow is unresolved;
that is not proof that these methods are unused. No original game process was
launched in this investigation.

| Additional complete native dossier | Evidence ID |
| --- | --- |
| Update `7E9AE0`, query545 | `ev_33792def1b7fe78448b32c2f38b76f3fe858e736c8e701ffa5daa911c0e61bbe` |
| Initializer `7E64E0`, query546 | `ev_4c0fe7a26847c3a405aa39c47bb9353b26d832682b23b36e97cf0949d464720b` |
| SSE setup `5402E0`, query547 | `ev_9fca542cb95f98852c4b9929ac27226733ffde08c37756c519c780dc4c8992f6` |

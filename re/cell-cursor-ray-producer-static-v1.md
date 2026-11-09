# Cell cursor ray producer: static boundary

Static trace of installed `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`;
no original code or live process was run. `cell_game.rs` currently uses an
explicit source asset as controlled prototype artwork while `CellWorld`,
actor positions and Bevy camera are prototype state. It does not supply the
source actor-pool identities, camera object, source geometry/cache state, or
name/provider state to the selection models. No source picking or live actor
selection follows from that art assignment.

`E6C7E0` calls `7C42A0` on the camera pointer returned by manager virtual
`+58`. That routine obtains the cursor provider through `67D870` and virtual
`+1C`, loads its signed dwords +08/+0C, converts each using `CVTSI2SS`, and
passes both to `7C40D0` together with two output pointers. This is a sampled
cursor path, conditional on the specific provider/vtable chain documented in
[cell-player-target-input-static-v1.md](cell-player-target-input-static-v1.md);
it does not prove the currently selected manager camera.

At `7C40D0`, camera +170 points to an object containing unsigned 16-bit
viewport fields +78/+7C/+80/+84. In SSE operation order the source computes
`x = ((cursor_x - viewport_78)*2 / viewport_80) - 1` and
`y = 1 - ((cursor_y - viewport_7C)*2 / viewport_84)`. The constants are
`14719E4 = 40000000` (2), `14861D0 = 3F800000` (1), and
`13EC4B0 = 322BCC77` (approximately 1e-8). The source does not validate zero
viewport divisors. Output origin is copied from camera +30/+34/+38.

For each `i=0..3`, four homogeneous components use separately rounded SSE
operations in this order: `h[i] = (((camera[110+4*i]*y) +
(camera[100+4*i]*x)) + camera[130+4*i]) + camera[120+4*i]`.
The source then computes binary32 `q = 1/h[3]` and
`delta[i] = h[i]*q - origin[i]` for i=0..2. `7C4216..7C4253` loads the
binary32 deltas into x87, sums z squared plus y squared plus x squared, adds
the 1e-8 constant, takes square root and reciprocal, and stores the reciprocal
to binary32. The result direction is each delta multiplied by that stored
value via separate `MULSS` operations (`7C4257..7C4291`). x87 precision,
rounding, MXCSR and nonfinite/zero cases therefore matter.

A checked reusable projection can take explicit cursor integers, viewport,
camera words and floating-point environment and reject unsupported faults;
that alone does not produce a genuine application ray. To use the source
selection path in the running application, resolve the actual manager +30
camera and controller selection/update, source viewport and cursor sampling
cadence, and source actor/cache/definition identity. The current
`cell_capability_source_join_v1` is an opt-in supplied-resource composition,
not a live cache or actor; its source-derived part and cache models do not
bridge those identities automatically. A new standalone fixture would not
resolve these missing producers.

## Camera update provenance

`7C45E0` copies a caller-supplied 16-word matrix into the camera object at
`+00..+3C`, writes rearranged/sign-adjusted words into its `+170` object,
then calls `7C42E0`. That routine invokes matrix helpers `45E5D0` and
`45F100` and writes their derived 16 words to camera `+100..+13C` at
`7C43DC..7C43EB`, the exact region consumed by the ray computation.
Viewport setter `7C4400` writes four values into `+170` at
`+78/+7C/+80/+84`, then also calls `7C42E0`. This identifies update and
viewport producers once the correct camera object and their arguments are
known, not the selected camera or its live matrix. Helper behavior,
singularity and intermediate floating-point policy need independent work
before reproducing the matrix update.

The direct source path `E5FEE0 -> 7C45E0` updates the camera returned by
manager `+58`, which is the same getter used in `E6C7E0`; this is conditional
on the manager pointer remaining stable between calls. The separate static
chain `E54890 -> 7C45E0` updates global `016B5298`, an auxiliary camera
candidate; this object has not been shown equal to the manager pointer. Manager `+30` starts null in its
constructor and is written by virtual +54 implementation `7D8180`, whose
active caller and argument remain unresolved; see
[cell-camera-manager-scan-v1.md](cell-camera-manager-scan-v1.md).
The Cell setup's `04A4D97F` and `A426730B` path is independently traced
to world-lighting construction and preset selection in
[cell-world-lighting-selection-v1.md](cell-world-lighting-selection-v1.md),
not a demonstrated camera selector. Thus these keys do not close the gap.

A bounded raw `.text` scan for `E8 rel32` targeting `7C45E0` found eight
candidate positions: `7C46B0`, `7C475D`, `E54981`, `E60DE6`, `101CDDE`,
`101F1DA`, `101F39A`, `1020154`. Byte matches alone are not all decoded
call sites; this scan excludes indirect calls and other encodings. Focused
disassembly validates `E60DE6`: `E60D03..E60D0F` retrieves manager `+58`,
keeps that pointer in ESI, calls `10436F0` on two calculated vectors and
global vector `015A8C34`, configures ESI through `7C4540`, `7C4560`,
`7C4CD0`, and passes the resulting stack matrix to `7C45E0`. `E54890` is
then called separately. The `7C42E0` rebuild described above makes these
updated fields relevant to subsequent `7C40D0` ray generation.

`E5FEE0` is the containing function. It skips the update if the object from
`B3D570` has nonzero byte `+2A`; otherwise it resolves the current player
from Cell global `+411C` via the pool, and resolves state `+E0` through the
same pool (`E5FEFD..E5FFB3`). Its mode byte selects branches, which use
Cell state `+48..+F8` and `+908`, available actor `+4C/+50/+54` and
`+90/+94/+98`, `E833B0` runtime data, and `E837F0` curves;
`E60A76..E60B09` is one branch interpolating against the resolved `+E0`
actor position. The full intermediate calculation, helper semantics,
reference lifetimes, data/curve provider and frame arguments remain needed;
no matrix is inferred from an isolated actor position.

Direct calls to `E5FEE0` appear at `E808FE` and `E80986` in `E806E0`,
passing mode bytes 0 and 1 respectively. `E809B0` forwards its two arguments
to `E806E0`; static `Game_Cell` primary vtable slot `+3C` at `1486044`
contains `E809B0`. This supports a conditional registered callback route,
not observed dispatch cadence, successful camera initialization,
manager-setter provenance, active actor identity or a measured frame.

The source image constants passed to the primary-camera calls at
`E60DAA..E60DDE` are `2.0` (`14719E4`) to `7C4540`, `2000.0`
(`145CC2C`) to `7C4560`, and `50.0` (`148467C`) to `7C4CD0`. The up
vector at `015A8C34..3C` is `(0,1,0)`; `10436F0`'s basis construction is
traced in [cell-aux-camera-transform-static-v1.md](cell-aux-camera-transform-static-v1.md).
Those are image constants, not a measured viewport or current pose. Other
source constants used earlier in the update include 1.5 (`1485FF4`), 6.0
(`147C4B0`) and -0.5 (`13EC480`); calls to runtime curve and data providers
can replace or influence the actual values along reached branches.

Under a completed `E54150` initialization and no intervening mutation, Cell
state +48/+54/+60 start from image vector `(100,100,0)`; +6C and +78 start
from `(0,0,20)`, and +E0/+E4 are zero. These are initial state writes, not
proof of the state consumed during `E5FEE0`. On the mode-0 route, actor
velocity words +90/+94/+98 feed magnitude and curve calls at
`E5FF46..E5FF96`; state +E4, a resolved +E0 actor, and current player
resolution choose further camera-target paths. The frame scalar passed by
`E806E0` and current state, curves, input and actor identity remain required.

On the general camera initializer `7C4770`, when camera +170 is null it
obtains a new internal object through `761140`, writes its viewport origin
to zero and reads viewport width/height from unsigned words +0C/+0E of the
object at global `015D1838` (`7C4791..7C4865`). That global is runtime
state, so the dimensions are not static defaults. The explicitly observed
Cell stage initializer passes `016B5298` to this function, but that remains
the auxiliary object unless manager +30 identity is independently established.
A raw `.text` scan found 66 `E8` candidates to `7C4770` and none to
manager setter `7D8180`; this is not a complete call graph or proof that the
setter is absent. The already documented indirect setter, shared-library
registration and selected-controller sources remain unresolved.

# Original Cell movement inputs and scalar conformance

Scope: immutable base executable SHA256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`,
REA v3 retained Evidence and local installed resources, 2026-10-23. This
continues `cell-behavior.md`; the full Cell gameplay gate remains open.

## Source globals

`[MEASURED]` Registrar `137BA60` names `globals`, type `2A3CE5B7`,
descriptor `16B1B3C`, field table `15A7AB8`, size `114` (276 bytes).
Initializer `137BAE0..137D5F8` assigns **68 named four-byte fields**:
43 floats, 16 source references, six enum/code words, two integers and
one type-7 word. Word `D0` has no descriptor and stays unclassified.

All 82 package indexes contain three `CellGame` (`A426730B`) copies:

| Package | Group | Bytes | Current parser |
|---|---|---:|---|
| Data/PatchData.package | `00010000` | 276 | Supported |
| Data/Spore_Game.package | `0001FFFF` | 264 | Explicit older-layout error |
| DataEP1/Spore_EP1_Data.package | `0001FFFF` | 276 | Supported |

The two current copies agree field-for-field. No current tuning values
are embedded in the Rust implementation. Source references remain IDs
before native fixups. A resource loader must resolve the version/overlay
explicitly; this parser does not silently pad the 264-byte resource.

Movement-related source members include:

| Source fields | Offsets |
|---|---|
| flowMultiplier / npcSpeedMultiplier | `44` / `48` |
| npcTurnSpeedMultiplier_Jet / Flagella / Cilia | `4C` / `50` / `54` |
| ciliaSpeedAsJet / flagellaSpeedAsJet | `74` / `78` |
| flagellaSpeedAsCilia / ciliaSpeedAsFlagella | `7C` / `80` |
| noPartSpeed | `90` |
| flagellaRampMinFactor / Time / ResetAngle | `94` / `98` / `9C` |
| flagellaTurnSpeedRampStart / End / Min / Max | `A0` / `A4` / `A8` / `AC` |
| ciliaTurnSpeed / jetTurnSpeed | `B0` / `B4` |
| nonAnimatingCiliaMovementFactor / Jet | `104` / `108` |

`tutorialMethod` is at `DC`; `endingMethod` is at `E0`. These fields
must not be interchanged when tracing `E7CE40`'s progression follow-up.

`tools/cell_schema_probe.py --globals` reproduces the five classes / 149
descriptor rows from bounded SDK disassembly. The normalized combined schema
is `target/source-cell/schema-with-globals-v1/schema.json`, SHA256
`46622a21bb2f6d36de98d7eb2e34b603d5d91b2e84baae51f90ff75d78197bee`.
The independent globals fixture covers two supported source resources,
one explicit unsupported source resource and three synthetic distinct-word
records. Source and synthetic denominators remain separate.

`[MEASURED]` The Rust globals verifier consumed all six fixture records:
five supported records matched all 68 typed accessors, and the original
264-byte record returned the explicit unsupported-layout error. The supported
denominators are **136 source fields** and **204 synthetic fields**. The two
focused globals tests also passed. This establishes the strict borrowed parser,
not the original resource overlay policy or gameplay use of these parameters.

## Native update route and ABI

`[MEASURED]` `E7A1C0` performs the controller update dispatch after
common actor handling. The source controller IDs from the native name table
correspond to these concrete targets:

| Controller | Native update |
|---|---|
| parts | `E71350` |
| slowSeek | `E706A0` |
| orbit | `E70500` |
| chomp | `E70320` |
| eater | `E7A0D0` |
| parasite | `E6FA00` |
| caterpillar | `E78FF0` |
| sleeper | `E70820` |
| mosquito | `E6FC20` |
| dragon | `E6FDE0` |
| poolball | `E6FD50` |
| leak | `E7A100` |
| player | `E6F620` |

This is the observed dispatch body, not proof that omitted controller IDs
never move: common integration and other state updates also exist.

`E52770` returns the same difficulty-selected AI subobject retained by
`cell_definition::selected_ai_variant`. The seek wrappers use actual source
`speed`, `turnFactor`, `axialMovement` and the movement style resolved by
`E67CB0`. That resolver returns one when actor word `FC` is zero, otherwise
the explicit nonzero AI style, otherwise the capability cache's `1254` word.
The pure helper takes those words explicitly; it does not invent a body
capability cache.

Bounded SDK establishes `E6F6D0` arguments as actor, target point, dt,
speed, turnFactor, movementStyle, animationHint and axial byte. It
passes actor in **ESI** to `E6EF30`, whose stack arguments are dt,
speed, turnFactor, movementStyle and animationHint. `E6F870` resolves
an entity's `4C` position and forwards to this seek route. Decompiler
integer-array casts on actor float stores are not movement integer conversions.

## Bounded axial target kernel

`[MEASURED]` The block `E6F707..E6F82A` has 58 decoded instructions,
including scalar SSE operations and branches. For nonzero axial bytes it subtracts actor
position from the target, calculates secondary/tertiary projections, and
uses the exact **secondary > tertiary** comparison. Both branches then
construct a point through the primary axis. This is not replaced by an
assumed geometric dominant-axis rule.

`cell_movement::axial_seek_target` accepts actor position, target point,
all three basis vectors and the source byte. Supplying basis vectors
explicitly avoids treating static executable initial values as proof of
all runtime frames. The helper preserves the source multiply/add ordering
and the nonzero-byte branch. It supports the finite arithmetic domain;
invalid/nonfinite arithmetic returns an error instead of a fabricated target.
The zero-byte branch preserves target bits without consuming unused axes.

`[MEASURED]` `tools/cell_movement_probe.py` reads the original SDK block
and translates only its ABI/memory operands and branch labels into a local
x64 conformance DLL. The original scalar instruction/branch order remains.
The kernel saves/restores caller control and executes with **MXCSR `1F80`**:
nearest rounding, preserved subnormals and masked exceptions. Generated
source-derived assembly, object and DLL remain ignored. No retail process,
window or desktop input is used.

The translated kernel executed **4,119 synthetic cases**, including branch
ties, nonzero flag bytes, signed zero, arbitrary supplied bases and subnormal
cases. Fixture `target/source-cell/movement-fixture-v2/corpus.bin` SHA256:
`781ebf8333637b6c8618d0f92f596a2b16a0ea412664e0749c8c88be8815d13d`.
This is CPU conformance of a bounded translated kernel, not execution of the
complete original actor integrator or proof of retail frame state.

`[MEASURED]` The independent Rust verifier matched **all 4,119 cases at
every returned binary32 bit** under the declared MXCSR control policy. The
subsequent fixture-v3 producer wording correction leaves the corpus byte-identical
to fixture-v2 and retains the same SHA256.

## Bounded movement application

`[MEASURED]` Native `E55FB0` takes a disjoint three-float input vector,
multiplies it by actor float `BC`, and adds it to actor accumulator `90..98`.
If actor word `248` is nonzero and signed count `250` is at least two, it first
applies the same operation separately to each environment vector selected by
the ordered actor indices at `254`. The environment vector-array base is
`4654`. Repeated indices are significant: separate rounded additions cannot
be collapsed to one multiplication by an occurrence count.

`cell_movement_application::apply_movement_delta` exposes these fields and
buffers explicitly. It leaves actor-word meaning, source scale creation and
steering computation unclassified. Invalid pointer/index domains and nonfinite
arithmetic return explicit boundary errors, with no partial commits. These are
Rust boundary checks, not claims that the source performs equivalent checks.

`tools/cell_movement_application_probe.py` translates the native complete
scalar application branch and operation order, replacing only ABI/register
widths, stack bookkeeping and the supplied environment-array base. Its local
x64 DLL executed **4,126 synthetic cases** with MXCSR `1F80`, including repeated
indices, signed count branches, signed zero, inactive malformed indices,
unconsumed NaN vectors and subnormal arithmetic. Fixture
`target/source-cell/application-fixture-v4/corpus.bin` SHA256:
`9a8f8a3e2e382e28c8d6832f9b2377b75b6f3810de3f4a5b6f25e1e962729ba0`.
This is a translated bounded CPU reference, not retail actor integration.
The Rust verifier matched **all 4,126 cases at every accumulator bit** against
the corrected v4 fixture. Both focused application tests passed.

`[MEASURED, FAILED FIXTURE]` The first Rust comparison against fixture-v2
failed at case 10, the first active indexed branch with signed zero. Inspection
found that the producer had placed the signed count at `24C` instead of `250`,
so every original CPU indexed branch had been skipped. Correcting that fixture
also exposed a padded SDK operand (`00004654h`) that needed to be removed from
the supplied environment-array base. Both producer errors are corrected, with
explicit count offsets, active-domain assertions and guarded output vectors.
The failed v2/v3 fixtures remain ignored evidence and must not be used as valid
references. No Rust arithmetic was changed to accommodate the failed reference.

## Complete sampled field arithmetic

`[MEASURED]` `cell_flow_field::sample_flow_field` recovers the complete
finite arithmetic of `E4DB80`, `E4DFD0` and `E4E220` with explicit runtime
origin, scale and the row-major 20-by-15 field vectors at environment `1C210`.
The coordinate upper bounds are source words `419FFFF8` and `416FFFF0`.
`E4DFD0` computes floor through nearest integer conversion plus correction,
then selects centered half-cell weights and a neighbor offset of -1/0/+1.
At field edges the sample index can repeat; all four products and sequential
rounded additions remain present. This is not replaced by ordinary floor-cell
bilinear interpolation or by an edge-collapsed shortcut.

The three source-order CPU coordinate/weight/blend kernels supply an independent
fixture of **2,220 synthetic cases**. The Rust verifier matches all output bits,
and both focused field tests pass. Fixture
`target/source-cell/flow-fixture-v2/corpus.bin` SHA256:
`22f53933de954c32573765c5030fee4638b9e20140e6160c50d5037b715c73af`.
The host between kernels resolves the measured integer row-major lookup; it
does not infer source field population or frame setup.

## Complete bounded turn-in-place steering

`[MEASURED]` `cell_swimming::turn_in_place_delta` preserves `E55E40`'s
mixed scalar arithmetic: source-ordered binary32 squared norm; x87 epsilon
addition, square root and reciprocal; explicit binary32 commits; retained
distance comparison; then scalar SSE scaled target/environment/current-vector
arithmetic. The early return for either signed zero step uses the supplied
source `16B4C08..10` vector without consuming unused position/field inputs.
The sampled environment vector is explicit, and the unrelated unused fifth
native argument does not become an invented gameplay parameter.

The x87 `Context` is an explicit caller input, including precision 24/53/64
and all four rounding modes. Retail precision/rounding setup remains unknown;
the complete game may change it during device creation or initialization.
The helper preserves each retained x87 value across binary32 stores instead
of rounding all arithmetic to binary32 or assuming a hardware default.

The translated source-order CPU reference executed **12,924 synthetic cases**,
**1,077 for each of twelve explicit x87 policies**, with SSE MXCSR `1F80`.
The Rust verifier matched every output bit across all twelve lanes; both focused
steering tests pass. Fixture
`target/source-cell/swimming-fixture-v1/corpus.bin` SHA256:
`f225c3870daee5d62bba184b4b9d6ab010576fa56dc65b9610abde3cf1ae7c18`.
This verifies the bounded arithmetic under supplied policies and environment,
not the retail policy or complete actor update.

## Original-instruction conformance host

`[MEASURED]` A separate reviewed no-CRT x86 fixture host maps only these
unaltered source ranges at their original addresses (ends exclusive):

| Start | End | Role |
|---|---|---|
| `E51230` | `E51290` | Clamped interpolation |
| `E55650` | `E55795` | Heading factor |
| `E557A0` | `E55B12` | Jet arithmetic |
| `E55B20` | `E55E3D` | Flagella/cilia arithmetic |
| `E55E40` | `E55FA2` | Turn-in-place arithmetic |
| `1041E40` | `1041E57` | Inverse interpolation |

Every direct source call targets an allowlisted entry or the `E4E220`
replacement. All direct branches remain inside their function; no source
indirect calls/jumps occur. The manifest retains source-range SHA256s, every
absolute read and all register-relative memory operands for pointer review.
The sole stub copies an explicitly supplied sampled vector into the requested
output and returns that pointer. Runtime root/zero-vector words use separate
bounded host inputs. Output vectors have guards on both sides.

The host imports only `GetStdHandle`, `ReadFile`, `WriteFile`, `VirtualAlloc`,
`FlushInstructionCache`, `GetLastError` and `ExitProcess` from Kernel32. It
reads/writes piped binary records with no window, network, game initializer or
retail process. Its own image base is `24000000`; only the reviewed low code/data
pages are mapped for the math. Raw bytes and the generated host remain ignored.

Reviewed host v4 SHA256:
`5d88efbc1c427806e4d692c69ad9431552c3376b7d6c5df81b316aad95b85492`.
Its **12,924 original-instruction turn cases** match every previously translated
CPU reference bit, with exact **206,784 reply bytes**, guard checks and zero
exit status. `native-turn-check-v2/report.json` retains this independent
protocol/ABI cross-check. Separate checks verify empty input, incomplete records,
a complete record followed by a partial one, and invalid mode rejection.

`[MEASURED, INVALID RUN]` The first v2 host run returned all expected reply bits
but exited nonzero at closed-pipe EOF. It remains invalid execution evidence.
V4 accepts only `ERROR_BROKEN_PIPE` at a fresh record boundary, while every
other failed read or incomplete record still fails. This correction changes
host I/O handling, with a fresh manifest/hash, and no native code/data range.

The unchanged reviewed host has also generated **45,432 original-instruction
flagella/cilia and jet cases**, 1,893 cases for each of 24 mode/policy lanes.
Fixture `target/source-cell/propulsion-fixture-v1/corpus.bin` SHA256:
`e0cb12a14ac6c3bf411bee4420f397ac6635728a88d7e2934e4231a09e948c75`.
The Rust propulsion verifier matches **all 45,432 output vectors and jet boolean
returns at exact bits**, across all 24 lanes. All four focused swimming tests
pass. The source kernels themselves ran in the bounded host; the original game
initializer and retail process did not run.

`cell_swimming::{flagella_cilia_delta,jet_delta}` preserve source differences
that a shared simplified movement equation would lose:

- Flagella/cilia sum target squares in `(z²+y²)+x²` order and compare a committed
  binary32 distance; jet sums `(y²+x²)+z²` and compares the retained x87 distance.
- E55650 normalizes heading vectors in retained x87 arithmetic, commits inverse
  norms, then calculates the SSE dot product as `(z+y)+x`.
- E51230 commits its inverse interpolation before the SSE clamp, then retains
  output interpolation until the caller's store. Both interpolation constants
  and commit boundaries remain present.
- Current-minus-field components remain retained through the x87 magnitude
  test even though separate binary32 copies also exist. Jet's hint byte gates
  the reverse branch and changes its inactive response factor.
- Only the low hint byte is consumed. The corpus includes word `100` with low
  byte zero, so a nonzero-word approximation cannot pass.

Numbered source factors, sampled vector, early-return vector and x87 context
remain explicit caller inputs. These helpers do not assign undocumented actor
word meanings, create a capability cache or integrate an invented simulation.

## Open integration and float policies

`[UNKNOWN]` The complete `E6EF30` path still requires flow `E56440`,
scalars `E56330` / `E563F0`, native selection of the explicit inputs to the
recovered `E557A0` / `E55B20` / `E55E40` kernels, turn-state helpers and
integration of the bounded `E55FB0` application with original actor/environment
state. Exact isolated math is not an original actor update or stage acceptance.
The capability cache's numeric construction is now independently verified;
its live actor/archive binding, input state and original movement/animation
transitions remain open. No approximate replacement of these routines was added.
See [the cache ledger](cell-capability-cache.md) for the complete source-class,
geometry, reduction and scalar gates.

The declared scalar policy is controlled IEEE binary32. Native x87 portions
elsewhere and the complete retail MXCSR/frame setup remain separate evidence
gates. This helper does not set global floating-point state in the game.

`[MEASURED]` The newly recovered steering functions mix scalar SSE with
x87 `FSQRT`, division and retained-register comparisons. In `E55E40`, for
example, the target-distance square root remains in x87 while a separately
rounded binary32 copy is stored; the speed comparison uses the retained value.
`[UNKNOWN]` Retail x87 precision/rounding setup is not yet established. An
explicit PC24/PC53/PC64 arithmetic context can support bounded conformance, but no
hardware-default or binary32-only steering policy may be substituted silently.

`E4E220` supplies a sampled vector field through a 20-by-15 grid and the
coordinate helpers `E4DB80` / `E4DFD0`. Its recovered sampling arithmetic is
available with explicit frame/field inputs. Native field population, frame
creation, actor flow scalars and their live consumer integration remain open.
A zero-vector substitute would alter the original steering behavior.

The electric weapon path `E71570` / `E694F0` consumes arc lengths and
`numArcs`; it must not be mislabeled as the swimming integrator. Its damage
helper `E58800` and source eating/progression paths remain separate work.

`[MEASURED]` `E67A50` is a capability-cache lookup/create wrapper, delegating
population to `E51EC0`; it is not itself a complete part-to-movement capability
algorithm. `E681E0` updates target-facing orientation and tracked-part quaternions
from actor target `170` / part `174` or target point `8..10`; it is not raw input
acquisition. `E61DD0` applies a bounded quaternion turn through native
acos/sin/cos and quaternion helpers. Their source math/runtime policies remain
separate gates; no simplified orientation or fabricated capability mapping was
integrated.

`[MEASURED]` Queries 445..450 extend the input-construction chain without
claiming an integrated simulation. `E4DC40` puts source points into the same
20-by-15 bins, retaining at most eight 32-byte rows per bin. Each row holds
source position, source vector, a scalar and source index. `E4DD50` clears the
300-vector and two 300-scalar outputs, then derives a mean vector, its length
and mean reciprocal scalar for each nonempty bin. `E4EA40` calls that
construction after `E4E730`. The population arithmetic/commit boundaries and
the frame's actual source point/scalar inputs are not yet ported or accepted;
the existing verified sampler still requires its field explicitly.

`E51EC0` constructs capability totals and bounds, then sets output word `1254`
from source totals `1224/1228/122C`. Output scalar `1258` comes through
`E837F0` using property IDs `2EF68471`, `BBAB1BF5` or `2C574746`, with the
source globals at `74/78/7C/80` supplying branch-specific factors; the empty
mask branch instead uses globals `90`. These are native field offsets/keys,
not fabricated part names or a replacement speed formula. The exact `E837F0`
curve and complete scalar cache tail now pass their own original-byte gates;
actual actor/cache binding and complete steering integration remain separate.

`[MEASURED]` Query 489 establishes `E837F0`'s actual source lookup and numeric
rule. It requests property-list instance `C951EB63` with manager scope zero,
then selects the caller's property ID. An all-82-index source census finds two
installed PROP copies, in `PatchData.package` and `Spore_Game.package`, both
1,124 bytes and SHA256
`c00576f276d3e3daa9f898985775ba13e173d66b492bc8e8fcf0b7b3b25a6462`.
Each has 18 native Vector2 arrays (`type0030/spec009C`), 104 points in total,
all finite with strictly increasing x. The initial fixture's legacy `group`
column is not a canonical scope: the later original index-reader proof
establishes the typed 96-bit `(low,type,scope)` resource key. Raw source and complete field/curve census stay
in `target/source-cell/capability-curves-source-v1/`.

This curve rule is not ordinary linear interpolation. Before the first x it
copies first y; otherwise it uses the first right x greater than or equal to
the supplied input, and calls `1041E40`. That helper divides by
`(right_x-left_x)+1/65536`, with retained x87 arithmetic and a binary32 commit.
Separate SSE y subtraction/multiplication/addition follow. At the exact last
x this epsilon-adjusted interpolation can differ from last y; only inputs
strictly above last x copy last y. Count-one and absent/empty paths remain
distinct. No approximate curve evaluator or inferred capability names have
been integrated from these source arrays. The isolated exact numeric consumer
is now implemented in `cell_capability_curve`: **107,208 original-byte golden
cases / twelve explicit x87 contexts pass exact bits**, including **94,440
independently decoded actual PROP cases** from both installed copies. There are
zero output exclusions. See [the curve evidence ledger](cell-capability-curves.md)
for the reviewed host, source provenance, invalid setup variants and API limits.

`[MEASURED]` A read-only import/reference census and queries 462..464 locate
scoped `_controlfp` / `_control87` regions. `A31760` temporarily selects
precision flag `10000` under mask `30000` around a helper and restores the
saved precision; `FCD870` similarly saves/restores decoder control state.
Their cleanup paths do not establish the Cell frame's x87 control word.
The control word at Cell entry remains open, so 24-, 53- and 64-bit explicit
precision contexts remain necessary. The later rendering trace in
[native-float-render-context.md](native-float-render-context.md) recovers
creation without FPU_PRESERVE, explicit post-startup PC24/nearest setters and
FTZ/CPU-gated DAZ setup; none of the existing MXCSR1F80 kernel fixtures is
silently relabeled as retail FTZ/DAZ conformance.

`[MEASURED]` The stronger bounded native startup trace establishes an explicit
**startup PC53 request**. PE entry `11E0F90` jumps to `11E0CD0`, whose call at
`11E0D5F` enters `MSVCR90!_initterm_e` with the source initializer range
`13EB09C..13EB0AC`. Its actual words are
`[0,11E0EAF,11E10F4,11E114A]`. Initializer `11E0EAF` calls `11E130A` at
`11E0F76`; that bounded helper calls `_controlfp_s(NULL,10000,30000)` through
thunk `11E14E6` and takes an `_invoke_watson` error route if needed. The helper
bytes hash to
`6ba5fe5b654e2efe769ece3c2814e967fbb8b8fd409764894978b73db73a323c`.
This is a pinned PE/import/Capstone trace, not a newly fabricated Ghidra body.
It establishes the precision request before later static/game initialization;
later rendering policy and the control word at a Cell frame remain separate
from this startup request. Evidence is `native-default-precision-startup-v2.json` and
`native-crt-startup-11E0CD0.asm` under `target/source-cell/`.

## Reproduction and retained evidence

```powershell
python tools/cell_schema_probe.py 'D:\Games\SPORE Collection\SporeBin\SporeApp.exe' --dumpbin 'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64\dumpbin.exe' --globals --out target/source-cell/schema-globals-new
python tools/cell_globals_probe.py 'D:\Games\SPORE Collection' --metadata target/source-cell/schema-globals-new/schema.json --out target/source-cell/globals-new
python tools/cell_movement_probe.py 'D:\Games\SPORE Collection\SporeBin\SporeApp.exe' --sdk-bin 'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64' --out target/source-cell/movement-new
.\tools\build.bat test -p spore-core cell_globals -- --nocapture
.\tools\build.bat test -p spore-core cell_movement -- --nocapture
.\tools\build.bat run -p spore-core --example cell_globals_verify -- target/source-cell/globals-new/corpus.bin
.\tools\build.bat run -p spore-core --example cell_movement_verify -- target/source-cell/movement-new/corpus.bin
python tools/cell_movement_application_probe.py 'D:\Games\SPORE Collection\SporeBin\SporeApp.exe' --sdk-bin 'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64' --out target/source-cell/application-new
.\tools\build.bat test -p spore-core cell_movement_application -- --nocapture
.\tools\build.bat run -p spore-core --example cell_movement_application_verify -- target/source-cell/application-new/corpus.bin
python tools/cell_flow_field_probe.py 'D:\Games\SPORE Collection\SporeBin\SporeApp.exe' --sdk-bin 'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64' --out target/source-cell/flow-new
.\tools\build.bat run -p spore-core --example cell_flow_field_verify -- target/source-cell/flow-new/corpus.bin
python tools/cell_swimming_probe.py 'D:\Games\SPORE Collection\SporeBin\SporeApp.exe' --sdk-bin 'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64' --out target/source-cell/swimming-new
.\tools\build.bat run -p spore-core --example cell_swimming_verify -- target/source-cell/swimming-new/corpus.bin
python tools/cell_native_math_probe.py 'D:\Games\SPORE Collection\SporeBin\SporeApp.exe' --sdk-bin-x86 'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86' --kernel32-lib 'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.18362.0\um\x86\kernel32.lib' --out target/source-cell/native-host-new
# Review the generated manifest/source before first execution of a new host.
python tools/cell_native_math_run.py --host-dir target/source-cell/native-host-new --turn-reference target/source-cell/swimming-new/corpus.bin --out target/source-cell/native-turn-new
python tools/cell_propulsion_probe.py --host-dir target/source-cell/native-host-new --turn-check target/source-cell/native-turn-new/report.json --out target/source-cell/propulsion-new
.\tools\build.bat run -p spore-core --example cell_propulsion_verify -- target/source-cell/propulsion-new/corpus.bin
```

REA v3 queries 323 / 332..335 / 341..345 retain the selector, controller
wrappers, dispatcher and shared movement route. Queries 365..371 / 382..388
retain the flow, steering, application and their initial dependencies. Queries
391..393 retain field coordinates, centered weights and inverse interpolation.
Queries 438..440 retain capability-cache lookup and orientation consumers;
441..444 resolve field/frame operand locations to enclosing functions.
Queries 445..450 retain field/capability construction; 462..464 retain scoped
floating-control regions. The import census is
`target/source-cell/native-float-control-census-v1.json`.
Queries 489/490 retain the actual curve evaluator and pre-aggregation field
density route. Queries 518/519 now retain its direct dependencies `E4E1A0`
and `E4E4C0`: ordered neighboring bins and per-particle density/velocity
forces. Their bounded arithmetic, source-bin population and live frame inputs
still require a separate implementation/conformance gate before field dynamics
are integrated.
Complete native SDK evidence
is in `native-cell-{globals-registrar,globals-fields,seek-axial}-sdk.asm`.
All source-derived evidence files are under `target/source-cell/`.

## Definition selection and original style/base-speed getters

`[MEASURED]` Native query 812 retains complete `E52770`: raw difficulty zero
selects source AI at `248`, raw two selects `194`, and every other raw word
selects base AI at `E0`. The easy/hard first word `FFFFFFFF` inherits base AI.
This independently agrees with the existing typed definition selector. It does
not use an inferred difficulty range or clamp unknown words.

`E67CB0` returns literal style one when actor word `FC` is zero, before reading
a definition or cache. Otherwise it uses `E52770`, returns a nonzero selected
AI `movementStyle` at local `10`, or requests the three-word actor key
`FC/100/104` through `E67A50` and returns cached word `1254`. Unknown nonzero
style bits pass through literally. A selected source style of zero is the
cache-fallback condition.

`[MEASURED]` Query 813 retains complete `E67D20`. Zero actor `FC` writes four
literal binary32 ones; otherwise it copies cached binary32 `1258` into each
of four output slots with four distinct `FLD/FSTP` pairs. This getter does not
apply authored AI speed/chase/wander/flee factors. Native nonfinite payload
behavior remains separate from the typed finite-speed API policy.

Queries 815/816 recover two actual consumers. `E70F70` updates tracked targets
and calls `E67D20`; it forwards the fourth slot to `E6F9B0`, the second to a
threat-target `E6F870` branch, and the third to other tracked-target/wander
branches. Those calls do not add an authored AI speed multiplier at this
site. `E71C50` calls `E67D20`, selects source AI with the same raw difficulty
and inheritance branches, obtains local `40` turn factor, resolves style
through `E67CB0`, builds a player-target point and passes first speed slot,
turn factor and style to `E6EF30`. Actor/target resolution and the complete
steering dispatcher remain required; a pure getter is not a complete actor
update.

The exported `cell_movement_source` implementation binds these exact getters
to the typed source definition API. Missing reached references and reached
nonfinite cache speeds are explicit checked errors; unused references on
zero-FC/override paths remain unused. Its accepted bounded original-byte host is
retained at
`target/source-cell/native-movement-source-host-v1/manifest.json`. It contains
only complete `E52770/E67CB0/E67D20`, a checked owned cache provider and explicit
difficulty pointer chain. After independent root and animation-peer review,
root executed the hidden host successfully: **68,340 finite requests**, including
all **489 source definition copies**, **5,460 source-derived scalar caches**
in a separate explicitly zero-style lane, and twelve explicit x87 policies.
An additional **192 nonfinite-policy requests** retain both reached errors and
unused-cache defaults. No definition-to-creation pairing is asserted by these
separate lanes. All ten protocol checks passed; original source, host and input
pins remained unchanged. No original game process, initializer or GUI ran.

The Rust verifier matched all **68,340 modeled 32-byte replies** exactly,
including selected source offset, style, provider-call count, four speed words
and reserved output word. Both modeled streams have SHA256
`15780218eb69b5015e61fcd64d9b6c4f641c062e17b95e1ada9a1984af40f0c6`.
The other eight reply bytes are native CW/SW/MXCSR metadata checked independently,
not copied into a falsely modeled result. Every pure Rust call also passed under
three hostile incoming MXCSR states, preserving the caller's complete MXCSR.
All twelve explicit native x87 policies passed. The separate 192-row policy gate
positively checked **96 reached nonfinite-cache errors** and **96 unused-cache
defaults**; native outcomes remain retained, including 24 signaling-NaN payload
quietings. These API errors do not substitute a game speed.

Full native 40-byte finite-reply SHA256 is
`02aca4f5285f9e51919bbf8c1f5e69949847d4fce5ea21949f0504d774ec8df4`;
separate nonfinite replies have SHA256
`37357675ee8b5134fe8e0556756f58ea80fe821ce2ee9c818e7429d67b4e6ba9`.
The accepted comparison is
`target/source-cell/movement-source-fixture-v1/rust-comparison-v1.json`.
Host EXE SHA256 is
`830cf4dc87ffd54c2701910c6ce49b291e5ea7f199d470824f6603b6f61c9186`;
assembly SHA256 is
`30eb5f21f647b26784edbc253e67c9ae1dd84e0f7576b9e9988ab826ccee9875`.

Preparation and comparison tools are
`tools/cell_movement_source_native_probe.py`,
`tools/cell_movement_source_native_run.py` and
`crates/spore-core/examples/cell_movement_source_verify.rs`. The default runner
only prepares requests; its execution flag is reserved for a reviewed host.

## Original size scaling and later runtime data inputs

`[MEASURED]` Complete `E56330` reads actor signed word `358`. Minus one returns
retained x87 one before reading the global object or table. Other admitted
indices use one of twenty original binary32 words at `1484680..14846CF` as
numerator. The denominator comes from `E4ECC0` using the object's signed word
`1C` through the `16B4BE4 ->5190` chain. For the explicit supported range
`0..999`, that helper scans the literal 50-step thresholds and selected words;
exactly 1000 has an independent selected-word-ten branch. This port does not
infer a generic clamped stage rule. Negative raw values would read the word at
`14846C4` as an index, and values above 1000 can scan outside the retained rows;
these are explicit unsupported ranges rather than fabricated fallbacks.

The ratio is committed with scalar `DIVSS`. Inclusive comparisons against
`0.5/1/2/4` return, respectively, retained x87 `4`, `4-(ratio-.5)*4`, `2`,
`2-(ratio-2)*.5`, then default `1`. Its retained result is not replaced by a
premature binary32 commit. Unordered ratios fall through to one; negative
infinity takes four. The explicit scale-table API preserves the exact `DIVSS`
operand order, including quotient NaN payloads, and scopes/restores the full
caller MXCSR while the x87 helpers preserve the caller's environment.

The exported `cell_movement_scaling` module and
`cell_movement_scaling_verify` example now pass original-byte conformance.
The accepted native host
`target/source-cell/native-movement-scaling-host-v1/manifest.json` contains only
the complete `E56330/E4ECC0` source ranges and has no replacement callback.
Its original rows/constants stay pinned; an explicit 80-byte scale table is
made read-only before each entry. A 112-byte request and 32-byte reply retain
the full 80-bit result, its separate `FST32` commit, selected row and native
overwritten-argument-slot ratio, with eight independent FP metadata bytes.
Root and peer reviewed the source/linked host boundaries, root independently
reproduced preparation, and root executed **261,960 finite-input requests** and
**3,864 separate nonfinite-table requests** successfully across all twelve
explicit x87 policies. Every admitted original-table actor index and integer
`0..1000` is covered: **252,252 complete source-table combinations**, plus
60 unused-progress defaults and 9,648 finite synthetic cases. All eighteen
protocol checks passed, all source/host/input pins remained unchanged, and no
original game process or GUI ran.

Rust matches all **24 modeled bytes** exactly in both cohorts: retained80,
separate committed32, selected denominator index and raw `DIVSS` quotient bits.
The eight native FP metadata bytes are independently validated, with sticky
SSE flags retained. Every call also passed under three hostile incoming MXCSR
states while preserving MXCSR and x87 CW/SW. The finite-input projection and
Rust output share SHA256
`9119d20ef6ccdbf0a78ef805e612511526cbaa32f6f2858725aaa4dbfbd80bb5`;
the separate nonfinite-table projection shares
`5fe49fa6a8d2af92b184054ef6063d78bfdc818cb0fc3627068f9be6e1b1695d`.
There are zero exclusions and zero nonfinite retained outputs. This explicitly
includes 1,416 nonfinite quotients from finite operands and 3,456 nonfinite
quotients in the separate raw-nonfinite-table cohort; their actual native
branch results remain modeled, rather than silently dropped.

Native 32-byte reply SHA256 values are
`fefbdc938cabc3bb38429eed51343107884c180bb73b4d50c4705109602fe463`
and `9363bbb9b0e921107f7d50e3001ebea37a0589afbfcd59098ee621b958cf615b`.
Host EXE SHA256 is
`a78321780022321c842deca614b54a312a7f10eee0d6f8b99b00586c1085c520`;
assembly SHA256 is
`feb7a9fd6bf411016e8bbbf285c451b8f0d3246b783becf5db8bba4cd5ef2ed9`.
The final comparison is
`target/source-cell/movement-scaling-fixture-v1/rust-comparison-v1.json`.
Live construction of actor `358`, progression state, source/cache binding and
the complete steering integrator remain separate functional gates.

`[MEASURED]` Queries 848..850 establish the next lookup chain. `E51760` only
resolves actor handle `248` when both it and actor `FC` are nonzero; it uses
the manager at `[16B4BE8]+168`, calls `B71D00`, then returns resolved object
word `24`. The downstream `E833B0` follows that result's `17C`, then pointee
`2E4`, and scans a signed count at aggregate `A0` with **20-byte records** at
`A4`. This is distinct from ordinary PROP lookup. The first matching ID wins.
Its numerator at record `8` is divided in retained x87 by record `10` plus
binary64 `1e-8` from `1447BB8`, then committed by `FSTP32`. Absent IDs or
nonpositive counts write positive binary32 zero and return `AL=0`; callers
test the low byte, not a canonical whole-register Boolean.

Native source strings `data3` at `14140EC` and `data2` at `14140F4` hash
exactly to `98C942B4` and `98C942B5`. Their actual literal pushes at
`7E40BD/AD99A5` and `7E4098/AD9980` reach a named-data lookup and virtual
processing operation. Neighboring strings include `data1/data4/onExit/noStop/
softStop`; that context is a producer lead, not yet an identified runtime class
or completed source-asset binding. Bounded slices, hashes and the exact epsilon
are retained in `target/source-cell/native-runtime-data-consumers-v1.json`.
The fourth caller, `E56440`, requests adjacent `data1` (`14140FC`, hash
`98C942B6`) and ignores the reader's `AL` result, directly using its missing
positive-zero output before the recovered flow math. These are measured
consumer mappings. Subsequent native recovery now ties the aggregate writers
to sampled ANIM kind3 components; live actor/pose construction remains open.
See [the runtime-data evidence ledger](cell-runtime-data-native.md) for the
complete source chain, authored census and accepted reader/writer gate.

`E563F0` requests `data2` only outside actor `18C` states `30/12/14/13` and
returns one on a missing object/value. `E560E0/E561E0` instead gate on actor
`1B0` states `12/13/14/30`, use `data3`, and commit binary32 multipliers of
100 or 20 into actor `1C0`. Their fallbacks remain distinct: current Globals
`108` or `104`, or `E50200(2)` on the cilia missing-data branch. `E50200`
acquires/releases its Globals reference, selects `104` for raw kind two,
`108` for four, or literal ten otherwise. No guessed data producer or new
state-speed rule is integrated while that live binding remains open.

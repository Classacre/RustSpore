# Original outline-font fixture boundary

This note records the bounded source-bitmap fixture and its native results.
The reviewed v2 host now executes the original scaler, FPGM/PREP and glyph
programs under an explicit hidden debug-created startup condition. All 578
glyphs have completed, giving 4,624 observations across four heap policies
and fresh or bounded ordered contexts. Seven fresh/ordered differences are
retained. Earlier pre-entry mapping failures remain preserved and unexplained
for ordinary child startup. The SFNT metadata and atlas comparisons in
[spui-font-native.md](spui-font-native.md) verify separate interfaces;
native output does not establish Rust hint/raster equivalence.

`[MEASURED]` means a fact read from the pinned installed bytes, a retained
native dossier or an explicitly scoped native fixture. `[PREPARED]` describes
an owned artifact without an associated successful native run. `[PROPOSED]` describes a
future experiment. `[UNKNOWN]` is an
unclosed dependency or an unmeasured result. The resemblance to the T2K API
does not establish an engine vendor/version or justify a substitute scaler.

## Inputs and source identity

`[MEASURED]` The base executable used throughout is
`SporeBin/SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
All addresses below belong to that image. The source title's ordinary mode-2
style chain selects TrebuchetEA Bold, size 11.0 and smoothing value 1; the
live retail selection and final bitmap have not been observed.

`[MEASURED]` The corresponding installed font payload is 117,628 bytes,
SHA-256
`f6ad6f1e12e2d2a7c1e28b0ea3cb93a653482840b2b6340fb8682f3eb4c75f26`.
The retained file is
`target/source-ui/font-file-Spore_Graphics-00010000-027C5CEF-0248E873511B49CA.bin`.
It has 578 glyphs and 2,048 units per em. Its Windows platform 3 / encoding 1
cmap supplies these small fixture candidates:

| Character | Glyph index | Source outline |
|---|---:|---|
| space U+0020 | 1 | Empty, no contours |
| A U+0041 | 34 | Two contours, 195 instruction bytes |
| O U+004F | 48 | Two contours, 97 instruction bytes |
| a U+0061 | 66 | Two contours, 201 instruction bytes |
| g U+0067 | 72 | Two contours, 185 instruction bytes |
| copyright U+00A9 | 137 | Three contours, 264 instruction bytes |
| eacute U+00E9 | 110 | Composite, ten instruction bytes |

The independent metadata extract is
`target/source-ui/font-title-fixture-inputs-v1.json`. The empty glyph is an
initial ownership/metrics case. The simple glyphs exercise bitmap production;
the composite is a separate extension after that path passes. A space result
alone cannot verify scan conversion.

## Original public call boundary

`[MEASURED]` The original outline-font wrapper exposes this initialization
sequence. These are interface facts, not reconstructed source code:

1. `8A6E60(allocate, free, reallocate)` installs three allocator callbacks.
2. `8D08D0(&error)` constructs the native memory handler; it delegates to
   `8D07D0(&error, 501)`.
3. `8CBBC0(memory, context, read_callback, font_length, &error)` constructs
   the input stream. There are five cdecl arguments. The fifth argument is
   visible in caller assembly although query 473's pseudocode omits it.
4. `8B5F50(stream, &error)` classifies the font.
5. `8D02F0(memory, classification, 0, stream, 0, 0, &error)` constructs font
   data for face index zero.
6. `8BF1A0(memory, font_data, &error)` constructs the scaler. The wrapper
   then writes platform 3 and encoding 1 to font-data fields `88` and `8A`.
7. `8BF6E0(scaler, 1, 96, 96, fixed_matrix, 1, &error)` sets the transform,
   where the diagonal entries are `11 * 65536` and off-diagonals are zero.
8. `8C1310(scaler, glyph_index, 0, 0, 3, 0x8B, &error)` requests the glyph.
9. Copy the reported bitmap and metric fields before requesting another
   glyph or purging the scaler.
10. `8BFDF0(scaler, 1, &error)` purges the rendered glyph. The original
    wrapper additionally clears scaler fields `118` and `11C` afterwards.

`[MEASURED]` These outer functions supply the stable cdecl boundary.
Several internal helpers have private register inputs: for example
`8CF6B0` consumes font data in ESI and `8BE9E0` consumes the scaler's table
area in EAX. Calling those helpers with invented C prototypes would change
the original interface. The scaler constructor initializes its own lookup
table at `scaler+198`; no replacement table is needed.

`[MEASURED]` Size 11.0 is not a proof of 11 pixels per em. `8BF6E0`
first scales by the supplied 96/72 resolution ratio. Its positive fixed
multiply `8D0C70` gives:

```text
fixed_size       = 11 * 65536 = 720896
fixed_resolution = (96 << 16) / 72 = 87381
scaled_size      = (720896 * 87381) >> 16 = 961191
rounded_ppem     = (961191 + 0x8000) >> 16 = 15
```

`8BF430` writes both rounded values to scaler `128` and `12C`. Native head
reader `8CD260` maps source `lowestRecPPEM` to its object field `32` and
`glyphDataFormat` to field `38`. This font supplies 12 and zero respectively.
The initial bit-`80` eligibility check in `8BFE40` therefore survives the
size threshold. Original hint program dependencies remain in scope; they
must not be removed on the assumption that the style's `11px` means ppem 11.

`[MEASURED]` During this transform, `8BF430` invokes `8BF2F0`, which
compares ppem with cached fields `16C`/`170` and runs `8CC2C0` when it has
changed. That helper prepares the native hint state and executes `prep`.
The first glyph consequently follows both the original `fpgm` bootstrap
and the ppem-dependent `prep` stage.

`[PROPOSED]` Keep fresh-font cases and ordered glyph sequences separate.
The source owns persistent font/hint/cache state; no glyph-order
independence has been established. Record the exact glyph sequence, purge
points and context reuse, rather than combining results from different
initial states into one bitmap claim.

## Proven format and flag restrictions

`[MEASURED]` The actual font directory contains ordinary `loca`/`glyf`,
`cvt `, `fpgm` and `prep` data. It contains no `DSIG`, `CFF `, `EBLC`,
`EBDT` or `EBSC`. Classifier `8B5B40` rejects its DSIG/CFF candidate;
`8B5C50` accepts the ordinary SFNT/loca candidate, selecting classification
2. The font-data constructor consequently takes `8CF6B0`'s SFNT path.

`[MEASURED]` Source head `glyphDataFormat=0` selects ordinary parser
`8CD700` from glyph loader `8CE6C0`. It excludes that loader's compressed
formats 2000/2002. A `loca` interval of zero length instead selects the
native empty-glyph constructor. The ordinary glyph parser's stream reads
are part of the same callback contract described below.

`[MEASURED]` With no CFF/embedded-bitmap tables, scaler transform `17C`
remains zero and the embedded-bitmap fast paths in `8C1310` are not selected
for this fixed font/transform. Flags `8B` contain none of `100`, `1000` or
`E000`, excluding that outer method's special post-filter branches
`8C2C50`, `8C2C60` and `8C3080`. In particular, `8C2C50` is a three-sample
post-filter wrapper; it is not the bitmap-release operation.

These restrictions are inputs to the ordinary-path audit below. They do
not by themselves verify the parser, hint processor, fixed-point math or
scan converter, or admit an unrestricted source code range.

`[MEASURED]` The optional bitmap allocator is scaler field `24`, with
context in `20`. Constructor `8BF1A0` explicitly clears both fields and the
post-render effect function/context in `40`/`44`. Each render `8C1310`
overwrites enable field `28` with
`allocator != 0 && effect == 0 && !(flags & 100)` before rendering. The
fixed fixture consequently takes the zero branch, independently of the
previous contents of `28`; no constructor initialization of that field is
assumed. `8BFE40` supplies null callback/context to `8C86E0`, whose null
branch allocates through guarded cache slot 5. Its optional callback would
otherwise have the two arguments `(context, requested_bytes)`.

`[MEASURED]` The separate effect call in `8C1310` is guarded by field `40`
and has arguments `(scaler, field44)`. The selected call sequence does not
install either optional hook. Native hint entry `8ABCE0` also receives a
null fifth, trace-callback argument from `8CBE80`, `8CC2C0` and `8CC5C0`,
selecting the ordinary `8ABCA0` loop for bootstrap, preparation and glyph
execution. These are explicit fixture contracts;
they do not prove that every indirect call in the conservative library
inventory is reachable or fully admitted.

## Allocation and stream contract

`[MEASURED]` Allocator adapters `8A6E30/40/50` jump through globals
`1668A2C/30/34`. The full game's UI constructor normally installs
`926380`, `6B4970` and `613EA0`, which access its engine-heap global
`16C9B20`. A standalone experiment can use the exposed callback setter
with an explicitly declared owned allocator. It would verify scaler output
under that provider; it would not verify the game's heap implementation.

`[MEASURED]` The native memory handler is `B0` bytes with a tracking table
of 501 pointers. Guarded allocation `8D0940` requests `size+0x12` bytes,
keeps a 16-byte header and returns `raw+10`. It checks/stores magic,
requested length, tracking index and two tail bytes `5A F0`.
Reallocation `8D0A10` preserves the tracked index and rewrites the guards.
A null data pointer returns null; it does not imply a malloc request.
Free `8D0B20` checks the guards and exact tracking slot before release.

`[MEASURED]` Cached allocator `8D0BB0(memory,size,slot)` uses seven
pointer/size/available arrays starting at handler offsets `50`, `6C` and
`88`. An available slot is marked busy and reuses or grows its allocation;
an already-busy slot falls back to an independent guarded allocation.
`8D0C10` frees all seven cached pointers and resets their sizes/flags.
Normal memory destruction `8D0C50` calls that helper and frees the tracking
table and handler. It does not itself walk every active tracked allocation;
the preceding object destructors must have released their owned objects.

`[PREPARED]` The owned provider keeps an independent 4,096-slot ledger,
limits each raw request to 1 MiB and live allocations to 64 MiB, and returns
16-byte-aligned pointers. It supplies the raw allocations consumed by the
unchanged native guarded allocator, retaining its original headers/tails.
The provider adds separate 16-byte outer canaries, checks every live guard
at allocation/free and phase boundaries, records allocation peaks, and
requires zero live allocations after normal destruction. These checks do
not instrument every original computed memory access; `maxp` bounds alone
are not a heap proof.

`[PROPOSED]` Repeat the first valid fixture with two payload poison values
and distinct fresh allocation layouts. Native constructors deliberately
leave some guarded fields uninitialized: for null style configuration,
`8CEA10` clears only font-data `A8`/`AC`, whereas non-null configuration
copies six words through `BC`. A zero-filled test heap alone could conceal
an accidental dependency on fields protected by a null callback. Capture
only defined result fields and bitmap bytes, and require identical outputs
across the admitted heap layouts.

`[MEASURED]` Source read adapter `890AF0(context,destination,offset,length)`
is cdecl. It calls context virtual slot `28` with `(offset,0)`, then slot
`30` with `(destination,length)`. It returns zero only when seek reports
success and read returns exactly the requested length, otherwise -1.
The glyph parser's ten inline indirect reads use stream field `4`, with
stream field `8` as context; these are read callbacks rather than an
unknown font plug-in interface.

`[MEASURED]` Substream constructor `8CBA40` copies the parent's first three
words (buffer, read callback and context), then adds the requested offset
to the parent's base and maintains its own cursor/end. The callback-backed
fixture's substreams therefore retain `890AF0`; they do not require a new
external reader. The measured virtual call supplies two 32-bit stack words
`(offset,0)`; this note does not infer additional IStream ABI arguments.

`[PREPARED]` The host retains `890AF0` and supplies a small owned context whose two
exercised thiscall slots seek/read the immutable, digest-pinned font bytes.
Every range must satisfy `offset <= font_length` and
`requested <= font_length-offset`. Writes must target a verified source
allocation or the admitted stack frame; EOF must not silently shorten a
read. The context forwards to no external file or engine service. Its owned
fastcall implementations consume context in ECX, ignore EDX and return with
eight stack bytes removed, matching the measured two-argument thiscall sites.
The source input wrapper allocation is `23C` bytes; no smaller extent is assumed.

## Bitmap capture and cleanup

`[MEASURED]` The normal byte-alpha result uses scaler `10C` width, `110`
height, `114` pitch and `118` buffer. Source fields `E4`/`E0` carry 26.6
offset values and `D0` a 16.16 advance. Their exact bytes, integer values
and source field names should be recorded without assigning unproved
layout/raster rounding conventions.

`[PREPARED]` The driver validates signed extents/pitch and multiplication
bounds before copying. For a nonempty result it requires the complete
bitmap interval to belong to a live source allocation, then emits the
pitched buffer with its width/height/pitch. A zero-byte result copies no
pixels and records the raw metrics/pointer state. Only active-width bytes
and defined metrics enter later semantic comparisons; padding is forensic.
This occurs before
`8BFDF0` or another render can release or reuse the allocation.

`[MEASURED]` Full outline-font cleanup `890DB0` proves this order:

```text
8C1600(scaler, &error)
8CEFB0(font_data, &error)
8CBCE0(input_stream, &error)
8D0C50(memory)
```

The game's wrapper clears each field after the corresponding call, then
releases its outer source stream. A standalone context must retain its
font payload until this sequence finishes. Destructor-specific error and
ownership branches are included in the ordinary-path audit below. Successful
cleanup is now measured for the explicit fixture cohort described below;
unexercised native error branches are not covered by those successful runs.

`[MEASURED]` The low-level scaler destructor `8C1600` verifies its head/tail
magic, purges the active outline/bitmap, releases the font-data scratch
allocation and its own arrays, clears the handler's seven caches and frees
the scaler. `8CEFB0` then releases the font tables and font-data object.
The callback input stream's native kind is 4; `8CBCE0` frees its native
wrapper without freeing the externally owned font payload. These bodies
need no game heap service beyond the installed allocation callbacks and
the already identified error imports.

## Hint program boundary

`[MEASURED]` Font-data helper `8CBE80` constructs the ordinary TrueType
hint context. It loads the original `cvt ` values and retains/copies the
`prep` and `fpgm` streams. If `fpgm` is present, it invokes `8ABCE0`
during construction. That occurs before the first glyph, including an
empty-glyph test. `8CC8C0` only builds fixed scaling descriptors; it is
not the hint interpreter.

`[MEASURED]` `8ABCE0` chooses interpreter loops `8ABCA0` (ordinary case)
or `8ABBA0` (callback case). Their actual opcode target is indexed from
hint-state field `8C`, initialized by `8AECC0` to source table `1432F30`.
The 256 pointer entries occupy 1,024 bytes and contain 94 distinct handler
addresses. Table SHA-256 is
`4ddd94de616cd2676114dca3079f5286160e82aa154eb6d2ebf68fb74a51fc9f`.
The read-only pointer inventory is
`target/source-ui/font-hint-opcode-table-v1.json`; the table bytes remain
in the ignored source evidence. Native glyph-hint setup `8CC5C0` feeds
the source glyph instructions to the same interpreter.

`[MEASURED]` A stricter control trace closes this fixed font's initial
`fpgm` bootstrap without interpreting the definition bodies. Its 1,514
bytes contain exactly one top-level NPUSHB and 53 top-level FDEF operations.
The source handler `8AE840` consumes one function ID, then skips the opaque
body with `8AE1E0`'s PUSH-aware scanner until ENDF. Those skipped opcodes
are not dispatched during bootstrap. The 53 IDs are unique, span 0..84
and stay inside the source's 85-entry function-definition table. The stack
peaks at 53 elements of the font's declared 327 and finishes empty.

`[MEASURED]` Native maxp reader `8CD330` places SFNT `maxFunctionDefs` and
`maxStackElements` in native fields `18` and `1C`. Constructor `8CBE80`
allocates eight and four bytes per element respectively, giving a 680-byte
function table and a 1,308-byte input stack. The static trace verifies every
definition-body read and table write against those bounds. FontTools
4.60.1 independently agrees on the complete definition-ID order and both
capacities. Reproduction is
`python target/source-ui/font_fpgm_bootstrap_static.py`; report
`target/source-ui/font-fpgm-bootstrap-static-v1.json` records trace SHA-256
`aa765af51df1ae4dc9b8d631a11a59c81c037c4bc2a44c8abc331f837b14e9b4`.
The tool rejects any other top-level opcode. It executes no original font
or scaler code and does not remove dependencies from `prep`, glyph
programs or later function-definition invocations.

`[MEASURED]` All 94 handler entries remain roots in the subsequent
ordinary-path closure. No handler was removed using lexical opcode
occurrence. A direct-call walk alone cannot discover this dispatch, and
the fixed font's program lengths and `maxp` limits do not justify replacing
the interpreter or assuming that an empty glyph avoids bootstrap execution.

## Error boundary and remaining execution gates

`[MEASURED]` Guarded native methods call imported MSVCR90 `_setjmp3` with
the memory handler's buffer at `10` and count zero. Error helper `8D07A0`
sets handler magic `A8` to `5500AAFF`, then invokes imported `longjmp`.
Some allocation guards call `longjmp` directly. Native error branches may
call `8D08F0`, which frees all tracked allocations and the handler.
Continuing with previously captured object pointers after that cleanup is
invalid.

`[MEASURED]` Hint entry `8ABCE0` additionally owns a separate 64-byte
stack-local `_setjmp3` buffer and returns its nonzero jump status to its
caller. The font-data constructor does not inspect that returned status.
This buffer is distinct from the memory handler's buffer. A reviewed error
provider must account for both stack and heap destinations; replacing all
setjmp results with zero would hide this native branch.

`[MEASURED]` Installed MSVCR90 identity is the one pinned in
[crt-ciacos-native.md](crt-ciacos-native.md). Its `_setjmp3` export has the
closed span `78592370..785923EB` (exclusive end), 123 bytes, with no calls
or global-data dependencies. It saves the return registers/stack position
and reads `FS:[0]`; when that chain is not -1 and count is zero it also
reads the saved scope field at chain+`C`. Its stack/SEH preconditions must
be retained. Returning zero from an invented setjmp stub does not preserve
the source error contract.

`[PREPARED]` The host admits only the original count-zero `_setjmp3` path.
A checked tail adapter validates a writable 64-byte stack/heap buffer and
the owned current-stack SEH record, then jumps to the unchanged 123-byte
CRT body so its saved caller EIP/ESP retain native meaning. The original
path writes 40 bytes. All twelve selected call sites are statically checked
for count zero, including the separate hint-stack buffer. The owned
`longjmp` adapter validates its buffer/SEH chain, records the original
native status and terminates the process. It provides no native recovery
or error-cleanup equivalence. A vectored handler records native exceptions
and terminates; no exception is resumed. The manual 16-byte SEH record and
`/SAFESEH:NO` produce the retained compiler C4733 warning and require review.

`[MEASURED]` The initial conservative report
`target/source-ui/font-fixture-frontier-v1.json` reaches 134 known bodies
totalling 75,420 bytes, with 78 missing direct targets and 48 indirect
transfers. The opcode-handler entries were not included as covered bodies
by that initial direct-call walk.

`[MEASURED]` A subsequent batch's item-level success flag did not prove
address fidelity. Of 172 batch items, 114 resolved to exactly the requested
analysis VA, 50 reported success while returning unrelated file-offset
fallback pseudocode, and eight failed explicitly. All 58 non-exact items
are quarantined in
`target/source-ui/font-ghidra-batch-v1/address-audit.json`; their pseudocode
is not admitted as font evidence. Separate strict dossiers for the same
172 addresses supply 113 exact bodies and 59 unavailable entries, with
zero mismatched bodies admitted. The extra unavailable entry is the
independently decoded `011E0534` memcpy import thunk.

`[MEASURED]` Bounded source-byte CFG recovery supplies the undefined opcode
entries and small callback helpers independently of those rejected
pseudocode results. Report
`target/source-ui/font-sdk-cfg-boundaries-v2.json` covers 94 entry roots
and 7,910 instruction bytes counted per entry. Every walk reaches source
returns, with no decode, overlap or instruction-interior-target problems.
Five compiler switches are followed only after proving their index bounds
from the actual CMP/JA and complete byte-remap tables. The report retains
all code spans, direct/indirect calls and exact switch-data ranges. This
recovers instruction boundaries; it does not close indirect-call semantics.

`[MEASURED]` The merged, unpruned
`target/source-ui/font-fixture-frontier-v7.json` reaches 408 entry bodies
totalling 161,612 unique, non-overlapping code bytes, with no missing direct
target bodies in the expanded ranges. All retained ranges decode
completely. These conservative counts include unsupported font formats and
optional paths. They are not the eventual required size, a fully audited
heap/data/callback interface or a runnable allowlist. The exact-address
audit remains mandatory for further dossiers; lexical opcode occurrence
remains separate from reachability.

`[MEASURED]` Some Ghidra owner-body ranges omitted actual compiler-switch
case blocks. Following real source targets, including nested switches,
recovers those bytes. Final report
`target/source-ui/font-switch-data-v8.json` proves the unsigned
compare/remap bounds for all 39 current compiler switches. Each guard's
fallthrough dominates its dispatch in the expanded intraprocedural CFG,
and every target belongs to the recovered owner code. Their 31 coalesced
data intervals total 1,763 bytes and overlap none of the code intervals.
`font-sdk-cfg-boundaries-v8.json` covers the 116 affected/undefined entry
roots, with 43,347 instruction bytes, 209 return exits, no unresolved
indirect jumps and no byte-boundary problems. These compiler-switch facts
are distinct from the three allocator jumps and 199 indirect calls still
listed by the conservative frontier.

`[MEASURED]` Report `target/source-ui/font-callback-contracts-v2.json`
classifies all 199 calls without treating an address bucket as a path proof:
two source-stream virtual slots; two opcode-table dispatches; one optional
hint trace call; 51 internal hint math/rounding/loop calls; 121 calls with
the input-stream read-field shape; seven optional scaler bitmap/effect calls;
seven local native curve-helper calls; four optional font-style calls;
and one each for an encoded-bitmap decoder, local index-offset reader,
format advance-width getter and font-name comparator. The report preserves the exact call
addresses, source instruction facts and unresolved target expressions.
It separately records the measured native function-pointer assignments.
Matching a stream field offset or observing assigned pointer literals does
not establish every receiver's ownership or exclude aliased writes.

`[MEASURED]` The additional bitmap call `8B1BD3` is the same guarded
scaler `24` allocator, with context in `20`. Local call `8BA514` follows
source branches assigning one of `8B9680`, `8B9730`, `8B98A0` or `8B9AB0`.
Name-table constructor `8D10E0` receives `8B8500` or `8BD190` from its two
retained callers, identifying the comparator invoked at `8D1274`.
Font constructor `8D02F0` supplies format advance-width callbacks
`8B8360`, `8BD900` or `8CAE70` for its optional font branches. The ordinary
SFNT font's `hmtx` object instead selects the direct native getter `8CEA70`;
`8CEAF4` belongs to the alternative cached-width wrapper `8CEA90`.

`[MEASURED]` Decoder callback `8B15BA` comes from a stack object constructed
by `8B1930`. It selects a source word at `1433330` using the encoded header's
top two bits. Indices 0..2 contain native entries `8B1630`, `8B16D0` and
`8B1730`, which are not retained by the current conservative frontier.
The fourth adjacent word is `03AA0FFF`, outside the source image; it is not
promoted to a valid function pointer. This selector is explicitly unadmitted.
The fixed ordinary SFNT constructor leaves its separate format-context field
`font+28` zero, excluding `8C1310`'s `8B2B20` route to that decoder.

`[MEASURED]` The conservative inventory includes optional formats; its
presence alone does not make an optional body part of this fixed fixture
or admit unsupported encoded selectors. The fixed ordinary-path audit
below retains 28 stream reads and all 51 internal hint calls, with declared
stream, hint-state, function-table, rounding and CVT receiver contracts.

`[MEASURED]` The native hint side tables also have finite source bounds.
Handler `8AD7E0` visits row indices 0..19, reading two words per eight-byte
row from hint-state `C0` and one word per row from `C4`. Any final selected
row was saved from that same bounded loop. Together with the opcode table,
the exact source R window is `01432E68..01433330` (exclusive end), 1,224
bytes, SHA-256
`76fd876ea4656f67fd2de02fcfa1f0b7fa2dd302fa55e7cf76fc4df2acf6669b`.
The report is `target/source-ui/font-hint-constant-window-v1.json`.
No coefficient or pointer-table bytes are embedded in tracked source.

`[MEASURED]` Inventory
`target/source-ui/font-core-instruction-inventory-v2.json` decodes 51,697
instructions in the 161,612 retained code bytes. It finds no x87 or vector
instructions; memory operand widths are one, two or four bytes. This
integer-core observation excludes the external CRT providers and outer
float-to-fixed/metric wrappers. It does not verify native bitmap output or
justify replacing the original hinting/scan conversion.

`[MEASURED]` Complete operand inventory
`target/source-ui/font-data-references-v1.json` includes register-relative
large displacements as well as absolute/indexed operands. Among the retained
instructions it finds 91 source-image memory operands: nine allocator-global
references, 49 bounded compiler-switch references, 17 declared import-slot
references and 16 other data references at 12 addresses. All directly or
displacement-addressed writable source globals are the three declared
allocator callbacks. This does not bound writes through computed heap or
stack pointers.

`[MEASURED]` The same conservative report retains 53 pointer-looking data immediates;
five refer to the bounded hint window and 48 remain individually listed.
A literal value alone does not supply an array extent. The ordinary-path
report excludes all 16 other memory operands and 48 literals using explicit
source predicates; none is admitted simply because it lies inside `.rdata`.
Register-computed pointers still require their separate object/table origin
proofs. Reproduction is
`python target/source-ui/font_data_reference_probe.py` and does not execute
font/scaler code.

`target/source-ui/font-fixture-static-status-v1.json` gives the current
report paths, measured counts and remaining gates in machine-readable form.

## Prepared ordinary-path host and review gate

`[MEASURED]` Report `target/source-ui/font-ordinary-path-static-v1.json`
selects 269 source entries, 24,341 instructions and 75,464 non-overlapping
code bytes using 35 explicit source predicates. These include classification
2, ordinary head format zero, null format/style/bitmap/effect contexts,
the selected flags and absent optional SFNT tags. All 94 native hint opcode
entries remain in scope. The selected CFG has no missing direct body or
unresolved indirect jump. Its 94 indirect calls comprise 28 stream reads,
51 internal hint operations, one opcode dispatch, two owned stream slots
and twelve checked error-import calls. The bounded contracts report is
`target/source-ui/font-fixture-contracts-static-v1.json`.

`[PREPARED]` `tools/native_font_glyph_probe.py` copies only the selected
unchanged source instructions, 670 bytes of bounded compiler-switch data,
the 1,224-byte hint window and the 123-byte pinned `_setjmp3` body into
the ignored bundle `target/source-ui/font-glyph-native-v1`. Eleven 64-KiB
NOACCESS reservations contain 43 committed 4-KiB pages. Unused committed
code bytes are INT3; unused data bytes are A5. Native code pages become
RX, bounded tables and font bytes R, and the three allocator callback
globals RW. Rewritten import slots become R after initialization. This
does not map or load either complete original executable or CRT module.

`[PREPARED]` The owned no-CRT driver imports exactly thirteen Kernel32
functions, with no loader, GUI, file-open or process-launch import. Its
complete 6,330-byte `.text` decodes as 2,278 instructions. An intraprocedural
CFG fixed point over all 52 MAP function entries resolves all nineteen
register-indirect transfers: fourteen source/CRT entry transfers and five
cached allowlisted import calls. Every predecessor must prove the same
constant/import; calls invalidate EAX/ECX/EDX and partial writes invalidate
the complete register. This audits owned call targets, not every computed
heap/VM access in original code. Full code, data/page hashes, compiled
disassembly, build warnings, protocol and provider policies are retained in
`manifest.json`, `owned-host-audit.json` and `build-log.json` in that bundle.

`[PREPARED]` The protocol accepts one to 64 in-range glyph indices, either
fresh contexts or one ordered sequence, and the two poison values and two
allocation layouts. It validates the complete request and exact EOF before
mapping or executing source code. Every context checks the 53 original
FPGM definition records before transform, the resulting 15-ppem cache and
ordinary branch fields after PREP, then emits defined metrics, alpha
pixels, controls, diagnostic CVT/storage snapshots and final ownership
counters. Hint-state snapshots remain diagnostic fields, separate from the
glyph semantic digest; pitch padding remains forensic evidence.
Purging and the full native destructor sequence are mandatory.

`[MEASURED]` A subsequent source-byte check closes initial storage
definition: `8CBE80` allocates `maxStorage*4` bytes at `8CBEE1..8CBEF0`,
then `8CBF00..8CBF0D` writes zero to every word from index zero to count-1.
`8CD3A3` supplies that count through native maxp field `16`. The fixed font
has 47 words, so all 188 bytes are defined before FPGM/PREP execution.
All loop instructions are retained by the ordinary-path allowlist.
`target/source-ui/font-hint-storage-initialization-static-v1.json` records
the exact span and independent FontTools count check. This makes later
storage snapshots useful as an additional state comparison; native
execution and resulting values still require measurement.

`[MEASURED]` The v1 protocol's final `ReadFile` requires a successful
zero-byte EOF result. A closed anonymous pipe instead reports
`ERROR_BROKEN_PIPE` and takes the explicit input-error exit. The reviewed
v1 run contract therefore uses regular-file stdin; output may still be
captured without a visible window. The frozen binary is unchanged.

`[PREPARED]` `tools/native_font_glyph_reply.py` parses the request/reply
without launching a host. It checks record ordering, capacities, controls,
bitmap bounds and final ownership, and retains fatal status/phase/context
diagnostics. A separate 63-case synthetic corpus covers malformed/truncated
protocols and verifies that storage, font+C8 and padding do not alter the
active-pixel/defined-metric digest. Its report is
`target/source-ui/font-reply-synthetic-protocol-v1.json`; these synthetic
cases execute no original font/scaler code.

`[MEASURED]` Two fresh regular-file-input v1 attempts stop with exit 130,
error kind 2/detail 3, `ERROR_INVALID_ADDRESS` 487 for address `00670000`.
Both report phase 1, zero setjmp calls and zero native heap allocations:
no original font/scaler entry ran. The same error shape is used for both
reservation and page commitment; the record alone cannot identify which
loop failed. A separately inspected suspended child had that region free.
Neither a later conflicting allocation nor its owner has been established. The original
frozen bundle and failed-run evidence are preserved.

`[PREPARED]` The separately frozen v2 bundle is
`target/source-ui/font-glyph-native-v2`. It omits only source leaf
`0067C860` (one RET byte), its otherwise-INT3 page and low reservation.
That leaf is opcode `59` (EIF) in the native hint table. Before table R
protection the owned driver checks original slot `01433094` equals
`0067C860`, then substitutes owned adapter `240028CC`. Its complete
MASM body is exactly one `C3` byte ending at `240028CD`: normal return
stack/EIP updates, with all other registers and EFLAGS preserved. This
four-byte pointer substitution is an explicit harness adaptation; original
table identity applies only before that patch.

`[PREPARED]` V2 retains every other original code/font/data byte and every
provider/control policy unchanged. It maps 75,463 original font code bytes,
93 original opcode-handler targets plus the one owned EIF adapter, 42
committed pages and ten reservations. Its 6,367-byte owned `.text` decodes
as 2,287 instructions; all nineteen register-indirect transfers remain
resolved across 53 MAP functions. The manifest records the exact omitted
span, original/replacement pointer, pre/post-patch hint-window hashes and
byte-equality checks against v1. Reproduction is
`python tools/native_font_glyph_probe_v2.py --v1-bundle target/source-ui/font-glyph-native-v1 --out <fresh-target-directory>`.
The v1 preparer, template, reply parser and bundle are unchanged.

`[MEASURED]` The reviewed v2 attempt also stops before original code, for
address `00890000`, with the same error kind 2/detail 3 and Win32 error
487. Its phase is 1, with zero setjmp calls and zero source heap allocations.
The retained run is `target/source-ui/font-glyph-root-smoke-v3`.
The EIF adaptation therefore does not close the address-space gate.
Further source substitutions are stopped.

`[MEASURED]` Independent compiled-table audit
`target/source-ui/font-mapping-static-audit-v1.json` checks both frozen
binaries. Reservation tables match their manifests, use eight-byte stride
and aligned non-overlapping 65,536-byte extents. All 43/42 compiled map
entries use sixteen-byte stride and each page belongs to exactly one
reservation. At `24001EC2` the compiled call supplies the correct address,
size, `MEM_RESERVE=2000` and `PAGE_NOACCESS=1` to the verified VirtualAlloc
import. Commit calls use `MEM_COMMIT=1000` and `PAGE_READWRITE=4`.
No requested reservation overlaps the owned image at `24000000`.
No parameter/size/table bug was found; this static result does not identify
the runtime failure stage or allocation owner. Both PEs request initial stack and heap
reserves of one MiB, with 4-KiB commits. Those sizes are possible census
correlation inputs, not ownership proof.

`[MEASURED]` The owned live-breakpoint report
`target/source-ui/font-map-debug-run-v2/report.json` observes the first v2
reserve call with actual arguments `00890000/00010000/00002000/1`.
That debug child's requested range is MEM_FREE immediately before the call,
the actual return is `00890000`, and it becomes a 65,536-byte MEM_RESERVE
range. Its process heap is `00580000`; its stack is a separate allocation.
The diagnostic stops before source execution. It proves success of this
call in that child, rather than a conflicting owner. The earlier diagnostic
did not handle the WOW64 breakpoint status correctly and is retained as a
failed owned diagnostic, not source evidence.

`[MEASURED]` The three-way owned-breakpoint census
`target/source-ui/font-map-debug-run-v4/report.json` reaches `24002013`,
immediately before the first source call, without either mapping failure
block. Every reserve, page commit, copy and protection step therefore
finishes in that explicitly debug-created child. Its heap is `00580000`.
No source font entry executes in that diagnostic. The normal-child mapping
failures remain unresolved; no mapping correction or further source
substitution is justified from this result.

`[PREPARED]` The separate v5 continuation wrapper has been independently
read and statically reviewed in
`target/source-ui/font-debug-continuation-v5-review-v1.json`. It restores
and reads back all three owned breakpoint bytes and RX protections, resets
only the WOW64 EIP to the original first-call instruction, and checks the
remaining GPR/segment/EFLAGS/ESP fields are unchanged. Unexpected later
exceptions are passed to the fail-closed VEH. This review executes no host.

`[MEASURED]` The v5/v6 continuation attempts stop before source release on
that exact equality. V6 records only the intended EIP rewind and EFLAGS
`00000202 -> 00000200`; all GPR, segment and ESP fields remain identical.
Both reports retain stage 1 and `native_font_entry_started=false`, with
frozen files unchanged. The local SDK `winnt.h` lines 10025–10026 marks
WOW64 SegCs/EFlags as sanitized. Microsoft's
[SetThreadContext documentation](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setthreadcontext)
permits OS correction of some status fields, but does not specify this
particular readback. The
[Intel SDM Figure 3-8](https://www.intel.co.jp/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-1-manual.pdf#page=78)
shows reserved bit 1 as one. These sources do not prove the actual flags
after this context operation or justify a generic comparison mask.

`[PREPARED]` The independently reviewed v7 diagnostic replaces only the
owned `FF D0` call instruction with `9C CC`, executes PUSHFD then stops at
the following INT3. It requires ESP to decrease by four, all other
GPR/segments to remain intact, and the actual pushed flags to equal the
pre-operation baseline. It then restores the complete owned instruction
and RX protections and terminates without a source call. Its review is
`target/source-ui/font-debug-pre-source-v7-review-v1.json`; no source
continuation condition is relaxed by that diagnostic.

`[MEASURED]` V7 attempt 2 completes this owned-only diagnostic. Actual
PUSHFD records `00000202`, equal to the baseline; immediate WOW64 context
readback after SetThreadContext was `00000200`, while readback at the
second trap again reports `00000202`. The diagnostic verifies every other
GPR/segment, ESP-4, complete owned-byte/RX restoration and no source call.
The pinned report is
`target/source-ui/font-map-debug-run-v7-attempt2/report.json`, SHA-256
`2f57b69efc47c1fcc7d1a1626340ce9938bdee10d7a09303857a77a9e15ea7f9`.
The first attempt's explicit reservation failure remains separately
retained. This establishes the fixed-one bit's serialization discrepancy
for the tested startup condition; it supplies no generic context mask.

`[PREPARED]` The separately reviewed v8 continuation pins that actual
proof and requires `before.EFLAGS & 2` and
`after.EFLAGS | 2 == before.EFLAGS`, preserving every other flag exactly.
All other GPR/segment/ESP, EIP and owned-byte/protection checks remain
unchanged. An independent single-bit check rejects differences at all
other flag positions. Its review is
`target/source-ui/font-debug-continuation-v8-review-v1.json`. The source
entry marker explicitly records authorization rather than completed
execution; a real reply or native setjmp evidence must establish progress.

## Native output and state-preserving comparison

`[MEASURED]` The reviewed v8 continuation succeeds in
`target/source-ui/font-map-debug-run-v8`. It retains the original font and
scaler code, true FPGM/PREP/glyph execution, the explicitly documented EIF
adapter and owned stream/heap callbacks. The 2,073-byte reply SHA-256 is
`c193d17d1651abb845e8969baf0f106eb84ff734d5f80867075642f455e198c8`.
Glyph 0 produces width 7, height 7, pitch 7 and 49 active alpha bytes,
SHA-256 `9743b9addf12cd626b675657fa32b9814517f0f1260ba89caed0904ccac7d265`.
The defined metrics are E4=64, E0=640 (26.6) and D0=524288 (16.16).
All 63 source allocations are freed, with 2,511 guard checks, zero final
live allocations and 13 total setjmp calls. FPGM creates 53 definitions and
the native transform is 15 by 15 ppem, with CW `037F`, MXCSR `1F80` and DF 0.
The independent binary audit is
`target/source-ui/font-first-native-glyph-independent-audit-v1.json`.

`[MEASURED]` The generic runner preserves the reviewed continuation and
uses regular-file input, hidden owned children and at most 64 glyphs per
request. Its independent review is
`target/source-ui/font-glyph-debug-runner-review-v1.json`. The full cohort
in `target/source-ui/font-glyph-corpus-v1` completes all 80 cases, 4,624
glyph observations and 2,352 font contexts, with 7,154,800 guard checks.
Every glyph is tested under zero/A5 allocation poison and two padding
layouts. Fresh mode creates a context for each glyph; ordered mode retains
one context per ascending chunk of at most 64 glyphs. This is not one
578-glyph context. Fifty-seven bounded pre-source mapping retries are
retained; the successful observations contain no native failure. The
runner's original flat fresh/ordered equality assertion fails and is
preserved, including its frozen executor. It is not rewritten into a pass.

`[MEASURED]` Root's independent context-aware audit re-reads all replies,
active exports and 185 current pinned inputs. All 1,156 identical ordered
input groups match exactly across the four heap policies. Its semantic TSV
SHA-256 is
`bad091f7c103d0b1a7bc7e86ef11d24124122871098013bc5b12f04e1c4156be`;
the report is `native-context-audit-v1.json` within the cohort directory.
A second wire reader in `tools/native_font_glyph_state_audit.py` imports
neither that comparator nor the frozen reply parser. It independently
checks all 4,624 observations, the same TSV hash, all active exports and
the captured CVT/storage/function/flag state across heap policies. Its
report is `target/source-ui/font-glyph-hint-state-audit-v2.json`.

`[MEASURED]` Seven glyphs differ between fresh and ordered conditions:

| Glyph | Character | Contours | Fresh height | Ordered height | Initial vertical MIAP point using CVT 14 |
|---:|---|---:|---:|---:|---:|
| 354 | gamma U+03B3 | 1 | 12 | 13 | 3 |
| 357 | zeta U+03B6 | 1 | 15 | 16 | 12 |
| 364 | xi U+03BE | 1 | 15 | 16 | 15 |
| 366 | rho U+03C1 | 2 | 12 | 13 | 0 |
| 371 | phi U+03C6 | 2 | 12 | 13 | 12 |
| 373 | psi U+03C8 | 1 | 12 | 13 | 4 |
| 442 | Cyrillic er U+0440 | 2 | 12 | 13 | 1 |

All seven are simple outlines. Width, E4/E0 offsets and advance are equal
within each fresh/ordered pair; active pixels and height differ. Before
each program, CVT 14 is -192 in a fresh context and -256 in the ordered
context: a difference of one pixel in 26.6 units. The initial unbranched
push/delta/vertical-MIAP prefix explicitly consumes CVT 14 at the points
listed above. CVT 41 also differs between these contexts; storage,
post-glyph rounding function, CVT function and E0/E1 flags match.

`[MEASURED]` The original same-ppem guard at `8CC2DB..8CC2E3` returns from
PREP setup when the cached X and Y ppem are unchanged. Per-glyph setup
`8CC5C0`, specifically `8CC7B3..8CC7D2`, clears E0/E1 and copies thirteen
32-bit graphics-state words from `hint+3C` to `hint+70`. It does not copy or
restore the separately allocated CVT/storage arrays. Native MIAP handler
`8ACB90` pops the CVT index, calls the active CVT reader and then pops the
point index. WCVTP handler `8ADC60` writes the mutable CVT array. These
facts explain the persistence mechanism and establish different native
inputs to the seven programs. Glyph 342 first leaves CVT 14 at -256 in
the Greek chunk; the Cyrillic chunk has its own intervening writers.
No single-variable intervention or internal point trace has yet shown
that CVT 14 alone accounts for every output difference. No reset is added
to force context independence.

`[MEASURED]` A subsequent order-only intervention compares `[354]` with
`[342,354]`, retaining the same reviewed host, source bytes, style and all
four heap policies. All eight cases and twelve observations complete.
Gamma reproduces the complete-cohort variants exactly: 9 by 12 with semantic
hash `f0356709a7bfcc111a446676153af3beebf42a9c09f706b184728eb66ed2659e`,
or 9 by 13 with hash
`33e661bf7fab00a188192b0d859fa6692db3bea1bb03175b03a5e6a2b7ea181c`.
The only captured CVT-array difference before and after gamma is index 14,
-192 versus -256; CVT 41 no longer differs. Storage and the captured
rounding/CVT functions match. Raw E0/E1 differs before target setup: `0001`
after PREP versus zero after glyph 342. Both original clearing stores at
`8CC7BF/8CC7C6` execute before the nonempty target hint program, and both
post-gamma flag snapshots are zero. Thus those pre-setup snapshots must not
be described as entirely equal. The independent wire/pin/source audit is
`target/source-ui/font-cvt14-order-independent-audit-v1.json`; it verifies
41 current pinned files, complete cleanup and 20,804 guard checks. Three
pre-source mapping retries remain retained. The intervention establishes
that preceding glyph 342 is sufficient to produce gamma's ordered variant.
It is not a direct single-memory-variable patch or an exhaustive internal
point/state trace, and it does not establish one cause for all seven glyphs.

`[MEASURED]` Native byte-alpha coverage is preserved exactly. Across all
409,588 captured active bytes, the range is 0 through 126, with 125 distinct
values and no 255. The first glyph's 49 bytes comprise 29 zeros and 20
values of 126. The state audit retains the complete histogram. This
establishes the output range for this explicit mode-3 fixture, not a
general scaler normalization rule. The measured atlas primitive takes the
high alpha nibble; a replacement backend must not silently normalize these
native bytes to a modern engine's coverage scale.

`[UNKNOWN]` Live retail font selection, actual cache order/reset lifetime,
normal-child mapping behavior, shaping/layout and complete title rendering
remain separate gates. Native glyph pixels are now measured for one pinned
font/style, fifteen ppem, the admitted scaler flags and reviewed debug
startup. No Rust outline/hint/raster pixel comparison has yet passed.

`[VERIFIED]` The separate `spui_font_outline` module reads ordinary glyph
packets as original unscaled points, contour endpoints, expanded raw flags,
borrowed instruction bytes and raw component descriptors. It preserves
the source zero-contour early return, signed instruction/point-count
limits, 16-bit coordinate accumulation and last-component instruction bit.
It does not flatten composites, append phantom points, execute glyph hints
or rasterize. Its explicit malformed-repeat guard is not claimed equivalent
to the native clamp. `tools/spui_font_outline_probe.py` checks every packet
of all five installed faces against FontTools 4.60.1: 51,680 glyphs,
2,888,616 simple points, 28,483 components and 13,084,181 original glyph
instruction bytes. The fresh canonical Rust export matches all 64,705,863
reference bytes, SHA-256
`d3e152068cc845caba0a62bc0a43af65b9e7049916507c90d1d52d53156d2c8e`;
root's five focused tests also pass. The result is
`target/source-ui/font-outline-rust-comparison-v1.json`. This is packet data
validation, separate from native scaled-outline or pixel conformance.
Native phantom/scale contracts and the next unexported helper port are
recorded in [spui-font-outline-native.md](spui-font-outline-native.md).

Evidence: native v3 queries 472, 473, 479, 480, 482, 531–535,
537–541, 548–552, 555, 556, 559–567, exact-address items from 570–574/577,
the 113 admitted strict dossiers from 578–749, 753–759 and 769–776,
plus the bounded SDK font extracts named
in [spui-font-native.md](spui-font-native.md). Additional pinned-byte
extracts and analysis reports remain under `target/source-ui/`; no source
font, coefficient table, native function body or bitmap is embedded here.

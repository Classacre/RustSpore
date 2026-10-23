# Original Cell capability property curves

Scope: pinned original executable SHA256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`,
REA v3 queries 393 / 449 / 489, 2026-10-23. This recovers the bounded numeric
property-curve evaluator, before live property-manager/cache/actor integration.

## Original source and consumers

`[MEASURED]` `E837F0` obtains property-list instance `C951EB63` with manager
scope zero, then asks for the caller's property identifier. The all-82-package
index census finds exactly two installed copies of that PROP resource: one in
`Data/PatchData.package`, one in `Data/Spore_Game.package`, both package group
`0001FFFF`, both 1,124 bytes and byte-identical SHA256
`c00576f276d3e3daa9f898985775ba13e173d66b492bc8e8fcf0b7b3b25a6462`.
The manager's scope argument is not relabeled as the on-disk package group.

Each source copy has **18 named Vector2 arrays**, type `0030` / specifier
`009C`, containing **104 finite points** with strictly increasing x. The two
copies therefore represent 36 array copies / 208 point copies. Source values
and complete property census stay ignored in
`target/source-cell/capability-curves-source-v1/`; no tuning values are invented
from property names or visual appearance.

`E51EC0` consumes several of these curves while constructing the native
capability cache. For example, its output word `1258` selects property
`2EF68471`, `BBAB1BF5` or `2C574746` from source totals and globals factors.
The empty propulsion-mask branch instead reads globals word `90`. Other
observed curve consumers include `77540C34`, `E5015D0C`, `03AB589E`,
`AEE2835E` and `92BAAE19`. Their native field offsets are retained without
assigning fabricated part/stat names. Complete cache construction remains a
separate acceptance gate.

## Exact numeric rule

`[MEASURED]` Missing lists and empty native property values return literal
positive zero. The first-point test is **strict**: `first_x > query` copies
first y. Otherwise the evaluator walks the points in their original order and
selects the first right point for which `query <= right_x`. It does not sort the
array or add a generic equality-based endpoint shortcut. If that search finds
no right point, it copies last y; a one-point value consequently copies that y.

For a selected pair, `1041E40` retains these operations in x87:

```text
numerator   = query - left_x
denominator = (right_x - left_x) + 1/65536
fraction    = numerator / denominator
```

The denominator epsilon is the exact native word `37800000` at `149A808`.
The ratio is committed to binary32 once. `E83923` / `E83927` / `E8392D` then
perform separate scalar SSE subtraction, multiplication and addition:
`(right_y-left_y) * committed_fraction + left_y`. There is no fused multiply-add,
generic clamp, or retained-x87 output interpolation. At exact final x, this
epsilon-adjusted interpolation can differ from last y. A strict above-last
sample takes the direct-copy route. Signed-zero behavior at exact first x can
also differ from a copy.

`spore-core::cell_capability_curve::sample_capability_curve` takes explicit
`Option<&[[f32;2]]>`, query and x87 `Context`. `None` / empty returns native
positive zero without reading an unused query. Only consumed point components
must be finite. Duplicate/nonmonotonic x values retain native sequential search
where the finite arithmetic domain permits them. Checked errors bound the Rust
API; they do not claim the original game has the same error handling.

All comparisons below explicitly use precision 24/53/64 and all four x87
rounding modes, with scalar SSE MXCSR control `1F80` (nearest rounding,
subnormals preserved). The retail render path's PC24/nearest and FTZ/DAZ setup
is documented separately in
[native-float-render-context.md](native-float-render-context.md); its binding
to each Cell entry/thread remains a separate gate. This helper does not change
the game's global floating state.

## Original-byte conformance host

`[MEASURED]` The host copies only original code `E837F0..E8393C` and
`1041E40..1041E57`, ends exclusive, plus the four-byte epsilon at `149A808`.
The property-manager dependency is a bounded checked provider, not game code:
the six-byte owned stub at `67D950` returns an immutable host manager object.
Read-only vtables expose only checked list lookup, property lookup and release
callbacks. They validate the exact native reference stack-slot address, requested
instance/scope/property ID, object identities and one lookup/get/release count.
The only possible point pointers are the bounded input rows or the fixed
20-byte inline property object. Input supplies no pointers or vtable entries.

The input is **160 bytes**: property ID, query, count, CW, source-provider mode
and sixteen Vector2 point rows. Array counts are 0..16; inline counts 0..1;
missing-list count is zero. The reply is **16 bytes**: native result word plus
manager-lookup, property-get and release counts. Six buffer boundaries are
guarded. ReadFile errors are retained; broken-pipe EOF is accepted only at a
fresh record; partial records fail. The seven Kernel32 imports are the same
bounded I/O/allocation/cache/exit helpers as the prior Cell math host.

Host v1/v2 are retained as **invalid setup-only evidence**. V1 exited with a
setup guard before reading records. V2's fixed diagnostic exits identified an
exact allocation collision at page `00670000`. No curve rows from those runs
are accepted. V3 moves only the owned host image to `00670000` and compiles the
owned provider at `.text` offset `C950`, verified by the linked symbol to equal
`0067D950`. Its actual bytes are `B8 7CE06700 C3`, an absolute manager pointer
and RET. The section is RX / not writable; there are no outbound relative
provider transfers. No original instruction, data address, constant, native
span or callback contract changed, and no native relocation was performed.

Reviewed v3 host EXE SHA256:
`977cf6d559ab82332d664a074af381b2004576ec252a1d20abd7eaad58834b37`.
Assembly SHA256:
`94efb90a064529f7465fce9c3ebff632b06831a4c0c734dbd7cbc0e676871122`.
Provider stub SHA256:
`742012cd7cc50156caa9a8dd3a3462ebcd2a796dc697f5da88e844a795c98883`.
The complete manifest, linked-section audit and small v2-to-v3 diff are in
`target/source-cell/native-capability-curve-host-v3/`.

## Measured corpus and limits

`[MEASURED]` The reviewed hidden/piped v3 host passes empty input and the exact
two-point smoke, then returns **107,208 complete replies**, **8,934 per each of
twelve explicit CW policies**, with exit zero, zero stderr and all guards and
provider counters intact. It passes **11 protocol checks**: one valid empty
stream and ten malformed/partial/invalid-mode, count, control or
nonfinite-consumed-query rejections.

Of those cases, **94,440 use the actual source Vector2 arrays** from both PROP
copies; **12,768 are explicitly synthetic**. Source queries cover knots, adjacent
binary32 values, interval quarters/midpoints, integer count inputs and seeded
random values. Synthetic coverage includes missing/empty/inline/array providers,
duplicate/nonmonotonic points, signed zeros, subnormals and large coordinates.
All **107,208 outputs are finite**, with **zero output exclusions**. There are
108 default-path cases, including 36 unused NaN queries which the native code
does not read. There are zero nonfinite consumed queries or active point values;
consumed NaN queries are separately rejected by the host's finite input domain.

Retained in `target/source-cell/capability-curve-fixture-v3/`:

| File | SHA256 |
|---|---|
| input.bin | `d2b9cabaa67bde3bfb207f48adfb2175e19c7f948f46f4a286c3bbafdc73c5b1` |
| output.bin | `2f8e0fab66d035f1c595abb2cce0e1bf393d54da5940dbc0a65c7db245b58896` |
| source-cases.tsv | `3d3c3a3f4c0a4152ea9fa3ff07f818a89290dd5cb5dca37963413fe665ccb895` |

`[MEASURED]` The independent Rust example passes **all 107,208 native records**
with exact result bits across all twelve contexts; its two focused unit tests
also pass. The example independently decodes both original PROP payloads with
Rust's property parser and passes **all 94,440 real-property cases**. Its
`rust-source-v1.tsv` is byte-identical to `source-cases.tsv` at the hash above.
The native result-word stream and `rust-values-v1.bin` are both 428,832 bytes,
SHA256 `a3d1ce0ebfdc9546fce6c4318c70f537560ae3741c693bfaa62798b23c15428e`.
`rust-comparison.json` retains the exact counts and independent hash comparison.

Property-manager lookup,
complete capability-cache construction, actor controllers and live Cell stage
integration remain unimplemented in this module. No retail process,
initializer, GUI or network was run by these hosts.

## Reproduction

```powershell
python tools/cell_capability_curve_probe.py 'D:\Games\SPORE Collection\SporeBin\SporeApp.exe' --sdk-bin-x86 'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86' --kernel32-lib 'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.18362.0\um\x86\kernel32.lib' --out target/source-cell/curve-host-new
# Review the fresh manifest and immutable linked provider before execution.
python tools/cell_capability_curve_run.py 'D:\Games\SPORE Collection' --host-dir target/source-cell/curve-host-new --reviewed-host-sha256 <reviewed-SHA256> --out target/source-cell/curve-fixture-new
.\tools\build.bat run -p spore-core --example cell_capability_curve_verify -- target/source-cell/curve-fixture-new target/source-cell/curve-fixture-new/rust-source.tsv target/source-cell/curve-fixture-new/rust-values.bin
```

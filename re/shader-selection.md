# Source shader fragment selection

The Rust decoder preserves source selectors as 25 bytes. The new
`spore_core::shader_selection` evaluator applies the eight condition codes
observed in the installed builders. This is a recovered selection contract;
actual draw bindings and rendered material fidelity remain open.

## Native and source evidence

`[MEASURED]` Native reader `6FC9C0` writes selector fields into 28-byte runtime
records. Native consumer `6FC780` evaluates those records in order. Vertex and
pixel wrappers `6FDF60` and `6F9D60` call the same consumer. Bounded instructions
in Evidence query 257 corroborate the branch conditions from query 254.
The pointer-table setter `777990` confirms indexed shader data and dirtiness.
Original native output stays under ignored
`target/rea/20261022-anim-rig-finalize-v2/`.

| Packed offset | Width | Role at the consumer |
|---|---:|---|
| 0 | 1 | Condition code |
| 1, 3, 5 | 2 each, BE | Three condition arguments |
| 7 | 4, BE | Required current declaration feature bits |
| 11 | 4, BE | Required accumulated bits |
| 15 | 4, BE | Forbidden accumulated bits |
| 19 | 4, BE | Bits added on acceptance |
| 23 | 1, signed | Threshold |
| 24 | 1 | Base fragment ID |

`[MEASURED]` All required declaration/accumulated bits must be set; any forbidden
accumulated bit rejects the selector. A negative threshold bypasses its test.
Otherwise shader-data `237`, when present, supplies a signed byte which must be
less than or equal to the selector threshold. An absent pointer permits only
threshold zero. These are native rules, not user-selected quality defaults.

## Conditions present in this install

`[MEASURED]` Across 12 source packs there are 15,469 selectors with codes
`0, 1, 4, 5, 7, 9, 10, 12`. Their counts are respectively
9,453; 3,249; 318; 600; 1,051; 96; 528; 174. Threshold bytes are
`FF` in 15,409 selectors and `00` in 60.

| Code | Acceptance and emitted offset |
|---|---|
| 0 | Accept, offset zero |
| 1 | Shader data at argument 0 exists, offset zero |
| 4 | Data at argument 0 exists; offset is its first byte |
| 5 | Data at arguments 0 and 1 both exist, offset zero |
| 7 | First byte at argument 0 equals argument 1, offset zero |
| 9 | Data at arguments 0 and 1 both exist; offset is argument 0's first byte |
| 10 | Both native sampler pointer and availability flag at argument 0 are set |
| 12 | Little-endian u16 at shader-data `24E`, masked by argument 0, equals argument 1 |

`[MEASURED]` Accepted selectors add their accumulated bits even if their base
fragment ID is zero. Nonzero bases emit a byte using wrapping addition with the
offset. Zero in the emitted list retains its native terminator meaning. The
Rust path limits output to 32 bytes and rejects unsupported conditions, short
data and missing mandatory data instead of inventing bindings.

`[MEASURED]` Other native conditions include 2, 6 and 11; none occurs in the
measured installed builders. They remain explicitly unsupported in this evaluator.
Condition 11's native count also depends on declaration records and declaration
features. Knowing that helper exists does not establish the caller's declaration.

## Current declaration feature word

`[MEASURED]` The first mask is not a quality or general rendering-option mask.
Selector consumer `6FC780` reads the current vertex-declaration pointer at
`16F75A0`, then its feature word at offset `10`. Native declaration finalizer
`11F2FD0` constructs it from each element's saved semantic index through table
`14F7EB8`. Entries 0 through 23 have exactly bit `1 << semantic`; the next
entry is END, not another vertex attribute. `11F29B0` independently consumes
the same element list to create the D3D9 declaration.

`VertexFormat::native_shader_features` derives those bits and rejects unknown
or END semantic IDs. `SelectionContext::declaration_features` explicitly takes
the currently bound declaration's word. Later runtime declaration changes must
be reflected by the caller; a saved declaration is not proof of every animated
draw's final declaration.

Jawhead's saved declaration has semantics `0, 2, 6, 19, 14, 15`, giving
`0008C045`. Its normal, tangent, blend indices and weights therefore participate
in shader selection through original data. This corrects the earlier unknown
renderer-mask interpretation. Native Evidence is under
`target/rea/20261022-source-runtime-v3/` queries 68, 72, 75, 78 and 80.

`[MEASURED]` Rust and independent Python agree on all 13,269 declarations and
51,437 elements in the full 11,041 GMDL population, across all 82 package indexes.
Observed semantic IDs are `0, 2, 6, 7, 14, 15, 19`; none needs an unknown-table
fallback. Sorted LF SHA-256 is
`3b79a2de191617b5c75a41b8aff281272bc31929c7b50196ae1e841fea3c408a`.
Reproduce with `vertex_declaration_verify` and `tools/vertex_declaration_probe.py`;
the report is `target/source-shaders/declaration-features-comparison-v1.json`.

## Fragment identity and the Cell material byte

`[MEASURED]` Vertex library reader `6FF150` starts one runtime entry beyond its
array base. Vertex code/uniform consumers `6FE4D0`/`6FE400` use the byte ID directly,
without decrementing it. Source vertex storage slot 0 therefore has runtime ID 1.
Pixel reader `6FC370` and consumer query 223 use the same base, so source pixel
slot 0 has runtime ID 0. `ShaderFragments::vertex_by_id` and `pixel_by_id` expose
these distinct mappings. Zero also terminates recipe lists, which constrains
runtime use of pixel entry zero.

`[MEASURED]` Every one of 17,730 GMDL `210` payloads has 20 bytes and a first
byte in `1,2,4,5,7`. The next three bytes happen to be `5C 44 26` throughout the
measured population. The native predicate consumes byte zero; interpreting the
entire first word `26445C04` as a shader ID was an unproven hypothesis.

The generic-skinning pixel builder contains condition 4 with argument `210` and
base 13. Jawhead's byte 4 consequently emits pixel fragment ID 17, whose source
name is `setupSHCellEditors`. This proves that branch when those selector inputs
are bound; it does not prove the complete native draw selected that builder.
The source fragment uniform itself is one float4 with metadata ID `210`.
Its native upload callback and compatibility marker are now recovered in
`material-links.md`; actual draw-time lighting and shader choice remain open.

## Compiled program lookup

`[MEASURED]` Native `6FD650` compares zero-terminated fragment byte sequences.
Unused bytes after the first zero do not participate in the key. Cache builder
`6FE6E0` retains all 32 saved bytes, and constructs the program dependency mask
by OR-ing each uniform descriptor's flags. The Rust `CompiledTable::lookup`
implements that key, rejects ambiguous entries and never substitutes another
program. Original padding remains available for exact round trips.

`[MEASURED]` All 14,355 compiled programs across the twelve installed packs have
terminated recipes, unique within their stage table, and matching uniform masks.
The Rust verifier resolves every source recipe to that same program. Independent
Python validation agrees, and the sorted per-resource digest remains
`1595eb958a876132e60b6ed09f64f9c064e188e9b6663588ac4a7ace2a5b38d6`.
Evidence is in `target/source-shaders/rust-pack-digests-v3.tsv` and
`python-pack-corpus-v3.json`.

`[MEASURED]` Native vertex/pixel upload dispatches `6FDDD0` and `6F9C50` consume
the saved register words as constant-register starts. They select a callback
using the first descriptor u16, use the second u16 for per-shader-data dirtiness,
and pass the third u16 as register count. The callbacks and source buffers
needed for the controlled static Cell draw are now recovered and exercised;
see `shader-execution.md` and `cell-lighting.md`. Additional runtime paths
still require their own source-buffer and frame-binding evidence.

## Checks and remaining bindings

```powershell
tools/build.bat test -p spore-core shader_
```

`[MEASURED]` Focused shader checks include the five selector tests and native
uniform-dispatch tests.
They check the recovered Cell byte branch while independently varying its separate marker bytes,
ordered include/exclude flags, signed and absent threshold inputs, actual byte
versus word comparisons, zero-size presence, sampler availability, mandatory
data errors, output capacity and wrapping terminators. Fragment tests also
check the native vertex/pixel ID mappings.

`[UNKNOWN]` Still needed: each retail frame's renderer-feature mask and material
overrides, dynamic generated GMDL material registration and frame-dependent
constant buffers beyond the supported inspection context. Source declarations,
samplers and constants now execute all 28 resolved primary Cell models in that
explicit context. Bounds fitting and white tint/material controls remain
inspection inputs; they are not asserted as retail defaults.

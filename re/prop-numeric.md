# PROP numeric and float-color corrections

Measured on 2026-10-22 against the local `D:/Games/SPORE Collection` install.
The source data and generated fixtures remain in ignored `target/` directories.
This report covers primitive values and the later float-color correction. It
does not certify every PROP type or gameplay meaning.

## Float-color correction

`[MEASURED]` RGB (`32`) stores three little-endian float components; RGBA (`34`)
stores four. The earlier typed decoder returned the first three/four individual
bytes. That could pass size checks while destroying signed/HDR lighting data.
Both `PropValue` variants now preserve source floats. No clamp, gamma conversion
or conversion to display bytes is performed during parsing.

Native RGB getter `6A0C50` copies three words from type `32`, and native lighting
reader `70A820` consumes those components and color arrays as floats. The Cell
editor's saved lighting contains negative and above-one coefficient values.
Independent Python reads their LE component bits directly from the complete
named lists, separately from the Rust typed decoder.

`[MEASURED]` Independent comparison passes across all 30,885 previously certified
named PROP lists: 4,404 color fields, 28,161 colors and 85,210 components. These
include 27,434 RGB values and 727 RGBA values, with 2,916 negative components and
1,321 above one. The four framing exclusions below remain excluded. The old
primitive fixture digest was rechecked before deriving each new color record.

The new local fixture SHA-256 is
`6aba0582d4733ce387a023ad7054814113801abdc7238cc6caf20e1cd56ba85a`.
The synthetic regressions cover signed/HDR RGBA, RGB arrays, negative zero,
NaN payload bits and an adjacent primitive property with different endianness.

```powershell
python tools/prop_color_probe.py target/source-prop-numeric/corpus.bin --out target/source-prop-colors-new
tools/build.bat build -p spore-bevy --example prop_color_verify
target/debug/examples/prop_color_verify.exe target/source-prop-colors-new/corpus.bin
```

Evidence is in `target/source-prop-colors-v1/`; native query records 92 and 110
are under `target/rea/20261022-source-runtime-v3/`.

## Contract and native corroboration

`[MEASURED]` PROP types `0x0009` (INT32), `0x000A` (UINT32), and `0x000D`
(FLOAT) are four-byte **big-endian** values, for both singles and array elements.
Reading these three types as little-endian preserved framing but changed values.
The Rust decoder now uses BE32 only for these types. UTF-16 characters, resource
key words and compound vector components keep their existing little-endian/raw
byte treatment; a blanket endianness reversal would be wrong.

The base executable's native reader and writer corroborate that boundary:

| Procedure | Observed role | Retained query |
|---|---|---|
| `006A2A10` | PropertyList reads count/identifiers and dispatches to Variant reader | `113-analyze_function.json` |
| `00693EC0` | Variant types 9/A/D use the 32-bit typed stream helper; compound values take raw byte reads | `115-analyze_function.json` |
| `00571DD0` | Thin count-one wrapper around the 32-bit reader | `119-read_bytes.json` |
| `0093A3F0` | Reads four-byte words and reverses their bytes in the PropertyList stream mode | `116-procedure_pseudo_code.json` |
| `0093A370` | Corresponding two-byte framing reader | `121-procedure_pseudo_code.json` |
| `00692C90` | Symmetric typed Variant writer | `114-analyze_function.json` |
| `0093A6E0` | Symmetric byte-swapping four-byte writer | `117-procedure_pseudo_code.json` |

Queries are retained under `target/rea/20261022-anim-rig-finalize-v2/`.
The reader and writer contracts were measured; no EA implementation was copied.
The decompiler's array switch uses an offset discriminator, so its displayed case
numbers must not be mistaken for the on-disk type codes.

## Independent corpus check

`tools/prop_numeric_probe.py` independently indexes all 82 installed packages,
retains named identifiers, enforces the declared property count, and requires
exact consumption. Primitive singles have size four; the walk does not recover
anonymous entries. For each accepted resource it hashes the identifier, type,
specifier, element count and every decoded numeric bit pattern. The Rust ignored
corpus test independently parses each original blob and compares that digest.
FLOAT values are checked by their bits, preserving negative zero and NaN payloads.

| Scope | Result |
|---|---:|
| Package indexes read / excluded | 82 / 0 |
| PROP resources inventoried | 30,889 |
| Exact named lists checked by both implementations | 30,885 |
| Explicit framing exclusions | 4 |
| Primitive single fields | 97,226 |
| Primitive array fields | 18,619 |
| FLOAT values | 614,892 |
| INT32 values | 62,624 |
| UINT32 values | 170,319 |
| Total bit patterns checked | 847,835 |

The generated `corpus.bin` is 34,630,218 bytes; SHA-256:
`9c86f5f3b36dd93e55cee12a24f1d90abc5bc5d190ebe1a811539e5714cb76f8`.
`target/source-prop-numeric/inventory.json` retains all identities, digests and
exclusions. This is broader than the historical six-package/23,762-resource walk.

The focused suite passed **18/18** tests, including this installed corpus, real
camera bytes, signed/unsigned sentinels, mixed compound/key endianness, both array
markers, negative zero and NaN bit preservation.

```powershell
python tools/prop_numeric_probe.py "D:/Games/SPORE Collection" --out target/source-prop-numeric --fixtures target/source-prop-numeric/corpus.bin
$env:RUSTSPORE_PROP_NUMERIC_FIXTURES='D:/AI/RustSpore/target/source-prop-numeric/corpus.bin'
.\tools\build.bat test -p spore-core prop::tests -- --nocapture --include-ignored
```

## Concrete camera check

The source prop `4041010002AD26C9`, labelled `EditorCameraCellThumbnail`, appears
in both PatchData and Spore_Graphics. Three FLOAT records retain these exact
identifiers and bytes:

| Identifier | Wire bytes | BE32 value |
|---|---|---:|
| `00C7C4FB` | `3F 80 00 00` | 1.0 |
| `00C7C4FC` | `C2 B4 00 00` | -90.0 |
| `00C7C4FD` | `C2 B4 00 00` | -90.0 |

Their native field names, coordinate conventions and camera application remain
open. Numeric plausibility alone does not establish yaw, pitch, units or order.

## Withdrawn assumptions and remaining limits

The old `tools/prop_v5.py` size-only walk treated four-byte primitive singles as
eight bytes, then permitted anonymous recovery. That combination could discard a
real next identifier and still reach EOF. Its earlier “100% decoded” claim did
not establish typed correctness. The tool now uses four-byte primitive singles;
its old aggregate pass rate is historical, not recertified evidence.

Four explicit exclusions remain in the independent named/count-bounded walk:

| Package | Group:instance | Observed limit |
|---|---|---|
| `bp1content/BoosterPack_01.package` | `00010000:40200100E1B8C21C` | Named/count framing unresolved |
| `bp1content/BoosterPack_01.package` | `00010000:40200100FDDDF5ED` | Named/count framing unresolved |
| `Data/PatchData.package` | `0001FFFF:02AE0C7E521D0174` | Separator heuristic collides with a named identifier |
| `Data/Spore_Game.package` | `0001FFFF:02AE0C7E521D0174` | Same duplicate source record |

The last two begin with real FLOAT identifier `0195A000`, whose low word matches
the old separator predicate. The raw first value `44 96 00 00` decodes to 1200.0.
This demonstrates that a low-word pattern alone is insufficient proof of a
separator. The Rust compatibility framing rules have not been redesigned in this
numeric correction. Anonymous/padding heuristics, nonprimitive typed semantics,
and the two BoosterPack records need separate native and count-aware validation.

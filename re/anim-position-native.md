# Original ANIM position-curve conformance

Scope: base executable SHA256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`,
REA v3 queries 424 / 428 / 429, 2026-10-23. This establishes bounded native
INFO/POS arithmetic, before selector/rig-space conversion or complete playback.

## Source chain and payload

`[MEASURED]` `99F6A0` evaluates channel components using the upper key supplied
by its caller. `99C6C0` derives previous/left/right/following row indices from
channel count `D4` and the header `118` bit `8`. That neighbor-wrapping bit is
separate from a guessed playback-loop policy. The caller's upper is the first
strictly greater key; the full native selector is outside this host's allowlist.
At endpoints, equal left/right rows enter `99F630`'s native copy route.

`[MEASURED]` INFO is 20 bytes: unsigned frame plus four unclassified words.
POS is **80 bytes**, consisting of four value floats (XYZ and weight), followed
by four 16-byte controls. Each control retains previous/next slope floats,
previous/next mode bytes at control `8/9`, and six other raw bytes. Mode
classification consumes only the low two bits. A descriptor is 32 bytes:
flags, four-byte tag, row offset, stride, slot, and three retained words.

The native size table at `15514CC` starts `[20,80,100,40]`; slot-table offsets
at `14477D8` / `14477F8` are zero / 20. Consequently endpoint copying preserves
the **whole 80-byte POS payload**, not merely its first 16 value bytes. The
bounded host uses INFO offset 0 and POS offset 20 in interleaved 100-byte rows.

## Arithmetic and explicit context

`[MEASURED]` The position path constructs its cubic basis from the supplied
frame and INFO times. Unsigned times use exact `FILD` of the signed word,
followed by conditional x87 addition of `2^32`; replacing every conversion with
an early binary32 cast changes some contexts. Source duration-minus-one and
wrapped spans remain distinct arithmetic inputs.

The four tangent modes are recovered independently for each XYZ/weight scalar:

| Low mode bits | Source operation |
|---|---|
| 0 | Auto tangent from the adjacent values and native time spans |
| 1 | Current segment value difference |
| 2 | Zero tangent |
| 3 | Authored slope multiplied by the segment span |

The auto tangents are asymmetric. The left tangent uses separate scalar SSE
subtraction/addition/division/multiplication and binary32 rounding. The right
tangent retains its x87 spans and arithmetic until its commit. Endpoint product
addition also has a different source operand order for weight. Interpolated
weight clamps to `[0,1]`; a directly copied endpoint weight preserves its source
value, including values outside that interval. The source operations must not
be replaced by one generic cubic helper with different commit boundaries.

`spore-core::anim_curve` uses explicit x87 precision/rounding contexts and the
recovered binary32 operation order. Retail floating-point setup remains a
separate evidence gate. Precision 24/53/64 and all four rounding modes are
covered; SSE uses declared MXCSR `1F80`. No default retail control word, time
rate, rig transform or unobserved selector rule is inferred from this corpus.

## Strict original-instruction host

The reviewed host copies only unmodified native fragments, with ends exclusive:

| Start | End | Scope |
|---|---|---|
| `99F6A0` | `99F8B6` | Entry, neighbors, basis and component dispatch |
| `9A0499` | `9A0974` | POS branch, shared iteration and return |
| `99C6C0` | `99C772` | Native neighbor helper |
| `99F630` | `99F697` | Native component-copy helper |

`[MEASURED]` Deformation/quaternion bytes `99F8B6..9A0499` are not copied;
their pages remain `INT3`-filled. Read-only descriptors are fixed INFO
`flags=0,tag=INFO,offset=0,stride=100,slot=0` and POS
`flags=11,tag=POS\0,offset=20,stride=100,slot=1`. INFO skips at `99F892`.
Enabled POS reaches the preceding type-1 branch at `99F89E`, directly entering
`9A0499`. The later type-2 branch and type-3 fallthrough are unreachable for
every accepted protocol input. Unsupported component math is not approximated.

The sole replacement, at memcpy thunk `11E0534`, checks an exact INFO20 or
POS80 destination, source-row alignment and active row bounds, then performs a
byte copy. Its actual copied machine code was audited: all nine relative
branches stay inside its 103-byte span; the only outbound transfer loads an
absolute failure address. The first prepared host was corrected before any
execution because an outbound relative failure branch would relocate wrongly.

Host inputs are bounded, descriptors read-only, native calls stack-aligned,
and output/input buffers guarded. The seven imported Kernel32 functions are
`GetStdHandle`, `ReadFile`, `WriteFile`, `VirtualAlloc`, `FlushInstructionCache`,
`GetLastError`, `ExitProcess`. Hidden piped execution runs no game initializer,
window, network or retail process. Code, data, generated assembly and host binary
remain ignored under `target/animation/`.

Reviewed host v3 SHA256:
`d4b2becc97dc710bcba333f751c041a084416a47ec54598ab43820301a906723`.
Manifest and machine-code audit:
`target/animation/native-position-host-v3/manifest.json`.

## Protocol and measured comparison

Each request is **1,632 bytes**: frame float at 0; unsigned upper/count at
4/8; duration float at 12; header flags at 16; x87 control word at 20;
reserved bytes through 31; then sixteen 100-byte INFO20+POS80 rows. Count is
1..16 and upper cannot exceed count. Frame/duration must be finite; only the
twelve masked explicit x87 contexts are admitted.

Each reply is **124 bytes**: native INFO20+POS80 output at 0, exact-key signed
index at 100, native boolean at 104, and four neighbor indices at 108..120.
The native output starts with explicit `A5` bytes. Interpolation writes its
value fields and INFO; untouched control payload stays supplied initial state.
Endpoint copying replaces the complete 100-byte output with the selected row.

`[MEASURED]` The synthetic native fixture contains **32,256 cases**,
**2,688 for each of twelve explicit x87 contexts**. All 32,256 returned
XYZ/weight tuples are finite; **zero outputs were excluded**. Counts 1/2/4/16,
wrapped/unwrapped neighbors, all sixteen mode pairs on every XYZ/weight scalar,
mixed per-axis modes, asymmetric spans, duplicate times, signed zeros and
unsigned times above `2^24` and `2^31` are represented. Those high-time families
remain exactly representable in every explicit context. Every supplied upper
is the first source frame strictly greater than the sample frame.

**All 32,256 Rust XYZ/weight and neighbor comparisons match exact bits.**
The native host separately verified **9,024 full 100-byte endpoint copies**.
All guards and exact reply counts passed with zero exit status. Ten negative
protocol cases verify empty/partial input, partial records after a full record,
invalid count/upper/control and a nonfinite frame.

Retained flat streams in `target/animation/native-position-fixture-v1/`:

| File | Bytes | SHA256 |
|---|---:|---|
| input.bin | 52,641,792 | `38228b9e3f29a0612483987a6ee58db45c6d518beddba5305a66f94c5cddc0cc` |
| output.bin | 3,999,744 | `fa0b0ed7b579af80c9c6c84ef32942c643663a15ab1e1d2be281eb82b5833196` |

`inventory.json` and `protocol-checks.json` retain scope/denominators;
`rust-v1.log` / `rust-v1.tsv` retain the independent Rust comparison.
This fixture establishes synthetic bounded arithmetic conformance. The separate
authored-source sweep below preserves the full source clip for Rust selection.

## Authored-source census and native fixture

`[MEASURED]` `tools/anim_position_source_probe.py` reads all **82 installed
package indexes** and **5,022 ANIM resource copies** (300,983,784 decoded bytes).
The native initial INFO/POS/ROT preparation rule is applied to transform
descriptors independently of deformation layouts. The census finds 48,700
channels, including **39,633 matched POS channels / 731,075 raw source rows**.
The other channels are explicitly counted: 238 empty and 8,829 without a
matched POS descriptor. There are no index, ANIM or channel parse failures.

There are 7,466 matched POS channels with interpolation enable clear. Native
copying tests the prepared slot, so their valid endpoint copies remain in the
fixture; 116,334 proposed interpolation samples are excluded with the named
`disabled_interpolation` reason. No generic interpolation is substituted for
those samples. `limitations.tsv` retains one row for each named channel/sample
limitation. All 691,426 distinct source time intervals are representable in
binary32; enabled interval samples contribute 582,122 midpoint cases.

Each eligible channel supplies a strictly-before-first sample and a sample
at/after its final key, plus its exact first key when representable and an
interior midpoint of every distinct enabled interval. Sample bits are deduped
within the channel. **Exactly the first key time is selected by the original
strict-greater rule and can interpolate**; it is not forced into a copy path.
Duplicate source times remain present during upper/neighbor selection.

The native fixture has **692,882 requests**, covering all 39,633 matched POS
channels in 5,010 source resource copies:

| Case kind | Requests |
|---|---:|
| Strictly before the first key | 39,633 |
| At or after the final key | 39,633 |
| Exact first-key time, after deduplication | 31,494 |
| Enabled interval midpoint | 582,122 |

For each request the generator first selects all four indices from the
**original full source channel**. If left equals right, the host receives one
unaltered INFO20+POS80 source row. Otherwise it receives the actual previous,
left, right and following source rows in four local rows and explicit upper=2.
The raw row payload, unsigned time, duration and header flags are retained.
This fixed selected-neighbor protocol intentionally bypasses the native full
clip selector. On a one-row copy the native local indices are all zero; the
source previous/following indices remain independent metadata and are unused
by that copy. Rust must select the original full-channel neighbors itself.

`[MEASURED]` The unchanged reviewed host-v3 returned exactly **692,882 replies**
with exit zero, zero stderr and all buffer guards intact. All **79,266 full
100-byte copies** and **613,616 interpolation store extents** were checked.
Every local native neighbor tuple and exact-key result also matched its
protocol inputs. File-backed stdin/stdout keep the 1.13-GB input stream out of
RAM; no retail process, initializer, visible window or network was run.

The source sweep explicitly uses **x87 CW027F (53-bit, nearest-even)** and
MXCSR1F80. It does not establish the retail process's floating-point policy.
**692,878 cases have finite consumed source scalars and finite XYZ/weight
outputs**. Four endpoint copies contain source XYZ NaNs (`FFC00000`) and weight
zero, from `Spore_Game.package` resources `00010000:000000003F376548` and
`0001FFFF:00000000A25EC4C2`, channel 10 in each, at both boundaries. Their
complete raw requests/replies remain present; `excluded.tsv` records their
original ordinals and both nonfinite-input/output reasons. Unused slope values
and opaque control bytes do not spuriously narrow the finite API domain.

The finite `expected.tsv` has exactly sixteen columns: package, type, group,
instance, channel, frame bits, CW, four output words, four **original source**
neighbor indices, and the zero-based original native request ordinal. Package
paths are installation-relative, resource keys and float words are hexadecimal,
and indices/ordinals are decimal. `case-plan.tsv` separately preserves the
local row geometry, source upper, sample kind and consumed-nonfinite fields.
All resource hashes are rechecked against the complete census before execution.

Retained in `target/animation/source-position-fixture-v1/`:

| File | Bytes | SHA256 |
|---|---:|---|
| input.bin | 1,130,783,424 | `ae11f39452068a84c8777c0170f5541fcc8dffb4976bab4c25afeef43e1b7840` |
| output.bin | 85,917,368 | `c0de029298c25833d4f0f7784d396052472af8851d3f3ca15aa313955c06b251` |
| expected.tsv | 89,045,182 | `79e498cfd31005ab10ea78149c39bf98cef699132d27fd02a294ed233379a975` |

`[MEASURED]` The independent full-source Rust sampler passes **all 692,878
finite cases**, from **5,010 clips / 39,631 channels**, with exact XYZ/weight
bits and original neighbor indices under CW027F. It reads/prepares the complete
original source clips and chooses its own upper and neighbor indices. Its
`rust-v1.tsv` and the current-example `rust-v2.tsv` are byte-identical to
`expected.tsv`, SHA256
`79e498cfd31005ab10ea78149c39bf98cef699132d27fd02a294ed233379a975`;
`rust-v1.log` / `rust-v2.log` retain the successful invocations and denominators. The two source
NaN channels account for the channel-count difference, with both boundary
copies retained in the separate four-case exclusion file.

`inventory.json` retains all exact counts and hashes; the census lives in
`target/animation/source-position-census-v1/`. The separate positive check in
`rust-nonfinite-v2.tsv` / `.log` confirms **all four authored NaN samples**
return `CurveError::NonFiniteInput` from full-source selection. The earlier
negative-v1 output has a malformed header and is retained as invalid evidence;
only v2 is accepted. Neither that bounded domain check nor source curve
agreement establishes complete playback.

## Reproduction and remaining gate

```powershell
python tools/anim_position_native_probe.py 'D:\Games\SPORE Collection\SporeBin\SporeApp.exe' --sdk-bin-x86 'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86' --kernel32-lib 'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.18362.0\um\x86\kernel32.lib' --out target/animation/native-position-host-new
# Review the new host's source/manifest and copied-stub branch audit.
python tools/anim_position_native_run.py --host-dir target/animation/native-position-host-new --out target/animation/native-position-new
python tools/anim_position_source_probe.py 'D:\Games\SPORE Collection' --out target/animation/source-position-census-new
python tools/anim_position_source_probe.py 'D:\Games\SPORE Collection' --execute --census-dir target/animation/source-position-census-new --host-dir target/animation/native-position-host-v3 --out target/animation/source-position-new
# The source runner pins the reviewed host-v3 hash; a rebuilt host requires
# its own manifest review and explicit hash update before this command accepts it.
.\tools\build.bat run -p spore-bevy --example anim_position_source_verify -- 'D:\Games\SPORE Collection' target/animation/source-position-new/expected.tsv target/animation/source-position-new/rust.tsv
.\tools\build.bat run -p spore-bevy --example anim_position_source_verify -- 'D:\Games\SPORE Collection' target/animation/source-position-new/excluded.tsv target/animation/source-position-new/rust-nonfinite.tsv --nonfinite
```

`[UNKNOWN]` Quaternion, deformation, selector binding, rig conversion and
complete playback remain outside this position-only acceptance result. Source
curve acceptance retains the four explicit nonfinite exclusions and the
disabled/absent channel census; it must not be extended to complete playback.

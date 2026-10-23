# Native title resources and linear fades, 2026-10-23

## Measured scope

`[MEASURED]` Native analysis refers to the installed
`SporeBin/SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The source `TitleScreen` and `SporeTitleSeq` layouts are different resources
with different runtime callers. Source parsing, bounded fade execution and
controlled original shader draws do not establish a complete retail screen.
Original resources, decompilation, disassembly and reports remain in ignored
`target/`.

## TitleScreen is not an implicitly visible logo screen

`[MEASURED]` `TitleScreen` names SPUI instance `404642000910F2B9`.
Native `813790` loads it through `811EC0`, looks up root control `0362C790`,
sets its area from `805BA0`, then resizes/recenters the first matching child
control `0362D0B0`. Its final `80D3C0(root,1,1)` traverses graphics/invalidation
operations through `807A30`, `80D360` and `80D2D0`; it does not set child
visibility. UI initialization `815C10` calls this bootstrap at `81640E`
only when its owner field `21C` has mask 1 set.

`[MEASURED]` Source root Window 12 has authored flags `13`; children 7 and
11 both have flags `2`. The native draw-tree builder `95A690`, called from
manager update `95B420`, accepts a Window exactly when:

```text
!(private_flags_at_30 & 4)
    && ((private_flags_at_30 & 1) || (authored_flags_at_2C & 1))
```

The ordinary constructor initializes private byte `30` to zero. The serialized
property `EEC1B000` writes the authored flags directly. Therefore the ordinary
initialized root is eligible and both children are ineligible. The ineligible
branch skips that subtree; eligible children are visited from the child-list
tail through previous links. A later runtime visibility override requires
evidence before drawing those children as an original screen.

`[MEASURED]` The root's FadeEffect has duration 1, trigger 1 (Invisible) and
interpolation 0 (Linear). On explicit attach event `11`, native `963740`
reads the Window's visible flag and initializes target/current/previous to
zero for this visible root. Callback `96EFB0` then assigns `FFFFFFFF`.
This establishes the attached root's opaque foreground, not the first
presented retail frame.

Evidence: native v3 queries 263, 336, 337, 339, 349, 350, 352, 354, 355,
362, 363 and 364; `target/source-ui/ui-title-bootstrap-static-v1.json`
retains 64 eligibility fixtures and the bounded call graph.

## SporeTitleSeq has explicit show/hide transitions

`[MEASURED]` Native `DEABF0` loads `SporeTitleSeq`, instance
`404642005C5AC8CB`, and initializes six records at global owner `16A2324`.
Each record has a control ID at `204 + 12*i`, delay at `208 + 12*i` and
duration at `20C + 12*i`:

| Step | Control | Delay | Duration |
|---|---|---|---|
| 0 | `0670A3F8` | 0 | 0.1 |
| 1 | `05B82B38` | 0 | 4.1 |
| 2 | `0670A3F8` | 0 | 0.5 |
| 3 | `05BFDC38` | 0 | 4.5 |
| 4 | `05BFDC68` | 0 | 4.5 |
| 5 | `066F3C30` | 10.5 | 4.5 |

These are stored float32 values, including the rounded representations of
0.1 and 4.1. The update takes a supplied float delta; the retail source of
that delta is a separate gate.

`[MEASURED]` The non-`zh-cn` branch starts at step 5. The `zh-cn` branch
starts at step 3 and explicitly shows auxiliary control `05BFE3A8`.
The bootstrap copies that step's delay/duration to local fields `1C`/`18`,
clears local gap `20`, sets global pending-show byte `1F2`, clears bytes
`1F3`/`1F4`, and sets global state `200` to 1.

`[MEASURED]` Native `DEA810` advances this state machine. State numbers
below are the global `200` field; the separate step index is local byte `24`.

1. **State 1, wait/show.** When `804EA0` returns true, subtract the supplied
   delta from local delay. Once delay is at most zero and pending-show is
   set, clear pending-show and call the selected control's Window virtual
   `7C` with mask 1. The new visible value is false for step 2 and true
   otherwise. Steps 0/2 also show/hide control `067763D0`. Step 5 sets byte
   `1F4`. Enter state 2.
2. **State 2, hold/hide.** If byte `1F4` is set and service `67C650` virtual
   `14` returns false, call cleanup `DEA0A0` and set state zero. Execution
   still reaches the `804EA0` gate and possible duration subtraction after
   this cleanup; cleanup's step 6 prevents the ordinary transition. The
   transition uses duration strictly less than zero, not at most zero.
   For a step below 6, hide its control except at step 0, advance the step,
   and enter state 3. Step 2 instead becomes step 6 and sets byte `1F3`.
3. **State 3, gap.** If local gap is positive and `804EA0` is true, subtract
   delta and return, even when that subtraction reaches/passes zero. In
   every other case enter state 4. In particular, a false gate skips this
   gap rather than freezing it.
4. **State 4, select next.** At step 6, cleanup and state zero occur when
   service `67C650` virtual `14` returns false. At a step below 6, reload
   its delay/duration and pending-show, set a one-unit gap for step 1,
   and enter state 1. Entering step 5 also hides `05BFE3A8` and calls
   `DEA760` before the final state assignment.

`[MEASURED]` `DEA760` also runs directly from the initial step-5 bootstrap.
It tests configuration key `C14680EF` through `DEA2F0`/`DEA270`. The true
branch calls `DEA610` and writes outer owner state `1EC = 2`. The false
branch calls cleanup `DEA0A0`, writes outer state `1EC = 6`, and sends
message `066F8F25`. These are distinct from the inner state at `200`.
The false branch can dispose the layout before a title step is shown.

`[MEASURED]` Cleanup `DEA0A0` sets step 6 and duration zero, optionally
sends message `066F6B6F`, hides controls `06776FE0`/`06776FE1`, disposes
the layout through `811830(1)`, and clears its owner handle.

`[UNKNOWN]` The configured `C14680EF` value, the retail delta producer,
the ownership/meaning of UI gate byte `220`, and the complete external
service status are not established by these bodies. `804EA0` obtains the
UI owner and `812EC0` reads byte `220`. Wrapper `DEAB00` calls `DEA810`
with its float argument, but a complete live clock chain is still missing.
These inputs must be explicit in a bounded reproduction.

Evidence: native v3 queries 374, 379, 380 and 381;
`target/source-ui/ui-title-sequence-{bootstrap,update,phase-show}-sdk.asm`
and the update-wrapper/timing-gate SDK excerpts.

## Source variants and the remaining package decision

`[MEASURED]` Three packages contain the exact key
`0001FFFF:250FE9A2:404642005C5AC8CB`:

| Package | Bytes | Objects | Atlases | SHA-256 |
|---|---|---|---|---|
| `Data/PatchData.package` | 4,471 | 36 | 3 | `9d7ce1da53bad3cbae4409b7a9c194d4a0a56e3439c283c7e95ffc2eda181e79` |
| `Data/Spore_Graphics.package` | 4,471 | 36 | 3 | `911fb5164f53a3eb3b6a4be904437a7634f522387a007b1000ffaaa0d6cfb708` |
| `DataEP1/Spore_EP1_Data.package` | 5,585 | 43 | 4 | `3733dcd36febc59299b43c0b44aeda2b16d3f566e16e1c3ae5d856280459b991` |

`[MEASURED]` The two base copies differ only in three Image reference
indices that compensate for a different atlas-table order. Resolving
`01BE0001` to exact resource keys makes all typed elements/properties equal.
The normalized JSON digest is
`03df9a6fced1f735bbefb4a10a348548ffa7a0a5599e8ee58ac22b9cff775b66`.
The EP1 hierarchy differs, so this does not settle its selection.

`[MEASURED]` Native package enumeration `686EC0` opens each matching
`*.package` and calls `6862C0` with a supplied value. That loader selects a
manifest PROP with instance-high `40404000`; property `06EF59E5`, when it
is INT32, replaces the supplied value passed to resource-manager virtual
`50`. The PatchData manifest `4040400000000001` stores 2; the EP1 Data
manifest `404040002C24AB68` stores 101. Spore_Graphics has no matching
manifest. Product/filter acceptance and virtual `50`'s ordering comparison
remain separate evidence requirements.

`[MEASURED]` The native caller `687180` supplies defaults 0, 1 and 100
for active product indices 0, 1 and 2. Folder loader `6871E0` uses package,
property and cache values 0/1/2 or 100/101/102. Enumeration `686EC0` has
five stack arguments; query 399's inferred signature omits/mislabels an
argument. The bounded SDK caller/entry disassembly establishes these
values. They still do not establish which products were loaded in a
particular retail run.

`[UNKNOWN]` No selected retail overlay follows from the manifest values
alone. `PackageSet::discover` currently searches sorted paths and would
find PatchData first for this key; that is Rust lookup behavior, not proof
of native loaded-product precedence. Queries 394/395 and ignored
`target/source-ui/package-meta-*.prop` retain the package lead.

## Implemented fade and missing drawing work

`[MEASURED]` `spui_effect::LinearFade` accepts explicit attachment,
visibility-change and elapsed-second inputs. Native 963D70 sets Clock mode
5, cross-checked as Seconds in the public
[Clock interface](https://raw.githubusercontent.com/Spore-Community/Spore-ModAPI/master/Spore%20ModAPI/Spore/Clock.h).
Linear progress
uses separate float32 division/add/subtract, strict native convergence
threshold `3A83126F`, and `spui_draw::fade_white_argb` for callback color.
It returns callback/completion conditions without fabricating generic
Clock or Window hold/message side effects. A moving zero-duration,
zero-elapsed input is rejected because native division would leave the
finite unit-progress domain.

Five focused tests pass, including 110 independent native-derived tick
fixtures and ten sequences containing 156 frames. Independent Python/Rust
agreement covers 82 package indexes, 748 SPUIs and all 1,161 source fade
records: 1,099 linear visibility/invisibility cases are supported, 61
unproven interpolation cases and one other trigger are explicit errors.
The sorted LF digest is
`8d449e907b200385b813ecb98b00cf7728508faa0eb6658461378f838e9d8029`.
See `tools/spui_effect_probe.py`, example `spui_effect_verify` and
`target/source-ui/fade-comparison-v1.json`.

`[MEASURED]` In the base-equivalent layout, phase-5 control `066F3C30`
is Window 19, with a one-second linear Invisible fade and a centered child
containing text class `6F15F51B`. The visible caption refers to localization
table/key `2B7A0311:05B08777`; English source text identifies it as the
copyright caption. EP1 changes this hierarchy and adds an image control
populated by the bootstrap. The separate `TitleScreen` atlas must not be
substituted for these source elements.

`[UNKNOWN]` Native text layout, font/style selection, glyph raster/upload,
ProportionalLayout execution, selected overlay, external playback and live
event timing still gate an identical full title screen. The current fade
and image work do not close these gates.

## Native text and shipped font prerequisites

`[MEASURED]` Class `6F15F51B` is native `WinText`: its factory/name methods
reference `WinText`, and constructor `989040` sets main/public vtables
`1446040`/`1445F20`. It initializes format `210 = FFFFFFFF`, size override
`214 = 0`, and four padding values `218..224 = 0`. Address `989030` is a
release adjustor thunk, not the constructor. The wrapper `9888A0` obtains
the style manager through `885B40(1)` and dispatches to paint `988960`.

`[MEASURED]` Paint `988960` calls ordinary Window paint, then looks up style
ID `C0` through `8942B0`. Missing styles skip the text body. A nonzero size
override temporarily replaces style field `200`; the original value is
restored after drawing. Layout builder `89D380` receives the caption's
UTF-16 range, left/top padding, width minus right padding, height minus
bottom padding, the style, prepared layout `758`, and literal flags `14`.
Its result and buffers at `784`/`7AC` go through `96E6E0` to Graphics2D.
This is a source layout/glyph path, not sufficient evidence to substitute
a host-system text renderer.

`[MEASURED]` `89D380` includes a recursive fitting branch for style field
`254 == 2`, per-line generation through `89D120`/`89C380`, explicit
floor/ceil metric operations, and final vertical translation through
`897740`. `96E6E0` copies a 16-bit token per item and a 32-byte placement
record per item. Optional effect passes translate six float coordinates
in each record and draw through Graphics2D virtual `30`; the final pass
uses the supplied format/color and the original buffers. The measured
vtable `1441470` maps this virtual slot to `955E50`. The exact font,
placement and glyph-atlas interpretation below that bridge remains a
separate evidence requirement.

`[MEASURED]` `955E50` groups adjacent placement records by their first
pointer and retains that shared object through virtual `8`. For each item
it floors coordinates at record offsets `10`, `14`, `18` and `1C`, applies
the active local clipping rectangle, and emits a clockwise quad of four
20-byte vertices `(x, y, ARGB, u, v)` with UV corners `(0,0)`, `(1,0)`,
`(1,1)`, `(0,1)`. It also writes a 12-byte record containing the supplied
16-bit token and two float clipping offsets, initially zero. Each run
contains shared object, starting vertex, starting token record, count and
zero. Therefore these unit UV values are not original glyph-atlas UVs;
the later consumer must resolve them. Submission `953250` obtains an
object from `952790`, whose vtable `1441124` has slot `C = DF5080`.
Query 437 establishes that this slot is a no-op, not the glyph renderer.
The source command type is 2, selected through `9531D0` with state object
`166BAAC`; that state object's interface vtable is distinct from the
submitted draw object's vtable. SDK `95BE80` passes manager and command to
the active context's virtual `14`; only a null context uses the no-op
fallback. Context consumer `82EDC0` initializes iterator `951CF0` and reads
it through `952800`. The latter calls `951D40` at an active glyph boundary
before returning a vertex range, which is the next glyph-cache lead.

`[MEASURED]` The shipped stylesheet is resource type `0248F226`, instance
`0248E87394DA47E5`; the low word is the filename hash of
`SporeUITextStyles`. Native loader `815A10` opens that resource, initializes
the parser and supplies callback `8149C0`. For a `font` directive, that
callback strips the font filename extension, resolves the resulting name
in namespace `0248E873`, and opens type `027C5CEF`, with type `043B1EC5`
as a fallback. It passes the stream to font-server virtual `2C` with
argument 2. This exact callback is established by SDK disassembly from
entry `8149C0`; query 419's interior address was not a defined procedure.

`[MEASURED]` The base copyright Window 17 selects style `7237635F`,
`GuiDataSmallCentered`, which inherits `DefaultStyleCentered` and then
`DefaultStyle`. In all three retained base/PatchData/EP1 stylesheet copies,
this chain declares family `Trebuchet EA`, bold weight, `font-smooth:
always`, centered/middle alignment, and a final `font-size: 11px` override.
The hidden ESRB Window 16 selects `D3D702AA`, `AssetAuthor`, which changes
weight to normal and adds no-wrap/ellipsis. These are source declarations;
native CSS conversion, face selection/fallback and final raster metrics
still require verification.

`[MEASURED]` Original Graphics package resources contain the following
SFNT name records. Each instance has high word `0248E873` and type
`027C5CEF`; the low words match the stylesheet font filenames with their
extensions removed.

| Instance low | Group | SFNT family / subfamily | Bytes |
|---|---|---|---|
| `511B49CA` | `00010000` | Trebuchet EA / Bold | 117,628 |
| `61639DFB` | `0001FFFF` | Palatino Sans EA / Bold | 228,016 |
| `6BBC4E0E` | `0001FFFF` | Maxis Password / Regular | 3,736 |
| `BACAAAA5` | `0001FFFF` | Arial Unicode EA / Regular | 23,311,968 |
| `FFE355DF` | `0001FFFF` | SporeCurrencyEA / Regular | 8,672 |

The Trebuchet EA payload SHA-256 is
`f6ad6f1e12e2d2a7c1e28b0ea3cb93a653482840b2b6340fb8682f3eb4c75f26`.
The bounded metadata reader uses the public
[OpenType name-table format](https://learn.microsoft.com/en-us/typography/opentype/spec/name)
to identify names, not to infer original glyph rendering. Original font
bytes, source stylesheets and table/name summaries are retained only in
`target/source-ui/font-*` and `target/source-ui/ui-source-fonts-v1.json`.

Evidence: native v3 queries 400, 401, 420, 422, 426, 427 and 432; bounded SDK
`ui-native-text-{factory,constructor,draw}-sdk.asm`,
`ui-font-resource-loader-entry-sdk.asm` and
`ui-font-style-loader-sdk.asm`, plus
`ui-native-glyph-{bridge,command-submit,submit-ctor}-sdk.asm`.
Native glyph generation/raster/upload and
exact layout outcomes remain unimplemented. The source font chain is
available; no approximate font or caption bitmap closes that gate.
The later cache/raster lead is recorded in
[spui-font-native.md](spui-font-native.md).

## Bounded source-font reader

`[VERIFIED]` `spui_font` reads bounded TrueType 1.0
table directories, strict Unicode name records from name format 0, explicit
format-4 cmap records, unscaled horizontal metrics, and loca/glyf header
bounding boxes. The caller chooses a cmap record explicitly. Other cmap
formats and legacy name encodings are reported as unsupported when
requested; their presence does not silently select a substitute. The
reader neither decodes outlines/instructions nor scales, shapes, hints,
rasterizes or uploads glyphs. It also does not choose a native face,
fallback order, CSS unit conversion or line-height policy.

The format references are the public OpenType
[cmap](https://learn.microsoft.com/en-us/typography/opentype/spec/cmap),
[hmtx](https://learn.microsoft.com/en-us/typography/opentype/spec/hmtx) and
[head](https://learn.microsoft.com/en-us/typography/opentype/spec/head)
table specifications. Checked directory/record ranges, monotonic bounded
loca offsets, cmap segments and glyph indices precede access. Checksums,
outline validity and native raster behavior are separate concerns.

`[MEASURED]` The independent `tools/spui_font_probe.py` uses FontTools
4.60.1 directly on all five original Graphics-package faces. Its source
oracle covers 51,680 glyph metrics/bounds and 40,246 nonzero mappings
across all six format-4 records. The 91,967 sorted LF rows have SHA-256
`624040b34e89f302edfca3eb18acc412042e27b21effa60bbd8027af1c073533`.
Raw unsupported Macintosh cmap formats 0/6 are explicitly identified.
The source report is `target/source-ui/font-python-corpus-v1.json`;
example `spui_font_verify` supplies an independently implemented Rust
comparison. All five focused Rust tests pass, and the complete Rust/source
comparison matches every row and the digest above. The retained result is
`target/source-ui/font-comparison-v1.json`. This verifies source metadata,
BMP mapping and design-unit metrics, not native text layout or raster output.

# Cell progress-control serialized graph audit v1

This read-only inventory covers explicitly selected installed SPUI copies, not
the active provider or a captured frame. Run `python
tools/cell_ui_progress_graph_audit_v1.py 'D:\Games\SPORE Collection' --out
target/source-ui/cell-ui-progress-graph-audit-v1.json` with a fresh output path.
The report retains typed properties, child paths, reference closures, resource
tables, archive group words and payload SHA-256s. A separate census scanned
748 SPUI entries across 82 packages without parse errors; IDs also occur in
other layouts, sometimes more than once.

Both candidates have type `250FE9A2`, instance `4046420026B068D2`:

| Package | Payload SHA-256 | Root | Progress parent | Text parent |
|---|---|---:|---:|---:|
| Data/PatchData.package | 5c93c578393784282700b0cdacc1d370870851d34fc5a76aa62bad1c9809edda | 492 | 192 | 436 |
| Data/Spore_Graphics.package | 6f85179ae30bda8f6aa9a7522523ca7c9bd8a33a3f44c31db50a70cfb7d567dc | 495 | 193 | 439 |

The progress parent is root child ordinal 0; the text parent is root child
ordinal 10. These are serialized `EEC1B00B` ordinals, not proven native paint
order. Under the progress parent, `051B5168`, `051B7E48`, and `051B54C0` through
`051B54C4` occupy ordinals 6, 7, 8, 9, 10, 11, and 13 respectively. The
`ABCD1001` control is text-parent child ordinal 1. Their object indices are
respectively 129, 131, 143, 155, 166, 177, 191, 435 in PatchData and 130,
132, 144, 156, 167, 178, 192, 438 in Spore_Graphics. Native `95A690` visits
eligible child lists from tail to previous link; construction, insertion,
private visibility flags, and later mutation remain conditions on any order.

Authored parent rectangles are `[223,-29,-88,-1]` and `[136,-29,225,-1]`,
and root is `[25,25,871,625]` in both copies. Segment rectangles contain
inverted x edges; these are inputs to procedures, not screen rectangles. The
text rectangle is `[11,7,79,20]`. The report preserves exact float bits.
Each segment references three Image objects through its serialized closure;
`051B5168` references one. All these Image resource ordinals resolve to PNG
type `2F7D0004`, instance `31A44893B8B50133`, while the actual atlas package
copy is not selected. The Image properties preserve UVs, dimensions and flags;
the existing `ImageRegion` implementation and its independent comparison are
documented in `spui-layout.md`.

## Bounded source class observations

The existing source analysis identifies `2F418D72` as SimpleLayout. The
reference closure includes byte flags 08 and 0E. Its float32 local adjustment,
clamp, and comparison coverage are documented in `spui-layout.md`.

On the SHA-256-pinned executable in `cell-ui-progress-value-route-v1.md`,
the `AF3DF411` getter at `97E850` and name string at `1444184` identify
`ProportionalLayout`. Its descriptor at `144415C` maps property `0F3D0000`
to four floats at object +10. The constructed interface at object +C has
callbacks `97EA70` and `97EAE0`: each computes parent right-minus-left or
bottom-minus-top, multiplies by the corresponding coefficient, and respectively
adds to or subtracts from each edge of a supplied rectangle, using ordered
`SUBSS`, `MULSS`, and `ADDSS`/`SUBSS` operations. These are bounded callback
observations; call sequencing, clamps, and full integration need separate proof.

The `B03C196F` getter at `9879F0` and string at `1445D04` identify
`StdDrawable`. Its descriptor at `1550BD0` includes reference array property
`0F3AC75E`. The observed `9879D0` returns one of eight slots indexed 0..7;
`987A00` selects slots using state bits and null fallbacks. This does not yet
resolve the complete caller state, scaling/slicing, effects, or rendering. The
source graph shows these drawables link Image objects; it does not establish
which state is drawn in a retail frame. In both candidate closures all five
segment StdDrawables have an eight-word type `0013` property; their first
word points to an Image object and the other seven words are `FFFF`. This
inventory does not itself establish deserializer pointer resolution.

The independent Python graph reader and Rust explicit-source intake agree on all
5/5 progress drawable arrays and 40/40 words in each package (10/10 arrays,
80/80 words total). Reproduce with the explicit-source build and comparison
commands below. These are raw words, with no pointer resolution assumed.

The separate `spui_std_drawable` module decodes those eight words and implements
the `987A00` pointer selector with explicitly supplied state and slot occupancy.
It preserves the raw words and requires the caller to supply resolution
outcomes; it does not infer `FFFF` pointer semantics or active state. Focused
tests passed 2/2 using `tools\build.bat test -p spore-core spui_std_drawable`.
The independently expressed Python branch model matched 8,192 explicit
state/occupancy combinations, including ignored upper state bits and every
eight-slot occupancy mask, after `tools\build.bat build -p spore-core --example
spui_std_drawable_selection_v1` and `python
tools/spui_std_drawable_selection_compare_v1.py
target/debug/examples/spui_std_drawable_selection_v1.exe`. This checks the
bounded models against each other; it is not original native execution.

The separate `spui_proportional_layout` module implements the bounded callbacks
for distinct, by-value parent and child inputs. Its three synthetic unit tests
pass; they do not execute native code. Finite input and output checks are model
admission rather than source behavior. Native pointer aliasing, MXCSR rounding,
FTZ and DAZ have not been established. Both explicit candidate copies' six
progress procedures have type 000B, count 4, and match Rust-decoded coefficient
bits against the independent graph audit (12/12). Reproduce with a fresh graph
report, `tools\build.bat build -p spore-bevy --example cell_ui_proportional_source_v1`,
then `python tools/cell_ui_proportional_source_compare_v1.py` with the install,
report and built example paths. This is source decoding evidence, not native
arithmetic execution.

An independent Python binary32 calculation matched the Rust callback on 1,032
operation rows: 12 source coefficient records, 500 seeded synthetic input sets,
and four synthetic rounding, inverted-span, nonfinite and overflow cases, each
for apply and unapply. The original build used Cargo directly; it was then
rebuilt through the repository wrapper. Reproduce with `tools\build.bat build -p
spore-core --example spui_proportional_arithmetic_v1`, then `python
tools/spui_proportional_arithmetic_compare_v1.py
target/source-ui/cell-ui-progress-graph-audit-v2.json
target/debug/examples/spui_proportional_arithmetic_v1.exe`. The comparison
assumes ordinary round-to-nearest ties-to-even and no FTZ/DAZ, rejects nonzero
subnormal intermediates, and is not native execution. Error rows compare the
model admission boundary, not original source error behavior.

A useful next slice is a fixed-state StdDrawable/atlas diagnostic.
An offscreen preview also needs explicit parent spans and transforms, ordered
procedure dispatch, child-list insertion, eligibility, clipping and composition,
atlas provider, text formatting and rendering, plus animation/update semantics.
None of this inventory establishes original visibility, gameplay meanings, or
a complete source HUD.

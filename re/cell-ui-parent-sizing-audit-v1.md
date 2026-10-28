# Cell UI source parent and explicit-input diagnostic design v1

This note describes a candidate path, not proof that a particular installed
package or a retail frame is selected. Addresses refer to base SporeApp.exe
SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.

## Parent construction

* Initialized data at `1545D68` associates key `2EDD95CA`, priority `100`, and
  UTF-16 name `UIWorld_GlobalUI`. The twenty 12-byte entries beginning at
  `1545CC0` are traversed backward at `811350..81136B`; each key and priority
  are passed to `810E70`.
* `810E70` obtains the Window manager via `957FF0`, calls manager virtual
  slot `04` for a parent, and passes that parent to `806070`. The latter
  constructs a Window, sizes it to its parent's spans, and attaches it via
  parent slot `D8`. `810E70` then sizes the created world to its parent's
  current laid-out spans and assigns the table key. These paths depend on
  successful calls and initialization; they do not provide a fixed resolution.
* `815C10` constructs the Window manager via `958E00`, obtains an eight-word
  descriptor through `67D870` and a virtual `1C` call, and passes descriptor
  words 0 and 1 to manager virtual `0C`. Their origin and interpretation
  require the associated vtables and display setup. Manager vtable `1441318`
  resolves `+04` to `958060` (root Window interface) and `+0C` to `95A9F0`,
  which enqueues event `4ECD5687` with the two descriptor words. `95AB90`
  drains the queue and `959890` handles that event: it computes half-spans
  and reciprocals, then calls the manager root Window `+60` setter with
  `[0,0,width,height]` when the root exists. Width and height are runtime
  values, and this establishes a conditional initial resize, not a live frame.
  `67D870` returns descriptor-provider singleton `15FE8C0`, whose setter is
  `67DA80`. `7A4910` gets the provider via `75FA70`, initializes it, and
  publishes it only on success; `7A65E0` clears it on teardown. `75FA70`
  constructs a Graphics object via `75F840`; its virtual `1C` resolves to
  `C86FB0` and returns Graphics `+5C`. Constructor initialization zeros that
  descriptor before runtime graphics setup. The manager singleton is set via
  `958000`. Runtime dimensions and root matrix are not captured here.
* Cell caller `E54150` requests `GlobalUICell-3` and passes `2EDD95CA` to
  `811E30`. `811D70` obtains the external parent, and `811890` attaches the
  loaded layout roots to it. `811090` looks up the root control `FFFFFFFF`;
  if present it sets its size from the parent's span, moves it to zero, and
  changes flags/background. Both candidate payloads have such a root.

## Bounded computational fixture

`cell_ui_ordered_local_v2` uses an explicitly authored root rectangle as a
fixture; it does not model the `811090` root sizing branch. The independent
comparison validates that fixture only. For a more representative offscreen
experiment, supply an **explicitly sourced or synthetic manager-root area**,
external parent matrix and viewport, retaining their provenance separately.
Under successful initialization and queued-event delivery, root, world and
layout-root spans can be computed from supplied width and height; attachment,
resize sequencing and later mutations must still be modeled and checked. Then compute the
selected object's SimpleLayout and the identity-local matrix subset using
`cell_ui_matrix_fixture_v2` only if its prerequisites are shown. Its inputs are
hex float32 bits and are not retail capture values.

The selected PatchData source chain is root `492` -> Window `456` ->
ImageDrawable `454` -> Image `453`, with local SimpleLayout `455`; the other
installed candidate has separate object indices and must be resolved through
its own references. Metadata at `1550000` maps `EEC1B007` to Window `+1E0`;
metadata at `1550708` maps `4F3C4A26` to ImageDrawable `+24`. Window
`960430` invokes the drawable with `(0,0,width,height)` after layout.

Rendering also requires source-backed texture resolution, state and clipping,
parent/world matrix and color, ordering, selected image copy, and any runtime
mutations. A single-image diagnostic is not full screen acceptance. In
particular, do not infer an active package or a retail frame from the table,
conditional layout, or controlled shader fixtures.

## Evidence

Retained dossiers: `target/rea/20261028-source-runtime-v6/` q36, 46, 208,
219, 223, 226, 228–230, 256, 260, 263–264, 278, 281, 283–286, 297–299,
318, 320–321, 325, 333 and 342–345. The bounded child-list decode is
`target/source-cell/child-and-feeding-routes-linear-v1.json`; the older
layout and draw evidence is in `re/spui-layout.md`, `re/spui-draw-state.md`
and `re/spui-title-native.md`. Evidence under target is local and ignored.

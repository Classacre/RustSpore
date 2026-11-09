# Cell UI graphics descriptor source trace v1

This static trace extends `cell-ui-parent-sizing-audit-v1.md`. Addresses refer to
base SporeApp.exe SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.

* Graphics constructor `75F840` installs vtable `140DC98`. Its slot `1C`,
  `C86FB0`, returns `this + 5C`, the descriptor subsequently read by UI
  initialization `815C10`. Slot `14` is `75EF50`: its successful device setup
  branch copies the 15-word structure at `1713444` and assigns words 2 and 3
  to Graphics `+5C` and `+60`. Earlier in that function the structure is
  populated from a local 15-word setup block and passed to `11F8430`. Because
  the decompiler reports unrecovered local inputs, the original option values
  and final runtime dimensions are not established from this static trace.
* Slot `34`, `75EBC0`, also copies words 2 and 3 from `1713444` to Graphics
  `+5C` and `+60`. Slot `18`, `75E820`, takes explicit dimension arguments:
  negative inputs fall back to those two structure words, and accepted values
  are assigned to Graphics `+5C/+60`. It passes dimensions to further graphics
  and window calls. This shows a dimension-change path; it does not establish
  that the UI resize queue runs after every such change.
* UI initialization reads the descriptor after successful graphics provider
  publication. At `815CAF` and `815CC3`, signed integer descriptor words 1 and
  0 respectively are converted to float before the manager call. The previously
  traced manager event conditionally sizes its
  root to `[0, 0, width, height]`; the world and layout root then use parent
  spans on the cited attachment path. These are runtime-dependent values. No
  active layout, package winner, global transform, visible frame, or actual
  screen position follows from this chain alone.

The saved `OptionScreenSize 25` is an index into a dynamically built option
list, not an installed fixed resolution ID. On the traced option provider,
`7CC620` finds or creates the `046170A1` option and rebuilds its candidate
list. It calls `11F80B0`, obtains adapter data through `11F8670`, and iterates
its available format/mode records. Under the standard D3D9 interface mapping,
`11F8670` uses vtable +18 and +1C for adapter mode count and enumeration. The
option builder accepts modes subject to format and dimension comparisons,
including two caller-provider values `046C0C15`/`046C0C16`; when either is
zero it substitutes 800 and 600 for these filter bounds. Accepted mode records
are appended through `7CA130`, with properties `038D0A06`, `038D0A07` and
`00D77E98` populated from source mode fields. `7C7750` rejects an out-of-range
index and otherwise looks up these three properties in the indexed record.
The D3D adapter, format support, enumeration order, current bounds, later list
updates and preference validation all affect index 25. The 800×600 fallback
is a candidate-filter bound, not a claim about the selected viewport. No live
mode or actual resolution follows from this trace.

A later reviewed no-window x86 D3D9 inventory on RTX `10DE:2520`, using
explicit minimum bounds 800×600 and that inventory's reported desktop mode
2560×1440 at 165 Hz as its maximum, produced 26 records after applying the
bounded source-modeled format and dimension filter. In this controlled list,
zero-based index 25 is 2560×1440 at 165 Hz. The retained receipt
`target/source-cell/screen-mode-inventory-result-v1/receipt.json` has SHA-256
`464d3d952cde15a25addec384715c9ad62d11aebad365a5cb30ff5354a341731` and
includes all rows and input hashes. The process created no device or window.
This is a local inventory and projection at the time of that run, not the
game's historical adapter enumeration, effective property bounds, saved-index
validation or current display state. In particular it does not establish that
the stale saved `OptionScreenSize 25` selected 2560×1440 in an actual game.

An explicit scan of 23,228 PROP payloads across PatchData, Spore_Game and
EP1 Data found both filter properties only in each package's `08EA246B`
resource. Every identified copy has scalar INT32 `046C0C15=800` and
`046C0C16=600`; full keys, decoded hashes and equal package endpoint hashes
are in `target/source-cell/screen-mode-bounds-payload-scan-20261108-v1.json`,
SHA-256 `ea0ba136c2a19f17d8ba5b19f419f2c8b379d0b2be0c4bb8278fc62b120ab247`.
That bounded corpus and the matching fallback narrow candidate filter values,
but do not establish live provider contents or rule out overrides.

Source mode enumeration at `11F8670` iterates four format words stored at
`015D17EC`: `23h`, `16h`, `18h`, `17h`, in that order, and invokes the D3D9
mode count and enumeration slots per adapter/format. The screen option builder
accepts format values `15h`, `16h` or `21h`, so only `16h` overlaps that
fixed four-word list under unchanged tables. It filters dimensions inclusively
against the supplied minimum and the current-mode words returned by
`11F8020`, then appends accepted records via `7CA130`; no deduplication is
present in that bounded iteration. `11F8304` fills the current-mode source via
the D3D9 `GetAdapterDisplayMode` slot, conditional on the selected adapter and
successful initialization. These static statements do not prove the list
at a particular runtime point or that later code never changes it.

The bounded source instruction listing is
`target/source-cell/screen-mode-source-audit-20261108-v1.json`, SHA-256
`c9419ce40b6366f3594ee8dbc185e19dd464056354ce0620f4e4e2cf58ed2b3b`.
The source's target record dimension writers and relation to the common mode
block are separately documented in [the screen trace](cell-screen-effect-render-trace-v1.md).

Evidence: ignored `target/rea/20261028-source-runtime-v6/` dossiers 333,
350–353 and 541–544. The vtable slot values also agree with the original PE
section bytes. `75EF50` and `75E820` are decompiler output with unresolved
locals and indirect calls; success, timing and actual values require further
runtime evidence. The 15-word mode block and its setup inputs are the next
source trace; display capture remains necessary for a claim about a live HUD.

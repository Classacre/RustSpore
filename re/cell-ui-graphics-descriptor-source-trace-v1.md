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

Evidence: ignored `target/rea/20261028-source-runtime-v6/` dossiers 333,
350–353 and 541–544. The vtable slot values also agree with the original PE
section bytes. `75EF50` and `75E820` are decompiler output with unresolved
locals and indirect calls; success, timing and actual values require further
runtime evidence. The 15-word mode block and its setup inputs are the next
source trace; display capture remains necessary for a claim about a live HUD.

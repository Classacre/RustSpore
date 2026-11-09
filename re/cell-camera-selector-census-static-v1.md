# B5CB80 selector call-site census: bounded static result

Follow-up to [cell-camera-service-selection-static-v1.md](cell-camera-service-selection-static-v1.md). The installed `D:\Games\SPORE Collection\SporeBin\SporeApp.exe` has SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`; addresses are virtual addresses. No game, UI or GPU code was executed.

A read-only scan of the PE `.text` bytes for direct x86 `E8 rel32` calls whose computed destination is `B5CB80` found 27 byte candidates. This is a byte-pattern search, not a complete disassembly or call graph; false positives, indirect calls, and code outside `.text` remain possible. Focused disassembly of four candidates in `E00000..EFFFFF` verifies:

| Call | Immediately supplied argument | Installed camera PROP label, if matched |
|---|---:|---|
| `EF1029` | `B03AF999` | `Creature game camera` |
| `EF151B` | `00000000` | unresolved zero request |
| `EF2344` | `00000000` | unresolved zero request |
| `EFE1E0` | `00EBB801` | `Gonzago (game) camera` |

For both nonzero cases the code pushes the key before calling `B3D2D0`; that call returns the receiver for `B5CB80`, leaving the key as its argument. `EFE1E5..EFE1FA` independently calls the camera service virtual +40 with `00EBB801` after selection. The labels and full keys `40410100B03AF999` and `4041010000EBB801` are present in the existing `target/source-cell/creation-camera-census-v2.json`, attributed to `Data\Spore_Graphics.package`. The census has 82 indexed packages and no reported errors, but it is a bounded camera/setup census; a label match is not evidence of execution, package overlay winner, or that either camera is active in Cell. The zero calls do not establish whether a zero-key entry exists or whether selection succeeds.

The other recognized nonzero candidates in the same direct-call scan include `8916F92D` (cinematics), `E3057616` (Tribe), `780786FD` (GGE), and `01103192` / `01103193` (space / space planet). No direct `B5CB80` call candidate appeared in `E50000..E6FFFF`. These observations do not rule out a generic or indirect Cell caller. The census contains several editor Cell cameras, but no record with a label explicitly naming a Cell gameplay camera; that negative is limited to this existing census and is not proof of absence from all resources.

In the Cell setup at `E5DA39..E5DA9A`, another object is acquired through singleton `67D8B0` virtual +18 with argument `04A4D97F`, stored at stage +161A0, then receives virtual +30 with `A426730B`. The object is subsequently passed to model-world virtual +140. Neither constant establishes a camera identity.

Subsequent tracing resolved this particular path as a world lighting selection,
not a camera identification: see [Cell world lighting selection](cell-world-lighting-selection-v1.md).
The separate camera-controller selection and its actual Cell-stage caller remain open.

The exact Cell selector, controller class, update cadence, actor anchor, and live transform remain unproven. Source-backed values for the separately identified auxiliary camera are in [cell-aux-camera-transform-static-v1.md](cell-aux-camera-transform-static-v1.md); the present census does not turn that fixture into the main camera.


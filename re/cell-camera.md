# Source Cell thumbnail camera

Measured on 2026-10-22 in the base SPORE executable and local source packages.
The offscreen preview now applies the proven orbit orientation and perspective
parameters. Model normalization, bounds fitting and lighting remain inspection
controls; retail thumbnail framing, capture timing and shading remain open.
The focused source suite passes 9/9, including the installed Cell art population.

## Source selection

`[MEASURED]` In both PatchData and Spore_Game, the PROP resource
`00010000:00B1B104:406001003615A30B` is labelled `Cell Editor Setup`. Its named keys
link these source camera records:

| Editor property | Camera low ID | Source label |
|---|---|---|
| `B02D871C` | `02AD26C9` | `EditorCameraCellThumbnail` |
| `121D3184` | `97F5B65C` | `EditorCameraCellThumbnail` (second preset) |
| `300DB745` | `D1A0765D` | `EditorCameraCellUI` |
| `B4A08D14` | `3F37CA0F` | `EditorCameraCellSporePedia` |

The labels independently FNV-1 hash to the low IDs except the second preset,
which retains the same embedded developer label as the first. This does not
establish the original name of the second resource.

The native setup code at `0058E79E` reads `B02D871C` into member `+1F8`, then
`121D3184` into `+1FC`. Native `005F30C0` selects the `B02D871C` configuration
path when obtaining the relevant stage camera. The camera manager `007C64F0`
enumerates source namespace `40410100`, reads class property `00ED3928`, and
instantiates the registered camera class.

The chosen camera is PatchData
`0001FFFF:00B1B104:4041010002AD26C9`. Its parent key has low ID `3F3B2714`; the
installed source parent is Graphics `404101003F3B2714`, labelled
`default template for a mouse-controlled camera`. It supplies class ID
`00EDD3BB`. Native factory registration at `00F48268` binds that class to
`007DA4C0`, which constructs `007D9CE0`. The superficially similar `005A24C0`
reader belongs to another orbit implementation and is not used for this preset.

## Bound fields and basis

`[MEASURED]` Reader `007D9190` binds the following fields. The names below describe
their observed mathematical role, not recovered source identifier names:

| PROP ID | Selected value | Native use |
|---|---:|---|
| `00C7C4FB` | 1.0 | Orbit distance state |
| `00C7C4FC` | -90.0 | Converted to radians; local X orbit rotation |
| `00C7C4FD` | -90.0 | Converted to radians; negated local Z orbit rotation |
| `6FDA2E1C` | 0.0 inherited | Local horizontal translation state |
| `8FDA2E23` | -0.55 | Local vertical translation state, negated in transform builder |
| `044C6220` | 0.872600019 | Radian FOV parameter, converted to degrees for the projection setter |
| `01102B20` | 0.0199999996 | Near plane |
| `01102B2F` | 200.0 | Far plane |

Native update `007DA720` sends the FOV and planes to `007C4CD0`, `007C4540` and
`007C4560`. Its transform builder `007DA320` starts from the anchor transform,
applies `00409920` with negated FD radians, then `005A2C50` with FC radians.
Those helpers mix the first/second and second/third basis vectors respectively.
The preset's third orbit angle remains its zero default.

For an identity anchor, the source basis is right `+X`, forward `+Y`, up `+Z`,
and the rotation is `Rz(-FD) * Rx(FC)`. Thus this preset's right/forward/up are
`+Y`, `-Z`, `-X`. The Bevy camera basis is `[right, up, -forward]` because Bevy
looks along local `-Z` with local `+Y` up. The source model's coordinates are
preserved; rotating the camera gives the source thumbnail's horizontal facing.

## Exact projection branch

`[MEASURED]` `007C4CD0` computes `tan(FOVDegrees * pi / 360)`. Constants `pi` at
`0153D7A8`, `1/360` at `01401BE4`, and `0.75` at `01410334` were checked in the
source executable bytes. It distinguishes **exactly square** from nonsquare
aspect using this flag sequence:

| Aspect comparison | `LAHF & 44` | Parity after TEST | JNP | Tangent factor |
|---|---|---|---|---:|
| Ordered equal to 1 | `40` | odd | skips multiply | 1.0 |
| Ordered unequal to 1 | `00` | even | takes multiply | 0.75 |

For valid positive finite aspect `a`, let `t = tan(sourceFov/2)` for `a == 1`,
otherwise `t = 0.75*tan(sourceFov/2)`. The native horizontal and vertical matrix
diagonals are `1/(a*t)` and `1/t`. The Bevy vertical FOV is therefore
`2*atan(t)`, not the source raw FOV copied unchanged. This behavior applies to
both landscape and portrait nonsquare viewports; it has an exact-square branch.

Native `007C3C40` writes finite perspective depth with `w=z` and near/far mapping
to 0/1. The preview uses Bevy's infinite reverse-depth matrix and source far-plane
frustum culling while preserving the recovered x/y ray geometry, field of view
and near plane. Its geometry is fitted wholly inside the source far plane; exact
finite far-plane pixel clipping remains a separate renderer fidelity gate. It
does not copy an incompatible Direct3D matrix into Bevy.

The real Bevy matrix test exposed a separate integration pitfall: changing its
`near` field while keeping the default `near_clip_plane` leaves a 0.1-unit
custom plane active. The preview explicitly sets both to the source 0.02-unit
plane, and the test projects that plane to reverse-depth 1.0.

## Remaining gates

`[UNKNOWN]` Native object-to-camera anchor setup, model fit, accumulated input,
capture-time easing and thumbnail timing have not been fully established. The
native camera update smooths several fields; a preset value alone does not prove
the value at the instant a saved thumbnail was captured. The preview consequently
uses its own explicit bounds-fit distance and does not claim native framing.

The current source reader also needs a separate follow-up for the fixed Cell
developer-name extent. The Cell format's numeric tail is not another name field.

## Evidence and verification

Ignored `target/source-cell/camera-props.json` retains all seven source PROP
records, all raw field bytes and the two editor-setup references. Its reproducible
read-only probe is:

```powershell
python tools/source_camera_probe.py "D:/Games/SPORE Collection" --out target/source-cell/camera-props.json --references
```

Native evidence is retained in
`target/rea/20261022-anim-rig-finalize-v2/`: reader/constructor queries 152/158,
update/transform 168/170, basis helpers 180/181, projection 182/201/209,
camera selection 190/191, bounded setup bytes 210 and factory registration 199.
Unrecovered functions are reported as bounded byte observations, without a
name-based decompiler fallback to an unrelated function. No EA code was copied.

Focused mathematical tests compare the orbit basis with independent row formulas,
check Bevy forward/up conventions, the exact-square flag truth table, source
projection rays and all bounds-fit corner limits. The installed corpus test also
checks the actual chosen camera, overlay winner, inherited template and raw
FOV/plane bit patterns. GPU screenshot results are recorded after validation.

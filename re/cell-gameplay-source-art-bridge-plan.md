# Cell gameplay source-art bridge

The experimental Cell game has an opt-in static source-art player. It loads a
baked GMDL and its painted atlas from the user's installation; it does not ship
those assets. The surrounding simulation, world, UI and other actors are
experimental. This is not original gameplay or original animation playback.

## Use and implementation

Pass `--source-cell-install <directory> --source-cell-id <hex>` together after
`--experimental --cell`. `--creation` cannot be combined with this mode. Errors
in package loading, resource resolution, source sampler support or mesh bounds
are reported instead of silently substituting procedural player art. The loader
uses its explicit inspection package order; this is not proof of retail resource
manager precedence. Each Cell definition, structure, model prop, model and skin
identity is logged with its package path and resource key.

The source mesh and atlas are loaded once. The existing `Camera2d` renders the
world; an orthographic `Camera3d`, layer 1, no clear color and later camera
order draws the source mesh with a PBR inspection material and a separate light.
The HUD is targeted to the later camera. The player mesh is a child of the
simulation actor transform and normalized to the actor radius by source bounds.
Its local +90 degree Z rotation derives from the measured source thumbnail
camera basis and turns Jawhead's thumbnail-facing mouth toward the prototype's
+X heading. That heading correspondence is an inference for Jawhead, not an
established source gameplay orientation for the corpus. The source material,
lighting, and independent depth layer are approximations; the 3D player is
composited over 2D actors rather than participating in source occlusion.

The selected asset remains in app resources across restart and compatible save
loads, and is not serialized into saves. Loading a snapshot containing an
authored creation is rejected in source-art mode so its appearance cannot be
silently replaced. These lifecycle paths have not been verified interactively.

`--offscreen --shot <new PNG>` uses the common headless render target and
readiness checks, including the atlas and nonempty UI glyphs. The offscreen
simulation waits for renderer readiness and advances no further than the demo
duration; the default screenshot duration is 1.5 seconds. AI input is sampled
per rendered frame, so this does not establish cross-machine deterministic
trajectories. Windowed gameplay retains its existing behavior.

## Bounded offscreen evidence

Three captures on NVIDIA GeForce RTX 3060 Laptop GPU / Vulkan were visually
inspected. They show the static source player in the experimental scene near
the projected screen positions below. v2 and v3 predate the full text-readiness
simulation latch; v4 uses the final latch implementation. They establish those specific captures,
not interactive control, resize, save/load, retail camera framing, or fidelity.
The logs record Cell `7D00E384`, creation `40616200780A9EAF`, zero pending
structure effects, package paths and keys for the source resources. The logs
also report that Vulkan validation layers were unavailable.

| Run | Duration | Tick | Player world position | Approximate screen position |
|---|---:|---:|---|---|
| v2 | 2 s | 120 | (4.7395, 0.6483) | (690, 393) |
| v3 | 4 s | 240 | (12.3349, -1.9510) | (770, 421) |
| v4 | 2 s | 120 | (4.7678, 0.6509) | approximately (690, 393) |

Evidence under `target/source-cell/`, SHA-256:

| File | SHA-256 |
|---|---|
| `cell-gameplay-source-bridge-v2.png` | `d03837d8fc11a6bf7bf6e159d77ef77511a030be76a204ffe6b16aaf3e3fc2aa` |
| `cell-gameplay-source-bridge-v2.log` | `8f90a6a35870a9694e7d2bd704ecf44fa3f7cb621ed90e2533777286b49ba368` |
| `cell-gameplay-source-bridge-v3.png` | `9ef094419c0488495bdfbbb5292bb9c8c924de51e481ec1754d04f269c677823` |
| `cell-gameplay-source-bridge-v3.log` | `6dffd3b5a7914627d6ca3385ea17c75599a95400df6b929ae1d11d5358bfa858` |
| `cell-gameplay-source-bridge-v4.png` | `fac039fe2ba7fcb5924d62733dd337b931a72ef1450692878d3e55dafa0c53ac` |
| `cell-gameplay-source-bridge-v4.log` | `1d80eeea85127b3fd00d2b7a6d23d4b14f068ee298e00a7175bb795e492ce514` |

Nine `cell_game` tests passed on the final latch version. These files are local
generated evidence and are not source assets. The captures do not prove original shader or lighting fidelity, animation, effects, camera,
physics, occlusion, gameplay behavior, cross-machine determinism, or complete
source content coverage. Interactive controls, resize and save/load behavior need
separate runtime checks.

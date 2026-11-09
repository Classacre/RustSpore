# Controlled earlier-vector to selected camera branch

The independently reviewed native fixture joins original E60750 earlier-vector
production and ramp to E60818 mode/selector dispatch, then for selector 1
executes E60A60's original `E833B0` reader of raw ID `98C942B2` and E60A76's
original B71CC0 actor lookup and interpolation. The ID has no established
semantic label here. Six controlled cases cover a null player, scripted null
provider, out-of-range state, an unrelated selector, valid actor and stale
generation. Other source dispatch targets are trapped. `E51760` is scripted;
the provider, aggregate, state, actor and pool are synthetic and supplied.
The host stops before E60B0D, and does not establish a live player, component,
provider, camera, or source-frame scheduling.

The original run matched its finite/range and source-output checks, exited zero
and closed the non-switched desktop after reaping. The earlier ramp is a
retained measurement, not independently exactly oracled in that host. Receipt
`target/cell-camera-dispatch-result-v2/receipt.json` SHA-256
`877b355c34470124b0f8ac1c610dbf268c5a6d326129ce353d861985d05d5b4a`.
Version 1 was not executed; peer review found that the standalone wrapper
needed to restore ESI, and the reviewed v2 adds that restoration.

`cell_camera_dispatch_v1` separately composes the checked earlier-vector,
raw-ID reader and actor interpolation. The provider return, post-provider
actor state, post-reader handle and readable slots are explicit snapshots;
other selectors and the next provider remain pending. Its comparator checks
the six retained B/C, ramp, callback count, lookup pointer, scalar and FP rows.
This finite controlled comparison does not establish actual pool ownership or
general provider/actor/camera behavior.

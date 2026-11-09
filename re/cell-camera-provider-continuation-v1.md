# Controlled provider-to-continuation camera chain

The independently reviewed native fixture joins original `E833B0`, the
`E60B0D..E60BAD` camera interpolation and `E60A3D..E60A5B` tail, followed by
original `E60C66..E60D03`, `E51230` and `1041E40`. A synthetic capture thunk
at the original tail jump records the already-produced reader value and vector,
then transfers to the original continuation. The resulting S+34 vector word
is the continuation ramp input. Four cases supply a synthetic aggregate,
current global vector and flag/vector state. The host stops before the camera
handoff; no live provider, actor, global or camera identity is established.

The original run matched its source/finite/layout checks, exited zero and
closed its non-switched desktop after reaping. Receipt
`target/cell-camera-chain-result-v1/receipt.json` SHA-256
`72971e73baced75028c710ae75f1bd7e1c0619429cc30ee3f72d47ff663b22a0`.
The original runner retained the ramp output as a native measurement, with no
independent exact helper oracle in that host.

`cell_camera_provider_continuation_v1` composes the previously checked reader
and continuation projections under explicit stable/disjoint state, CW 027F
and MXCSR 1F80. Its separate comparator checks the reader value, vector,
ramp result, output, flag and FP fields for all four retained rows. This
composition preserves the synthetic-state and capture-thunk limitations and
does not establish the active gameplay camera or following handoff.

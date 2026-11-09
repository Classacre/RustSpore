# Conditional camera continuation and ramp

The reviewed original `E60C66..E60D03` slice, `E51230` and `1041E40`
executed four controlled cases. They supply S+34 and synthetic mutable flag
and vector state; one case exercises the source's flag-clear initialization
to `(0,-0.5,0)`, while three preserve a supplied `(2,-3,4)` vector. The host
retained the original helper's binary32 result and checked its finite range,
the vector multiplication, source state and FP controls. It did not contain
an independent exact helper oracle. Native exit was zero and the non-switched
desktop closed after reaping. Receipt
`target/cell-camera-continuation-result-v1/receipt.json` SHA-256
`eb41d3b5a1292329a2d232df592ef083960344905f4f775da39fdc1f6f78c6d8`.
Live globals, provider/actor state and the following camera handoff remain
outside the original fixture.

`cell_camera_continuation_v1` is a separate checked composition using the
existing source-backed E51230/1041E40 retained-ramp model and explicit input
state. It admits CW 027F/MXCSR 1F80 and finite normal/zero reached values; it
retains the initialized/preserved global vector, helper result and ordered
three multiplications, stopping at the next handoff. Its comparator checks all
four original rows including helper result bits. This does not select the real
camera or establish the mutable source globals' values.

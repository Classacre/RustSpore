# Controlled provider-to-matrix camera chain

The reviewed original fixture joins the source data4 reader and camera
interpolation through the conditional E60C66 continuation and E60D03 handoff,
including original E51230/1041E40 and 10436F0 helpers. It ran four controlled
aggregate cases, with synthetic globals, up vector, manager/camera and scripted
setters. The earlier-branch S+1C/20/24 vector `(1,2,3)` was separately supplied;
the preceding joined source does not produce it. Two synthetic capture thunks
record intermediate words before continuing. The fixture stops before the
auxiliary update and does not establish live provider, actor or camera state.

Native exit was zero, finite/layout and scoped handoff checks matched, and the
non-switched desktop closed after reaping. Receipt
`target/cell-camera-fullchain-result-v1/receipt.json` SHA-256
`7e00b914d35718474520ca396cc5bf4e64672ecdc7ad8d124ad964fb2ce29266`.
Ramp and matrix words were retained native measurements, not independently
exact-oracled by the original host's checks.

`cell_camera_fullchain_v1` separately composes the checked provider,
continuation, handoff and matrix projections, including an explicit earlier
vector and host-binary64/PC53 arithmetic admission. The comparison example
checks every retained intermediate, matrix word and FP control for all four
rows. This limited comparison does not prove general x87 equivalence, actual
setter effects, a selected gameplay camera or original frame scheduling.

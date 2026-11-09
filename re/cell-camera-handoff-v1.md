# Conditional camera handoff: bounded source comparison

The guarded `E60D03..E60DEB` source slice ran three explicitly supplied
vector cases on a non-switched private desktop. All three matched the retained
CPU predictions, native exit was zero and the desktop closed after the child
was reaped. Receipt `target/cell-camera-handoff-result-v3/receipt.json`
SHA-256 `3967242fccb910a786fa2642515c741bb7a9e770343dfb5668f2b1d446480f8b`.
The independently reviewed host used a synthetic manager/camera, fixed up
vector `(0,1,0)`, scripted matrix helper and four scripted setters. The helper
writes and setter checks only the first and last matrix words; the original
matrix helper and camera state updates were not executed. It stops before the
separate auxiliary update. Native evidence does not establish live camera
selection, actor identity, upstream branch inputs or gameplay rendering.

`cell_camera_handoff_v1` separately projects the six ordered source additions
for caller-admitted normal/zero binary32 inputs under MXCSR 1F80. It takes the
up-vector snapshot explicitly and reports setter constants; the matrix setter
and auxiliary update remain pending. Frame, camera, callback lifetime and
return behavior are caller admissions. The comparison example reads all three
retained rows and checks vector bits, FP controls and the host's scripted call
count/marker. Its comparison is separate from native execution.

The initial build-only attempts are preserved in `target/cell-camera-handoff-prep-v1`
and `target/cell-camera-handoff-prep-v2`; they failed before native execution
due to command quoting and an assembler identifier respectively. The final source preparation and
static-only receipt are `target/cell-camera-handoff-prep-v3` and
`target/cell-camera-handoff-static-v3`.

The separately reviewed v4 joins the same original handoff to the full
original `10436F0` body. Its three cases include one nondegenerate direction
and two zero-direction controls, use CW 027F/MXCSR 1F80, and explicitly supply
the up vector. The synthetic setter captures all 16 output words. The run
matched its vector, finite-matrix, homogeneous-layout, stack and callback
checks; native exit was zero. Receipt
`target/cell-camera-handoff-result-v4/receipt.json` SHA-256
`40b26cee7f992ca2f349c80009f732954c0d9a0060a2c64391bf1096a8e0ba52`.
The inputs, manager, camera and setters remain synthetic, and neither actual
camera state nor the auxiliary continuation is exercised. Finite/layout checks
do not by themselves establish a mathematical matrix reference.

`cell_camera_matrix_v1` is a separate bounded Rust projection of the original
SSE operations and PC53 normalization using binary64 arithmetic and square
root. It explicitly admits matching host arithmetic and stable disjoint inputs;
its retained-row comparison is limited to these three fixtures, not a general
claim of x87 bit equivalence or a live view matrix.

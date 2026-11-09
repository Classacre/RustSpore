# Data4 reader to conditional camera vector

The reviewed original `E833B0` reader and `E60B0D..E60BAD` camera slice,
including its `E60A3D..E60A5B` tail, matched four controlled cases: missing
data4, 0.5, 2.0 and -0.5. The original reader uses its binary64 epsilon; the
camera slice caps values above the image's binary32 0.95 and interpolates the
supplied vector toward supplied mutable-global words. The host used synthetic
object/animation/pose aggregate storage and stopped before E60C66. Native exit
was zero; the non-switched desktop closed after reaping. Receipt
`target/cell-camera-provider-join-result-v3/receipt.json` SHA-256
`8c425408b68e9bb91ce8ddc377d21c722699b00c09750e860d615439a1529fb8`.
The retained v1 assembly and v2 near-jump audit failures occurred before native
execution. The fixture does not establish a live provider, active animation,
actual global vector, actor ownership or gameplay camera.

`cell_camera_provider_join_v1` composes the existing checked E833B0 data4
reader with source-ordered finite SSE clamp/interpolation under CW 027F and
MXCSR 1F80. Aggregate bytes, both vectors and lifetime/nonaliasing are supplied;
E60C66 remains pending. Its separate comparator checks all four native rows,
including reader value, vector words and FP controls. This comparison does not
prove actual runtime provider selection or the remaining camera calculation.

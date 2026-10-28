# Fresh Cell channel sampling: retained source comparison

The first sampled channel now has a bounded Rust/native comparison across five
actual installed animation resources. This is an animation-sampling result; it
does not complete Cell gameplay or live actor integration.

The corpus contains 300 eight-step histories and 2,700 snapshots. The source
host and Rust result agree byte for byte on the 4,896-byte semantic projection
per history, including the 440-byte sample payload and defined channel/key
event fields. The projection SHA-256 is
`b8b40b3a5da968d84059bf904ad14bda5f5f763c896acfad5907eb19a54fd868`.
The histories include 369 event occurrences, 12 x87 contexts and three incoming
SSE states. The native run also passed 28 rejection and five framing checks;
601 pinned files remained unchanged. The host and earlier runner have independent
reviews; the new corpus orchestration received an independent static review
after execution, not before it. A subsequent clean Rust rebuild in a fresh Cargo
target directory pinned 144 source, input and tool files before and after the
build and reproduced the same complete projection. Its evidence is
`target/source-cell/runtime-fresh-sampled-channel-pinned-build-v1/`.

The comparison covers fresh activation, explicit isolated serial/generation
state, the first source channel with lookup and sampling masks enabled, and
unchanged source channel counts. Other channels' masks remain zero. Event padding
and unused capacity are deliberately excluded from the semantic projection;
their original bytes remain in the ignored raw native output. Event dispatch,
actor association, scheduling and shared-resource lifetime remain open.

All 2,700 measured frames in the first corpus take interpolation branches.
Exact end-of-duration wraps to frame zero before sampling. A separate original
clip, whose last key is frame197 while playback extends to frame217, tests the
full-slot endpoint copy. Its 12 histories include 48 endpoint and 60
interpolation snapshots; all 108 Rust projections match the native output,
SHA-256 `41ee7f78b69a7552f55f7fda570b3b02629228956c9444c5a20347050c384c4e`.
A clean Rust rebuild reproduced that result with 145 source and input files
pinned; the native host and runner had independent prebuild and linked reviews.
This separate clip has no authored events, so endpoint events remain untested.

Earlier attempts remain under `target/source-cell/`: an event-shadow guard
incorrectly rejected retained buffer contents, and a coverage assertion
incorrectly required an endpoint in the interpolation corpus. The first endpoint
carrier then omitted one original slot-offset table cell, causing the position
copy to overwrite another field. The corrected carrier checks all nine slot
offsets and four copy lengths against the installed executable before use.

The original candidate remains frozen as
`crates/spore-core/src/cell_runtime_fresh_sampled_channel.rs`. The public
`cell_runtime_fresh_sampled_channel_v2` candidate adds explicit SSE setup error
conversion and matched the native corpus; it is not a complete gameplay API.
The verifier is `cell_runtime_fresh_sampled_channel_full_verify_v2`. Rebuild
through `tools/build.bat`; the original assets, native host, constants, inputs
and comparison output are local-only ignored evidence. The acceptance record is
`target/source-cell/runtime-fresh-sampled-channel-corpus-run-v3/rust-comparison-v3.json`.

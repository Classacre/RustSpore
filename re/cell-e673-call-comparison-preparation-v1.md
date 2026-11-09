# E673A0 call-input fragment comparison preparation

The post-arbitration source slice `E6A548..E6A592` was extracted from the pinned
SporeApp executable and built into an x86 host; neither the fragment nor game
was executed during preparation. Its 75-byte SHA-256 is
`0ce625c52a945160508c11f0c76ebde1678592b8d4a11d63b3b54b313f39d6c6`.
The subsequent reviewed run is documented in [the handoff note](cell-controlled-e673-handoff-v1.md).
Only its four-byte global-pointer operand at offset 7 is relocated to owned
host data. Static PE inspection found the rest identical and the pointer chain
`00408222 -> 0040300A +5190 -> 004081A6`, with the supplied +74 word.
This is an address-bound fixture, not an original global or live player.

The six finite cases exercise 3-4-5, signed zero, minimum subnormal, sqrt(3),
a large-value rounding case and mixed-sign coordinates. Each starts with x87
FNINIT/control 027F and MXCSR 1F80; actor buffers, synthetic root/state and
point are guarded. The source slice pushes two stack words and stores the
binary32 x87 result over the second; the wrapper captures distance, actor IDs,
current word, retained EBP point identity, x87 control/status and MXCSR, then
removes exactly eight bytes before return. The runner checks the source and
host pins, patched pointer chain, imported functions, model/native distance,
fixture IDs/current/point identity, x87 control and TOP, and MXCSR controls.
Sticky status is recorded but not compared. Outputs/errors and failure receipt
are retained in an exclusive run directory. Hash checks do not rule out a
transient replacement between reads.

Review before execution: `tools/cell_e673_call_prepare_v1.py`,
`tools/cell_e673_call_run_v1.py`,
`crates/spore-bevy/examples/cell_e673_call_model_v1.rs`, and
`target/cell-e673-call-prep-v1/host-x86.asm`, linked PE, corpus, manifest and
fragment. In particular check the patched pointer operand, actor byte ranges,
CALL/push/FSTP stack offsets, EBP lifetime, x87 control/TOP and output mapping.
Host SHA-256 is
`1b94d9bd4e6adf183fbfd19931dc46e35978bda4cbe170c546df72c22cb25a5a`;
runner SHA-256 is
`b912a49266aadbe26bfa4581d4c5e7fb2f43550f3de7e974c0f9f97bc68b1ec7`.
After independent review, the proposed invocation is:

```powershell
python tools/cell_e673_call_run_v1.py b912a49266aadbe26bfa4581d4c5e7fb2f43550f3de7e974c0f9f97bc68b1ec7
```

A pass covers only these cases and call-input words. It does not execute
E673A0 or validate real actor/point identities, callbacks, native exception
status, container changes, event dispatch or gameplay. The owned fixture's
synthetic point identity is assigned at producer-prefix time and carried
through its pending arbitration boundary; the controlled model still does not
establish a native pointer identity.

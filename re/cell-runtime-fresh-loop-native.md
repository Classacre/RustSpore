# Source fresh activation and private loop lifecycle

[MEASURED] Full fresh construction, direct Repeat setter, activation and clock
execution match 300 native/public Rust histories with 2,700 snapshots. The
2,442 original bytes include the complete activation, clock, delta advance,
channel-mask loop and finite CRT helper closure. Twelve x87 policies, three
incoming CSR states, 23 rejection protocols and five framing checks pass.
All 143 native execution pins remain unchanged. The verifier executes 6,300
typed guard checks covering seven rejection kinds.

The 792-byte history projection SHA256 is
`330a8cefef71111238dc28a6269efea7ac654009783982fc7629d69213b66509`.
Acceptance: `target/source-cell/runtime-fresh-loop-native-run-v1/rust-comparison-v1.json`.
Source plan: `target/source-cell/runtime-fresh-loop-source-plan-v2/plan.json`.
The frozen pre-admission plan-v1 and all original span bytes remain unchanged.

Fresh private state derives epoch 1 and crossing counter 0. Full activation
increments epoch to 2, installs selection 0, resets time/crossing/events, updates
serial and resource generation, then calls the original full clock. Absolute
steps and delta steps retain their original x87 commits. New integer crossings
update generation and increment epoch once when the source comparison succeeds;
backward or repeated times do not invent additional crossings. The returned
normalized fraction and the frame argument supplied to the channel loop are
separate observed values.

This gate uses five actual version-25 resources and their measured durations,
with all 7–11 source channels retained. Four zero masks suppress per-channel
sampling before wrapper/channel dereferences. Borrowed resource/wrapper binding,
fresh unbound owner 0, direct primary Repeat policy, isolated explicit global
serial/generation seeds and finite SSE2 CRT dispatch remain explicit boundaries.
Shared-resource interference, actor selection, live sampling, event dispatch,
registry routing and scheduling are outside this admission.

The first root assembly failed before linking because a PUBLIC declaration
exceeded MASM's line limit. A versioned wrapper splits those declarations while
preserving their symbol order and every other assembly line. The failed packet
and formatting proof remain retained. The host repeats full execution and
compares all output bytes; it does not load distinct XMM poison values.

The next source boundary is nonzero channel masks. Full 9A1590 first tests
channel key count +D4 and channel byte +AC bit 0, then calls 9A14C0. Its return
feeds 99F6A0 only when the separate sample flag and second mask also select that
channel. All five first channels pass the first two source predicates. That
proves the helper is reached, without proving that its event/lookup work is
empty. Exact 9A14C0 is the next requested dependency; mask/event bounds and a
native sampling host await that closure.
Exact 204-byte 9A14C0 confirms that the lookup uses the first enabled kind-0
INFO descriptor in authored order. It returns the first timestamp greater than
the current frame, or the key count. Its timestamp compare retains the x87
integer conversion; the assembly does not commit timestamps to binary32.
All measured timestamps lie in 0..405 and are exact under the admitted policies.
This corpus does not cover the high-unsigned timestamp conversion branch.

The fifth source clip, `8d26f08b5e…`, has genuine INFO events: key 2 at timestamp
6 has byte 1, and key 7 at timestamp 216 has byte 3. The other four first
channels have zero event bytes. Those authored bytes will remain intact. Event
append additionally requires a nonnull per-key epoch table, an older key epoch
and a nonnull event vector. Exact 9A0BB0 is requested to close bounded append
before selecting the next sampler boundary. Detailed observations are retained
in `runtime-fresh-loop-source-plan-v2/INFO-event-observations-v1.json`.

The full 4,820-byte sampler is now source-closed with the retained quaternion,
neighbor, endpoint-copy and finite CRT kernels. A separate source packet records
8,979 game bytes and 1,424 CRT bytes; this is preparation, without native
sampling admission. The immutable source image's rotation-policy word is one;
its runtime producer remains an explicit boundary.

Fresh wrapper defaults are no longer proposed as caller guesses. Full 99EA40
clears all 440 payload bytes, assigns the old payload serial at +4, then
increments global 15514DC. Full 9A1450 initializes the wrapper's other defined
fields and null epoch vector. Its vector-constructor callback at 99CC80 is a
29-byte straight-line closure to the exact RET at 99CC9C. Function-manager
metadata is absent; the failed query remains retained. Exact entry/RET
observations, literal callback use and independent byte/control-flow checks
support this bounded callback claim, without inventing a procedure boundary.

The new sampler draft preserves every payload byte across successive steps.
Endpoint copies replace whole 20/80/100/40-byte slots; interpolation replaces
only the defined numeric fields plus INFO bytes. Original channel installation,
epoch zeroing and cached INFO initialization can execute as declared partial
source spans over preallocated buffers. Full binder allocation, references and
nonzero-owner lifetime belong to a separate integration gate.

# Source fresh animation-state construction and reset

[MEASURED] Full original9A33C0/9A2FA0/9A1690/7D0230/9A1B40/9A1A20,
1,049 bytes in seven spans, match288 native/Rust cases.11 native protocols,
12 typed guards and three incoming CSR states pass. All59 execution pins
remain unchanged. The56-byte projection SHA256 is
`215e43a9c0db36d430760e16979df840f9f6768b48727ae2572603b93b49c0a1`.
Report: `target/source-cell/runtime-fresh-aid-sparse-native-run-v1/rust-comparison-v1.json`.

The private state derives epoch C0=1, crossing counter C4=0, inactive/nonloop
bytes AC/AD=0 and zero time/duration. Selection C=-1 and sampled wrapper14=0
also derive from the full fresh path. Original global serial15514E0 remains an
explicit input; the state stores the old serial and the global increments with
wrapping32-bit arithmetic. The corpus includes wraparound and12 x87 policies.

Fresh construction preclears owner/resource/binding/wrapper references, excluding
seven release/copy/allocator calls by source predicates. A subsequent bounded
vector reset uses original code, clearing length only for8-byte POD records;
capacity and underlying record bytes remain intact. These calls are proven
unreached, rather than described as seven installed trap stubs. Two repeated
calls test determinism; this host does not provide empirical dual-XMM poison.

The first allocation-based host failed with empty exit10 before input. Its
failure is retained. The reviewed sparse carrier retains RunOnce and all source
instructions while replacing startup page allocation with fixed PE sections:
7D/9A read-execute,148 read-only and155 owned mutable serial page. Whole-page
shadow comparison permits only the four serial bytes to change. Each new
protocol and the corpus ran once, with no retries.

[UNKNOWN] Resource binding, recycled references, activation, loop-clock updates,
event dispatch, live actor association and scheduling remain separate. Original
bytes and native outputs stay under ignored `target/`.

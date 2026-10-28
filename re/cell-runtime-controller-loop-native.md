# Source controller loop setter and resolver

[MEASURED] Full original A00310 and9FFE30 (183 bytes) match the Rust model in
536 supplied-state cases.14 native protocols, three incoming CSR states and15
typed guards pass; all34 execution pins remain unchanged. The28-byte projection
SHA256 is `b1e25a5cc24096367b690849a70997430141a630d9866d1145c3964c7297003f`.
Evidence is `target/source-cell/runtime-controller-loop-native-run-v1/rust-comparison-v1.json`.

The source resolver uses the handle's low byte and epoch to locate primary and
optional linked secondary slots. The bounded API admits indices1..16 and rejects
unsafe indices before native execution. Cases retain distinct primary/secondary
slots and aids, empty slots, stale epochs and direct/linked topology.

The setter writes primary slot+D4 and, when present, primary aid+AD. It preserves
secondary slot/aid bytes. A0C120 maps property+3C equal1 to loop1, otherwise0,
but A01A20/A00B30 can install the active aid in a linked fallback secondary.
That secondary may retain the forced loop1 from A00B30. Request mode alone
therefore does not establish the active secondary clip's policy.

[UNKNOWN] Actual queue construction, selected resource/actor, activation,
generation/crossing updates, event dispatch and scheduler order remain separate.
Original bytes and execution outputs stay under ignored `target/`; source data
and the installed executable are read-only.

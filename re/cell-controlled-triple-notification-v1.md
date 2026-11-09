# Controlled player triple notification prefix

Static decoding of pinned `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`
shows `E829D0..E829E1` passes a pointer to its stack triple through `401090`
to `4DF9E0`. `4DF9E9..4DFA02` copies the three words to receiver +A4/+A8/+AC.
If the first input word is nonzero, `4DFA0D..4DFA26` calls the object from
`67C680` through virtual +58, passing the input pointer and literal 1. At
`4DFA28..4DFA75`, source constructs message `03795725`, then rereads input
words in second/first/third order into its three message slots. The virtual
call and message constructor occur between the first copy and those reads;
their effects and input pointer lifetime cannot be assumed away. Subsequent
message dispatch and callbacks are not implemented.

The controlled source-player inspection reports the initial copied triple and
conditional pending +58 call. It previews message words only with an explicit
unchanged-input assumption; no original service is called. The model's message
projection accepts a separate fresh snapshot, permitting tests of changed input.
It does not reproduce concurrent mutation between the three source reads.

The actor passed to the saved-triple path is the result of the conditional
`E74A50` allocation in the Cell actor pool. On successful allocation and
stable owner/slot, `E751CB` publishes the returned handle at +411C, then
`E751DE` projects it via unchecked `B71D00`; this does not prove a live actor
or valid identity on failure. See [actor lifecycle](cell-current-player-handle-lifecycle-static-v1.md)
and [manager and sender trace](cell-player-actor-pick-producers-static-v1.md).
The actual provider, saved object, callbacks, pool lifetime and mounted winner
remain unresolved, so the inspection does not publish a gameplay actor.

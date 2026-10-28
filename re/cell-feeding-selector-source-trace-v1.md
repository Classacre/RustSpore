# Cell feeding selector production: bounded static trace

This records a static analysis of the installed x86 `SporeApp.exe` with SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. It extends [the event-dispatch trace](cell-feeding-event-dispatch-v1.md). REA/Ghidra dossiers cited below are in ignored `target/rea/20261028-source-runtime-v6/`. Names for unnamed source fields are descriptive and do not establish original types or intent. This is not runtime reachability or complete gameplay validation.

## Contact routes and candidate arbitration

`E7F170` calls `E73E10` twice, reversing actors and definitions, and shares one output; it later uses its selected output if the selector is neither 0 nor 0x16. For each call, let A be its first actor and B its second. `E73E10` tail-jumps according to whether each actor has *both* nonzero `+248` and `+FC`:

| A has both | B has both | Callee |
| --- | --- | --- |
| yes | yes | `E6ADE0` |
| no | yes | `E73660` |
| yes | no | `E73930` |
| no | no | `E73BF0` |

The listing shows JMP instructions, notwithstanding Ghidra's terminal-call flow warnings. The four branches ultimately pass contact candidates to `E6A450`. The first iterates both resolved feature arrays; the next two iterate one side against a fallback shape; the last compares fallback shapes and, when A `+FC` is nonzero, also invokes `E733D0` for additional feature candidates. The routines calculate transformed contact geometry, reject separated candidates in their distance branches, and pass feature codes/indices, definition `+B4` and `+B0`, and an `E57260` result to the shared evaluator. `E733D0` also reaches `E6A450`. Degenerate-distance paths and Ghidra's stack typing require additional validation before implementing geometry. Evidence: q253 (`ev_df8c3ea3f2c32c70f7808861c80960c2bfeade9437ec0f9696261043209e109c`), q481–484 (`ev_66867490aed15f731fd2be587be7d5c346dd9745616ca1a0796077c5510b1050`, `ev_8c8a38554927b80c3f1bbd27b4c6f1fc708849f52c7a276043a1a344c4eb6b66`, `ev_f5369212e4272c80e46ba4f850b712b0bdd0bcf065524fbb91d67288d6ed5b1d`, `ev_7808a35276a91acc74037671ae06b4526321babba6963b706487c78e5d3fa0af`), and q488 (`ev_1caa51a357935aafb97106ced0f71ab5564c5d2442b421e0a9221eeb9766124c`).

`E57260` takes each actor's word `+358`, calls `E4ECC0` for a value of -1, and returns `clamp((B_value - A_value) + 2, 0, 4)` with signed comparisons. The identity and fallback meaning of this value are not yet established. Evidence: q494 (`ev_b85740dfa6f1dbef601b133803dcbe0792eb6bc8dbed9005357be0eaed92ff58`).

`E6A450` has many ordered branches and calls `E59B40` to submit candidate selector values. Two bounded examples are:

- Candidate 9: after earlier exits, A feature code is 3, B definition `+B4` is 3, the `E57260` result is 1, and A byte `+112` is zero.
- Candidate 10: after earlier exits, A feature code is 2, B definition `+B4` is 2, the relation is 0, and both actor bytes `+112` are zero. A separate branch also emits 10 under different conditions.

These are candidate submissions, not exhaustive rules or a guarantee that 9/10 wins. Earlier branches include rejection of feature code 9 and special cases for definition codes 6 and 7; other candidates and `E6A2B0` may modify the shared result. Side effects of intervening native calls and both reversed passes matter. Evidence: q487 (`ev_83ad54e5a1a5eb26edb1d0b01da42140ddf31159660cd405c481090bc0df4c48`).

`E59B40` updates the shared output when the candidate selector is greater than the stored selector by signed comparison, or when equal and the candidate contact depth compares strictly greater (`COMISS`/`JBE`). For selector 8, an additional rule overwrites when the candidate's first actor ID is the current player ID and the stored selector is 8, irrespective of the depth comparison. A winning update copies the two IDs, two feature indices, depth, point, normal, and selector. For equal selectors, unordered floating comparisons do not pass the normal depth comparison; floating exceptions and edge behavior still need a separate admission if reproduced. Evidence: q493 (`ev_ca9d95f9fe0f313cabe5c27eec324c49da02888e50e70e66447be17def314e8e`).

## Record scheduling and lifetime

`E7E7D0` obtains an iterator from the pool at global `+54`, repeatedly calls `B71D20`, and passes each nonzero result in EAX plus its own float argument to `E7E6F0`. It does not establish the update cadence or iterator behavior under mutation. Evidence: q496 (`ev_35ec0b8184fc6a4fd6c5ca1f2ce38d09b211aeb5e03d1120ee1b7a53308a7788`).

`E7E6F0` receives a record in EAX and a float argument. It subtracts that float from record `+1C`, then calls `E59A90`. If that call returns zero it calls `E7E160(float, 0, 0)` immediately. Otherwise it walks four float fields at `+0C..+18`; a positive field crossing below zero is clamped to zero and invokes `E7B570`, whose zero return leads to dispatch. Afterward the record `+1C` test, current special handle `global +5198`, and `E7BA60` result determine whether it returns or dispatches. This establishes local control flow, not wall-clock timing or the cadence of the enclosing update. Evidence: q485 (`ev_1a69bc8459da0bc31d98327829149ac219d74da0d653d656c49c460df047ecfa`).

`E7F580` and `E7F5D0` resolve and clear global `+5194` and can invoke `E7E160(0, 1, 0)` on the resolved record; the former then recurses. Their full reachability and chain lifetime are not established. Evidence: q489–490 (`ev_c204b8a11a941e3cbb4e974c60c0686fce43798a6799ae0fee951337b2de1713`, `ev_770801749111414cce3b94cde611dd38a3f95b0b16cf013772f96c483d20cdcf`).

As previously traced, `E7E160` calls the record callback only when `+08` is zero, `+04` nonzero and the third argument's low byte zero; after callback return it writes `+08 = 1` and reads the *current* selector and fields. Selector 1 calls `E7D830`, then calls `B71D50` with the record's handle. `B71D50` links the low-16-bit slot into a free list and decrements the pool count, so this path releases the slot after the handler; neither callback safety nor correspondence to the original construction is guaranteed merely by allocation. Callback or handler may alter external state and potentially alias the record. Evidence: q236 (`ev_a66d55179cd6b40f2611da8f3a031053ff7b39284a7b155d1d236a67d268ff58`) and q486 (`ev_6bad75047db0a5798ab7dfbe55f8444b7928800d816947ba563bd877b684dc33`).

## Narrow implementable slice and open questions

A faithful next slice can model only `E59B40` arbitration over explicitly supplied, ordered candidates and an admitted current player ID. Preserve signed selector ordering, the selector-8 override, raw candidate payload copies and the exact ordered floating comparison; reject unsupported float environments or model IEEE unordered behavior explicitly. A supplied candidate list is an input, not a claim that the native geometry or `E6A450` produced it. Tests should cover larger/lower/equal selectors, ordered equal/greater depth, unordered depth if admitted, the player selector-8 override, and reversal/order dependence. This slice does not establish construction, callback survival, or feeding.

The public Rust model now matches a controlled native comparison on all 314
cases: signed priorities, ordered and unordered float inputs, and the
selector-eight player override. The 12 final output words match in each case;
four focused Rust tests also pass. The retained local report is
`target/cell-arbitration-oracle-review-v3/results.json`, SHA-256
`702d0cb939cca24a2eda00a3213d3af3eafd306c738798fb614e7a0eb3742bae`.
The source routine and generated host remain under ignored `target/`, with
independent static source and linked-host inspection. The host uses non-aliasing
buffers and MXCSR `0x1f80`; its comparison does not establish both write-path
flags, unmasked exception behavior, candidate generation or live gameplay.

Remaining work includes complete ordered `E6A450` and `E6A2B0` behavior; `E4ECC0` fallback and actor field meanings; geometry and degenerate cases; all intervening writes, indirect calls and event update scheduling; pool handle validity and callback/handler aliasing; and live reachability. The relationship between candidate contact output and eventual selector-1 event remains conditional on selection, `E7E820` gates, and record lifetime.

# Cell indexed-input target helper E5AF20: bounded numeric trace

Static PE evidence from `D:\Games\SPORE Collection\SporeBin\SporeApp.exe` SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. This extends [the indexed input target route](cell-indexed-input-player-target-static-v1.md). It does not observe a player, a key event, helper execution or a frame. Call-site aliases, callback effects, runtime values and FP controls remain admissions.

## Call and early paths

At `E5B428..E5B459`, caller supplies, in cdecl order, output three-word storage, current state vector A at `+CC/+D0/+D4`, normalized candidate vector B, and the incoming binary32 frame scalar. The caller stages the scalar first, then pushes the candidate, state and output addresses; tracking the changing stack pointer is necessary to distinguish A from B. `E5AF20` returns the output pointer in EAX on normal paths. This is an operation on supplied values; no specific control intent is needed to describe it.

`E5AF27..E5AF63` computes binary32 A length-squared as `(A.x*A.x + A.y*A.y) + A.z*A.z` in that SSE order. If image threshold `01485C9C` is ordered greater than that result (strictly less magnitude-squared), `E5AF65..80` copies B's three floats through x87 `FLD/FSTP` to output and returns. Equality takes the other path. `E5AF86..DB` separately computes `A-B` component-wise and squared difference as `(dy*dy + dx*dx) + dz*dz`; if threshold is ordered greater than that, `E5AFDD..F5` copies B.x via SSE and B.y/B.z via x87 and returns. Unordered comparisons do not take these early branches: `COMISS` sets flags that make `JBE` branch to the non-early path. Signaling/quiet NaNs, x87 stack/traps, unmasked exceptions, aliasing, short or invalid pointers and partial writes are not modeled by treating these as ordinary vector copies.

## Angle-related path

On the remaining branch, `E5AFF6..E5B07E` loads A.x, then B.x, each into x87 and calls `011E06AC`; the thunk jumps through import `013CC484`, identified in the PE import table as `MSVCR90.dll!_CIasin`. Each result is committed to binary32. For each vector independently, a positive Y chooses its committed asin result; otherwise source constructs `signed_pi - asin`, with `signed_pi` initially image `015A8C24 = 40490FDB` and sign-flipped when the committed asin is negative. The comparison instructions and signed zeros matter; this is not a validated replacement with a host library `atan2` or abstract shortest-angle operation.

`E5B081..C3` calls `E50B10` with the A angle, B angle and a pointer for a binary32 magnitude. `E50B10` first performs `SUBSS` B minus A, then repeatedly adds the runtime word at `016B4DB0` while `-pi >= delta`, and repeatedly subtracts that same word while `delta > pi`, with separate binary32 commits; `pi` is the image constant above. It stores x87 `FABS` of the resulting delta through the pointer and returns x87 `-1.0` if delta is ordered negative, otherwise `+1.0`. This word is runtime input: no particular period, positivity or finiteness is proven by the helper itself. Zero, wrong-sign, nonfinite or insufficiently advancing period values can cause nontermination or exceptional behavior. NaNs and unordered comparisons also require explicit analysis rather than real-number simplification.

The caller commits that sign and compares the stored magnitude against the separately rounded `MULSS` of image constants `015A8C1C = 3F490FDB` and `013EF544 = 3F8CCCCD` (`E5B0A3..C3`). For finite ordered comparisons, magnitude greater than or equal to that product returns B via the same SSE-X/x87-YZ copy sequence. Otherwise it forms `A_angle + sign * scalar` with separate SSE multiplication and addition, calls `E50AB0` to apply the same period-wrap style, and commits the returned x87 value. It calls `E50B10` again using that wrapped candidate and the B angle; `FUCOMPI` and the following flag test select between the wrapped candidate and B angle (`E5B11B..139`). This test is instruction/flag based and its exact unordered behavior must be preserved; it is not described here as a general-purpose interpolation rule.

`E5B139..150` calls `E5AE70` with the selected angle and output pointer. That helper first writes the three runtime baseline words at `016B4C08/0C/10`; its sign/quadrant branches compare the angle with image half-pi `015A8C20 = 3FC90FDB` and sign-flipped half-pi. The branches execute x87 `FSIN`, commit X while retaining the x87 value, square that retained value, calculate a square root of a one-minus-square expression using the actual `FSUBRP` opcode, and store a signed Y; the latter two branches explicitly write positive zero to Z while the positive-half-pi branch retains the baseline Z. x87 precision, rounding, exceptions, stack state and `FSIN` range-reduction status matter. No assertion that the incoming values form unit vectors, or that the baseline Z is zero in a live environment, follows.

## Controlled implementation boundary

A practical staged fixture can first model just the two early paths for finite explicitly supplied A/B words, with exact SSE order, masked x87 exceptions, sufficient x87 stack and FP-status exclusion. It must include threshold-adjacent values, different operand-order effects, signed zeros and transactional rejection of unsupported nonfinite inputs; even these fixture admissions are stricter than native behavior. A complete path requires a source-bound implementation or separately admitted results for `_CIasin`, exact `E50B10` / `E50AB0` binary32 loop ordering and finite advancing runtime period, `E5AE70` x87 `FSIN` and stack sequence, sign/flag behavior, and explicit `MXCSR`/x87 control and exception policies. It should bound iterations and reject nonprogress in the model rather than silently invent native output. Helper or model agreement does not establish retail execution, actor ownership or gameplay.

`cell_input_target_early_v1` implements those two finite early paths and returns
`NotEarly` for the remaining angle path. A reviewed hidden 32-bit host ran the
original `E5AF20..E5AFF6` instructions on two fixed synthetic cases: one takes
the small-current path, and the other has a non-small current vector and a small
difference. The Rust model matched the inferred branch identities and all six
output words, including signed zeros and subnormals. The receipt is
`target/cell-e5af20-early-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `1c947a44331584e143067df4e68aac215f11e4caefebd55be18c7ced8c014a34`.
The branch identities follow from the fixed inputs and checked source control
flow; they were not instrumented. The long path, live actor and scheduling
were not executed. FP sticky status is reported by the host but excluded from
the Rust result. Host/source endpoint hashes do not prove continuous stability
or complete build provenance, and the host mappings had no guard pages.

`cell_input_angle_wrap_v1` separately models the finite `E50AB0` and `E50B10`
loops for an explicitly supplied positive period, adding checked iteration and
nonprogress limits that the native instructions do not have. A reviewed
original-instruction host and the Rust model agree on four wrapped results and
three sign/magnitude pairs from seven fixed synthetic cases, including both π
boundaries and signed zero. The receipt is
`target/cell-angle-wrap-native-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `dd8b76071f0a506c2a9ce6a21ff3749b56f07bbc8c269473a7bfb5b974fa40a4`.
The host did not capture the difference delta, and the real runtime period,
CRT dispatch, full angle helper and live input remain unestablished. Host and
source endpoint hashes do not prove continuous stability or full build provenance.

`cell_input_angle_vector_v1` models the finite `E5AE70` angle-to-vector helper
with explicit x87 and MXCSR controls. A reviewed original-instruction host and
the Rust model agree on all 18 output words across six fixed synthetic angles,
including signed zero, both half-pi boundaries and both outer branches. The
positive-outer case retained its supplied baseline Z. The receipt is
`target/cell-angle-vector-native-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `9f7198a32655d88e2f022668f1236f59b7a80b265900c19f4e7a88835df5c256`.
Branch identities were inferred from the fixed inputs and source control flow;
the host did not instrument them or capture FSIN's immediate C2. This does not
establish the live baseline, actor, angle producer, or full helper. Endpoint
hashes do not establish continuous stability or complete build provenance, and
the host mappings had no guard pages.

`cell_input_target_angle_v1` now composes the finite long-path arithmetic from
two caller-supplied, already committed `_CIasin` results, the period, scalar and
baseline. A reviewed original-instruction host substituted fixed x87 return words
for `_CIasin` and ran three synthetic cases covering the inferred direct-copy,
stepped and overshoot selections. Rust matched all nine resulting vector words;
the native branch identities and FSIN's immediate C2 were not captured. The
receipt is `target/cell-e5af20-long-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `05147d82e102eac006c3617cfcc832052472dfa66436469af06d0a4f6162076e`.
The stub is not the CRT, and its supplied results are not checked against a live
vector. Hardware FSIN, FP status, stable paths and the host's absent guard pages
remain limits; this does not establish live input, actor binding or gameplay.

## Reproduction

Exact PE bytes and instruction-aligned disassembly are retained in `target/cell-e5af20-helper-disassembly-20261105.txt`; the file was created exclusively from the hash-checked image without executing it. Decode `E5AF20..E5B155`, `E5AE70..E5AF17`, `E50AB0..E50B04`, `E50B10..E50BA9`, `11E06AC..11E06B2` using `pefile` and x86-32 Capstone, and read import slot `013CC484` plus the constants above. Cross-check against the caller `E5B3D5..E5B4D7`. The initial static trace did not execute the game; the subsequent bounded comparisons are described above.

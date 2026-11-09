# Generated skin: query, ranking, and smoothing boundary

Static x86 trace of `SporeApp.exe` SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. This extends [the 52B700 producer trace](creature-generated-skin-producer-static-v1.md). It identifies a bounded next data transformation; no live editor or actual baked regeneration was observed.

## Geometry query input

At `52B8E6..52B90C`, `52B700` takes position number `i` from the geometry pointer supplied as argument 7, adds `12*i` to its +08 position-array pointer, and passes that point and a temporary vector to `4F9C30`, with receiver supplied by argument 6. It then consumes the temporary vector; a shared object identity with the export call still requires separate evidence.

`4F9C30..4F9C94` calls `4F9630` with the point. If it returns null, the helper adds no records on that route. Otherwise it reads a count from returned pointer -08 and calls `4F8AE0` on each 0x80-byte record in increasing order; output-vector state, record allocation validity and helper effects are required admissions. `4F9630` passes `(point, receiver+50 pointer)` to `41D9B0`, which subtracts the latter three binary32 components from the former with separate `SUBSS`. It passes the result and receiver+4C pointer to `4FCBD0`, which separately multiplies corresponding binary32 components. `4F96D4..ED` uses `CVTTSS2SI` for each coordinate, truncating toward zero; `4F9720..47` computes the 32-bit expression `z + 8*y + 64*x` and uses an **unsigned** `< 512` check before indexing receiver+54's pointer table. Individual coordinates are not independently range-checked, so an independently supplied grid model must preserve 32-bit arithmetic and reject/declare unsupported exceptional conversions rather than silently clamp axes. The receiver, grid construction, stored list pointers and original position source are still unknown.

Each 0x80-byte `4F8AE0` record holds four lanes. For each lane in order, finite admitted inputs take the following source path: a zero at +30+4*lane skips; otherwise `4F8B20..A5` computes point minus the three component arrays at +00/+10/+20, squares each difference, and forms `(dx² + dy²) + dz²` using ordered scalar SSE. `COMISS` at `4F8BB6` admits only distance squared strictly below the lane's +30 value for finite ordered inputs. It then evaluates `t = distance² * lane(+40) - 1`, `q = t*t`, `weight = (lane(+50) * q) * q`, each in recorded instruction order. When the x87 ordered positive test succeeds, it supplies `(lane dword +70, weight)` to the vector append helper `475DE0`. `+30`'s initial zero test and positive-result test use x87 comparisons; flags, x87 stack, control, exceptions, malformed vectors and nonfinite/unordered cases are separate boundaries. The table value's semantic identity is not established by the slot alone.

`creature_generated_skin_kernel_v1` models that finite four-lane slice. Four
fixed synthetic fixtures matched the original instructions' ordered append
words, including a strict cutoff, a subnormal and a one-bit multiplication-order
case. The reviewed host relocated the source, patched its `1.0` constant
address and substituted a checked append sink; it did not exercise native
allocation, spatial lookup or a live editor. The receipt is
`target/creature-skin-kernel-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `cc9c268f7cc20561fa91577836705a68dd6b6db7c868d1bb7bc951063580a731`.
No guard pages were used; endpoint hashes do not establish continuous stability
or complete build provenance.

## Accumulation to four staged records

The larger `52B700` loop initializes an accumulation vector, then processes the query's eight-byte `(index,weight)` records in order. Its branches use the index's high bit and separate +40/+54 arrays to distribute values, with optional remapping through caller-provided integer arrays; an extra positive scalar branch can replace weights using row-derived geometry. The full math, row-to-index interpretation, aliasing, helper calls and source input lifetime are not yet recovered. The retained decompilation `target/ghidra-creature-entry-v1/skin-producer-52b700-v1.log` is a lead, not a validated whole-function implementation.

A concrete smaller boundary is independently visible at `52C1A5..52C2D7`: `52C9F0` prepares four eight-byte entries initialized with `(index 0, weight +0)`. The outer loop scans the caller's accumulated weight array by increasing unsigned index. For each weight it scans the four current entries from the front; `COMISS(candidate, current); JBE` skips an insertion for ordered `candidate <= current` and unordered comparisons. On the first strictly smaller current weight, `52C6A0` inserts `(index, candidate)` there and the caller reduces vector end by eight, dropping the last element. With valid distinct vector storage, successful allocation/insert helpers and **finite** admitted weights, this selects the four highest positive weights in descending order, preserving the earlier index when weights tie; nonpositive candidates leave +0 placeholders. It then reaches the already compared `52C2D7..52C3DF` record writer. Fewer than four positive values are not silently padded with a made-up palette: the source placeholders retain index zero and +0 weight; if all four remain zero, the downstream native divide has zero denominator, while the checked Rust record model deliberately rejects it.

This makes a practical source-backed bridge from an **explicit post-accumulation weight snapshot** to the four candidate entries accepted by `creature_generated_skin_record_v1`. A bounded implementation can admit stable nonaliasing snapshot storage, finite weights, masked SSE with a declared MXCSR policy, successful temporary-vector behavior, source-ordered index generation and no positive smoothing passes; construct a fixed four-entry array with the same strict comparisons/insertion/drop, then use the existing record writer. It must retain native 32-bit index words until that writer truncates to a byte, bound lengths before allocation, reject unsupported zero-sum/nonfinite results transactionally, and test equal ties, fewer than four, all zero, order and index truncation. It should not call the weights source-derived or equate these indices with CreatureData row ordinals or animation palette entries without an independently proven identity and input provenance. Such a bridge does not evaluate `4F9C30` or reproduce a surface.

`creature_generated_skin_ranking_v1` implements that bounded bridge. A
reviewed host executed the relocated ranking instructions on four fixed
synthetic arrays; all 32 index/weight words matched Rust. Its insertion call was
replaced with a checked fixed-capacity substitute, so the native allocator and
insertion helper were not tested. The receipt is
`target/creature-skin-ranking-host-prep-20261105/reviewed-fixed-result-v1.json`,
SHA-256 `8dbfc96db366d0b43182ec040893c936ee31410cda9ef64f971e66c579a39f19`.
There were no guard pages, and endpoint hashes do not establish continuous
path stability or complete build provenance.

## Optional smoothing

When the signed iteration count is positive, `52C3E4..52C4C8` sets up two 20-byte-record buffers and repeatedly calls `52B270`, swapping source and destination each pass; do not treat the unsmoothed four-entry writer as the final record on this branch. `52B270` walks geometry +08/+0C positions and starts each neighborhood from the +A8 index array. It traverses linked topology through +94/+58 and byte tables `013F1890`/`013F1898`; for neighboring positions it computes a three-component dot product, and only when it is strictly above the image word at `0150DB00` (`3F5DB22D`) calls `52B420` with its separately squared weight. It then calls `52B530` with the supplied smoothing parameter, accumulates the current record through `52B420` with `1 - parameter`, and calls `52B5E0` for its output. `52B420` merges consecutive positive-weight source influences by index byte; `52B530` and `52B5E0` perform additional normalization/ordering. These are bounded static leads; source loop termination, duplicate/link semantics, full FP operation order, allocation/capacity, helper sort and actual geometry state require separate analysis. The linked cycle is not safe to reconstruct merely as an undirected mesh-neighbor set.

## Reproduce

Hash the named PE, map VAs through image base 400000h and PE sections, and decode x86-32 `4F9C30..4F9C95`, `4F9630..4F9756`, `41D9B0..41DA40`, `4FCBD0..4FCC5E`, `4F8AE0..4F8C3E`, `52C1A5..52C2D7`, `52B270..52B41B`, `52B420..52B6F8`, `52C3E4..52C4C8` and helper `52C6A0`. The first ranking's instructions and the query loop were checked directly; smoothing descriptions relying on helper pseudocode are explicitly conditional. Headless Ghidra 12.1.4 read-only/no-analysis output is retained in `target/ghidra-creature-entry-v1/skin-candidate-query-selection-v1.log` and `skin-candidate-query-helpers-v1.log`; Ghidra reported stack/global warnings and cannot establish runtime identity. No original process was run for this trace.

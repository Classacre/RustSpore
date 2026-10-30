# Blockdata multidimensional query: static implementation plan

This follows [blockdata-native-sample-static-v1.md](blockdata-native-sample-static-v1.md). The source is installed `D:\Games\SPORE Collection\SporeBin\SporeApp.exe`, SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`; addresses are virtual addresses. Findings are static x86 instructions, not an executed original query. They establish neither a terrain role nor a particular gameplay use.

## Query setup and coordinate policy

`414DC0` finds the buffer by the key described in the prior note. If the key is present and its first output pointer is null, it returns true without calling the evaluator. If the key is absent and the first output pointer is null, it returns false without writing outputs. With a non-null first output, it assumes the second output pointer is usable, initializes eight index pairs to zero and calls `414FC0`.

`414FC0` counts consecutive nonzero unsigned bytes from buffer +28, at most eight, and computes their product using 32-bit `IMUL`. For each dimension `i`, its stride is the 32-bit product of later dimension values. For `i < supplied_count`, it obtains coordinate `coord[i]`, uses `MAXSS` with zero then `MINSS` with one; for missing coordinates it uses +0.0. Extra supplied coordinates are never read. In normal finite cases this clamps to `[0,1]`.

`4151C0` uses `n = dimension[i] - 1`. If `n < 1`, both offsets are zero and the weight is not written. Otherwise it computes f32 `scaled = float(n) * coord`, `lower = CVTTSS2SI(scaled)`, f32 `weight = scaled - float(lower)`, and `lower_offset = lower * stride`. If `COMISS(weight, +0)` followed by `JBE` is not taken and `lower < n`, it selects `upper_offset = lower_offset + stride`; otherwise both offsets are lower. Dimensions of one therefore have identical endpoints and never use their unwritten weight. For ordinary finite coordinates this amounts to interpolation on `n` intervals, with coordinate one selecting the last point without an upper neighbor. Negative and greater-than-one input are clamped. The MAXSS/MINSS operand order matters for NaNs and signed zero; do not replace it with a language clamp and claim bit parity for special values. The code can dereference a null coordinate pointer when the supplied count requires it; a safe API should not copy that unsafe behavior.

## Evaluation and arithmetic order

For zero dimensions, `415184..4151B0` copies the first two header triples to the output directly. Otherwise `41516B..41517D` calls `4152A0` with accumulated index zero and dimension zero. At every recursion `4152A0` first compares the accumulated index against the stored total **as unsigned**. An out-of-range index writes the fallback triples `(-0.1,-0.1,-0.1)` and `(0.1,0.1,0.1)` from image constants and returns. If the index is in range and all dimensions are consumed it calls sample decoder `415440`. Otherwise it first recursively computes the lower-offset result. Only if the two offsets differ does it recursively compute the upper-offset result and blend the two returned triples. Thus lower branches run first, and the last dimension is evaluated innermost.

For each component of either triple, `4153C1..41542E` calls `413C50(lower, upper, weight)`. `41D9B0`, `41DA40` and `41DAB0` perform the following **separate SSE scalar operations**, in this order:

```text
D = f32(upper - lower)
M = f32(D * weight)
result = f32(lower + M)
```

`413C50` then loads and stores the resulting f32 values via x87. Do not use `lower*(1-weight)+upper*weight`, fuse the multiply/add, or change recursion order if evaluating numerical agreement. The decoded endpoint itself is the separate f32 multiply and subtract/add described in the prior note. MXCSR rounding mode, denormal handling, x87 state, exceptional values and NaN payload propagation are not established, so static formulas alone do not establish bit parity with every original runtime configuration.

After evaluation `414EF4..414F07` compares **only component zero** of the second and first output using `COMISS`; it returns true only if the second is strictly greater. Otherwise, including unordered comparison, it overwrites both triples with the same `-0.1` / `+0.1` defaults and returns false. This wrapper-level validity check is separate from the recursive index fallback, and it is not an all-axis ordering check. A standalone buffer evaluator should make clear whether it models just `414FC0` or also this lookup/wrapper contract.

## Safe implementation boundary and tests

A bounded helper can build on `NativeSamples`: require exact source length and checked dimension products, reject nonfinite coordinates and nonfinite header/scale values in the supported API, use a coordinate slice so missing components are zero and extras are ignored, and compute checked strides and indices. Its arithmetic must remain ordered as above; explicitly document any policy for finite inputs whose intermediates overflow. These rejections are safety restrictions, not claims that the native routine rejects such data. The loader's size limit and exact-size policy do not prove native interpolation itself checks malformed payloads. Return the direct header triples for zero dimensions without sample access. Preserve native wrapper fallback as a distinct optional layer if that wrapper is implemented.

Meaningful tests should cover: two-by-three dimensions where changing each coordinate identifies last-axis-fastest indexing; endpoints, midpoint, and a multidimensional case where recursion order matters numerically; fewer and extra coordinates; dimension one; all-zero dimensions; clamp below zero and above one; malformed length and arithmetic bounds; wrapper behavior for equal/reversed component-zero outputs and valid component zero with reversed other axes. Compare intermediate binary32 bits against an independent reference that rounds each SSE operation and preserves order. A native reference under a captured floating-point environment would be a separate, stronger gate. This note provides a static plan only; it does not assert that such an implementation or native execution has been tested.

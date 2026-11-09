# Cell feature-pair contact geometry: static continuation

Read-only Capstone x86 decoding of installed `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`,
preferred virtual addresses `E6B06F..E6B1A9`. No original code or live game
was executed for this trace. The already modeled pair prefix and its six-case
comparison are in [the pair note](cell-controlled-feature-pair-prefix-v1.md).

Let A and B be the supplied first and second projected features, with centers
`Axyz`, `Bxyz`, radii `ra`, `rb`, deltas `B-A` and separately rounded squared
distance `d2` from the prefix. The executable constant at `01485C9C` has bits
`37800000` (2^-16); `014861D0` has `3F800000` (1), and `01471B2C` has
`3F000000` (0.5). `COMISS(threshold,d2)` followed by `JBE` takes the normal
path when threshold <= d2 or comparison is unordered. The following statements
assume finite ordered inputs, masked exceptions and stable, nonaliasing inputs;
they do not establish runtime FP controls or exception behavior.

For `d2 >= threshold`, `E6B099..A2` loads binary32 d2 to x87, performs FSQRT,
and stores binary32 distance. `E6B0A6` performs one SSE subtraction of that
distance from the previously computed `ADDSS(rb,ra)` radius sum. `COMISS(0,
depth)` and `JA` skip the pair only when ordered zero is greater than depth;
equality reaches the point/normal construction. Do not simplify the x87
operation or its binary32 store into an unadmitted host expression.

Writing `M`, `A`, `S`, and `D` for individually rounded SSE multiply, add,
subtract and divide, the continuation `E6B0BB..E6B14B` computes:

```text
inv = D(1, distance)
nx = M(inv, dx); ny = M(inv, dy); nz = M(dz, inv)
dr = S(ra, rb)
px = M(A(A(Bx, Ax), M(nx, dr)), 0.5)
py = M(A(A(Ay, By), M(ny, dr)), 0.5)
pz = M(A(A(Az, Bz), M(nz, dr)), 0.5)
```

This expresses dependencies and operand order, not a claim that the native
instruction scheduler groups all independent operations in that order. The
normal words are written to the current frame's `+3C/+40/+44` at
`E6B0ED/F7/B101` and point words to `+48/+4C/+50` at `E6B13F/145/14B`.
Accounting for the 11 and 12 argument pushes at `E6B18F` and `E6B194`, their
LEAs refer to those respective frame addresses; `E6B19A` calls E6A450 with
normal and point pointers among the inputs. Source definitions, actor and
feature identifiers, relation and any side effects remain separate admissions.

For ordered `d2 < threshold`, `E6B08E` stores the radius sum as depth and
jumps directly to `E6B151`. It performs no normal or point stores. A linear
instruction decode from entry `E6ADE0` through `E6B1D1` found direct stores
to these six frame offsets only in the normal path above. This is a bounded
static observation, not proof that other calls or aliases cannot affect those
bytes. The first small pair can therefore see preexisting stack contents; a
later pair can carry earlier normal-path values or intervening call effects.
A reproduction must admit the actual six words, their identity and lifetime at
the call, and callback effects; it cannot assume a zero point or normal.

A bounded next model could cover only finite, nondegenerate inputs, use an
explicit x87 control/stack admission and scoped MXCSR, preserve each binary32
rounding and ordered comparison, and suspend before E6A450. Original-byte
comparison would require a separately reviewed host, exact source pins and
address relocation for the constants, controlled x87 state and stack, guards,
branch and output capture, and finite tangent/separated/rounding cases. Small
distance needs explicit carried local bytes and a separate alias/callback
protocol. Neither slice would establish actual actor/cache identity, pair
traversal, E6A450 side effects or gameplay contact.

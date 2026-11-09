# Controlled second-feature pair prefix

Static x86 decoding of the pinned `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`
shows the second feature projection at `E6AF72..E6B033`. It has the same
coordinate grouping as the first-side projection, but its final X product is
`center_x * m0`, rather than `m0 * center_x`. The checked model preserves that
operand order and uses the caller-supplied actor transform. A later bounded
six-case original-instruction comparison is recorded below.

At `E6B039..E6B08C` the source computes the second projected center minus the
first: x, y, then y squared, x squared, their sum, z, z squared, and the final
sum. It compares this squared distance with image word `37800000` (2^-16),
adds the second radius to the first, and branches. The checked prefix exposes
these values and the strict `distance_squared < 2^-16` branch under MXCSR 1F80,
rejecting nonfinite inputs and intermediates. It neither computes a square root
nor decides contact. Independent instructions can be scheduled differently in
the checked implementation; native sticky flags and partial writes are not
modeled. The earlier first-side six-case comparison applied to an earlier
pinned model file, not the expanded current source file.

The small-distance source branch jumps to `E6B151` without writing the local
point and normal at +3C..+50. They can retain prior stack contents or earlier
loop state; no zero/default point is justified. The other branch uses x87 FSQRT,
separation and normalization before constructing point and normal; both paths
then enter `E6A450` with definition fields and actor/feature identities, and
external calls may mutate later loop inputs. Actual execution additionally needs
both actor +248/+FC states, resolved cache rows, stable actors and transforms,
second definition, source FP state, row traversal and callback/lifetime rules.

For opt-in inspection only, the caller explicitly copies the first eligible
retained row as a synthetic second feature and supplies a second position
(102,100,0); this is not a second native actor or cache. The owned fixture
reports at most its first pair prefix and retains the first-side pre-override
geometry separately. No native actor, overlap, acceptance, contact point,
callback, event or gameplay is asserted.

## Original-instruction comparison

The reviewed host executed the pinned `E6AF72..E6B08B` fragment with its
four-byte threshold-address relocation and six fixed synthetic inputs. All nine
numerical output words and the branch derived from CF/ZF matched the expanded
Rust model; both processes exited zero. First center and radius were supplied
directly to the host; this run did not re-execute the first-side fragment.

| Case | Second input and first snapshot | Distance squared | Small branch |
|---|---|---|---|
| 0 | Both centers zero; both radii 1 | `00000000` | yes |
| 1 | First center zero; second X `3B7FFFFF`; both radii 1 | `377FFFFE` | yes |
| 2 | First center zero; second X `3B800000`; both radii 1 | `37800000` | no |
| 3 | First center zero; second X `3B800001`; both radii 1 | `37800002` | no |
| 4 | First center (1,2,3), radius 2; second position (10,20,30), row center (1,2,3), radius 2, scale 2, flag 02 and matrix words (1..9) | `46C4BC00` | no |
| 5 | First center zero, radius 1; second row center bits `00000001 80000001 80000000`, radius 1, zero position and scale 0.5 | `00000000` | yes |

Cases 0–3 use zero row center, scale 1 and flag clear; case 5 also has flag
clear. Output includes second center/radius, delta, squared distance and radius
sum. The retained raw rows contain every word and EFLAGS. The six MXCSR values
were `1F80, 1FA0, 1F80, 1FA0, 1F80, 1FB2`; control bits passed with sticky
flags masked, and sticky flags were not compared.

Receipt SHA-256
`34bcbcd68eee414a9cbb14a8012cedc2dea7feea7e82cad6ad551689d7162953`;
native stdout SHA-256
`d8db598cfc6250e8f978abd534202f4f482c0bd6abf175afe1907b85b8eb55ef`;
model stdout SHA-256
`bf02b6ab0da8a81d7b8c0b6c48dfa61605de3ab3d7a3418bd7f28db052b8fd6d`.
Files are under `target/cell-feature-pair-run-v1/`; fragment, corpus, host and
runner pins are in [the preparation note](cell-feature-pair-comparison-preparation-v1.md).
The compared expanded model source SHA-256 is
`ad980714263d2213e9830108fafd61ff995d5ac8281c654651e6e86797dc2280`.
This result establishes agreement for these six supplied cases only, not
exception parity, a stable-file guarantee against transient replacement, or a
native cache, actor, contact or gameplay result.

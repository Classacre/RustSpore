# Candidate screen-effect render path

This static trace concerns `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
It extends `cell-screen-effect-source-trace-v1.md`; it does not establish
that the candidate Cell skybox is selected, its texture is loaded, or any
branch is executed. The original executable was read without modification.

## Receiver and conditional paths

Constructor `6EDBE0` installs primary vtable `140B188` at whole object +0
and secondary vtable `140B150` at +18. The latter's slot +0C points to
`6F0DB0`; this method operates on the +18 subobject. Its call at `6F0E30`
passes `ECX = subobject - 18` to `6E81B0`, corroborating the adjustment.
Another conditional branch at `6F0ECE` reads subobject +34C, equivalent to
whole object +364, and calls `776D10` at `6F0ED6`. Its path is gated by the
caller flags, the object pointer, and the result of `776340`; the surrounding
switch and scheduling context still need full independent analysis.

The primary receiver's slot +68 is `6E7C60`. It sends mode values 5 and 6
to `776350` when whole-object +360 is nonzero; otherwise the eligible path
uses whole-object +364 and `7763B0`. `776350` writes three floats and a fourth
word to its target +120 through +12C. `7763B0` writes the same shape to its
target +12C through +138. These are distinct targets and their identical
offsets must not be merged. Mode values here are the forwarded arguments;
they alone do not prove which path a candidate takes at runtime.

`6E81B0` conditionally invokes `776910` on whole-object +360 at `6E8270`,
`6E857E`, and `6E8700`. The last path checks that +360 is non-null and that
the incoming flags contain mask `0x08`. Other gates apply to the earlier calls.
`776910` returns without the draw path if target byte +13A is zero. REA
dossier `814-analyze_function.json` identifies `776910` as a function from
`776910` through `776D01` (evidence
`ev_7f166ca495f6858d301de3da692c4af77d019c4299f6bc908e6ab058dba96493`).
The coordinating agent reports an independent sibling review of dossiers 814
and 815; the separate `6E8190` boundary was corrected during that review.
The unrelated primary-vtable slot +DC points to `6E8190`, a separate short
method ending before `6E81B0`; it is not an entry into the latter.

Within `776910`, the selected 16-bit value at target +138 controls branches.
For value 5, +130/+134 form the lookup key; a special key invokes a virtual
method on +1B4, while other non-sentinel keys go through the +140 map and
+160 pointer array. An unresolved lookup can return without drawing. Value 4
has a separate +174-dependent path. These fields are separate from the
four-word +120 tuple written by `776350`.

At `776CB8` the selected material-like object is passed to `11EE360`; when a
texture-like pointer is nonzero, `776CC6` calls `11F1030` with slot 0 and that
pointer. At `776CDD` the address of target +120 is passed to `761D60` with
the selected object and a caller argument. Before that call, `7C4950` must
return nonzero. The actual meanings of the selected object, texture pointer,
shader, blending, and sampler remain to be established.

The separate fallback `776D10` requires target byte +13C and a successful
`7C4950` call. It resolves its +120/+124 key and +128 selector, calls
`11EE360` and `11F1030`, then passes target +12C to `761D60` at `776DD6`.
It may return before drawing when a map-based resource lookup fails.

## Quad helper

REA dossier `815-analyze_function.json` covers `761D60..76201A` (evidence
`ev_02f8d5ba2dc8218ca9f538655671b371c7288c96d2a0497bad3d2799ff1b82ae`).
The function calls `4584E0` on its second argument, requests four records via
`6DD7B0(2, 4, ...)`, and on success writes four 24-byte vertices. In order,
the UV pairs are `(0,1)`, `(1,1)`, `(1,0)`, `(0,0)`. Their positions are
`(left,0,bottom)`, `(right,0,bottom)`, `(right,0,top)`, `(left,0,top)`,
where left/right are `-1`/`1` minus `2 * [16087BC] / width`, and bottom/top
are `-1`/`1` minus `2 * [16087C0] / height`. Width and height are 16-bit
fields +0C/+0E of the object reached through `11F0ED0` and +40. Those global
offset values and their runtime providers have not been resolved here.

The screen-offset initializer is now partially identified. On its successful
relevant startup path, `6DDA60..6DDAF5` reads global `015FE918`, then its
pointer +3C and word at +110. If that word is nonzero, it writes the source
float constants +0.5 and -0.5 respectively to `16087BC` and `16087C0`; if
zero, it copies words from `1608724` and `1608728` instead. These inputs are
conditional on object validity and initialization order; the active flag,
alternate values, viewport dimensions and later writes are not established.
Bounded instruction and constant evidence is
`target/source-cell/screen-offset-source-audit-20261108-v1.json`, SHA-256
`bf7368ab8c7b959aa3ac733ef189a9d146469a76dd742f07683dc8f68d57da92`.

The main screen-effect draw's camera-like receiver is constructed locally on
the observed initialization path, not simply the Cell world manager's +30
camera value. At `776E74..776EB5` it allocates `174h` bytes, calls `7C3910`,
stores the result at effect object +13C, and calls `7C4770(0)`. `7C3910`
initializes receiver +170 to null; successful `7C4770` calls `761140` and
stores that returned render-context pointer at +170. It copies global
`015D1838` into context +40, and initializes context +80/+84 from the 16-bit
words at that pointed object's +0C/+0E. Its zero argument takes the branch
that computes context +5C/+60 from source constants, another global and those
dimensions. This is initialization evidence only: the global pointer and
its dimensions are mutable and actual target identity is unobserved.

At `776CA9..776CB6`, the main draw calls `7C4950` on this +13C receiver; on
the path where its context +40 is non-null and `11F3CD0` succeeds, the latter
stores the context pointer in global `016FB5B8` at `11F3F9F`. `11F0ED0`
returns that global, which `761D60` uses to find +40 and the +0C/+0E words
for the screen quad. This joins the conditional draw's quad dimensions to the
locally constructed context's current target, without identifying it with the
main or auxiliary gameplay camera. Reassignment, lifetime, current dimensions,
device state and branch success remain admissions. Bounded source bytes are in
`target/source-cell/screen-draw-camera-audit-20261108-v1.json`, SHA-256
`b6b2be8115d9bcc7c1caf54964e68d3e4cbf9cb42befb844dd804ffe8e44aefe`.

Further bounded tracing identifies the target-storage origin and two later
dimension writes. At `11F843E..11F8454` the device-creation routine aligns
static addresses and writes `01713480` and `017134A0` into globals `015D1838`
and `015D183C`, then passes its supplied setup block to `11F7AB0`. On a
conditional setup-change path, `11F7DA9..11F7E04` copies the 15-word input
block to `01713444` and writes the low 16 bits of its words +8/+0C into +0C/
+0E of both target records. On the separate path `11F7E20..11F7EB2`, subject
to its mode and handle gates, the explicit 16-bit width/height arguments are
also written into both records. The words `0171344C` and `01713450` are the
same mode-block words +8/+0C that the independently traced graphics provider
can expose as its UI descriptor; a shared current-block relationship is
conditional on the relevant update paths, target lifetime and no intervening
mutation. It does not assert a particular viewport or runtime timing.
Evidence is `target/source-cell/screen-target-source-audit-20261108-v1.json`,
SHA-256 `cc9118e67c806f6f81286770fa79dba23945ab9d844f30627fbe465594d53988`;
the prior UI-side trace is [here](cell-ui-graphics-descriptor-source-trace-v1.md).

The conversion helper `4584E0` reads four floats and applies SSE max, multiply
by the verified constant 255, min, then `CVTSS2SI` conversion to integers,
and packs the resulting bytes at bit positions 16, 8, 0 and 24. The resulting
word is stored in each vertex at +0C. In the conventional component spelling
this is `0xAARRGGBB`. The SSE conversion is governed by MXCSR; NaN behavior,
rounding outcomes, color-space and blending are not established here.
The helper then calls `6DD800(3, caller_argument)` after successful allocation.
`6DD800` is a dispatch wrapper, not by itself proof of the final graphics
primitive. Dossiers 817 and 818 document these two helpers (evidence
`ev_b56959f132541066977d944f48627df1e38c9415d63d01804750b59fea55f68b`,
`ev_66a94fe55bf6a800d5c1e340c05c507ce72895e113785723ebc8cccd85123b30`).

## Next evidence

Trace the full `6F0DB0` switch and callers to establish scheduling and flags;
the creation and lifetime of both +360/+364 objects; and `11EE360`,
`11F1030`, `6DD800`, and `6DD030` through the actual shader, sampler, blend,
texture binding, and draw calls. Verify the `2F4E681C` provider request and
the runtime directory/resource precedence before assigning an installed
skybox texture to this path. Inspect `16087BC`/`16087C0` producers and the
viewport dimensions before claiming exact rendered coordinates. The trace is
static source evidence, not a visual or runtime acceptance test.

An independent read-only review found no blocking errors in the receiver
adjustment, gates, tuple-to-quad connection, vertex/UV layout, or cited dossiers.
That review did not execute the path or verify its shader.

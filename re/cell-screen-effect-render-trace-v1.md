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

# Cell animation request and palette follow-up

Static evidence below refers to the base executable with SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Original bytes and REA dossiers remain in ignored `target/`. This trace does not
establish that a particular installed Cell actor played a particular clip.

## Request to controller

[MEASURED] `A0C120` looks up its request-key argument in an unsigned-key tree
at global `01552A68` using `810230`. A missing entry or empty variant list
returns zero. With multiple variants, a valid caller-provided index is used;
otherwise `935F40` supplies the value compared against variant thresholds.
The chosen variant's 28-byte entries are converted to 12-byte triples with
the source words in +8, +4, +0 order. `A0C120` supplies these triples, the
controller at object+184, and a fallback pointer from
`*(*(object+190)+1C)+14` to `A01A20`.

[MEASURED] `A01A20` selects a free controller slot, clears it, copies up to
eight triples and initializes its controls. It calls `9FFA20`, falling back
to `A00B30` when that call returns zero. In `9FFA20`, the first word of each
slot triple is passed to `9A3680`. That function uses `810230` on the tree
pointed to by `0166D038` and returns its node+14 value, or zero. A missing
value reaches `9A37D0`, which searches another structure using the underlying
controller+FBC owner and the key. A found nonzero resource and successful
`9A4430` binding reach `9A3140`. Exhausting eight triples returns success
without itself proving a clip was installed.

[MEASURED] In the fallback `A00B30`, the slot address is calculated at
`A00B4D..A00B55`. The instance pointer is stored at `A00BA9` for a freshly
constructed instance or `A00BCA` for the pool path. The passed fallback
resource reaches `9A4430` at `A00BFE` and `9A3140` at `A00C11`; the clip
loop byte +AD is set at `A00C2B`. `A00D70` sets slot+C0 at `A00E12` or
`A00E21`; `A01F60` writes slot+94 at `A02159` and `A02291`. The object+184
controller and object+17C CID share construction context in `A0B110`.

[UNKNOWN] REA reports `A0C4FF` and `A0C52B` as references to `A0C120`, but
does not identify enclosing procedures at those addresses. Their actual caller
arguments, the production of object+190, population of both trees, and the
association with a live Cell actor remain open. The key and lookup result must
not be labeled an installed ANIM identity without a source-backed resource
trace. Dossiers: `target/rea/20261028-source-runtime-v6/613` through `618`,
and `626` through `628`; exact retained assembly for slot installation is
`target/source-cell/runtime-clip-association-trace-v1/lifetime-followup-v8/q48-assembly.txt`.

## Palette consumer boundary

[MEASURED] The previously established `9C6E40` model-world forwarding and
`740530` palette gathering are described in `anim-palette-native.md`. REA
finds references to `740530` at `7432AD`, `744A2E`, and `775BE3`. The first
belongs to `743250`, which gathers into a stack buffer and passes it to
`73A100` for each retrieved object. The third belongs to `7755F0`, where a
conditional branch gathers into a stack buffer later supplied to `73A100`.
No containing procedure was identified for `744A2E`.

[MEASURED] `73A100` discovers data streams via `71DB60`, prepares source
and destination stream descriptions, then forwards the palette argument and
those descriptions to `731CF0` at `73A3EB`, its sole reported reference.
`731CF0` dispatches on its second argument: value 7 selects `7300C0` when
the seventh argument's stream+4 is nonzero and its short at +8 is 4, and
otherwise selects `730B90`; values 8 or 9 select `7314A0`. Each receives
the palette argument. This establishes a downstream candidate, not a shader
constant upload or a completed CPU skinning operation: the three dispatched
implementations establish a CPU deformation path. Dossiers are
`target/rea/20261028-source-runtime-v6/620`, `622`, `623`, and `629` through
`634`.

[MEASURED] `7300C0` and `730B90` read byte indices and packed byte weights;
`7314A0` reads 16-bit indices and float weights. Per vertex, each obtains
twelve affine matrix floats beginning at palette + index*16 bytes. When
multiple influences are present, it forms a weighted matrix; otherwise it
uses the first. It transforms source XYZ into destination XYZ, and, when
present, transforms and normalizes additional direction streams. `7300C0`
uses packed byte direction data while `730B90` and `7314A0` use float
directions. This is a CPU skinning operation on source and destination
streams, not evidence of a shader constant upload. Exact floating-point
behavior, stream selection and every branch remain unvalidated by an
independent implementation.

[MEASURED] `71DB60` selects matching entries from the geometry object's
32-byte stream descriptors, with caller-provided matching fields and wildcard
values; it returns whether all requested entries were found. `734CF0` can
perform further conditional transformations and replace stream storage on the
resulting object, so the output of `73A100` is not necessarily the final draw
buffer. `743250` has reported references at `466F55`, `7435BA`, `74A5AC`,
`74A62F`, and `74A6FE`; their kinds and call contexts have not been resolved.
Dossiers are `target/rea/20261028-source-runtime-v6/635` through `637`.

[UNKNOWN] The native Cell preview's current draw supplies static source
vertex bytes and its input structure contains no animation palette. A bounded
diagnostic could eventually deform a copied buffer with an explicitly supplied
palette, then draw it with an established compatible non-skinning declaration
and shader; its output must not be passed through a second skinning stage.
Before source-equivalent integration, identify actual stream selection at
the stream descriptor construction, `743250` callers, and the unresolved
`744A2E` context. The original SkinBones uniform/register producer remains
unestablished. Nonzero-child pose traversal, final commit, actor request
identity and original scheduling remain separate gates.

The installed Jawhead LOD0 model `40617100780A9EAF` is a concrete first
candidate: its two meshes' saved declaration includes blend-index and weight
semantics, and the existing preview has its source mesh and painted atlas.
The current `gmdl_skin::vertex_skinning` explicitly rejects multi-mesh palette
mapping, however; a single palette cannot be assumed from that declaration.
The next bounded slice is a read-only source-asset census of both meshes,
their declarations, vertex extents, raw active indices/weights, and actual
tail palette descriptors. Admit a diagnostic only for a source-evidenced
mesh/palette mapping and an explicit palette of sufficient extent; compare
identity output and a known synthetic translation against an independent
scalar reference. Preserve register-offset units (the CPU source addresses
palette + raw index*16) and original weight bytes. Such a diagnostic would
demonstrate buffer deformation, not original actor or clip selection.

# Cell UI candidate source audit

Later static registry analysis establishes a guarded association from requested
type `0510A95B` to alternate type `250FE9A2`; runtime initialization, provider
success and package precedence remain unverified. See
`cell-ui-registry-audit-v1.md` for that newer evidence.

The Rust explicit-source intake example has now been run against the two
installed package copies. It matches all six control IDs and all 78/79 image
reference ordinals against the independent source audit, with 413/415 objects.
The comparison is `target/source-ui/cell-ui-explicit-comparison-v1.json`. It
does not select an active overlay or render a HUD.

The conditional ordered-local diagnostic now matches an independent Python
calculation for all 58 eligible rows in each installed copy, including every
supported local rectangle or explicit unknown. The audit regenerated the
complete element reports from the exact package payloads and checked their
SHA-256 values. Its assumptions are fresh empty child lists, successful
attachments, no runtime mutation, zero private flags and the authored root
rectangle. It does not establish global screen positions or an active HUD.
See `cell_ui_ordered_local_v2` and `cell_ui_ordered_local_audit_v2.py`.

The installed base executable SHA-256 is
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Ghidra function `E54150` hashes the literal `GlobalUICell-3`, constructs a
key with type `0510A95B` and high word `40464100`, and passes it to `811E30`.
`811E30` passes the key to `810D30`, which calls `80FA70` and, on success,
`810980`. `80FA70` copies the key and applies `(third_word & FFFF42FF) | 4200`
when global byte `164DF30` is nonzero, or on the fallback after a failed
first call when that byte is zero. This changes `40464100` to `40464200`.
`998EE0` stores all three key words and calls virtual slot `14`. Constructor
`80FDE0` sets the embedded object's vtable to `1418E10`; the slot at `1418E24`
contains `811980`. REA did not recognize that address as a function. Bounded
linear x86 decoding at `811980..811C7F` shows a copied key's type word forced
to `0510A95B` before an indirect manager call at `811A01`. `8DD7C0` supplies
that manager from global pointer `1668A80`. Bounded decoding at `7E8BE6`
shows a call to constructor `8E1AB0` and passes the resulting object to
global setter `8DD7D0`. The constructor sets vtable `1437740`, whose slot
`0C` is `8DEC20`. REA did not recognize this latter entry; bounded decoding
shows it forwards to `8DE8A0`, which uses further provider and type-registry
virtual calls. `6ABAA0` can initialize metadata on the returned object for
argument 6. Manager slot `48` resolves to `8DF310`, which looks up a registry
and may test candidates through another virtual call; registry contents are
not established here. Subsequent paths invoke another indirect loader with data from
the returned object. The unrecognized callsite and entries do not establish
whole-function boundaries. This is
not a proved function boundary. The later registry trace establishes a guarded
alternate association to SPUI type `250FE9A2`, but successful runtime dispatch
and selection of either installed copy remain unproven. The bounded decode is retained at
`target/source-ui/cell-ui-loader-811980-linear-v1.asm`.
The caller then
queries six control IDs via `810310`. The allocation
labels nearby say `Simulator/Cell/UI`. This is static executable evidence;
the exact loader indirection, conditions under which this function runs,
and actual displayed frame remain open. Evidence is retained in ignored
`target/rea/20261028-source-runtime-v6/{36,46,47,49,51,54,55,56,57,60,61,62,63}-analyze_function.json`.

The FNV-1 hash of the literal is `26B068D2`. Two SPUI resources have instance
`4046420026B068D2` and type `250FE9A2`; the hash alone does not establish
that the native key selects these resources. Their serialized `EEC1B001`
properties do, however, contain the six queried control IDs:

| ID pair | PatchData object indices | Spore_Graphics object indices |
|---|---|---|
| `05AF4AF0`, `05AF4BF0` | 221, 223 | 222, 224 |
| `05AF4AF1`, `05AF4BF1` | 236, 238 | 237, 239 |
| `05AF4AF2`, `05AF4BF2` | 251, 253 | 252, 254 |

The AF objects have class `6F15F51B` and the BF objects `4EC1B8D8`.
Their visual meanings and runtime visibility are not established by IDs.

The two SPUI payload SHA-256 values are `5c93c578393784282700b0cdacc1d370870851d34fc5a76aa62bad1c9809edda`
for `Data/PatchData.package` and `6f85179ae30bda8f6aa9a7522523ca7c9bd8a33a3f44c31db50a70cfb7d567dc`
for `Data/Spore_Graphics.package`. They contain respectively 413/415
objects and 78/79 Image objects. All of those Image objects refer to PNG
type `2F7D0004`, instance `31A44893B8B50133`. Two package copies of that PNG
have different payload hashes and each passes the PNG chunk CRC check at
512 by 512 pixels. The source package and the archive group word are recorded
for every copy; neither layout nor PNG precedence is inferred.

Run the read-only, versioned audit with a new output directory:

```powershell
python tools/cell_ui_candidate_audit_v1.py 'D:\Games\SPORE Collection' --out target/source-ui/cell-ui-candidate-audit-v3
```

The report contains the top-level serialized object-reference graph, Image-to-resource
links, provider identities, hashes and the six caller-control matches. It
scans all 82 package indexes and fails if an Image key is absent or a PNG CRC check fails. It does not
decode procedure execution, draw order, parent layout, asset selection or UI
events, and does not establish a visually equivalent Cell HUD. The report is
kept under ignored `target/` and no game bytes are added to the repository.

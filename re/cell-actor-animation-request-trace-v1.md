# Cell actor animation request trace

Static evidence refers to the base executable SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Addresses are virtual addresses. This trace does not establish that a live
actor selected, loaded, bound, or played any particular animation.

The public table reader matches the hash-bound source audit on all 131 records,
1,179 raw words and 262 state/key accessor values. The local Rust projection
`target/source-cell/cell-anim-state-table-verify-v1/rust-output.txt` has
SHA-256 `082ad6d98df25dd5c9a08239025e4bf6329b2c583b1cbddbc5a1319cbe10ca5b`.
This does not execute the actor-dependent selector or establish a chosen clip.

A public source-data join also matches an independent package inventory: all
131 table rows have a record in both the PatchData v10 and Spore_Game v7
AnimLists, producing 262 row-to-record links. The 131 v10 entry links each
have two matching typed ANIM index entries in the scanned inventory. Rust and
Python agreed on 524 ordered output lines; the Rust output SHA-256 is
`c7c6f18143c6a078148497e6292d99c3d8decf72ce6e8c5d48b8193d6ad822d1` and
the local provenance report SHA-256 is
`0c69db6710f9c836f4cd738ade122b0e3ddc6d9f2a049178b30d9f45c1824622`.
The scan covers 82 package files with no reported errors; these numeric links
do not decide mounted precedence, keyed versus filename loading, or playback.

The bounded Rust snapshot composition now combines the table rows with the
admission projection and weighted suffix. Its five focused tests pass; callers
must supply immutable actor/part/predicate observations and an already committed
draw. The separate nonzero-state RNG projection passes four focused tests and
matches the static recurrence numerically for x87 precision 53/64; its zero-seed
timing path, zero sign, x87 scaling and live state are outside that model. These
tests do not execute the original full selector.

## Dispatch and actor association

[MEASURED] `A05D60` installs the vtable at `01449660`. Its entries at +4 and
+8 are `A0C4F0` and `A0C520`; both forward their object and two arguments to
`A0C120`, while the +4 method additionally calls `A01B70`. `A0B110` allocates
the object, calls `A05D60` at `A0B5D5`, and installs its CID at +17C, queue at
+184 and context at +190. This corrects the earlier unknown table-base
boundary. The adjacent table beginning at `01449648` is separate.

[MEASURED] `E5CCE0` calls `E4EE10`, stores its result at its object +24,
and uses methods accessing that result's +17C. `E4EE10` obtains the result
through a manager's virtual +30 and calls `A04D20` and `A04D40` on it.
At `E5CE97`, the result's virtual +4 is called with two zero arguments;
`EBX` was set to zero at `E5CCED` on that path. This is a request-shaped call,
not proof of its runtime virtual implementation or successful binding. No
zero-key record was found in the two installed version-10 lists or in the two
version-7 lists (24 and 1,864 records); index coverage has the limits stated
in `cell-animlist-resource-selection-v1.md`.

## State to request

[MEASURED] `E6D260` checks the Cell-side actor's +FC and +248 fields. It
calls `E6D1E0`, which calls `E6CC30` and returns its result through an output
pointer. After resolving actor +248 through `B71D00`, `E6D260` passes that
key to `E4F190`. `E4F190` calls virtual +8 with the key and variant index 0
on eligible component objects and, when present, its own +24 object. The
returned value then goes to other virtual methods. `E4EF60` has a related
path that first checks `A02B60` before calling virtual +8. These are static
call paths; virtual identity and actual execution remain runtime gates.

[MEASURED] `E6CC30` scans 36-byte records beginning at `01484A20`, compares
the requested state with row +0, applies row +0C player/other eligibility
(or value 2 as a wildcard), part and additional predicate checks, and
accumulates row +8 float weights. It draws using `935D40` and subtracts
weights in selected-row order; on selection `E6D1D5` returns row +4. It can
return zero if no row is selected. Other fields and tests, including calls to
`E5C300` and `E5C2B0`, also affect admission. If the first-pass weight sum
compares equal to zero or unordered, a second pass collects state- and mode-matching rows
whose part code at +10 is zero, without those two predicates. Addressed table
rows connect the state argument to the following request keys:

| State | Request key | Source row | Installed PatchData version-10 label |
|---:|---:|---:|---|
| `0` | `AAAA0015` | `01484A20` | `cell_idle` |
| `12` | `AAAA0021` | `01484CCC` | `cell_move_jet` |
| `13`, `14` | `AAAA0021` | `01484DC8`, `01484DEC` | `cell_move_jet` |
| `12` | `AAAA0060` | `01485320` | `Cell_mov_cil` |
| `30` | `AAAA0091` | `014857A0` | `cell_mov_cil` |
| `30` | `AAAA0092` | `014857C4` | `cell_move_jet` |

State values in this table are hexadecimal. This is a source-table join to
AnimList records, not proof that every row passes its predicates. State 0
also has rows for `AAAA0016`–`AAAA0018`, `AAAA0049`–`AAAA0052`, and
`AAAA0097`. The `AAAA0015` and `AAAA0097` rows each have weight 5, and the
latter has wildcard player/other eligibility. Thus state 0 alone does not
deterministically identify `AAAA0015`; the full admission predicates and
weighted draw must be known. No probability is asserted without them.

[MEASURED] In `E6D3A0`, a conditional path uses actor +248 and calls
`E4EF60` with `AAAA0066`; separate conditions at actor +390 and +391 lead
to calls with `AAAA0101`, `AAAA0103`, `AAAA0105` and `AAAA0100`,
`AAAA0102`, `AAAA0104`, respectively. Those flags and calls do not establish
that the requests are reached during an observed run.

## Remaining boundary

The static joins identify plausible source requests, but not the current
actor state, concrete runtime vtable, admitted row, random draw, AnimList
enumeration/replacement order, winning mounted ANIM resource, successful
controller binding, or playback. Resolve those separately before treating a
source record as the active clip. The comparison reports for the installed
lists and REA dossiers are retained under ignored `target/`; source packages
and the executable were read without modification.

# Bounded native runtime block ordering

`[MEASURED]` `crates/spore-core/src/anim_order.rs` implements the ordering
region of native `9C3150` and its recursive helper `9C2FE0`. The input boundary
requires an already rerooted runtime block tree, the explicitly chosen root,
and each runtime block's finalized capability mask. This is a usable native
operation with explicit inputs. It does not establish the missing conversion
from source CreatureData into the finalized runtime tree.

## Native evidence

The executable SHA256 is
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Proprietary decompilation and assembly remain in ignored `target/`.

| Retained query | Evidence ID | Role |
| --- | --- | --- |
| focused-v1 60, `9C3150` | `ev_3dbb86207589fa36be3c6f46d0a0dc248ab8efac58dcfc8924f8ad57b7ddee12` | Sibling table at `9C4213..9C435B`, final block permutation and parent remapping |
| finalize-v2 04, `9C2FE0` | `ev_f85f10655fbea984c3a099ff2f4dcdbbe1f5590dab3d109e0373eac3a4ca8397` | Child-contiguous order and descendant count |
| finalize-v2 10, `9C1A00` | `ev_8b9389eab1a08545ce6d700760d99ebdc7054aaa8dd4e0525ec286bc7cea7cc9` | Range insertion into the ordering vector; callers append at its end |

`[MEASURED]` Runtime blocks have stride `468`. The relevant fields are:

| Hex offset | Contract at this boundary |
| --- | --- |
| `150` | Capability mask; this operation reads only `8`, the spine bit |
| `1FC` | Parent runtime ordinal, `FFFFFFFF` for the selected root |
| `204` | First direct-child ordinal in the resulting order |
| `208` | Number of direct children |
| `20C` | Number of descendants, excluding this block |
| `21C` | Separate palette identity, preserved by block copying |

`[MEASURED]` Child insertion visits blocks in previous runtime order. A new
spine-capable child swaps with the first existing child. At the root, when the
first child is already spine-capable, the second child is retained when it is
the only other child; each later spine child swaps with the second position.
Other capability bits do not affect this operation. This is a sequence of
swaps, so it must not be replaced by a stable sort of spine children.

For example, root children with previous ordinals `1 2 3 4 5` and spine bits
`0 1 1 0 1` become `2 5 1 4 3`. The same children under a nonroot parent become
`5 1 2 4 3`.

`[MEASURED]` The output starts with the chosen root. Each visited block appends
all its direct children and then recursively visits those children in the new
sibling order. This gives a contiguous range for the direct children and a
contiguous range for all descendants. The descendant range excludes the block
itself; a block's descendants need not immediately follow its own ordinal.
Leaf ranges begin at the output size when that leaf is visited, matching the
native assignment even when the empty range begins at the total block count.

`[MEASURED]` The Rust operation returns both permutation directions, remapped
parents, direct-child ranges and descendant ranges. It validates root and
parent bounds, additional parentless blocks, and parent cycles before ordering.
Those typed errors define a bounded API for malformed inputs; they do not
assert that the retail function performs the same defensive checks. An
explicit work stack preserves the native recursion order for deep inputs.

## Independent synthetic verification

`[MEASURED]` `tools/anim_order_probe.py` uses an independent flat child table
with the measured native insertion/swapping sequence and recursive traversal.
It also checks direct children and descendants against parent-chain sets,
permutation completeness, and parent-before-child ordering. It reads no
installed payloads. The Rust implementation uses separate per-parent child
vectors and an explicit traversal stack.

The generated fixture set contains every rooted labeled tree of sizes one
through five with every combination of spine bits, plus three 255-block
chain, star and branched trees. Other capability bits vary without changing
the selected spine bit. There are **21,109 cases / 105,095 explicit runtime
blocks**, with zero Python invariant failures. This covers the sibling swap
branches and nonzero root ordinals. It is synthetic validation of a measured
operation, not a recovered retail creature-pose corpus.

`[MEASURED]` All five focused Rust tests pass, and the Rust verifier agrees
exactly with the independent Python digest records for all 21,109 cases.
The comparison is `target/animation/anim-order-compared-v1.json`.

The normalized expected digest SHA256 is
`6dfd3ef7c3988c3b3813608ba0cdc359e34d0f1e63b25cdcdf89c84fbe5c655e`.
The canonical feed is `runtime-block-order-v1\0`, the input count and root,
input parent/mask pairs, both permutation arrays, and each ordered block's
original ordinal, remapped parent, direct-child start/count and descendant
start/count. Integers are LE u32, absent parents are `FFFFFFFF`, and each case
uses FNV-1a-64. The full synthetic fixtures and reports stay under `target/`.

```powershell
python tools/anim_order_probe.py --fixtures target/animation/anim-order-fixtures-v1.tsv --digest-out target/animation/anim-order-python-v1.txt --out target/animation/anim-order-python-v1.json
tools\build.bat test -p spore-core anim_order::tests
tools\build.bat run -p spore-bevy --example anim_order_verify -- target/animation/anim-order-fixtures-v1.tsv target/animation/anim-order-rust-v1.txt
python tools/anim_order_probe.py --fixtures target/animation/anim-order-fixtures-v1.tsv --digest-out target/animation/anim-order-python-v1.txt --out target/animation/anim-order-compared-v1.json --rust-digest target/animation/anim-order-rust-v1.txt
```

`[UNKNOWN]` Root selection, rerooting transforms, optional block insertion,
capability relabeling, complete rest-pose generation, original animation curves
and model palette association remain separate gaps. No source row or candidate
GMDL body-prefix association is promoted to a runtime bone index by this API.

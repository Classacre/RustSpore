# E5E100 per-candidate route

The installed source's E5E100 iterator visits candidate records, then tests
candidate +04 for 1 and +08 for the supplied row level. For a matching
candidate, finite +0 and -0 weights enter the existing-effect inspection path;
finite nonzero weights enter the placement/update path. The checked model
returns these as pending boundaries and does not read candidate +300, call
virtual methods, allocate, execute placement, or assert live pool membership.
Its finite-weight and stable-candidate requirements are additional safeguards.

The reviewed original-instruction comparison covers type and level mismatches,
both signed zeros, positive and negative finite weights, and three weights from
the separately checked E50080 installed rows. It captures the route immediately
before side effects; it does not execute the source pool iterator or establish
real candidate lifetimes. All nine controlled cases matched. Native receipt
`target/cell-e5e100-route-result-v2/receipt.json` SHA-256 is
`6804fa8d5f4a2dbfddc9c6add921444f7a13fef45edd1e608f54ca57f2777293`.
The checked Rust adapter matched all nine retained native route words; receipt
`target/cell-e5e100-route-rust-result-v1/receipt.json` SHA-256 is
`f2a8389570250b7c4e20d2163c5dcce0f71319f7fabce8715f03044a196183ca`.

The opt-in composition can use the two already authenticated E50080 curves and
their installed type-5 row levels, or the separately checked start-table progress
projection followed by those curves. Stable row, scalar, global words and
candidate snapshots remain caller admissions. These compositions do not find a
runtime effect object, prove scheduling, establish ownership, or execute a
cleanup, placement or callback. The progress composition is unit-tested; the
nine-row original-instruction comparison validates the route and three supplied
curve outputs, not a live progress-to-pool execution.

## Zero-weight effect pointer handoff

On the zero-weight path, E5E16F first reads candidate +300. A null pointer
skips to the iterator continuation. Otherwise it calls that object's virtual
+0C with literal 1, then re-reads candidate +300. If the re-read is non-null,
the source clears the slot and calls the re-read object's virtual +04. The
first call may change the slot; the second object's identity is not necessarily
the first object's identity. Its callback can also change later state, so no
post-callback slot value or object lifetime is inferred.

The reviewed source-instruction fixture used guarded synthetic vtables and
scripted first-call behavior: initial null, retained object A, first callback
clearing the slot, and first callback replacing A with B. All four controlled
cases matched; native receipt `target/cell-e5e100-zero-result-v1/receipt.json`
SHA-256 is `3843f19f3dcf2a8d6c7963531b837d302f9572c093722eba09b708f1bf960e1e`.
The checked Rust projection matched the four retained rows; receipt
`target/cell-e5e100-zero-rust-result-v1/receipt.json` SHA-256 is
`0e5cf5fef2639740e68a6c08de7a78845e821a8408dfcbfdef5337345dcca111`.
It admits the first callback outcome as a caller script and stops at the pending
second callback, with slot clearing established before that call. The comparator
adapter's call counts describe inert test stubs, not executed live callbacks.

## Nonzero arithmetic boundary

On the matching nonzero path, E5E19E reads candidate level +08 and grid words
+0C/+10, indexes the installed spacing table, converts coordinates to float and
forms two corners. It calls E51E50 with level and candidate +04; the helper's
result is multiplied by the installed 0.85 word and committed to float before
ordered SSE divisions with global +514C and a second reciprocal. The scale is
then applied to the grid edges; the current +300 slot selects existing versus
absent object continuation. E51E50 itself reads mutable global state and calls
E4ECC0, so its return and register effects cannot be inferred from an installed
row alone.

After independent review, `target/cell-e5e100-placement-prep-v1` matched four
controlled cases. It supplies finite positive +514C and a guarded scripted
E51E50 x87 return, preserves source arithmetic order and checks six corner
words for controlled coordinates and both +300 branch values. Native receipt
`target/cell-e5e100-placement-result-v1/receipt.json` SHA-256 is
`75ee0aebc515b3b1cec24315cb19c893d6ac9de72b02fbab251824c174125e64`.
The earlier checked Rust revision matched all four retained native rows, receipt
`target/cell-e5e100-placement-rust-result-v1/receipt.json` SHA-256
`0e5cf5fef2639740e68a6c08de7a78845e821a8408dfcbfdef5337345dcca111`.
That revision evaluated independent x87 multiplication before grid SSE math.
The corrected source-order revision is pinned separately and matched all four
retained rows again; receipt `target/cell-e5e100-placement-rust-result-v2/receipt.json`
SHA-256 is `0e5cf5fef2639740e68a6c08de7a78845e821a8408dfcbfdef5337345dcca111`.
The synthetic helper preserves source intermediate XMM registers by admission.
Both object paths stop before allocation, update, virtual calls or ownership
effects; neither comparison establishes live placement or helper behavior.

## Existing-object callback handoff

For a supplied non-null stable object slot, original `E5E392..E5E3A3`
re-reads the slot, fetches its vtable +44 entry, and calls it with receiver
and arguments `4`, the address of the caller's weight local, and `1`. The
comparison cuts immediately after that call; the guarded fixture method uses
`ret 12` and records the argument value and address check without performing
any actual object method effects.

After independent review, the exact original 18-byte slice matched four
scripted, finite nonzero weights retained from the separately checked E50080
curve corpus: `3f800000`, `3f40009f`, `3362b819`, and `3f800000`. Receipt
`target/cell-e5e100-existing-result-v3/receipt.json` SHA-256
`a96afe306fda86d3dd206733f94b1babd8973f617b8fc3b6ae0f8b4f46cabb4a`;
raw output SHA-256
`ad9db4f31da5874dca7e1bb1c41f4cbf406bd24166390434d37754eb91967596`.
The reviewed runner SHA-256 is
`c611919d317bbe2fd63d76003c24265f959acfcd708ee3606cac4cb0799336e4`.
Each process control and private-desktop closure check passed. This admits a
stable synthetic object, slot, vtable and stack local; it does not establish
live ownership, method side effects, allocation, the absent-object branch,
or continued iteration. The initial v1 preparation failed assembly without
executing the host and is preserved separately.

The checked Rust projection accepts only the separately admitted nonzero route,
existing-object branch, nonzero synthetic identity tokens and finite nonzero
weight under explicit stable/disjoint lifetime admission. It stops at the
pending call and does not invoke a callback. Its inert-stub adapter matched all
four retained native argument rows without rerunning source instructions;
`target/cell-e5e100-existing-rust-result-v1/receipt.json` SHA-256
`53cf86042bcc0883b0d33f6be4cb969c916c39e94141e35002acc3a036bd6d40`.
The adapter's final two observations describe the comparison stub, not a live
method's effects.

## Primary and fallback provider handoff

For the absent-object path, E5E282 calls original 627DE0 with the selected
row ID, group zero and the address of candidate +300. With an admitted
non-null background effects world, 627DE0 calls its virtual +08 with the ID,
zero and slot address. Only callback AL exactly 1 suppresses a second request;
otherwise source retries using ID E18D6423 and the same slot. The callback can
mutate that slot, so its fresh value is supplied rather than inferred.

A separately reviewed source-instruction host matched four synthetic scripts:
primary AL 1, primary 0 with fallback, primary 2 with fallback, and primary 1
writing a null slot. The host records both calls and pre/post slot values;
receipt `target/cell-effect-provider-fallback-result-v1/receipt.json` SHA-256
`ce43ac8323dad4636f5ac98f395cdfe0f193b88833e1dc3b35547d21eed4c1e3`.
It excludes the null-world manager route, actual effect provider, asset
resolution and lifetime, following transform and live placement.
The checked Rust projection and standalone comparator matched all four
retained callback/slot and FP rows; they model callback snapshots rather than
executing a provider or establishing a live effect.

## Conditional concrete world and provider route

On the inspected setup route, `7EAA4D` calls `A72B50`, which allocates 0x258
bytes and calls `A6E220`; that constructor installs primary table `01456688`.
`7EAA97..9F` publishes the resulting +60 pointer through `67DB10` to global
`015FE8E8`, returned by `67D8F0`. The table's +4C entry is `A6DFB0`.
It allocates 0xA4 bytes and calls `A705C0`, which installs primary table
`01456AC0`; this table's +08 entry is `A712F0`. Thus, if the same initialized
factory/world survives, these calls complete and the named Cell world is not
replaced, its virtual +08 conditionally reaches `A712F0`. This is a source
route, not an observed object or a guarantee that callbacks preserve identity.

`A712F0` immediately returns AL 0 if the supplied output slot is null or world
+20 equals 5. Otherwise it calls `A6D7B0` with the requested effect ID and
group, and returns AL 0 when that result is negative. On its nonnegative path
it allocates or acquires an object, initializes it and stores it through the
supplied slot; these later calls and their lifetime effects remain unverified.
`A6D7B0` scans the manager's +2C..+30 array of table pointers in order, probes
each non-null table's +68 structure, skips misses and negative values, and
packs the table index into the high byte of the first nonnegative result.
This depends on valid array bounds, table and map lifetime, current contents,
callback behavior, and unchanged manager/world association. It does not
identify a mounted package winner or prove that `44C489B7` is active.

A useful next bounded comparison joins the already checked `627DE0` fallback
route to original `A712F0` with a guarded synthetic world and scripted negative
`A6D7B0` result, trapping before allocation. It can establish the ordered
primary/fallback lookup arguments under explicit state admissions; actual map
contents, provider precedence, positive-result product, placement and live
rendering remain separate.

The bounded original `E5E282` → `627DE0` → `A712F0` join was then run with
synthetic non-null world and stable vtable. Four controlled rows alternate
world +20 values 0 and 5; a scripted negative manager lookup causes primary
and fallback lookups on the first state, while state 5 rejects both before
lookup. The positive lookup branch traps before allocation and the null-world
manager path is excluded. All four rows matched; receipt
`target/cell-effect-concrete-world-result-v1/receipt.json` SHA-256
`3cbbe7153cf3509b80d1286c757f7b99ee60327bff114fab620f30cf98301bac`.
Synthetic world and manager, callback results, ownership and state are
admitted; no live provider, positive selection, allocation or placement follows.
The checked concrete-world projection matched all four retained source rows;
its admitted lookup callbacks preserve the slot and world +20 across both
requests, since the source reads that field again during fallback. Positive
lookup and all subsequent allocation remain excluded.

## Guarded original effect-map search

Original `A6D7B0` scans its supplied manager table array in order and calls
`5948D0` for each non-null table. The latter divides the first key word by the
nonzero bucket count and walks the selected bucket chain, comparing both words.
The caller skips a sentinel/miss and a negative node value, returning the first
nonnegative value ORed with table index shifted by 24 bits. Runtime map content,
nonzero bucket count, array/chain bounds, table order and lifetime remain
admissions; the result does not identify the live mounted provider.

A separately reviewed synthetic two-table fixture matched six cases under
CW 027F and 037F: miss, first hit, negative then second hit, a collision chain,
null first table, and the second installed row ID. Receipt
`target/cell-effect-map-lookup-result-v3/receipt.json` SHA-256
`67ecca75b3f8ab1c4b787df4a6c304a498f14122a69e64b3c39afcfe740943b4`.
The pre-execution v2 review found that its original success `ret 8` immediate
had been truncated; v2 was never executed. The corrected v3 includes and audits
all three bytes. Tables and nodes are synthetic; installed EFFDIR loading,
actual map order, provider identity, active effect and placement are unproven.
The checked Rust projection matched all 12 retained source rows and passed a
focused unit test. It abstracts the sentinel to a terminating `None`, rejects
cycles and out-of-range chains, and narrows packed table index/value ranges;
those safeguards do not establish live table contents or order.

The conditional loader route now reaches the table boundary: `7E9730`
requests type `EA5118B0`; on a successful provider result it registers the
returned +14 object with manager virtual +64, conditionally `A6E0D0` under
table `01456688`. The EFFDIR table constructor `A6B7A0` installs table
`01456458`, whose +30 is `A6C090`. Its loader reads each export index and
8-byte key and inserts the resulting tuple into table +68 with `594990`.
This assumes the same concrete factory/table and successful loading; provider
precedence and runtime registration remain unknown.

A separately reviewed fixture decoded two explicit 12-byte records from the
pinned `40450200C1195706` EFFDIR payload: group-zero IDs `5A314EBD` and
`5A314EB9`, indices 4809 and 4810. It ran original `594990` insertion followed
by original `A6D7B0`/`5948D0` lookup, with a synthetic one-bucket map, fixed
two-node allocator and a no-rehash helper. Six rows matched, receipt
`target/cell-effect-insert-lookup-result-v2/receipt.json` SHA-256
`a93ce3dd77718e87d1e69215b316a8e372cc9ce57f491b8aa21978a42bf00dc3`.
The independently reviewed Rust insertion/lookup comparator matched those six
rows and its focused test passed. Stream parsing, real allocation/rehash,
full directory registration, provider order and live placement remain outside
this fixture; stable pinned input is admitted.

A further original-source join begins with a supplied export index, reads the
exact eight-byte key portion of either pinned EFFDIR record through original
`93A470`/`8DD640`, inserts it with `594990`, reads a synthetic `FFFFFFFF`
terminator with `93A3F0`, then looks it up through `A6D7B0`/`5948D0`. Four
cases matched under CW 027F/037F; receipt
`target/cell-effect-stream-insert-result-v1/receipt.json` SHA-256
`8e453adb64d31fbecd829bee8a43da9f80c18102b770fa284a414fca40ac0942`.
The one-record stream, supplied index and terminator, successful synthetic
stream/allocator, no-rehash behavior and stable inputs are admitted. Earlier
loader parsing, real provider selection, live directory registration and
placement remain unverified.
The standalone Rust comparator decoded those same two record bytes, reused the
bounded insertion/lookup projections, and matched all four retained rows and FP
controls. It does not execute the source loader or establish runtime state.

A reviewed original source-fragment fixture then processed all 3,049 export
records in the pinned `40450200C1195706` payload, in their on-disk order and
with their real `FFFFFFFF` terminator. The first index was supplied at the
slice boundary. Original key/next-index reads and byte swaps, map insertion
and lookup ran against a synthetic 4,096-bucket table and a preallocated
3,049-node pool with a scripted no-rehash helper. Six cases (the two candidate
IDs and a miss under CW 027F/037F) matched; each traversed 6,098 stream reads
and inserted 3,049 nodes. Receipt
`target/cell-effect-full-exports-result-v1/receipt.json` SHA-256
`e00235072c0d01de03b2ce15dec422bc254f0392d375de43f5836a8cb7f8b8f7`;
extracted export slice SHA-256
`578dbcda2501569a20514716f25dfd71e15b91d78346cc5a03cc51d35daadd92`.
Stable source bytes, successful bounded synthetic stream/allocation, no
rehash and supplied first index are admissions. The earlier full directory
loader, runtime registration, provider precedence and placement were not run.
The standalone Rust comparator parsed all 3,049 admitted export records,
rebuilt a synthetic 4,096-bucket map with the separately checked insertion
model and matched the six retained source lookup/count/FP rows. Its input is a
caller-supplied slice; the original host runner pins that slice's source.

## Post-creation transform boundary

Original `E5E29A..E5E37A` forms the midpoint of two supplied corners and their
X span, initializes the transform flags/counter, and copies a supplied 3×3
basis via original `41CAE0`. It stops before dereferencing the effect's virtual
method. The supplied global vector is read into the transform area and then
overwritten by the computed midpoint in this slice; it is not inferred as a
live position.

The reviewed v1 harness exited with access violation `0xC0000005`, receipt
`target/cell-effect-transform-result-v1/receipt.json` SHA-256
`ce5be0a1592317c545c22053869eba08683238ff62cb7943ee07f95f948ba738`.
Static stack analysis proves its wrapper put a return address at S; after the
source advanced ESP by 0x10, the original helper's `[ebp-4]` local overwrote
that address. This harness defect is a plausible cause; the fault address was
not captured. A v2 correction failed assembly due to label scope, without host
execution. Both are retained.

The independently reviewed v3 wrapper enters/exits by jumps, reserves helper
stack below S and checks in-stack canaries. It matched two supplied finite
corner pairs and identity basis under CW 027F/037F, receipt
`target/cell-effect-transform-result-v3/receipt.json` SHA-256
`db119de1270cb0e69deb8129c6cadc5478d8abf977f23f799e96e8ec116ffdc2`.
The corners, basis, vector, object slot and FP settings are synthetic; the
real pool, creation, effect dispatch, placement and camera remain unverified.
The checked Rust transform projection matched all four retained transform and
FP rows after independent review and passed its focused unit test. Finite
corners/basis, stable storage and fixed MXCSR remain explicit admissions.

The original startup bounds slice E50670 was run with the source startup vector (100,100,0), a synthetic resolver, and supplied +68 values 0 and 15 under CW 027F/037F. The four retained rows matched in `target/cell-start-bounds-result-v1/receipt.json` (SHA-256 `fc0972a42b07de3f81f7bb3d7893a10e5cf79eb8eb06f2ae51c890a073c63d48`). Fixed bounds are (90,92.5,0)–(110,107.5,0); dynamic XY bounds are (90,92.5)–(110,107.5) for zero and (80,85)–(120,115) for 15. The host cuts before dynamic Z and downstream calls; resolver, +68, and live state are not established. `cell_start_bounds_v1` and its comparator project only this admitted boundary.

The bounded startup→grid→candidate host linked original E50670, E5F450, E5F840, E51E50 and E4ECC0. With synthetic resolver +68 of 0/15, progress 0, +514C=1, level 3, types 0/1 and CW 027F/037F, all eight guarded enumerations matched four candidates each: x/y pairs (2,2), (2,3), (3,2), (3,3). Receipt `target/cell-start-grid-result-v1/receipt.json` SHA-256 `cd0d2e7decc4c1ea9b2b88708c1b912524de1bcf02b6da2051377a0493e759e3`. Before the original write loop the harness checked exact stack locals and output pointer, ordered 0..4 bounds and product at most 16; the fifth record word was harness-zeroed. The cut/synthetic resolver, supplied globals and guard are fixture controls; actual active pool, provider and live placement remain unestablished. `cell_start_grid_v1` composes the retained startup bounds with that fixed source arithmetic under explicit admitted helper/FP assumptions.

The guarded source startup→grid→fresh-allocation→rectangle-prefix join matched eight fixed rows (`target/cell-start-spawn-result-v1/receipt.json`, SHA-256 `adb03eb5c9cd07dff9b1ef4c012c57cda0c853e48f49652512014e4e751d5ed0`). For the first candidate in each admitted enumeration, original B71C50/B71D00 and E7D529 copied the record into a synthetic fresh 0x304-byte slot, then original E7C6F0 produced corners (63.75,63.75,0) and (95.625,95.625,0), cutting before configuration/list selection. The slot allocator and zeroing callback, flag, globals and resolver were controlled; existing pool, order, active resource, actual camera and live state remain unestablished. The separate Rust `cell_start_spawn_plane_compare_v1` matched all six corner words in all eight rows against the existing Bevy grid projection; its rectangle words are `429f6000,429f6000,41ff0000,41ff0000`. The host's earlier assembly operand-width failure is retained as `target/cell-start-spawn-prep-v1/host-build-failed.asm`.

The existing opt-in Bevy plane path was captured for `--source-effect-plane-grid 3,0,2,2,3f800000` alongside the equivalent explicit rectangle and a no-plane control. On RTX 3060/Vulkan at the same seed and 60 ticks, grid and explicit-rect PNGs were byte-identical (SHA-256 `bc392b29980b6764882f270122cdc1c2b7a86ed10d551681176d87c7af83fd38`); receipt `target/startup-spawn-plane-capture-result-v1/receipt.json` SHA-256 `bc0b01a48f0b2683a6f5552758d5999ec92fb900fecfae925b021c7f4b8bce80`. The inspected image did not show an identifiable controlled plane, consistent with this supplied placement being outside the view. The no-plane comparison changed 476,801 whole-frame pixels, but removing the plane also changes camera/layer configuration, so that figure does not prove plane visibility. This diagnostic does not establish a live environment, resource selection or gameplay camera.

# Optional generated-foot construction trace

[MEASURED] The canonical loader9CBFA0 calls9CBE70 only when its nonzero-foot
count+3C8 is zero and source-root-nomination flag+3F0 is zero. Fresh canonical
source intake encounters this call gate in106 of1,365 creation resources.
The call gate alone does not establish that a foot is inserted.

The retained original base-executable evidence is:

| Operation | Evidence |
| --- | --- |
| Canonical loader9CBFA0 | ev_0789da66e7c2b2f79cb1e09c7665a7ab598558b8968857cb0bc80cdee48647ab |
| Driver9CBE70 | ev_11d623564cb51e809f27a43e5e5876d3bbe8416b4231a3fe9e26b2f35b903cad |
| Spine minimum9C84B0 | ev_fc739a51c387fd82ff26bce527abdd757691e7e64dee61b38f4b1aed31de86e4 |
| Leaf-limb minimum9C8560 | ev_42ec8ec57461327b189c183149fcf1128cf9771bcb34906f6d79a67a0403f40d |
| Point selection9C8660 | ev_02a5e495bf1d7614d1d78ceb25e79fd4dd7cf173813dff67f5891b2fbe49e840 |
| Quaternion vector rotation99BE70 | ev_5ff30ee6689c344bef88cef089a9ab1ad8dbd4ee92d6fb329a82bfc758a9954c |
| Foot append9CBAB0 | ev_bd862b6c5c6224aac9348833a70a6f916735415ab82b16fe01b94699d82a5d0a |

Addresses refer only to SHA-pinned base SporeApp.exe
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Disassembly remains under ignored target/animation/optional-feet-*.

## Driver and minima

[MEASURED] Both minimum helpers compute the same literal binary32 expression:

```text
t0 = block13C * abs(block134)
t1 = block140 * abs(block138)
t2 = block138 * abs(block130)
height = block114 - ((t0+t1)+t2)
```

The absolute values use x87 FLD/FABS/FSTP; products, sums, subtraction and
comparison use scalar SSE. These offsets must not be replaced by a conventional
oriented-box distance formula. Constructor9B2F90 writes local quaternion
128/12C/130 to zero and134 to one at9B2FCE..9B2FE6. Source half-extents at
138/13C/140 and local origin144..14C have separate loader producers; the later
finalizer recalculates local quaternions after ordering.

9C84B0 takes the strict minimum over **presence-spin+150 bit8**, starting at
positive MAX. 9C8560 takes the strict minimum over blocks whose child count+208
is zero, presence-limb bit2 is set and presence-foot bit4 is clear. Equal
values retain the first runtime ordinal. Both helpers use presence masks,
including zero-number capabilities.

[MEASURED] Let S be the once-computed spine minimum, L0 the initial leaf-limb
minimum, and L the current leaf-limb minimum. After finding an initial limb,
the driver returns when either source SSE comparison succeeds:

```text
S <= L
(S-L0) * threshold <= (L-L0)
```

The threshold is global155223C. Its on-disk bits are3DCCCCCD (binary32 about0.1);
this does not establish a live runtime policy. The comparisons are COMISS/JBE,
so unordered operands also return. S and L0 remain fixed across iterations.
After attempted insertion, only the leaf-limb minimum is recomputed.

## Point and append boundaries

[MEASURED] The directly called9C8660 entry has a bounded reachable body through
9C8703, with two returns and one call to99BE70. For presence mask+150 &60 equal
to zero it selects the block's own runtime ordinal+1F8 and returns:

```text
point = center10C..114 - rotate(absoluteQuaternion118..124,localOrigin144..14C)
```

For mouth/grasp presence bits20/40 it instead selects parent ordinal+1FC and
copies the absolute center. The helper does not itself mutate the source block
or its capabilities. Exact requested/returned Ghidra dossiers851/855 and their
pinned source bytes agree on the point and rotation helper extents.

[MEASURED] Rotation99BE70 through99BFD0 is scalar SSE, with no callees or
normalization. Its summation order differs from the loader's source-center
expansion. With q=(x,y,z,w), v=(vx,vy,vz), retained products
`xxn=-(x*x),yyn=-(y*y),zzn=-(z*z),xy=y*x,xz=z*x,yz=z*y,wx=x*w,wy=y*w,wz=z*w`:

```text
rx=(((zzn+yyn)*vx + (xz+wy)*vz) + (xy-wz)*vy)*2 + vx
ry=(((xy+wz)*vx + (yz-wx)*vz) + (zzn+xxn)*vy)*2 + vy
rz=(((xz-wy)*vx + (yyn+xxn)*vz) + (yz+wx)*vy)*2 + vz
```

No approximate quaternion/vector library call is an established substitute.

[MEASURED] Append9CBAB0 adds one block, or two when the chosen parent's symmetry
word+200 is not FFFFFFFF. Generated blocks use palette ID+21C=FFFFFFFF,
presence/nonzero foot number1, part PROP reference40600000:6B2C0771 and 0.01 extents.
The second point negates only the X sign bit and attaches to the original
parent's symmetric ordinal; the pair's symmetry words point to one another.
Each insertion increments its parent's child count and the definition's foot
count. Parent-local anchor350..358 is expanded from the parent absolute
quaternion and point-minus-parent-center in source arithmetic order.

The optional gate requires nomination0, so its original source prefix has no
nontrivial initial root permutation. Appended FFFFFFFF IDs remain distinct from
that source palette prefix. This does not prove the complete appended block
state, absolute pose or later selector binding.

## Fresh structural denominator

[MEASURED] `tools/anim_body_optional_probe.py` rereads all82 canonical indexes,
1,365 source resources and46,418 rows, matching each payload hash and source
ordinal to the accepted native168-byte capability projection. The106 call-gate
resources contain1,633 source rows. One hundred have no eligible leaf-limb
candidate:76 have an admitted initial nonzero-spin root and24 do not. The other
six have14 eligible leaves:two resources have an initial nonzero-spin root and
four do not. None of those14 leaves has mouth/grasp presence bits20/40.
The report is `target/animation/optional-feet-structural-census-v1.json`.

The empty-candidate path is directly bounded by source control flow. The driver
initializes the selected ordinal toFFFFFFFF at9CBEA6. Helper9C8560 stores an
ordinal only after all three structural conditions at9C85A8..9C85C7 pass. With
no such block,9CBEC7 branches directly to9CBF91 return, before point/append.
The source loader zeroes208 at9CC975 and increments each parent once at9CCA02;
these counts agree with the fresh source-parent edges. This proves the empty
candidate branch, independent of geometry and the threshold. The accepted
topology interface still retains its106-resource driver-call flag and has not
silently reinterpreted it as a completed insertion or bypassed required work.

## Accepted construction boundary

[MEASURED] Canonical half-extents/origins have now passed the independent
three-context source-instruction/Rust gate for all139,254 evaluated source rows;
see `anim-body-source-fields-native.md`. Source anchors also passed their
full-source three-context gate, documented in `anim-body-source-anchors-native.md`.

[MEASURED] Point/append arithmetic, the complete projected constructor fields,
the original driver and its masks now pass the original-byte/Python/Rust gate
below. Complete constructor padding, debug strings and allocator behavior are
outside that accepted projection. No required optional call is bypassed.

The driver has no explicit iteration limit. A bounded reconstruction must
establish progress for its admitted contexts or return an explicit interface
error; it must not invent a retry, foot placement or deletion rule. All106
call-gate resources are covered by accepted bounded optional construction.
Later finalizer admission and assembled animation remain separate gates.

## Prepared source-bound operation and native fixture

[MEASURED] The exported `anim_body_optional_feet.rs` prepares the original
source prefix using the separately conformed geometry/anchor/capability
operations. It retains the original root nomination, remapped parent and raw
symmetry word. The constructor's local quaternion128..134 is still identity
at this boundary: source loader writes only absolute118..124; name formatting
receives the bounded name buffers, capability preparation receives150, and
getter9B5200 only reads the block. Exact constructor Evidence is
`ev_23ee6d5df976b414155ef438f4ed27905b766f69fa2fff2fa7f6d1f3d9eac606`.
The append resize helper9CB900 is
`ev_d99824e051c319b7f860053e0f69767ec7acc9f4475b72dfe821ee8fbe219236`.

The supported optional-driver domain excludes mouth/grasp eligible leaves,
as do all14 actual source candidates. Point selection therefore returns the
selected leaf itself. Original append increments its child count at9CBC73,
so that leaf becomes ineligible. New foot-only blocks cannot enter the limb
minimum. Each batch removes at least one initial candidate and adds at most
two blocks. The strict operation requires `count+2*initialCandidates<=255`
before mutation; its progress guard is an invariant check, not a retry rule.
Consumed quaternions are finite with magnitude at most2 per component; consumed
coordinates/extents have magnitude at most2^100 and extents are positive.
These explicit bounds close point/anchor output arithmetic, while original
extent-product/threshold overflow and unordered COMISS/JBE branches remain.
Unconsumed input anchor/scale/field words are preserved bit-for-bit, including
nonfinites. No matrix/quaternion quality filter is added.

[MEASURED] `tools/anim_body_optional_fixture.py` streams the complete fresh
1,365-resource source intake under three x87 contexts and both initial3F5
flags into8,190 resource-context/flag digests, with zero exclusions. The source
digest SHA256 is
`b5002489765ba1778ef401a2372a2f354c125c02f1118fbc66149dbf6c5304cb`.
The final v2 cohort emits636 original call-gate contexts and576 synthetic
contexts,1,212 native requests total. It adds exact growth to255 blocks,
repeated finite-spine threshold stopping/equality and presence-foot with a
zero-number capability. The independent integer-SSE projection SHA256 is
`3675514158d42c004abedc05d2d1fa6837de9746ae8439a3f24ccb86a2d872bd`;
input SHA256 is
`5fda00ed148a574473f98ea048008d721e34913901c5b6de4a472734597c4965`.
Preparation is retained under `target/animation/body-optional-inputs-v2/`;
v1 remains the earlier636+480 preparation, with identical source digests.

At the explicit0.1 threshold, the independent source reference leaves the100
empty-candidate resources unchanged and inserts two mirrored feet in each of
the two admitted-spin resources. The other four resources, whose later
finalizer input lacks a nonzero-spin root, receive3,3,1,1 feet respectively.
Those four records stay in the constructor denominator; they are not promoted
to successfully finalized creatures. These counts match original-byte/Rust
comparison, and0.1 is the on-disk selected context rather than a retail policy
guarantee.

[MEASURED] The reviewed and executed host at
`target/animation/body-optional-native-v2/manifest.json` contains ten complete
unchanged source spans: driver/minima/point/rotate/append/default-constructor,
foot insertion and native mask helpers. Checked fixed storage, bounded memset
and excluded debug-name formatting are the three owned providers. It preserves
all projected fields, guards input/definition/blocks/frame/reply, shadows the
source prefix except native childcount writes, checks exact stack/CW/x87TOP/
MXCSR controls and compares zero/all-one scratch runs. Constructor padding,
debug name contents and allocator behavior are explicitly outside the returned
projection. Each316-byte block record contains all168 capability bytes and37
exact field words. Every80,596-byte projection is streamed; no casewise list
of tens of thousands of integers is retained in memory.

The hidden runner uses file-backed stdin/stdout/stderr and streams each
projection comparison. Every attempt retains the input before launch, partial
output/error and exit/timeout/exception status. A finally block verifies all
source/host/cohort/tool pins even after smoke/protocol failures. Only an empty
pre-ReadRecord allocation exit10 is retried, at most8 distinct attempts; native
failure7, output-bearing failures, timeout and exceptions are never retried.
Ten controlled benign-Python runner checks pass at
`target/animation/body-optional-runner-evidence-check-v1/report.json`, covering
partial failure/timeout output, launch failure, exact retry bounds and finally
immutability checks. They execute no original bytes or GUI.

The v2 host ASM SHA256 is
`0a13cfc1ca547c212d8a9f3fca0fc066afa85746d79f5a9f2a4b5eae5b58c4a9`;
EXE SHA256 is
`b1fbd363b6e81d9542d398a5ff7307f6c1062b75a794df3e5dfb00a01b0aa84f`.
Unexecuted v1 is retained preparation history; v2 narrows finite validation to
the actually consumed fields so opaque source words are not rejected.

[MEASURED] Independent peer/root review and root reproduction accepted the
v2 host. Root's hidden native execution passed all1,212 cases and36 protocol
checks with all42 execution pins unchanged. Every80,596-byte native projection
matches both the independent Python projection and fresh Rust output exactly,
with zero excluded cases. The shared projection SHA256 is
`3675514158d42c004abedc05d2d1fa6837de9746ae8439a3f24ccb86a2d872bd`.
Fresh Rust intake also matches all8,190 source resource/context/initial-flag
digests for all1,365 resources and46,418 source rows, with zero exclusions and
SHA256 `b5002489765ba1778ef401a2372a2f354c125c02f1118fbc66149dbf6c5304cb`.
All five focused Rust tests pass. The accepted comparison is
`target/animation/body-optional-native-v2-run/comparison-v2.json`; retained v1
has identical results but copied the preparation-only no-execution scope into
its combined report. V2 names source-input and actual native-execution scopes
separately and rechecks the retained outputs without another native execution.

The host checks its explicit stack/CW/x87TOP/MXCSR context and guarded projected
memory. It does not establish caller DF or nonvolatile-register restoration,
or equivalence of every constructor padding byte. The accepted topology API's
earlier call gate is unchanged; callers can now supply actual optional output.
[UNKNOWN] Finalizer reroot anchors/local orientation, relabeling/classes,
channel selection and evaluated pose binding remain separate gates.

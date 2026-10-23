# Canonical source parent and child anchors

[MEASURED] The canonical loader9CBFA0's second source loop uses the source
palette ID at block21C to recover each raw140-byte row. Parent ordinals have
already been remapped through the initial root nomination. It increments each
remapped parent's child count208 at9CCA02. This count is equivalent in source
palette order; it is not a finalized child-range or runtime-order claim.

The loader Evidence is
`ev_0789da66e7c2b2f79cb1e09c7665a7ab598558b8968857cb0bc80cdee48647ab`.
Exact default constructor9B2F90 is
`ev_23ee6d5df976b414155ef438f4ed27905b766f69fa2fff2fa7f6d1f3d9eac606`;
the directly called delta getter9B5200 is
`ev_39370fae1da3a505611cb368f2641b29a4db1d2e0edb95fa7043378f14431025`.
All addresses refer to base SporeApp.exe SHA256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.

## Original source operations

[MEASURED] A parentless row skips the entire anchor operation. All ten floats
at344..368 retain literal positive zeros from the block constructor, even if
its presence mask includes spin or its local origin is nonzero. Every other
row starts from its own raw translation54/58/5C. Source class byteB equal5
instead selects the midpoint with its raw source parent's translation:

```text
pointX=(ownX+parentX)*0.5
pointY=(parentY+ownY)*0.5
pointZ=(parentZ+ownZ)*0.5
```

The class5 branch checks a nonnegative signed16-bit source parent; the strict
source operation bounds every non-FFFF parent below the255-row maximum. It
does not substitute a geometric nearest point or a normalized orientation.

[MEASURED] For q=(x,y,z,w), define the committed binary32 products
`nx=-(x*x),ny=-(y*y),nz=-(z*z),nwx=-(x*w),nwy=-(y*w),nwz=-(z*w),
xy=y*x,xz=z*x,yz=z*y`. With d=point-parentCenter and the parent's absolute
orientation118..124, parent anchor350/354/358 is:

```text
px=(((xy-nwz)*dy+(xz+nwy)*dz)+(nz+ny)*dx)*2+dx
py=(((nz+nx)*dy+(yz-nwx)*dz)+(xy+nwz)*dx)*2+dy
pz=(((xz-nwy)*dx+(ny+nx)*dz)+(yz+nwx)*dy)*2+dz
```

With d=point-ownCenter and the child's absolute orientation, child anchor
344/348/34C uses the same X/Y expressions but a different Z summation:

```text
cz=(((ny+nx)*dz+(yz+nwx)*dy)+(xz-nwy)*dx)*2+dz
```

These expressions preserve the actual expanded source instruction order.
They are distinct from the earlier local-origin expansion and rotate99BE70.
No generic quaternion rotation or reassociated matrix product replaces them.

[MEASURED] Getter9B5200 is read-only with respect to the block. At literal
scale1 it returns each child-anchor component multiplied by1. If presence
mask150 has spin bit8, it separately multiplies local origin144/148/14C by1
and subtracts that result. Nonzero-number mask154 is not tested. The loader
commits this vector at35C/360/364 before computing length368 with retained
x87 operations. In machine order, the norm squares X, squares Y, adds Y2+X2,
squares Z, adds Z2+the previous sum, then FSQRT and FSTP32. The x87 precision
and rounding context remains explicit; the retail thread context is unknown.

## Strict source interface and independent oracle

`anim_body_source_anchors.rs` projects ten anchor/delta/length floats, source
palette/parent IDs and direct child counts. Its convenience operation reads
the original raw rows, uses the conformed source rest/field head and prepares
capabilities with the caller's explicit PCTP dictionary. The explicit field
entry checks palette identity, lengths, source parent bounds and finite
consumed fields. These errors define the Rust interface and do not claim
native diagnostics. Both public entries scope SSE toMXCSR1F80 and restore all
incoming control/status bits on every exit.

`tools/anim_body_anchors_probe.py` independently decodes/interprets the original
anchor region9CCA09..9CCE20 and its actual9B5200 callee, including CALL/RET and
the x87 stack operations. It admits no unseeded register or memory reads.
At entry EAX is the source palette row, ECX is the remapped parent's byte
offset, EDX is the runtime block base, and9CCA18 completes the parent pointer.
XMM2 is the sign mask and XMM6 is0.5, as loaded by9CC995/9CC99D. Inputs come
from fresh canonical archive reads, the conformed source-instruction geometry
head/tail and accepted native capability projections, matched by payload SHA.
Parentless defaults follow the exact constructor stores rather than skipping
the row from the denominator.

[MEASURED] The completed fresh source oracle covers all82 indexes,1,365 source
resources and46,418 rows under007F/027F/037F, producing4,095 digests and139,254
evaluated rows with zero failures or arithmetic exclusions. Each context has
35,014 ordinary source-translation rows,10,039 class5-midpoint rows and1,365
parentless constructor-default rows. The normalized digest SHA256 is
`af519227ecee9e6822f927d64d39e23428b120e7f5eadef86dea2262150ceb8f`.
Evidence is `target/animation/body-source-anchors-oracle-v3/report.json`.
The earlier v1/v2 preparation attempts stopped at the first source record on
an unseeded EDX and a Python local-name collision respectively; neither is
accepted source evidence. The fresh v3 entry seed and complete run are retained.

[MEASURED] The exported module passes all four focused tests. Root independently
reran the complete source oracle and fresh canonical Rust intake: all4,095
digests and139,254 evaluated rows agree with zero exclusions and the same
SHA256 `af519227ecee9e6822f927d64d39e23428b120e7f5eadef86dea2262150ceb8f`.
The accepted comparison is
`target/animation/body-source-anchors-rust-comparison-v1.json`.

[UNKNOWN] This is a bounded decoded source-instruction oracle, not original-byte
host execution.
Optional-foot append, finalizer reroot/local quaternion and class producers,
channel selection and evaluated runtime pose binding remain separate gates.

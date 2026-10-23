# Canonical source extents and origin fields

[MEASURED] The direct loader9CBFA0's tail9CC5A8..9CC800 produces twelve floats
after the separately conformed seven-float rest boundary. Its601 bytes decode
to119 original instructions, with source SHA256
`5f02dabd78dadb36338943041c5de4c3e4717e890ed2444e46c077ff4ef64118`.
Executable SHA256 is
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`;
loader Evidence is
`ev_0789da66e7c2b2f79cb1e09c7665a7ab598558b8968857cb0bc80cdee48647ab`.
The tail has no calls or x87 operations; its only loop applies the same clamp
to three extents. Original bytes/disassembly remain local under target/.

## Exact returned fields

The new strict `anim_body_source_fields.rs` keeps the existing rest block and
returns half-extents138/13C/140, local origin144/148/14C, literal positive zeros
at324/328/32C, and effects origin330/334/338. It does not upgrade SourceRig or
claim the full initial block, anchors, optional construction or runtime poses.

[MEASURED] The six scaled bounding values commit to binary32 first, as in the
existing source intake. Each half-extent is `(scaledMax-scaledMin)*0.5`.
COMISS0,extent/JB retains positive values, including positive subnormals.
Zero and negative values receive original readonly constant1489324, bits
3DCCCCCD. There is no absolute value, minimum-positive tolerance or geometry
quality filter. The strict API reports finite input/output interface errors;
it does not describe them as native diagnostics.

The local origin starts with three committed differences
`d=sourceTranslation-restCenter`. With q=(x,y,z,w), retained binary32 products
`nx=-(x*x),ny=-(y*y),nz=-(z*z),nwx=-(x*w),nwy=-(y*w),nwz=-(z*w),
xy=y*x,xz=z*x,yz=z*y`, the original tail order is:

```text
ox=(((nz+ny)*dx + (xy-nwz)*dy) + (xz+nwy)*dz)*2 + dx
oy=(((xy+nwz)*dx + (nz+nx)*dy) + (yz-nwx)*dz)*2 + dy
oz=(((xz-nwy)*dx + (ny+nx)*dz) + (yz+nwx)*dy)*2 + dz
```

This is the actual expanded arithmetic, not a generic quaternion call. The
source-center producer, this inverse-origin producer and rotate99BE70 have
different summation orders that must remain distinct. No quaternion
normalization or canonical sign is inserted.

Effects parameters come from raw row60/64/68. The X output adds scaled parameter
to origin; Y/Z add origin to scaled parameter, in this order:

```text
ex=(row60*scale)+ox
ey=oy+(row64*scale)
ez=oz+(row68*scale)
```

The public APIs scope scalar SSE to MXCSR1F80 and restore all incoming caller
bits on every exit. The caller still explicitly supplies the x87 context used
by the earlier matrix-to-quaternion head. Retail thread policy is not inferred.

## Independent source-instruction oracle

[MEASURED] `tools/anim_body_fields_probe.py` decodes and interprets the original
tail instructions using independent integer binary32 arithmetic. It seeds
the exact prior machine state: scaled bounds at stack30/34/38 and44/48/4C;
XMM1 is committed centerY from9CC594; XMM3 is translationX-centerX from9CC590;
XMM5 is translationY until9CC5B0; XMM6 is translationZ-centerZ from9CC580;
XMM4 is the sign mask. The seven rest words come from the already conformed
original-instruction head/matrix-quaternion oracle. Every unseeded memory or
register read is an error. Original tail constants2,0.5,0.1 are pinned separately.

Fresh canonical reads cover all82 package indexes,1,365 creation resources and
46,418 source rows under007F/027F/037F, giving4,095 resource-context records
and139,254 evaluated rows. All returned fields are finite; there are zero
failures or source exclusions. Source data, original row bytes and all nineteen
returned words are included in each resource digest. The digest SHA256 is
`fe9cde4af3ec66ad2a94ae39addccc7ef7abdc40ffdf2404f55da1b25d3f5138`.
Evidence is `target/animation/body-source-fields-oracle-v1/report.json`.

In each context295 rows contain an extent exactly3DCCCCCD. This is a literal
output count; it is not asserted to count only clamp branches because an
unclamped source dimension could share those bits. The denominator includes
flora and other creation records, not only verified Creature-stage species.

[MEASURED] The exported Rust module passes all four focused tests, including
the hostile caller-state/subnormal restoration witness. A fresh independent
Python run and a separate fresh canonical Rust intake agree for all4,095
resource-context digests and139,254 evaluated rows, with zero arithmetic
exclusions. Both projections have SHA256
`fe9cde4af3ec66ad2a94ae39addccc7ef7abdc40ffdf2404f55da1b25d3f5138`.
The comparison is
`target/animation/body-source-fields-rust-comparison-v1.json`.

[UNKNOWN] No original-byte field host has run. Parent/child anchors, exact
default block state, optional-foot driver
progress and append behavior, finalizer relabeling/classes and evaluated
animation binding remain separate gates.

# Canonical finalizer anchor and local-orientation fields

[MEASURED] The canonical loader's accepted optional output now supplies all
inputs for the following projected finalizer operations. The new separate
`anim_body_finalizer_fields.rs` leaves the approximate `SourceRig` diagnostic
unchanged. It does not claim the later finalizer wrapper, selector classes,
constraints, animation-channel binding or an assembled evaluated creature.

The source remains the immutable base executable with SHA256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Finalizer9C3150 Evidence is
`ev_3dbb86207589fa36be3c6f46d0a0dc248ab8efac58dcfc8924f8ad57b7ddee12`;
complete relabel helper9B5DA0 is
`ev_e2aa9f86a82dbacd71fbeaa0d315c0fbcd75f5e3f3e88554449e12d965bf580a`.
Original bytes and disassembly stay under ignored `target/animation/`.

## Native source boundaries

| Operation | Native half-open range | Bytes | SHA256 |
| --- | --- | --- | --- |
| Root capability and reroot anchors | 9C3D80..9C407F | 767 | 4b572e91067412b87cd939a84816813f766479f7e2f529673e9f17ac70c77eea |
| Ordered local quaternion | 9C4463..9C4598 | 309 | 378173705528e688f5dfe96303da533193133107b3edf6af12124e718a8f9515 |
| Complete limb/spine relabel | 9B5DA0..9B5FF2 | 594 | 0a79b1e63fb0e22a5eed0fc0cd8ff56660ce9938c07b5ce6579e7b24202aed9f |

All three ranges have no CALL or absolute data dependency. Every conditional
branch stays within its declared range. The first two are explicit internal
instruction boundaries of9C3150, not complete functions. Their fixture supplies
the actual bounded register/stack state and an owned exit immediately after
each unchanged span. The complete helper returns to an owned caller.
Source preparation v1 stopped at an incorrectly requested mid-instruction end;
v2 retains the corrected source-only ranges. No original bytes were executed
by that preparation.

## Root capability, reversal and zeroing

[MEASURED] Earlier9C3150 admission requires the original initial root and later
selected root to have nonzero-spin mask154 bit8. The canonical wrapper starts
with original root ordinal0. At9C3D6C..9C3D78, ECX is the selected block,
EDI/EDX its ID array158, EAX and EBX are zero. Native code searches32 slots for
ROOT or the first zero, setting its number1. A full table with neither silently
keeps its slots. Mask rebuilding stops at the first zero and preserves all
duplicate IDs/stale number bytes; it is not a set insertion or compaction.

If selected root differs from initial ordinal0, the finalizer reverses that
parent chain. Each block receives the previous child's old parent-local anchor
as child anchor344, and its old child-local anchor as parent anchor350. The
selected root's temporary first swap is overwritten by root zeroing. Each
visited block recalculates35C..364 from its new child anchor, subtracting local
origin144..14C only when **presence-spin150 bit8** is set, including number0.
Its368 is the retained x87 norm, with each operation using the explicit caller
context:

```text
xx = x87.mul(load(dx),load(dx))
yy = x87.mul(load(dy),load(dy))
zz = x87.mul(load(dz),load(dz))
length = x87.store_f32(x87.sqrt(x87.add(zz,x87.add(yy,xx))))
```

The old root loses one direct child and the new root gains one. Intermediate
path degrees remain unchanged. Whether or not reversal runs, all ten selected
root words344..368 are set to positive zero. With selected root0 its original
anchor words are not consumed, so opaque NaNs there are not rejected.
Absolute centers/quaternions and palette IDs21C are untouched.

## Ordering and local quaternions

[MEASURED] The new operation calls the separately conformed child-contiguous
ordering using the rerooted parent tree and presence masks. It remaps ordinal,
parent, symmetry and child counts through that permutation. It preserves source
palette IDs, including appended FFFFFFFF feet. Automatic symmetry -2 is outside
the canonical source domain; no such association is invented.

The actual local quaternion range uses the parent and child's unchanged
absolute quaternions, with each product/sum committed to binary32 and source
sum order retained. For child(cx,cy,cz,cw), parent(px,py,pz,pw):

```text
lx = ((cx*pw-cw*px)+cy*pz)-cz*py
ly = ((cy*pw-cw*py)-cx*pz)+cz*px
lz = ((cx*py-cw*pz)-cy*px)+cz*pw
lw = ((cy*py+cx*px)+cw*pw)+cz*pz
```

The parentless root skips these stores and keeps its prior local quaternion,
including opaque/signed-zero bits. No normalization, norm division, sign
canonicalization or quaternion-quality tolerance is applied. Source constructors
provide identity at this point, but the explicit operation preserves the actual
supplied root words. Consumed finite quaternion components are bounded by2;
consumed chain anchors/origins by2^100. These are declared interface bounds,
not native quality checks. All binary32 arithmetic scopes MXCSR1F80 and restores
the complete caller state. Retail thread policy remains unestablished.

## Sequential capability paths

[MEASURED] Definition3AC initially stores initial root0;3B0 stores the last
initial ordinal with nonzero-spin mask154 bit8, before ROOT insertion/order.
Both ordinals are remapped during copying. Only when3F4 says the original
maximum-foot list is nonempty does9C3150 call9B5DA0 at remapped3AC and then3B0.
Each walk stops before ordinal0 or a block with more than one direct child
having nonzero-limb154 bit2. It ensures LIMB number1 and rebuilds masks, then
ensures SPIN number0 and rebuilds again, before walking to its parent.
Both calls are sequential and read masks changed by the first call. A retained
zero-number SPIN ID still sets presence-spin while clearing nonzero-spin unless
another duplicate nonzero SPIN remains. Full tables, first holes, duplicates,
and stale bytes keep native behavior.

The source palette prefix count reproduces37C's removal of trailing FFFFFFFF
IDs before ordering. Returned child/descendant ranges represent the later
runtime pointer relations without inventing process addresses. Rest matrix
production and model palette count/clamping remain their separately conformed
operations; selector classes and pose binding remain later work.

## Prepared source and native gates

[MEASURED] Fresh independent preparation opened all82 canonical package indexes,
1,365 creation resources and46,418 rows under three explicit nearest x87
precision contexts. All139,254 geometry/anchor evaluations are retained. There
are4,095 resource/context digests:3,966 admitted finalizer contexts and129
initial-nonzero-spin failures (43 creation resources). This includes flora;
it is not a count of Creature-stage species. The selected source policy is
threshold0.1, initial3F5 false, nearest-origin false and spine-midpoint true.
Those are explicit contexts, not a retail runtime-policy guarantee.

Preparation `target/animation/body-finalizer-fields-inputs-v2/` streams4,590
native requests:1,322 source reroot,1,322 source local-orientation and1,322 source
relabel stages at CW027F, plus624 synthetic stages across all12 masked CWords.
Synthetic cases include a255-block chain, presence-only spine/limb masks,
nonspine reversal, unconsumed root NaNs, nonunit quaternions, signed subnormals,
full32-slot tables, holes, duplicates and branch-stopped/sequential relabeling.
Each fixed projection is82,652 bytes, streamed instead of retained as a large
Python word array per case. The source digest SHA256 is
`71458fc4d9abc870ddf74d34bdc5cdb6d265ca51bc04b8e4cea4770401f8e595`;
input SHA256 is
`c41b69cdeaca2767102815f3ffb191377d96188e596d749fdbf8167d3c682b33`;
independent expected projection SHA256 is
`e09a5b85cc07bfbe51cd97179b82f2b4630ac746f7af3190a4857d3e7b62367b`.
Source preparation v1 stopped before reading a resource because its Capstone
runtime import path was not installed; v2 fixes only that preparation setup.

[MEASURED] Prepared native host v3 compiles/links and passes source/owned-code,
import, embedded-byte, section-permission and guarded-buffer layout audits.
Its unchanged source spans are copied into RX pages9B0000/9C0000. Two owned
seven-byte exit trampolines immediately follow the partial spans; gaps are
INT3. Owned code guards all buffers, input/definition shadows and every
unpermitted block byte, checks ESP/DF/CW/x87TOP/MXCSR and compares zero/all-one
XMM scratch projections. It does not claim partial-function GPR preservation.
V1 failed preparation on PROC-local cross-procedure labels. V2 corrected those
labels; v3 retains identical assembly and adds linked section/layout checks.

[MEASURED] Before the user-requested pause, root independently reproduced and
reviewed the linked host; the separate peer reviewed all4,590 input domains,
153,312 active blocks and21 pinned preparation files. Native execution v3 passes
all4,590 rows/34 protocol checks with52 unchanged execution pins, on the first
corpus attempt. Rust and Python projected streams have the same SHA256 as the
native projection above. Seven focused Rust tests pass. Three redundant trailing
`?` return operators were fixed after the first export compile failed; historical
pins and failed build log remain retained. The current core source SHA256 is
`08cbfd1cdadcff12652ac23c8bb6a2fea278c69a05a68bd3c4c0e33b9c43c739`.

[UNKNOWN] Full fresh-source Rust reconstruction and the formal combined report
remain pending at the pause. The module and both verifier examples are exported.
See the leading pause handback in `docs/continuation-2026-10-23.md` for the two
remaining commands. No rendering or complete runtime-animation milestone is
claimed. Later native constraints/classes and channel/pose binding remain open.

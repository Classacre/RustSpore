# Ordinary primary-root constructor, bounded native/Rust agreement

## Accepted continuation, 2026-10-27

[MEASURED] Root reproduced and reviewed the complete source constructor/mirror,
tables, owned assembly and linked executable. Independent peer review passed.
The exported Rust module and verifier match all15,113 native/Python contexts:
12,516 isolated source contexts using2,086 descriptors from818 clips and2,597
synthetics. All22 protocol checks and six typed Rust exclusions pass;37 native
execution pins remain unchanged. Projection SHA256:
`418fb855ce5f9e3a08b3159eafd5595a26ab03ed8cd7821c9ff7ee812f48aa70`.
Acceptance report:
`target/animation/root-binding-constructor-native-run-v2/native-v1/rust-comparison-v1.json`.

[MEASURED] Earlier host v3 passed22 protocols but its corpus exited71 with no
output. The failure is unclassified and retained; it was never retried. New
v4 changes only six pre-input VirtualAlloc/VirtualProtect failure branches to
owned exit10, preserving all other failure71 guards and all original bytes.
Root reproduced/audited v4, EXE SHA256
`4a7babb63b7d9d7ea078c9b28c9c6b97f53f4f843c452404a5d7c37b06a6b53d`.
The accepted corpus passes on its first attempt; per-protocol mapping retries
remain recorded separately. Only proven empty pre-input mapping failures can
retry, at most eight attempts. Neither failure nor preparation was overwritten.

[UNKNOWN] This gate supplies owner/claims/secondary/mirror state and isolates
one actual descriptor ordinal per source fixture. It does not prove complete
channel/builder pruning or actual actor association. Fresh whole-clip census
finds0/818 entirely supported root-only clips. A separate217/818 first-root
retention proof also encounters1,644 enabled nonroot selectors and leaves601
explicit exclusions. See `target/animation/root-whole-builder-followup-v1/`.

The following sections preserve the original preparation boundary. Statements
about pending compilation/execution describe that historical packet; the measured
acceptance above supersedes them. Frozen packets retain former `.pending` paths;
a new run after promotion requires new review pins rather than rewriting them.

## Historical preparation

[MEASURED] Saved source evidence maps primary selector mode1 to finalized runtime
root ordinal0. Function009A0E30 packs this ordinal into ordinary binding byte0C.
This byte identifies the finalized root, independently of source palette IDs.
The executable identity is
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The prior evidence IDs and exact source chain remain under ignored
`target/animation/root-selector-binding-source-prep-v1/`.

[INFERRED] The new unexported `anim_root_binding` implements the constructor's
ordinary record projection for exact selector flags1/5, selected root0, no more
than24 prepared descriptors, secondary count0 and available vector capacity.
It preserves component-ordinal mask bits, sequential nine-slot claims,
kind1 flag20, mirror flag2, explicit secondary flag10 and root/secondary counters.
Slot values outside0..8 are skipped. An admitted disabled descriptor sets the
native admission flag, which matters for empty fallback. An empty mask emits no
ordinary record. Empty fallback with AC bit8 is a typed unsupported boundary.
Special kind3 tag70E47545 is rejected independently of enabled/claim state.

[MEASURED] The retained97-byte mirror helper, both switch tables and zero
constant are source-pinned. Its source-reached modes are AC&32hex=0, returning
the explicit mirror argument, and20hex, returningzero. All2,086 literal prepared
data descriptors from818 clips reach these integer branches:2,071 return the
argument and15 returnzero. Spatial branches are retained unchanged in the draft
host, but unsupported in its input contract.

[MEASURED] The independent Python reference preparation produces15,113 contexts:
12,516 isolated source contexts and2,597 synthetic contexts. Each source fixture
contains one literal prepared descriptor at its original component ordinal;
other ordinals are explicit invalid-slot placeholders. Claims, owner byte,
secondary override, mirror argument and opaque record words are supplied state.
The literal metadataAC flag bits are retained, while CHAN+AE ownership is supplied
explicitly. This is coverage of a constructor boundary, not all live channels.
Six negative inputs exercise unsupported selector/root/mirror mode/pseudo/fallback
and descriptor limit. The verifier remains `.rs.pending`; no Rust acceptance is
claimed before root review and compilation.

[MEASURED] `anim_root_binding_native_probe.py` reads the immutable base-game
executable, checks its full SHA256, decodes the complete818-byte constructor and
97-byte mirror helper, and verifies every direct transfer against instruction
boundaries or the two declared unreachable external callees. It retains all
original bytes unchanged under ignored `target/`. Generated MASM is preparation
only: no assembler, linker, native process, retail process, GPU or desktop ran.
The EP1 executable was rejected by the source identity check before any output.

[INFERRED] The draft host bounds count/slots/root and uses separate guarded
buffers with full immutable shadows. Its available32-byte vector capacity
exceeds the single admitted ordinary16-byte record. It validates empty-fallback
exclusion before entering original code. Growth9A0980 and fallback9A0DD0 remain
unmodified external transfers into INT3 gaps and must be statically unreachable.
No replacement implementation or success stub runs for either callee.

[UNKNOWN] Native host compile/link, linked import/section audit, negative
protocol tests, scratch-state and native/Python/Rust agreement remain required.
Whole-builder pruning, initial prepared ownership and claims, live clip admission,
actual actor association and pose integration remain unsupported. Do not promote
the module or verifier based on Python reference preparation alone.

Root review commands, serialized with other builds:

```powershell
python tools/anim_root_binding_fixture.py --out target/animation/root-binding-constructor-inputs-root-review-v1
python tools/anim_root_binding_native_probe.py 'D:/Games/SPORE Collection/SporeBin/SporeApp.exe' --out target/animation/root-binding-constructor-host-root-review-v1
# After reviewing the module/reference and promoting only the pending example:
tools/build.bat run -p spore-bevy --example anim_root_binding_verify -- target/animation/root-binding-constructor-inputs-root-review-v1 target/animation/root-binding-constructor-rust-v1.bin
```

The source/host manifest lists remaining native gates. A root-owned linked audit
and reviewed runner are required before executing that host.

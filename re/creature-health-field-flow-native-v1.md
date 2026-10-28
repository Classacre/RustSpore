# Creature health candidate: constructor and bounded field consumers

The following is static native evidence, not a completed Creature-stage actor or
health system. Source: `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`;
REA dossiers reside under `target/rea/20261028-source-runtime-v6/`.

* Constructor `C1F940` (q80) calls `BCE6B0` with hashes of `Combat` and
  `StandardCreatureHealth`, fallback bits `42C80000`, then stores its x87 return
  as binary32 at whole-object offsets `+5D4` and `+5E0`.
* `C06870` (q269) and `C26FA0` (q270) call this constructor then replace its
  primary and subobject vtables. The former also constructs `Simulator/cAIMemory`.
  This supports a shared base, but is not proof of an individual actor identity.
* The primary vtable installed by `C1F940` (`146B0A8`) and the `C06870` derived
  primary table (`146AB98`) both have `C222B0` at byte offset `+78`. These table
  words were read directly from the PE image; a table entry is not proof of an
  actual live dispatch.
* `C222B0` (q276) captures the incoming ECX in ESI. It invokes the object at
  `+5A8` through vtable byte offset `+58`, computes the x87 division
  `field(+5E0) / returned_value`, and stores the quotient as binary32 on the
  stack at `C222E0`. After other calls it invokes that virtual method again,
  multiplies the result by the stored quotient, converts to binary32 and calls
  `CBCAA0` with ECX equal to object `+5A8`. `CBCAA0` (q292) stores its argument
  at subobject `+38`, i.e. whole-object `+5E0`. The Ghidra pseudocode of q276
  omits the division; use the assembly at `C222CA` and `C22331`. This is a
  conditional static ratio-preserving path, not a claim about exact numerical
  equivalence under all x87 control words, zeros, infinities, or NaNs.
* `C080F0` (q277), installed at byte offset `+8C` of the `146AB98` primary
  table, writes positive zero to whole-object `+5E0` at `C0812A` amid a larger
  sequence. Its event and dispatch context are unproven.
* The derived embedded table `146A948` slot `+58` points to `C05C40` (q295).
  That method uses a primary-object virtual call and runtime globals, flags,
  branches and conditional rounding. Its result is not yet established as a
  particular gameplay quantity. `C24890` (q291) calls `C222B0` and performs
  further operations, but its event and live actor identity also remain open.

The hash-matched candidate PROP is not proven to be the active source, and the
constructor field names, later mutations and event transitions are unresolved.
No game simulation should consume this result as health until lookup ownership,
live receiver identity, producer, transitions and numerical behavior are traced.
The unexported `creature_health_word_source_v1.rs` models only pre-x87 raw-word
selection and does not implement these downstream effects.

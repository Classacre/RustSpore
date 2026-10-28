# Cell animation association: effects-world correction

This note corrects the statement in `cell-runtime-animation-integration.md` that
E73F90 passes a selected animation resource to E83420. The retained evidence
establishes an effects-world product and an actor-associated object; it does not
establish that the first argument is an ANIM resource or a selected active clip.

## Evidence

For the SHA-256 pinned executable
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`,
Ghidra's exact function dossier for E5D9E0 shows factory 67D8F0 virtual slot
+4C called with `(01010011, "CellBackgroundEffectsWorld")`; its returned
pointer is assigned to global object +161BC. The same factory slot called with
`(01010021, "CellEffectsWorld")` supplies +161C8. These are factory-call
identities, not proof of the concrete runtime class or virtual implementation.
The matching E66A00 teardown passes those IDs to factory slot +50, clears both
fields and invokes the old pointers' virtual +4. EBP is zero at the two clear
instructions E66C3A/E66D86.

E73F90 selects +161C8 or +161BC based on an actor field, calls its virtual +8
with an output pointer, and, only when the resulting object is non-null, passes
that object as the first stack argument to E83420. A separate argument is
obtained from the actor's +248 handle through B71D00 and +24, conditional on
actor +FC; the third is actor +174. At E83420, the second argument's +17C is
passed as ECX to 9CB2F0, and 9CA920 receives the factory-output object and
third argument. This connects an effects-world product to an animation-side
cache path. Its precise type, the virtual method's semantics, and the third
argument's meaning remain unresolved. In particular, no selected clip, ANIM
key, controller assignment, or channel-to-rig mapping follows from this chain.

Evidence is retained locally under ignored `target/`: REA dossiers
`rea/20261028-source-runtime-v6/20-analyze_function.json` (E5D9E0) and
`17-analyze_function.json` (E66A00), and the earlier disassembly files
`source-cell/runtime-clip-association-trace-v1/v4-q10-assembly.txt` and
`v4-q04-assembly.txt`. No source executable is included in the repository.

The local candidate-location audit contains source instruction signatures and
is kept out of the public checkpoint. Raw matches alone are candidate locations;
the dossiers establish the observed instruction boundaries and semantics. The
source-resource-to-active-clip association still needs a separate original-code
trace before any preview animation can claim a source-backed selection.

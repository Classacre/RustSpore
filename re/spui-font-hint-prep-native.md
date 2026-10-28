# Original font0 PREP execution, 2026-10-27

[MEASURED] The source-instruction reference now executes the complete457-byte
font0 PREP at15x15ppem:690 VM dispatches,40 CALLs into five actual FPGM
bodies(8,31,63,69,84), and an empty final operand stack. It interprets29,610
original x86 instructions including CVT initialization. The incoming cursor,
CPU stack and nonvolatile registers restore. This is instruction interpretation,
not new native execution. It does not run the retail game.

[MEASURED] All173 CVT integers,47 storage integers,round callback8AB710,
CVT callback8CC230 and flags1 agree with2352 retained pre-glyph native context
snapshots across all80 corpus cases. Native context controls are x87CW037F,
MXCSR1F80,15x15ppem. The complete source-derived graphics/frame/twilight
state is also retained, but the host never emitted those complete structures:
that state is source-instruction-only, not independently native-verified.

Source SHA256 is17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a.
Font0 SHA256 isf6ad6f1e12e2d2a7c1e28b0ea3cb93a653482840b2b6340fb8682f3eb4c75f26.
The source oracle verifies the pinned saved font header. The pending Rust
verifier reconstructs the face from the installed Graphics package.

[MEASURED] Important initialization boundaries recovered from original code:

-8CC304..313 admits equal axis sizes by setting state+B6=1 and ppem+C.
-8CC4B6..4E1 stores CVT coefficient fields. B4 is aWORD: an accidentalDWORD
 write overwrites B6. Native comparison exposed that preparation bug.
-8CC156..1D9 allocates16 twilight points with fourDWORD, twoWORD and twoBYTE
 arrays, two zero contour endpoints, descriptor words1/0 and stride2C.
-8CC500..548 zeroes those arrays.8CC553..566 copies52-byte default graphics
 state at+24 into active state+58 before PREP.
-Frame+24 is the zone descriptor base. SZPS16 computes base+index*2C.
 Frame+2C is ordinary-loop mode0, not a graphics-state pointer.
-8ACB90..8ACC64 MIAP0 writes original/current twilight coordinates and moves
 through the current vector callbacks. The admitted source PREP touches point0.

[REJECTED] The initial prefix packet and twilight exploratoryv1..v7 are
preserved under target. Their square-state admission was false because the
coefficient preparation overwrote B6; they also initially lacked the52-byte
active/default graphics copy. They are debugging evidence, not acceptance.
The corrected full reference isfont-hint-prep-full-reference-v1.json.

The exportedspui_font_hint_prep.rs implementation executes the actual font0 path with
checked storage/CVT/stack/point access and a fuel bound. These checks are owned
API behavior, distinct from native unchecked accesses and nonlocal errors.
It uses only wrapping integer policies, including source grid/super rounding
and the original delta lower-bound search. Unknown opcodes, non-axis points,
nonzero zones and unsupported state fail closed. No floating-point fallback,
Swash or Skrifa is used. Root accepted all913 canonical rows against fresh independent source replay
and Rust, including both positive recursion/reserved-period guards. See
`target/source-ui/font-hint-prep-rust-comparison-v1.json`. This accepts the
bounded font0 PREP path; native complete graphics/twilight state was not emitted
and is not claimed independently verified.

[MEASURED] Review correctionv2 preserves literal SCANCTRL85 behavior:
8AD714 masks old flags toFFFF0000,8AD71A ORs the full poppedDWORD,
and8AD71C stores aDWORD. The popped value is not truncated toWORD.
SROUND reserved period bitsC0 explicitly select decimal999 at8AC894;
that is a measured source literal. The Rust API rejects that unvalidated
reserved case withUnsupportedSuperRoundPeriod, rather than admitting it.
A separate32-level CALL depth guard complements10,000-dispatch fuel.
Input table lengths are owned API admission guards, not font content identity.
The caller/verifier must independently bind source content. The v1 Rust and
verifier bytes are archived in ignoredfont-hint-prep-full-review-v1.
The pending verifier also constructs source-byte copies that reach recursion
and reserved-period guards; both checks passed in the accepted root verifier.

[UNKNOWN] Glyph programs, original outline-to-pixel rendering, other ppem,
aspect-scaledB6=0, other faces, complete font graphics native state and complete
UI screens remain open. Full font0 PREP is a necessary component, not a pixel
or gameplay acceptance claim.

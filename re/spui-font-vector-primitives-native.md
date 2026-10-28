# Font0 vector/intersection source frontier and primitive candidate

Root acceptance: all2,135 source/root/Rust primitive rows and five typed guard
assertions pass, with33 unchanged pins. The public module reproduces the private
candidate TSV exactly, SHA256
`70c059dbd2d99153f8930ed199ffc40fddc26ee15b407ec327a354e4f9e0bc9b`.
See `target/source-ui/font-vector-primitives-rust-run-v1/acceptance-v1.json`.
This is a primitive arithmetic gate; general VM opcodes, intersection and whole
glyph coverage require their separate comparisons. The preparation record below
remains historical.

Read-only successor research after the frozen state/contour packet. No VM
opcode, generalvector, intersection or font coverage is admitted by this packet.
No accepted module changed; root-only compiler/native gates remain separate.

Saved complete frontier closes07=8AC050(166B),87=8AC100(292B),0F=8AC3C0(641B).
Their exact callees are8D0EB0length185B,8D0DB0signed16.16divide90B,
8D0C70signed16.16multiply108B,8D0D50signed2.14multiply86B,
8ABE80projection/freedomdot92B,8AB870generalpointmovement251B,
8AB9D0currentprojection46B,8ABA00dualprojection46B. All are integer-only:
no x87/SSE/sqrt instruction or floating normalization library is involved.
All original ranges/bytes/hashes/disassembly are pinned in the new reference.
No REA query is missing.

8D0EB0 is a source approximation: take magnitudes; a zeroaxis returns the other;
seed max+floor(min/2),then exactly three fixed-div/mul updates, each
(y*div(y,guess)+x*div(x,guess)+guess+1)>>1 in original integer operations.
It must not become sqrt(x*x+y*y).8D0DB0 separately truncates integer quotient,
then repeatedly halves unsigned remainder and positive denominator until
remainder<=FFFF before fractionalDIV. This reduction was absent from the old
axis-only fixed_div guard and is preserved explicitly.8D0C70 truncates the
magnitude product at16bits then reapplies sign with DWORDwrap.8D0D50 rounds
magnitude product+2000 before14shift,then reapplies sign; WORDfactor is signed.

07 pops lastpoint then firstpoint and takes current first-minus-second;
87 does the same twice for current and original26.6 arrays. Each difference is
SHL10, length/divided,ADD1/SAR2,storedasWORD. Bit0 rotates [-y,x].07sets both
projection callbacks to8AB9D0;87sets current8AB9D0 anddual8ABA00 andretains both
normalized vectors. Neither rewrites freedom. Both install8AB870movement and
call8ABE80: separately rounded component products,WORDsum; a nearzero signed
dot(-1023..1023) clamps to+/-4000. Projection rounds components separately,
then DWORDadds. Movement has dot4000 fastpath2.14mul; otherwise it divides
(freedom<<4)/signeddot,16.16multipliesdelta,ADD4/SAR4. It touches only nonzero
freedom components,even when movementdelta0. These policies are literal source,
not algebraic vector identities.

New standalone unexported spui_font_vector_primitives.rs exposes only bounded
integerhelpers,not a VM. Magnitudes<=2^27 for scalarhelpers,<=2^24 forlength,
andlinecoordinate differences<=16384 are owned API guards proving positive
normalizer denominators/no signed-overflow magnitude corner. Typedbounds and
zero-denominator failures are explicit. Original source oracle interprets
2135ownedcases directly through original helpers and whole07/87handlers,
including remainder-reduction,negativecomponents,zero/nearperpendicular dot,
source movement/touches,dualvector andcallbackframe results. Expected2135rows
SHA70c059dbd2d99153f8930ed199ffc40fddc26ee15b407ec327a354e4f9e0bc9b.
Pending verifier reads those inputs and computes results independently; it
never uses source expected output to choose an answer. Five positive typed
failure assertions are included. Rust execution/primitive acceptance open.

0F is not replaced by an ideal intersection formula. It pops five indices,
marks both touchbits before branching, and has explicit horizontal/vertical
and zero-denominator fallback paths. Nondegenerate paths compare absolute
linecomponents,form fixed slopes,then choose fixedmultiply versus fixeddivide
again according to numerator/denominator magnitudes. Parallel fallback uses
separately arithmetic-shifted deltas/coordinates; averaging allfourpoints is
not equivalent. Next work should translate its original branch graph only
after this shared helper gate passes. The whole body and directcall closure
are already retained; no missing normalization query is necessary.

Full VM integration must replace projection,current/originaldualvector and
movement across every existing point handler coherently. Merely admitting87
while retaining r.axis-based movement would be wrong. Future canonical output
must retain fullframe projection/freedom/dualWORDs,dot andall3callbacks;
the existing finalaxis-only emitter rejects8nonaxis end states andcannotbe
used to erase them. Original wholepoints/state/rawalpha/phantomalignedorigins
remain the intended gate. Emptyprogram/maxp andpersistentCVT/cacheorder are
separate subsequent proofs; no glyph whitelist or pixel substitution is used.

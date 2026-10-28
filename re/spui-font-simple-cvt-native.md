# Ordinary-axis WCVTP and persistent font state plan

Root acceptance: the private and public Rust batch now matches352/389 simple
glyphs,160,922 numeric rows and1,408 retained native observations. All338 prior
successes remain. The1,183 candidate pins stayed unchanged; projection SHA256 is
`97a692b019412aaf50d644bf31fab12c40d4c3603d093683822555449bc430e4`.
See `target/source-ui/font-simple-cvt-rust-run-v1/acceptance-v1.json`.
The frozen pending verifier's old module alias was corrected in the archived
normal verifier before compilation; source model and numeric references did not
change. The following preparation record remains historical.

New isolated spui_font_hint_simple_cvt.rs adds only44 to the frozen338glyph
state/contour predecessor. Source8ADC60 first calls8ABA50,then pops value and
index. For sourcefactor10000(or0) it literalDWORDstores; otherwise it uses
original8D0DB0fixeddivide. This candidate admits only original ordinary-axis
projection with explicit sourceB8/BC=[10000,10000]. New API scale argument must
match those two values or typedState rejects. Shape/length still does not prove
fontidentity; originalfont0square15provenance is a caller requirement. General
vector/anisotropic factor paths remain excluded rather than silently treating
all WCVTP as directstore.

Source proof interprets whole44 and8ABA50 for4axisdirections×3CVTindices×10
DWORDvalues,including signedextremes. All120cases consumeexactly2stackvalues
and changeonlythe indexedDWORD. ReferenceSHA d8539e9befba994106c622fb86025be82105e34732d6489bbc9e04cefd802a00.
Batch expected389source/1556native targets remain176993numericrows SHA126226d6...
unchanged. Staticpotential352simple glyphs; two first44stops subsequently reach
87,so not354. Pending verifier has wrong-source-scale andCVTindex173 positive
failure assertions. No acceptedmodule/native/compiler action occurred.

Actual source342 writesCVT14 -192→-256 through44.354/gamma reads that shared
CVT and changesitsheight/alpha underthe retained342→354 source/native order
experiment. The existing accepted UI lifetime ledger closes same-font/sameppem
CVTstoragepersistence,perglyphGSdefaultrestore andcachemiss-onlyVM invocation;
cachehits skip this producer. PREP/defaults must not rerun for everycharacter.

The current pure VM's input PrepResult carries graphics/CVT/storage. A future
ordinary-sequence caller can retain the original PREP graphics as defaults,
copyforward only returned CVT/storage aftereach cachemiss,then invokeeachnew
glyphwithrestoreddefaultgraphics. It must not copythelastglyph'sgraphics back as
PREPdefaults. Twilight/generalvector paths remain outside this simpleordinary
model andrequiretheiractualpersistentstructures. Scale/facechanges and cache
ownership/reuse must followoriginalsourceepochs,not arbitrary percall choices.

342 becomes a concrete positive candidate here. Full342→354Rust pixels are
still open:354 uses87,projection/freedom controls and additional FPGM bodies/
arithmetic beyond74/75. Adding44 alone does not admitgamma or allseven native
sequence differences. The original12sequence observations remain a separate
futurewholepipelinegate. Emptyprogramhelperbypass andactualmaxpguardrelaxation
also remain separate proofs; no glyph-specific exception is introduced.

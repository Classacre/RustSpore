# Font0 controls successor candidate

The frozen predecessor passed230/389simple glyphs,101872numeric rows and920
retained native observations; all81prior successes were preserved. The new
unexported spui_font_hint_simple_controls.rs is a separate successor. Accepted
VM/raster files are unchanged. Root now accepts301/389 simple glyphs,135,475
numeric rows and1,204 retained native observations, preserving all230 earlier
successes.1,192 candidate pins held unchanged; public wiring yields identical
TSV. Evidence: `target/source-ui/font-simple-controls-rust-run-v1/acceptance-v1.json`,
SHA256 `a2a4d07a4258d0432bcb96aea9099e15380652f758bf050f46ff9ec11bada4a4`.
The module and normal verifier are public; subsequent candidate wording records
the preserved preparation stage.88 simple glyphs remain excluded.

The successor adds only SMD1A,RTG18,RTHG19,RTDG3D,the fullMIRPE0..FFfamily and
DELTAP3 72. Saved complete source ranges suffice; no REA query is missing.
8AC650 stores minimum distance as literalDWORD atstate6C(graphics20).8AC790's
byte/table switch resolves18to8AB710,19to8AB740,3Dto8AB680 and updatesstate70
(graphics24). Each admitted rounding caller passes compensation0. The new
round dispatcher preserves source wrapping/add/mask/sign behavior and rejects
other callback addresses. All49155 original instruction cases for three callbacks
and distances-8192..8192 were independently replayed,not calculated from Rust;
source proof manifest pins their TSV4194f6d0f471eab8bd7a5a24b0a929ca7085687b57f9856bff73ba1d43776121.

All32MIRP table entries target the same608B8AE590. The ordinary equal-axis and
singlewidth0/autoflip1 initial guards remain explicit. Lowdistance-color bits
are unused and original round callback gets literal0. Twilight/generalvector,
singlewidth and othercompensation paths remain rejected. DELTAP3's8AEBF9 adds32
inDX and8AEAF3 sign-extendsWORD base. Successor uses wrapped signedWORD base for
allthree groups,then the original unsigned0..15target test; the accepted actual
15ppem/default9 numeric path is unchanged. Existing packed argument search,
point movement and source shift policy are reused.

The independent expected generator reuses only frozen original-instruction
outputs and retained1556native observations for389simple glyphs. It does not
read Rust to derive numeric expected values. Whole176993expected rows are
unchanged from predecessor:SHA126226d6fae1d3e664740655960fc32a014ec2608b9d3cefc41663ddf3c26c2b.
Static necessary conditions leave301potentialsimple glyphs; actual typed guards
may reduce it. New first stops include87=22,GC49=16,SDB5E=5,DUP20=4,0F=7,
07=1,4D=1,shape5 andrasterstart27,withopcode-before-shape ordering. Dispatch
labels remain omitted because of the frozen>100byte historicallabel issue;
fullpoints/state/canonicalrawalpha and sourcephantom-alignedorigins remain.

Shape relaxation was evaluated but not silently included. Actual eight shape
first-stop candidates are gid5(92points,emptyprogram),132(59,empty),219(9,empty),
230(40,empty),400(69,403B),563(77,329B),569(81,331B),570(76,267B). Empty source
programs may take a driver bypass before VM phantom initialization. More-than64
points requires coordinated actualmaxp/API/raster-bound proof. Current guards
remainexplicit; this is separate work rather than per-glyph exceptions.
Full simple-font coverage,composites,sequence/cache/texture/shader/layout and
actualUI selection remain open. No normalization or recoloring is introduced.

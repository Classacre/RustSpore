# Font0 small-state and contour-start successor

Predecessor accepted301simple glyphs/135475numeric rows/1204retained native
observations. Accepted modules remain frozen. New unexported state VM and
contour raster are separate candidates; root compilation/numeric checks open.

5E uses original8AEA70 lowWORD store atgraphics42.4D uses8ADF40 byte1 at
graphics50.49 is not GC:8ADDF0 decrements49to48,TESTDL1=0 selects8ADE46's
current coordinate arrays and projectioncallbackframe48. It replaces two stack
points with current first-minus-second; design-coordinate branch belongs48.
Only the already admitted ordinary equal-axis zones are supported.

Original function74 is exactly13B atFPGM1332:
208a2349648a2353583c1b2159. Its completeDUP20/ROLL8A/SWAP23/49/ABS64/
GTEQ53/IF/ALIGNRP/ELSE/POP/EIF body is supported generically. Original switch
maps53to8ADFDBGTEQ,notGT;64maps8AE183 wrappingABS. Function identity is carried
through boundedCALL recursion so74's reachedtrue branch is allowed while75's
previous unsupportedtrue branch remains rejected. All other function IDs still
fail closed. DUP is bounded by source maxp stack capacity.

Original8C8000's complete two-range contour traversal closes the former start
exclusions. On-curve halfgrid starts seed prior-axis values from the lastpoint,
possibly previouslastoncurvepoint, and decrement firstcoordinate under the
literal original conditions. Off-curve starts use lastoncurve or implied
ceil-midpoint anchors; exact-halfgrid endpoints are decremented in copied
coordinate arrays. The first control remains the original pre-mutation point,
matching savedsource locals20/24. No contour rotation or modern approximation
is used. Ordinary line/quadratic/gray arithmetic stays identical. Input points
remain untouched because only the raster's owned copy is mutated. Source
bounds are computed before these mutations,as before.

Source-proof-v1 retains complete handler/contour bytes/ranges and46 original
instruction handler cases,including signed comparison/ABS/DUP/ROLL/SWAP/WORD
stores. Eight owned geometry cases exercise oncurvehalfX/Y,offcurvefirst,
lastoff,starttie,previousoff/equal andboth-off midpointtie paths; both source
dropoutsettings produce326numeric rows SHA3633f8e231edbf44f56a3ba7895217f208bf0b7be18521e981d8747135406993.
Their Rust verifier reads externalownedgeometry.tsv; no coordinates/pixels are
embedded. Root must compare this geometry gate and all578glyph batch before
accepting the candidate.

The source/native expected389simple targets remain176993rows SHA126226d6...
unchanged. Static necessaryconditions leave338potentialsimple glyphs; actual
VM/geometry guards may reduce it. First remaining stops are87=22,44=16,
0F=7,07=1,shape5 withopcode-firstordering. Generalvectors/intersections are
not admitted. Emptyprogram andactualmaxp bounds remain separate open proof.
No sourcequerymissing. Historical>100bytePC/stream labels remain excluded;
whole state andrawalpha/finalphantom-alignedorigins remain mandatory. Cache,
sequences,atlas/shader/layout/fullUI andcomposites remain separate gates.

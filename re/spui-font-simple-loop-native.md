# Font0 MDRP and SLOOP isolated candidate

The accepted baseline remains81/389simple glyphs,34520numeric rows and324retained
fresh native observations (root acceptance-v1). Accepted modules are unchanged.
The new unexported spui_font_hint_simple_loop.rs copies the accepted curved VM
and extends only MDRPC0..DF and SLOOP17 plus its three existing loop consumers.
The accepted raster is reused; no per-glyph coordinates, alpha or color mapping
is embedded. Root now accepts230/389 simple glyphs,101,872 numeric rows and920
retained native observations, preserving all81 earlier successes.1,187 candidate
pins held unchanged; public import wiring gives the identical TSV. Evidence:
`target/source-ui/font-simple-loop-rust-run-v1/acceptance-v1.json`, SHA256
`af550e389dec462886f2c98635e7433cd0fd202af6565ea1479835eb26229693`.
The module and normal verifier are public; earlier preparation wording below
is historical. No complete font/UI acceptance follows from this bounded gate.

All32 MDRP entries resolve to original8AE410, whose complete379B body is already
retained. The existing ordinary equal-axis/design-word source-unit projection,
round and minimum-distance arithmetic applies to all32 values: lowbits0/1 are
unused,roundbit4 passes literalgray0,bit8 clamps using unrounded sign,bit10sets
RP0; RP1oldRP0/RP2point always. The required sourcegraphicsDWORD4=0 excludes the
unvalidated single-width branch. No new single-width/twilight/general-vector
claim is made. Expanding the opcode match changes no existing arithmetic.

SLOOP dispatch8AC6E0 index7 targets8AC70B: DECfullECX then store lowCX atframe3E.
SHP8AD030,IP8AD310 andALIGNRP8AD530 test that WORD as signed before consuming
points and reset it to0 afterwards. Thus lowword1..32768 executes that many;
0or32769..65535 executeszero; upperDWORDbits ignored. Each consumer computes
its source reference/ratio setup before the count test. The owned API requires
sufficient stack before iteration,otherwise typedStack rejection. All counts
remain bounded by original maxp stack capacity. Full source boundaries/hashes,
original bytes/disassembly and196608 independently derived WORD reference rows
are in font-simple-loop-source-proof-v2. The first partial proof directory is
retained: its failed checker literal omitted the MOVWORD66prefix,not source or
native execution. No REA query is missing.

The new pending batch verifier attempts all578 glyphs transactionally through
this generic candidate. New source fixtures preserve all389original numeric
references and1556retainednative observations; their values are not generated
from Rust. They add original finalframe3E to whole point/state/rawalpha/event/
origin checks. Static necessary conditions nowleave230potentialsimple glyphs;
actual Rust first-stop results may reduce that. Remaining first opcode stops
includeSMD1A39,DELTAP372=26,RTDG3D18,RTG1917,SDPVTL8717,GC4913.
No other handler is silently admitted. OldPC>100 trace-label issue is still
explicit; dispatch labels are excluded,whole numeric source state is retained.
Final origin still subtracts current first phantom thenANDFFFFFFC0; noalpha
normalization. No font sequence/cache/atlas/shader/layout/UI admission follows
from this fresh-glyph candidate.

# Original locale tables and source captions

`[MEASURED]` The canonical archive reader exposes 4,497 type `02FAC0B6`
resources in scope `02FABF01`, across 23 installed language packages and all
82 inspected package indexes. The previous compatibility index mislabeled
these resources as type zero. No payload bytes changed.

`[MEASURED]` Exact original function `6B4EE0` requests the three-word key
`[table ID, 02FAC0B6, 02FABF01]` through the resource service. The returned
resource's slot `14` resolves a string ID. `6B6BB0` performs an unsigned map
lookup and returns an empty string for a missing key. Map assignment `564880`
means a repeated ID selects the last stored entry. `6B4EE0` treats an empty
resolved C string as lookup failure. These facts do not establish the active
product, language fallback or package overlay order.

`[MEASURED]` Reader `6B6CC0`, with byte source `6B5F50`, implements a state
machine rather than one ID/value pair per line:

- Optional `EF BB BF` is consumed only at the beginning. An initial `EF`
  without the remaining BOM bytes fails.
- ASCII space and bytes `09..0D` separate tokens. `#` begins a comment while
  waiting for an ID or its text; comments can separate an ID from its value.
- An explicit ID starts with `0x` or `0X` and consumes at most eight hex
  digits, including zero digits. A ninth hex digit starts the value.
- Unprefixed text receives the previous stored ID plus one with wrapping.
  `FFFFFFFF` is also the unset-ID sentinel. A trailing ID without a value
  succeeds without storing an entry.
- Text consumes bytes up to CR, LF, NUL or EOF. Inline `#`, markup and
  trailing whitespace remain literal. NUL terminates the byte reader.

`[MEASURED]` Conversion wrapper `93CBF0` invokes `93C5E0` with explicit
source encoding `8`, destination encoding `10`, a byte length and output
capacity. The selected path converts ASCII and two/three-byte sequences to
one 16-bit unit. Instructions `93C8A2..93C8BA` also use 16-bit shifts for a
four-byte sequence, followed by the single word store at `93C8D1`. Thus a
valid supplementary character becomes its low 16 bits, not a surrogate pair.
The Rust checked implementation preserves this quirk. All 4,497 installed
tables contain only BMP characters, so this distinction changes no current
source entry.

`[MEASURED]` `spore-core::locale_table` retains stored order, overwritten
duplicates, exact converted units and source byte ranges. Independent Python
and fresh Rust canonical-archive intake agree on all 4,497 per-resource
digests, 419,485 stored entries and 27,408,306 converted units. The source
cohort includes 175 automatic IDs and 46 overwritten entries. The sorted LF
TSV has 296,802 bytes and SHA256
`caf1aed378f249d517c98793e0d2dfa738f02a4f159a2f9e9825f8bcf54a0059`.
Evidence is under `target/source-locale/corpus-v1/`; raw tables remain ignored.

Reproduce fresh intake with:

```powershell
python tools/locale_table_probe.py "D:/Games/SPORE Collection" --out target/source-locale/NEW-PYTHON
tools\build.bat run -p spore-bevy --example locale_table_verify -- "D:/Games/SPORE Collection" target/source-locale/NEW-RUST.tsv
```

`[UNKNOWN]` The native table parser has not yet been executed by a fixture.
This is independent corpus agreement with instruction-derived grammar,
not native-runtime text conformance. Invalid UTF8 is an explicit Rust error;
the original converter's partial/malformed conversion is outside this checked
domain. Automatic numbering after `FFFFFFFE` would loop at the unset sentinel
in the original parser, so the bounded API reports a separate error.

`[UNKNOWN]` Final detokenization, runtime variable substitution, locale
selection, text shaping/layout and original glyph pixels remain separate
gates. `6B4D40` stores the raw text, optionally owns a copy and asks the string
service to classify it. It is not itself the token substitution engine.
No host-system font or invented caption closes those gates.

The strict REA dossiers are queries 824, 826 and 829..833, plus 835 for raw
text assignment, under `target/rea/20261022-source-runtime-v3/`. Every function
used above has exact requested/returned entry agreement and original executable
SHA256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.

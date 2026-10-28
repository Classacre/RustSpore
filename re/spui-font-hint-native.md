# Original ordinary font hint-program setup

This note covers original SPORE font-program setup recovered from the pinned
`SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The existing native glyph oracle uses the original scaler; the Rust hint VM
and rasterizer remain incomplete. Source font/native-code payloads and
canonical comparison data stay under ignored `target/`.

`[RECOVERED]` Ordinary loop `8ABCA0` reads an opcode, stores it at its frame+4,
increments the cursor at frame+0, and calls the source opcode-table target.
The table is state+8C; the operand stack top is frame+20. Frame+28 points at
the inner hint state. The loop restores its incoming frame cursor before
returning. These are measured private interfaces, separate from a proposed
public Rust engine interface.

Source table `1432F30` maps NPUSHB40 to `8ADB40`, NPUSHW41 to `8ADBD0`,
PUSHB B0..B7 to `8ADB00`, PUSHW B8..BF to `8ADB80`, and FDEF2C to `8AE840`.
Byte values are zero-extended into dword stack entries. Word values combine
two big-endian bytes and sign-extend the resulting word. Both variable-count
handlers admit count0 and perform no element write in that case.

FDEF pops a dword index, selects an eight-byte record in state+90, and writes
program selector from the byte at state+C9 as a word at record+6. It stores
the body offset relative to program-base array state+9C+selector*4 at record+0.
It scans to the first opcode ENDF using helper `8AE1E0`, then stores body
length as a word at record+4, excluding ENDF. A later FDEF at the same index
overwrites that binding. The native code has no corresponding bounds checks.
Its word-length store truncates; the bounded Rust API rejects bodies longer
than65535 bytes rather than claiming native malformed-input equivalence.

The scanner's original remap `8AE238` and five-target table `8AE224` distinguish
NPUSHB, NPUSHW, fixed PUSHB, fixed PUSHW and ordinary one-byte opcodes.
Thus an ENDF-valued byte inside any PUSH payload remains data. FDEF scanning
does not execute its bodies, evaluate their jumps or dispatch their handlers.
Lexical body opcodes do not establish later reachability.

`[ACCEPTED: BOUNDED SETUP]` Exported `spui_font_hint_program` implements only PUSH/FDEF
setup with an initially empty operand stack and FPGM program selector1.
It borrows original body slices, preserves offset/length/selector and reports
indexed definitions, top-level dispatch positions, stack depth and remaining
stack. None bindings do not assert native unused-slot initialization.
Explicit maxp1.0 capacities bound stack/table access. Other top-level opcodes,
including CALL, are unsupported. Its public guards are owned API behavior.

`[MEASURED]` At the user-requested pause, root reproduced66,626 reference rows
exactly, SHA256
`2d261c579c37dcc2972eeb7474531ee88b44ff4e50d067363570b077ed7b3f2b`.
On2026-10-27 root passed allfive focused tests and reproduced every66,626
canonical row with Rust. See
`target/source-ui/font-hint-program-rust-comparison-v1.json`. This accepts
only setup; no function body executes in that verifier.

The independent source-instruction oracle interprets the entire original
ordinary loop, all four PUSH handlers, FDEF and the original PUSH scanner,
including actual source dispatch/switch table reads. It uses strict bounded
model memory; only the specified indirect targets are admitted. It checks
native stack and nonvolatile-register restoration, prior frame-cursor
restoration and untouched unused table slots in its explicit A5 model.
No native machine code runs in this oracle.

The canonical reference covers all256 opcode widths, both variable counts
0..255 (766 scanner cases total), all65536 signed PUSH-word encodings and
nine owned setup samples. Those include count0, unsigned-byte values, signed
word boundaries, opaque ENDF data and duplicate-definition overwrite.
Fresh canonical package reads then supply all five installed font faces:

| Canonical instance | FPGM bytes | Definitions | Setup dispatches | Peak stack |
| --- | ---: | ---: | ---: | ---: |
| 511B49CA | 1514 | 53 | 54 | 53 |
| 61639DFB | 354 | 10 | 20 | 1 |
| 6BBC4E0E | 354 | 10 | 20 | 1 |
| BACAAAA5 | 1898 | 64 | 65 | 64 |
| FFE355DF | absent | 0 | 0 | 0 |

The four present streams contain only PUSH/FDEF at top level; their bodies
remain opaque. FontTools4.60.1 independent disassembly agrees on definition
order and maxp capacities. The total is137 defined bindings and159 actual
setup dispatches. The expected file has66,626 rows and1,147,937 bytes, SHA-256
`2d261c579c37dcc2972eeb7474531ee88b44ff4e50d067363570b077ed7b3f2b`.
It is `target/source-ui/font-hint-program-expected-v2.tsv`, with report
`font-hint-program-instruction-oracle-v2.json` and frozen review packet
`font-hint-program-reference-packet-v1.json`. The earlier identical reference
is retained; v2 refreshes the Rust verifier pin after a local lifetime fix.
Five prepared tests and the Rust comparison await root's serialized checks.

`[NATIVE OVERLAP]` The already reviewed native v2 glyph host's `check_fpgm`
checks each of the53 defined font0 records for exact offset, length and
selector1 after native font construction. The new instruction-derived
font0 records agree with that pinned header. Successful original glyph
contexts therefore already cover those53 defined stores through native
assertions. This does not measure unused slots, execute Rust function bodies,
or extend the native font0 pixel scope to the other four faces.

`[RECOVERED]` CALL2B dispatches to `8AE800`. It decrements the operand-stack
pointer, reads the complete dword function index, and resolves record selector,
offset and word length. It calls frame+50 with the same frame pointer and an
exclusive body end. It does not switch the inner-state program-selector byte
to the record's selector. Ordinary callback `8ABCA0` saves/restores the caller
cursor, while operand-stack mutations and last-opcode writes persist.
ENDF2D itself dispatches to error1101 at `8AB660`; a correctly resolved body
ends before ENDF and therefore never dispatches that opcode.

LOOPCALL2A (`8AE8B0`) first pops/resolves the dword function index, then pops
its count. The count is read with MOV AX, decremented in AX, zero-extended
into EBX, and tested with signed TEST BX/JL. Low-word counts1..32768 execute
that many iterations;0 and32769..65535 execute zero. The high word is ignored.
The body range and callback are resolved once before the loop. Each successful
ordinary callback restores the cursor and preserves the loop counter through
the native nonvolatile-register convention. The source caches its callback
by overwriting its own caller-argument slot at entryESP+4 (`8AE8F7`); later
`8AE903` reads that slot after pushing its three callback arguments. This is
an internal ABI fact; the ordinary caller subsequently discards that slot.

`[ACCEPTED: BOUNDED CONTROL]` Exported `spui_font_hint_control` models this range lookup,
count rule and shared-frame scope. Its explicit program windows use modeled
u32 addresses, never dereferenced native pointers. It checks the stored range
against the borrowed body and rejects undefined/out-of-range bindings and
address wrap. Definition lookup precedes the count pop even when the count
will select zero calls. A supplied ordinary body processor can mutate the
shared stack, opcode, selector and external state. Successful scope exit
restores only the cursor. Cursor cleanup on a returned error is owned policy,
not a claim to emulate native longjmp recovery. No body opcode is implemented
by this control foundation. Root promoted its verifier after acceptance;
the original pending-name preparation pin remains historical evidence.

The separate control instruction reference exhausts every count word and
upper-word aliases. Full-call fixtures use only an owned PUSHB1/ADD body,
through the original CALL/LOOPCALL, ordinary dispatcher and PUSH/arithmetic
handlers. The shared arithmetic entry is admitted only for constant opcode60;
its other operation paths remain excluded. Actual installed FPGM bindings
are checked for source range resolution with execution stopped before body
entry. This is a bounded control gate, not a traced/matched retail PREP path.

The control reference interprets11,001,450 original integer instructions:
65,536 count words,9 upper-word aliases,12 complete owned-body cases and137
installed range-resolution records. It observes the original internal caller
slot overwrite and verifies native-model ESP/nonvolatile preservation,
same-frame operand-stack changes, cursor restoration and unchanged selector
byte. Its65,699 canonical rows contain1,035,129 bytes, SHA-256
`f0d5dd557d74f4f33b555044933d6b01d3b481a1a34035e393b3ee03e2352c7b`.
The expected file and source/model/port pins are retained in
`target/source-ui/font-hint-control-reference-packet-v1.json`.
On2026-10-27 root passed allfive focused tests and matched all65,699 Rust
rows. See`target/source-ui/font-hint-control-rust-comparison-v1.json`.

`[UNKNOWN]` Rust PREP execution and its persistent CVT/storage state, point-zone
setup, source function-body execution, glyph hint execution and scan conversion
remain separate implementation gates. The new fullfont0 PREP source-instruction
reference and unexported Rust draft are described in
[spui-font-hint-prep-native.md](spui-font-hint-prep-native.md).
The measured fresh/sequence differences
in seven font0 glyphs require retaining native CVT lifetime; see
[spui-font-fixture-native.md](spui-font-fixture-native.md). No system font
engine is substituted and no Rust native-pixel equivalence is claimed.

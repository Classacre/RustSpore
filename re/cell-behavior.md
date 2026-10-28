# Native Cell definitions and behavior consumers

## Bounded progress-accounting comparison, 2026-10-28

The integer accounting slice `E7CE91..E7CF18` now matches the Rust
`cell_progression_v1` model in 1,504 boundary and seeded cases. The comparison
includes signed 32-bit wrapping, the source 1,000 cap, source-byte counter
routing and the changed-route branch; both hosts also reject seven malformed
protocol inputs. The cap was verified in the installed executable's read-only
`.rdata`. Source, input, runner and binary hashes matched again after the run.
Evidence is `target/source-cell/cell-progress-run-v3/`.

A separate `cell_feeding_route_v2` model connects statically recovered food
dispatch to that accounting slice. It requires explicit snapshots before healing,
after intervening calls for routing, and after `E79750` for accounting; it also
requires caller admissions for valid pointers and callback preservation.
Zero-food cleanup remains pending, and earlier bomb/nova effects are not
reconstructed from later data. Four focused Rust tests and an independent static
review pass. This composition has not been compared to a complete native feeding
operation or integrated into gameplay.

This test supplies the state after the earlier `E79750` call and isolates the
136-byte numeric slice. It does not establish live memory interference, the
effects of that earlier call, food/source selection, growth, tutorial or stage
transition side effects. The native host and x64 Rust adapter had independent
static/linked reviews; the adapter's build record is not a cryptographic proof
of compiler semantics. The previous rejected host reviews remain in `target/`.

## Earlier source context

Scope: immutable base `SporeBin/SporeApp.exe`, SHA256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`,
and every installed package index, inspected 2026-10-23. Retained REA v3
Evidence is under `target/rea/20261022-source-runtime-v3/`. Disassembly,
source resource bytes and fixtures stay under ignored `target/`.

## Native structure identity

`[MEASURED]` Static registrar `1375C30` gives descriptor `16AE490` the
name `cell`, size `31C` (796 bytes), fields `15A6FC8`, and the lowercase
FNV-1 type identity `DFAD9F51`. Native `E4A560` stores the name, fields,
hash and size and links the descriptor into registry `16AE044`.
`E4A750` hashes a non-null field name; `94E9D0` is an identity function.
The field initializer at `1375C70` therefore gives actual names, offsets
and primitive/nested types rather than guesses from numeric values.

Related native registrars are `1374500` (`cAIData`, `B4` bytes),
`13758E0` (`cEatData`, `C` bytes) and `1375AF0`
(`cLocalizedString`, `A4` bytes). Their corresponding initializers are
`1374560`, `1375920` and `1375B30`. Some entries are undefined in the
focused Ghidra database. Those boundaries and initializers were checked
with bounded Microsoft SDK disassembly, without substituting unrelated
pseudocode.

| Source cell field | Offset | Stored representation |
|---|---:|---|
| structure | `000` | 32-bit source reference ID |
| name | `004` | `cLocalizedString`, `A4` bytes |
| hp | `0A8` | 32-bit integer |
| fixedOrientation | `0AC` | byte flag |
| flags | `0B0` | 32-bit word |
| cellType / unlockType / density / sound | `0B4` / `0B8` / `0BC` / `0C0` | raw 32-bit codes |
| break / pieces / leak / expel | `0C4` / `0C8` / `0CC` / `0D0` | source reference IDs |
| loot / explosionTable / poison | `0D4` / `0D8` / `0DC` | source reference IDs |
| ai / ai_hard / ai_easy | `0E0` / `194` / `248` | three distinct `B4`-byte subobjects |
| friendGroup | `2FC` | 32-bit integer |
| wontAttackPlayer / wontAttackPlayerWhenSmall | `300` / `301` | byte flags |
| size | `304` | two float words, read by `E5D100` |
| eat | `30C` | `C`-byte `cEatData` |
| triggersEscapeMission | `318` | byte flag |

`[MEASURED]` The localized text member is exactly `A0` bytes, or **80
UTF-16 code units**. Its following unnamed integer is at local `A0`,
absolute `A4`. The former 396-unit legacy label reader and the preview's
128-unit bound could read structured data as text when no null terminator
occurred. Both now honor the native bound and decode surrogate pairs.
The trailing word is preserved without inventing a localization role.

## AI and eating data

`[MEASURED]` Each `cAIData` has 50 descriptor leaves: one unnamed code,
49 named fields, including 30 floats and nine byte flags. The parser
preserves unknown codes and all padding through its immutable raw view.

| Source AI fields | Local offsets |
|---|---|
| unnamed code / movementStyle / flocking | `00` / `10` / `14` |
| awarenessRadius / awarenessRadiusFood / awarenessRadiusPredator | `04` / `08` / `0C` |
| speed / chaseSpeed / wanderSpeed / fleeSpeed | `18` / `1C` / `20` / `24` |
| fearsNearbyDamageRadius / fearsNearbyDeathRadius | `28` / `2C` |
| fearsNearbyDamageTime / fearsNearbyDeathTime | `30` / `34` |
| protectRadius / protectTime / turnFactor / axialMovement | `38` / `3C` / `40` / `44` |
| spawnTime / spawnRestTime / spawnOutput | `48` / `4C` / `50` |
| arcLength / arcLengthSecondary / numArcs | `54` / `58` / `5C` |
| keyTransformation / keyProjectile / food | `60` / `64` / `68` |
| growCount / digestionCount / digestionTime / digestionOutput | `6C` / `70` / `74` / `78` |
| fleeTime / fleeRestTime / chaseTime / chaseRestTime | `7C` / `80` / `84` / `88` |
| chasesDamage / fearsMouths / fearsWeapons / fearsElectric | `8C` / `8D` / `8E` / `8F` |
| fearsPoison / fearsDamage / ignoresFood | `90` / `91` / `92` |
| awakeTime / sleepTime / growAmount / hatchDuration | `94` / `98` / `9C` / `A0` |
| poisonRecharge / electricRecharge / electricRechargeVsSmall / electricDischarge | `A4` / `A8` / `AC` / `B0` |

`cEatData` has integer `foodValue` at `0`, integer `hpValue` at `4`,
byte `bomb` at `8` and byte `poisonNova` at `9`. No diet name, movement
formula or effect implementation is inferred solely from these labels.

`[MEASURED]` Associated native name/value tables are retained in
`target/source-cell/native-cell-enum-tables-v1.json`. The unnamed AI word's
table (`14834B0`) names `none`/`player` as zero, `parts` as `1001`,
`straight` as `1002`, and the controller sequence `slowSeek`, `orbit`,
`chomp`, `eater`, `parasite`, `caterpillar`, `sleeper`, `waterballoon`,
`mosquito`, `dragon`, `poolball`, `leak` as `1003` through `100E`.
`movementStyle` names flagella/cilia/jet bits 1/2/4 and turn-in-place bit 8.
These table associations still need the text-parser consumer before adding
a complete typed enum dictionary to the public API.

`cellType` and AI `food` share table `1483368`: none 0, rock 1, solid 2,
liquid 3, air 4, poison 5, unlock 6 and nanite 7. This is not the
experimental simulation's Herbivore/Carnivore/Omnivore enum. In particular,
source `sound` has a separate plant label at code 2; exchanging these
tables would create incorrect behavior from superficially similar values.

The exact 223 PatchData definitions select the same controller distribution
at all three measured difficulty codes after inheritance: 71 eater,
22 parts, 16 caterpillar, five straight, 99 zero and ten remaining named
controller rows. Zero also covers nonactor content; it is not an unused-
resource classification. The census is retained in
`target/source-cell/native-cell-patch-controller-census-v1.json`.

## Actual native read and reference route

`[MEASURED]` `E4AE20` calls base resource loading, finds the descriptor
by type hash, then applies `E4ABC0` and `E4AAD0` fixups. `E4AAD0`
walks 40-byte field descriptors, uses their `+8` offset and `+10` type,
and replaces nonzero type-3 IDs with `E82450` reference handles. That
helper returns a cache handle, **not the final Cell data pointer**.
`E823D0` acquires the resource through `E822F0` / `E82200`, queries
interface `2269ED1`, and returns its `+14` data member. `E4CAA0` is
the Cell consumers' thunk to this helper.

`E4A630` corroborates primitive sizes: four bytes for types 1/2/3/5/6/7,
`C` for 8, `24` for 9, `A0` for 10 and eight for 11. Its nominal size
for type 6 must not be used to read four bytes from packed byte flags;
the descriptors and actual boolean instructions establish those offsets.

`[UNKNOWN]` Directory/configuration functions `E4CC40`, `E4C930`,
`E4BF90` and their source `.cel` caching route are not standalone
Cell-definition readers. Their presence does not establish gameplay rules.

## Measured gameplay consumer slices

- `E572E0` resolves each actor's definition handle at actor `108`.
  Its predicate returns one for equal definition handles, or matching
  nonzero `friendGroup`, or the measured player-exemption branches.
  The exemption bytes are compared to **one**, while another actor byte
  at `112` can force a zero result. Caller semantics and the helper
  `E571C0` still need tracing before this becomes a complete combat policy.
- `E4FD70` returns the resolved definition's escape byte at `318`.
  Caller `E5D3D0` is the next transition/mission consumer to inspect.
  A flag read does not complete the Creature-stage transition.
- `E573E0` selects `ai_easy` for raw difficulty zero, `ai_hard` for two,
  and base `ai` otherwise. An alternate's first word `FFFFFFFF` inherits
  base `ai`. The final predicate accepts a nonzero selected first word or
  the player. `selected_ai_variant` and `passes_native_ai_gate` retain
  these branches; additional meanings of the unnamed code remain open.
- Bounded SDK `E57470` uses the same selection, then prioritizes
  `fearsDamage == 1` over `chasesDamage == 1`, loading `fleeTime` or
  `chaseTime` into actor timers and clearing opposing state. Wrapper
  `E57570` restricts this route by controller. Its complete dispatch and
  timer/target semantics remain outside the implemented numeric helpers.
- `E5D100` reads `size` as floats, interpolates a random value between
  the two endpoints, bounds it, multiplies a stage-indexed table value
  and `0.033333335`, then rounds to float. Native RNG, stage bounds,
  table identity and x87 rounding remain outside the implemented parser.
- `E7D0A0` resolves a consumed definition, invokes bomb/nova effects,
  updates the player's integer health and dispatches food/progress effects.
  The bounded `eating_numerics` implementation reproduces its health and
  food-value branches only. Health below six gains two on the explicit
  absent/special-definition path. A present positive `hpValue` gains twice
  that amount when raw save-state `+7C` is zero, otherwise once. Health
  is capped at six. Present zero `foodValue` exits after effect/healing
  work; the explicit absent/special path uses food value ten.
- `E7CE40` then performs integer progress accounting and source transitions.
  Its decompiler's float parameter is misleading: SDK `E7CEA0` loads the
  raw stack word and `E7CEA4` performs an integer add. The procedure caps
  progress at 1000, tracks source-dependent counters and calls growth,
  tutorial and transition routines. Those side effects are not included
  in the numeric eating helper. Its integer ABI must be retained in later
  reconstruction.

`[MEASURED]` Some native boolean branches use `== 1`; bomb/nova use
comparison to zero. `CellBool` retains its exact byte and only offers
canonical 0/1 interpretation. It does not invent one truthiness rule for
every native consumer. All 16,137 source boolean bytes in this corpus
are zero or one.

`[UNKNOWN]` Native eating also updates progression, counters, event/audio
routes and source-dependent food channels. The numeric helper does not
apply those operations and is not wired into the experimental simulation.
Native movement, food selection, growth, damage, death/restart, editor
return and stage-transition acceptance remain open.

## Independent corpus and verification

`[MEASURED]` The metadata-driven Python pass scans all **82 package
indexes**. Three packages contain **489 definition resource copies**,
covering **298 distinct exact keys**. Every resource has 176 typed leaves,
for **86,064 field values**; 44,010 float fields are finite. This includes
base/patch/expansion copies and is distinct from the preview's 223-row
PatchData population and its 65 resolved baked-model rows.

The Rust verifier uses typed accessors, while the Python fixture derives
offsets/types recursively from retained native descriptor evidence.
Both canonicalize offset, native type, byte count and source-value bits.
All **489 resources / 86,064 leaves agree exactly**. The immutable raw
view also retains bytes omitted from typed leaves, including padding.

Fixture: `target/source-cell/definition-corpus-v4/corpus.bin`, SHA256
`f0a940e9c6a2a6baa994f5df3b2b21b19bd316920a43cc9e77649547ed168673`.
Its inventory records each source package, exact key, payload digest and
expected semantic digest. New parser tests cover bounded UTF-16,
surrogate pairs, independent AI variants, nonfinite float bits and
noncanonical boolean preservation. The existing legacy label tests pass
with a boundary regression. Eating adds a native cap/difficulty/early-exit
regression; selection adds inheritance and player-gate boundaries.

`[MEASURED]` Independent Python/native-byte rules and Rust helpers agree
for all 489 copies, covering **2,445 AI selections**, **4,890 AI gates**
and **34,230 numeric eating cases**. Rules TSV SHA256 is
`c1cd55dfac573cac64ab22fe2ea564cc227476ad593a746d0346b4d2221d8084`.
The verified v3 fixture and fully reproduced v4 fixture are byte-for-byte
equal, including this TSV. This is a bounded branch check, not execution
of an entire original Cell stage or completion of its effects/progression.

```powershell
python tools/cell_schema_probe.py 'D:\Games\SPORE Collection\SporeBin\SporeApp.exe' --dumpbin 'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64\dumpbin.exe' --out target/source-cell/schema-new
python tools/cell_definition_probe.py 'D:\Games\SPORE Collection' --metadata target/source-cell/schema-new/schema.json --out target/source-cell/definition-corpus-new
.\tools\build.bat test -p spore-core cell_definition -- --nocapture
.\tools\build.bat test -p spore-core string_table -- --nocapture
.\tools\build.bat run -p spore-core --example cell_definition_verify -- target/source-cell/definition-corpus-new/corpus.bin target/source-cell/definition-corpus-new/rules.tsv
```

`cell_schema_probe.py` pins the source executable digest, invokes four
bounded SDK disassemblies and extracts names, offsets and primitive/nested
types. It does not execute the game or copy default tuning into source code.
The 81 descriptor rows are retained in ignored
`target/source-cell/schema-v1/schema.json`, SHA256
`bb09a044956f5787e16a3bed5317018806d1bd89eb5dd40a7d1e7fdfa8e1177b`.
This reproduces the first SDK/REA field ledger independently of the Rust
parser's layout.

## Evidence ledger

| Finding | REA v3 query / retained SDK |
|---|---|
| Main named field offsets | 209 `1375C70`, `ev_dce498ddad2c03b934d35cc70d5cda55b58d362afdfa3efe82ec9c697173abb6` |
| Descriptor registration | 212 `E4A560`, `ev_2eb82d1982fd9f4a4c5bd726e4160f5d2e6db2a19f1941eaf8f6726755e9768d` |
| Name hash and identity wrapper | 223 `E4A750`, 224 `94E9D0` |
| Reference fixups | 239 `E4AAD0`, `ev_7fcdcac6e6e506c769d043ec7b9515ca61c8acb5a16453a10455b70fc7e01a41` |
| Primitive extents | 240 `E4A630`, `ev_b2a4211ceff82239695988adb6139f6fee416ca8bcafbb892f3001ac5bfab4a6` |
| Handle creation and acquisition | 241 `E82450`, 256 `E823D0`, 257 `E82200`, 258 `E822F0` |
| Actual binary resource loader | `native-cell-resource-fixup-sdk.asm`, start `E4AE20`; Ghidra entry undefined |
| Relation predicate | 292 `E572E0`, `ev_ade0d35957a547242753e9edfc649cd704439ab7e5677f543c85b56b6b54d07b` |
| Difficulty inheritance and AI predicate | 296 `E573E0`, `ev_9d8d4b8b46bb48dde02258fc54361e117e317624bbb1b3b51351a51a2df06661` |
| Relative size-class consumer | 297 `E571C0`, `ev_4befdc07da7091bcafeb9fc9a16a43f9a8edcb5f264e6e9f03b4c88d58cc37b9` |
| Escape flag read | 293 `E4FD70`, `ev_bb0eb45202f80dc11e2721d6039d377fc4ea94e50789d31ae5e905815398b242` |
| Size float consumer | 294 `E5D100`, `ev_dc8d8b503aa412434be59a484ac8627c6b4b3b5892496ea0441aa00bb6223e08` |
| Eating numeric consumer | 295 `E7D0A0`, `ev_a087b3ecd9eeb1cc0636f12c2f76929366b2e7863a6ce2b60c854a7052ca2291` |
| Integer progression and follow-up calls | 307 `E7CE40`, `ev_d9bb84d9785384a8b6b787511b974f72ff5e1f533bfc0e65067f3a1e374001e4`, `native-cell-progression-add-sdk.asm` |
| Progression-to-size-class table | 308 `E4ECC0`, `ev_adf81eb8ad8cd9d01fdda005592a9b237780b4a2d732edf8b8ddfd0945e6aa3d` |

No gameplay/editor milestone is completed by this parser or partial consumer.

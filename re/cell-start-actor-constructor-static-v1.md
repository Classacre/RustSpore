# Cell start candidate: constructor dependencies and source fields

Static trace of installed `D:\Games\SPORE Collection\SporeBin\SporeApp.exe`, SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. Constructor dossier is `target/rea/20261028-source-runtime-v6/824-analyze_function.json`, evidence `ev_72312fb7b745cd140fa1ddf023ed4a24a2106ba2f40c1b501d7c42da09b3d979`; other cited dossiers are from the same retained session. Assembly was checked for the call arguments and field writes. The installed records below are explicit candidates, not evidence of mounted winner, native fixup, successful player creation or live behavior. No game, Cargo or GPU code was run.

## Player caller and constructor inputs

The conditional route `E803D8 -> E750F0 -> E74A50` is described in [the start-definition trace](cell-player-start-definition-static-v1.md). `E803D3` passes the vector at executable global `015A8D3C`, whose bytes decode to `(100,100,0)`, and a scalar computed immediately before the call as current Cell global +514C times an earlier `xmm0` value. The source and branch that produced that earlier value remain to be traced; static placement alone is not an observed spawn.

At the `E751C0` call, stack arguments to `E74A50` in order are: current global +40FC word; supplied position pointer; float zero; `E4CCA0` result +34 reference handle; signed -1; supplied scalar; float zero; raw word 1; null pointer; raw word 0. The decompiler's argument types are not a substitute for that calling convention; argument 8's low byte is tested, and the two final zero words do not establish their semantic names. The returned handle becomes current player word +411C, then is resolved through the actor pool. The configuration object, `startCell` fixup, active provider and input global values must be valid at that instant.

The constructor resolves argument 4 using `E4CAA0`, samples definition +304/+308 through `4DF9A0` when argument 7 compares equal to zero, and combines the sampled result with the supplied scalar and current global +514C. Even equal size endpoints do not prove the RNG call is skipped. It allocates through actor pool `B71C50`, immediately projects with unchecked `B71D00`, and copies the supplied position to actor +08..+10. The checked model must reject missing/invalid resolution, pool failure, zero-handle/generation ambiguity and unsupported arithmetic before committing; those checks are safe additions, not native branches.

## Explicit installed candidate

A fresh read-only flag-driven package-index/RefPack decode of `Data/PatchData.package` gives the following exact entries and hashes. These correspond to explicit package copies; no retail provider winner is asserted.

| Canonical key (instance:type:scope) | Decoded length | SHA-256 |
|---|---:|---|
| `CBCD1287:DFAD9F51:00000000` | 796 | `f12d98c0f59af6d4d58174016f1e04d709647480310103341193df9b0fa76542` |
| `CBCD1287:4B9EF6DC:00000000` | 108 | `a01b950a54889f86429f4e847b7aa7e1ff8d5e354027b017eb80ee8e07a6da87` |
| `9A391385:F9C3D770:00000000` | 120 | `29b0f195b26bf2480586518e385933284819ff360a991ea706a939717a5d5f8a` |

The CELL raw bytes have structure reference ID `CBCD1287`; `fixedOrientation` byte +AC = 0, flags +B0 = 0, `cellType` +B4 = 2, `density` +BC = 0, size floats +304/+308 both bits `3E8CCCCD` (approximately 0.275), and `hp` +A8 = 3. Base AI first word +E0 is zero; easy +248 and hard +194 first words are `FFFFFFFF`, so the documented AI-variant rule inherits base for those cases. The base AI +48 float is zero. `eat` +30C/+310 is signed `(10,1)` and bomb/nova bytes +314/+315 are zero. These are decoded raw fields; hp +A8 is not interchangeable with actor +244, which this constructor initializes to six.

The explicit structure record has two 40-byte rows after its 28-byte header: first kind 4 with raw +0C `9A391385`, then kind 1. `E656C0` resolves the definition's structure reference and visits entries in order; its first kind-4 branch calls `E65640` using the structure handle and that row's +0C word (dossier 841, `ev_0b4d6736a17804361c050cd2a265a9d65a483b78fec08680313511ef0194a161`). The 120-byte candidate list named by the raw ID has four 28-byte rows after an eight-byte header; each raw row starts with word zero, followed by `A1C46DF2`, `E219E5D0`, `E219E5D3`, or `E219E5D2` and weight bits `3F800000`. The independently decoded `Data/Spore_Game.package` and `DataEP1/Spore_EP1_Data.package` indexes also contain that canonical list key; their payload equivalence was not checked here. The type-3 handle fixup, source provider, cache and identity of the ultimately selected row remain separate gates.

`E5C0F0` (dossier 849, `ev_2e1748bb9ebe415cfdeed37aaf25dbf2724e740a126782d216d69c6baf967e6f`) iterates a resolved list, accumulates row weights and calls RNG during selection; the resulting type-zero row returns its +4 word with tag `3D97A8E4` and a global word. The cached `E65640` path can affect whether selection runs. Do not replace this with a fixed first row, a single unverified RNG draw, or claim any candidate is the live player. This triple is written to actor +FC/+100/+104, then may be modified by the player caller according to save state; it is used downstream by `E67A50` and `E6D7F0`.

## Source-state needed before a real update

For this supplied-candidate branch, constructor and its helpers depend on more than the CELL bytes:

- a validated definition handle to the selected CELL object and its descriptor-fixed structure reference; a selected current globals object including +34 and the fields consumed by `E4F830`, with provider/cache ownership and lifetime;
- the ten call arguments above, current +40FC and +4100, +514C scale, the actor pool's generation/free-list/capacity state, and checked validity of all returned handles and pointers;
- source RNG sequence and x87/SSE environment for size sampling, orientation (the candidate's fixed-orientation byte is zero and this caller supplies a null orientation pointer), `E6CB30` when reached, and the final AI timer calculation; constructor call ordering must be retained;
- source orientation generation and `59C000` matrix; actor +4C..+54, +58, +48, +5C..+7C, and +FC..+104 for `E6D7F0`; actor +108 reference handle and +244 integer health;
- for this candidate, admitted kind-4 list/selection and any saved-player triple overriding +FC..+104, followed by the actual `E67A50` cache result and its lifetime; the first raw list word is not itself a ready runtime pointer;
- `E6CB30`'s `E69710`/`E697B0` results and RNG/state, `E86960` insertion and actor +360 ownership, the +4104 collection's registration branch and +364 result, and the update/removal lifetimes. Dossier 842 `ev_c52d68f7b30549951d5de338239ae9ac50b5fc1a0fbe9b9dd0aa29269c0e6af6` and dossier 843 `ev_52507a4a561fa84f1de64d613c8d59a5c4dd5eaf53422cee69934fd9c6aabcc9` describe those helper boundaries.

At `E74D12`, `E4F830` returns a density/type-dependent scalar in XMM0; the caller multiplies it by computed scale twice to write actor +B8 and writes its reciprocal to +BC. For candidate density 0 and cellType 2, the jump tables choose the current globals object's +5C float. The earlier loaded literal 10 is therefore **not** evidence that +B8 always uses a factor of 10. Other helpers can mutate referenced state, and the observed input arithmetic does not prove every side effect or exact floating-point result.

A minimal fixture can explicitly supply these dependencies, create the actor in the real source-update architecture and stop at unsupported calls; it must label the selected package, candidate list row and RNG as fixture inputs. A runnable player requires the native provider/fixup/selection, spawn admission, RNG and cache, actor and collection lifetimes, contact/event and renderer/input paths, and original-source-bound validation. Neither this note nor the existing inspection loader proves those gates.

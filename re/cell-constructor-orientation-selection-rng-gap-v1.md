# Cell constructor RNG gap after null orientation

Static x86 disassembly of installed `SporeApp.exe` (identity pinned in `cell-start-null-orientation-static-v1.md`: SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`). Preferred-base virtual addresses below. Scope is the conditional `E74A50` path after its null-orientation draw and up to the first kind-4 selection draw; no original code was executed. This is not a closed transitive call graph and does not establish retail RNG continuity.

## Instruction order

`E74B98` loads global RNG object `01602760` and `E74B9D` calls `935D40` for null orientation. The constructor then performs the trigonometric and direct actor writes documented in the orientation note. Before `E74E67` calls `E656C0`, its only direct calls are:

1. `E74C31 -> 59C000`, quaternion-to-matrix; `59C14B -> 41CAE0` copies nine components. Within the inspected function bounds `59C000..59C156` and `41CAE0..41CBB5`, there are no other calls and no explicit RNG-global references. This observation assumes valid, non-aliasing actor/output pointers; arbitrary writes or faults are not modeled.
2. Conditional `E74C6B -> E5E960` when the source comparison at `E74C5D..E74C63` is equal. The complete `E5E960..E5EA43` body is arithmetic and stores and has no calls or explicit RNG-global references. The alternative branch copies three global words directly. Both remain subject to valid non-aliasing input/output admissions.
3. `E74D12 -> E4F830` always on the inspected route. It initializes two wrappers using `7B2350`, resolves supplied and global references via `E4CAA0` and `E4CCA0`, reads fields and releases wrappers with `E82160`. Those wrappers matter for RNG continuity: their resolver is not a proven pure read.

`E656C0` itself initializes and resolves the definition and structure wrappers (`E656C9..E656F3`) before walking the structure rows and calling `E65640` for kind 4 at `E6576F`. `E65640` executes cache lookup and, on a miss, obtains/inserts the entry (`E6565D..E65688`) before `E5C0F0`. On a miss `E5C0F0` initializes and resolves the list wrapper (`E5C100..E5C10E`) before the first row call to `935D40` at `E5C14E`. A hit does not enter that selector; do not infer a selection draw from the orientation draw alone.

## Resolver and cache frontiers

`E4CAA0` is a jump to `E823D0`; `E4CCA0` supplies a global handle and calls the same resolver. At `E823D0..E82404`, the resolver increments a reference counter, calls `E822F0` if the resource pointer is null, and, when nonnull, calls the resource's virtual slot +0C at `E823FF`. The slot target is runtime-dependent. The cold `E822F0` path calls `E82200`, can dispatch further virtual calls and fallback work, and conditionally calls `E82170` (`E82315`, `E82359`); its resource/provider callbacks have not been bounded here. Even a hot pointer therefore still reaches a virtual call. `7B2350` initializes a wrapper without a call; `E82160` decrements its pointed counter without a call (`E82160..E82169`). This does not make the whole wrapper lifecycle pure or resolve callback identities.

The cache lookup `810230..810273` uses only a tree walk with no direct calls. On a miss, `E650B0` can invoke `E63D10`; this insertion path reaches `E61710` and allocation `F472B0` (`E61752`), plus tree-maintenance helpers. Their entire transitive behavior and allocator hooks were not proved here. A stable pre-existing cache hit avoids insertion and selector RNG, but the preceding `E4F830` and `E656C0` resolvers still execute. No direct `935D40` call was observed in the constructor interval or the explicitly bounded leaf routines above; this is not a whole-program search, nor proof that indirect targets, allocator paths, asynchronous mutation or aliasing cannot change `01602760`.

## Practical admission

For a bounded size → orientation → selection composition, treat the selector entry RNG state as an explicit supplied snapshot until the actual resolver targets, resource state and cache insertion/allocator transitive paths are constrained and audited. Alternatively, admit as a test precondition that all intervening resolved targets and mutations preserve the RNG word and global object's identity, then check that assumption in a separately guarded native fixture; that admission is not proof of the real runtime. Record whether cache hit or miss, the state at each boundary, pointer/handle and lifetime assumptions, and any callback side effects. Static source evidence here does not demonstrate an intervening RNG draw or prove its absence on all paths.

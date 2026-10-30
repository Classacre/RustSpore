# Cell actor geometry consumer and capability-cache boundary

Static Capstone x86 trace of installed `SporeApp.exe` SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. This follows [the source actor sweep plan](cell-source-actor-sweep-integration-plan-v1.md) and [capability-cache evidence](cell-capability-cache.md). It is not a live actor, provider selection or retail contact observation. No original game, Cargo, GPU or native snippet was executed in this pass.

## Actor triple and cache acquisition

`E6D7F0` takes the actor in ESI, a three-word output at EDI and a scalar output pointer as its stack argument. If actor +FC is zero, `E6D7FE..E6D815` copies +4C/+50/+54 to the vector and copies +58 through x87 FLD/FSTP to the scalar. It returns without looking at +100/+104, +48 or the matrix. Do not transform this branch a second time.

When +FC is nonzero, `E6D816..E6D835` copies actor +FC/+100/+104 in that order into a 12-byte stack argument and calls `E67A50`. The words correspond to the candidate triple created by the separate `E656C0` path, subject to later writes; no particular candidate or native pointer is inferred. `E67A50` searches the cache at Cell global `[016B4BE4]+70`. Both search `E39BC0` and value access/insertion `E679B0` compare **only the first dword** of the passed triple, using signed comparisons against the node key at +10. On hit it returns the cached value without passing the other two words to the population calls. Thus an admitted stable hit reuses the first-key value even when +100/+104 differ; this is a lookup observation, not proof that differing triples are valid or coexist in retail.

On a miss, `E67A96..E67AB6` obtains/inserts the cache value first with `E679B0`, calls `E83E60` with addresses of the cache value's +00 count and +04 records and the full input triple, then calls `E51EC0` with ESI set to the cache value, EAX loaded from its count and arguments containing the +04 records pointer and zero. It returns the cache pointer. The cache insertion happens before the helper calls; allocation failures, partial initialization, reentrancy, pointer lifetime, invalidation, mutation and helper side effects are not resolved. A fixture should treat hit/miss and caller-supplied snapshots as separate explicit admissions and stop on unsupported population rather than inventing a cache result.

## Creation/resource boundary

For the non-special `E83E60` path, the entire input triple is passed to `4BC760` (`E83EE2..E83EEF`). Its inspected start constructs a request whose first word is triple +00, type word is `0F43029A`, and scope/third word comes from triple +08. It branches on bits 24..28 of that third word and may alter a constructed key on one route (`4BC769..4BC7FF`); the complete provider and transformed-key behavior must be followed before asserting which resource is loaded. `E83E60` uses the returned object's +98/+9C range to determine rows and limits processing to 64; per-row classification is supplied by `E827A0`. If input triple +04 equals `476A98C7`, `E83E60` instead writes the observed single-row special case using globals and does not follow that ordinary route. The complete model, resource identity, mounted winner and classifier bindings are separate from the selected three words.

The repository's `cell_part_geometry.rs` and `cell_capability_geometry.rs` already model bounded `E83E60` and the skip-flag geometry/count/index reduction of `E51EC0`; [the capability note](cell-capability-cache.md) reports prior isolated original-byte comparisons over supplied classes and arithmetic contexts. That evidence does not establish that this `E67A50` miss has actually supplied a particular source creation, classifier result, live cache or player. In this call `E51EC0` receives zero as its explicit final argument, so the cited skip-flag-one reduction comparison alone cannot prove the full miss path; later scalar/curve/global and source binding admissions remain necessary. The miss can also call providers, allocate and populate shared structures, so extracting just model bounds is not equivalent.

## Exact geometry projection

For a valid nonzero +FC cache result, `E6D83A..E6D865` writes scalar `M(cache[+1244], actor[+58])` using one `MULSS`, then copies cache +1238/+123C/+1240 to a local output vector `(cx,cy,cz)`. `E51EC0` initializes those cache fields from mutable globals and derives geometry from supplied rows; their actual contents cannot be inferred from the actor triple alone.

Let `m0..m8` be the nine actor words at +5C through +7C in physical order. Let `M(a,b)` and `A(a,b)` denote one separately rounded source `MULSS` or `ADDSS`. If actor byte +48 has bit `02`, `E6D872..E6D8F1` writes:

```text
x = A(A(M(m3,cy), M(m6,cz)), M(cx,m0))
y = A(A(M(m1,cx), M(m4,cy)), M(m7,cz))
z = A(A(M(m2,cx), M(m5,cy)), M(m8,cz))
```

This establishes the indices consumed for each output without naming a row/column convention. If bit 02 is clear, the copied cache center is unchanged at this stage. `E6D8F6..E6D919` multiplies each component by actor +58, then `E6D91E..E6D942` adds actor +4C/+50/+54 as the **first** operand for x/y/z respectively. SSE operation ordering, signed zero, nonfinite values and MXCSR matter. The source does not set MXCSR in this body. The zero-+FC branch is a distinct direct-copy path and does not use these multiplications or additions.

`E7F290` later resolves each stored pair word with checked `B71CC0`, calls this geometry routine for both actors, computes separately rounded squared center distance and squared sum of scalars, then calls `E7F170` only when the squared sum is strictly greater (`COMISS`/`JBE` at `E7F37E..E7F381`). This is one broadphase gate. It does not prove feature-level contact, selector arbitration, event creation or feeding.

## Smallest actionable gate

A standalone pure `E6D7F0` projection can take explicit actor fields and an explicitly admitted fresh cache snapshot; implement the zero branch and nonzero branch with the exact operand order above under a named scoped MXCSR, reject unmodelled nonfinite/alias conditions, and test both bit-02 paths, signed zeros, boundary rounding and scalar output independently. It should not call `E67A50` or fabricate its snapshot. A separate cache fixture can model signed first-word hit/miss and insertion order with explicit post-`E83E60`/`E51EC0` snapshots, retaining pending state where side effects or lifetimes are unsupported. To integrate actual contact, source initialization still needs canonical source/resource resolution, creation/classifier binding, zero-flag `E51EC0` dependencies, cache ownership and update lifetime, actor-pool handles, pair production and ordered callback paths. The source geometry projection alone is not an active Cell simulation.

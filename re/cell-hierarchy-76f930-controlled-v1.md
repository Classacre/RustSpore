# Controlled 76F930 hierarchy comparison

This records a bounded static audit and an eight-case controlled comparison of source function `76F930` with the Rust example. It does not establish a live Cell actor, model or clip selection, complete retargeting, or an original animation pose. The preceding lookup and caller observations and their limitations are in [the RW4 binding lookup follow-up](cell-rw4-binding-lookup-followup-v1.md).

## Source and linked-host audit

The original `SporeBin/SporeApp.exe` SHA-256 is `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. Independently read PE bytes for `76F930..76F99B` inclusive are 108 bytes, SHA-256 `81fc0c1de942b4c378cf5b24662f2b5b7104e577ca6c98108e8ba46a403bc0f6`. Static x86 disassembly shows only relative branches within this slice, no external calls, absolute address operands or memory writes. The cdecl function receives `(A, B, &map_descriptor)` and returns its boolean in AL; upper EAX is not specified.

The generated `target/cell-hierarchy-76f930-v1/host.c` SHA-256 is `1604f08390109e71233e2594315e0dababd0c20fa05ee9cb434058c21849b9ae`, matching its preparation manifest and the current preparer template with the independently extracted bytes substituted. Its embedded byte array matches the full original slice exactly. The compiled `host.exe` SHA-256 is `04987e5888a7d2365c7f7fe8b22ce07303b85993191454729bef414ca94328aa`; it is x86 PE32 and contains exactly one copy of that slice at file offset `0x14170` (VA `415170` at its preferred image base). All eight linked fixture records match the source C initializers.

Static linked-code inspection shows `VirtualAlloc` with writable, non-executable protection, a 108-byte copy, `VirtualProtect` to execute/read, and `FlushInstructionCache`, followed by a three-argument indirect call and AL comparison. The caller checks its input snapshots for mutation and runs eight fixtures. No static PE section is writable and executable. Imported KERNEL32 routines include the expected memory APIs and CRT support; the static import table alone does not prove absence of dynamic loading. This audit did not execute the host. The preparation manifest's `built=false` and `executed=false` describe preparation time and are not a current execution-status report.

## Recorded comparison

The coordinating agent's `target/cell-hierarchy-76f930-v1/comparison.json` reports eight native/Rust matches. The saved native and Rust output files each contain these ordered values:

```text
0 0 1 1 0 0 0 1
```

Both output files have SHA-256 `44bc27c8e1ee1ecacfb2d98253ddc32b490a93c125668addd2ab1e70f12f976f`. The comparison records Rust binary SHA-256 `c0a65e586dea15f3a010169e9c6b54dcffcc0a4f02329d809fdd0ef8665fe679`; this note relies on that report for the Rust binary identity. Comparison JSON SHA-256 is `afac7291de859d6f15208656eaab86be228ca6e51acc52448b8d821c928ca9ba`. Matching these fixed fixtures does not establish exhaustive equivalence.

A's +0C is a signed count and A+4 points to its i32 parent array; B+4 points to its i32 parent array; the descriptor's first word points to the i32 mapping array. For each mapped A node, the function checks whether its mapped nearest mapped ancestor equals the B parent of its mapped node; a mapped A root requires a B root. An unmapped current node is skipped. Zero count or no mapped entries returns false.

The Rust candidate admits equal A and map lengths, at most 4,096 nodes in each array, A parents of -1 or an earlier ordinal, B parents of -1 or valid ordinal with no cycles, and map entries of -1 or valid B ordinal. These are conservative API safety restrictions, not proven native data limits. Native ancestor skipping specifically tests -1; the candidate rejects other negative map entries. The controlled fixtures are synthetic and cover only eight selected cases. The native host uses ordinary stack allocations without guard pages or read guards; snapshot comparisons detect changes within the copied objects but do not detect out-of-bounds reads or all writes outside those objects. Invalid-input Rust tests do not establish safe native execution on invalid pointers or graphs.

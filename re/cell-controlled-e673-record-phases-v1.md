# Controlled E673A0 record phases

The inspection-only source-player module now includes a bounded model of
individual E673A0 phases, not a native container or active collision system.
The source evidence is [the continuation boundary](cell-e673a0-continuation-boundary-v1.md)
and [the record trace](cell-feeding-e673a0-producer-side-effect-v1.md).

The point/bounds phase requires finite, valid caller-supplied bounds and point,
checks ordered lower/upper comparisons under scoped MXCSR 1F80, and returns
outside or a pending provider operation. The opt-in owned fixture applies this
phase to its pending E673A0 data using synthetic bounds `[0,200]` and stops
before any provider call. A separate `after_provider` helper accepts explicitly
admitted fresh +6C value, returns pending cleanup when +6C exceeds the
finite distance, or pending lookup with the low-16-bit packed key. It does not
construct a holder or resolve a package.

A separate checked lower-bound helper models the decoded E616C0 signed-key
ordering for a caller-supplied strictly sorted key slice; rejecting duplicates
and limiting length are added fixture policies. It does not model E65810
allocation, comparator mutations, vector movement or native ownership. Matching
packed keys deliberately does not assert full-ID equality.

For an explicitly admitted existing entry and first lookup, `begin_existing`
updates only the named +10 current word and returns a one-use ticket for the
second lookup. `finish_existing` requires the same owned entry identity/key and
point identity and uses freshly supplied point words; a rejected second lookup
does not undo the earlier +10 write. On an explicitly admitted fresh slot,
`new_entry` constructs only the named source-written fields, including ordered
full IDs, equal creation/current words, strict impact flag against a separately
supplied fresh +70 value, freshly reread point words under the same admitted
point identity, and zero
handles. These are typed fixture snapshots, not native byte layouts or actual
allocation. Source provider, holder cleanup, E65810, callbacks, subsequent
E59CD0, audio/event routing and gameplay are not executed. Stable disjoint
memory and no intervening mutation are explicit admissions where a helper
completes a phase. A later isolated original-instruction comparison is described
below; it does not validate a contiguous E673A0 invocation.

## Reviewed isolated-slice comparison

The reviewed host ran separate source slices `E616C0..E6170C` (signed lower
bound), `E674DD..E674DF` and `E674FE..E6750C` (existing-record stores), and
`E67524..E6756D` (new-record stores). All six lookup results, one complete
existing record and four complete new records matched the pinned Rust adapter;
both processes exited zero. The exact fragment and host pins are in
[the preparation note](cell-e673-record-comparison-preparation-v1.md).

The signed sorted keys were `80000000, 00010002, 7FFFFFFF`. Queries
`80000000, 80000001, 00010002, 00010003, 7FFFFFFF, FFFFFFFF` produced
indices `0, miss, 1, miss, 2, miss`. The existing record kept its colliding
full IDs `00090001/000A0002`, creation word 5, impact byte 1 and handles
8/9 while its current word became 42 and its freshly supplied point became
`4,5,6`. The two source store slices received the same explicitly admitted
owned entry; neither E65810 lookup was executed.

The four new-record cases used distance/threshold values 5/5, 5/4, 5/6 and
-0/+0; impact bytes were respectively 0, 1, 0 and 0. One case used distinct
full IDs `00090001/000A0002` with the same packed key `00010002`. The full
0x2C-byte comparisons also included initially sentinel padding and zeroed
handle words. All four captured MXCSR values were `1F80`; control bits were
checked and sticky flags were not separately compared.

The compared model source SHA-256 is
`c1749a45b37883fab6117c5debac027706e2edbee87f458c18f7fe6f6d13d271`.
Receipt SHA-256
`1307dafddcadb71d0f0286be88d8224696e9a1f23a51d78e74ccb20b046de653`;
native stdout SHA-256
`9178a7ce222bb378b174327788570960b2d707c539d4d73959b5baf6e08710df`;
model stdout SHA-256
`ef13ff0d86a58d23b7376a39b1b4cefbcc68504d9744001f644abd7e7baa2f90`.
The run is retained under `target/cell-e673-record-run-v1/`. This result covers
only the named slices and fixed inputs. In particular, no provider, E65810,
allocation, callbacks, concurrent mutations, bounds gate or complete source
call ran; stable entry and fresh point identity were fixture admissions. Hash
checks cannot exclude transient replacement, and no live contact follows.

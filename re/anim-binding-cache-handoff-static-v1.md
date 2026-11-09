# Animation cache to instance handoff: static boundary

2026-10-30. The highest-impact remaining binding is the real actor's resource,
owner and binding association into `009A3140`, followed by nonzero-child pose
evaluation and palette delivery. The full ungrouped builder already matches
1,085 prepared cases, but the five-body cohort and preallocated capacity are
not evidence that a live actor has selected or installed those records. The
Cell candidate bridge is likewise caller-selected. See
`anim-ungrouped-binding-builder-native.md` and
`cell-anim-candidate-bridge-v1.md`.

## New inspection of the saved instructions

The base executable was re-hashed this session as
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The following are static observations from retained Ghidra instruction lists,
not new native execution or source-equivalent implementation:

* `009A4430` calls `009A37D0` first. On lookup failure it allocates a `0x38`
  object, invokes `0099EDC0`, and calls `009A1E00` at `009A44B2`. A zero
  builder return releases a non-null result via `0099EDE0` at `009A44CA`.
  Hence the prepared builder's output is not by itself a cache hit, allocation,
  lifetime, or failure model.
* `009A3140` retains a changed resource by incrementing its `+114` before
  releasing the previous one through `0099CD90`. It similarly calls `A16FC0`
  on a changed non-null owner before storing it and calls `9C4B50` on the old
  owner. Equal pointers skip those changes. These orderings matter for a
  replacement and self-assignment fixture.
* With a supplied non-null binding argument, `009A3140` increments its `+30`
  and replaces instance `+8`, releasing its previous binding with `0099EDE0`.
  With a null binding argument it instead reuses instance `+8` or allocates a
  zero-initialized `0x38` object, then calls `009A1E00` at `009A3371`. Thus
  even an instance path has a builder fallback; neither a universal cache hit
  nor treating null as an empty successful binding captures the source path.
* `009A3198..31BF` calculates `(resource+120 - 1.0f) * resource+11C` in two
  SSE single-precision operations, stores it as float, then widens it to
  double at instance `+B8`. `009A32D6..32E1` transfers bit 3 of resource+118
  to instance byte `+AD`. The conditional zero duration for a null resource
  does not prove a complete null-resource path: `009A31C5` subsequently reads
  `[EAX+144]`; admission requires a separately established call domain.
* `A00B30` passes the cache result from `009A4430` directly into `009A3140`
  at `A00BFE..A00C11`. This is an observed fallback-route callsite, not proof
  of which installed resource or actor reached it in live gameplay.

## Next reproducible gate

Trace the actual production and identities of the resource, owner and supplied
binding through `A0C120 -> A01A20 -> 9FFA20/A00B30`, including the two lookup
trees. Construct a bounded fixture that separately exercises cache hit, cache
miss plus successful builder, builder failure, supplied-binding replacement,
null-binding fallback and pointer self-assignment. Preserve original refcount
call order and source single-precision duration arithmetic; establish vector
growth (`009A0980`) for the actual fresh cache instead of importing the
prepared 7,140-record capacity. Independently review source bytes, host and
linked behavior before any native execution. Then join actual sampled channels,
nonzero-child stage/commit and palette mapping; a successful binding fixture
alone is not visible motion or 1:1 animation.

The retained source dossiers are ignored local evidence and must be preserved:

| Dossier under `target/rea/20261027-source-runtime-v4/` | SHA-256 |
| --- | --- |
| `38-analyze_function.json` (`009A3140`) | `d69c43d246b22b03ee0ccff8b11d14372cd2988380f1c5b09a10f24d4d3ea3f2` |
| `39-analyze_function.json` (`009A4430`) | `30d39802d2e228e19eb92675e45d8617677d053ad6c6fa3fd7a74e08056ca282` |
| `48-analyze_function.json` (`A00B30`) | `b1889a3a829585cf64e7d7b895b97aa0db553616e3cc9bf373601c459c77deea` |

No Cargo, GPU, GUI or native host was run for this note. The instructions
alone do not establish all callers, allocator failure semantics, source
resource serialization mapping, or dynamic actor behavior.

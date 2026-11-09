# Cell current-player handle: creation and lifecycle stores

Static x86 PE analysis of `D:\Games\SPORE Collection\SporeBin\SporeApp.exe`, SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. This extends [the start-definition chain](cell-player-start-definition-static-v1.md), [constructor trace](cell-start-actor-constructor-static-v1.md), and [pool trace](cell-actor-pool-definition-handle-static-v1.md). It does not observe a player, resource winner, callback execution or game session. Pointers, memory identity, valid callees, stack discipline and normal returns are conditions on every join below.

## Startup and actor-pool identity

On the conditional startup path, `E803D3..E803D8` passes position pointer `015A8D3C` and an SSE-derived scalar to `E750F0`. The latter obtains the globals object through `E4CCA0`, reads its +34 word and passes that word as the fourth argument to `E74A50` (`E751A8..E751C0`). The value is a descriptor-fixed definition reference handle on the documented type-3 loading path, not directly the raw package ID. The explicit installed-data fixture has candidate `globals` and Cell definition bytes described in the linked start-definition note; provider priority, reference fixup, lifetime and actual choice are unobserved.

`E74A50` calls `E4CAA0` on that definition argument (`E74A60..6A`). At `E74AF8..E74B14` it calls `B71C50` on `[016B4BE4]+1C`, saves the returned word, and passes the same word to `B71D00` to obtain a slot pointer. On the normal path, it later reloads the saved handle into EDI at `E74F67`, uses it in the `E86960` call, and returns it in EAX at `E750DF`. This relation depends on valid stack/callee conventions and no intervening corruption. `E751C5..CB` reloads the global owner and writes that return word to owner `+411C`; `E751D1..DE` then uses `B71D00` on it. There is no visible allocation-zero check before that unchecked pool projection. Thus a successful valid allocation and stable owner are required; an allocation failure cannot be promoted into a valid player identity.

`B71C50` increments the pool counter and composes `(counter << 16) | index`, writes the resulting word as the allocated slot's first dword and returns it. The high portion is the low 16 bits of the incremented counter after shifting; the index is ORed without an explicit mask. `B71CC0` rejects zero, masks the low 16 bits as an index, checks against pool +10 using signed `JG` (equal reaches a read), and accepts only if the slot's first dword equals the complete supplied word. `B71D00` instead uses only the low 16 bits and stride to form a pointer, with no matching or bounds check. These instructions are not a permanent-uniqueness guarantee: pool identity, capacity, high-water semantics, generation wrap, free-list state and slot lifetime remain material. A checked Rust bridge must not silently treat the source's unchecked `B71D00` as verified lookup.

The `02C21781` / `cPlayer` provider registration and `B1E880 -> C7B440` object documented in [the separate simulator factory note](cell-start-player-factory-static-v1.md) describe a different factory candidate. The cited startup actor path above goes through `E74A50` and the Cell actor pool; no instruction in these slices proves the simulator `cPlayer` object is that pool slot, is the +411C word, or selects the installed Cell candidate.

## Other observed stores

A bounded search of the `.text` bytes for little-endian displacement `1C 41 00 00`, followed by local x86 decode, identified the following writes to a `+411C` operand. Their global-owner identity is confirmed in the cited local windows; this scan is not a complete writer census (computed addresses, aliases, other encodings, other modules and dynamically generated code are not covered).

| Store | Conditional local meaning |
|---|---|
| `E751CB` | Writes `E74A50`'s returned EAX to reloaded Cell global owner as described above. |
| `E780FC` | `E780D0` first asks `B71CC0` to resolve its supplied handle; if successful and equal to current global +411C, it stores zero before later cleanup. Subsequent effects and other callbacks remain separate. |
| `E7D8E8` | In `E7D8B0`, the first word of the second supplied object must equal current +411C and global `+5190` object's byte +E8 must be nonzero; the first word of the first supplied object is then stored to +411C. This is a conditional transfer, not proof either object is live or a selected player. |
| `E7F9EB` | `E7F9E0` zeroes EBX and stores it to +411C before its later cleanup work. |
| `E7FD6B` | `E7FD30` zeroes EBX at `E7FD5D` and writes it to +411C before the later branch containing the documented startup route. |
| `E7FE3B` | During the `E7FD30` pool traversal, after a nonzero `B71CC0` result and a current-handle equality comparison, it writes EBX, which was zeroed at `E7FD5D` under ordinary callee-save behavior. Calls and global state can change between the earlier clear and this comparison; it is not proof the branch runs. |

Several consumers read or compare the word, including `E5B160`'s checked-style `B71CC0` target-input route and `E7A329`'s actor-ID comparison before a conditional call to `E6F620`. Those are source control-flow connections, not proof that the startup slot remains current after a callback. The controlled `--source-player` inspection derives candidate source fields and resource projections; it does not populate this native pool, perform these stores, or establish a live handle. Re-read and validate the actual actor/pool identity after any intervening mutation or callback before composing a player tick.

## Reproduction

Hash the named PE, then use `pefile` and x86-32 Capstone at instruction boundaries for `E74A50..E74B25`, `E74F10..E750E9`, `E75180..E751F0`, `E780D0..E78125`, `E7D8B0..E7D902`, `E7F9E0..E7FA18`, `E7FD30..E7FD80`, `E7FDD0..E7FE60`, `B71C50..B71D12`, and `E80328..E803E5`. Use `get_data(start - ImageBase, end - start)` and follow branch targets; the listed endpoints may cut a subsequent instruction. For the limited displacement scan, search `.text` for bytes `1c 41 00 00` and decode candidate instruction starts around each hit, treating results as leads until local dataflow is checked. No Rust edits, Cargo, GUI, original game execution or runtime capture were used.
